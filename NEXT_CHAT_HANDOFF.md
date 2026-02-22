# Next Chat Handoff

Date: 2026-02-22
Branch: `master`
Last commit before this handoff update: `e31ebe6`

## Current Status

1. Bootstrap prompt exchange is implemented and tested end-to-end.
2. Replace-mode import safety bug was fixed (no more implicit data wipe of omitted sections).
3. Full backend Rust test suite is green after fixes.

## Scope of Analysis (This Pass)

1. Reviewed all newly added bootstrap backend code:
- `src-tauri/src/db/projects/bootstrap.rs`
- `src-tauri/src/api/bootstrap.rs`
- route and contract wiring in `routes.rs`, `server.rs`, OpenAPI
2. Reviewed integration/contract coverage for bootstrap routes.
3. Audited edge cases around import mode semantics (`merge` vs `replace`) and payload omission behavior.

## Issues Discovered

1. High-risk logic issue in bootstrap import replace-mode:
- `mode=replace` deleted provider accounts, style guides, and prompt templates even when those sections were omitted from payload.
- This could silently wipe unrelated project settings.
2. Missing regression coverage for partial replace payloads.

## Fixes Implemented

1. Replace-mode safety fix in bootstrap store:
- Only replaces sections explicitly present in input payload.
- Omitted sections are now preserved.
2. Input model hardening:
- Switched bootstrap settings lists to `Option<Vec<...>>` so section presence is tracked accurately.
3. Validation refinement:
- Import payload is rejected only when no sections and no project patch are provided.
4. Added regression test:
- `bootstrap_replace_mode_only_replaces_provided_sections` in `src-tauri/tests/bootstrap_endpoints.rs`.
5. Updated UX doc wording to reflect corrected semantics:
- `docs/DESKTOP_UI_PLAYBOOK.md` (`replace` mode description).

## Validation

Commands run:
1. `cargo fmt`
2. `cargo test`

Result: passing.

## Completed Work

1. Completed targeted code review on newly added bootstrap feature.
2. Fixed destructive replace-mode behavior.
3. Added regression tests and revalidated full backend suite.
4. Updated handoff + desktop playbook semantics.

## Remaining Risks / TODO

1. Bootstrap import currently handles:
- project metadata
- provider accounts
- style guides
- prompt templates
It does not yet include characters/reference sets/secrets.
2. No dry-run endpoint yet (cannot preview diffs before apply).
3. Very large project settings can produce long exported prompts; UI should support robust copy/download UX.

## Open Tasks

1. Implement frontend integration for bootstrap export/import controls.
2. Add optional dry-run preview mode for bootstrap import.
3. Extend bootstrap schema support to characters/reference sets after preview flow is in place.

## Recommended Next Steps

1. Add a preview API contract for bootstrap import (`dry_run=true`) returning proposed changes without writes.
2. Wire desktop UI actions:
- Export prompt
- Paste AI response
- merge/replace selector
- explicit confirmation modal for replace
3. Add tests for duplicate/case-collision behavior in imported names/codes.
