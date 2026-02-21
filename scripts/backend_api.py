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

    def _json_cell(self, raw, fallback):
        try:
            parsed = json.loads(raw or "")
            return parsed if parsed is not None else fallback
        except Exception:
            return fallback

    def _coalesce_run_row(self, row):
        return {
            "id": row["id"],
            "project_id": row["project_id"],
            "run_mode": row["run_mode"] or row["mode"],
            "status": row["status"],
            "stage": row["stage"],
            "time_of_day": row["time_of_day"],
            "weather": row["weather"],
            "model_name": row["model_name"] or row["model"],
            "provider_code": row["provider_code"],
            "settings_snapshot_json": self._json_cell(row["settings_snapshot_json"] or row["meta_json"], {}),
            "started_at": row["started_at"],
            "finished_at": row["finished_at"],
            "created_at": row["created_at"],
            "run_log_path": row["run_log_path"],
            "image_size": row["image_size"],
            "image_quality": row["image_quality"],
        }

    def _fetch_candidates_for_job(self, conn, job_id):
        rows = conn.execute(
            """
            SELECT
              COALESCE(rc.id, ljc.id) AS id,
              COALESCE(rc.job_id, ljc.job_id) AS job_id,
              COALESCE(rc.candidate_index, ljc.candidate_index) AS candidate_index,
              COALESCE(rc.status, ljc.status) AS status,
              rc.output_asset_id AS output_asset_id,
              rc.final_asset_id AS final_asset_id,
              ljc.output_path AS output_path,
              ljc.final_output_path AS final_output_path,
              COALESCE(rc.rank_hard_failures, ljc.rank_hard_failures, 0) AS rank_hard_failures,
              COALESCE(rc.rank_soft_warnings, ljc.rank_soft_warnings, 0) AS rank_soft_warnings,
              COALESCE(rc.rank_avg_chroma_exceed, ljc.rank_avg_chroma_exceed, 0) AS rank_avg_chroma_exceed,
              COALESCE(rc.meta_json, ljc.meta_json, '{}') AS meta_json,
              COALESCE(rc.created_at, ljc.created_at) AS created_at
            FROM run_job_candidates ljc
            LEFT JOIN run_candidates rc ON rc.id = ljc.id
            WHERE ljc.job_id = ?

            UNION ALL

            SELECT
              rc.id AS id,
              rc.job_id AS job_id,
              rc.candidate_index AS candidate_index,
              rc.status AS status,
              rc.output_asset_id AS output_asset_id,
              rc.final_asset_id AS final_asset_id,
              NULL AS output_path,
              NULL AS final_output_path,
              rc.rank_hard_failures AS rank_hard_failures,
              rc.rank_soft_warnings AS rank_soft_warnings,
              rc.rank_avg_chroma_exceed AS rank_avg_chroma_exceed,
              rc.meta_json AS meta_json,
              rc.created_at AS created_at
            FROM run_candidates rc
            WHERE rc.job_id = ?
              AND NOT EXISTS (SELECT 1 FROM run_job_candidates x WHERE x.id = rc.id)
            ORDER BY candidate_index ASC, created_at ASC
            """,
            (job_id, job_id),
        ).fetchall()
        return [
            {
                "id": r["id"],
                "job_id": r["job_id"],
                "candidate_index": r["candidate_index"],
                "status": r["status"],
                "output_asset_id": r["output_asset_id"],
                "final_asset_id": r["final_asset_id"],
                "output_path": r["output_path"],
                "final_output_path": r["final_output_path"],
                "rank_hard_failures": r["rank_hard_failures"],
                "rank_soft_warnings": r["rank_soft_warnings"],
                "rank_avg_chroma_exceed": r["rank_avg_chroma_exceed"],
                "meta_json": self._json_cell(r["meta_json"], {}),
                "created_at": r["created_at"],
            }
            for r in rows
        ]

    def _fetch_jobs_with_candidates(self, conn, run_id):
        jobs = conn.execute(
            """
            SELECT *
            FROM run_jobs
            WHERE run_id = ?
            ORDER BY created_at ASC, job_key ASC
            """,
            (run_id,),
        ).fetchall()
        out = []
        for r in jobs:
            out.append(
                {
                    "id": r["id"],
                    "run_id": r["run_id"],
                    "job_key": r["job_key"],
                    "status": r["status"],
                    "prompt_text": r["prompt_text"] or "",
                    "selected_candidate_index": (
                        r["selected_candidate_index"]
                        if r["selected_candidate_index"] is not None
                        else r["selected_candidate"]
                    ),
                    "final_asset_id": r["final_asset_id"],
                    "final_output": r["final_output"],
                    "meta_json": self._json_cell(r["meta_json"], {}),
                    "created_at": r["created_at"],
                    "candidates": self._fetch_candidates_for_job(conn, r["id"]),
                }
            )
        return out

    def _coalesce_style_guide(self, row):
        return {
            "id": row["id"],
            "project_id": row["project_id"],
            "name": row["name"],
            "description": row["description"],
            "rules_json": self._json_cell(row["rules_json"] or row["specs_json"], {}),
            "is_default": bool(row["is_default"] or 0),
            "created_at": row["created_at"],
            "updated_at": row["updated_at"],
        }

    def _coalesce_character(self, row):
        return {
            "id": row["id"],
            "project_id": row["project_id"],
            "code": row["code"],
            "name": row["name"],
            "bio": row["bio"],
            "identity_constraints_json": self._json_cell(row["identity_constraints_json"], {}),
            "created_at": row["created_at"],
            "updated_at": row["updated_at"],
        }

    def _coalesce_reference_set(self, row):
        return {
            "id": row["id"],
            "project_id": row["project_id"],
            "name": row["name"] or row["title"],
            "kind": row["kind"] or "other",
            "metadata_json": self._json_cell(row["metadata_json"], {}),
            "notes": row["notes"],
            "created_at": row["created_at"],
            "updated_at": row["updated_at"],
        }

    def _coalesce_reference_item(self, row):
        return {
            "id": row["id"],
            "reference_set_id": row["reference_set_id"],
            "asset_id": row["asset_id"],
            "weight": float(row["weight"] if row["weight"] is not None else 1.0),
            "notes": row["notes"],
            "created_at": row["created_at"],
        }

    def _coalesce_provider_account(self, row):
        api_key_value = row["api_key"] if "api_key" in row.keys() else None
        return {
            "id": row["id"],
            "project_id": row["project_id"],
            "provider_code": row["provider_code"],
            "is_enabled": bool(row["is_enabled"] if row["is_enabled"] is not None else 1),
            "config_json": self._json_cell(row["config_json"] or row["meta_json"], {}),
            "has_api_key": bool(api_key_value),
            "created_at": row["created_at"],
            "updated_at": row["updated_at"],
        }

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

    def _emit_audit_event(self, conn, project_id, event_code, payload, actor_user_id=None, target_type=None, target_id=None):
        be.emit_audit_event(
            conn,
            project_id,
            actor_user_id,
            event_code,
            payload or {},
            target_type=target_type,
            target_id=target_id,
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
            self._emit_audit_event(
                conn,
                instruction_row["project_id"],
                "instruction.dispatch_failed",
                {"instruction_id": instruction_row["id"], "error": exc_message, "target_url": target_url},
                target_type="agent_instruction",
                target_id=instruction_row["id"],
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
        self._emit_audit_event(
            conn,
            instruction_row["project_id"],
            "instruction.dispatched",
            {
                "instruction_id": instruction_row["id"],
                "status": db_status,
                "target_url": target_url,
                "http_status": dispatch.get("http_status"),
            },
            target_type="agent_instruction",
            target_id=instruction_row["id"],
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
                  JOIN app_users u ON u.id = COALESCE(p.owner_user_id, p.user_id)
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
                storage = be.project_storage_payload(self.server.repo_root, project, conn)
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
                payload = be.project_storage_payload(self.server.repo_root, project, conn)
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

        if len(parts) == 4 and parts[:2] == ["api", "projects"] and parts[3] == "runs":
            project_slug = parts[2]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                try:
                    limit = int((query.get("limit", ["200"])[0] or "200").strip())
                except Exception:
                    limit = 200
                limit = max(1, min(limit, 1000))
                rows = conn.execute(
                    """
                    SELECT *
                    FROM runs
                    WHERE project_id = ?
                    ORDER BY created_at DESC
                    LIMIT ?
                    """,
                    (project["id"], limit),
                ).fetchall()
                items = [self._coalesce_run_row(r) for r in rows]
                return self._json({"ok": True, "count": len(items), "runs": items})
            finally:
                conn.close()

        if len(parts) == 5 and parts[:2] == ["api", "projects"] and parts[3] == "runs":
            project_slug = parts[2]
            run_id = parts[4]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                run_row = conn.execute(
                    """
                    SELECT *
                    FROM runs
                    WHERE id = ? AND project_id = ?
                    """,
                    (run_id, project["id"]),
                ).fetchone()
                if not run_row:
                    return self._error(HTTPStatus.NOT_FOUND, "Run not found")
                jobs = self._fetch_jobs_with_candidates(conn, run_row["id"])
                return self._json({"ok": True, "run": self._coalesce_run_row(run_row), "jobs": jobs})
            finally:
                conn.close()

        if len(parts) == 6 and parts[:2] == ["api", "projects"] and parts[3] == "runs" and parts[5] == "jobs":
            project_slug = parts[2]
            run_id = parts[4]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                run_row = conn.execute(
                    """
                    SELECT id
                    FROM runs
                    WHERE id = ? AND project_id = ?
                    """,
                    (run_id, project["id"]),
                ).fetchone()
                if not run_row:
                    return self._error(HTTPStatus.NOT_FOUND, "Run not found")
                jobs = self._fetch_jobs_with_candidates(conn, run_row["id"])
                return self._json({"ok": True, "run_id": run_id, "count": len(jobs), "jobs": jobs})
            finally:
                conn.close()

        if len(parts) == 4 and parts[:2] == ["api", "projects"] and parts[3] == "assets":
            project_slug = parts[2]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                try:
                    limit = int((query.get("limit", ["500"])[0] or "500").strip())
                except Exception:
                    limit = 500
                limit = max(1, min(limit, 2000))
                rows = conn.execute(
                    """
                    SELECT *
                    FROM assets
                    WHERE project_id = ?
                    ORDER BY created_at DESC
                    LIMIT ?
                    """,
                    (project["id"], limit),
                ).fetchall()
                assets = [
                    {
                        "id": r["id"],
                        "project_id": r["project_id"],
                        "kind": r["kind"] or r["asset_kind"],
                        "asset_kind": r["asset_kind"] or r["kind"],
                        "storage_uri": r["storage_uri"] or r["rel_path"],
                        "rel_path": r["rel_path"] or r["storage_uri"],
                        "storage_backend": r["storage_backend"],
                        "mime_type": r["mime_type"],
                        "width": r["width"],
                        "height": r["height"],
                        "sha256": r["sha256"],
                        "run_id": r["run_id"],
                        "job_id": r["job_id"],
                        "candidate_id": r["candidate_id"],
                        "metadata_json": self._json_cell(r["metadata_json"] or r["meta_json"], {}),
                        "created_at": r["created_at"],
                    }
                    for r in rows
                ]
                return self._json({"ok": True, "count": len(assets), "assets": assets})
            finally:
                conn.close()

        if len(parts) == 5 and parts[:2] == ["api", "projects"] and parts[3] == "assets":
            project_slug = parts[2]
            asset_id = parts[4]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                r = conn.execute(
                    """
                    SELECT *
                    FROM assets
                    WHERE id = ? AND project_id = ?
                    """,
                    (asset_id, project["id"]),
                ).fetchone()
                if not r:
                    return self._error(HTTPStatus.NOT_FOUND, "Asset not found")
                asset = {
                    "id": r["id"],
                    "project_id": r["project_id"],
                    "kind": r["kind"] or r["asset_kind"],
                    "asset_kind": r["asset_kind"] or r["kind"],
                    "storage_uri": r["storage_uri"] or r["rel_path"],
                    "rel_path": r["rel_path"] or r["storage_uri"],
                    "storage_backend": r["storage_backend"],
                    "mime_type": r["mime_type"],
                    "width": r["width"],
                    "height": r["height"],
                    "sha256": r["sha256"],
                    "run_id": r["run_id"],
                    "job_id": r["job_id"],
                    "candidate_id": r["candidate_id"],
                    "metadata_json": self._json_cell(r["metadata_json"] or r["meta_json"], {}),
                    "created_at": r["created_at"],
                }
                return self._json({"ok": True, "asset": asset})
            finally:
                conn.close()

        if len(parts) == 4 and parts[:2] == ["api", "projects"] and parts[3] == "quality-reports":
            project_slug = parts[2]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                try:
                    limit = int((query.get("limit", ["500"])[0] or "500").strip())
                except Exception:
                    limit = 500
                limit = max(1, min(limit, 2000))
                rows = conn.execute(
                    """
                    SELECT *
                    FROM quality_reports
                    WHERE project_id = ?
                    ORDER BY created_at DESC
                    LIMIT ?
                    """,
                    (project["id"], limit),
                ).fetchall()
                reports = [
                    {
                        "id": r["id"],
                        "project_id": r["project_id"],
                        "run_id": r["run_id"],
                        "job_id": r["job_id"] or r["run_job_id"],
                        "candidate_id": r["candidate_id"] or r["run_job_candidate_id"],
                        "report_type": r["report_type"],
                        "summary_json": self._json_cell(r["summary_json"], {}),
                        "rating": r["rating"],
                        "notes": r["notes"],
                        "created_at": r["created_at"],
                    }
                    for r in rows
                ]
                return self._json({"ok": True, "count": len(reports), "quality_reports": reports})
            finally:
                conn.close()

        if len(parts) == 4 and parts[:2] == ["api", "projects"] and parts[3] == "cost-events":
            project_slug = parts[2]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                try:
                    limit = int((query.get("limit", ["500"])[0] or "500").strip())
                except Exception:
                    limit = 500
                limit = max(1, min(limit, 2000))
                rows = conn.execute(
                    """
                    SELECT *
                    FROM cost_events
                    WHERE project_id = ?
                    ORDER BY created_at DESC
                    LIMIT ?
                    """,
                    (project["id"], limit),
                ).fetchall()
                events = [
                    {
                        "id": r["id"],
                        "project_id": r["project_id"],
                        "run_id": r["run_id"],
                        "provider_code": r["provider_code"],
                        "operation_code": r["operation_code"] or r["event_type"],
                        "units": r["units"],
                        "cost_usd": r["cost_usd"],
                        "currency": r["currency"],
                        "meta_json": self._json_cell(r["meta_json"], {}),
                        "amount_cents": r["amount_cents"],
                        "event_type": r["event_type"],
                        "notes": r["notes"],
                        "created_at": r["created_at"],
                    }
                    for r in rows
                ]
                return self._json({"ok": True, "count": len(events), "cost_events": events})
            finally:
                conn.close()

        if len(parts) == 4 and parts[:2] == ["api", "projects"] and parts[3] == "provider-accounts":
            project_slug = parts[2]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                rows = conn.execute(
                    """
                    SELECT *
                    FROM provider_accounts
                    WHERE project_id = ?
                    ORDER BY provider_code ASC, created_at DESC
                    """,
                    (project["id"],),
                ).fetchall()
                items = [self._coalesce_provider_account(r) for r in rows]
                return self._json({"ok": True, "count": len(items), "provider_accounts": items})
            finally:
                conn.close()

        if len(parts) == 5 and parts[:2] == ["api", "projects"] and parts[3] == "provider-accounts":
            project_slug = parts[2]
            provider_code = parts[4].strip().lower()
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                row = conn.execute(
                    """
                    SELECT *
                    FROM provider_accounts
                    WHERE project_id = ? AND provider_code = ?
                    """,
                    (project["id"], provider_code),
                ).fetchone()
                if not row:
                    return self._error(HTTPStatus.NOT_FOUND, "Provider account not found")
                return self._json({"ok": True, "provider_account": self._coalesce_provider_account(row)})
            finally:
                conn.close()

        if len(parts) == 4 and parts[:2] == ["api", "projects"] and parts[3] == "style-guides":
            project_slug = parts[2]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                rows = conn.execute(
                    """
                    SELECT *
                    FROM style_guides
                    WHERE project_id = ?
                    ORDER BY is_default DESC, created_at DESC
                    """,
                    (project["id"],),
                ).fetchall()
                items = [self._coalesce_style_guide(r) for r in rows]
                return self._json({"ok": True, "count": len(items), "style_guides": items})
            finally:
                conn.close()

        if len(parts) == 5 and parts[:2] == ["api", "projects"] and parts[3] == "style-guides":
            project_slug = parts[2]
            style_guide_id = parts[4]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                row = conn.execute(
                    """
                    SELECT *
                    FROM style_guides
                    WHERE id = ? AND project_id = ?
                    """,
                    (style_guide_id, project["id"]),
                ).fetchone()
                if not row:
                    return self._error(HTTPStatus.NOT_FOUND, "Style guide not found")
                return self._json({"ok": True, "style_guide": self._coalesce_style_guide(row)})
            finally:
                conn.close()

        if len(parts) == 4 and parts[:2] == ["api", "projects"] and parts[3] == "characters":
            project_slug = parts[2]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                rows = conn.execute(
                    """
                    SELECT *
                    FROM characters
                    WHERE project_id = ?
                    ORDER BY created_at DESC
                    """,
                    (project["id"],),
                ).fetchall()
                items = [self._coalesce_character(r) for r in rows]
                return self._json({"ok": True, "count": len(items), "characters": items})
            finally:
                conn.close()

        if len(parts) == 5 and parts[:2] == ["api", "projects"] and parts[3] == "characters":
            project_slug = parts[2]
            character_id = parts[4]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                row = conn.execute(
                    """
                    SELECT *
                    FROM characters
                    WHERE id = ? AND project_id = ?
                    """,
                    (character_id, project["id"]),
                ).fetchone()
                if not row:
                    return self._error(HTTPStatus.NOT_FOUND, "Character not found")
                return self._json({"ok": True, "character": self._coalesce_character(row)})
            finally:
                conn.close()

        if len(parts) == 4 and parts[:2] == ["api", "projects"] and parts[3] == "reference-sets":
            project_slug = parts[2]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                rows = conn.execute(
                    """
                    SELECT *
                    FROM reference_sets
                    WHERE project_id = ?
                    ORDER BY created_at DESC
                    """,
                    (project["id"],),
                ).fetchall()
                items = [self._coalesce_reference_set(r) for r in rows]
                return self._json({"ok": True, "count": len(items), "reference_sets": items})
            finally:
                conn.close()

        if len(parts) == 5 and parts[:2] == ["api", "projects"] and parts[3] == "reference-sets":
            project_slug = parts[2]
            reference_set_id = parts[4]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                row = conn.execute(
                    """
                    SELECT *
                    FROM reference_sets
                    WHERE id = ? AND project_id = ?
                    """,
                    (reference_set_id, project["id"]),
                ).fetchone()
                if not row:
                    return self._error(HTTPStatus.NOT_FOUND, "Reference set not found")
                return self._json({"ok": True, "reference_set": self._coalesce_reference_set(row)})
            finally:
                conn.close()

        if (
            len(parts) == 6
            and parts[:2] == ["api", "projects"]
            and parts[3] == "reference-sets"
            and parts[5] == "items"
        ):
            project_slug = parts[2]
            reference_set_id = parts[4]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                ref_set = conn.execute(
                    """
                    SELECT id
                    FROM reference_sets
                    WHERE id = ? AND project_id = ?
                    """,
                    (reference_set_id, project["id"]),
                ).fetchone()
                if not ref_set:
                    return self._error(HTTPStatus.NOT_FOUND, "Reference set not found")
                rows = conn.execute(
                    """
                    SELECT *
                    FROM reference_items
                    WHERE reference_set_id = ?
                    ORDER BY created_at ASC
                    """,
                    (reference_set_id,),
                ).fetchall()
                items = [self._coalesce_reference_item(r) for r in rows]
                return self._json({"ok": True, "count": len(items), "reference_items": items})
            finally:
                conn.close()

        if (
            len(parts) == 7
            and parts[:2] == ["api", "projects"]
            and parts[3] == "reference-sets"
            and parts[5] == "items"
        ):
            project_slug = parts[2]
            reference_set_id = parts[4]
            item_id = parts[6]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                row = conn.execute(
                    """
                    SELECT ri.*
                    FROM reference_items ri
                    JOIN reference_sets rs ON rs.id = ri.reference_set_id
                    WHERE ri.id = ? AND ri.reference_set_id = ? AND rs.project_id = ?
                    """,
                    (item_id, reference_set_id, project["id"]),
                ).fetchone()
                if not row:
                    return self._error(HTTPStatus.NOT_FOUND, "Reference item not found")
                return self._json({"ok": True, "reference_item": self._coalesce_reference_item(row)})
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
                    JOIN app_users u ON u.id = s.user_id
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
                user = conn.execute("SELECT username FROM app_users WHERE id = ?", (session["user_id"],)).fetchone()
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
                self._emit_audit_event(
                    conn,
                    project["id"],
                    "project.upserted",
                    {"slug": project["slug"], "name": project["name"], "source": "api"},
                    actor_user_id=user["id"],
                    target_type="project",
                    target_id=project["id"],
                )
                conn.commit()
                payload = be.project_storage_payload(self.server.repo_root, project, conn)
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
                self._emit_audit_event(
                    conn,
                    project["id"],
                    "secret.upserted",
                    {"provider_code": provider_code, "secret_name": secret_name, "source": "api"},
                    target_type="project_api_secret",
                    target_id=secret_id,
                )
                conn.commit()
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

        if len(parts) == 4 and parts[:2] == ["api", "projects"] and parts[3] == "provider-accounts":
            project_slug = parts[2]
            provider_code = str(body.get("provider_code", "")).strip().lower()
            if not provider_code:
                return self._error(HTTPStatus.BAD_REQUEST, "Field 'provider_code' is required")
            is_enabled = 1 if parse_bool(body.get("is_enabled"), True) else 0
            config_json = body.get("config_json", body.get("meta_json", {}))
            if not isinstance(config_json, dict):
                return self._error(HTTPStatus.BAD_REQUEST, "Field 'config_json' must be an object")
            api_key = str(body.get("api_key", "")).strip()
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                existing = conn.execute(
                    """
                    SELECT *
                    FROM provider_accounts
                    WHERE project_id = ? AND provider_code = ?
                    """,
                    (project["id"], provider_code),
                ).fetchone()
                now = be.now_iso()
                if existing:
                    next_api_key = api_key if api_key else existing["api_key"]
                    conn.execute(
                        """
                        UPDATE provider_accounts
                        SET is_enabled = ?, config_json = ?, meta_json = ?, api_key = ?, updated_at = ?
                        WHERE id = ?
                        """,
                        (is_enabled, be.to_json(config_json), be.to_json(config_json), next_api_key, now, existing["id"]),
                    )
                    provider_account_id = existing["id"]
                else:
                    provider_account_id = be.uid()
                    conn.execute(
                        """
                        INSERT INTO provider_accounts
                          (id, project_id, provider_code, is_enabled, config_json, meta_json, api_key, created_at, updated_at)
                        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                        """,
                        (
                            provider_account_id,
                            project["id"],
                            provider_code,
                            is_enabled,
                            be.to_json(config_json),
                            be.to_json(config_json),
                            api_key,
                            now,
                            now,
                        ),
                    )
                self._emit_audit_event(
                    conn,
                    project["id"],
                    "provider_account.upserted",
                    {"provider_account_id": provider_account_id, "provider_code": provider_code, "source": "api"},
                    target_type="provider_account",
                    target_id=provider_account_id,
                )
                conn.commit()
                row = conn.execute("SELECT * FROM provider_accounts WHERE id = ?", (provider_account_id,)).fetchone()
                return self._json({"ok": True, "provider_account": self._coalesce_provider_account(row)})
            finally:
                conn.close()

        if len(parts) == 4 and parts[:2] == ["api", "projects"] and parts[3] == "style-guides":
            project_slug = parts[2]
            name = str(body.get("name", "")).strip()
            if not name:
                return self._error(HTTPStatus.BAD_REQUEST, "Field 'name' is required")
            description = str(body.get("description", "")).strip()
            rules_json = body.get("rules_json", body.get("specs_json", {}))
            if not isinstance(rules_json, dict):
                return self._error(HTTPStatus.BAD_REQUEST, "Field 'rules_json' must be an object")
            is_default = 1 if parse_bool(body.get("is_default"), False) else 0
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                style_guide_id = be.uid()
                now = be.now_iso()
                conn.execute(
                    """
                    INSERT INTO style_guides
                      (id, project_id, name, description, specs_json, rules_json, is_default, created_at, updated_at)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                    """,
                    (
                        style_guide_id,
                        project["id"],
                        name,
                        description,
                        be.to_json(rules_json),
                        be.to_json(rules_json),
                        is_default,
                        now,
                        now,
                    ),
                )
                self._emit_audit_event(
                    conn,
                    project["id"],
                    "style_guide.created",
                    {"style_guide_id": style_guide_id, "name": name, "source": "api"},
                    target_type="style_guide",
                    target_id=style_guide_id,
                )
                conn.commit()
                row = conn.execute("SELECT * FROM style_guides WHERE id = ?", (style_guide_id,)).fetchone()
                return self._json({"ok": True, "style_guide": self._coalesce_style_guide(row)})
            finally:
                conn.close()

        if len(parts) == 4 and parts[:2] == ["api", "projects"] and parts[3] == "characters":
            project_slug = parts[2]
            code = str(body.get("code", "")).strip()
            name = str(body.get("name", "")).strip()
            if not code or not name:
                return self._error(HTTPStatus.BAD_REQUEST, "Fields 'code' and 'name' are required")
            bio = str(body.get("bio", "")).strip()
            constraints = body.get("identity_constraints_json", {})
            if not isinstance(constraints, dict):
                return self._error(HTTPStatus.BAD_REQUEST, "Field 'identity_constraints_json' must be an object")
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                character_id = be.uid()
                now = be.now_iso()
                conn.execute(
                    """
                    INSERT INTO characters
                      (id, project_id, code, name, bio, identity_constraints_json, created_at, updated_at)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                    """,
                    (character_id, project["id"], code, name, bio, be.to_json(constraints), now, now),
                )
                self._emit_audit_event(
                    conn,
                    project["id"],
                    "character.created",
                    {"character_id": character_id, "code": code, "name": name, "source": "api"},
                    target_type="character",
                    target_id=character_id,
                )
                conn.commit()
                row = conn.execute("SELECT * FROM characters WHERE id = ?", (character_id,)).fetchone()
                return self._json({"ok": True, "character": self._coalesce_character(row)})
            finally:
                conn.close()

        if len(parts) == 4 and parts[:2] == ["api", "projects"] and parts[3] == "reference-sets":
            project_slug = parts[2]
            name = str(body.get("name", body.get("title", ""))).strip()
            if not name:
                return self._error(HTTPStatus.BAD_REQUEST, "Field 'name' is required")
            kind = str(body.get("kind", "other")).strip().lower() or "other"
            metadata_json = body.get("metadata_json", {})
            if not isinstance(metadata_json, dict):
                return self._error(HTTPStatus.BAD_REQUEST, "Field 'metadata_json' must be an object")
            notes = str(body.get("notes", "")).strip()
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                reference_set_id = be.uid()
                now = be.now_iso()
                conn.execute(
                    """
                    INSERT INTO reference_sets
                      (id, project_id, title, name, kind, notes, metadata_json, created_at, updated_at)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                    """,
                    (
                        reference_set_id,
                        project["id"],
                        name,
                        name,
                        kind,
                        notes,
                        be.to_json(metadata_json),
                        now,
                        now,
                    ),
                )
                self._emit_audit_event(
                    conn,
                    project["id"],
                    "reference_set.created",
                    {"reference_set_id": reference_set_id, "name": name, "kind": kind, "source": "api"},
                    target_type="reference_set",
                    target_id=reference_set_id,
                )
                conn.commit()
                row = conn.execute("SELECT * FROM reference_sets WHERE id = ?", (reference_set_id,)).fetchone()
                return self._json({"ok": True, "reference_set": self._coalesce_reference_set(row)})
            finally:
                conn.close()

        if (
            len(parts) == 6
            and parts[:2] == ["api", "projects"]
            and parts[3] == "reference-sets"
            and parts[5] == "items"
        ):
            project_slug = parts[2]
            reference_set_id = parts[4]
            asset_id = str(body.get("asset_id", "")).strip()
            if not asset_id:
                return self._error(HTTPStatus.BAD_REQUEST, "Field 'asset_id' is required")
            notes = str(body.get("notes", "")).strip()
            weight_raw = body.get("weight", 1.0)
            try:
                weight = float(weight_raw)
            except Exception:
                return self._error(HTTPStatus.BAD_REQUEST, "Field 'weight' must be a number")
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                ref_set = conn.execute(
                    """
                    SELECT id
                    FROM reference_sets
                    WHERE id = ? AND project_id = ?
                    """,
                    (reference_set_id, project["id"]),
                ).fetchone()
                if not ref_set:
                    return self._error(HTTPStatus.NOT_FOUND, "Reference set not found")
                asset = conn.execute(
                    """
                    SELECT id
                    FROM assets
                    WHERE id = ? AND project_id = ?
                    """,
                    (asset_id, project["id"]),
                ).fetchone()
                if not asset:
                    return self._error(HTTPStatus.BAD_REQUEST, "asset_id does not belong to this project")

                existing = conn.execute(
                    """
                    SELECT *
                    FROM reference_items
                    WHERE reference_set_id = ? AND asset_id = ?
                    """,
                    (reference_set_id, asset_id),
                ).fetchone()
                now = be.now_iso()
                if existing:
                    conn.execute(
                        """
                        UPDATE reference_items
                        SET weight = ?, notes = ?
                        WHERE id = ?
                        """,
                        (weight, notes, existing["id"]),
                    )
                    item_id = existing["id"]
                else:
                    item_id = be.uid()
                    conn.execute(
                        """
                        INSERT INTO reference_items
                          (id, reference_set_id, asset_id, weight, notes, created_at)
                        VALUES (?, ?, ?, ?, ?, ?)
                        """,
                        (item_id, reference_set_id, asset_id, weight, notes, now),
                    )
                self._emit_audit_event(
                    conn,
                    project["id"],
                    "reference_item.upserted",
                    {"reference_set_id": reference_set_id, "item_id": item_id, "asset_id": asset_id, "source": "api"},
                    target_type="reference_item",
                    target_id=item_id,
                )
                conn.commit()
                row = conn.execute("SELECT * FROM reference_items WHERE id = ?", (item_id,)).fetchone()
                return self._json({"ok": True, "reference_item": self._coalesce_reference_item(row)})
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
                self._emit_audit_event(
                    conn,
                    project["id"],
                    "instruction.created",
                    {"instruction_id": instruction_id, "instruction_type": instruction_type, "status": initial_status},
                    target_type="agent_instruction",
                    target_id=instruction_id,
                )
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
                self._emit_audit_event(
                    conn,
                    project["id"],
                    "instruction.confirmed",
                    {"instruction_id": instruction_id},
                    actor_user_id=user["id"],
                    target_type="agent_instruction",
                    target_id=instruction_id,
                )
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
                self._emit_audit_event(
                    conn,
                    project["id"],
                    "instruction.canceled",
                    {"instruction_id": instruction_id, "reason": reason},
                    target_type="agent_instruction",
                    target_id=instruction_id,
                )
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
                self._emit_audit_event(
                    conn,
                    project["id"],
                    "run.ingest.request",
                    {"run_id": result.get("run_id"), "run_log": run_log, "source": "api"},
                    target_type="run",
                    target_id=result.get("run_id"),
                )
                conn.commit()
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
                storage_payload = be.project_storage_payload(self.server.repo_root, project, conn)
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
                self._emit_audit_event(
                    conn,
                    project["id"],
                    "project.export.request",
                    {"export_asset_id": result.get("export_asset_id"), "output": str(output_path), "source": "api"},
                    target_type="project_export",
                    target_id=result.get("export_asset_id"),
                )
                conn.commit()
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
                payload = be.project_storage_payload(self.server.repo_root, project, conn)
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
                conn = self._db()
                try:
                    project = be.get_project(conn, "", project_slug)
                    if project:
                        self._emit_audit_event(
                            conn,
                            project["id"],
                            "storage.s3.sync_requested",
                            {
                                "dry_run": dry_run,
                                "delete": delete,
                                "allow_missing_local": allow_missing_local,
                                "source": "api",
                            },
                            target_type="project_storage",
                            target_id=project["id"],
                        )
                        conn.commit()
                finally:
                    conn.close()
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

        if len(parts) == 5 and parts[:2] == ["api", "projects"] and parts[3] == "style-guides":
            project_slug = parts[2]
            style_guide_id = parts[4]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                row = conn.execute(
                    "SELECT * FROM style_guides WHERE id = ? AND project_id = ?",
                    (style_guide_id, project["id"]),
                ).fetchone()
                if not row:
                    return self._error(HTTPStatus.NOT_FOUND, "Style guide not found")
                name = str(body.get("name", row["name"])).strip()
                description = str(body.get("description", row["description"])).strip()
                rules_json = body.get("rules_json")
                if rules_json is None:
                    rules_json = self._json_cell(row["rules_json"] or row["specs_json"], {})
                if not isinstance(rules_json, dict):
                    return self._error(HTTPStatus.BAD_REQUEST, "Field 'rules_json' must be an object")
                is_default = 1 if parse_bool(body.get("is_default"), bool(row["is_default"] or 0)) else 0
                now = be.now_iso()
                conn.execute(
                    """
                    UPDATE style_guides
                    SET name = ?, description = ?, specs_json = ?, rules_json = ?, is_default = ?, updated_at = ?
                    WHERE id = ?
                    """,
                    (name, description, be.to_json(rules_json), be.to_json(rules_json), is_default, now, style_guide_id),
                )
                self._emit_audit_event(
                    conn,
                    project["id"],
                    "style_guide.updated",
                    {"style_guide_id": style_guide_id, "source": "api"},
                    target_type="style_guide",
                    target_id=style_guide_id,
                )
                conn.commit()
                refreshed = conn.execute("SELECT * FROM style_guides WHERE id = ?", (style_guide_id,)).fetchone()
                return self._json({"ok": True, "style_guide": self._coalesce_style_guide(refreshed)})
            finally:
                conn.close()

        if len(parts) == 5 and parts[:2] == ["api", "projects"] and parts[3] == "provider-accounts":
            project_slug = parts[2]
            provider_code = parts[4].strip().lower()
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                row = conn.execute(
                    """
                    SELECT *
                    FROM provider_accounts
                    WHERE project_id = ? AND provider_code = ?
                    """,
                    (project["id"], provider_code),
                ).fetchone()
                if not row:
                    return self._error(HTTPStatus.NOT_FOUND, "Provider account not found")
                is_enabled = 1 if parse_bool(body.get("is_enabled"), bool(row["is_enabled"] or 0)) else 0
                config_json = body.get("config_json")
                if config_json is None:
                    config_json = self._json_cell(row["config_json"] or row["meta_json"], {})
                if not isinstance(config_json, dict):
                    return self._error(HTTPStatus.BAD_REQUEST, "Field 'config_json' must be an object")
                api_key = body.get("api_key")
                next_api_key = row["api_key"] if api_key is None else str(api_key).strip()
                now = be.now_iso()
                conn.execute(
                    """
                    UPDATE provider_accounts
                    SET is_enabled = ?, config_json = ?, meta_json = ?, api_key = ?, updated_at = ?
                    WHERE id = ?
                    """,
                    (is_enabled, be.to_json(config_json), be.to_json(config_json), next_api_key, now, row["id"]),
                )
                self._emit_audit_event(
                    conn,
                    project["id"],
                    "provider_account.updated",
                    {"provider_account_id": row["id"], "provider_code": provider_code, "source": "api"},
                    target_type="provider_account",
                    target_id=row["id"],
                )
                conn.commit()
                refreshed = conn.execute("SELECT * FROM provider_accounts WHERE id = ?", (row["id"],)).fetchone()
                return self._json({"ok": True, "provider_account": self._coalesce_provider_account(refreshed)})
            finally:
                conn.close()

        if len(parts) == 5 and parts[:2] == ["api", "projects"] and parts[3] == "characters":
            project_slug = parts[2]
            character_id = parts[4]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                row = conn.execute(
                    "SELECT * FROM characters WHERE id = ? AND project_id = ?",
                    (character_id, project["id"]),
                ).fetchone()
                if not row:
                    return self._error(HTTPStatus.NOT_FOUND, "Character not found")
                code = str(body.get("code", row["code"])).strip()
                name = str(body.get("name", row["name"])).strip()
                bio = str(body.get("bio", row["bio"])).strip()
                constraints = body.get("identity_constraints_json")
                if constraints is None:
                    constraints = self._json_cell(row["identity_constraints_json"], {})
                if not isinstance(constraints, dict):
                    return self._error(HTTPStatus.BAD_REQUEST, "Field 'identity_constraints_json' must be an object")
                now = be.now_iso()
                conn.execute(
                    """
                    UPDATE characters
                    SET code = ?, name = ?, bio = ?, identity_constraints_json = ?, updated_at = ?
                    WHERE id = ?
                    """,
                    (code, name, bio, be.to_json(constraints), now, character_id),
                )
                self._emit_audit_event(
                    conn,
                    project["id"],
                    "character.updated",
                    {"character_id": character_id, "source": "api"},
                    target_type="character",
                    target_id=character_id,
                )
                conn.commit()
                refreshed = conn.execute("SELECT * FROM characters WHERE id = ?", (character_id,)).fetchone()
                return self._json({"ok": True, "character": self._coalesce_character(refreshed)})
            finally:
                conn.close()

        if len(parts) == 5 and parts[:2] == ["api", "projects"] and parts[3] == "reference-sets":
            project_slug = parts[2]
            reference_set_id = parts[4]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                row = conn.execute(
                    "SELECT * FROM reference_sets WHERE id = ? AND project_id = ?",
                    (reference_set_id, project["id"]),
                ).fetchone()
                if not row:
                    return self._error(HTTPStatus.NOT_FOUND, "Reference set not found")
                name = str(body.get("name", row["name"] or row["title"])).strip()
                kind = str(body.get("kind", row["kind"] or "other")).strip().lower() or "other"
                notes = str(body.get("notes", row["notes"] or "")).strip()
                metadata_json = body.get("metadata_json")
                if metadata_json is None:
                    metadata_json = self._json_cell(row["metadata_json"], {})
                if not isinstance(metadata_json, dict):
                    return self._error(HTTPStatus.BAD_REQUEST, "Field 'metadata_json' must be an object")
                now = be.now_iso()
                conn.execute(
                    """
                    UPDATE reference_sets
                    SET title = ?, name = ?, kind = ?, notes = ?, metadata_json = ?, updated_at = ?
                    WHERE id = ?
                    """,
                    (name, name, kind, notes, be.to_json(metadata_json), now, reference_set_id),
                )
                self._emit_audit_event(
                    conn,
                    project["id"],
                    "reference_set.updated",
                    {"reference_set_id": reference_set_id, "source": "api"},
                    target_type="reference_set",
                    target_id=reference_set_id,
                )
                conn.commit()
                refreshed = conn.execute("SELECT * FROM reference_sets WHERE id = ?", (reference_set_id,)).fetchone()
                return self._json({"ok": True, "reference_set": self._coalesce_reference_set(refreshed)})
            finally:
                conn.close()

        if (
            len(parts) == 7
            and parts[:2] == ["api", "projects"]
            and parts[3] == "reference-sets"
            and parts[5] == "items"
        ):
            project_slug = parts[2]
            reference_set_id = parts[4]
            item_id = parts[6]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                row = conn.execute(
                    """
                    SELECT ri.*
                    FROM reference_items ri
                    JOIN reference_sets rs ON rs.id = ri.reference_set_id
                    WHERE ri.id = ? AND ri.reference_set_id = ? AND rs.project_id = ?
                    """,
                    (item_id, reference_set_id, project["id"]),
                ).fetchone()
                if not row:
                    return self._error(HTTPStatus.NOT_FOUND, "Reference item not found")
                asset_id = str(body.get("asset_id", row["asset_id"])).strip()
                notes = str(body.get("notes", row["notes"] or "")).strip()
                weight_raw = body.get("weight", row["weight"] if row["weight"] is not None else 1.0)
                try:
                    weight = float(weight_raw)
                except Exception:
                    return self._error(HTTPStatus.BAD_REQUEST, "Field 'weight' must be a number")
                asset = conn.execute(
                    """
                    SELECT id
                    FROM assets
                    WHERE id = ? AND project_id = ?
                    """,
                    (asset_id, project["id"]),
                ).fetchone()
                if not asset:
                    return self._error(HTTPStatus.BAD_REQUEST, "asset_id does not belong to this project")
                conn.execute(
                    """
                    UPDATE reference_items
                    SET asset_id = ?, weight = ?, notes = ?
                    WHERE id = ?
                    """,
                    (asset_id, weight, notes, item_id),
                )
                self._emit_audit_event(
                    conn,
                    project["id"],
                    "reference_item.updated",
                    {"reference_set_id": reference_set_id, "item_id": item_id, "asset_id": asset_id, "source": "api"},
                    target_type="reference_item",
                    target_id=item_id,
                )
                conn.commit()
                refreshed = conn.execute("SELECT * FROM reference_items WHERE id = ?", (item_id,)).fetchone()
                return self._json({"ok": True, "reference_item": self._coalesce_reference_item(refreshed)})
            finally:
                conn.close()

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
                self._emit_audit_event(
                    conn,
                    project["id"],
                    "storage.local.updated",
                    {"base_dir": local.get("base_dir"), "project_root": local.get("project_root"), "source": "api"},
                    target_type="project_storage",
                    target_id=project["id"],
                )
                conn.commit()
                payload = be.project_storage_payload(self.server.repo_root, refreshed, conn)
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
                self._emit_audit_event(
                    conn,
                    project["id"],
                    "storage.s3.updated",
                    {
                        "enabled": bool(s3.get("enabled")),
                        "bucket": s3.get("bucket"),
                        "prefix": s3.get("prefix"),
                        "region": s3.get("region"),
                        "profile": s3.get("profile"),
                        "endpoint_url": s3.get("endpoint_url"),
                        "source": "api",
                    },
                    target_type="project_storage",
                    target_id=project["id"],
                )
                conn.commit()
                payload = be.project_storage_payload(self.server.repo_root, refreshed, conn)
                return self._json({"ok": True, "updated": "s3", **payload})
            finally:
                conn.close()

        return self._error(HTTPStatus.NOT_FOUND, "Route not found")

    def do_DELETE(self):
        parsed = urlparse(self.path)
        parts = [p for p in parsed.path.split("/") if p]

        if len(parts) == 5 and parts[:2] == ["api", "projects"] and parts[3] == "style-guides":
            project_slug = parts[2]
            style_guide_id = parts[4]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                cur = conn.execute(
                    """
                    DELETE FROM style_guides
                    WHERE id = ? AND project_id = ?
                    """,
                    (style_guide_id, project["id"]),
                )
                self._emit_audit_event(
                    conn,
                    project["id"],
                    "style_guide.deleted",
                    {"style_guide_id": style_guide_id, "deleted": cur.rowcount, "source": "api"},
                    target_type="style_guide",
                    target_id=style_guide_id,
                )
                conn.commit()
                return self._json({"ok": True, "style_guide_id": style_guide_id, "deleted": cur.rowcount})
            finally:
                conn.close()

        if len(parts) == 5 and parts[:2] == ["api", "projects"] and parts[3] == "provider-accounts":
            project_slug = parts[2]
            provider_code = parts[4].strip().lower()
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                row = conn.execute(
                    """
                    SELECT id
                    FROM provider_accounts
                    WHERE project_id = ? AND provider_code = ?
                    """,
                    (project["id"], provider_code),
                ).fetchone()
                provider_account_id = row["id"] if row else None
                cur = conn.execute(
                    """
                    DELETE FROM provider_accounts
                    WHERE project_id = ? AND provider_code = ?
                    """,
                    (project["id"], provider_code),
                )
                self._emit_audit_event(
                    conn,
                    project["id"],
                    "provider_account.deleted",
                    {
                        "provider_account_id": provider_account_id,
                        "provider_code": provider_code,
                        "deleted": cur.rowcount,
                        "source": "api",
                    },
                    target_type="provider_account",
                    target_id=provider_account_id,
                )
                conn.commit()
                return self._json({"ok": True, "provider_code": provider_code, "deleted": cur.rowcount})
            finally:
                conn.close()

        if len(parts) == 5 and parts[:2] == ["api", "projects"] and parts[3] == "characters":
            project_slug = parts[2]
            character_id = parts[4]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                cur = conn.execute(
                    """
                    DELETE FROM characters
                    WHERE id = ? AND project_id = ?
                    """,
                    (character_id, project["id"]),
                )
                self._emit_audit_event(
                    conn,
                    project["id"],
                    "character.deleted",
                    {"character_id": character_id, "deleted": cur.rowcount, "source": "api"},
                    target_type="character",
                    target_id=character_id,
                )
                conn.commit()
                return self._json({"ok": True, "character_id": character_id, "deleted": cur.rowcount})
            finally:
                conn.close()

        if len(parts) == 5 and parts[:2] == ["api", "projects"] and parts[3] == "reference-sets":
            project_slug = parts[2]
            reference_set_id = parts[4]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                cur = conn.execute(
                    """
                    DELETE FROM reference_sets
                    WHERE id = ? AND project_id = ?
                    """,
                    (reference_set_id, project["id"]),
                )
                self._emit_audit_event(
                    conn,
                    project["id"],
                    "reference_set.deleted",
                    {"reference_set_id": reference_set_id, "deleted": cur.rowcount, "source": "api"},
                    target_type="reference_set",
                    target_id=reference_set_id,
                )
                conn.commit()
                return self._json({"ok": True, "reference_set_id": reference_set_id, "deleted": cur.rowcount})
            finally:
                conn.close()

        if (
            len(parts) == 7
            and parts[:2] == ["api", "projects"]
            and parts[3] == "reference-sets"
            and parts[5] == "items"
        ):
            project_slug = parts[2]
            reference_set_id = parts[4]
            item_id = parts[6]
            conn = self._db()
            try:
                project = self._project_or_404(conn, project_slug)
                if not project:
                    return
                cur = conn.execute(
                    """
                    DELETE FROM reference_items
                    WHERE id = ?
                      AND reference_set_id = ?
                      AND reference_set_id IN (SELECT id FROM reference_sets WHERE project_id = ?)
                    """,
                    (item_id, reference_set_id, project["id"]),
                )
                self._emit_audit_event(
                    conn,
                    project["id"],
                    "reference_item.deleted",
                    {"reference_set_id": reference_set_id, "item_id": item_id, "deleted": cur.rowcount, "source": "api"},
                    target_type="reference_item",
                    target_id=item_id,
                )
                conn.commit()
                return self._json({"ok": True, "item_id": item_id, "deleted": cur.rowcount})
            finally:
                conn.close()

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
                self._emit_audit_event(
                    conn,
                    project["id"],
                    "secret.deleted",
                    {"provider_code": provider_code, "secret_name": secret_name, "deleted": cur.rowcount, "source": "api"},
                    target_type="project_api_secret",
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
