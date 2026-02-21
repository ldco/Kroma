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


RUNTIME_BASE_DIR = "var"
DEFAULT_DB = f"{RUNTIME_BASE_DIR}/backend/app.db"
DEFAULT_MASTER_KEY_FILE = f"{RUNTIME_BASE_DIR}/backend/master.key"
DEFAULT_PROJECTS_BASE_DIR = f"{RUNTIME_BASE_DIR}/projects"
DEFAULT_EXPORTS_BASE_DIR = f"{RUNTIME_BASE_DIR}/exports"
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


def table_exists(conn: sqlite3.Connection, table_name: str) -> bool:
    row = conn.execute(
        "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?",
        (table_name,),
    ).fetchone()
    return bool(row)


def row_has_key(row: sqlite3.Row | None, key: str) -> bool:
    return bool(row) and key in row.keys()


def _settings_storage_defaults(project_slug: str) -> dict:
    return {
        "local": {
            "base_dir": DEFAULT_PROJECTS_BASE_DIR,
            "project_root": "",
        },
        "s3": {
            "enabled": False,
            "bucket": "",
            "prefix": "iat-projects",
            "region": "",
            "profile": "",
            "endpoint_url": "",
        },
        "project_slug": project_slug,
    }


def _settings_to_storage_columns(settings: dict, project_slug: str) -> dict:
    resolved = resolve_storage_settings(settings, project_slug)
    local = resolved.get("local", {})
    s3 = resolved.get("s3", {})
    return {
        "local_base_dir": (
            str(local.get("base_dir") or DEFAULT_PROJECTS_BASE_DIR).strip() or DEFAULT_PROJECTS_BASE_DIR
        ),
        "local_project_root": str(local.get("project_root") or "").strip() or None,
        "s3_enabled": 1 if bool(s3.get("enabled")) else 0,
        "s3_bucket": str(s3.get("bucket") or "").strip() or None,
        "s3_prefix": str(s3.get("prefix") or "").strip() or None,
        "s3_region": str(s3.get("region") or "").strip() or None,
        "s3_profile": str(s3.get("profile") or "").strip() or None,
        "s3_endpoint_url": str(s3.get("endpoint_url") or "").strip() or None,
    }


def _storage_columns_to_settings(storage_row, project_slug: str) -> dict:
    base = _settings_storage_defaults(project_slug)
    if not storage_row:
        return base
    local = base["local"]
    s3 = base["s3"]
    local["base_dir"] = (
        str(storage_row["local_base_dir"] or DEFAULT_PROJECTS_BASE_DIR).strip() or DEFAULT_PROJECTS_BASE_DIR
    )
    local["project_root"] = str(storage_row["local_project_root"] or "").strip()
    s3["enabled"] = bool(int(storage_row["s3_enabled"] or 0))
    s3["bucket"] = str(storage_row["s3_bucket"] or "").strip()
    s3["prefix"] = str(storage_row["s3_prefix"] or "").strip() or "iat-projects"
    s3["region"] = str(storage_row["s3_region"] or "").strip()
    s3["profile"] = str(storage_row["s3_profile"] or "").strip()
    s3["endpoint_url"] = str(storage_row["s3_endpoint_url"] or "").strip()
    return base


def upsert_project_storage_from_settings(conn: sqlite3.Connection, project_row, settings: dict):
    if not project_row:
        return
    cols = _settings_to_storage_columns(settings, project_row["slug"])
    ts = now_iso()
    existing = conn.execute(
        "SELECT id FROM project_storage WHERE project_id = ?",
        (project_row["id"],),
    ).fetchone()
    if existing:
        conn.execute(
            """
            UPDATE project_storage
            SET local_base_dir = ?,
                local_project_root = ?,
                s3_enabled = ?,
                s3_bucket = ?,
                s3_prefix = ?,
                s3_region = ?,
                s3_profile = ?,
                s3_endpoint_url = ?,
                updated_at = ?
            WHERE id = ?
            """,
            (
                cols["local_base_dir"],
                cols["local_project_root"],
                cols["s3_enabled"],
                cols["s3_bucket"],
                cols["s3_prefix"],
                cols["s3_region"],
                cols["s3_profile"],
                cols["s3_endpoint_url"],
                ts,
                existing["id"],
            ),
        )
        return
    conn.execute(
        """
        INSERT INTO project_storage
          (id, project_id, local_base_dir, local_project_root, s3_enabled, s3_bucket,
           s3_prefix, s3_region, s3_profile, s3_endpoint_url, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            uid(),
            project_row["id"],
            cols["local_base_dir"],
            cols["local_project_root"],
            cols["s3_enabled"],
            cols["s3_bucket"],
            cols["s3_prefix"],
            cols["s3_region"],
            cols["s3_profile"],
            cols["s3_endpoint_url"],
            ts,
            ts,
        ),
    )


def _sync_users_to_app_users(conn: sqlite3.Connection):
    if not table_exists(conn, "users") or not table_exists(conn, "app_users"):
        return
    rows = conn.execute(
        """
        SELECT id, username, display_name, email, is_active, created_at, updated_at
        FROM users
        """
    ).fetchall()
    for r in rows:
        conn.execute(
            """
            INSERT INTO app_users (id, username, display_name, email, is_active, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
              username = excluded.username,
              display_name = excluded.display_name,
              email = excluded.email,
              is_active = excluded.is_active,
              updated_at = excluded.updated_at
            """,
            (r["id"], r["username"], r["display_name"], r["email"], r["is_active"], r["created_at"], r["updated_at"]),
        )


def _sync_project_owner_columns(conn: sqlite3.Connection):
    if not table_exists(conn, "projects"):
        return
    if table_has_column(conn, "projects", "owner_user_id"):
        conn.execute(
            """
            UPDATE projects
            SET owner_user_id = user_id
            WHERE owner_user_id IS NULL OR owner_user_id = ''
            """
        )
    conn.execute(
        """
        UPDATE projects
        SET user_id = owner_user_id
        WHERE (user_id IS NULL OR user_id = '') AND owner_user_id IS NOT NULL
        """
    )


def _sync_canonical_columns(conn: sqlite3.Connection):
    conn.execute("UPDATE assets SET kind = asset_kind WHERE kind IS NULL OR kind = ''")
    conn.execute("UPDATE assets SET storage_uri = rel_path WHERE storage_uri IS NULL OR storage_uri = ''")
    conn.execute("UPDATE assets SET metadata_json = meta_json WHERE metadata_json IS NULL OR metadata_json = ''")
    conn.execute("UPDATE assets SET asset_kind = kind WHERE (asset_kind IS NULL OR asset_kind = '') AND kind IS NOT NULL")
    conn.execute(
        "UPDATE assets SET rel_path = storage_uri WHERE (rel_path IS NULL OR rel_path = '') AND storage_uri IS NOT NULL"
    )
    conn.execute(
        """
        UPDATE assets
        SET meta_json = metadata_json
        WHERE (meta_json IS NULL OR meta_json = '' OR meta_json = '{}') AND metadata_json IS NOT NULL
        """
    )

    conn.execute("UPDATE runs SET run_mode = mode WHERE run_mode IS NULL OR run_mode = ''")
    conn.execute("UPDATE runs SET model_name = model WHERE model_name IS NULL OR model_name = ''")
    conn.execute(
        """
        UPDATE runs
        SET settings_snapshot_json = meta_json
        WHERE settings_snapshot_json IS NULL OR settings_snapshot_json = ''
        """
    )

    conn.execute(
        """
        UPDATE run_jobs
        SET selected_candidate_index = selected_candidate
        WHERE selected_candidate_index IS NULL AND selected_candidate IS NOT NULL
        """
    )
    conn.execute(
        """
        UPDATE project_exports
        SET sha256 = export_sha256
        WHERE (sha256 IS NULL OR sha256 = '') AND export_sha256 IS NOT NULL
        """
    )
    conn.execute(
        """
        UPDATE project_api_secrets
        SET kms_key_ref = key_ref
        WHERE kms_key_ref IS NULL OR kms_key_ref = ''
        """
    )
    conn.execute(
        """
        UPDATE project_api_secrets
        SET key_ref = COALESCE(key_ref, kms_key_ref, 'local-master')
        WHERE key_ref IS NULL OR key_ref = ''
        """
    )
    conn.execute(
        """
        UPDATE style_guides
        SET rules_json = specs_json
        WHERE (rules_json IS NULL OR rules_json = '' OR rules_json = '{}')
          AND specs_json IS NOT NULL
        """
    )
    conn.execute(
        """
        UPDATE provider_accounts
        SET config_json = meta_json
        WHERE (config_json IS NULL OR config_json = '' OR config_json = '{}')
          AND meta_json IS NOT NULL
        """
    )
    conn.execute(
        """
        UPDATE provider_accounts
        SET meta_json = config_json
        WHERE (meta_json IS NULL OR meta_json = '' OR meta_json = '{}')
          AND config_json IS NOT NULL
        """
    )
    conn.execute(
        """
        UPDATE provider_accounts
        SET is_enabled = 1
        WHERE is_enabled IS NULL
        """
    )
    conn.execute(
        """
        UPDATE reference_sets
        SET name = title
        WHERE (name IS NULL OR name = '')
          AND title IS NOT NULL
        """
    )
    conn.execute(
        """
        UPDATE reference_sets
        SET kind = 'other'
        WHERE kind IS NULL OR kind = ''
        """
    )
    ref_rows = conn.execute(
        """
        SELECT id, notes
        FROM reference_sets
        WHERE (metadata_json IS NULL OR metadata_json = '' OR metadata_json = '{}')
          AND notes IS NOT NULL
          AND notes != ''
        """
    ).fetchall()
    for row in ref_rows:
        conn.execute(
            "UPDATE reference_sets SET metadata_json = ? WHERE id = ?",
            (to_json({"notes": row["notes"]}), row["id"]),
        )


def _find_asset_id_by_uri(conn: sqlite3.Connection, project_id: str, storage_uri: str | None):
    path = normalize_rel_path(storage_uri or "")
    if not path:
        return None
    row = conn.execute(
        """
        SELECT id
        FROM assets
        WHERE project_id = ? AND (storage_uri = ? OR rel_path = ?)
        ORDER BY created_at DESC
        LIMIT 1
        """,
        (project_id, path, path),
    ).fetchone()
    return row["id"] if row else None


def _ensure_asset_id_for_uri(
    conn: sqlite3.Connection,
    project_id: str,
    storage_uri: str | None,
    asset_kind: str,
    run_id: str | None = None,
    job_id: str | None = None,
    candidate_id: str | None = None,
):
    clean_uri = normalize_rel_path(storage_uri or "")
    if not clean_uri:
        return None
    existing = conn.execute(
        """
        SELECT id
        FROM assets
        WHERE project_id = ? AND (storage_uri = ? OR rel_path = ?)
        ORDER BY created_at DESC
        LIMIT 1
        """,
        (project_id, clean_uri, clean_uri),
    ).fetchone()
    if existing:
        meta = to_json({"source": "phase3_backfill"})
        conn.execute(
            """
            UPDATE assets
            SET run_id = COALESCE(run_id, ?),
                job_id = COALESCE(job_id, ?),
                candidate_id = COALESCE(candidate_id, ?),
                kind = COALESCE(NULLIF(kind, ''), ?),
                asset_kind = COALESCE(NULLIF(asset_kind, ''), ?),
                storage_uri = COALESCE(NULLIF(storage_uri, ''), ?),
                rel_path = COALESCE(NULLIF(rel_path, ''), ?),
                metadata_json = CASE WHEN metadata_json IS NULL OR metadata_json = '' THEN ? ELSE metadata_json END,
                meta_json = CASE WHEN meta_json IS NULL OR meta_json = '' THEN ? ELSE meta_json END
            WHERE id = ?
            """,
            (
                run_id,
                job_id,
                candidate_id,
                asset_kind,
                asset_kind,
                clean_uri,
                clean_uri,
                meta,
                meta,
                existing["id"],
            ),
        )
        return existing["id"]

    ts = now_iso()
    meta = to_json({"source": "phase3_backfill"})
    asset_id = uid()
    conn.execute(
        """
        INSERT INTO assets
          (id, project_id, run_id, job_id, candidate_id, asset_kind, kind, rel_path, storage_uri,
           sha256, meta_json, metadata_json, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, ?, ?, ?)
        """,
        (
            asset_id,
            project_id,
            run_id,
            job_id,
            candidate_id,
            asset_kind,
            asset_kind,
            clean_uri,
            clean_uri,
            meta,
            meta,
            ts,
        ),
    )
    return asset_id


def _upsert_asset_link(
    conn: sqlite3.Connection,
    project_id: str,
    parent_asset_id: str | None,
    child_asset_id: str | None,
    link_type: str = "derived_from",
):
    if not parent_asset_id or not child_asset_id or parent_asset_id == child_asset_id:
        return
    safe_type = str(link_type or "derived_from").strip().lower() or "derived_from"
    if safe_type not in {"derived_from", "variant_of", "mask_for", "reference_of"}:
        safe_type = "derived_from"
    conn.execute(
        """
        INSERT INTO asset_links
          (id, project_id, parent_asset_id, child_asset_id, link_type, created_at)
        VALUES (?, ?, ?, ?, ?, ?)
        ON CONFLICT(parent_asset_id, child_asset_id, link_type) DO NOTHING
        """,
        (uid(), project_id, parent_asset_id, child_asset_id, safe_type, now_iso()),
    )


def _sync_run_candidates(conn: sqlite3.Connection):
    if not table_exists(conn, "run_candidates") or not table_exists(conn, "run_job_candidates"):
        return
    rows = conn.execute(
        """
        SELECT c.*, j.run_id, r.project_id
        FROM run_job_candidates c
        JOIN run_jobs j ON j.id = c.job_id
        JOIN runs r ON r.id = j.run_id
        """
    ).fetchall()
    for r in rows:
        output_asset_id = _ensure_asset_id_for_uri(
            conn,
            r["project_id"],
            r["output_path"],
            "candidate_output",
            run_id=r["run_id"],
            job_id=r["job_id"],
            candidate_id=r["id"],
        )
        final_asset_id = _ensure_asset_id_for_uri(
            conn,
            r["project_id"],
            r["final_output_path"],
            "candidate_final_output",
            run_id=r["run_id"],
            job_id=r["job_id"],
            candidate_id=r["id"],
        )
        if not final_asset_id and r["final_output_path"] and r["final_output_path"] == r["output_path"]:
            final_asset_id = output_asset_id
        _upsert_asset_link(conn, r["project_id"], output_asset_id, final_asset_id, "derived_from")
        conn.execute(
            """
            INSERT INTO run_candidates
              (id, job_id, candidate_index, status, output_asset_id, final_asset_id,
               rank_hard_failures, rank_soft_warnings, rank_avg_chroma_exceed, meta_json, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
              job_id = excluded.job_id,
              candidate_index = excluded.candidate_index,
              status = excluded.status,
              output_asset_id = excluded.output_asset_id,
              final_asset_id = excluded.final_asset_id,
              rank_hard_failures = excluded.rank_hard_failures,
              rank_soft_warnings = excluded.rank_soft_warnings,
              rank_avg_chroma_exceed = excluded.rank_avg_chroma_exceed,
              meta_json = excluded.meta_json
            """,
            (
                r["id"],
                r["job_id"],
                r["candidate_index"],
                r["status"],
                output_asset_id,
                final_asset_id,
                r["rank_hard_failures"],
                r["rank_soft_warnings"],
                r["rank_avg_chroma_exceed"],
                r["meta_json"],
                r["created_at"],
            ),
        )


def _sync_run_job_final_assets(conn: sqlite3.Connection):
    if not table_exists(conn, "run_jobs") or not table_exists(conn, "run_candidates"):
        return
    rows = conn.execute(
        """
        SELECT j.id,
               j.run_id,
               j.selected_candidate_index,
               j.final_output,
               j.final_asset_id,
               r.project_id
        FROM run_jobs j
        JOIN runs r ON r.id = j.run_id
        """
    ).fetchall()
    for row in rows:
        job_id = row["id"]
        project_id = row["project_id"]
        run_id = row["run_id"]
        selected_idx = row["selected_candidate_index"]
        final_output = normalize_rel_path(str(row["final_output"] or ""))
        final_asset_id = row["final_asset_id"]

        candidate_parent_asset_id = None
        if selected_idx is not None:
            selected = conn.execute(
                """
                SELECT output_asset_id, final_asset_id
                FROM run_candidates
                WHERE job_id = ? AND candidate_index = ?
                ORDER BY created_at DESC
                LIMIT 1
                """,
                (job_id, selected_idx),
            ).fetchone()
            if selected:
                candidate_parent_asset_id = selected["final_asset_id"] or selected["output_asset_id"]
                if not final_asset_id:
                    final_asset_id = candidate_parent_asset_id

        if not final_asset_id and final_output:
            final_asset_id = _ensure_asset_id_for_uri(
                conn,
                project_id,
                final_output,
                "job_final_output",
                run_id=run_id,
                job_id=job_id,
            )

        if final_asset_id and not final_output:
            asset = conn.execute(
                """
                SELECT COALESCE(storage_uri, rel_path) AS uri
                FROM assets
                WHERE id = ?
                """,
                (final_asset_id,),
            ).fetchone()
            if asset:
                final_output = normalize_rel_path(str(asset["uri"] or ""))

        if final_asset_id or final_output:
            conn.execute(
                """
                UPDATE run_jobs
                SET final_asset_id = COALESCE(?, final_asset_id),
                    final_output = COALESCE(NULLIF(final_output, ''), ?)
                WHERE id = ?
                """,
                (final_asset_id, final_output or None, job_id),
            )
        _upsert_asset_link(conn, project_id, candidate_parent_asset_id, final_asset_id, "derived_from")


def _sync_project_export_asset_fk(conn: sqlite3.Connection):
    if not table_exists(conn, "project_exports"):
        return
    rows = conn.execute(
        """
        SELECT id, project_id, export_path, export_asset_id
        FROM project_exports
        """
    ).fetchall()
    for row in rows:
        export_asset_id = row["export_asset_id"]
        export_path = normalize_rel_path(str(row["export_path"] or ""))
        if not export_asset_id and export_path:
            export_asset_id = _ensure_asset_id_for_uri(
                conn,
                row["project_id"],
                export_path,
                "export",
            )
            conn.execute(
                """
                UPDATE project_exports
                SET export_asset_id = ?
                WHERE id = ?
                """,
                (export_asset_id, row["id"]),
            )


def _seed_asset_links_from_run_graph(conn: sqlite3.Connection):
    if not table_exists(conn, "run_candidates") or not table_exists(conn, "run_jobs"):
        return
    candidate_links = conn.execute(
        """
        SELECT rc.output_asset_id, rc.final_asset_id, r.project_id
        FROM run_candidates rc
        JOIN run_jobs j ON j.id = rc.job_id
        JOIN runs r ON r.id = j.run_id
        WHERE rc.output_asset_id IS NOT NULL
          AND rc.final_asset_id IS NOT NULL
          AND rc.output_asset_id != rc.final_asset_id
        """
    ).fetchall()
    for row in candidate_links:
        _upsert_asset_link(conn, row["project_id"], row["output_asset_id"], row["final_asset_id"], "derived_from")

    job_rows = conn.execute(
        """
        SELECT j.id, j.final_asset_id, j.selected_candidate_index, r.project_id
        FROM run_jobs j
        JOIN runs r ON r.id = j.run_id
        WHERE j.final_asset_id IS NOT NULL
        """
    ).fetchall()
    for row in job_rows:
        if row["selected_candidate_index"] is None:
            continue
        selected = conn.execute(
            """
            SELECT output_asset_id, final_asset_id
            FROM run_candidates
            WHERE job_id = ? AND candidate_index = ?
            ORDER BY created_at DESC
            LIMIT 1
            """,
            (row["id"], row["selected_candidate_index"]),
        ).fetchone()
        if not selected:
            continue
        parent_asset_id = selected["final_asset_id"] or selected["output_asset_id"]
        _upsert_asset_link(conn, row["project_id"], parent_asset_id, row["final_asset_id"], "derived_from")


def _sync_project_storage_from_projects(conn: sqlite3.Connection):
    if not table_exists(conn, "project_storage"):
        return
    projects = conn.execute("SELECT * FROM projects").fetchall()
    for p in projects:
        settings = load_project_settings(p)
        upsert_project_storage_from_settings(conn, p, settings)


def _apply_phase1_backfills(conn: sqlite3.Connection):
    _sync_users_to_app_users(conn)
    _sync_project_owner_columns(conn)
    _sync_canonical_columns(conn)
    _sync_project_storage_from_projects(conn)
    _sync_run_candidates(conn)


def _sync_phase2_columns(conn: sqlite3.Connection):
    conn.execute(
        """
        UPDATE quality_reports
        SET job_id = run_job_id
        WHERE job_id IS NULL AND run_job_id IS NOT NULL
        """
    )
    conn.execute(
        """
        UPDATE quality_reports
        SET candidate_id = run_job_candidate_id
        WHERE candidate_id IS NULL AND run_job_candidate_id IS NOT NULL
        """
    )
    conn.execute(
        """
        UPDATE quality_reports
        SET report_type = 'human_review'
        WHERE report_type IS NULL OR report_type = ''
        """
    )
    quality_rows = conn.execute(
        """
        SELECT id, rating, notes
        FROM quality_reports
        WHERE summary_json IS NULL OR summary_json = '' OR summary_json = '{}'
        """
    ).fetchall()
    for row in quality_rows:
        conn.execute(
            "UPDATE quality_reports SET summary_json = ? WHERE id = ?",
            (to_json({"rating": row["rating"], "notes": row["notes"]}), row["id"]),
        )

    conn.execute(
        """
        UPDATE cost_events
        SET provider_code = 'unknown'
        WHERE provider_code IS NULL OR provider_code = ''
        """
    )
    conn.execute(
        """
        UPDATE cost_events
        SET operation_code = COALESCE(NULLIF(event_type, ''), 'legacy_event')
        WHERE operation_code IS NULL OR operation_code = ''
        """
    )
    conn.execute(
        """
        UPDATE cost_events
        SET cost_usd = (COALESCE(amount_cents, 0) / 100.0)
        WHERE cost_usd IS NULL
        """
    )
    cost_rows = conn.execute(
        """
        SELECT id, notes
        FROM cost_events
        WHERE meta_json IS NULL OR meta_json = ''
        """
    ).fetchall()
    for row in cost_rows:
        payload = {"notes": row["notes"]} if row["notes"] else {}
        conn.execute("UPDATE cost_events SET meta_json = ? WHERE id = ?", (to_json(payload), row["id"]))
    conn.execute(
        """
        UPDATE cost_events
        SET event_type = COALESCE(NULLIF(event_type, ''), operation_code, 'legacy_event')
        WHERE event_type IS NULL OR event_type = ''
        """
    )
    conn.execute(
        """
        UPDATE cost_events
        SET amount_cents = CAST(ROUND(COALESCE(cost_usd, 0) * 100.0) AS INTEGER)
        WHERE amount_cents IS NULL
        """
    )

    conn.execute(
        """
        UPDATE audit_events
        SET actor_user_id = user_id
        WHERE actor_user_id IS NULL AND user_id IS NOT NULL
        """
    )
    conn.execute(
        """
        UPDATE audit_events
        SET event_code = COALESCE(NULLIF(action, ''), 'legacy_event')
        WHERE event_code IS NULL OR event_code = ''
        """
    )
    conn.execute(
        """
        UPDATE audit_events
        SET payload_json = COALESCE(NULLIF(details_json, ''), '{}')
        WHERE payload_json IS NULL OR payload_json = ''
        """
    )
    conn.execute(
        """
        UPDATE audit_events
        SET user_id = COALESCE(user_id, actor_user_id)
        WHERE user_id IS NULL AND actor_user_id IS NOT NULL
        """
    )
    conn.execute(
        """
        UPDATE audit_events
        SET action = COALESCE(NULLIF(action, ''), event_code, 'legacy_event')
        WHERE action IS NULL OR action = ''
        """
    )
    conn.execute(
        """
        UPDATE audit_events
        SET details_json = COALESCE(NULLIF(details_json, ''), payload_json, '{}')
        WHERE details_json IS NULL OR details_json = ''
        """
    )


def _seed_quality_reports_from_candidates(conn: sqlite3.Connection):
    rows = conn.execute(
        """
        SELECT c.id AS candidate_id,
               c.job_id AS job_id,
               j.run_id AS run_id,
               r.project_id AS project_id,
               c.status AS status,
               c.rank_hard_failures AS rank_hard_failures,
               c.rank_soft_warnings AS rank_soft_warnings,
               c.rank_avg_chroma_exceed AS rank_avg_chroma_exceed,
               c.meta_json AS meta_json,
               c.created_at AS created_at
        FROM run_job_candidates c
        JOIN run_jobs j ON j.id = c.job_id
        JOIN runs r ON r.id = j.run_id
        LEFT JOIN quality_reports q ON (q.run_job_candidate_id = c.id OR q.candidate_id = c.id)
        WHERE q.id IS NULL
        """
    ).fetchall()
    for row in rows:
        summary = {
            "status": row["status"],
            "rank": {
                "hard_failures": int(row["rank_hard_failures"] or 0),
                "soft_warnings": int(row["rank_soft_warnings"] or 0),
                "avg_chroma_exceed": float(row["rank_avg_chroma_exceed"] or 0.0),
            },
            "source": "phase2_backfill",
        }
        try:
            parsed = json.loads(row["meta_json"] or "{}")
            if isinstance(parsed, dict) and isinstance(parsed.get("output_guard"), dict):
                summary["output_guard"] = parsed["output_guard"]
        except Exception:
            pass
        insert_quality_report(
            conn,
            row["project_id"],
            row["run_id"],
            row["job_id"],
            row["candidate_id"],
            "output_guard",
            summary,
            created_at=row["created_at"] or now_iso(),
        )


def _apply_phase2_backfills(conn: sqlite3.Connection):
    _sync_phase2_columns(conn)
    _seed_quality_reports_from_candidates(conn)


def _apply_phase3_backfills(conn: sqlite3.Connection):
    _sync_run_candidates(conn)
    _sync_run_job_final_assets(conn)
    _sync_project_export_asset_fk(conn)
    _seed_asset_links_from_run_graph(conn)


def insert_quality_report(
    conn: sqlite3.Connection,
    project_id: str,
    run_id: str | None,
    job_id: str | None,
    candidate_id: str | None,
    report_type: str,
    summary: dict,
    created_at: str | None = None,
):
    ts = created_at or now_iso()
    summary_obj = summary if isinstance(summary, dict) else {"value": summary}
    rating_raw = summary_obj.get("rating", 0)
    try:
        rating = int(rating_raw)
    except Exception:
        rating = 0
    notes = str(summary_obj.get("notes", "") or "")
    conn.execute(
        """
        INSERT INTO quality_reports
          (id, project_id, run_id, run_job_id, run_job_candidate_id, job_id, candidate_id, report_type, summary_json, rating, notes, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            uid(),
            project_id,
            run_id,
            job_id,
            candidate_id,
            job_id,
            candidate_id,
            str(report_type or "output_guard"),
            to_json(summary_obj),
            rating,
            notes,
            ts,
        ),
    )


def insert_cost_event(
    conn: sqlite3.Connection,
    project_id: str,
    run_id: str | None,
    provider_code: str,
    operation_code: str,
    units: float,
    cost_usd: float,
    currency: str = "USD",
    meta: dict | None = None,
    created_at: str | None = None,
):
    ts = created_at or now_iso()
    safe_provider = str(provider_code or "unknown").strip() or "unknown"
    safe_operation = str(operation_code or "legacy_event").strip() or "legacy_event"
    safe_currency = str(currency or "USD").strip() or "USD"
    safe_meta = meta if isinstance(meta, dict) else {}
    try:
        safe_units = float(units or 0)
    except Exception:
        safe_units = 0.0
    try:
        safe_cost_usd = float(cost_usd or 0)
    except Exception:
        safe_cost_usd = 0.0
    amount_cents = int(round(safe_cost_usd * 100.0))
    notes = str(safe_meta.get("notes", "") or "")
    conn.execute(
        """
        INSERT INTO cost_events
          (id, project_id, run_id, amount_cents, currency, event_type, notes, provider_code, operation_code, units, cost_usd, meta_json, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            uid(),
            project_id,
            run_id,
            amount_cents,
            safe_currency,
            safe_operation,
            notes,
            safe_provider,
            safe_operation,
            safe_units,
            safe_cost_usd,
            to_json(safe_meta),
            ts,
        ),
    )


def extract_cost_event_rows(run_data: dict) -> list[dict]:
    events: list[dict] = []

    raw_events = run_data.get("cost_events")
    if isinstance(raw_events, list):
        for item in raw_events:
            if not isinstance(item, dict):
                continue
            provider_code = item.get("provider_code") or item.get("provider") or "unknown"
            operation_code = item.get("operation_code") or item.get("operation") or item.get("event_type") or "legacy_event"
            units = item.get("units") or item.get("quantity") or 0
            cost_usd = item.get("cost_usd")
            if cost_usd is None and item.get("amount_cents") is not None:
                try:
                    cost_usd = float(item.get("amount_cents")) / 100.0
                except Exception:
                    cost_usd = 0
            events.append(
                {
                    "provider_code": provider_code,
                    "operation_code": operation_code,
                    "units": units,
                    "cost_usd": cost_usd or 0,
                    "currency": item.get("currency") or "USD",
                    "meta": item,
                }
            )

    generation = run_data.get("generation")
    if isinstance(generation, dict):
        provider_code = generation.get("provider_code") or generation.get("provider") or "openai"
        operation_code = generation.get("operation_code") or "image_generation"
        units = generation.get("units") or generation.get("images") or generation.get("count") or 0
        cost_usd = generation.get("cost_usd")
        if cost_usd is None and generation.get("amount_cents") is not None:
            try:
                cost_usd = float(generation.get("amount_cents")) / 100.0
            except Exception:
                cost_usd = 0
        if cost_usd is not None:
            events.append(
                {
                    "provider_code": provider_code,
                    "operation_code": operation_code,
                    "units": units,
                    "cost_usd": cost_usd,
                    "currency": generation.get("currency") or "USD",
                    "meta": generation,
                }
            )

    if not events:
        top_level_cost = run_data.get("cost_usd")
        if top_level_cost is None and run_data.get("amount_cents") is not None:
            try:
                top_level_cost = float(run_data.get("amount_cents")) / 100.0
            except Exception:
                top_level_cost = None
        if top_level_cost is not None:
            events.append(
                {
                    "provider_code": "unknown",
                    "operation_code": "run_total",
                    "units": 1,
                    "cost_usd": top_level_cost,
                    "currency": run_data.get("currency") or "USD",
                    "meta": {"source": "run_log_top_level"},
                }
            )

    return events


def emit_audit_event(
    conn: sqlite3.Connection,
    project_id: str | None,
    actor_user_id: str | None,
    event_code: str,
    payload: dict | None = None,
    target_type: str | None = None,
    target_id: str | None = None,
):
    ts = now_iso()
    safe_code = str(event_code or "legacy_event").strip() or "legacy_event"
    payload_obj = payload if isinstance(payload, dict) else {}
    conn.execute(
        """
        INSERT INTO audit_events
          (id, project_id, user_id, actor_user_id, action, event_code, target_type, target_id, details_json, payload_json, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            uid(),
            project_id,
            actor_user_id,
            actor_user_id,
            safe_code,
            safe_code,
            target_type,
            target_id,
            to_json(payload_obj),
            to_json(payload_obj),
            ts,
        ),
    )


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

        CREATE TABLE IF NOT EXISTS app_users (
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

        CREATE TABLE IF NOT EXISTS run_candidates (
          id TEXT PRIMARY KEY,
          job_id TEXT NOT NULL,
          candidate_index INTEGER NOT NULL,
          status TEXT NOT NULL,
          output_asset_id TEXT,
          final_asset_id TEXT,
          rank_hard_failures INTEGER NOT NULL DEFAULT 0,
          rank_soft_warnings INTEGER NOT NULL DEFAULT 0,
          rank_avg_chroma_exceed REAL NOT NULL DEFAULT 0,
          meta_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          UNIQUE(job_id, candidate_index),
          FOREIGN KEY(job_id) REFERENCES run_jobs(id) ON DELETE CASCADE,
          FOREIGN KEY(output_asset_id) REFERENCES assets(id) ON DELETE SET NULL,
          FOREIGN KEY(final_asset_id) REFERENCES assets(id) ON DELETE SET NULL
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

        CREATE TABLE IF NOT EXISTS project_storage (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          total_bytes INTEGER NOT NULL DEFAULT 0,
          used_bytes INTEGER NOT NULL DEFAULT 0,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE(project_id),
          FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS provider_accounts (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          provider_code TEXT NOT NULL,
          api_key TEXT NOT NULL,
          meta_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE(project_id, provider_code),
          FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS style_guides (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          name TEXT NOT NULL,
          description TEXT NOT NULL DEFAULT '',
          specs_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS characters (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          code TEXT NOT NULL,
          name TEXT NOT NULL,
          bio TEXT NOT NULL DEFAULT '',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE(project_id, code),
          FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS reference_sets (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          title TEXT NOT NULL,
          notes TEXT NOT NULL DEFAULT '',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS reference_items (
          id TEXT PRIMARY KEY,
          reference_set_id TEXT NOT NULL,
          asset_id TEXT NOT NULL,
          notes TEXT NOT NULL DEFAULT '',
          created_at TEXT NOT NULL,
          UNIQUE(reference_set_id, asset_id),
          FOREIGN KEY(reference_set_id) REFERENCES reference_sets(id) ON DELETE CASCADE,
          FOREIGN KEY(asset_id) REFERENCES assets(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS asset_links (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          parent_asset_id TEXT NOT NULL,
          child_asset_id TEXT NOT NULL,
          link_type TEXT NOT NULL,
          created_at TEXT NOT NULL,
          UNIQUE(parent_asset_id, child_asset_id, link_type),
          FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE,
          FOREIGN KEY(parent_asset_id) REFERENCES assets(id) ON DELETE CASCADE,
          FOREIGN KEY(child_asset_id) REFERENCES assets(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS quality_reports (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          run_id TEXT,
          run_job_id TEXT,
          run_job_candidate_id TEXT,
          job_id TEXT,
          candidate_id TEXT,
          report_type TEXT NOT NULL DEFAULT 'output_guard',
          summary_json TEXT NOT NULL DEFAULT '{}',
          rating INTEGER NOT NULL DEFAULT 0,
          notes TEXT NOT NULL DEFAULT '',
          created_at TEXT NOT NULL,
          FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE,
          FOREIGN KEY(run_id) REFERENCES runs(id) ON DELETE SET NULL,
          FOREIGN KEY(run_job_id) REFERENCES run_jobs(id) ON DELETE SET NULL,
          FOREIGN KEY(run_job_candidate_id) REFERENCES run_job_candidates(id) ON DELETE SET NULL,
          FOREIGN KEY(job_id) REFERENCES run_jobs(id) ON DELETE SET NULL,
          FOREIGN KEY(candidate_id) REFERENCES run_candidates(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS prompt_templates (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          name TEXT NOT NULL,
          template_text TEXT NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS cost_events (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          run_id TEXT,
          amount_cents INTEGER NOT NULL,
          currency TEXT NOT NULL,
          event_type TEXT NOT NULL,
          notes TEXT NOT NULL DEFAULT '',
          provider_code TEXT NOT NULL DEFAULT 'unknown',
          operation_code TEXT NOT NULL DEFAULT 'legacy_event',
          units REAL NOT NULL DEFAULT 0,
          cost_usd REAL NOT NULL DEFAULT 0,
          meta_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE,
          FOREIGN KEY(run_id) REFERENCES runs(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS audit_events (
          id TEXT PRIMARY KEY,
          project_id TEXT,
          user_id TEXT,
          actor_user_id TEXT,
          action TEXT NOT NULL,
          event_code TEXT NOT NULL DEFAULT 'legacy_event',
          target_type TEXT,
          target_id TEXT,
          details_json TEXT NOT NULL DEFAULT '{}',
          payload_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE SET NULL,
          FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE SET NULL,
          FOREIGN KEY(actor_user_id) REFERENCES app_users(id) ON DELETE SET NULL
        );

        CREATE INDEX IF NOT EXISTS idx_project_storage_project ON project_storage(project_id);
        CREATE INDEX IF NOT EXISTS idx_provider_accounts_project ON provider_accounts(project_id);
        CREATE INDEX IF NOT EXISTS idx_style_guides_project ON style_guides(project_id);
        CREATE INDEX IF NOT EXISTS idx_characters_project ON characters(project_id);
        CREATE INDEX IF NOT EXISTS idx_reference_sets_project ON reference_sets(project_id);
        CREATE INDEX IF NOT EXISTS idx_reference_items_set ON reference_items(reference_set_id);
        CREATE INDEX IF NOT EXISTS idx_asset_links_parent ON asset_links(parent_asset_id);
        CREATE INDEX IF NOT EXISTS idx_asset_links_child ON asset_links(child_asset_id);
        CREATE INDEX IF NOT EXISTS idx_quality_reports_project ON quality_reports(project_id, created_at);
        CREATE INDEX IF NOT EXISTS idx_quality_reports_run ON quality_reports(run_id);
        CREATE INDEX IF NOT EXISTS idx_prompt_templates_project ON prompt_templates(project_id);
        CREATE INDEX IF NOT EXISTS idx_cost_events_project ON cost_events(project_id, created_at);
        CREATE INDEX IF NOT EXISTS idx_cost_events_run ON cost_events(run_id);
        CREATE INDEX IF NOT EXISTS idx_audit_events_project ON audit_events(project_id, created_at);
        CREATE INDEX IF NOT EXISTS idx_assets_sha256 ON assets(project_id, sha256);
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

    # canonical compatibility columns
    ensure_column(conn, "projects", "owner_user_id", "TEXT")
    ensure_column(conn, "assets", "kind", "TEXT")
    ensure_column(conn, "assets", "storage_uri", "TEXT")
    ensure_column(conn, "assets", "metadata_json", "TEXT NOT NULL DEFAULT '{}'")
    ensure_column(conn, "runs", "run_mode", "TEXT")
    ensure_column(conn, "runs", "model_name", "TEXT")
    ensure_column(conn, "runs", "settings_snapshot_json", "TEXT NOT NULL DEFAULT '{}'")
    ensure_column(conn, "run_jobs", "selected_candidate_index", "INTEGER")
    ensure_column(conn, "run_jobs", "final_asset_id", "TEXT")
    ensure_column(conn, "project_exports", "export_asset_id", "TEXT")
    ensure_column(conn, "project_exports", "sha256", "TEXT")
    ensure_column(conn, "project_api_secrets", "kms_key_ref", "TEXT")
    ensure_column(
        conn,
        "project_storage",
        "local_base_dir",
        f"TEXT NOT NULL DEFAULT '{DEFAULT_PROJECTS_BASE_DIR}'",
    )
    ensure_column(conn, "project_storage", "local_project_root", "TEXT")
    ensure_column(conn, "project_storage", "s3_enabled", "INTEGER NOT NULL DEFAULT 0")
    ensure_column(conn, "project_storage", "s3_bucket", "TEXT")
    ensure_column(conn, "project_storage", "s3_prefix", "TEXT")
    ensure_column(conn, "project_storage", "s3_region", "TEXT")
    ensure_column(conn, "project_storage", "s3_profile", "TEXT")
    ensure_column(conn, "project_storage", "s3_endpoint_url", "TEXT")
    ensure_column(conn, "style_guides", "rules_json", "TEXT NOT NULL DEFAULT '{}'")
    ensure_column(conn, "style_guides", "is_default", "INTEGER NOT NULL DEFAULT 0")
    ensure_column(conn, "characters", "identity_constraints_json", "TEXT NOT NULL DEFAULT '{}'")
    ensure_column(conn, "reference_sets", "name", "TEXT")
    ensure_column(conn, "reference_sets", "kind", "TEXT NOT NULL DEFAULT 'other'")
    ensure_column(conn, "reference_sets", "metadata_json", "TEXT NOT NULL DEFAULT '{}'")
    ensure_column(conn, "reference_items", "weight", "REAL NOT NULL DEFAULT 1.0")
    ensure_column(conn, "provider_accounts", "is_enabled", "INTEGER NOT NULL DEFAULT 1")
    ensure_column(conn, "provider_accounts", "config_json", "TEXT NOT NULL DEFAULT '{}'")
    ensure_column(conn, "quality_reports", "job_id", "TEXT")
    ensure_column(conn, "quality_reports", "candidate_id", "TEXT")
    ensure_column(conn, "quality_reports", "report_type", "TEXT NOT NULL DEFAULT 'output_guard'")
    ensure_column(conn, "quality_reports", "summary_json", "TEXT NOT NULL DEFAULT '{}'")
    ensure_column(conn, "cost_events", "provider_code", "TEXT NOT NULL DEFAULT 'unknown'")
    ensure_column(conn, "cost_events", "operation_code", "TEXT NOT NULL DEFAULT 'legacy_event'")
    ensure_column(conn, "cost_events", "units", "REAL NOT NULL DEFAULT 0")
    ensure_column(conn, "cost_events", "cost_usd", "REAL NOT NULL DEFAULT 0")
    ensure_column(conn, "cost_events", "meta_json", "TEXT NOT NULL DEFAULT '{}'")
    ensure_column(conn, "audit_events", "actor_user_id", "TEXT")
    ensure_column(conn, "audit_events", "event_code", "TEXT NOT NULL DEFAULT 'legacy_event'")
    ensure_column(conn, "audit_events", "payload_json", "TEXT NOT NULL DEFAULT '{}'")

    # additional additive columns used by current runtime
    ensure_column(conn, "assets", "storage_backend", "TEXT NOT NULL DEFAULT 'local'")
    ensure_column(conn, "assets", "mime_type", "TEXT")
    ensure_column(conn, "assets", "width", "INTEGER")
    ensure_column(conn, "assets", "height", "INTEGER")
    ensure_column(conn, "runs", "provider_code", "TEXT")
    ensure_column(conn, "runs", "started_at", "TEXT")
    ensure_column(conn, "runs", "finished_at", "TEXT")
    ensure_column(conn, "run_jobs", "prompt_text", "TEXT NOT NULL DEFAULT ''")
    ensure_column(conn, "project_exports", "format", "TEXT NOT NULL DEFAULT 'tar.gz'")

    conn.execute("CREATE INDEX IF NOT EXISTS idx_projects_owner_slug ON projects(owner_user_id, slug)")
    conn.execute("CREATE INDEX IF NOT EXISTS idx_runs_project_created ON runs(project_id, created_at DESC)")
    conn.execute("CREATE INDEX IF NOT EXISTS idx_run_jobs_run_status ON run_jobs(run_id, status)")
    conn.execute("CREATE INDEX IF NOT EXISTS idx_run_candidates_job_idx ON run_candidates(job_id, candidate_index)")
    conn.execute("CREATE INDEX IF NOT EXISTS idx_assets_project_kind_created ON assets(project_id, kind, created_at DESC)")
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS uq_assets_project_storage_uri ON assets(project_id, storage_uri) WHERE storage_uri IS NOT NULL"
    )
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS uq_projects_owner_slug ON projects(owner_user_id, slug) WHERE owner_user_id IS NOT NULL"
    )

    _apply_phase1_backfills(conn)
    _apply_phase2_backfills(conn)
    _apply_phase3_backfills(conn)

    record_migration(conn, "20260220_0001_base_schema", "base schema + chat + storage + exports")
    record_migration(conn, "20260220_0002_instruction_queue", "instruction retries/locks columns")
    record_migration(conn, "20260220_0003_project_api_secrets", "encrypted provider secret storage")
    record_migration(conn, "20260221_0004_provider_accounts", "provider_accounts table")
    record_migration(conn, "20260221_0006_creative_knowledge", "style_guides, characters, reference_sets, reference_items tables")
    record_migration(conn, "20260221_0007_assets_additive_cols", "assets: storage_backend, mime_type, width, height columns")
    record_migration(conn, "20260221_0008_asset_links", "asset_links table")
    record_migration(conn, "20260221_0009_runs_additive_cols", "runs: provider_code, started_at, finished_at columns")
    record_migration(conn, "20260221_0010_run_jobs_prompt_text", "run_jobs: prompt_text column")
    record_migration(conn, "20260221_0012_quality_reports", "quality_reports table")
    record_migration(conn, "20260221_0013_prompt_templates", "prompt_templates table")
    record_migration(conn, "20260221_0014_cost_events", "cost_events table")
    record_migration(conn, "20260221_0015_project_exports_format", "project_exports: format column")
    record_migration(conn, "20260221_0016_audit_events", "audit_events table")
    record_migration(conn, "20260221_0003_project_storage_table", "project_storage table (schema only, data migration deferred)")
    record_migration(conn, "20260221_0018_phase1_canonical_schema", "canonical columns/tables for app_users/run_candidates/owner_user_id")
    record_migration(conn, "20260221_0019_phase1_backfill", "canonical backfill for users/projects/assets/runs/jobs/storage/candidates")
    record_migration(conn, "20260221_0020_phase2_event_columns", "canonical quality_reports/cost_events/audit_events columns")
    record_migration(conn, "20260221_0021_phase2_backfill", "backfill canonical event columns from legacy fields")
    record_migration(conn, "20260221_0022_creative_schema_columns", "canonical creative columns for style_guides/characters/reference_sets/items")
    record_migration(conn, "20260221_0023_provider_account_columns", "provider_accounts: is_enabled + config_json canonical columns")
    record_migration(conn, "20260221_0024_phase3_asset_fk_backfill", "backfill asset FKs + derived asset_links for legacy rows")
    conn.commit()


def get_user_by_username(conn: sqlite3.Connection, username: str):
    row = conn.execute("SELECT * FROM app_users WHERE username = ?", (username,)).fetchone()
    if row:
        return row
    return conn.execute("SELECT * FROM users WHERE username = ?", (username,)).fetchone()


def get_user_by_id(conn: sqlite3.Connection, user_id: str):
    row = conn.execute("SELECT * FROM app_users WHERE id = ?", (user_id,)).fetchone()
    if row:
        return row
    return conn.execute("SELECT * FROM users WHERE id = ?", (user_id,)).fetchone()


def _upsert_user_dual(
    conn: sqlite3.Connection,
    user_id: str,
    username: str,
    display_name: str,
    email: str | None,
    ts: str,
):
    conn.execute(
        """
        INSERT INTO app_users (id, username, display_name, email, is_active, created_at, updated_at)
        VALUES (?, ?, ?, ?, 1, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
          username = excluded.username,
          display_name = excluded.display_name,
          email = excluded.email,
          is_active = 1,
          updated_at = excluded.updated_at
        """,
        (user_id, username, display_name, email, ts, ts),
    )
    conn.execute(
        """
        INSERT INTO users (id, username, display_name, email, is_active, created_at, updated_at)
        VALUES (?, ?, ?, ?, 1, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
          username = excluded.username,
          display_name = excluded.display_name,
          email = excluded.email,
          is_active = 1,
          updated_at = excluded.updated_at
        """,
        (user_id, username, display_name, email, ts, ts),
    )


def ensure_user(conn: sqlite3.Connection, username: str, display_name: str, email: str | None):
    username = slugify(username)
    ts = now_iso()
    row = get_user_by_username(conn, username)
    if row:
        _upsert_user_dual(conn, row["id"], username, display_name, email, ts)
        conn.commit()
        return get_user_by_id(conn, row["id"])

    legacy = conn.execute("SELECT * FROM users WHERE username = ?", (username,)).fetchone()
    new_id = legacy["id"] if legacy else uid()
    _upsert_user_dual(conn, new_id, username, display_name, email, ts)
    conn.commit()
    return get_user_by_id(conn, new_id)


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
    project_row = get_project(conn, project_id, None)
    conn.execute(
        "UPDATE projects SET settings_json = ?, updated_at = ? WHERE id = ?",
        (to_json(settings or {}), now_iso(), project_id),
    )
    if project_row:
        upsert_project_storage_from_settings(conn, project_row, settings or {})
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
            "base_dir": str(local.get("base_dir", DEFAULT_PROJECTS_BASE_DIR)).strip(),
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

    base_dir = str(local.get("base_dir", DEFAULT_PROJECTS_BASE_DIR)).strip() or DEFAULT_PROJECTS_BASE_DIR
    base_path = Path(base_dir)
    base_abs = base_path if base_path.is_absolute() else (repo_root / base_path).resolve()
    return (base_abs / project_slug).resolve()


def project_storage_payload(repo_root: Path, project_row, conn: sqlite3.Connection | None = None) -> dict:
    storage = None
    if conn and table_exists(conn, "project_storage"):
        storage_row = conn.execute(
            """
            SELECT *
            FROM project_storage
            WHERE project_id = ?
            """,
            (project_row["id"],),
        ).fetchone()
        if storage_row and row_has_key(storage_row, "local_base_dir"):
            storage = _storage_columns_to_settings(storage_row, project_row["slug"])
    if storage is None:
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
    owner_user_id: str,
    slug: str,
    name: str,
    description: str = "",
    status: str = "active",
):
    ts = now_iso()
    safe_slug = slugify(slug)
    row = conn.execute(
        "SELECT * FROM projects WHERE owner_user_id = ? AND slug = ?",
        (owner_user_id, safe_slug),
    ).fetchone()
    if not row:
        row = conn.execute(
            "SELECT * FROM projects WHERE user_id = ? AND slug = ?",
            (owner_user_id, safe_slug),
        ).fetchone()
    if row:
        conn.execute(
            """
            UPDATE projects
            SET name = ?, description = ?, status = ?, owner_user_id = ?, user_id = COALESCE(user_id, ?), updated_at = ?
            WHERE id = ?
            """,
            (name, description, status, owner_user_id, owner_user_id, ts, row["id"]),
        )
        conn.commit()
        return conn.execute("SELECT * FROM projects WHERE id = ?", (row["id"],)).fetchone()

    project_id = uid()
    conn.execute(
        """
        INSERT INTO projects (id, owner_user_id, user_id, slug, name, description, status, settings_json, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, '{}', ?, ?)
        """,
        (project_id, owner_user_id, owner_user_id, safe_slug, name, description, status, ts, ts),
    )
    project = conn.execute("SELECT * FROM projects WHERE id = ?", (project_id,)).fetchone()
    if project:
        upsert_project_storage_from_settings(conn, project, load_project_settings(project))
    conn.commit()
    return project


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
        return None

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
        """
        SELECT id
        FROM assets
        WHERE project_id = ? AND (rel_path = ? OR storage_uri = ?)
        ORDER BY created_at DESC
        LIMIT 1
        """,
        (project_id, clean_rel, clean_rel),
    ).fetchone()
    if existing:
        conn.execute(
            """
            UPDATE assets
            SET run_id = ?,
                job_id = ?,
                candidate_id = ?,
                asset_kind = ?,
                kind = ?,
                rel_path = ?,
                storage_uri = ?,
                sha256 = ?,
                meta_json = ?,
                metadata_json = ?,
                created_at = ?
            WHERE id = ?
            """,
            (
                run_id,
                job_id,
                candidate_id,
                asset_kind,
                asset_kind,
                clean_rel,
                clean_rel,
                file_hash,
                to_json(payload),
                to_json(payload),
                ts,
                existing["id"],
            ),
        )
        return existing["id"]

    conn.execute(
        """
        INSERT INTO assets
          (id, project_id, run_id, job_id, candidate_id, asset_kind, kind, rel_path, storage_uri, sha256, meta_json, metadata_json, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            uid(),
            project_id,
            run_id,
            job_id,
            candidate_id,
            asset_kind,
            asset_kind,
            clean_rel,
            clean_rel,
            file_hash,
            to_json(payload),
            to_json(payload),
            ts,
        ),
    )
    return conn.execute("SELECT id FROM assets WHERE project_id = ? AND storage_uri = ?", (project_id, clean_rel)).fetchone()[
        "id"
    ]


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
    run_mode = str(run_data.get("mode", ""))
    model_name = str(run_data.get("model", ""))
    run_meta = {
        "timestamp": run_data.get("timestamp"),
        "generation": run_data.get("generation"),
        "postprocess": run_data.get("postprocess"),
        "output_guard": run_data.get("output_guard"),
    }

    existing_run = conn.execute(
        "SELECT id FROM runs WHERE project_id = ? AND run_log_path = ?",
        (project_row["id"], rel_run_log_path),
    ).fetchone()
    if existing_run:
        conn.execute("DELETE FROM quality_reports WHERE run_id = ?", (existing_run["id"],))
        conn.execute("DELETE FROM cost_events WHERE run_id = ?", (existing_run["id"],))
        conn.execute("DELETE FROM runs WHERE id = ?", (existing_run["id"],))

    run_id = uid()
    conn.execute(
        """
        INSERT INTO runs
          (id, project_id, run_log_path, mode, run_mode, stage, time_of_day, weather, model, model_name,
           image_size, image_quality, status, meta_json, settings_snapshot_json, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            run_id,
            project_row["id"],
            rel_run_log_path,
            run_mode,
            run_mode,
            str(run_data.get("stage", "")),
            str(run_data.get("time", "")),
            str(run_data.get("weather", "")),
            model_name,
            model_name,
            str(run_data.get("size", "")),
            str(run_data.get("quality", "")),
            run_status,
            to_json(run_meta),
            to_json(run_meta),
            ts,
        ),
    )

    jobs = run_data.get("jobs", [])
    if not isinstance(jobs, list):
        jobs = []

    inserted_jobs = 0
    inserted_candidates = 0
    inserted_assets = 0
    quality_reports_written = 0
    cost_events_written = 0

    for idx, job in enumerate(jobs, start=1):
        if not isinstance(job, dict):
            continue
        job_key = str(job.get("id") or f"job_{idx}")
        job_id = uid()
        selected_candidate = int(job["selected_candidate"]) if isinstance(job.get("selected_candidate"), int) else None
        final_output_rel = normalize_rel_path(str(job.get("final_output") or "")) or None
        prompt_text = str(job.get("prompt") or job.get("prompt_text") or "")
        conn.execute(
            """
            INSERT INTO run_jobs
              (id, run_id, job_key, status, selected_candidate, selected_candidate_index, final_output, final_asset_id,
               prompt_text, meta_json, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            (
                job_id,
                run_id,
                job_key,
                str(job.get("status", "")),
                selected_candidate,
                selected_candidate,
                final_output_rel,
                None,
                prompt_text,
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
            output_asset_id = None
            final_asset_id = None
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
                output_asset_id = upsert_asset(
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
                final_asset_id = upsert_asset(
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
            elif final_output_path and final_output_path == output_path:
                final_asset_id = output_asset_id

            conn.execute(
                """
                INSERT INTO run_candidates
                  (id, job_id, candidate_index, status, output_asset_id, final_asset_id,
                   rank_hard_failures, rank_soft_warnings, rank_avg_chroma_exceed, meta_json, created_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(id) DO UPDATE SET
                  job_id = excluded.job_id,
                  candidate_index = excluded.candidate_index,
                  status = excluded.status,
                  output_asset_id = excluded.output_asset_id,
                  final_asset_id = excluded.final_asset_id,
                  rank_hard_failures = excluded.rank_hard_failures,
                  rank_soft_warnings = excluded.rank_soft_warnings,
                  rank_avg_chroma_exceed = excluded.rank_avg_chroma_exceed,
                  meta_json = excluded.meta_json
                """,
                (
                    candidate_id,
                    job_id,
                    candidate_index,
                    str(candidate.get("status", "")),
                    output_asset_id,
                    final_asset_id,
                    int(rank.get("hard_failures", 0) or 0),
                    int(rank.get("soft_warnings", 0) or 0),
                    float(rank.get("avg_chroma_exceed", 0.0) or 0.0),
                    to_json(candidate),
                    ts,
                ),
            )

            report_summary = {
                "status": str(candidate.get("status", "")),
                "rank": {
                    "hard_failures": int(rank.get("hard_failures", 0) or 0),
                    "soft_warnings": int(rank.get("soft_warnings", 0) or 0),
                    "avg_chroma_exceed": float(rank.get("avg_chroma_exceed", 0.0) or 0.0),
                },
                "output_path": output_path,
                "final_output_path": final_output_path,
            }
            if isinstance(candidate.get("output_guard"), dict):
                report_summary["output_guard"] = candidate.get("output_guard")
            if isinstance(candidate.get("qa"), dict):
                report_summary["qa"] = candidate.get("qa")
            insert_quality_report(
                conn,
                project_row["id"],
                run_id,
                job_id,
                candidate_id,
                "output_guard",
                report_summary,
                created_at=ts,
            )
            quality_reports_written += 1

        final_output = normalize_rel_path(str(job.get("final_output") or ""))
        if final_output:
            final_asset_id = upsert_asset(
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
            conn.execute(
                """
                UPDATE run_jobs
                SET final_asset_id = ?, final_output = COALESCE(final_output, ?)
                WHERE id = ?
                """,
                (final_asset_id, final_output, job_id),
            )
            inserted_assets += 1

    output_guard = run_data.get("output_guard")
    if isinstance(output_guard, dict):
        insert_quality_report(
            conn,
            project_row["id"],
            run_id,
            None,
            None,
            "output_guard",
            {"scope": "run", "output_guard": output_guard},
            created_at=ts,
        )
        quality_reports_written += 1

    for row in extract_cost_event_rows(run_data):
        insert_cost_event(
            conn,
            project_row["id"],
            run_id,
            row.get("provider_code", "unknown"),
            row.get("operation_code", "legacy_event"),
            row.get("units", 0),
            row.get("cost_usd", 0),
            row.get("currency", "USD"),
            row.get("meta", {}),
            created_at=ts,
        )
        cost_events_written += 1

    emit_audit_event(
        conn,
        project_row["id"],
        None,
        "run.ingested",
        {
            "run_id": run_id,
            "run_log_path": rel_run_log_path,
            "jobs": inserted_jobs,
            "candidates": inserted_candidates,
            "assets_upserted": inserted_assets,
            "quality_reports_written": quality_reports_written,
            "cost_events_written": cost_events_written,
        },
        target_type="run",
        target_id=run_id,
    )

    conn.commit()
    return {
        "run_id": run_id,
        "run_log_path": rel_run_log_path,
        "jobs": inserted_jobs,
        "candidates": inserted_candidates,
        "assets_upserted": inserted_assets,
        "quality_reports_written": quality_reports_written,
        "cost_events_written": cost_events_written,
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
        owner_user_id = (
            project_row["owner_user_id"]
            if row_has_key(project_row, "owner_user_id") and project_row["owner_user_id"]
            else project_row["user_id"]
        )
        owner = conn.execute("SELECT * FROM app_users WHERE id = ?", (owner_user_id,)).fetchone()
        if owner:
            copy_rows(conn, export_conn, "app_users", "id = ?", (owner["id"],))
            copy_rows(conn, export_conn, "users", "id = ?", (owner["id"],))
        copy_rows(conn, export_conn, "projects", "id = ?", (project_row["id"],))

        # Runs and nested data.
        run_rows = conn.execute("SELECT id FROM runs WHERE project_id = ?", (project_row["id"],)).fetchall()
        run_ids = [r["id"] for r in run_rows]
        runs_copied = copy_rows(conn, export_conn, "runs", "project_id = ?", (project_row["id"],))

        jobs_copied = 0
        candidates_copied = 0
        run_candidates_copied = 0
        job_ids: list[str] = []
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
        if job_ids:
            job_placeholders = ", ".join(["?"] * len(job_ids))
            run_candidates_copied = copy_rows(
                conn,
                export_conn,
                "run_candidates",
                f"job_id IN ({job_placeholders})",
                tuple(job_ids),
            )
        snapshots_copied = copy_rows(conn, export_conn, "project_snapshots", "project_id = ?", (project_row["id"],))
        quality_reports_copied = copy_rows(conn, export_conn, "quality_reports", "project_id = ?", (project_row["id"],))
        cost_events_copied = copy_rows(conn, export_conn, "cost_events", "project_id = ?", (project_row["id"],))
        audit_events_copied = copy_rows(conn, export_conn, "audit_events", "project_id = ?", (project_row["id"],))
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
                "owner_user_id": owner_user_id,
            },
            "copied_rows": {
                "runs": runs_copied,
                "jobs": jobs_copied,
                "candidates": candidates_copied,
                "run_candidates": run_candidates_copied,
                "assets": assets_copied,
                "snapshots": snapshots_copied,
                "quality_reports": quality_reports_copied,
                "cost_events": cost_events_copied,
                "audit_events": audit_events_copied,
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
    export_storage_uri = path_for_storage(output_path, repo_root)
    export_asset_id = upsert_asset(
        conn,
        project_row["id"],
        None,
        None,
        None,
        "export",
        export_storage_uri,
        repo_root,
        compute_hashes=True,
        extra_meta={"format": "tar.gz" if output_path.suffix == ".gz" else output_path.suffix.lstrip(".")},
    )
    conn.execute(
        """
        INSERT INTO project_exports
          (id, project_id, export_path, export_asset_id, export_sha256, sha256, created_at, format)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            uid(),
            project_row["id"],
            export_storage_uri,
            export_asset_id,
            export_hash,
            export_hash,
            now_iso(),
            "tar.gz" if output_path.suffix == ".gz" else (output_path.suffix.lstrip(".") or "folder"),
        ),
    )
    emit_audit_event(
        conn,
        project_row["id"],
        None,
        "project.exported",
        {
            "export_path": export_storage_uri,
            "export_asset_id": export_asset_id,
            "sha256": export_hash,
            "include_files": bool(include_files),
        },
        target_type="project_export",
        target_id=export_asset_id,
    )
    conn.commit()
    return {"export_path": str(output_path), "export_sha256": export_hash, "export_asset_id": export_asset_id}


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
            SET secret_ciphertext = ?, key_ref = 'local-master', kms_key_ref = 'local-master', updated_at = ?
            WHERE id = ?
            """,
            (ciphertext, ts, row["id"]),
        )
        emit_audit_event(
            conn,
            project_id,
            None,
            "secret.updated",
            {"provider_code": provider_code, "secret_name": secret_name},
            target_type="project_api_secret",
            target_id=row["id"],
        )
        conn.commit()
        return row["id"]

    secret_id = uid()
    conn.execute(
        """
        INSERT INTO project_api_secrets
        (id, project_id, provider_code, secret_name, secret_ciphertext, key_ref, kms_key_ref, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, 'local-master', 'local-master', ?, ?)
        """,
        (secret_id, project_id, provider_code, secret_name, ciphertext, ts, ts),
    )
    emit_audit_event(
        conn,
        project_id,
        None,
        "secret.created",
        {"provider_code": provider_code, "secret_name": secret_name},
        target_type="project_api_secret",
        target_id=secret_id,
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
        SELECT id, provider_code, secret_name, key_ref, kms_key_ref, created_at, updated_at
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
    payload = project_storage_payload(repo_root, project, conn)
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
                "kms_key_ref": r["kms_key_ref"] or r["key_ref"],
                "key_ref": r["key_ref"] or r["kms_key_ref"],
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
    emit_audit_event(
        conn,
        project["id"],
        None,
        "secret.deleted",
        {"provider_code": provider_code, "secret_name": secret_name, "deleted": cur.rowcount},
        target_type="project_api_secret",
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
    emit_audit_event(
        conn,
        project["id"],
        None,
        "storage.local.updated",
        {"base_dir": local.get("base_dir"), "project_root": local.get("project_root")},
        target_type="project_storage",
        target_id=project["id"],
    )
    conn.commit()
    payload = project_storage_payload(repo_root, refreshed, conn)
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
    emit_audit_event(
        conn,
        project["id"],
        None,
        "storage.s3.updated",
        {
            "enabled": bool(s3.get("enabled")),
            "bucket": s3.get("bucket"),
            "prefix": s3.get("prefix"),
            "region": s3.get("region"),
            "profile": s3.get("profile"),
            "endpoint_url": s3.get("endpoint_url"),
        },
        target_type="project_storage",
        target_id=project["id"],
    )
    conn.commit()
    payload = project_storage_payload(repo_root, refreshed, conn)
    print(to_json({"ok": True, "updated": "s3", **payload}))
    conn.close()


def cmd_sync_project_s3(args):
    repo_root = Path.cwd()
    db_path = (repo_root / args.db).resolve()
    conn = connect_db(db_path)
    init_schema(conn)
    project = require_project(conn, args.project_id, args.project_slug)
    payload = project_storage_payload(repo_root, project, conn)
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
    emit_audit_event(
        conn,
        row["id"],
        user["id"],
        "project.upserted",
        {"slug": row["slug"], "name": row["name"]},
        target_type="project",
        target_id=row["id"],
    )
    conn.commit()
    print(
        to_json(
            {
                "ok": True,
                "project": {
                    "id": row["id"],
                    "slug": row["slug"],
                    "name": row["name"],
                    "owner_user_id": row["owner_user_id"] if row_has_key(row, "owner_user_id") else row["user_id"],
                    "user_id": row["user_id"] if row_has_key(row, "user_id") else row["owner_user_id"],
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
      JOIN app_users u ON u.id = COALESCE(p.owner_user_id, p.user_id)
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
    storage_payload = project_storage_payload(repo_root, project, conn)
    source_files_root = Path(storage_payload["storage"]["local"]["project_root"])

    if args.output:
        output_path = (repo_root / args.output).resolve()
    else:
        stamp = now_iso().replace(":", "-")
        output_path = (repo_root / DEFAULT_EXPORTS_BASE_DIR / f"{project['slug']}_{stamp}.tar.gz").resolve()

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
        help="Include local project files into export package",
    )
    p_export.set_defaults(func=cmd_export_project)

    p_get_storage = sub.add_parser("get-project-storage", help="Get resolved project storage configuration")
    add_project_ref(p_get_storage)
    p_get_storage.set_defaults(func=cmd_get_project_storage)

    p_set_local = sub.add_parser("set-project-storage-local", help="Configure local storage for a project")
    add_project_ref(p_set_local)
    p_set_local.add_argument(
        "--base-dir",
        default="",
        help=f"Base dir for all projects, e.g. {DEFAULT_PROJECTS_BASE_DIR}",
    )
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
