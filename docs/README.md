# Docs Folder Policy

This folder contains repository documentation for architecture, specs, audits, and migration plans.

Key docs:
- `DESKTOP_UI_PLAYBOOK.md` - practical desktop UX flow and style-consistency strategy without Codex
- `USER_FLOW_JOURNEY_MAP.md` - canonical user journey, step IDs, and implementation acceptance gates
- `BACKEND_CONTRACT_FREEZE.md` - Step B backend contract baseline (error taxonomy, endpoint surface, breaking-change policy)
- `Kroma_—_Project_Spec_(Current_State_&_Roadmap).md` - source of truth for current architecture, backend state, and roadmap
- `MIGRATION_STATUS.md` - exact Rust vs scripts vs legacy migration status by subsystem

Quick status note:
- Rust backend (`src-tauri`) is the primary API/runtime for metadata and contract-tested endpoints.
- `scripts/` still exists for pipeline orchestration/tooling and compatibility paths that are not yet fully migrated.

What belongs in `docs/`:
- technical specs and architecture decisions
- schema audits and migration plans
- workflow and implementation notes

What does **not** belong in `docs/`:
- per-project runtime knowledge/content that should be stored in the app database
- generated runtime artifacts

Production rule:
- project/user knowledge data lives in DB tables (not in a `knowledge/` filesystem folder)
- repository-level engineering docs live in `docs/`
