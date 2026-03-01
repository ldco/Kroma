# Legacy Scripts (Transitional Only)

`src-tauri` (Rust) is the only supported backend/runtime for Kroma.

This `scripts/` folder exists only as migration scaffolding while remaining responsibilities are moved into Rust modules.

Rules:
- Do not add new production runtime behavior here.
- When a script responsibility is migrated to Rust, delete the script path/subcommand in the same phase.
- Legacy Python backend fallback scripts are opt-in gated and should not run by accident.

Legacy fallback gate:
- Set `KROMA_ENABLE_LEGACY_SCRIPTS=1` only when explicitly validating a migration fallback path.
- Legacy npm script entrypoints are intentionally namespaced as `*:legacy` to avoid accidental use in normal Rust runtime workflows.
- Direct `node scripts/image-lab.mjs ...` execution is blocked unless `KROMA_ENABLE_LEGACY_SCRIPTS=1` is set.

Removed legacy entrypoint:
- `scripts/backend_api.py` has already been removed; the Rust HTTP server is the active backend.
