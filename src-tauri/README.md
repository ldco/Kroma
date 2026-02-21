# Kroma Rust Backend Core (Scaffold)

This directory is the clean Rust/Tauri backend baseline.

## Architecture Shape

- `src/api`: route catalog and endpoint metadata (contract-first surface)
- `src/contract`: HTTP contract primitives + OpenAPI loader
- `src/db`: DB facade boundary
- `src/storage`: project storage policy boundary
- `src/pipeline`: pipeline stage model
- `src/worker`: instruction lifecycle model

## Contract Parity Gate

`tests/contract_parity.rs` enforces:

1. every route declared in `openapi/backend-api.openapi.yaml` exists in the Rust route catalog;
2. the Rust route catalog has no duplicate method/path entries.

`tests/http_contract_surface.rs` enforces:

1. every contract route is mounted in the Axum router;
2. mounted status behavior is deterministic (`200` for `GET /health`, `501` stub for unimplemented endpoints).

This keeps OpenAPI and Rust command/API surface synchronized from day one.

## Implemented Endpoints

The following routes are fully wired (SQLite-backed):

- `GET /api/projects`
- `POST /api/projects`
- `GET /api/projects/{slug}`

All other contract routes are mounted and return deterministic `501` stub responses.

DB path defaults to `var/backend/app.db` (repo-root relative). Override with:

- `KROMA_BACKEND_DB=/abs/or/relative/path/to/app.db`

## Run Locally

```bash
cd src-tauri
cargo test
```

Run stub server:

```bash
cd src-tauri
KROMA_BACKEND_BIND=127.0.0.1:8788 cargo run
```

If Rust is not installed yet:

```bash
curl https://sh.rustup.rs -sSf | sh
```
