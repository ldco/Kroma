# Docs Folder Policy

This folder contains repository documentation for architecture, specs, audits, and migration plans.

Key docs:
- `DESKTOP_UI_PLAYBOOK.md` - practical desktop UX flow and style-consistency strategy without Codex

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
