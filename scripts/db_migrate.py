#!/usr/bin/env python3
import argparse
from pathlib import Path

import backend as be


def main():
    parser = argparse.ArgumentParser(description="Apply IAT DB migrations")
    parser.add_argument("--db", default=be.DEFAULT_DB, help=f"SQLite DB path (default: {be.DEFAULT_DB})")
    args = parser.parse_args()

    repo_root = Path.cwd()
    db_path = (repo_root / args.db).resolve()
    conn = be.connect_db(db_path)
    be.init_schema(conn)
    rows = conn.execute("SELECT version, note, applied_at FROM schema_migrations ORDER BY applied_at ASC").fetchall()
    conn.close()
    print(
        be.to_json(
            {
                "ok": True,
                "db": str(db_path),
                "applied_count": len(rows),
                "migrations": [
                    {"version": r["version"], "note": r["note"], "applied_at": r["applied_at"]}
                    for r in rows
                ],
            }
        )
    )


if __name__ == "__main__":
    main()
