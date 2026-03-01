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

Removed legacy entrypoints:
- `scripts/backend_api.py` - removed; Rust HTTP server is the active backend.
- `scripts/image-lab.mjs` - removed; Rust CLI commands (`cargo run -- generate-one|upscale|color|bgremove|qa|archive-bad`) are the active runtime.

Rust CLI utility commands (replacement for image-lab.mjs):
```bash
cargo run -- generate-one --project-slug <slug> --prompt <text> --input-images-file <file> --output <path>
cargo run -- upscale --project-slug <slug> [--input PATH] [--output PATH] [--upscale-backend ncnn|python]
cargo run -- color --project-slug <slug> [--input PATH] [--output PATH] [--profile PROFILE]
cargo run -- bgremove --project-slug <slug> [--input PATH] [--output PATH]
cargo run -- qa --project-slug <slug> [--input PATH]
cargo run -- archive-bad --project-slug <slug> --input PATH
```
