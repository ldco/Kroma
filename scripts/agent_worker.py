#!/usr/bin/env python3
import argparse
import datetime as dt
import json
import os
import time
from pathlib import Path

if os.environ.get("KROMA_ENABLE_LEGACY_SCRIPTS", "").strip().lower() not in {"1", "true", "yes", "on"}:
    raise SystemExit(
        "Legacy worker disabled. Use the Rust backend/runtime (src-tauri). "
        "Set KROMA_ENABLE_LEGACY_SCRIPTS=1 only for explicit migration fallback."
    )

import backend as be
from agent_dispatch import dispatch_instruction_http


def now_dt() -> dt.datetime:
    return dt.datetime.now(dt.UTC).replace(microsecond=0)


def iso(dt_value: dt.datetime) -> str:
    return dt_value.isoformat().replace("+00:00", "Z")


def parse_dt(value: str | None) -> dt.datetime | None:
    if not value:
        return None
    raw = str(value).strip()
    if raw.endswith("Z"):
        raw = raw[:-1] + "+00:00"
    return dt.datetime.fromisoformat(raw)


def emit_event(conn, instruction_id: str, event_type: str, payload: dict):
    conn.execute(
        """
        INSERT INTO agent_instruction_events (id, instruction_id, event_type, event_payload_json, created_at)
        VALUES (?, ?, ?, ?, ?)
        """,
        (be.uid(), instruction_id, event_type, be.to_json(payload or {}), be.now_iso()),
    )


def reserve_next_instruction(conn, worker_id: str, max_locked_seconds: int):
    now = now_dt()
    lock_cutoff = iso(now - dt.timedelta(seconds=max_locked_seconds))
    now_iso = iso(now)

    conn.execute("BEGIN IMMEDIATE")
    row = conn.execute(
        """
        SELECT *
        FROM agent_instructions
        WHERE status = 'queued'
          AND (next_attempt_at IS NULL OR next_attempt_at <= ?)
          AND (locked_at IS NULL OR locked_at <= ?)
        ORDER BY priority ASC, created_at ASC
        LIMIT 1
        """,
        (now_iso, lock_cutoff),
    ).fetchone()
    if not row:
        conn.execute("COMMIT")
        return None

    cur = conn.execute(
        """
        UPDATE agent_instructions
        SET status = 'running',
            started_at = COALESCE(started_at, ?),
            updated_at = ?,
            locked_by = ?,
            locked_at = ?,
            next_attempt_at = NULL
        WHERE id = ? AND status = 'queued'
        """,
        (now_iso, now_iso, worker_id, now_iso, row["id"]),
    )
    if cur.rowcount != 1:
        conn.execute("COMMIT")
        return None
    conn.execute("COMMIT")
    return conn.execute("SELECT * FROM agent_instructions WHERE id = ?", (row["id"],)).fetchone()


def resolve_agent_target(conn, repo_root: Path, project_row):
    url = (os.environ.get("IAT_AGENT_API_URL") or "").strip()
    token = (os.environ.get("IAT_AGENT_API_TOKEN") or "").strip()

    if not url:
        try:
            secret_url = be.fetch_project_secret_value(conn, repo_root, project_row["id"], "agent_api", "url")
            if secret_url:
                url = secret_url.strip()
        except Exception:
            pass
    if not token:
        try:
            secret_token = be.fetch_project_secret_value(conn, repo_root, project_row["id"], "agent_api", "token")
            if secret_token:
                token = secret_token.strip()
        except Exception:
            pass
    return url, token


def map_remote_status(remote_status: str):
    val = str(remote_status or "").strip().lower()
    if val in {"done", "failed", "running"}:
        return val
    if val in {"accepted", "queued"}:
        return "done"
    return "done"


def process_instruction(conn, repo_root: Path, instruction_row, args):
    instruction_id = instruction_row["id"]
    now_iso = be.now_iso()
    emit_event(conn, instruction_id, "started", {"worker_id": args.worker_id, "started_at": now_iso})
    conn.commit()

    project = conn.execute("SELECT * FROM projects WHERE id = ?", (instruction_row["project_id"],)).fetchone()
    if not project:
        conn.execute(
            """
            UPDATE agent_instructions
            SET status = 'failed', finished_at = ?, updated_at = ?, last_error = ?, locked_by = NULL, locked_at = NULL
            WHERE id = ?
            """,
            (now_iso, now_iso, "project_not_found", instruction_id),
        )
        emit_event(conn, instruction_id, "error", {"message": "Project not found"})
        conn.commit()
        return

    payload = json.loads(instruction_row["payload_json"] or "{}")
    objective = str(payload.get("objective", "")).strip()
    remote_payload = {
        "instruction_id": instruction_id,
        "project_slug": project["slug"],
        "instruction_type": instruction_row["instruction_type"],
        "objective": objective or f"Execute {instruction_row['instruction_type']}",
        "constraints": payload.get("constraints", {}),
        "inputs": payload.get("inputs", {}),
        "execution": payload.get("execution", {}),
        "confirmation_required": bool(instruction_row["requires_confirmation"]),
        "requested_by": payload.get("requested_by", "local"),
        "callback": payload.get("callback", {}),
        "payload": payload,
    }

    target_url, token = resolve_agent_target(conn, repo_root, project)
    if not target_url:
        attempts = int(instruction_row["attempts"] or 0) + 1
        max_attempts = int(instruction_row["max_attempts"] or args.default_max_attempts)
        retryable = attempts < max_attempts
        next_attempt_at = (
            iso(now_dt() + dt.timedelta(seconds=args.retry_backoff_seconds * attempts)) if retryable else None
        )
        new_status = "queued" if retryable else "failed"
        conn.execute(
            """
            UPDATE agent_instructions
            SET status = ?,
                attempts = ?,
                max_attempts = ?,
                next_attempt_at = ?,
                finished_at = CASE WHEN ? = 'failed' THEN ? ELSE finished_at END,
                updated_at = ?,
                last_error = ?,
                locked_by = NULL,
                locked_at = NULL
            WHERE id = ?
            """,
            (new_status, attempts, max_attempts, next_attempt_at, new_status, now_iso, now_iso, "missing_agent_api_url", instruction_id),
        )
        emit_event(
            conn,
            instruction_id,
            "error" if not retryable else "retry_scheduled",
            {
                "message": "Agent API URL is not configured",
                "attempts": attempts,
                "max_attempts": max_attempts,
                "next_attempt_at": next_attempt_at,
            },
        )
        conn.commit()
        return

    dispatch = dispatch_instruction_http(
        target_url=target_url,
        token=token or None,
        payload=remote_payload,
        timeout_sec=args.dispatch_timeout,
        retries=args.dispatch_retries,
        backoff_sec=args.dispatch_backoff_seconds,
    )
    attempts = int(instruction_row["attempts"] or 0) + 1
    max_attempts = int(instruction_row["max_attempts"] or args.default_max_attempts)

    if dispatch.get("ok"):
        response = dispatch.get("response") if isinstance(dispatch.get("response"), dict) else {}
        remote_status = map_remote_status(response.get("status", "done"))
        finished_at = be.now_iso() if remote_status in {"done", "failed"} else None
        conn.execute(
            """
            UPDATE agent_instructions
            SET status = ?,
                attempts = ?,
                max_attempts = ?,
                agent_response_json = ?,
                finished_at = COALESCE(?, finished_at),
                updated_at = ?,
                last_error = NULL,
                locked_by = NULL,
                locked_at = NULL
            WHERE id = ?
            """,
            (remote_status, attempts, max_attempts, be.to_json(response), finished_at, be.now_iso(), instruction_id),
        )
        emit_event(
            conn,
            instruction_id,
            "result",
            {
                "remote_status": remote_status,
                "attempts": attempts,
                "http_status": dispatch.get("http_status"),
                "response": response,
            },
        )
        conn.commit()
        return

    retryable = attempts < max_attempts
    next_attempt_at = iso(now_dt() + dt.timedelta(seconds=args.retry_backoff_seconds * attempts)) if retryable else None
    new_status = "queued" if retryable else "failed"
    err = str(dispatch.get("error", "dispatch_failed"))
    conn.execute(
        """
        UPDATE agent_instructions
        SET status = ?,
            attempts = ?,
            max_attempts = ?,
            next_attempt_at = ?,
            finished_at = CASE WHEN ? = 'failed' THEN ? ELSE finished_at END,
            updated_at = ?,
            last_error = ?,
            locked_by = NULL,
            locked_at = NULL
        WHERE id = ?
        """,
        (new_status, attempts, max_attempts, next_attempt_at, new_status, be.now_iso(), be.now_iso(), err, instruction_id),
    )
    emit_event(
        conn,
        instruction_id,
        "error" if not retryable else "retry_scheduled",
        {"error": err, "attempts": attempts, "max_attempts": max_attempts, "next_attempt_at": next_attempt_at},
    )
    conn.commit()


def run_loop(args):
    repo_root = Path.cwd().resolve()
    db_path = (repo_root / args.db).resolve()
    conn = be.connect_db(db_path)
    be.init_schema(conn)
    processed = 0
    try:
        while True:
            row = reserve_next_instruction(conn, args.worker_id, args.max_locked_seconds)
            if not row:
                if args.once:
                    break
                time.sleep(args.poll_interval_seconds)
                continue
            process_instruction(conn, repo_root, row, args)
            processed += 1
            if args.once and processed >= 1:
                break
    finally:
        conn.close()
    print(be.to_json({"ok": True, "worker_id": args.worker_id, "processed": processed, "db": str(db_path)}))


def build_parser():
    parser = argparse.ArgumentParser(description="IAT agent instruction queue worker")
    parser.add_argument("--db", default=be.DEFAULT_DB, help=f"SQLite DB path (default: {be.DEFAULT_DB})")
    parser.add_argument("--worker-id", default=f"worker-{be.uid()[:8]}", help="Worker id for lock tracking")
    parser.add_argument("--once", action="store_true", help="Process at most one queued instruction and exit")
    parser.add_argument("--poll-interval-seconds", type=float, default=2.0, help="Polling interval")
    parser.add_argument("--max-locked-seconds", type=int, default=120, help="Stale lock timeout")
    parser.add_argument("--default-max-attempts", type=int, default=3, help="Default retry attempts")
    parser.add_argument("--retry-backoff-seconds", type=int, default=10, help="Base retry backoff seconds")
    parser.add_argument("--dispatch-timeout", type=float, default=20.0, help="HTTP dispatch timeout")
    parser.add_argument("--dispatch-retries", type=int, default=2, help="HTTP dispatch retries")
    parser.add_argument("--dispatch-backoff-seconds", type=float, default=1.5, help="HTTP dispatch backoff")
    return parser


def main():
    args = build_parser().parse_args()
    run_loop(args)


if __name__ == "__main__":
    main()
