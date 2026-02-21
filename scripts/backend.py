#!/usr/bin/env python3
import argparse
import base64
import datetime as dt
import hashlib
import json
import os
import re
import shutil
import sqlite3
import subprocess
import tarfile
import tempfile
import time
import uuid
from pathlib import Path

from cryptography.fernet import Fernet, InvalidToken


DEFAULT_DB = "generated/backend/app.db"
DEFAULT_MASTER_KEY_FILE = "generated/backend/master.key"
DEFAULT_SECRET_SERVICE = "iat-toolkit"
DEFAULT_SECRET_ACCOUNT = "backend-master-key"


def now_iso() -> str:
    return dt.datetime.now(dt.UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def uid() -> str:
    return str(uuid.uuid4())


def to_json(value) -> str:
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"))


def read_json(path: Path):
    with path.open("r", encoding="utf-8") as fh:
        return json.load(fh)


def normalize_rel_path(value: str) -> str:
    return str(value or "").replace("\\", "/").strip()


def path_for_storage(path: Path, repo_root: Path) -> str:
    try:
        return normalize_rel_path(str(path.resolve().relative_to(repo_root)))
    except ValueError:
        return normalize_rel_path(str(path.resolve()))


def parse_optional_bool(raw: str | None):
    if raw is None:
        return None
    value = str(raw).strip().lower()
    if value in {"1", "true", "yes", "on"}:
        return True
    if value in {"0", "false", "no", "off"}:
        return False
    raise SystemExit(f"Invalid boolean value: {raw}")


def _secret_tool_lookup(service: str, account: str) -> str | None:
    bin_path = shutil.which("secret-tool")
    if not bin_path:
        return None
    proc = subprocess.run(
        [bin_path, "lookup", "service", service, "account", account],
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        return None
    value = (proc.stdout or "").strip()
    return value or None


def _secret_tool_store(service: str, account: str, secret_value: str) -> bool:
    bin_path = shutil.which("secret-tool")
    if not bin_path:
        return False
    proc = subprocess.run(
        [bin_path, "store", "--label", f"IAT {service} {account}", "service", service, "account", account],
        input=(secret_value + "\n"),
        capture_output=True,
        text=True,
    )
    return proc.returncode == 0


def load_or_create_master_key(repo_root: Path, allow_create: bool = True) -> str:
    env_key = (os.environ.get("IAT_MASTER_KEY") or "").strip()
    if env_key:
        return env_key

    service = os.environ.get("IAT_SECRET_SERVICE", DEFAULT_SECRET_SERVICE)
    account = os.environ.get("IAT_SECRET_ACCOUNT", DEFAULT_SECRET_ACCOUNT)
    key = _secret_tool_lookup(service, account)
    if key:
        return key

    key_file_raw = os.environ.get("IAT_MASTER_KEY_FILE", DEFAULT_MASTER_KEY_FILE)
    key_file = (repo_root / key_file_raw).resolve()
    if key_file.exists():
        return key_file.read_text(encoding="utf-8").strip()

    if not allow_create:
        raise RuntimeError("Master key not found. Set IAT_MASTER_KEY or configure secret-tool.")

    generated = Fernet.generate_key().decode("utf-8")
    if _secret_tool_store(service, account, generated):
        return generated

    # Fallback for environments without running secret service.
    key_file.parent.mkdir(parents=True, exist_ok=True)
    key_file.write_text(generated + "\n", encoding="utf-8")
    os.chmod(key_file, 0o600)
    return generated


def encrypt_secret_value(secret_value: str, repo_root: Path) -> str:
    if not secret_value:
        raise ValueError("Secret value must not be empty")
    key = load_or_create_master_key(repo_root, allow_create=True)
    token = Fernet(key.encode("utf-8")).encrypt(secret_value.encode("utf-8"))
    return token.decode("utf-8")


def decrypt_secret_value(secret_ciphertext: str, repo_root: Path) -> str:
    key = load_or_create_master_key(repo_root, allow_create=False)
    try:
        value = Fernet(key.encode("utf-8")).decrypt(secret_ciphertext.encode("utf-8"))
    except InvalidToken as exc:
        raise RuntimeError("Unable to decrypt secret (invalid token or master key mismatch)") from exc
    return value.decode("utf-8")


def mask_secret_value(secret_value: str) -> str:
    raw = str(secret_value or "")
    if len(raw) <= 6:
        return "*" * len(raw)
    return f"{raw[:3]}***{raw[-3:]}"


def slugify(value: str) -> str:
    out = re.sub(r"[^a-zA-Z0-9_-]+", "_", (value or "").strip().lower())
    out = re.sub(r"_+", "_", out).strip("_")
    return out or "project"


def sha256_of_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fh:
        while True:
            chunk = fh.read(1024 * 1024)
            if not chunk:
                break
            h.update(chunk)
    return h.hexdigest()


def connect_db(db_path: Path) -> sqlite3.Connection:
    db_path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(db_path)
    conn.row_factory = sqlite3.Row
    conn.execute("PRAGMA foreign_keys = ON;")
    return conn


def table_has_column(conn: sqlite3.Connection, table_name: str, column_name: str) -> bool:
    rows = conn.execute(f"PRAGMA table_info({table_name})").fetchall()
    return any(str(r["name"]) == column_name for r in rows)


def ensure_column(conn: sqlite3.Connection, table_name: str, column_name: str, column_sql: str):
    if table_has_column(conn, table_name, column_name):
        return
    conn.execute(f"ALTER TABLE {table_name} ADD COLUMN {column_name} {column_sql}")


def record_migration(conn: sqlite3.Connection, version: str, note: str = ""):
    conn.execute(
        """
        INSERT OR IGNORE INTO schema_migrations (version, note, applied_at)
        VALUES (?, ?, ?)
        """,
        (version, note, now_iso()),
    )


def init_schema(conn: sqlite3.Connection) -> None:
    conn.executescript(
        """
        CREATE TABLE IF NOT EXISTS users (
          id TEXT PRIMARY KEY,
          username TEXT NOT NULL UNIQUE,
          display_name TEXT NOT NULL,
          email TEXT,
          is_active INTEGER NOT NULL DEFAULT 1,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS schema_migrations (
          version TEXT PRIMARY KEY,
          note TEXT NOT NULL DEFAULT '',
          applied_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS projects (
          id TEXT PRIMARY KEY,
          user_id TEXT NOT NULL,
          slug TEXT NOT NULL,
          name TEXT NOT NULL,
          description TEXT NOT NULL DEFAULT '',
          status TEXT NOT NULL DEFAULT 'active',
          settings_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE(user_id, slug),
          FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS runs (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          run_log_path TEXT NOT NULL,
          mode TEXT NOT NULL,
          stage TEXT,
          time_of_day TEXT,
          weather TEXT,
          model TEXT,
          image_size TEXT,
          image_quality TEXT,
          status TEXT NOT NULL,
          meta_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          UNIQUE(project_id, run_log_path),
          FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS run_jobs (
          id TEXT PRIMARY KEY,
          run_id TEXT NOT NULL,
          job_key TEXT NOT NULL,
          status TEXT NOT NULL,
          selected_candidate INTEGER,
          final_output TEXT,
          meta_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          UNIQUE(run_id, job_key),
          FOREIGN KEY(run_id) REFERENCES runs(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS run_job_candidates (
          id TEXT PRIMARY KEY,
          job_id TEXT NOT NULL,
          candidate_index INTEGER NOT NULL,
          status TEXT NOT NULL,
          output_path TEXT,
          final_output_path TEXT,
          rank_hard_failures INTEGER NOT NULL DEFAULT 0,
          rank_soft_warnings INTEGER NOT NULL DEFAULT 0,
          rank_avg_chroma_exceed REAL NOT NULL DEFAULT 0,
          meta_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          UNIQUE(job_id, candidate_index),
          FOREIGN KEY(job_id) REFERENCES run_jobs(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS assets (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          run_id TEXT,
          job_id TEXT,
          candidate_id TEXT,
          asset_kind TEXT NOT NULL,
          rel_path TEXT NOT NULL,
          sha256 TEXT,
          meta_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          UNIQUE(project_id, rel_path),
          FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS project_snapshots (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          snapshot_tag TEXT NOT NULL,
          notes TEXT NOT NULL DEFAULT '',
          manifest_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          UNIQUE(project_id, snapshot_tag),
          FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS project_exports (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          export_path TEXT NOT NULL,
          export_sha256 TEXT,
          created_at TEXT NOT NULL,
          FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS project_api_secrets (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          provider_code TEXT NOT NULL,
          secret_name TEXT NOT NULL,
          secret_ciphertext TEXT NOT NULL,
          key_ref TEXT NOT NULL DEFAULT 'local-master',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE(project_id, provider_code, secret_name),
          FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS chat_sessions (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          user_id TEXT NOT NULL,
          title TEXT NOT NULL DEFAULT '',
          status TEXT NOT NULL DEFAULT 'active',
          context_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE,
          FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS chat_messages (
          id TEXT PRIMARY KEY,
          session_id TEXT NOT NULL,
          role TEXT NOT NULL,
          content_text TEXT NOT NULL,
          content_json TEXT NOT NULL DEFAULT '{}',
          voice_asset_id TEXT,
          token_usage_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          FOREIGN KEY(session_id) REFERENCES chat_sessions(id) ON DELETE CASCADE,
          FOREIGN KEY(voice_asset_id) REFERENCES assets(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS agent_instructions (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          session_id TEXT,
          message_id TEXT,
          instruction_type TEXT NOT NULL,
          payload_json TEXT NOT NULL,
          status TEXT NOT NULL,
          priority INTEGER NOT NULL DEFAULT 100,
          requires_confirmation INTEGER NOT NULL DEFAULT 0,
          confirmed_by_user_id TEXT,
          queued_at TEXT,
          started_at TEXT,
          finished_at TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE,
          FOREIGN KEY(session_id) REFERENCES chat_sessions(id) ON DELETE SET NULL,
          FOREIGN KEY(message_id) REFERENCES chat_messages(id) ON DELETE SET NULL,
          FOREIGN KEY(confirmed_by_user_id) REFERENCES users(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS agent_instruction_events (
          id TEXT PRIMARY KEY,
          instruction_id TEXT NOT NULL,
          event_type TEXT NOT NULL,
          event_payload_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          FOREIGN KEY(instruction_id) REFERENCES agent_instructions(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS voice_requests (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          session_id TEXT,
          message_id TEXT,
          direction TEXT NOT NULL,
          provider_code TEXT NOT NULL,
          input_asset_id TEXT,
          output_asset_id TEXT,
          status TEXT NOT NULL,
          latency_ms INTEGER,
          meta_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE,
          FOREIGN KEY(session_id) REFERENCES chat_sessions(id) ON DELETE SET NULL,
          FOREIGN KEY(message_id) REFERENCES chat_messages(id) ON DELETE SET NULL,
          FOREIGN KEY(input_asset_id) REFERENCES assets(id) ON DELETE SET NULL,
          FOREIGN KEY(output_asset_id) REFERENCES assets(id) ON DELETE SET NULL
        );

        CREATE INDEX IF NOT EXISTS idx_projects_user ON projects(user_id);
        CREATE INDEX IF NOT EXISTS idx_runs_project ON runs(project_id);
        CREATE INDEX IF NOT EXISTS idx_jobs_run ON run_jobs(run_id);
        CREATE INDEX IF NOT EXISTS idx_candidates_job ON run_job_candidates(job_id);
        CREATE INDEX IF NOT EXISTS idx_assets_project ON assets(project_id);
        CREATE INDEX IF NOT EXISTS idx_project_api_secrets_proj ON project_api_secrets(project_id);
        CREATE INDEX IF NOT EXISTS idx_chat_sessions_project ON chat_sessions(project_id, updated_at);
        CREATE INDEX IF NOT EXISTS idx_chat_messages_session ON chat_messages(session_id, created_at);
        CREATE INDEX IF NOT EXISTS idx_agent_instructions_project ON agent_instructions(project_id, status, priority, created_at);
        CREATE INDEX IF NOT EXISTS idx_agent_instruction_events_instr ON agent_instruction_events(instruction_id, created_at);
        CREATE INDEX IF NOT EXISTS idx_voice_requests_project ON voice_requests(project_id, status, created_at);
        """
    )

    # Incremental queue/runtime columns for backwards-compatible upgrades.
    ensure_column(conn, "agent_instructions", "attempts", "INTEGER NOT NULL DEFAULT 0")
    ensure_column(conn, "agent_instructions", "max_attempts", "INTEGER NOT NULL DEFAULT 3")
    ensure_column(conn, "agent_instructions", "next_attempt_at", "TEXT")
    ensure_column(conn, "agent_instructions", "last_error", "TEXT")
    ensure_column(conn, "agent_instructions", "locked_by", "TEXT")
    ensure_column(conn, "agent_instructions", "locked_at", "TEXT")
    ensure_column(conn, "agent_instructions", "agent_response_json", "TEXT NOT NULL DEFAULT '{}'")
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_agent_instructions_queue ON agent_instructions(status, priority, next_attempt_at, created_at)"
    )

    record_migration(conn, "20260220_0001_base_schema", "base schema + chat + storage + exports")
    record_migration(conn, "20260220_0002_instruction_queue", "instruction retries/locks columns")
    record_migration(conn, "20260220_0003_project_api_secrets", "encrypted provider secret storage")
    conn.commit()


def get_user_by_username(conn: sqlite3.Connection, username: str):
    return conn.execute("SELECT * FROM users WHERE username = ?", (username,)).fetchone()


def ensure_user(conn: sqlite3.Connection, username: str, display_name: str, email: str | None):
    username = slugify(username)
    ts = now_iso()
    row = get_user_by_username(conn, username)
    if row:
        conn.execute(
            """
            UPDATE users
            SET display_name = ?, email = ?, is_active = 1, updated_at = ?
            WHERE id = ?
            """,
            (display_name, email, ts, row["id"]),
        )
        conn.commit()
        return conn.execute("SELECT * FROM users WHERE id = ?", (row["id"],)).fetchone()

    new_id = uid()
    conn.execute(
        """
        INSERT INTO users (id, username, display_name, email, is_active, created_at, updated_at)
        VALUES (?, ?, ?, ?, 1, ?, ?)
        """,
        (new_id, username, display_name, email, ts, ts),
    )
    conn.commit()
    return conn.execute("SELECT * FROM users WHERE id = ?", (new_id,)).fetchone()


def get_project(conn: sqlite3.Connection, project_id: str | None, project_slug: str | None):
    if project_id:
        return conn.execute("SELECT * FROM projects WHERE id = ?", (project_id,)).fetchone()
    if project_slug:
        return conn.execute("SELECT * FROM projects WHERE slug = ?", (slugify(project_slug),)).fetchone()
    return None


def load_project_settings(project_row) -> dict:
    raw = project_row["settings_json"] if project_row and "settings_json" in project_row.keys() else "{}"
    try:
        parsed = json.loads(raw or "{}")
    except Exception:
        parsed = {}
    if not isinstance(parsed, dict):
        parsed = {}
    return parsed


def save_project_settings(conn: sqlite3.Connection, project_id: str, settings: dict):
    conn.execute(
        "UPDATE projects SET settings_json = ?, updated_at = ? WHERE id = ?",
        (to_json(settings or {}), now_iso(), project_id),
    )
    conn.commit()


def resolve_storage_settings(settings: dict, project_slug: str) -> dict:
    storage = settings.get("storage", {}) if isinstance(settings, dict) else {}
    if not isinstance(storage, dict):
        storage = {}

    local = storage.get("local", {})
    if not isinstance(local, dict):
        local = {}

    s3 = storage.get("s3", {})
    if not isinstance(s3, dict):
        s3 = {}

    return {
        "local": {
            "base_dir": str(local.get("base_dir", "generated/projects")).strip(),
            "project_root": str(local.get("project_root", "")).strip(),
        },
        "s3": {
            "enabled": bool(s3.get("enabled", False)),
            "bucket": str(s3.get("bucket", "")).strip(),
            "prefix": str(s3.get("prefix", "iat-projects")).strip(),
            "region": str(s3.get("region", "")).strip(),
            "profile": str(s3.get("profile", "")).strip(),
            "endpoint_url": str(s3.get("endpoint_url", "")).strip(),
        },
        "project_slug": project_slug,
    }


def resolve_project_local_root(repo_root: Path, project_slug: str, storage_settings: dict) -> Path:
    local = storage_settings.get("local", {}) if isinstance(storage_settings, dict) else {}
    project_root_cfg = str(local.get("project_root", "")).strip()
    if project_root_cfg:
        p = Path(project_root_cfg)
        return p if p.is_absolute() else (repo_root / p).resolve()

    base_dir = str(local.get("base_dir", "generated/projects")).strip() or "generated/projects"
    base_path = Path(base_dir)
    base_abs = base_path if base_path.is_absolute() else (repo_root / base_path).resolve()
    return (base_abs / project_slug).resolve()


def project_storage_payload(repo_root: Path, project_row) -> dict:
    settings = load_project_settings(project_row)
    storage = resolve_storage_settings(settings, project_row["slug"])
    local_root = resolve_project_local_root(repo_root, project_row["slug"], storage)
    return {
        "project": {
            "id": project_row["id"],
            "slug": project_row["slug"],
            "name": project_row["name"],
        },
        "storage": {
            "local": {
                **storage["local"],
                "project_root": str(local_root),
            },
            "s3": storage["s3"],
        },
    }


def ensure_project(
    conn: sqlite3.Connection,
    user_id: str,
    slug: str,
    name: str,
    description: str = "",
    status: str = "active",
):
    ts = now_iso()
    safe_slug = slugify(slug)
    row = conn.execute(
        "SELECT * FROM projects WHERE user_id = ? AND slug = ?",
        (user_id, safe_slug),
    ).fetchone()
    if row:
        conn.execute(
            """
            UPDATE projects
            SET name = ?, description = ?, status = ?, updated_at = ?
            WHERE id = ?
            """,
            (name, description, status, ts, row["id"]),
        )
        conn.commit()
        return conn.execute("SELECT * FROM projects WHERE id = ?", (row["id"],)).fetchone()

    project_id = uid()
    conn.execute(
        """
        INSERT INTO projects (id, user_id, slug, name, description, status, settings_json, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, '{}', ?, ?)
        """,
        (project_id, user_id, safe_slug, name, description, status, ts, ts),
    )
    conn.commit()
    return conn.execute("SELECT * FROM projects WHERE id = ?", (project_id,)).fetchone()


def upsert_asset(
    conn: sqlite3.Connection,
    project_id: str,
    run_id: str | None,
    job_id: str | None,
    candidate_id: str | None,
    asset_kind: str,
    rel_path: str,
    repo_root: Path,
    compute_hashes: bool,
    extra_meta: dict | None = None,
):
    clean_rel = normalize_rel_path(rel_path)
    if not clean_rel:
        return

    abs_path = (repo_root / clean_rel).resolve()
    file_hash = None
    if compute_hashes and abs_path.exists() and abs_path.is_file():
        file_hash = sha256_of_file(abs_path)

    ts = now_iso()
    payload = {
        "path_exists": bool(abs_path.exists()),
    }
    if extra_meta:
        payload.update(extra_meta)

    existing = conn.execute(
        "SELECT id FROM assets WHERE project_id = ? AND rel_path = ?",
        (project_id, clean_rel),
    ).fetchone()
    if existing:
        conn.execute(
            """
            UPDATE assets
            SET run_id = ?, job_id = ?, candidate_id = ?, asset_kind = ?, sha256 = ?, meta_json = ?, created_at = ?
            WHERE id = ?
            """,
            (
                run_id,
                job_id,
                candidate_id,
                asset_kind,
                file_hash,
                to_json(payload),
                ts,
                existing["id"],
            ),
        )
        return

    conn.execute(
        """
        INSERT INTO assets (id, project_id, run_id, job_id, candidate_id, asset_kind, rel_path, sha256, meta_json, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (uid(), project_id, run_id, job_id, candidate_id, asset_kind, clean_rel, file_hash, to_json(payload), ts),
    )


def derive_run_status(run_data: dict) -> str:
    jobs = run_data.get("jobs", [])
    if not isinstance(jobs, list):
        return "unknown"
    statuses = [str(j.get("status", "")).strip().lower() for j in jobs if isinstance(j, dict)]
    if any(s.startswith("failed") for s in statuses):
        return "failed"
    if statuses and all(s in {"done", "planned"} for s in statuses):
        return "ok"
    return "partial"


def ingest_run(conn: sqlite3.Connection, repo_root: Path, project_row, run_log_path: Path, compute_hashes: bool):
    run_data = read_json(run_log_path)
    rel_run_log_path = path_for_storage(run_log_path, repo_root)
    run_status = derive_run_status(run_data)
    ts = now_iso()

    existing_run = conn.execute(
        "SELECT id FROM runs WHERE project_id = ? AND run_log_path = ?",
        (project_row["id"], rel_run_log_path),
    ).fetchone()
    if existing_run:
        conn.execute("DELETE FROM runs WHERE id = ?", (existing_run["id"],))

    run_id = uid()
    conn.execute(
        """
        INSERT INTO runs
          (id, project_id, run_log_path, mode, stage, time_of_day, weather, model, image_size, image_quality, status, meta_json, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            run_id,
            project_row["id"],
            rel_run_log_path,
            str(run_data.get("mode", "")),
            str(run_data.get("stage", "")),
            str(run_data.get("time", "")),
            str(run_data.get("weather", "")),
            str(run_data.get("model", "")),
            str(run_data.get("size", "")),
            str(run_data.get("quality", "")),
            run_status,
            to_json(
                {
                    "timestamp": run_data.get("timestamp"),
                    "generation": run_data.get("generation"),
                    "postprocess": run_data.get("postprocess"),
                    "output_guard": run_data.get("output_guard"),
                }
            ),
            ts,
        ),
    )

    jobs = run_data.get("jobs", [])
    if not isinstance(jobs, list):
        jobs = []

    inserted_jobs = 0
    inserted_candidates = 0
    inserted_assets = 0

    for idx, job in enumerate(jobs, start=1):
        if not isinstance(job, dict):
            continue
        job_key = str(job.get("id") or f"job_{idx}")
        job_id = uid()
        conn.execute(
            """
            INSERT INTO run_jobs (id, run_id, job_key, status, selected_candidate, final_output, meta_json, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            """,
            (
                job_id,
                run_id,
                job_key,
                str(job.get("status", "")),
                int(job["selected_candidate"]) if isinstance(job.get("selected_candidate"), int) else None,
                normalize_rel_path(str(job.get("final_output") or "")) or None,
                to_json(job),
                ts,
            ),
        )
        inserted_jobs += 1

        candidates = job.get("candidates", [])
        if not isinstance(candidates, list) or not candidates:
            synthetic_candidate = {
                "candidate_index": 1,
                "status": str(job.get("status", "")),
                "output": job.get("output"),
                "final_output": job.get("final_output"),
                "rank": {
                    "hard_failures": 0,
                    "soft_warnings": 0,
                    "avg_chroma_exceed": 0.0,
                },
            }
            candidates = [synthetic_candidate]

        for candidate in candidates:
            if not isinstance(candidate, dict):
                continue
            candidate_id = uid()
            rank = candidate.get("rank") if isinstance(candidate.get("rank"), dict) else {}
            candidate_index = int(candidate.get("candidate_index", inserted_candidates + 1))
            output_path = normalize_rel_path(str(candidate.get("output") or "")) or None
            final_output_path = normalize_rel_path(str(candidate.get("final_output") or "")) or None
            conn.execute(
                """
                INSERT INTO run_job_candidates
                  (id, job_id, candidate_index, status, output_path, final_output_path,
                   rank_hard_failures, rank_soft_warnings, rank_avg_chroma_exceed, meta_json, created_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    candidate_id,
                    job_id,
                    candidate_index,
                    str(candidate.get("status", "")),
                    output_path,
                    final_output_path,
                    int(rank.get("hard_failures", 0) or 0),
                    int(rank.get("soft_warnings", 0) or 0),
                    float(rank.get("avg_chroma_exceed", 0.0) or 0.0),
                    to_json(candidate),
                    ts,
                ),
            )
            inserted_candidates += 1

            if output_path:
                upsert_asset(
                    conn,
                    project_row["id"],
                    run_id,
                    job_id,
                    candidate_id,
                    "candidate_output",
                    output_path,
                    repo_root,
                    compute_hashes,
                )
                inserted_assets += 1
            if final_output_path and final_output_path != output_path:
                upsert_asset(
                    conn,
                    project_row["id"],
                    run_id,
                    job_id,
                    candidate_id,
                    "candidate_final_output",
                    final_output_path,
                    repo_root,
                    compute_hashes,
                )
                inserted_assets += 1

        final_output = normalize_rel_path(str(job.get("final_output") or ""))
        if final_output:
            upsert_asset(
                conn,
                project_row["id"],
                run_id,
                job_id,
                None,
                "job_final_output",
                final_output,
                repo_root,
                compute_hashes,
                extra_meta={"selected_candidate": job.get("selected_candidate")},
            )
            inserted_assets += 1

    conn.commit()
    return {
        "run_id": run_id,
        "run_log_path": rel_run_log_path,
        "jobs": inserted_jobs,
        "candidates": inserted_candidates,
        "assets_upserted": inserted_assets,
        "status": run_status,
    }


def table_columns(conn: sqlite3.Connection, table_name: str):
    rows = conn.execute(f"PRAGMA table_info({table_name})").fetchall()
    return [r["name"] for r in rows]


def copy_rows(src_conn, dst_conn, table_name: str, where_sql: str, params: tuple):
    cols = table_columns(src_conn, table_name)
    col_sql = ", ".join(cols)
    placeholders = ", ".join(["?"] * len(cols))
    rows = src_conn.execute(f"SELECT {col_sql} FROM {table_name} WHERE {where_sql}", params).fetchall()
    if not rows:
        return 0
    values = [tuple(r[c] for c in cols) for r in rows]
    dst_conn.executemany(f"INSERT INTO {table_name} ({col_sql}) VALUES ({placeholders})", values)
    return len(values)


def export_project_package(
    conn: sqlite3.Connection,
    db_path: Path,
    repo_root: Path,
    project_row,
    source_files_root: Path,
    output_path: Path,
    include_files: bool,
):
    stamp = now_iso().replace(":", "-")
    with tempfile.TemporaryDirectory(prefix="iat_project_export_") as temp_dir_raw:
        temp_dir = Path(temp_dir_raw)
        package_root = temp_dir / f"{project_row['slug']}_{stamp}"
        package_root.mkdir(parents=True, exist_ok=True)

        export_db_path = package_root / "project.db"
        export_conn = connect_db(export_db_path)
        init_schema(export_conn)

        # User + project.
        owner = conn.execute("SELECT * FROM users WHERE id = ?", (project_row["user_id"],)).fetchone()
        if owner:
            copy_rows(conn, export_conn, "users", "id = ?", (owner["id"],))
        copy_rows(conn, export_conn, "projects", "id = ?", (project_row["id"],))

        # Runs and nested data.
        run_rows = conn.execute("SELECT id FROM runs WHERE project_id = ?", (project_row["id"],)).fetchall()
        run_ids = [r["id"] for r in run_rows]
        runs_copied = copy_rows(conn, export_conn, "runs", "project_id = ?", (project_row["id"],))

        jobs_copied = 0
        candidates_copied = 0
        if run_ids:
            placeholders = ", ".join(["?"] * len(run_ids))
            jobs_copied = copy_rows(conn, export_conn, "run_jobs", f"run_id IN ({placeholders})", tuple(run_ids))
            job_rows = conn.execute(
                f"SELECT id FROM run_jobs WHERE run_id IN ({placeholders})",
                tuple(run_ids),
            ).fetchall()
            job_ids = [r["id"] for r in job_rows]
            if job_ids:
                job_placeholders = ", ".join(["?"] * len(job_ids))
                candidates_copied = copy_rows(
                    conn,
                    export_conn,
                    "run_job_candidates",
                    f"job_id IN ({job_placeholders})",
                    tuple(job_ids),
                )

        assets_copied = copy_rows(conn, export_conn, "assets", "project_id = ?", (project_row["id"],))
        snapshots_copied = copy_rows(conn, export_conn, "project_snapshots", "project_id = ?", (project_row["id"],))
        export_conn.commit()
        export_conn.close()

        copied_files = 0
        if include_files:
            if source_files_root.exists() and source_files_root.is_dir():
                target_root = package_root / "files" / "generated" / "projects" / project_row["slug"]
                target_root.parent.mkdir(parents=True, exist_ok=True)
                shutil.copytree(source_files_root, target_root)
                copied_files = sum(1 for p in target_root.rglob("*") if p.is_file())

        metadata = {
            "exported_at": now_iso(),
            "source_db": path_for_storage(db_path, repo_root),
            "project": {
                "id": project_row["id"],
                "slug": project_row["slug"],
                "name": project_row["name"],
                "user_id": project_row["user_id"],
            },
            "copied_rows": {
                "runs": runs_copied,
                "jobs": jobs_copied,
                "candidates": candidates_copied,
                "assets": assets_copied,
                "snapshots": snapshots_copied,
            },
            "copied_files": copied_files,
        }
        (package_root / "metadata.json").write_text(to_json(metadata), encoding="utf-8")

        output_path.parent.mkdir(parents=True, exist_ok=True)
        if output_path.suffix == ".gz" and output_path.name.endswith(".tar.gz"):
            with tarfile.open(output_path, "w:gz") as tf:
                tf.add(package_root, arcname=package_root.name)
        elif output_path.suffix == ".tgz":
            with tarfile.open(output_path, "w:gz") as tf:
                tf.add(package_root, arcname=package_root.name)
        else:
            if output_path.exists():
                shutil.rmtree(output_path)
            shutil.copytree(package_root, output_path)

    export_hash = sha256_of_file(output_path) if output_path.is_file() else None
    conn.execute(
        """
        INSERT INTO project_exports (id, project_id, export_path, export_sha256, created_at)
        VALUES (?, ?, ?, ?, ?)
        """,
        (
            uid(),
            project_row["id"],
            path_for_storage(output_path, repo_root),
            export_hash,
            now_iso(),
        ),
    )
    conn.commit()
    return {"export_path": str(output_path), "export_sha256": export_hash}


def require_project(conn: sqlite3.Connection, project_id: str, project_slug: str):
    project = get_project(conn, project_id, project_slug)
    if not project:
        raise SystemExit("Project not found. Use --project-id or --project-slug.")
    return project


def upsert_project_secret(
    conn: sqlite3.Connection,
    repo_root: Path,
    project_id: str,
    provider_code: str,
    secret_name: str,
    secret_value: str,
):
    ts = now_iso()
    ciphertext = encrypt_secret_value(secret_value, repo_root)
    row = conn.execute(
        """
        SELECT id
        FROM project_api_secrets
        WHERE project_id = ? AND provider_code = ? AND secret_name = ?
        """,
        (project_id, provider_code, secret_name),
    ).fetchone()
    if row:
        conn.execute(
            """
            UPDATE project_api_secrets
            SET secret_ciphertext = ?, key_ref = 'local-master', updated_at = ?
            WHERE id = ?
            """,
            (ciphertext, ts, row["id"]),
        )
        conn.commit()
        return row["id"]

    secret_id = uid()
    conn.execute(
        """
        INSERT INTO project_api_secrets
        (id, project_id, provider_code, secret_name, secret_ciphertext, key_ref, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, 'local-master', ?, ?)
        """,
        (secret_id, project_id, provider_code, secret_name, ciphertext, ts, ts),
    )
    conn.commit()
    return secret_id


def fetch_project_secret_value(
    conn: sqlite3.Connection,
    repo_root: Path,
    project_id: str,
    provider_code: str,
    secret_name: str,
) -> str | None:
    row = conn.execute(
        """
        SELECT secret_ciphertext
        FROM project_api_secrets
        WHERE project_id = ? AND provider_code = ? AND secret_name = ?
        """,
        (project_id, provider_code, secret_name),
    ).fetchone()
    if not row:
        return None
    return decrypt_secret_value(row["secret_ciphertext"], repo_root)


def list_project_secrets(conn: sqlite3.Connection, project_id: str):
    return conn.execute(
        """
        SELECT id, provider_code, secret_name, key_ref, created_at, updated_at
        FROM project_api_secrets
        WHERE project_id = ?
        ORDER BY provider_code, secret_name
        """,
        (project_id,),
    ).fetchall()


def cmd_get_project_storage(args):
    repo_root = Path.cwd()
    db_path = (repo_root / args.db).resolve()
    conn = connect_db(db_path)
    init_schema(conn)
    project = require_project(conn, args.project_id, args.project_slug)
    payload = project_storage_payload(repo_root, project)
    print(to_json({"ok": True, **payload}))
    conn.close()


def cmd_migrate(args):
    repo_root = Path.cwd()
    db_path = (repo_root / args.db).resolve()
    conn = connect_db(db_path)
    init_schema(conn)
    rows = conn.execute("SELECT version, note, applied_at FROM schema_migrations ORDER BY applied_at ASC").fetchall()
    print(
        to_json(
            {
                "ok": True,
                "db": str(db_path),
                "applied": [
                    {"version": r["version"], "note": r["note"], "applied_at": r["applied_at"]}
                    for r in rows
                ],
            }
        )
    )
    conn.close()


def cmd_set_project_secret(args):
    repo_root = Path.cwd()
    db_path = (repo_root / args.db).resolve()
    conn = connect_db(db_path)
    init_schema(conn)
    project = require_project(conn, args.project_id, args.project_slug)
    provider_code = str(args.provider_code).strip().lower()
    secret_name = str(args.secret_name).strip()
    secret_value = str(args.secret_value).strip()
    if not provider_code or not secret_name or not secret_value:
        raise SystemExit("--provider-code, --secret-name and --secret-value are required")
    secret_id = upsert_project_secret(conn, repo_root, project["id"], provider_code, secret_name, secret_value)
    print(
        to_json(
            {
                "ok": True,
                "project_slug": project["slug"],
                "secret": {
                    "id": secret_id,
                    "provider_code": provider_code,
                    "secret_name": secret_name,
                    "masked": mask_secret_value(secret_value),
                },
            }
        )
    )
    conn.close()


def cmd_list_project_secrets(args):
    repo_root = Path.cwd()
    db_path = (repo_root / args.db).resolve()
    conn = connect_db(db_path)
    init_schema(conn)
    project = require_project(conn, args.project_id, args.project_slug)
    rows = list_project_secrets(conn, project["id"])
    items = []
    for r in rows:
        masked = None
        if args.include_mask:
            try:
                plain = decrypt_secret_value(
                    conn.execute(
                        "SELECT secret_ciphertext FROM project_api_secrets WHERE id = ?",
                        (r["id"],),
                    ).fetchone()["secret_ciphertext"],
                    repo_root,
                )
                masked = mask_secret_value(plain)
            except Exception:
                masked = "***"
        items.append(
            {
                "id": r["id"],
                "provider_code": r["provider_code"],
                "secret_name": r["secret_name"],
                "masked": masked,
                "key_ref": r["key_ref"],
                "created_at": r["created_at"],
                "updated_at": r["updated_at"],
            }
        )
    print(to_json({"ok": True, "project_slug": project["slug"], "count": len(items), "secrets": items}))
    conn.close()


def cmd_delete_project_secret(args):
    repo_root = Path.cwd()
    db_path = (repo_root / args.db).resolve()
    conn = connect_db(db_path)
    init_schema(conn)
    project = require_project(conn, args.project_id, args.project_slug)
    provider_code = str(args.provider_code).strip().lower()
    secret_name = str(args.secret_name).strip()
    cur = conn.execute(
        """
        DELETE FROM project_api_secrets
        WHERE project_id = ? AND provider_code = ? AND secret_name = ?
        """,
        (project["id"], provider_code, secret_name),
    )
    conn.commit()
    print(
        to_json(
            {
                "ok": True,
                "project_slug": project["slug"],
                "deleted": cur.rowcount,
                "provider_code": provider_code,
                "secret_name": secret_name,
            }
        )
    )
    conn.close()


def cmd_set_project_storage_local(args):
    repo_root = Path.cwd()
    db_path = (repo_root / args.db).resolve()
    conn = connect_db(db_path)
    init_schema(conn)
    project = require_project(conn, args.project_id, args.project_slug)

    if not args.base_dir and not args.project_root:
        raise SystemExit("Specify --base-dir and/or --project-root")

    settings = load_project_settings(project)
    storage = settings.setdefault("storage", {})
    local = storage.setdefault("local", {})
    if args.base_dir:
        local["base_dir"] = args.base_dir
    if args.project_root:
        local["project_root"] = args.project_root

    save_project_settings(conn, project["id"], settings)
    refreshed = require_project(conn, project["id"], "")
    payload = project_storage_payload(repo_root, refreshed)
    print(to_json({"ok": True, "updated": "local", **payload}))
    conn.close()


def cmd_set_project_storage_s3(args):
    repo_root = Path.cwd()
    db_path = (repo_root / args.db).resolve()
    conn = connect_db(db_path)
    init_schema(conn)
    project = require_project(conn, args.project_id, args.project_slug)

    settings = load_project_settings(project)
    storage = settings.setdefault("storage", {})
    s3 = storage.setdefault("s3", {})

    enabled = parse_optional_bool(args.enabled)
    if enabled is not None:
        s3["enabled"] = enabled
    if args.bucket is not None:
        s3["bucket"] = args.bucket
    if args.prefix is not None:
        s3["prefix"] = args.prefix
    if args.region is not None:
        s3["region"] = args.region
    if args.profile is not None:
        s3["profile"] = args.profile
    if args.endpoint_url is not None:
        s3["endpoint_url"] = args.endpoint_url

    save_project_settings(conn, project["id"], settings)
    refreshed = require_project(conn, project["id"], "")
    payload = project_storage_payload(repo_root, refreshed)
    print(to_json({"ok": True, "updated": "s3", **payload}))
    conn.close()


def cmd_sync_project_s3(args):
    repo_root = Path.cwd()
    db_path = (repo_root / args.db).resolve()
    conn = connect_db(db_path)
    init_schema(conn)
    project = require_project(conn, args.project_id, args.project_slug)
    payload = project_storage_payload(repo_root, project)
    local_root = Path(payload["storage"]["local"]["project_root"])
    s3_cfg = payload["storage"]["s3"]
    conn.close()

    if not s3_cfg.get("enabled"):
        raise SystemExit("S3 storage is disabled for this project. Enable via set-project-storage-s3.")
    bucket = str(s3_cfg.get("bucket", "")).strip()
    if not bucket:
        raise SystemExit("S3 bucket is not configured for this project.")

    prefix = str(s3_cfg.get("prefix", "")).strip().strip("/")
    slug = payload["project"]["slug"]
    dst = f"s3://{bucket}/{prefix}/{slug}/" if prefix else f"s3://{bucket}/{slug}/"

    aws_bin = shutil.which("aws")
    if not aws_bin:
        raise SystemExit("AWS CLI not found. Install aws cli v2 to use sync-project-s3.")

    cmd = [aws_bin, "s3", "sync", str(local_root), dst, "--only-show-errors"]
    if args.delete:
        cmd.append("--delete")
    if args.dry_run:
        cmd.append("--dryrun")
    if s3_cfg.get("region"):
        cmd.extend(["--region", s3_cfg["region"]])
    if s3_cfg.get("profile"):
        cmd.extend(["--profile", s3_cfg["profile"]])
    if s3_cfg.get("endpoint_url"):
        cmd.extend(["--endpoint-url", s3_cfg["endpoint_url"]])

    if not local_root.exists():
        if args.allow_missing_local:
            print(
                to_json(
                    {
                        "ok": True,
                        "skipped": True,
                        "reason": "missing_local_project_root",
                        "project_root": str(local_root),
                        "destination": dst,
                    }
                )
            )
            return
        raise SystemExit(f"Local project root not found: {local_root}")

    proc = subprocess.run(cmd, capture_output=True, text=True)
    if proc.returncode != 0:
        raise SystemExit(f"AWS sync failed ({proc.returncode}): {(proc.stderr or proc.stdout).strip()}")
    print(
        to_json(
            {
                "ok": True,
                "project_slug": slug,
                "project_root": str(local_root),
                "destination": dst,
                "dry_run": bool(args.dry_run),
                "delete": bool(args.delete),
            }
        )
    )


def cmd_init_db(args):
    repo_root = Path.cwd()
    db_path = (repo_root / args.db).resolve()
    conn = connect_db(db_path)
    init_schema(conn)
    if args.with_default_user:
        ensure_user(conn, "local", "Local User", None)
    print(to_json({"ok": True, "db": str(db_path), "default_user": bool(args.with_default_user)}))
    conn.close()


def cmd_ensure_user(args):
    repo_root = Path.cwd()
    db_path = (repo_root / args.db).resolve()
    conn = connect_db(db_path)
    init_schema(conn)
    row = ensure_user(conn, args.username, args.display_name, args.email)
    print(
        to_json(
            {
                "ok": True,
                "user": {
                    "id": row["id"],
                    "username": row["username"],
                    "display_name": row["display_name"],
                    "email": row["email"],
                },
            }
        )
    )
    conn.close()


def cmd_create_project(args):
    repo_root = Path.cwd()
    db_path = (repo_root / args.db).resolve()
    conn = connect_db(db_path)
    init_schema(conn)
    user = ensure_user(conn, args.username, args.user_display_name, None)
    slug = slugify(args.slug or args.name)
    row = ensure_project(conn, user["id"], slug, args.name, args.description)
    print(
        to_json(
            {
                "ok": True,
                "project": {
                    "id": row["id"],
                    "slug": row["slug"],
                    "name": row["name"],
                    "user_id": row["user_id"],
                },
            }
        )
    )
    conn.close()


def cmd_list_projects(args):
    repo_root = Path.cwd()
    db_path = (repo_root / args.db).resolve()
    conn = connect_db(db_path)
    init_schema(conn)
    sql = """
      SELECT p.id, p.slug, p.name, p.status, p.created_at, p.updated_at, u.username
      FROM projects p
      JOIN users u ON u.id = p.user_id
    """
    params = []
    if args.username:
        sql += " WHERE u.username = ?"
        params.append(slugify(args.username))
    sql += " ORDER BY p.updated_at DESC, p.created_at DESC"
    rows = conn.execute(sql, tuple(params)).fetchall()
    print(
        to_json(
            {
                "ok": True,
                "count": len(rows),
                "projects": [
                    {
                        "id": r["id"],
                        "slug": r["slug"],
                        "name": r["name"],
                        "status": r["status"],
                        "username": r["username"],
                        "created_at": r["created_at"],
                        "updated_at": r["updated_at"],
                    }
                    for r in rows
                ],
            }
        )
    )
    conn.close()


def cmd_ingest_run(args):
    repo_root = Path.cwd()
    db_path = (repo_root / args.db).resolve()
    run_log_path = (repo_root / args.run_log).resolve()
    if not run_log_path.exists():
        raise SystemExit(f"Run log not found: {run_log_path}")

    conn = connect_db(db_path)
    init_schema(conn)

    user = ensure_user(conn, args.username, args.user_display_name, None)
    project = get_project(conn, args.project_id, args.project_slug)
    if not project and args.create_project_if_missing:
        if not args.project_slug:
            raise SystemExit("--project-slug is required when --create-project-if-missing is true")
        project = ensure_project(conn, user["id"], args.project_slug, args.project_name or args.project_slug)
    if not project:
        raise SystemExit("Project not found. Use --project-id or --project-slug.")

    result = ingest_run(conn, repo_root, project, run_log_path, args.compute_hashes)
    print(to_json({"ok": True, "project_slug": project["slug"], **result}))
    conn.close()


def cmd_export_project(args):
    repo_root = Path.cwd()
    db_path = (repo_root / args.db).resolve()
    conn = connect_db(db_path)
    init_schema(conn)
    project = require_project(conn, args.project_id, args.project_slug)
    storage_payload = project_storage_payload(repo_root, project)
    source_files_root = Path(storage_payload["storage"]["local"]["project_root"])

    if args.output:
        output_path = (repo_root / args.output).resolve()
    else:
        stamp = now_iso().replace(":", "-")
        output_path = (repo_root / "generated" / "exports" / f"{project['slug']}_{stamp}.tar.gz").resolve()

    result = export_project_package(conn, db_path, repo_root, project, source_files_root, output_path, args.include_files)
    print(to_json({"ok": True, "project_slug": project["slug"], **result}))
    conn.close()


def build_parser():
    parser = argparse.ArgumentParser(description="IAT backend data service CLI")
    parser.add_argument("--db", default=DEFAULT_DB, help=f"SQLite database path (default: {DEFAULT_DB})")
    sub = parser.add_subparsers(dest="cmd", required=True)

    def add_project_ref(p):
        p.add_argument("--project-id", default="", help="Project id")
        p.add_argument("--project-slug", default="", help="Project slug")

    p_init = sub.add_parser("init-db", help="Create database schema")
    p_init.add_argument("--with-default-user", action="store_true", help="Ensure default local user")
    p_init.set_defaults(func=cmd_init_db)

    p_migrate = sub.add_parser("migrate", help="Apply/verify schema migrations")
    p_migrate.set_defaults(func=cmd_migrate)

    p_user = sub.add_parser("ensure-user", help="Create or update a user")
    p_user.add_argument("--username", required=True, help="Stable user handle")
    p_user.add_argument("--display-name", required=True, help="Display name")
    p_user.add_argument("--email", default=None, help="Optional email")
    p_user.set_defaults(func=cmd_ensure_user)

    p_project = sub.add_parser("create-project", help="Create or update a project for a user")
    p_project.add_argument("--username", default="local", help="Owner username")
    p_project.add_argument("--user-display-name", default="Local User", help="Owner display name")
    p_project.add_argument("--name", required=True, help="Project name")
    p_project.add_argument("--slug", default="", help="Project slug (auto from name if omitted)")
    p_project.add_argument("--description", default="", help="Project description")
    p_project.set_defaults(func=cmd_create_project)

    p_list = sub.add_parser("list-projects", help="List projects")
    p_list.add_argument("--username", default="", help="Filter by username")
    p_list.set_defaults(func=cmd_list_projects)

    p_ingest = sub.add_parser("ingest-run", help="Ingest run log JSON into database")
    p_ingest.add_argument("--run-log", required=True, help="Path to run_*.json")
    p_ingest.add_argument("--project-id", default="", help="Project id")
    p_ingest.add_argument("--project-slug", default="", help="Project slug")
    p_ingest.add_argument("--project-name", default="", help="Project name for auto-create")
    p_ingest.add_argument("--username", default="local", help="Owner username for auto-create")
    p_ingest.add_argument("--user-display-name", default="Local User", help="Owner display name")
    p_ingest.add_argument(
        "--create-project-if-missing",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="Create project if missing",
    )
    p_ingest.add_argument(
        "--compute-hashes",
        action=argparse.BooleanOptionalAction,
        default=False,
        help="Compute SHA256 for ingested files",
    )
    p_ingest.set_defaults(func=cmd_ingest_run)

    p_export = sub.add_parser("export-project", help="Export one project into project-scoped package")
    add_project_ref(p_export)
    p_export.add_argument("--output", default="", help="Output .tar.gz path or directory")
    p_export.add_argument(
        "--include-files",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="Include generated/projects/<slug> files into export package",
    )
    p_export.set_defaults(func=cmd_export_project)

    p_get_storage = sub.add_parser("get-project-storage", help="Get resolved project storage configuration")
    add_project_ref(p_get_storage)
    p_get_storage.set_defaults(func=cmd_get_project_storage)

    p_set_local = sub.add_parser("set-project-storage-local", help="Configure local storage for a project")
    add_project_ref(p_set_local)
    p_set_local.add_argument("--base-dir", default="", help="Base dir for all projects, e.g. generated/projects")
    p_set_local.add_argument(
        "--project-root",
        default="",
        help="Explicit absolute or repo-relative root for this project (overrides base-dir)",
    )
    p_set_local.set_defaults(func=cmd_set_project_storage_local)

    p_set_s3 = sub.add_parser("set-project-storage-s3", help="Configure S3 storage for a project")
    add_project_ref(p_set_s3)
    p_set_s3.add_argument("--enabled", default=None, help="true|false")
    p_set_s3.add_argument("--bucket", default=None, help="S3 bucket")
    p_set_s3.add_argument("--prefix", default=None, help="S3 prefix root")
    p_set_s3.add_argument("--region", default=None, help="AWS region")
    p_set_s3.add_argument("--profile", default=None, help="AWS CLI profile")
    p_set_s3.add_argument("--endpoint-url", default=None, help="Optional S3-compatible endpoint")
    p_set_s3.set_defaults(func=cmd_set_project_storage_s3)

    p_sync_s3 = sub.add_parser("sync-project-s3", help="Sync project local files to configured S3 destination")
    add_project_ref(p_sync_s3)
    p_sync_s3.add_argument("--dry-run", action="store_true", help="Run aws s3 sync with --dryrun")
    p_sync_s3.add_argument("--delete", action="store_true", help="Propagate deletions to destination")
    p_sync_s3.add_argument(
        "--allow-missing-local",
        action="store_true",
        help="Do not fail when local project root does not exist",
    )
    p_sync_s3.set_defaults(func=cmd_sync_project_s3)

    p_set_secret = sub.add_parser("set-project-secret", help="Store encrypted API secret for a project")
    add_project_ref(p_set_secret)
    p_set_secret.add_argument("--provider-code", required=True, help="Provider code (openai, removebg, ...)")
    p_set_secret.add_argument("--secret-name", required=True, help="Secret name (api_key, token, ...)")
    p_set_secret.add_argument("--secret-value", required=True, help="Secret plaintext value")
    p_set_secret.set_defaults(func=cmd_set_project_secret)

    p_list_secrets = sub.add_parser("list-project-secrets", help="List project secrets (masked)")
    add_project_ref(p_list_secrets)
    p_list_secrets.add_argument(
        "--include-mask",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="Include masked value in response",
    )
    p_list_secrets.set_defaults(func=cmd_list_project_secrets)

    p_delete_secret = sub.add_parser("delete-project-secret", help="Delete project secret by provider/name")
    add_project_ref(p_delete_secret)
    p_delete_secret.add_argument("--provider-code", required=True, help="Provider code")
    p_delete_secret.add_argument("--secret-name", required=True, help="Secret name")
    p_delete_secret.set_defaults(func=cmd_delete_project_secret)

    return parser


def main():
    parser = build_parser()
    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
