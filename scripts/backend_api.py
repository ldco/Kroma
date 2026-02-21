#!/usr/bin/env python3
import argparse
import json
import os
import sys
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlparse
from agent_dispatch import dispatch_instruction_http

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

import backend as be  # noqa: E402


def parse_bool(value, default=False):
    if value is None:
        return default
    if isinstance(value, bool):
        return value
    raw = str(value).strip().lower()
    if raw in {"1", "true", "yes", "on"}:
        return True
    if raw in {"0", "false", "no", "off"}:
        return False
    return default


class ApiServer(ThreadingHTTPServer):
    def __init__(self, server_address, handler_cls, repo_root: Path, db_path: Path, cors_origin: str):
        super().__init__(server_address, handler_cls)
        self.repo_root = repo_root
        self.db_path = db_path
        self.cors_origin = cors_origin


class Handler(BaseHTTPRequestHandler):
    server: ApiServer

    def log_message(self, fmt, *args):
        # Keep logs concise for CLI use.
        print(f"[backend-api] {self.address_string()} {self.command} {self.path} - {fmt % args}")

    def _set_headers(self, status=HTTPStatus.OK):
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Access-Control-Allow-Origin", self.server.cors_origin)
        self.send_header("Access-Control-Allow-Methods", "GET,POST,PUT,DELETE,OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        self.end_headers()

    def _json(self, payload, status=HTTPStatus.OK):
        self._set_headers(status=status)
        self.wfile.write(json.dumps(payload, ensure_ascii=False).encode("utf-8"))

    def _error(self, status: HTTPStatus, message: str):
        self._json({"ok": False, "error": message}, status=status)

    def _read_json_body(self):
        length_raw = self.headers.get("Content-Length", "0")
        try:
            length = int(length_raw)
        except Exception:
            length = 0
        if length <= 0:
            return {}
        raw = self.rfile.read(length)
        if not raw:
            return {}
        try:
            return json.loads(raw.decode("utf-8"))
        except Exception:
            raise ValueError("Invalid JSON body")

    def _db(self):
        conn = be.connect_db(self.server.db_path)
        be.init_schema(conn)
        return conn

    def _project_or_404(self, conn, project_slug):
        project = be.get_project(conn, "", project_slug)
        if not project:
            self._error(HTTPStatus.NOT_FOUND, "Project not found")
            return None
        return project

    def _session_or_404(self, conn, project_id, session_id):
        row = conn.execute(
            "SELECT * FROM chat_sessions WHERE id = ? AND project_id = ?",
            (session_id, project_id),
        ).fetchone()
        if not row:
            self._error(HTTPStatus.NOT_FOUND, "Chat session not found")
            return None
        return row

    def _instruction_or_404(self, conn, project_id, instruction_id):
        row = conn.execute(
            "SELECT * FROM agent_instructions WHERE id = ? AND project_id = ?",
            (instruction_id, project_id),
        ).fetchone()
        if not row:
            self._error(HTTPStatus.NOT_FOUND, "Instruction not found")
            return None
        return row

    def _voice_request_or_404(self, conn, project_id, request_id):
        row = conn.execute(
            "SELECT * FROM voice_requests WHERE id = ? AND project_id = ?",
            (request_id, project_id),
        ).fetchone()
        if not row:
            self._error(HTTPStatus.NOT_FOUND, "Voice request not found")
            return None
        return row

    def _ensure_actor_user(self, conn, username):
        safe_username = str(username or "local").strip() or "local"
        display = "Local User" if safe_username == "local" else safe_username
        return be.ensure_user(conn, safe_username, display, None)

    def _emit_instruction_event(self, conn, instruction_id, event_type, payload):
        conn.execute(
            """
            INSERT INTO agent_instruction_events (id, instruction_id, event_type, event_payload_json, created_at)
            VALUES (?, ?, ?, ?, ?)
            """,
            (be.uid(), instruction_id, event_type, be.to_json(payload or {}), be.now_iso()),
        )

    def _dispatch_instruction_if_configured(self, conn, instruction_row):
        target_url = (os.environ.get("IAT_AGENT_API_URL") or "").strip()
        if not target_url:
            return {"dispatched": False, "reason": "agent_api_url_not_configured"}
        token = (os.environ.get("IAT_AGENT_API_TOKEN") or "").strip()

        payload = {
            "instruction_id": instruction_row["id"],
            "project_id": instruction_row["project_id"],
            "instruction_type": instruction_row["instruction_type"],
            "payload": json.loads(instruction_row["payload_json"] or "{}"),
            "status": instruction_row["status"],
        }
        dispatch = dispatch_instruction_http(
            target_url=target_url,
            token=token or None,
            payload=payload,
            timeout_sec=20.0,
            retries=2,
            backoff_sec=1.5,
        )
        if not dispatch.get("ok"):
            exc_message = str(dispatch.get("error", "dispatch_failed"))
            conn.execute(
                """
                UPDATE agent_instructions
                SET status = ?, finished_at = ?, updated_at = ?
                WHERE id = ?
                """,
                ("failed", be.now_iso(), be.now_iso(), instruction_row["id"]),
            )
            self._emit_instruction_event(
                conn,
                instruction_row["id"],
                "error",
                {"message": exc_message, "target_url": target_url},
            )
            conn.commit()
            return {"dispatched": True, "ok": False, "error": exc_message}

        parsed = dispatch.get("response") if isinstance(dispatch.get("response"), dict) else {}
        status = str(parsed.get("status", "accepted")).strip().lower()
        mapped = status if status in {"accepted", "queued", "running", "done", "failed"} else "accepted"
        db_status = "queued" if mapped in {"accepted", "queued"} else mapped
        finished_at = be.now_iso() if db_status in {"done", "failed"} else None
        conn.execute(
            """
            UPDATE agent_instructions
            SET status = ?, started_at = COALESCE(started_at, ?), finished_at = COALESCE(?, finished_at), updated_at = ?
            WHERE id = ?
            """,
            (db_status, be.now_iso(), finished_at, be.now_iso(), instruction_row["id"]),
        )
        self._emit_instruction_event(
            conn,
            instruction_row["id"],
            "result",
            {"agent_response": parsed, "target_url": target_url, "http_status": dispatch.get("http_status")},
        )
        conn.commit()
        return {"dispatched": True, "ok": True, "agent_response": parsed}

    def do_OPTIONS(self):
        self._set_headers(status=HTTPStatus.NO_CONTENT)

    def do_GET(self):
        parsed = urlparse(self.path)
        parts = [p for p in parsed.path.split("/") if p]
        query = parse_qs(parsed.query)

        if parts == ["health"]:
            return self._json(
                {
                    "ok": True,
                    "service": "iat-backend-api",
                    "db": str(self.server.db_path),
                }
            )

        if parts == ["api", "projects"]:
            username = (query.get("username", [""])[0] or "").strip()
            conn = self._db()
            try:
                sql = """
                  SELECT p.id, p.slug, p.name, p.description, p.status, p.created_at, p.updated_at, u.username
                  FROM projects p
                  JOIN users u ON u.id = p.user_id
                """
                params = []
                if username:
                    sql += " WHERE u.username = ?"
                    params.append(be.slugify(username))
                sql += " ORDER BY p.updated_at DESC, p.created_at DESC"
                rows = conn.execute(sql, tuple(params)).fetchall()
                return self._json(
                    {
                        "ok": True,
                        "count": len(rows),
                        "projects": [
                            {
                                "id": r["id"],
                                "slug": r["slug"],
                                "name": r["name"],
                                "description": r["description"],
                                "status": r["status"],
                                "username": r["username"],
                                "created_at": r["created_at"],
                                "updated_at": r["updated_at"],
                            }
                            for r in rows
                        ],
                    }
                )
            finally:
                conn.close()

        if len(parts) == 3 and parts[:2] == ["api", "projects"]:
            project_slug = parts[2]
            conn = self._db()
            try:
                project = be.get_project(conn, "", project_slug)
                if not project:
                    return self._error(HTTPStatus.NOT_FOUND, "Project not found")
                counts = {
                    "runs": conn.execute("SELECT COUNT(*) AS c FROM runs WHERE project_id = ?", (project["id"],))
                    .fetchone()["c"],
                    "jobs": conn.execute(
                        "SELECT COUNT(*) AS c FROM run_jobs WHERE run_id IN (SELECT id FROM runs WHERE project_id = ?)",
                        (project["id"],),
                    ).fetchone()["c"],
                    "assets": conn.execute("SELECT COUNT(*) AS c FROM assets WHERE project_id = ?", (project["id"],))
                    .fetchone()["c"],
                }
                storage = be.project_storage_payload(self.server.repo_root, project)
                return self._json(
                    {
                        "ok": True,
                        "project": {
                            "id": project["id"],
                            "slug": project["slug"],
                            "name": project["name"],
                            "description": project["description"],
                            "status": project["status"],
                            "created_at": project["created_at"],
                            "updated_at": project["updated_at"],
                        },
                        "counts": counts,
                        "storage": storage["storage"],
                    }
                )
            finally:
                conn.close()

        if len(parts) == 4 and parts[:2] == ["api", "projects"] and parts[3] == "storage":
            project_slug = parts[2]
            conn = self._db()
            try:
                project = be.get_project(conn, "", project_slug)
                if not project:
                    return self._error(HTTPStatus.NOT_FOUND, "Project not found")
                payload = be.project_storage_payload(self.server.repo_root, project)
                return self._json({"ok": True, **payload})
            finally:
                conn.close()

        if len(parts) == 4 and parts[:2] == ["api", "projects"] and parts[3] == "secrets":
            project_slug = parts[2]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                rows = be.list_project_secrets(conn, project["id"])
                items = []
                for r in rows:
                    masked = "***"
                    try:
                        plain = be.fetch_project_secret_value(
                            conn,
                            self.server.repo_root,
                            project["id"],
                            r["provider_code"],
                            r["secret_name"],
                        )
                        if plain:
                            masked = be.mask_secret_value(plain)
                    except Exception:
                        masked = "***"
                    items.append(
                        {
                            "id": r["id"],
                            "provider_code": r["provider_code"],
                            "secret_name": r["secret_name"],
                            "masked": masked,
                            "created_at": r["created_at"],
                            "updated_at": r["updated_at"],
                        }
                    )
                return self._json({"ok": True, "count": len(items), "secrets": items})
            finally:
                conn.close()

        if len(parts) == 5 and parts[:2] == ["api", "projects"] and parts[3:] == ["chat", "sessions"]:
            project_slug = parts[2]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                rows = conn.execute(
                    """
                    SELECT s.*, u.username
                    FROM chat_sessions s
                    JOIN users u ON u.id = s.user_id
                    WHERE s.project_id = ?
                    ORDER BY s.updated_at DESC, s.created_at DESC
                    """,
                    (project["id"],),
                ).fetchall()
                return self._json(
                    {
                        "ok": True,
                        "count": len(rows),
                        "sessions": [
                            {
                                "id": r["id"],
                                "title": r["title"],
                                "status": r["status"],
                                "username": r["username"],
                                "created_at": r["created_at"],
                                "updated_at": r["updated_at"],
                            }
                            for r in rows
                        ],
                    }
                )
            finally:
                conn.close()

        if len(parts) == 6 and parts[:2] == ["api", "projects"] and parts[3:5] == ["chat", "sessions"]:
            project_slug = parts[2]
            session_id = parts[5]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                session = self._session_or_404(conn, project["id"], session_id)
                if not session:
                    return
                user = conn.execute("SELECT username FROM users WHERE id = ?", (session["user_id"],)).fetchone()
                return self._json(
                    {
                        "ok": True,
                        "session": {
                            "id": session["id"],
                            "project_id": session["project_id"],
                            "user_id": session["user_id"],
                            "username": user["username"] if user else None,
                            "title": session["title"],
                            "status": session["status"],
                            "context_json": json.loads(session["context_json"] or "{}"),
                            "created_at": session["created_at"],
                            "updated_at": session["updated_at"],
                        },
                    }
                )
            finally:
                conn.close()

        if (
            len(parts) == 7
            and parts[:2] == ["api", "projects"]
            and parts[3:5] == ["chat", "sessions"]
            and parts[6] == "messages"
        ):
            project_slug = parts[2]
            session_id = parts[5]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                session = self._session_or_404(conn, project["id"], session_id)
                if not session:
                    return
                rows = conn.execute(
                    """
                    SELECT *
                    FROM chat_messages
                    WHERE session_id = ?
                    ORDER BY created_at ASC
                    """,
                    (session_id,),
                ).fetchall()
                return self._json(
                    {
                        "ok": True,
                        "count": len(rows),
                        "messages": [
                            {
                                "id": r["id"],
                                "session_id": r["session_id"],
                                "role": r["role"],
                                "content_text": r["content_text"],
                                "content_json": json.loads(r["content_json"] or "{}"),
                                "voice_asset_id": r["voice_asset_id"],
                                "token_usage_json": json.loads(r["token_usage_json"] or "{}"),
                                "created_at": r["created_at"],
                            }
                            for r in rows
                        ],
                    }
                )
            finally:
                conn.close()

        if len(parts) == 5 and parts[:2] == ["api", "projects"] and parts[3:] == ["agent", "instructions"]:
            project_slug = parts[2]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                rows = conn.execute(
                    """
                    SELECT *
                    FROM agent_instructions
                    WHERE project_id = ?
                    ORDER BY created_at DESC
                    """,
                    (project["id"],),
                ).fetchall()
                return self._json(
                    {
                        "ok": True,
                        "count": len(rows),
                        "instructions": [
                            {
                                "id": r["id"],
                                "instruction_type": r["instruction_type"],
                                "status": r["status"],
                                "priority": r["priority"],
                                "requires_confirmation": bool(r["requires_confirmation"]),
                                "created_at": r["created_at"],
                                "updated_at": r["updated_at"],
                            }
                            for r in rows
                        ],
                    }
                )
            finally:
                conn.close()

        if len(parts) == 6 and parts[:2] == ["api", "projects"] and parts[3:5] == ["agent", "instructions"]:
            project_slug = parts[2]
            instruction_id = parts[5]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                instruction = self._instruction_or_404(conn, project["id"], instruction_id)
                if not instruction:
                    return
                return self._json(
                    {
                        "ok": True,
                        "instruction": {
                            "id": instruction["id"],
                            "project_id": instruction["project_id"],
                            "session_id": instruction["session_id"],
                            "message_id": instruction["message_id"],
                            "instruction_type": instruction["instruction_type"],
                            "payload_json": json.loads(instruction["payload_json"] or "{}"),
                            "status": instruction["status"],
                            "priority": instruction["priority"],
                            "requires_confirmation": bool(instruction["requires_confirmation"]),
                            "confirmed_by_user_id": instruction["confirmed_by_user_id"],
                            "queued_at": instruction["queued_at"],
                            "started_at": instruction["started_at"],
                            "finished_at": instruction["finished_at"],
                            "created_at": instruction["created_at"],
                            "updated_at": instruction["updated_at"],
                        },
                    }
                )
            finally:
                conn.close()

        if (
            len(parts) == 7
            and parts[:2] == ["api", "projects"]
            and parts[3:5] == ["agent", "instructions"]
            and parts[6] == "events"
        ):
            project_slug = parts[2]
            instruction_id = parts[5]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                instruction = self._instruction_or_404(conn, project["id"], instruction_id)
                if not instruction:
                    return
                rows = conn.execute(
                    """
                    SELECT *
                    FROM agent_instruction_events
                    WHERE instruction_id = ?
                    ORDER BY created_at ASC
                    """,
                    (instruction_id,),
                ).fetchall()
                return self._json(
                    {
                        "ok": True,
                        "count": len(rows),
                        "events": [
                            {
                                "id": r["id"],
                                "instruction_id": r["instruction_id"],
                                "event_type": r["event_type"],
                                "event_payload_json": json.loads(r["event_payload_json"] or "{}"),
                                "created_at": r["created_at"],
                            }
                            for r in rows
                        ],
                    }
                )
            finally:
                conn.close()

        if len(parts) == 6 and parts[:2] == ["api", "projects"] and parts[3:5] == ["voice", "requests"]:
            project_slug = parts[2]
            request_id = parts[5]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                req = self._voice_request_or_404(conn, project["id"], request_id)
                if not req:
                    return
                return self._json(
                    {
                        "ok": True,
                        "request": {
                            "id": req["id"],
                            "project_id": req["project_id"],
                            "session_id": req["session_id"],
                            "message_id": req["message_id"],
                            "direction": req["direction"],
                            "provider_code": req["provider_code"],
                            "input_asset_id": req["input_asset_id"],
                            "output_asset_id": req["output_asset_id"],
                            "status": req["status"],
                            "latency_ms": req["latency_ms"],
                            "meta_json": json.loads(req["meta_json"] or "{}"),
                            "created_at": req["created_at"],
                        },
                    }
                )
            finally:
                conn.close()

        return self._error(HTTPStatus.NOT_FOUND, "Route not found")

    def do_POST(self):
        parsed = urlparse(self.path)
        parts = [p for p in parsed.path.split("/") if p]
        try:
            body = self._read_json_body()
        except ValueError as exc:
            return self._error(HTTPStatus.BAD_REQUEST, str(exc))

        if parts == ["api", "projects"]:
            name = str(body.get("name", "")).strip()
            if not name:
                return self._error(HTTPStatus.BAD_REQUEST, "Field 'name' is required")
            username = str(body.get("username", "local")).strip() or "local"
            display_name = str(body.get("user_display_name", "Local User")).strip() or "Local User"
            slug = str(body.get("slug", "")).strip() or be.slugify(name)
            description = str(body.get("description", "")).strip()

            conn = self._db()
            try:
                user = be.ensure_user(conn, username, display_name, None)
                project = be.ensure_project(conn, user["id"], slug, name, description)
                payload = be.project_storage_payload(self.server.repo_root, project)
                return self._json({"ok": True, "project": payload["project"], "storage": payload["storage"]})
            finally:
                conn.close()

        if len(parts) == 4 and parts[:2] == ["api", "projects"] and parts[3] == "secrets":
            project_slug = parts[2]
            provider_code = str(body.get("provider_code", "")).strip().lower()
            secret_name = str(body.get("secret_name", "")).strip()
            secret_value = str(body.get("secret_value", "")).strip()
            if not provider_code or not secret_name or not secret_value:
                return self._error(
                    HTTPStatus.BAD_REQUEST,
                    "Fields 'provider_code', 'secret_name', 'secret_value' are required",
                )
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                secret_id = be.upsert_project_secret(
                    conn,
                    self.server.repo_root,
                    project["id"],
                    provider_code,
                    secret_name,
                    secret_value,
                )
                return self._json(
                    {
                        "ok": True,
                        "secret": {
                            "id": secret_id,
                            "provider_code": provider_code,
                            "secret_name": secret_name,
                            "masked": be.mask_secret_value(secret_value),
                        },
                    }
                )
            finally:
                conn.close()

        if len(parts) == 5 and parts[:2] == ["api", "projects"] and parts[3:] == ["chat", "sessions"]:
            project_slug = parts[2]
            title = str(body.get("title", "")).strip()
            username = str(body.get("username", "local")).strip() or "local"
            context_json = body.get("context_json", {})
            if not isinstance(context_json, dict):
                return self._error(HTTPStatus.BAD_REQUEST, "Field 'context_json' must be an object")

            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                user = self._ensure_actor_user(conn, username)
                session_id = be.uid()
                now = be.now_iso()
                conn.execute(
                    """
                    INSERT INTO chat_sessions (id, project_id, user_id, title, status, context_json, created_at, updated_at)
                    VALUES (?, ?, ?, ?, 'active', ?, ?, ?)
                    """,
                    (session_id, project["id"], user["id"], title, be.to_json(context_json), now, now),
                )
                conn.commit()
                return self._json(
                    {
                        "ok": True,
                        "session": {
                            "id": session_id,
                            "project_id": project["id"],
                            "user_id": user["id"],
                            "username": user["username"],
                            "title": title,
                            "status": "active",
                            "context_json": context_json,
                            "created_at": now,
                            "updated_at": now,
                        },
                    }
                )
            finally:
                conn.close()

        if (
            len(parts) == 7
            and parts[:2] == ["api", "projects"]
            and parts[3:5] == ["chat", "sessions"]
            and parts[6] == "messages"
        ):
            project_slug = parts[2]
            session_id = parts[5]
            role = str(body.get("role", "user")).strip().lower()
            if role not in {"user", "assistant", "system", "tool"}:
                return self._error(HTTPStatus.BAD_REQUEST, "Field 'role' must be one of: user|assistant|system|tool")
            content_text = str(body.get("content_text") or body.get("text") or "").strip()
            if not content_text:
                return self._error(HTTPStatus.BAD_REQUEST, "Field 'content_text' is required")
            content_json = body.get("content_json", {})
            if not isinstance(content_json, dict):
                return self._error(HTTPStatus.BAD_REQUEST, "Field 'content_json' must be an object")
            token_usage_json = body.get("token_usage_json", {})
            if not isinstance(token_usage_json, dict):
                return self._error(HTTPStatus.BAD_REQUEST, "Field 'token_usage_json' must be an object")

            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                session = self._session_or_404(conn, project["id"], session_id)
                if not session:
                    return
                message_id = be.uid()
                now = be.now_iso()
                conn.execute(
                    """
                    INSERT INTO chat_messages (id, session_id, role, content_text, content_json, voice_asset_id, token_usage_json, created_at)
                    VALUES (?, ?, ?, ?, ?, NULL, ?, ?)
                    """,
                    (message_id, session_id, role, content_text, be.to_json(content_json), be.to_json(token_usage_json), now),
                )
                conn.execute(
                    "UPDATE chat_sessions SET updated_at = ? WHERE id = ?",
                    (now, session_id),
                )
                conn.commit()
                return self._json(
                    {
                        "ok": True,
                        "message": {
                            "id": message_id,
                            "session_id": session_id,
                            "role": role,
                            "content_text": content_text,
                            "content_json": content_json,
                            "token_usage_json": token_usage_json,
                            "created_at": now,
                        },
                    }
                )
            finally:
                conn.close()

        if len(parts) == 5 and parts[:2] == ["api", "projects"] and parts[3:] == ["agent", "instructions"]:
            project_slug = parts[2]
            instruction_type = str(body.get("instruction_type", "")).strip().lower()
            if not instruction_type:
                return self._error(HTTPStatus.BAD_REQUEST, "Field 'instruction_type' is required")
            payload_json = body.get("payload_json", body.get("payload", {}))
            if not isinstance(payload_json, dict):
                return self._error(HTTPStatus.BAD_REQUEST, "Field 'payload_json' must be an object")
            session_id = str(body.get("session_id", "")).strip() or None
            message_id = str(body.get("message_id", "")).strip() or None
            priority = body.get("priority", 100)
            try:
                priority = int(priority)
            except Exception:
                return self._error(HTTPStatus.BAD_REQUEST, "Field 'priority' must be integer")
            requires_confirmation = parse_bool(body.get("requires_confirmation"), False)
            dispatch_to_agent = parse_bool(body.get("dispatch_to_agent"), False)

            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return

                if session_id:
                    session = self._session_or_404(conn, project["id"], session_id)
                    if not session:
                        return
                if message_id:
                    msg = conn.execute(
                        """
                        SELECT m.*
                        FROM chat_messages m
                        JOIN chat_sessions s ON s.id = m.session_id
                        WHERE m.id = ? AND s.project_id = ?
                        """,
                        (message_id, project["id"]),
                    ).fetchone()
                    if not msg:
                        return self._error(HTTPStatus.BAD_REQUEST, "message_id does not belong to this project")

                now = be.now_iso()
                instruction_id = be.uid()
                initial_status = "draft" if requires_confirmation else "queued"
                queued_at = None if requires_confirmation else now
                conn.execute(
                    """
                    INSERT INTO agent_instructions
                    (id, project_id, session_id, message_id, instruction_type, payload_json, status, priority,
                     requires_confirmation, confirmed_by_user_id, queued_at, started_at, finished_at, created_at, updated_at)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, ?, NULL, NULL, ?, ?)
                    """,
                    (
                        instruction_id,
                        project["id"],
                        session_id,
                        message_id,
                        instruction_type,
                        be.to_json(payload_json),
                        initial_status,
                        priority,
                        1 if requires_confirmation else 0,
                        queued_at,
                        now,
                        now,
                    ),
                )
                self._emit_instruction_event(
                    conn,
                    instruction_id,
                    "created",
                    {"status": initial_status, "instruction_type": instruction_type},
                )
                if not requires_confirmation:
                    self._emit_instruction_event(conn, instruction_id, "queued", {"queued_at": now})
                conn.commit()

                instruction_row = conn.execute(
                    "SELECT * FROM agent_instructions WHERE id = ?",
                    (instruction_id,),
                ).fetchone()
                dispatch_result = (
                    self._dispatch_instruction_if_configured(conn, instruction_row)
                    if (dispatch_to_agent and not requires_confirmation)
                    else {"dispatched": False, "reason": "dispatch_disabled_or_confirmation_required"}
                )
                refreshed = conn.execute("SELECT * FROM agent_instructions WHERE id = ?", (instruction_id,)).fetchone()
                return self._json(
                    {
                        "ok": True,
                        "instruction": {
                            "id": refreshed["id"],
                            "project_id": refreshed["project_id"],
                            "session_id": refreshed["session_id"],
                            "message_id": refreshed["message_id"],
                            "instruction_type": refreshed["instruction_type"],
                            "payload_json": json.loads(refreshed["payload_json"] or "{}"),
                            "status": refreshed["status"],
                            "priority": refreshed["priority"],
                            "requires_confirmation": bool(refreshed["requires_confirmation"]),
                            "queued_at": refreshed["queued_at"],
                            "started_at": refreshed["started_at"],
                            "finished_at": refreshed["finished_at"],
                            "created_at": refreshed["created_at"],
                            "updated_at": refreshed["updated_at"],
                        },
                        "dispatch": dispatch_result,
                    }
                )
            finally:
                conn.close()

        if (
            len(parts) == 7
            and parts[:2] == ["api", "projects"]
            and parts[3:5] == ["agent", "instructions"]
            and parts[6] == "confirm"
        ):
            project_slug = parts[2]
            instruction_id = parts[5]
            username = str(body.get("username", "local")).strip() or "local"
            dispatch_to_agent = parse_bool(body.get("dispatch_to_agent"), False)
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                instruction = self._instruction_or_404(conn, project["id"], instruction_id)
                if not instruction:
                    return
                user = self._ensure_actor_user(conn, username)
                now = be.now_iso()
                conn.execute(
                    """
                    UPDATE agent_instructions
                    SET status = 'queued',
                        confirmed_by_user_id = ?,
                        queued_at = COALESCE(queued_at, ?),
                        updated_at = ?
                    WHERE id = ?
                    """,
                    (user["id"], now, now, instruction_id),
                )
                self._emit_instruction_event(conn, instruction_id, "status_change", {"status": "queued"})
                self._emit_instruction_event(conn, instruction_id, "confirmed", {"user_id": user["id"]})
                conn.commit()

                refreshed = conn.execute("SELECT * FROM agent_instructions WHERE id = ?", (instruction_id,)).fetchone()
                dispatch_result = (
                    self._dispatch_instruction_if_configured(conn, refreshed)
                    if dispatch_to_agent
                    else {"dispatched": False, "reason": "dispatch_disabled"}
                )
                refreshed = conn.execute("SELECT * FROM agent_instructions WHERE id = ?", (instruction_id,)).fetchone()
                return self._json(
                    {
                        "ok": True,
                        "instruction_id": instruction_id,
                        "status": refreshed["status"],
                        "dispatch": dispatch_result,
                    }
                )
            finally:
                conn.close()

        if (
            len(parts) == 7
            and parts[:2] == ["api", "projects"]
            and parts[3:5] == ["agent", "instructions"]
            and parts[6] == "cancel"
        ):
            project_slug = parts[2]
            instruction_id = parts[5]
            reason = str(body.get("reason", "")).strip()
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                instruction = self._instruction_or_404(conn, project["id"], instruction_id)
                if not instruction:
                    return
                now = be.now_iso()
                conn.execute(
                    """
                    UPDATE agent_instructions
                    SET status = 'canceled', finished_at = COALESCE(finished_at, ?), updated_at = ?
                    WHERE id = ?
                    """,
                    (now, now, instruction_id),
                )
                self._emit_instruction_event(conn, instruction_id, "status_change", {"status": "canceled", "reason": reason})
                conn.commit()
                return self._json({"ok": True, "instruction_id": instruction_id, "status": "canceled"})
            finally:
                conn.close()

        if len(parts) == 5 and parts[:2] == ["api", "projects"] and parts[3:] == ["voice", "stt"]:
            project_slug = parts[2]
            provider_code = str(body.get("provider_code", "stt_default")).strip() or "stt_default"
            session_id = str(body.get("session_id", "")).strip() or None
            message_id = str(body.get("message_id", "")).strip() or None
            input_asset_id = str(body.get("input_asset_id", "")).strip() or None
            transcript_text = str(body.get("transcript_text", "")).strip()
            create_message = parse_bool(body.get("create_message"), True)
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                if session_id:
                    session = self._session_or_404(conn, project["id"], session_id)
                    if not session:
                        return

                now = be.now_iso()
                request_id = be.uid()
                status = "done" if transcript_text else "queued"
                meta = {"mode": "mock" if transcript_text else "queued", "transcript_text": transcript_text}
                conn.execute(
                    """
                    INSERT INTO voice_requests
                    (id, project_id, session_id, message_id, direction, provider_code, input_asset_id, output_asset_id,
                     status, latency_ms, meta_json, created_at)
                    VALUES (?, ?, ?, ?, 'stt', ?, ?, NULL, ?, NULL, ?, ?)
                    """,
                    (request_id, project["id"], session_id, message_id, provider_code, input_asset_id, status, be.to_json(meta), now),
                )

                created_message = None
                if transcript_text and create_message and session_id:
                    msg_id = be.uid()
                    conn.execute(
                        """
                        INSERT INTO chat_messages (id, session_id, role, content_text, content_json, voice_asset_id, token_usage_json, created_at)
                        VALUES (?, ?, 'user', ?, '{}', NULL, '{}', ?)
                        """,
                        (msg_id, session_id, transcript_text, now),
                    )
                    conn.execute("UPDATE chat_sessions SET updated_at = ? WHERE id = ?", (now, session_id))
                    created_message = {
                        "id": msg_id,
                        "session_id": session_id,
                        "role": "user",
                        "content_text": transcript_text,
                        "created_at": now,
                    }
                conn.commit()
                return self._json(
                    {
                        "ok": True,
                        "request": {
                            "id": request_id,
                            "direction": "stt",
                            "status": status,
                            "provider_code": provider_code,
                            "session_id": session_id,
                            "message_id": message_id,
                            "input_asset_id": input_asset_id,
                            "meta_json": meta,
                            "created_at": now,
                        },
                        "message": created_message,
                    }
                )
            finally:
                conn.close()

        if len(parts) == 5 and parts[:2] == ["api", "projects"] and parts[3:] == ["voice", "tts"]:
            project_slug = parts[2]
            provider_code = str(body.get("provider_code", "tts_default")).strip() or "tts_default"
            session_id = str(body.get("session_id", "")).strip() or None
            message_id = str(body.get("message_id", "")).strip() or None
            text = str(body.get("text", "")).strip()
            if not text:
                return self._error(HTTPStatus.BAD_REQUEST, "Field 'text' is required")
            output_asset_id = str(body.get("output_asset_id", "")).strip() or None

            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                if session_id:
                    session = self._session_or_404(conn, project["id"], session_id)
                    if not session:
                        return

                now = be.now_iso()
                request_id = be.uid()
                status = "done" if output_asset_id else "queued"
                meta = {"mode": "mock" if output_asset_id else "queued", "text": text}
                conn.execute(
                    """
                    INSERT INTO voice_requests
                    (id, project_id, session_id, message_id, direction, provider_code, input_asset_id, output_asset_id,
                     status, latency_ms, meta_json, created_at)
                    VALUES (?, ?, ?, ?, 'tts', ?, NULL, ?, ?, NULL, ?, ?)
                    """,
                    (request_id, project["id"], session_id, message_id, provider_code, output_asset_id, status, be.to_json(meta), now),
                )
                conn.commit()
                return self._json(
                    {
                        "ok": True,
                        "request": {
                            "id": request_id,
                            "direction": "tts",
                            "status": status,
                            "provider_code": provider_code,
                            "session_id": session_id,
                            "message_id": message_id,
                            "output_asset_id": output_asset_id,
                            "meta_json": meta,
                            "created_at": now,
                        },
                    }
                )
            finally:
                conn.close()

        if len(parts) == 5 and parts[:2] == ["api", "projects"] and parts[3:] == ["runs", "ingest"]:
            project_slug = parts[2]
            run_log = str(body.get("run_log", "")).strip()
            if not run_log:
                return self._error(HTTPStatus.BAD_REQUEST, "Field 'run_log' is required")
            run_log_path = (self.server.repo_root / run_log).resolve()
            if not run_log_path.exists():
                return self._error(HTTPStatus.BAD_REQUEST, f"Run log not found: {run_log_path}")
            compute_hashes = parse_bool(body.get("compute_hashes"), False)
            conn = self._db()
            try:
                project = be.get_project(conn, "", project_slug)
                if not project:
                    return self._error(HTTPStatus.NOT_FOUND, "Project not found")
                result = be.ingest_run(conn, self.server.repo_root, project, run_log_path, compute_hashes)
                return self._json({"ok": True, "project_slug": project_slug, **result})
            finally:
                conn.close()

        if len(parts) == 4 and parts[:2] == ["api", "projects"] and parts[3] == "export":
            project_slug = parts[2]
            include_files = parse_bool(body.get("include_files"), True)
            output = str(body.get("output", "")).strip()
            conn = self._db()
            try:
                project = be.get_project(conn, "", project_slug)
                if not project:
                    return self._error(HTTPStatus.NOT_FOUND, "Project not found")
                storage_payload = be.project_storage_payload(self.server.repo_root, project)
                source_files_root = Path(storage_payload["storage"]["local"]["project_root"])
                if output:
                    output_path = (self.server.repo_root / output).resolve()
                else:
                    stamp = be.now_iso().replace(":", "-")
                    output_path = (
                        self.server.repo_root / "generated" / "exports" / f"{project['slug']}_{stamp}.tar.gz"
                    ).resolve()
                result = be.export_project_package(
                    conn,
                    self.server.db_path,
                    self.server.repo_root,
                    project,
                    source_files_root,
                    output_path,
                    include_files,
                )
                return self._json({"ok": True, "project_slug": project_slug, **result})
            finally:
                conn.close()

        if len(parts) == 4 and parts[:2] == ["api", "projects"] and parts[3] == "sync-s3":
            project_slug = parts[2]
            dry_run = parse_bool(body.get("dry_run"), False)
            delete = parse_bool(body.get("delete"), False)
            allow_missing_local = parse_bool(body.get("allow_missing_local"), True)
            conn = self._db()
            try:
                project = be.get_project(conn, "", project_slug)
                if not project:
                    return self._error(HTTPStatus.NOT_FOUND, "Project not found")
                payload = be.project_storage_payload(self.server.repo_root, project)
            finally:
                conn.close()

            # Reuse existing CLI command implementation semantics by emulating args object.
            class SyncArgs:
                pass

            sync_args = SyncArgs()
            sync_args.db = str(self.server.db_path)
            sync_args.project_id = ""
            sync_args.project_slug = project_slug
            sync_args.dry_run = dry_run
            sync_args.delete = delete
            sync_args.allow_missing_local = allow_missing_local
            try:
                # Command prints JSON on success; for API we return a deterministic payload.
                be.cmd_sync_project_s3(sync_args)
                return self._json(
                    {
                        "ok": True,
                        "project_slug": project_slug,
                        "requested": {
                            "dry_run": dry_run,
                            "delete": delete,
                            "allow_missing_local": allow_missing_local,
                        },
                        "storage": payload["storage"],
                    }
                )
            except SystemExit as exc:
                return self._error(HTTPStatus.BAD_REQUEST, str(exc))

        return self._error(HTTPStatus.NOT_FOUND, "Route not found")

    def do_PUT(self):
        parsed = urlparse(self.path)
        parts = [p for p in parsed.path.split("/") if p]
        try:
            body = self._read_json_body()
        except ValueError as exc:
            return self._error(HTTPStatus.BAD_REQUEST, str(exc))

        if len(parts) == 5 and parts[:2] == ["api", "projects"] and parts[3:] == ["storage", "local"]:
            project_slug = parts[2]
            base_dir = body.get("base_dir")
            project_root = body.get("project_root")
            if not base_dir and not project_root:
                return self._error(HTTPStatus.BAD_REQUEST, "Provide at least one of: base_dir, project_root")

            conn = self._db()
            try:
                project = be.get_project(conn, "", project_slug)
                if not project:
                    return self._error(HTTPStatus.NOT_FOUND, "Project not found")
                settings = be.load_project_settings(project)
                storage = settings.setdefault("storage", {})
                local = storage.setdefault("local", {})
                if base_dir is not None:
                    local["base_dir"] = str(base_dir).strip()
                if project_root is not None:
                    local["project_root"] = str(project_root).strip()
                be.save_project_settings(conn, project["id"], settings)
                refreshed = be.get_project(conn, project["id"], "")
                payload = be.project_storage_payload(self.server.repo_root, refreshed)
                return self._json({"ok": True, "updated": "local", **payload})
            finally:
                conn.close()

        if len(parts) == 5 and parts[:2] == ["api", "projects"] and parts[3:] == ["storage", "s3"]:
            project_slug = parts[2]
            conn = self._db()
            try:
                project = be.get_project(conn, "", project_slug)
                if not project:
                    return self._error(HTTPStatus.NOT_FOUND, "Project not found")
                settings = be.load_project_settings(project)
                storage = settings.setdefault("storage", {})
                s3 = storage.setdefault("s3", {})

                if "enabled" in body:
                    s3["enabled"] = parse_bool(body.get("enabled"), False)
                for key in ["bucket", "prefix", "region", "profile", "endpoint_url"]:
                    if key in body:
                        s3[key] = str(body.get(key) or "").strip()

                be.save_project_settings(conn, project["id"], settings)
                refreshed = be.get_project(conn, project["id"], "")
                payload = be.project_storage_payload(self.server.repo_root, refreshed)
                return self._json({"ok": True, "updated": "s3", **payload})
            finally:
                conn.close()

        return self._error(HTTPStatus.NOT_FOUND, "Route not found")

    def do_DELETE(self):
        parsed = urlparse(self.path)
        parts = [p for p in parsed.path.split("/") if p]

        if len(parts) == 6 and parts[:2] == ["api", "projects"] and parts[3] == "secrets":
            project_slug = parts[2]
            provider_code = parts[4].strip().lower()
            secret_name = parts[5].strip()
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                cur = conn.execute(
                    """
                    DELETE FROM project_api_secrets
                    WHERE project_id = ? AND provider_code = ? AND secret_name = ?
                    """,
                    (project["id"], provider_code, secret_name),
                )
                conn.commit()
                return self._json(
                    {
                        "ok": True,
                        "provider_code": provider_code,
                        "secret_name": secret_name,
                        "deleted": cur.rowcount,
                    }
                )
            finally:
                conn.close()

        return self._error(HTTPStatus.NOT_FOUND, "Route not found")


def build_parser():
    parser = argparse.ArgumentParser(description="IAT backend REST API server")
    parser.add_argument("--host", default="127.0.0.1", help="Bind host")
    parser.add_argument("--port", type=int, default=8787, help="Bind port")
    parser.add_argument("--db", default=be.DEFAULT_DB, help="SQLite DB path")
    parser.add_argument("--cors-origin", default="*", help="CORS Access-Control-Allow-Origin value")
    parser.add_argument("--with-default-user", action="store_true", help="Ensure local default user at startup")
    return parser


def main():
    parser = build_parser()
    args = parser.parse_args()
    repo_root = Path.cwd().resolve()
    db_path = (repo_root / args.db).resolve()
    conn = be.connect_db(db_path)
    be.init_schema(conn)
    if args.with_default_user:
        be.ensure_user(conn, "local", "Local User", None)
    conn.close()

    server = ApiServer((args.host, args.port), Handler, repo_root, db_path, args.cors_origin)
    print(
        json.dumps(
            {
                "ok": True,
                "service": "iat-backend-api",
                "host": args.host,
                "port": args.port,
                "db": str(db_path),
                "repo_root": str(repo_root),
            },
            ensure_ascii=False,
        )
    )
    server.serve_forever()


if __name__ == "__main__":
    main()
