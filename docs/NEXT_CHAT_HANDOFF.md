# Next Chat Handoff

Date: 2026-02-28
Branch: `master`
HEAD (pre-handoff commit) / upstream (`origin/master`): `bc91256` / `35729af`
Worktree: dirty (6 local/uncommitted files: `package.json`, `scripts/image-lab.mjs`, `scripts/README.md`, `docs/WORKFLOW.md`, `docs/ROADMAP.md`, `docs/NEXT_CHAT_HANDOFF.md`)

## Session Update (2026-02-28, Step A legacy image-lab hard gate enforcement)

### Scope

1. Finalize the in-progress Step A legacy command quarantine slice.
2. Enforce explicit legacy gating for direct `scripts/image-lab.mjs` execution.
3. Refresh handoff state with current repo evidence for next-chat continuity.

### What Landed (this session, local/uncommitted)

1. Enforced direct-execution legacy gate in `scripts/image-lab.mjs`:
   - added `requireLegacyGateEnabled()` and wired it in `main()` for all non-help modes.
   - behavior change: direct non-help execution now fails unless `KROMA_ENABLE_LEGACY_SCRIPTS=1`.
2. Added explicit legacy runtime notice in `scripts/image-lab.mjs`:
   - logs transitional status and gate-state (`0|1`) on gated execution paths.
3. Completed command quarantine docs alignment:
   - `package.json`: legacy image-lab utilities are `*:legacy` and gated.
   - `docs/WORKFLOW.md`: all script command examples updated to `*:legacy`.
   - `scripts/README.md`: direct invocation gate requirement documented.
   - `docs/ROADMAP.md`: Step A script gating note updated.
4. Updated continuity artifact:
   - `docs/NEXT_CHAT_HANDOFF.md` entry stack expanded with this session and current repo state.

### Validation

1. `node scripts/image-lab.mjs --help` -> passing (usage shows `lab:legacy` commands).
2. `node scripts/image-lab.mjs dry` -> blocked with expected gate error:
   - `Legacy script execution requires KROMA_ENABLE_LEGACY_SCRIPTS=1 ...`
3. `KROMA_ENABLE_LEGACY_SCRIPTS=1 node scripts/image-lab.mjs dry` -> passing expected gate path:
   - prints legacy notice, then expected `--project` validation error.
4. `rg -n "npm run (lab|upscale|color|bgremove|qa|archivebad) --" docs/WORKFLOW.md scripts/image-lab.mjs package.json scripts/README.md` -> no matches (stale non-legacy command names removed in active docs/files).

### Known Gaps / Risks

1. `scripts/image-lab.mjs` still exists as transitional fallback utility code; it is now gated but not removed.
2. Help/usage mode is intentionally not blocked by the legacy gate (execution-only block); keep or tighten this behavior in next slice.
3. Local changes are not committed yet for this slice.

### Next Chat Starting Point

1. Commit current Step A legacy-gate slice (`package.json`, `scripts/image-lab.mjs`, `scripts/README.md`, `docs/WORKFLOW.md`, `docs/ROADMAP.md`, `docs/NEXT_CHAT_HANDOFF.md`).
2. Decide Step A direction for `scripts/image-lab.mjs`: full removal vs retained gated fallback.
3. If retention is chosen, add one focused regression test/documented operator flow that validates `*:legacy` command behavior under both gate states.

## Session Update (2026-02-27, Step A legacy image-lab command quarantine)

### Scope

1. Continue Step A by reducing accidental script-path usage while preserving explicit fallback utilities.
2. Make legacy script invocation visibly opt-in at npm command level.

### What Landed (this session, local/uncommitted)

1. Renamed image-lab npm commands to explicit legacy names in `package.json`:
   - `lab` -> `lab:legacy`
   - `upscale` -> `upscale:legacy`
   - `color` -> `color:legacy`
   - `bgremove` -> `bgremove:legacy`
   - `qa` -> `qa:legacy`
   - `archivebad` -> `archivebad:legacy`
   - each now explicitly sets `KROMA_ENABLE_LEGACY_SCRIPTS=1`.
2. Updated workflow command examples in `docs/WORKFLOW.md` to use `*:legacy` command names.
3. Updated legacy script policy notes:
   - `scripts/README.md`
   - `docs/ROADMAP.md`
4. Added startup legacy-only runtime notice in `scripts/image-lab.mjs`:
   - logs explicit transitional context on non-help execution
   - includes gate state (`KROMA_ENABLE_LEGACY_SCRIPTS=0|1`)
   - direct non-help execution is now blocked unless the legacy gate is enabled.

### Validation

1. `node scripts/image-lab.mjs --help` -> passing (usage banner aligned to `lab:legacy` naming).
2. command-reference consistency check passed for active files (`docs/WORKFLOW.md`, `scripts/image-lab.mjs`, `package.json`, `scripts/README.md`).
3. `node scripts/image-lab.mjs dry` -> blocked with explicit legacy-gate error.
4. `KROMA_ENABLE_LEGACY_SCRIPTS=1 node scripts/image-lab.mjs dry` -> prints gated legacy notice + expected project-validation error.

### Known Gaps / Risks

1. This change intentionally preserves `scripts/image-lab.mjs`; it only reduces accidental invocation via npm naming and explicit env gate.
2. Full removal of `scripts/image-lab.mjs` remains a future Step A scope decision once utility parity is no longer needed.

### Next Chat Starting Point

1. Decide whether to fully remove `scripts/image-lab.mjs` and its legacy npm aliases or retain them for explicit operator fallback.
2. If retained, decide whether the hard gate should also apply to help/usage mode or remain execution-only (current behavior).

## Session Update (2026-02-27, Step A runtime trigger path de-scripting semantics)

### Scope

1. Start Step A follow-through by removing remaining script-fallback semantics from the normal Rust trigger/runtime path.
2. Keep behavior stable while aligning API contract labels and docs with current Rust-native execution reality.

### What Landed (this session, local/uncommitted)

1. Updated post-run wrapper request behavior:
   - `src-tauri/src/pipeline/runtime_orchestrators.rs`
   - `RustPostRunPipelineOrchestrator` now forwards the original request directly to inner runtime execution (no script-era request mutation).
2. Updated trigger adapter label to Rust-native:
   - `src-tauri/src/api/runs_assets.rs`: response `pipeline_trigger.adapter` now returns `rust_native`.
3. Updated test assertions and wrapper expectations:
   - `src-tauri/tests/pipeline_trigger_endpoints.rs`
   - `src-tauri/src/pipeline/runtime.rs` wrapper tests now assert request options are preserved rather than force-overridden.
4. Updated contract/docs wording:
   - `openapi/backend-api.openapi.yaml`: trigger adapter example updated to `rust_native`.
   - `docs/ROADMAP.md`: removed stale script-fallback wording for trigger/runtime stack and post-run wrapper semantics.

### Validation

1. `cargo test rust_post_run_wrapper_ --lib` -> passing.
2. `cargo test --test pipeline_trigger_endpoints --test contract_parity --test http_contract_surface` -> passing.

### Known Gaps / Risks

1. `scripts/image-lab.mjs` still exists for manual utility scripts, but normal `/runs/trigger` path semantics are now explicitly Rust-native.
2. Full script file removal/deprecation sequencing is still a separate cleanup decision.

### Next Chat Starting Point

1. Continue Step A by deciding whether to retire `scripts/image-lab.mjs` utility npm commands or keep them as explicit non-runtime tools.
2. Keep extending Rust-owned execution evidence with endpoint-level tests that exercise real tool adapter paths.

## Session Update (2026-02-27, Step B remaining endpoint taxonomy/openapi normalization)

### Scope

1. Continue Step B by closing taxonomy assertion coverage for the remaining endpoint groups called out in the previous handoff.
2. Align OpenAPI error-schema refs for those same endpoint groups so docs and runtime behavior stay contract-consistent.

### What Landed (this session, local/uncommitted)

1. Expanded endpoint taxonomy assertions in integration tests:
   - `src-tauri/tests/provider_accounts_endpoints.rs`
   - `src-tauri/tests/style_guides_endpoints.rs`
   - `src-tauri/tests/prompt_templates_endpoints.rs`
   - `src-tauri/tests/characters_endpoints.rs`
   - `src-tauri/tests/asset_links_endpoints.rs`
   - `src-tauri/tests/chat_endpoints.rs`
   - `src-tauri/tests/agent_instructions_endpoints.rs`
   - `src-tauri/tests/secrets_endpoints.rs`
2. Updated OpenAPI error-schema refs in `openapi/backend-api.openapi.yaml` for:
   - `asset-links` list/create/detail/update/delete
   - `prompt-templates` list/create/detail/update/delete
   - `provider-accounts` list/create/detail/update/delete
   - `style-guides` list/create/detail/update/delete
   - `characters` list/create/detail/update/delete
   - `chat/sessions` list/create/detail/messages list/create
   - `agent/instructions` list/create/detail/events/confirm/cancel
   - `secrets` list/upsert/rotation-status/rotate/delete
3. Synced Step B docs:
   - `docs/ROADMAP.md` Step B progress expanded with remaining endpoint coverage.
   - `docs/BACKEND_CONTRACT_FREEZE.md` verification baseline expanded.

### Validation

1. `cargo test --test provider_accounts_endpoints --test style_guides_endpoints --test prompt_templates_endpoints --test characters_endpoints --test asset_links_endpoints --test chat_endpoints --test agent_instructions_endpoints --test secrets_endpoints --test error_taxonomy_endpoints --test contract_parity --test http_contract_surface` -> passing.

### Known Gaps / Risks

1. Step B coverage is materially broader now, but there is still no explicit single “freeze green” checklist artifact marking Gate 1 complete.
2. Step A runtime/script-removal work remains open in parallel (`image-lab.mjs` ownership completion and runtime decomposition follow-through).

### Next Chat Starting Point

1. Decide whether to mark Step B Gate 1 complete now or add one more pass for non-journey/secondary endpoints before freezing.
2. Resume Step A runtime consolidation slices from the latest runtime module decomposition state.

## Session Update (2026-02-27, Step B taxonomy expansion + OpenAPI alignment)

### Scope

1. Continue Step B by expanding taxonomy assertions beyond initial project/run/export coverage.
2. Align OpenAPI error schemas for additional journey-critical endpoint groups.

### What Landed (this session, local/uncommitted)

1. Expanded endpoint regression assertions for taxonomy fields:
   - `src-tauri/tests/bootstrap_endpoints.rs`
   - `src-tauri/tests/reference_sets_endpoints.rs`
   - `src-tauri/tests/storage_endpoints.rs`
2. Added bootstrap missing-project taxonomy coverage:
   - `bootstrap_prompt_missing_project_has_not_found_taxonomy`
3. Updated OpenAPI contract (`openapi/backend-api.openapi.yaml`):
   - added `components.schemas.ErrorKind`
   - added `components.schemas.ErrorResponse`
   - wired error schema refs for:
     - `projects` create/detail errors
     - `bootstrap-prompt` and `bootstrap-import` errors
     - `storage` read/update errors
     - `reference-sets` and `reference-set` detail/update/delete errors
4. Synced Step B docs:
   - `docs/BACKEND_CONTRACT_FREEZE.md` verification baseline expanded
   - `docs/ROADMAP.md` Step B kickoff progress expanded

### Validation

1. `cargo test --test bootstrap_endpoints --test reference_sets_endpoints --test storage_endpoints --test error_taxonomy_endpoints` -> passing.
2. `cargo test --test contract_parity --test http_contract_surface` -> passing.

### Known Gaps / Risks

1. OpenAPI coverage for taxonomy is expanded for Step B baseline paths, but not yet fully normalized across every endpoint family.
2. Runtime Step A completion items remain open in parallel (`image-lab.mjs` orchestration removal).

### Next Chat Starting Point

1. Finish Step B taxonomy/OpenAPI normalization for remaining endpoint groups (`provider_accounts`, `style_guides`, `prompt_templates`, `characters`, `asset_links`, `chat`, `agent_instructions`, `secrets`).
2. After Step B Gate 1 is fully green, continue Step A runtime/script-removal completion slices.

## Session Update (2026-02-27, Step B contract-freeze kickoff)

### Scope

1. Continue roadmap execution into Step B (backend contract freeze).
2. Publish stable error taxonomy for journey-critical backend endpoints.
3. Add endpoint-level regression tests to lock taxonomy behavior.

### What Landed (this session, local/uncommitted)

1. Added additive error taxonomy fields to API error responses:
   - `error_kind`
   - `error_code`
2. Updated shared API error mapper in `src-tauri/src/api/handler_utils.rs`:
   - repo validation/not-found/internal paths now emit taxonomy fields.
3. Updated run/pipeline error mapping in `src-tauri/src/api/runs_assets.rs`:
   - policy/validation/provider/infra classification for trigger and config-validation failures.
4. Added new integration tests:
   - `src-tauri/tests/error_taxonomy_endpoints.rs`
   - verifies taxonomy on project validation errors, not-found errors, and run spend-policy errors.
5. Published Step B baseline contract doc:
   - `docs/BACKEND_CONTRACT_FREEZE.md`
   - includes journey-critical endpoint surface, error taxonomy baseline, and breaking-change policy.
6. Synced docs:
   - `docs/ROADMAP.md` (Step B kickoff progress section)
   - `docs/BACKEND_ARCHITECTURE_FREEZE.md` (Gate 1 contract-baseline reference)
   - `docs/README.md` (new doc indexed)

### Validation

1. `cargo fmt` (in `src-tauri`) -> passing.
2. `cargo test --test error_taxonomy_endpoints` -> passing.
3. `cargo test --test projects_endpoints --test runs_assets_endpoints --test exports_endpoints --test pipeline_trigger_endpoints` -> passing.
4. `cargo test --test http_contract_surface --test contract_parity` -> passing.

### Known Gaps / Risks

1. This is a Step B kickoff baseline; not every endpoint has explicit code-level taxonomy mapping yet (priority paths are covered).
2. Existing response shape remains backward-compatible (`ok/error` preserved), so frontend should begin consuming `error_kind/error_code` as preferred fields.

### Next Chat Starting Point

1. Expand taxonomy coverage and assertions to remaining journey-related endpoints (`bootstrap`, `reference_sets`, `storage`) to complete Step B Gate 1.
2. Add/align OpenAPI error schemas/examples with `error_kind/error_code` baseline to reduce frontend ambiguity.
3. Continue Step B acceptance checklist until backend freeze is marked green.

## Session Update (2026-02-27, worker hardening follow-up)

### Scope

1. Continue the Rust worker migration slice with reliability hardening.
2. Close status-mapping edge cases and secret-token dispatch coverage gaps.
3. Keep worker tests deterministic across environments.

### What Landed (this session, local/uncommitted)

1. Fixed worker remote-status normalization bug in `src-tauri/src/worker/mod.rs`:
   - `map_remote_status(...)` now uses normalized/trimmed status matching directly.
   - prevents `" failed "` / `" Running "` from being mis-mapped to `"done"`.
2. Expanded worker status unit coverage:
   - `map_remote_status_accepts_known_values` now includes mixed-case + whitespace variants.
3. Added dispatch auth fallback regression test in `src-tauri/src/db/projects.rs`:
   - new test `worker_uses_project_secret_token_when_cli_token_missing`.
   - spins up a local listener, runs `run_agent_worker_loop(...)`, and asserts `Authorization: Bearer <secret>` comes from project secret fallback.
4. Hardened existing URL fallback test for determinism:
   - `worker_uses_project_secret_dispatch_target_when_cli_target_missing` now uses an invalid URL (`not-a-url`) instead of a potentially open local port.
   - assertions now track retry/error event semantics (`retry_scheduled` or `error`) and avoid retry-policy-coupled status assumptions.

### Validation

1. `cargo fmt` (in `src-tauri`) -> passing.
2. `cargo test worker_uses_project_secret_` -> passing.
3. `cargo test map_remote_status_accepts_known_values` -> passing.
4. `cargo test parse_agent_worker_accepts_once_and_target_url` -> passing.

### Known Gaps / Risks

1. This slice is worker-hardening only; broader Step A/Step B roadmap items remain unchanged.
2. Full suite was not rerun after this patch (focused worker/CLI targets were rerun and passing).

### Next Chat Starting Point

1. Continue roadmap Step A/B progression from `docs/ROADMAP.md`:
   - either close remaining runtime/script-removal governance updates
   - or start Step B contract freeze deliverables (error taxonomy + endpoint contract publication) with tests.

## Session Update (2026-02-27, Rust worker runtime migration slice)

### Scope

1. Continue Step A by starting the Python worker runtime replacement in Rust.
2. Add Rust-owned queue reserve/dispatch/complete worker flow on top of current `agent_instructions`.
3. Switch default worker npm entrypoints to the Rust worker CLI while preserving explicit legacy fallback commands.

### What Landed (this session, local/uncommitted)

1. Added Rust worker runtime loop + dispatch client:
   - expanded `src-tauri/src/worker/mod.rs` with:
     - `AgentWorkerOptions`
     - `run_agent_worker_loop(...)`
     - HTTP dispatch + retry/backoff handling
     - remote-status mapping parity (`accepted|queued -> done`)
2. Added worker-oriented DB lease/complete methods in `src-tauri/src/db/projects/chat_instructions.rs`:
   - `reserve_next_agent_instruction(...)`
   - `complete_agent_instruction_success(...)`
   - `complete_agent_instruction_retry_or_fail(...)`
3. Added/normalized worker support columns on `agent_instructions`:
   - `attempts`, `max_attempts`, `next_attempt_at`, `last_error`, `locked_by`, `locked_at`, `agent_response_json`
4. Improved instruction action transitions:
   - confirm now clears lock/retry/error fields.
   - cancel now clears lock fields.
5. Added CLI command for worker runtime in `src-tauri/src/main.rs`:
   - `cargo run -- agent-worker [flags]`
   - added parser + usage + unit coverage.
6. Switched default npm worker commands to Rust, preserved explicit legacy aliases in `package.json`:
   - `backend:worker` -> Rust `agent-worker`
   - `backend:worker:once` -> Rust `agent-worker --once`
   - added `backend:worker:legacy` and `backend:worker:once:legacy`
7. Added regression tests:
   - DB worker reserve/success path
   - DB worker retry/fail path
   - worker status-mapping unit
   - CLI parser unit for `agent-worker`

### Validation

1. `cargo fmt` (in `src-tauri`) -> passing.
2. `cargo test parse_agent_worker_accepts_once_and_target_url` -> passing.
3. `cargo test agent_worker_reserve_and_complete_success` -> passing.
4. `cargo test agent_worker_retry_then_fail_updates_status` -> passing.
5. `cargo test` (in `src-tauri`) -> passing (full suite, `0` failed).

### Known Gaps / Risks

1. Rust worker dispatch currently uses CLI/env target URL/token (`IAT_AGENT_API_URL`, `IAT_AGENT_API_TOKEN`) and does not yet resolve per-project encrypted secret fallback automatically.
2. Full replacement/removal of `scripts/agent_worker.py` and `scripts/agent_dispatch.py` is not complete yet; they remain explicit legacy fallback paths.
3. Step A still has an open major item: replacing remaining `scripts/image-lab.mjs` generation/post-process orchestration ownership.

### Next Chat Starting Point

1. Extend Rust worker target resolution to support project-scoped secret fallback (`agent_api/url`, `agent_api/token`) with encrypted secret reads.
2. After parity is confirmed, remove legacy Python worker scripts/paths per script-removal rule.
3. Continue Step A functional migration on the remaining `image-lab.mjs` orchestration ownership.

## Session Update (2026-02-27, runtime stack/wrapper extraction continuation)

### Scope

1. Continue Step A runtime decomposition from the latest handoff.
2. Extract remaining orchestration construction/wrapper logic from `runtime.rs` into dedicated modules.
3. Keep runtime API behavior stable and close validation gaps with focused + full regression runs.

### What Landed (this session, local/uncommitted)

1. Extracted default runtime stack-construction wiring:
   - added `src-tauri/src/pipeline/runtime_stack.rs`.
   - moved default orchestrator composition (`default_pipeline_orchestrator_with_*`) and rust-only unsupported fallback orchestrator into the new module.
2. Extracted wrapper orchestrators from `runtime.rs`:
   - added `src-tauri/src/pipeline/runtime_orchestrators.rs`.
   - moved `RustPostRunPipelineOrchestrator`, `RustDryRunPipelineOrchestrator`, and `RustRunModePipelineOrchestrator` plus their `PipelineOrchestrator` impls.
3. Kept `pipeline::runtime` API stable:
   - `runtime.rs` now re-exports moved wrappers and default stack constructors.
   - `validate_project_slug` is now `pub(crate)` for shared runtime module use.
4. Updated pipeline module exports:
   - `src-tauri/src/pipeline/mod.rs` now exports `runtime_stack` and `runtime_orchestrators`.
5. Behavioral impact:
   - no intended runtime contract or behavior change; this slice is structural decomposition only.

### Validation

1. `cargo fmt` (in `src-tauri`) -> passing.
2. `cargo test rust_post_run_wrapper_` -> passing.
3. `cargo test rust_dry_run_wrapper_` -> passing.
4. `cargo test rust_run_mode_wrapper_` -> passing.
5. `cargo test` (in `src-tauri`) -> passing (full suite, `0` failed).

### Known Gaps / Risks

1. `runtime.rs` is still test-heavy; test-module decomposition is still open if we want smaller runtime files.
2. Step A functional migration is not complete yet:
   - remaining `scripts/image-lab.mjs` orchestration ownership removal is still pending.
   - Python worker runtime migration (`agent_worker.py`, `agent_dispatch.py`) is still pending.
3. Worktree remains intentionally mixed (docs freeze + runtime refactor slices), so commit slicing remains an open decision.

### Next Chat Starting Point

1. Continue Step A functional migration by replacing remaining `image-lab.mjs` generation/post-process orchestration ownership with Rust modules.
2. Keep script-removal parity per slice, then rerun focused pipeline wrapper tests + full `cargo test`.
3. Finalize the mixed worktree commit strategy (split docs vs runtime slices, or curate one intentional combined commit) before push.

## Session Update (2026-02-27, journey-map freeze + fixed roadmap execution order)

### Scope

1. Align planning docs with the product north star: project-first comic/graphic-novel continuity.
2. Convert roadmap into a fixed execution plan with explicit frontend start gate.
3. Establish canonical user flow/journey map as implementation contract.
4. Add workflow-level rule that every feature maps to a journey step ID.

### What Landed (this session, local/uncommitted)

1. Added canonical journey contract:
   - new file `docs/USER_FLOW_JOURNEY_MAP.md`.
   - defines primary flow `J00-J08`, utility flow `U01`, and recovery flows `R01-R02`.
   - adds mandatory implementation gate: no feature without journey-step mapping.
2. Reworked roadmap into fixed sequence:
   - updated `docs/ROADMAP.md` with `Planning Control Docs`, `Journey-First Execution Rule`, and `Fixed Execution Plan (Step A/B/C)`.
   - added locked immediate next slices and explicit frontend-start condition (only after Step A + Step B).
3. Enforced journey traceability in workflow/freeze docs:
   - `docs/WORKFLOW.md` now requires `Jxx/Uxx/Rxx` mapping plus acceptance evidence in PR/commit notes.
   - `docs/BACKEND_ARCHITECTURE_FREEZE.md` Gate 5 now requires frozen journey steps in `docs/USER_FLOW_JOURNEY_MAP.md`.
4. Synced spec/index/root docs to the same planning model:
   - `docs/README.md` now includes `USER_FLOW_JOURNEY_MAP.md`.
   - `README.md` now includes `Product Goal (North Star)` and `Planning Source of Truth`.
   - `docs/TECH_SPEC.md` now includes `4.3 Journey-Driven Scope Gate`.
   - `docs/Kroma_—_Project_Spec_(Current_State_&_Roadmap).md` now states explicit journey-planning contract.
5. Behavioral/security impact:
   - no runtime code-path changes in this session.
   - planning governance tightened: implementation scope and frontend sequencing are now explicit and testable against journey steps.

### Validation

1. `git branch --show-current` -> pass (`master`).
2. `git rev-parse --short HEAD` -> pass (`35729af`).
3. `git rev-parse --short origin/$(git branch --show-current)` -> pass (`35729af`).
4. `git status --short` -> pass (dirty; includes docs planning updates and existing runtime refactor files).
5. `git diff --stat` -> pass (`13 files changed, 392 insertions(+), 250 deletions(-)` at check time).
6. `git diff -- docs/ROADMAP.md docs/WORKFLOW.md docs/USER_FLOW_JOURNEY_MAP.md docs/README.md docs/BACKEND_ARCHITECTURE_FREEZE.md docs/TECH_SPEC.md README.md "docs/Kroma_—_Project_Spec_(Current_State_&_Roadmap).md"` -> pass (verified fixed plan + journey gate diffs).
7. `python3 /home/ldco/.codex/skills/.system/skill-creator/scripts/quick_validate.py /home/ldco/.codex/skills/general-handoff` -> pass (`Skill is valid!`).
8. Automated tests -> Not run in this session (docs/planning-only changes).

### Known Gaps / Risks

1. Runtime refactor files from prior slices remain uncommitted and mixed with docs changes (`src-tauri/src/pipeline/*`).
2. Worktree includes `src-tauri/src/pipeline/post_run_execution.rs` (untracked) plus docs updates; commit slicing decision is still open.
3. Full Rust test suite was not rerun in this docs-only session; next runtime code step should revalidate with focused tests + `cargo test`.

### Next Chat Starting Point

1. Execute roadmap Step A / Slice 1: continue Rust parity for `J04-J07` by extracting remaining generation/post-process orchestration from scripts into Rust modules.
2. After code changes, run focused pipeline wrapper tests first, then run `cargo test` in `src-tauri`.
3. Finalize commit strategy: either split docs-governance updates into a dedicated commit first, then runtime refactor commit(s), or curate one combined commit intentionally.

## Session Update (2026-02-27, runtime post-run extraction + shared run-log helper consolidation)

### Scope

1. Continue runtime decomposition from the latest handoff next steps.
2. Extract post-run wrapper/finalization orchestration out of `runtime.rs`.
3. Remove duplicated run-log timestamp/stamp helper logic from dry/run execution modules.
4. Revalidate focused runtime wrappers and then run full suite for regression safety.

### What Landed (this session, local/uncommitted)

1. Extracted post-run orchestration into a dedicated module:
   - added `src-tauri/src/pipeline/post_run_execution.rs`.
   - moved summary parse, project-slug mismatch warning, run-log normalization, planned-metadata enrichment, and backend finalize (ingest + optional S3 sync) flow into that module.
2. Rewired `src-tauri/src/pipeline/runtime.rs` to delegate post-run finalize behavior:
   - `RustPostRunPipelineOrchestrator::execute` now calls `run_post_run_finalize_best_effort(...)`.
   - removed moved post-run helper methods/functions from `runtime.rs`.
3. Consolidated shared run-log time/stamp helpers:
   - added `iso_like_timestamp_now()` and `run_log_stamp_now()` in `src-tauri/src/pipeline/runlog.rs`.
   - updated `src-tauri/src/pipeline/dry_run_execution.rs` and `src-tauri/src/pipeline/run_mode_execution.rs` to use shared helpers.
   - removed duplicated local helper implementations from both execution modules.
4. Updated module exports:
   - `src-tauri/src/pipeline/mod.rs` now exports `post_run_execution`.
5. Behavioral impact:
   - no intended API/contract change; refactor keeps existing wrapper behavior while reducing duplication and concentrating post-run orchestration logic.

### Validation

1. `cargo fmt` (in `src-tauri`) -> passing.
2. `cargo test rust_post_run_wrapper_` -> passing.
3. `cargo test rust_dry_run_wrapper_` -> passing.
4. `cargo test rust_run_mode_wrapper_` -> passing.
5. `cargo test iso_like_timestamp_now_matches_expected_shape` -> passing.
6. `cargo test run_log_stamp_now_matches_expected_shape` -> passing.
7. `cargo test` (in `src-tauri`) -> passing (full suite, `0` failed).

### Known Gaps / Risks

1. `runtime.rs` remains large and still contains stack-construction/orchestration glue that can be decomposed further.
2. Current slice is local/uncommitted (along with this handoff update).

### Next Chat Starting Point

1. Continue runtime decomposition by extracting orchestrator stack-construction/wiring from `src-tauri/src/pipeline/runtime.rs` into a dedicated builder module while preserving behavior.
2. Run focused runtime wrapper tests after extraction, then rerun `cargo test`.
3. Finalize/push the accumulated runtime decomposition slice via `[$general-git](/home/ldco/.codex/skills/general-git/SKILL.md)` once ready.

## Session Update (2026-02-27, pipeline runtime modular decomposition continuation)

### Scope

1. Continue iterative runtime decomposition without behavior drift.
2. Reduce `src-tauri/src/pipeline/runtime.rs` responsibilities by extracting cohesive modules.
3. Keep focused regression coverage green at each extraction slice.
4. Commit and push each validated slice to `master`.

### What Landed (this session, local/uncommitted)

1. Landed and pushed 7 refactor commits on `master` (all currently synced to `origin/master`):
   - `bc16e86` planned run-log template shaping extraction.
   - `ed455b3` planned run-log enrichment shaping centralization.
   - `189b50e` run-log summary parsing module extraction.
   - `5df98f7` planning preflight module extraction.
   - `daee15b` request settings resolution module extraction.
   - `938c415` run-mode execution module extraction.
   - `35729af` dry-run execution module extraction.
2. Added new runtime-focused modules and rewired orchestration call sites:
   - `src-tauri/src/pipeline/runlog_enrich.rs`
   - `src-tauri/src/pipeline/runlog_parse.rs`
   - `src-tauri/src/pipeline/planning_preflight.rs`
   - `src-tauri/src/pipeline/request_settings.rs`
   - `src-tauri/src/pipeline/run_mode_execution.rs`
   - `src-tauri/src/pipeline/dry_run_execution.rs`
3. Updated module exports in `src-tauri/src/pipeline/mod.rs` and trimmed `runtime.rs` orchestration to delegate into dedicated modules.
4. Net file impact from this continuation range (`7b759d3..35729af`):
   - `8 files changed, 1365 insertions(+), 981 deletions(-)`.
   - touched files: `src-tauri/src/pipeline/{mod.rs,runtime.rs,runlog_enrich.rs,runlog_parse.rs,planning_preflight.rs,request_settings.rs,run_mode_execution.rs,dry_run_execution.rs}`.
5. Behavioral impact:
   - no intended contract/API changes; refactor focused on separation of concerns.
   - preserved dry-run/run-mode run-log shaping and post-run compatibility behavior through existing wrapper paths.
6. Local/uncommitted status at handoff write time:
   - none; all landed changes are committed and pushed (`worktree clean`).

### Validation

1. `cargo fmt` (in `src-tauri`) -> passing (run repeatedly across slices).
2. `cargo test build_planned_template_shapes_jobs_and_context` -> passing.
3. `cargo test rust_post_run_wrapper_normalizes_run_log_job_finalization_before_ingest` -> passing.
4. `cargo test rust_dry_run_wrapper_writes_planned_job_fields_from_typed_builder` -> passing.
5. `cargo test build_planned_template_from_request_resolves_defaults_and_guard` -> passing.
6. `cargo test parse_script_run_summary_from_stdout_prefers_marker_payload` -> passing.
7. `cargo test parses_script_run_summary_from_stdout_lines` -> passing.
8. `cargo test parses_script_run_summary_from_marker_line` -> passing.
9. `cargo test rust_post_run_wrapper_warns_when_run_log_line_is_missing` -> passing.
10. `cargo test rust_dry_run_wrapper_handles_scene_refs_without_inner_script_call` -> passing.
11. `cargo test rust_dry_run_wrapper_handles_input_path_without_inner_script_call` -> passing.
12. `cargo test rust_dry_run_wrapper_handles_jobs_file_without_inner_script_call` -> passing.
13. `cargo test rust_dry_run_wrapper_uses_manifest_generation_defaults_when_candidates_omitted` -> passing.
14. `cargo test apply_settings_overlay_to_request_fills_missing_postprocess_fields` -> passing.
15. `cargo test apply_settings_overlay_to_request_keeps_explicit_request_values` -> passing.
16. `cargo test rust_dry_run_wrapper_applies_project_settings_layer_for_postprocess_defaults` -> passing.
17. `cargo test rust_run_mode_wrapper_executes_optional_passes_with_script_parity_paths` -> passing.
18. `cargo test rust_run_mode_wrapper_returns_failure_and_archives_bad_outputs_on_output_guard_fail` -> passing.
19. `cargo test rust_post_run_wrapper_ingests_rust_run_mode_failure_with_run_log` -> passing.
20. Failed-attempt evidence:
   - compile/test runs temporarily failed during extraction with `E0425`/`E0609` (missing moved symbols / incorrect test field path); fixed in-session and rerun to green.

### Known Gaps / Risks

1. `runtime.rs` still contains post-run wrapper flow and orchestration glue that can be further decomposed.
2. Timestamp/run-log stamp helpers currently live in both run-mode and dry-run execution modules; potential duplication cleanup remains.
3. Broad full-suite execution (`cargo test` without filter) was not rerun after the final extraction; validation relied on focused regression targets.

### Next Chat Starting Point

1. Extract post-run wrapper/finalization orchestration from `src-tauri/src/pipeline/runtime.rs` into a dedicated module while preserving summary parsing + ingestion/sync behavior.
2. Consolidate shared runtime helpers (for example run-log timestamp/stamp generation) into a small shared module used by `run_mode_execution` and `dry_run_execution`.
3. After the next extraction slice, run focused post-run + dry/run wrapper tests first, then run a broader `cargo test` pass to close residual regression risk.

## Session Update (2026-02-25, backend_ops decoupling continuation)

### Scope

1. Continue Phase 1 runtime consolidation after secrets API/CLI hardening.
2. Remove residual structural coupling between native ingest/S3 sync runtime path and script-wrapper backend ops state.
3. Preserve behavior and keep full test suite green.

### What Landed (this session, local/uncommitted)

1. Refactored `src-tauri/src/pipeline/backend_ops.rs`:
   - replaced `NativeIngestScriptSyncBackendOps` with `NativeIngestAwsSyncBackendOps`.
   - native ingest+sync path now stores `runner` + `app_root` directly instead of embedding `ScriptPipelineBackendOps`.
2. Preserved native behavior:
   - ingest remains Rust-native via `ProjectsStore::ingest_run_log`.
   - S3 sync remains Rust precheck + `aws s3 sync` execution path.
   - no contract/API surface changes.
3. Updated default runtime wiring:
   - `default_backend_ops_with_native_ingest(...)` now constructs native ops directly with default app root + `StdPipelineCommandRunner`.
4. Updated backend ops tests to the new constructor while preserving assertions for:
   - no external command call on precheck failures/skips
   - AWS CLI invocation when sync precheck is ready.
5. Synced roadmap note:
   - `docs/ROADMAP.md` now states runtime native ingest+sync no longer depends on script-wrapper backend ops state.

### Validation

1. `cargo fmt` (in `src-tauri`) -> passing.
2. `cargo test --test pipeline_trigger_endpoints --test runs_assets_endpoints --test http_contract_surface` -> passing.
3. `cargo test hybrid_sync_precheck_skips_missing_local_without_calling_script` -> passing.
4. `cargo test hybrid_sync_executes_aws_cli_when_ready` -> passing.
5. `cargo test` (in `src-tauri`) -> passing (full suite, `0` failed).

### Known Gaps / Risks

1. Changes remain local/uncommitted along with prior secrets/API/CLI/handoff updates.
2. Broader runtime/tool migration milestones remain (external tool adapters and remaining legacy scripts outside this backend_ops decoupling step).

### Next Chat Starting Point

1. Run `[$general-git](/home/ldco/.codex/skills/general-git/SKILL.md)` to finalize/push the accumulated continuation slices.
2. Continue Phase 1 migration by reducing remaining script dependencies in tool adapter execution paths where feasible.

## Session Update (2026-02-25, secrets operator CLI continuation)

### Scope

1. Continue after secrets rotation/status API landing.
2. Add a local operator CLI path for secret rotation/status workflows without HTTP dependency.
3. Keep docs and validation aligned.

### What Landed (this session, local/uncommitted)

1. Added new CLI commands in `src-tauri/src/main.rs`:
   - `cargo run -- secrets-rotation-status --project-slug <slug>`
   - `cargo run -- secrets-rotate --project-slug <slug> [--from-key-ref <ref>] [--force]`
2. CLI implementation details:
   - resolves backend config and initializes `ProjectsStore` using the same app-root/db resolution path as runtime defaults.
   - supports SQLite backend for these CLI operations; explicitly errors with a clear message if `KROMA_BACKEND_DB_URL` (PostgreSQL mode) is set.
   - returns JSON payloads for both commands (`status` or `rotation` body).
3. Added CLI argument parser unit tests in `src-tauri/src/main.rs`:
   - required `--project-slug` validation
   - optional `--from-key-ref` and `--force` parsing
4. Updated docs:
   - `README.md` now includes CLI examples for both rotation status and rotation execution.
   - `docs/ROADMAP.md` now notes the local operator CLI fallback under secrets hardening progress.

### Validation

1. `cargo fmt` (in `src-tauri`) -> passing.
2. `cargo test parse_rotate_accepts_optional_flags` -> passing.
3. `cargo test parse_rotation_status_accepts_project_slug` -> passing.
4. `cargo test` (in `src-tauri`) -> passing (full suite, `0` failed).

### Known Gaps / Risks

1. All changes remain local/uncommitted (including handoff updates).
2. CLI currently does not implement PostgreSQL backend operations (same project-wide limitation as the rest of backend wiring).

### Next Chat Starting Point

1. Run `[$general-git](/home/ldco/.codex/skills/general-git/SKILL.md)` to finalize and push the accumulated secrets rotation/status/API/CLI slice.
2. Continue Phase 1 runtime consolidation work as needed after commit finalization.

## Session Update (2026-02-25, secrets rotation visibility + continuation)

### Scope

1. Continue immediately after landing project secret key rotation/re-encryption support.
2. Add explicit visibility for legacy plaintext secret rows to reduce migration risk.
3. Keep API/OpenAPI route contract parity and full test suite green.

### What Landed (this session, local/uncommitted)

1. Added secret encryption-status aggregation in Rust store (`src-tauri/src/db/projects/secrets.rs`):
   - new `ProjectsStore::get_project_secret_encryption_status(slug)` method.
   - returns `total`, `encrypted`, `plaintext`, `empty`, and `key_refs` distribution.
2. Added secrets migration visibility endpoint:
   - `GET /api/projects/{slug}/secrets/rotation-status`
   - handler implemented in `src-tauri/src/api/secrets.rs`.
   - route wired in `src-tauri/src/api/server.rs` and `src-tauri/src/api/routes.rs`.
3. Updated contract artifacts:
   - OpenAPI path added in `openapi/backend-api.openapi.yaml`.
   - route catalog count updated to `74`.
   - HTTP contract expected-status matrix updated.
4. Added/updated tests:
   - unit: `encryption_status_counts_plaintext_encrypted_and_empty_rows`
   - integration: `secrets_rotation_status_reports_plaintext_and_key_refs`
   - existing secrets + contract parity/surface tests remain passing.
5. Updated docs:
   - `README.md` includes status endpoint usage and rotation workflow references.
   - `docs/ROADMAP.md` notes explicit migration visibility endpoint.

### Validation

1. `cargo fmt` (in `src-tauri`) -> passing.
2. `cargo test encryption_status_counts_plaintext_encrypted_and_empty_rows` -> passing.
3. `cargo test secrets_rotation_status_reports_plaintext_and_key_refs` -> passing.
4. `cargo test --test secrets_endpoints --test http_contract_surface --test contract_parity` -> passing.
5. `cargo test` (in `src-tauri`) -> passing (full suite, `0` failed).

### Known Gaps / Risks

1. Changes remain local/uncommitted (including this handoff update).
2. Secrets rotation/status is now available, but broader Phase 1 runtime consolidation milestones remain open (generation/orchestration migration out of `scripts/image-lab.mjs`).

### Next Chat Starting Point

1. Run `[$general-git](/home/ldco/.codex/skills/general-git/SKILL.md)` to finalize and push this secrets rotation + status slice.
2. Continue Phase 1 runtime consolidation: next extraction step from `scripts/image-lab.mjs` into Rust orchestration/execution modules.
3. Optionally add CLI/operator convenience path for secret rotation/status if non-API maintenance flow is needed.

## Session Update (2026-02-25, git finalization + handoff/git skill standardization)

### Scope

1. Finalize and push the in-progress Rust security/auth roadmap work to `master`.
2. Create a reusable handoff skill so next chats can continue without rediscovery.
3. Create/upgrade a reusable git finalization skill with deterministic branch-default behavior.
4. Refresh `NEXT_CHAT_HANDOFF.md` to reflect the true latest repository and workflow state.

### What Landed (this session, local/uncommitted)

1. Finalized and pushed roadmap security changes with `general-git` default branch behavior (`master`):
   - commit: `8b3fce3`
   - subject: `feat: harden auth bootstrap and encrypt project secrets`
   - pushed: `origin/master` advanced `7d0934b -> 8b3fce3`.
2. Added a new global skill at `/home/ldco/.codex/skills/general-handoff`:
   - `SKILL.md` now enforces evidence-backed, complete session handoffs into `NEXT_CHAT_HANDOFF.md`.
   - `references/handoff-template.md` added with standardized section template + completeness checklist.
3. Upgraded existing global skill at `/home/ldco/.codex/skills/general-git`:
   - `SKILL.md` now enforces best-practice commit/push workflow with explicit branch rule:
     - no argument -> push to `master`
     - single trailing word -> use that as branch.
   - `references/git-finalization-checklist.md` added for deterministic command sequence.
4. Synchronized handoff continuity:
   - previous top handoff header state was stale (`09ed65e`/`7d0934b`) and is now corrected to `8b3fce3`/`8b3fce3`.

### Validation

1. `python3 /home/ldco/.codex/skills/.system/skill-creator/scripts/quick_validate.py /home/ldco/.codex/skills/general-handoff` -> passing (`Skill is valid!`).
2. `python3 /home/ldco/.codex/skills/.system/skill-creator/scripts/quick_validate.py /home/ldco/.codex/skills/general-git` -> first run failed (YAML frontmatter parse), then fixed and rerun -> passing (`Skill is valid!`).
3. `git fetch origin && git pull --ff-only origin master` -> passing (`Already up to date`).
4. `git diff --cached --check` before commit -> passing (no output/errors).
5. `git commit ...` -> passing (created `8b3fce3`, `12 files changed, 821 insertions, 21 deletions`).
6. `git push -u origin master` -> passing (`master -> master`, tracking set).

### Known Gaps / Risks

1. `general-handoff` and `general-git` live under `/home/ldco/.codex/skills` and are not part of the project repository; other machines/users will not receive these updates unless Codex home is synchronized separately.
2. Project roadmap follow-up items remain open after the pushed security patch (for example key rotation/re-encryption flow for `project_secrets.key_ref`).

### Next Chat Starting Point

1. Decide whether to vendor/document these global skills in-repo (or install via shared bootstrap) so team environments stay consistent.
2. Continue security roadmap work from `8b3fce3`: implement key rotation + re-encryption flow for encrypted `project_secrets`.
3. After this handoff edit, optionally run `[$general-git](/home/ldco/.codex/skills/general-git/SKILL.md)` again to commit/push the updated handoff file itself.

## Session Update (2026-02-25, secrets-at-rest migration + auth protected-route coverage)

### Scope

1. Continue roadmap security work from the previous handoff.
2. Migrate Rust `project_secrets` storage from plaintext-at-rest to encrypted-at-rest.
3. Close the auth test blind spot for a representative protected endpoint with dev bypass disabled.

### What Landed (this session, local/uncommitted)

1. Added encrypted-at-rest secret writes in `src-tauri/src/db/projects/secrets.rs`:
   - `upsert_project_secret(...)` now encrypts incoming secret values before DB insert/update.
   - ciphertext format is `enc:v1:<base64url(payload)>` where payload is `nonce(12B) + AES-256-GCM ciphertext+tag`.
   - added schema support for `key_ref` on `project_secrets` (table create + migration via `ensure_column`).
2. Added master-key loading/generation parity behavior in Rust:
   - first source: `IAT_MASTER_KEY` (base64url 32-byte key)
   - fallback: `IAT_MASTER_KEY_FILE` (default `var/backend/master.key`)
   - if missing, generate and persist local key file (Unix permissions set to `0600`).
3. Extended error handling for non-SQL internal failures:
   - added `ProjectsRepoError::Internal(String)` in `src-tauri/src/db/projects.rs`.
   - mapped internal repo errors to sanitized `500` in `src-tauri/src/api/handler_utils.rs`.
4. Added/updated tests:
   - `src-tauri/src/db/projects/secrets.rs`:
     - `upsert_secret_encrypts_value_at_rest`
     - `upsert_secret_creates_master_key_file_when_missing`
   - `src-tauri/tests/auth_endpoints.rs`:
     - `protected_projects_endpoint_requires_bearer_when_dev_bypass_is_off`
5. Added docs updates:
   - `README.md` config table now documents `IAT_MASTER_KEY` and `IAT_MASTER_KEY_FILE`.
   - `docs/ROADMAP.md` marks secrets-at-rest hardening as landed.

### Validation

1. `cargo fmt` (in `src-tauri`) -> passing.
2. `cargo check` (in `src-tauri`) -> passing.
3. `cargo test` (in `src-tauri`) -> passing (full suite, including new secrets/auth coverage).

### Next Chat Starting Point

1. Add an explicit key-rotation path for secret re-encryption (`key_ref` is now present but rotation flow is not implemented).
2. Decide whether non-secret bootstrap/import/export paths should emit explicit migration warnings when legacy plaintext rows are detected.
3. Continue Phase 1 runtime consolidation milestones (generation/orchestration migration from `scripts/image-lab.mjs` into Rust modules).

## Session Update (2026-02-25, auth bootstrap lockout fix + security follow-through)

### Scope

1. Resolve auth bootstrap deadlock after secure-by-default bypass change.
2. Keep auth posture hardened without reopening broad unauthenticated access.
3. Add explicit tests and docs for the bootstrap policy.

### What Landed (this session, local/uncommitted)

1. Added active-token bootstrap guard in `ProjectsStore`:
   - `has_active_api_tokens()` in `src-tauri/src/db/projects/auth_audit.rs`.
2. Extended app auth state/policy wiring in `src-tauri/src/api/server.rs`:
   - new state field: `auth_bootstrap_allow_unauth_token_create`
   - new router constructor for explicit auth-mode testing:
     - `build_router_with_projects_store_auth_mode(...)`
   - bootstrap policy defaults:
     - controlled by `KROMA_API_AUTH_BOOTSTRAP_FIRST_TOKEN` (default `true`)
     - only effective when `KROMA_BACKEND_BIND` resolves to loopback.
3. Updated auth middleware in `src-tauri/src/api/auth.rs`:
   - allows unauthenticated `POST /auth/token` only when:
     - route is exactly `/auth/token` + `POST`
     - bootstrap policy is enabled
     - there are no active API tokens
   - marks bootstrap pass-through requests with explicit auth principal (`BootstrapFirstToken`) instead of generic dev bypass.
   - once a token exists, bearer auth is required again for token creation.
4. Added atomic first-token DB guard in `src-tauri/src/db/projects/auth_audit.rs`:
   - new `create_first_api_token_local(...)` uses `TransactionBehavior::Immediate`
   - rechecks active token count in-transaction before insert to close concurrent bootstrap mint race.
5. Added integration coverage:
   - new file `src-tauri/tests/auth_endpoints.rs`
   - validates:
     - first unauthenticated token creation succeeds once
     - second unauthenticated creation is rejected (`401`)
     - authenticated token creation still works
     - bootstrap path can be disabled.
6. Added/updated docs:
   - `README.md` config table includes `KROMA_API_AUTH_BOOTSTRAP_FIRST_TOKEN`.
   - `docs/ROADMAP.md` notes constrained first-token bootstrap behavior.

### Validation

1. `cargo fmt` (in `src-tauri`) -> passing.
2. `cargo check` (in `src-tauri`) -> passing.
3. `cargo test --test auth_endpoints` -> passing (`2` tests).
4. `cargo test` (in `src-tauri`) -> passing (full suite).
5. `cargo clippy --all-targets --all-features -- -D warnings` -> not runnable in current env (`cargo-clippy` component missing).

### Next Chat Starting Point

1. Continue roadmap security work: encrypted-at-rest migration for Rust `project_secrets` parity with legacy behavior.
2. Add at least one auth integration test path using the non-dev-bypass router for a representative protected `/api/projects/{slug}` endpoint (to avoid auth-test blind spots).
3. Decide whether to add a local CLI bootstrap-token command as an ops fallback when first-token bootstrap is intentionally disabled.

## Session Update (2026-02-25, roadmap continuation: auth hardening + explicit legacy gating)

### Scope

1. Continue roadmap execution with security hardening and migration ergonomics.
2. Keep test coverage green while removing implicit insecure defaults.

### What Landed (this session, local/uncommitted)

1. Auth dev bypass default changed to secure-by-default (`false` when `KROMA_API_AUTH_DEV_BYPASS` is unset) in `src-tauri/src/api/server.rs`.
2. Added explicit dev-bypass router constructors for test usage:
   - `build_router_with_projects_store_dev_bypass(...)`
   - `build_router_with_projects_store_and_pipeline_trigger_dev_bypass(...)`
3. Updated integration tests to use explicit dev-bypass router constructors (no implicit bypass assumptions).
4. Added unit tests covering auth bypass env behavior in `src-tauri/src/api/server.rs`.
5. Updated npm legacy-script commands to explicitly opt in:
   - `KROMA_ENABLE_LEGACY_SCRIPTS=1` for `backend:init`, `backend:migrate`, `backend:user:local`, `backend:project:list`, `backend:worker`, `backend:worker:once`.
6. Updated docs:
   - `README.md` config table now documents `KROMA_API_AUTH_DEV_BYPASS` default.
   - `docs/ROADMAP.md` updated with this milestone.

### Validation

1. `cargo fmt` (in `src-tauri`) -> passing.
2. `cargo test` (in `src-tauri`) -> passing (`117` tests in lib + integration suite, `0` failed).
3. `npm run -s backend:migrate -- --help` -> passing (legacy migration helper is callable with explicit gate).

### Next Chat Starting Point

1. Continue roadmap security work: migrate Rust `project_secrets` storage from plaintext-at-rest to encrypted-at-rest parity with legacy behavior.
2. Keep reducing script fallback surface and remove migrated script paths once Rust parity lands.

## Session Update (2026-02-24, tool adapter refactor continuation)

### Scope

1. Resume from interrupted `tool_adapters` extraction work after IDE crash.
2. Continue splitting large helper/config/report blocks out of `src-tauri/src/pipeline/tool_adapters.rs`.
3. Refresh handoff with the exact local WIP state.

### What Landed (this session, local/uncommitted)

1. Added extracted modules:
   - `src-tauri/src/pipeline/tool_adapters/bgremove_config.rs`
   - `src-tauri/src/pipeline/tool_adapters/color_config.rs`
   - `src-tauri/src/pipeline/tool_adapters/color_ops.rs`
   - `src-tauri/src/pipeline/tool_adapters/qa_report.rs`
   - `src-tauri/src/pipeline/tool_adapters/file_ops.rs`
2. `src-tauri/src/pipeline/tool_adapters.rs` now wires and uses the extracted modules (`mod ...` + imports) and removes the duplicated inline helper implementations.
3. Cleanup pass removed stale imports so the refactor checkpoint compiles warning-free in normal `cargo check`.

### Repo State Snapshot

1. Branch: `master`
2. HEAD: `7d0934b`
3. Upstream (`origin/master`): `7d0934b`
4. Worktree dirty files:
   - `src-tauri/src/pipeline/tool_adapters.rs`
   - `src-tauri/src/pipeline/tool_adapters/bgremove_config.rs`
   - `src-tauri/src/pipeline/tool_adapters/color_config.rs`
   - `src-tauri/src/pipeline/tool_adapters/color_ops.rs`
   - `src-tauri/src/pipeline/tool_adapters/file_ops.rs`
   - `src-tauri/src/pipeline/tool_adapters/qa_report.rs`
   - `NEXT_CHAT_HANDOFF.md`

### Validation

1. `cargo check` (run in `src-tauri`) -> passing.
2. `cargo test tool_adapters -- --nocapture` (run in `src-tauri`) -> passing (`14 passed, 0 failed`).

### Next Chat Starting Point

1. Continue module extraction in `tool_adapters.rs` only if it still improves readability; remaining free helper footprint is now small.
2. Stage and commit this refactor checkpoint once the module boundaries are accepted.

## Session Update (2026-02-24, handoff refresh)

### Scope

1. User requested `NEXT_CHAT_HANDOFF.md` refresh.
2. No source/runtime code changes were made in this session.

### Repo State Snapshot

1. Branch: `master`
2. HEAD: `d18510c`
3. Upstream (`origin/master`): `a7dfc6f`
4. Worktree dirty files: `NEXT_CHAT_HANDOFF.md` only

### Validation

1. Not run (docs-only update).

### Next Chat Starting Point

1. Continue from the existing "Suggested Starting Point For Next Chat" section later in this file.
2. Prioritize remaining Rust runtime migration debt: replace transitional inline Python shims and reduce script-runtime dependencies.

## Architecture Decision (2026-02-24, desktop persistence policy)

### Decision

1. Keep desktop/local architecture on:
   - SQLite for metadata
   - local filesystem for image assets
2. Keep cloud storage optional:
   - S3 sync remains an additive capability (backup/sync/team workflows), not a baseline dependency.
3. Defer PostgreSQL:
   - no immediate PostgreSQL backend wiring work for desktop mode.
   - revisit only when hosted/shared multi-user mode becomes an active product requirement.

### Roadmap Impact

1. Continue Phase 1 on runtime/tooling Rust consolidation and local reliability hardening.
2. Treat PostgreSQL support as future hosted-tier work, not current critical path.

## Latest Patch (2026-02-24, python upscale inline shim removal)

### Scope

1. `src-tauri/src/pipeline/tool_adapters.rs`
2. `config/postprocess.json.example`
3. `tool_adapters` unit tests

### What Landed

1. Replaced the remaining inline RealESRGAN Python shim (`python -c ...`) with direct script invocation.
   - Python upscale now executes:
   - `<python_bin> tools/realesrgan-python/src/Real-ESRGAN/inference_realesrgan.py ...`
2. Added `upscale.python.inference_script` config support with secure path validation under app root.
3. Added explicit runtime guard:
   - returns a clear error when inference script path is missing, with setup hint (`scripts/setup-realesrgan-python.sh`).
4. Updated unit tests:
   - python upscale command-shape assertions now check script-path invocation (not `-c`).
   - new regression test for missing inference script path error.
5. Updated `config/postprocess.json.example` with explicit Python upscale fields (`python_bin`, `inference_script`, model/tile knobs).

### Validation

1. `cargo fmt --manifest-path src-tauri/Cargo.toml`
2. `cargo test -q tool_adapters::tests:: --manifest-path src-tauri/Cargo.toml`
3. `cargo test -q --manifest-path src-tauri/Cargo.toml`

Result: passing.

### Remaining Risk / Next Step

1. `tool_adapters.rs` is still a large hotspot; next migration step should split it into focused modules (`upscale`, `bgremove`, `color`, `qa`, shared path/config helpers).

## Latest Patch (2026-02-24, request-path containment + rembg module invocation)

### Scope

1. `src-tauri/src/pipeline/tool_adapters.rs`
2. `tool_adapters` unit tests

### What Landed

1. Added strict request-path containment for native tool adapter request fields.
   - New resolver rejects empty, absolute, and parent-traversal paths (`..`) for request/config paths expected to live under app root.
   - Applied to `generate_one`, `upscale`, `color`, `bgremove`, `qa`, `archive_bad`, and postprocess config loading.
2. Replaced inline rembg shim execution with direct module invocation:
   - `run_bgremove_rembg(...)` now calls `python -m rembg i -m <model> <input> <output>`.
   - Removed the embedded `REMBG_INLINE_PYTHON` script blob.
3. Updated rembg command-shape assertions in tests to match module invocation.
4. Added regression tests for path hardening:
   - resolver rejects absolute paths and `..` traversal
   - native QA request rejects traversal input path

### Validation

1. `cargo fmt --manifest-path src-tauri/Cargo.toml`
2. `cargo test -q tool_adapters::tests:: --manifest-path src-tauri/Cargo.toml`
3. `cargo test -q --manifest-path src-tauri/Cargo.toml`

Result: passing.

### Remaining Risk / Next Step

1. Python upscale still uses an embedded inline shim (`REALESRGAN_UPSCALE_INLINE_PYTHON`) and remains the next migration target.

## Latest Review Pass (2026-02-24, QA adapter cleanup follow-up)

### Scope Reviewed

1. `38e6308` `refactor(pipeline): drop qa python override from rust adapters`
2. `e41e263` `chore(rust): scope tool adapter CommandOutput import to tests`
3. Surrounding QA adapter/runtime flow in:
   - `src-tauri/src/pipeline/runtime.rs`
   - `src-tauri/src/pipeline/tool_adapters.rs`

### Issues Discovered (this pass)

1. No new functional regressions found in the reviewed commits.
2. Compatibility surface change (intentional): `QaCheckRequest` no longer supports request-level `qa_python_bin` override.
   - Impact: any internal callers attempting to set that field will now fail at compile time (preferred over silent runtime drift).

### Fixes Implemented (this pass)

1. No additional code fix was required during this review pass.
2. Verified the warning cleanup (`CommandOutput` test-only import) is correct and does not affect non-test builds.

### Validation (this pass)

1. `cargo test -q tool_adapters::tests:: --manifest-path src-tauri/Cargo.toml`
2. `cargo test -q pipeline::runtime:: --manifest-path src-tauri/Cargo.toml`
3. `rg -n "qa_python_bin" src-tauri/src src-tauri/tests` (no remaining matches)

Result: passing.

### Current Status (this pass)

1. QA/output-guard Rust adapter path no longer carries dead `qa_python_bin` request plumbing.
2. `src-tauri/src/pipeline/tool_adapters.rs` warning cleanup is committed and validated.
3. Repository was clean before this handoff update (no pending code changes).

### Completed Work (this pass)

1. Reviewed the latest two commits in full diff form and checked surrounding code paths.
2. Ran targeted regression validation for `tool_adapters` and `pipeline::runtime`.
3. Confirmed the `qa_python_bin` removal is complete within `src-tauri` and did not leave stale Rust references.

### Open Tasks / Remaining Risks (this pass)

1. Risk (low): downstream script/tooling docs outside `src-tauri` may still mention `--qa-python-bin`; not validated in this pass.
2. Testing gap: no explicit unit test asserts that `build_qa_command(...)` no longer emits `--qa-python-bin` (behavior is indirectly covered by type removal + grep).
3. Migration risk shifted: native adapter/runtime no longer depends on wrapper script files, but now embeds inline Python shims for `rembg` and python-upscale.
   - Impact: duplication risk vs the standalone scripts if both remain maintained in parallel.

### Recommended Next Steps (updated)

1. Continue the Rust migration cleanup by replacing the inline Python shims (rembg/upscale) with Rust-owned execution or a cleaner direct tool/module invocation path.
2. Add one focused regression test for QA command args only if the script adapter path remains long enough to justify the extra coverage.
3. Run a broader `src-tauri` test subset after the next adapter cleanup to catch cross-module regressions early.

## Latest Patch (2026-02-24, native wrapper-file dependency removal)

### Scope

1. `src-tauri/src/pipeline/tool_adapters.rs` (`bgremove` + python `upscale` native backend paths)
2. `tool_adapters` unit tests covering native `bgremove`/`upscale` command behavior

### What Landed

1. Native `bgremove` rembg backend no longer requires `scripts/rembg-remove.py` at runtime.
2. Native python `upscale` backend no longer requires `scripts/realesrgan-python-upscale.py` at runtime.
3. `run_bgremove_rembg(...)` now invokes the configured Python interpreter directly with an inline rembg/Pillow routine (`python -c ...`) to preserve current output-format behavior (`png`/`jpg`/`webp`).
4. `upscale_native(...)` python backend path now invokes an inline RealESRGAN Python routine (`python -c ...`) that preserves the previous wrapper behavior (source-path injection, model selection/download, directory/file handling).
5. Removed wrapper-file existence gates from both native paths.
6. Updated tests to stop creating/asserting wrapper script paths and instead assert direct Python invocation (`-c`) for rembg/upscale paths.

### Validation

1. `cargo fmt --manifest-path src-tauri/Cargo.toml`
2. `cargo test -q tool_adapters::tests:: --manifest-path src-tauri/Cargo.toml`
3. `cargo test -q pipeline::runtime:: --manifest-path src-tauri/Cargo.toml`

Result: passing.

### Remaining Risk / Next Step

1. Inline Python shim duplication is now one transitional debt; next step is to replace these with Rust-owned execution or direct module invocations without embedded script blobs.
2. Legacy script orchestration/adapter types (`ScriptPipelineOrchestrator`, `ScriptPipelineToolAdapters`) still exist for compatibility/test coverage and remain a cleanup target.

## Latest Patch (2026-02-24, script-compat decoupling in native adapter path)

### Scope

1. `src-tauri/src/pipeline/tool_adapters.rs`
2. `src-tauri/src/pipeline/runtime.rs`

### What Landed

1. `NativeQaArchiveScriptToolAdapters` no longer embeds `ScriptPipelineToolAdapters` as an internal field.
   - It now owns `runner` and `app_root` directly, removing an unnecessary script-adapter coupling from the active native path.
2. Removed dead script-only default constructors that were no longer referenced anywhere:
   - `default_script_pipeline_tool_adapters(...)`
   - `default_script_pipeline_orchestrator(...)`
3. Removed legacy passthrough customization methods on `NativeQaArchiveScriptToolAdapters` that only mutated script adapter settings and were unused.

### Validation

1. `cargo fmt --manifest-path src-tauri/Cargo.toml`
2. `cargo test -q tool_adapters::tests:: --manifest-path src-tauri/Cargo.toml`
3. `cargo test -q pipeline::runtime:: --manifest-path src-tauri/Cargo.toml`

Result: passing.

### Remaining Risk / Next Step

1. `ScriptPipelineOrchestrator` and `ScriptPipelineToolAdapters` themselves still exist and are the next cleanup seam if compatibility exports/tests are no longer needed.

## Latest Patch (2026-02-24, legacy script types gated out of production build)

### Scope

1. `src-tauri/src/pipeline/runtime.rs`
2. `src-tauri/src/pipeline/tool_adapters.rs`

### What Landed

1. `ScriptPipelineOrchestrator` and its impls are now `#[cfg(test)]` only.
2. `ScriptPipelineToolAdapters` and its impls are now `#[cfg(test)]` only.
3. Script-only helper functions/imports uncovered by the gating pass were also moved behind `#[cfg(test)]`:
   - `append_pipeline_options_args(...)`
   - `RustPlanningPreflightSummary::ids_preview(...)`
   - `strip_script_planning_inputs_when_jobs_file_present(...)`
   - `write_planned_jobs_temp_file(...)`
   - `serde::de::DeserializeOwned` import in `tool_adapters.rs`
4. Production build surface no longer compiles the legacy script orchestrator/adapter compatibility implementations.

### Validation

1. `cargo check -q --manifest-path src-tauri/Cargo.toml`
2. `cargo test -q tool_adapters::tests:: --manifest-path src-tauri/Cargo.toml`
3. `cargo test -q pipeline::runtime:: --manifest-path src-tauri/Cargo.toml`

Result: passing.

### Remaining Risk / Next Step

1. Legacy script compatibility tests/types still exist in test builds; next step is full deletion (types + tests) once the team decides those regression fixtures are no longer needed.

## Latest Patch (2026-02-24, legacy script orchestrator/adapter deletion)

### Scope

1. `src-tauri/src/pipeline/runtime.rs`
2. `src-tauri/src/pipeline/tool_adapters.rs`
3. `src-tauri/src/api/runs_assets.rs`

### What Landed

1. Deleted legacy script compatibility types and their dedicated tests:
   - `ScriptPipelineOrchestrator` (runtime)
   - `ScriptPipelineToolAdapters` (tool adapters)
2. Removed script-only helper functions that existed solely for the deleted compatibility path/tests.
3. Removed dead `ScriptNotFound` variants from:
   - `PipelineRuntimeError`
   - `ToolAdapterError`
4. Updated API trigger error mapping in `src-tauri/src/api/runs_assets.rs` to stop matching the removed `PipelineRuntimeError::ScriptNotFound`.
5. Cleaned resulting dead test scaffolding/warnings in `runtime.rs` (unused `FakeRunner`, `ids_preview`).

### Validation

1. `cargo fmt --manifest-path src-tauri/Cargo.toml`
2. `cargo check -q --manifest-path src-tauri/Cargo.toml`
3. `cargo test -q tool_adapters::tests:: --manifest-path src-tauri/Cargo.toml`
4. `cargo test -q pipeline::runtime:: --manifest-path src-tauri/Cargo.toml`

Result: passing.

### Remaining Risk / Next Step

1. The active Rust path still carries transitional compatibility around script-shaped run summaries/run-log normalization in post-run handling.
2. Inline Python shims (`rembg`, python-upscale) remain the primary migration debt.

## Architecture Decisions (2026-02-24, Rust-Only Direction)

1. Default pipeline execution path is now Rust-only for orchestration (`dry`/`run` wrappers + post-run ingest).
   - No default fallback to `scripts/image-lab.mjs` for unsupported request shapes.
   - Unsupported shapes now fail fast with a Rust `PlanningPreflight` error.
2. Compatibility code is no longer treated as a design constraint.
   - Script-backed behaviors may now fail fast instead of delegating when unsupported in Rust (example: unsupported `bgremove` backend names).
3. New work should replace script ownership directly, not add more adapter layering.
   - Priority order for remaining script deletions:
   - `color` (done in this batch)
   - `qa/output-guard` (done in this batch)
   - `bgremove` rembg helper script
   - `upscale` python helper script
   - final removal of `image-lab.mjs` legacy orchestration + script tool adapters

## Completed Work (Latest Batch, 2026-02-24)

1. Removed default Node orchestration fallback from the runtime stack.
   - `default_pipeline_orchestrator_with_rust_post_run_*` now uses a strict Rust-only unsupported-request inner orchestrator.
2. Hardened Rust-native `bgremove` behavior:
   - unsupported backend names now fail in Rust instead of falling back to `image-lab.mjs`.
3. Ported output guard / QA from `scripts/output-guard.py` into Rust (`image` crate based analysis).
   - direct chroma-delta computation and report shaping now happen in Rust.
4. Ported color correction from `scripts/apply-color-correction.py` into Rust (`image` crate based pipeline).
   - file + directory input modes
   - typed color profile settings parsing
   - profile transforms (brightness/contrast/saturation/sharpness/gamma/channel multipliers)
5. Expanded regression tests to assert Rust-native behavior (no subprocess calls for QA/color).

## Remaining Tasks (High Priority)

1. Replace inline Python shims used by native adapters with Rust-owned execution or cleaner direct-module invocation paths:
   - rembg backend shim
   - python RealESRGAN upscale shim
2. Reduce remaining script-shape compatibility in runtime/post-run handling as Rust run output ownership expands.
3. Split `src-tauri/src/pipeline/tool_adapters.rs` into focused modules (generation/bgremove/upscale/color/qa/archive + shared image ops/config).
   - Current file is now a major structural hotspot and should be decomposed after the remaining script dependencies are removed.

## Next Phase Goals (Immediate)

1. Replace inline python shims in `tool_adapters.rs` (rembg + python upscale) with Rust-owned backend paths or cleaner direct binary/module invocation.
2. After that, continue collapsing script-shaped compatibility layers in runtime/post-run handling.

## Current Status

1. Rust backend (`src-tauri`, `axum` + SQLite) is the primary local API and remains the active backend surface.
2. Bootstrap prompt exchange is implemented and shipped on `master`:
   - `GET /api/projects/{slug}/bootstrap-prompt`
   - `POST /api/projects/{slug}/bootstrap-import` (`merge` / `replace`)
   - `dry_run` preview mode
   - change-summary / diff metadata in responses
3. Bootstrap import/export coverage now includes:
   - project metadata
   - provider accounts
   - style guides
   - `characters` (committed on `master`)
   - `reference_sets` + nested items (committed on `master`)
   - `secrets` metadata only (`provider_code`, `secret_name`, `has_value`; no secret values) (committed on `master`)
   - prompt templates
4. Product direction clarified: `scripts/` is transitional only; target architecture is a single Rust-owned application (API + runtime/orchestration + workers).
5. Rust pipeline runtime/trigger boundary and Rust-owned typed trigger endpoint (`POST /api/projects/{slug}/runs/trigger`) are now committed on `master` (script-backed execution remains behind the runtime boundary).
6. Product-scope cleanup is now committed on `master`: voice feature code and the old Python HTTP backend entrypoint (`scripts/backend_api.py`) were removed; active pipeline/runtime script dependencies remain.
7. `docs/ROADMAP.md` is now tracked on `master` as the day-to-day progress board (kept alongside the larger spec doc).
8. Rust pipeline runtime now has a file-backed layered settings path for pipeline planning/runtime defaults:
   - app settings (preferred): `config/pipeline.settings.toml`
   - app settings fallback: `config/pipeline.settings.json`
   - project settings: `<project_root>/.kroma/pipeline.settings.json`
   - precedence enforced in Rust: request overrides > project settings > app settings > Rust defaults
9. Rust now owns additional `image-lab.mjs` planning/runtime configuration logic:
   - manifest parsing for `generation`, `safe_batch_limit`, and `output_guard`
   - postprocess planning config parsing/validation + planned postprocess record shaping
   - Rust dry-run run-log top-level/job planned postprocess + output-guard fields (manifest/settings-aware)
10. Example config templates now exist under `config/`:
   - `pipeline.settings.toml.example`
   - `pipeline.manifest.json.example`
   - `postprocess.json.example`

## Latest Review Pass (2026-02-23, settings layer + runtime wiring)

### Scope Reviewed

1. `src-tauri/src/pipeline/settings_layer.rs` (new layered config loader/parser)
2. `src-tauri/src/pipeline/runtime.rs` (effective request merge + settings application in dry/script paths)
3. `src-tauri/src/pipeline/postprocess_planning.rs` (planning subset parser/resolver)
4. `README.md` + `config/*.example` templates (new config documentation/examples)

### Issues Discovered (this pass)

1. Bug (fixed): explicit relative project settings paths were resolved from process CWD instead of `project_root`.
   - Impact: `project_settings_path` overrides could silently miss the intended file depending on launch directory.
2. Validation bug (fixed): settings parser accepted empty strings for path/profile fields.
   - Impact: invalid values like `"manifest_path": "   "` flowed into runtime resolution and failed later with less clear errors.
3. Performance/behavior issue (fixed): `RustDryRunPipelineOrchestrator` resolved layered settings even for `run` mode before delegating to the inner script orchestrator, which also resolves settings.
   - Impact: duplicated config file reads on run path and unnecessary extra parsing work.

### Fixes Implemented (this pass)

1. `settings_layer`: explicit relative project settings paths now resolve relative to `project_root`.
2. `settings_layer`: empty-string values are rejected as invalid typed settings fields.
3. `runtime`: `RustDryRunPipelineOrchestrator` now short-circuits non-dry requests to the inner orchestrator before layered settings resolution (removes duplicate run-path config loads).
4. Added regression coverage for:
   - empty-string settings rejection
   - explicit relative project settings path resolution
5. Added app-settings TOML support (new) with JSON fallback:
   - app loader now prefers `config/pipeline.settings.toml`
   - legacy `config/pipeline.settings.json` remains supported as fallback
6. Added config examples + README docs for the Rust-owned layered pipeline settings path.

### Validation (this pass)

1. `cargo test pipeline::settings_layer --lib`
2. `cargo test pipeline::runtime --lib`
3. `cargo test pipeline::trigger --lib`

Result: passing.

### Remaining Risks / TODO (this pass)

1. Request-level postprocess toggle booleans in `PipelinePostprocessOptions` are not tri-state (`bool` instead of `Option<bool>`).
   - Impact: runtime can express "enable" via request overrides, but cannot cleanly express an explicit request-level "disable" that overrides a settings-file `true` value.
   - Current mitigation: not exposed on typed API yet; mostly affects future request-level config override surfaces.
2. Explicit app/project settings paths are still treated as optional if the file is missing (loader returns defaults).
   - Could be hardened to fail fast for explicitly provided paths to avoid silent misconfiguration.
3. Project settings format is JSON-only for now (intentional compatibility choice); TOML support is app-layer only.

### Recommended Next Steps

1. Add a Rust config validation command/path (CLI or endpoint-internal utility) that validates layered app/project settings + manifest + postprocess config together before pipeline runs.
2. Convert request-level postprocess toggles in the typed runtime/request model to tri-state semantics (`Option<bool>` or equivalent) before exposing them on the API/UI.
3. Continue run-mode migration: move generation/post-process execution loop from `scripts/image-lab.mjs` into Rust execution services and replace script-owned run-log writing.

## Latest Patch (2026-02-23, runtime dry-run log shaping cleanup)

### Scope Reviewed

1. `src-tauri/src/pipeline/runtime.rs` (Rust dry-run run-log JSON assembly)
2. `src-tauri/src/pipeline/execution.rs` (typed planned run-log job record builders)

### Issues Discovered

1. Design debt (fixed): `RustDryRunPipelineOrchestrator` still hand-built script-shaped planned job JSON inline.
   - Impact: duplicate schema defaults in `runtime.rs`; drift risk vs typed `pipeline::execution` helpers during migration off `image-lab.mjs`.

### Fixes Implemented

1. Added typed planned run-log job record builder in `pipeline::execution`:
   - `build_planned_run_log_job_record(...)`
   - script-parity defaults for planned generation/postprocess/output-guard fields centralized in Rust
2. `RustDryRunPipelineOrchestrator` now maps planned jobs through the typed builder instead of inline `serde_json::json!` object shaping.
3. Added regression coverage:
   - `pipeline::execution` unit test for planned-job default fields
   - `pipeline::runtime` dry-run test reads generated run log and asserts planned job JSON fields (`planned_generation`, `planned_postprocess.pipeline_order`, `planned_output_guard`)

### Validation (latest patch)

1. `cargo fmt`
2. `cargo test pipeline::execution --lib`
3. `cargo test pipeline::runtime --lib`

Result: passing.

## Next Phase Started (2026-02-23, typed dry-run log envelope extraction)

### Scope

1. `src-tauri/src/pipeline/execution.rs`
2. `src-tauri/src/pipeline/runtime.rs`

### What Landed

1. `pipeline::execution` now defines typed dry-run run-log envelope structs/builders:
   - `ExecutionPlannedRunLogRecord`
   - `ExecutionPlannedRunLogContext`
   - `build_planned_run_log_record(...)`
   - shared defaults for planned postprocess/output-guard fields
2. `pipeline::runtime::RustDryRunPipelineOrchestrator` now serializes the typed dry-run run-log record directly instead of hand-assembling the top-level JSON envelope.
3. Added unit coverage for the new top-level typed dry-run run-log builder.

### Validation

1. `cargo fmt`
2. `cargo test pipeline::execution --lib`
3. `cargo test pipeline::runtime --lib`

Result: passing.

## Latest Patch (2026-02-23, runtime/execution extraction)

### Scope Reviewed

1. `src-tauri/src/pipeline/runtime.rs` (Rust dry-run path + `--input` file discovery)
2. `src-tauri/src/pipeline/execution.rs` (new Rust run-mode execution foundations)
3. Script parity behavior in `scripts/image-lab.mjs` for file output naming / candidate winner ranking / generation directory layout

### Issues Discovered

1. Bug (fixed): Rust recursive image discovery for typed dry `--input` runs used filesystem iteration order.
   - Impact: nondeterministic job ordering across machines/filesystems; unstable run-log job order and migration parity checks.
2. Parity gap (fixed): Rust dry-run path only created `runs/` while the script generation path creates `outputs`, `runs`, and archive directories up front.
   - Impact: dry-run directory side effects diverged from script behavior and from the future Rust run-mode path.

### Fixes Implemented

1. `pipeline::runtime` now sorts recursive directory entries during Rust `--input` image discovery for deterministic planning order.
2. `pipeline::runtime` dry-run path now uses `pipeline::execution` project directory helpers (`execution_project_dirs`, `ensure_generation_mode_dirs`) instead of hardcoded path joins.
3. `pipeline::execution` extracted additional script-parity helpers:
   - generation directory creation (`outputs`, `runs`, `archive/*`)
   - sanitized output file path builder (port of `buildFileOutputPath`)
   - candidate winner selection ranking (port of `pickBestCandidate`)
   - stricter candidate filename contract (`total_candidates >= 1`)
4. Next-phase starter landed after this patch:
   - `pipeline::execution::plan_candidate_output_paths(...)` now ports script path planning for generate/bg-remove/upscale/color pass order and naming (typed + unit tested)
   - `pipeline::execution::plan_job_candidate_output_paths(...)` now ports the per-job candidate loop expansion into ordered typed plans
   - `pipeline::execution::summarize_output_guard_report(...)` now ports script output-guard ranking math (`summarizeGuardReport`) with rounding parity tests
   - `pipeline::execution::resolve_job_outcome_from_candidates(...)` now ports script winner/failure job outcome shaping (`selected_candidate`, `final_output`/`output`, failure reason) with strict missing-output validation
   - `pipeline::execution::finalize_job_from_candidates(...)` now ports script `jobMeta` final field shaping and failed-output-guard counter increment behavior
   - `pipeline::execution::build_run_log_candidate_record(...)` / `build_run_log_job_record(...)` now assemble typed run-log candidate/job entries from execution results (generated `output` preserved separately from `final_output`)
   - `pipeline::execution::build_run_log_output_guard_record(...)` now ports `candidateMeta.output_guard` log shaping (checked_input/summary/files/bad_archive)

### Validation (latest patch)

1. `cargo test pipeline::execution --lib`
2. `cargo test pipeline::runtime --lib`
3. `cargo test --test pipeline_trigger_endpoints --test http_contract_surface`
4. `cargo fmt`

Result: passing.

### Open Tasks / Recommended Next Steps (updated)

1. Use the new Rust `pipeline::execution` path-building/ranking helpers inside an actual Rust run-mode loop (not only dry-run support/foundations).
   - started: candidate output/post-process path planning is now available as a typed Rust helper
2. Port candidate generation/post-process orchestration from `scripts/image-lab.mjs` into Rust execution services:
   - generation call step
   - optional bg-remove/upscale/color pass ordering
   - output-guard evaluation + candidate ranking/winner selection
3. Replace script run-log writing for run-mode with `pipeline::runlog` and remove `image-lab.mjs` run-mode ownership.

## Security / Audit Foundation Update (2026-02-23)

### Completed

1. Added Rust DB schema support for `api_tokens` (hashed bearer tokens) and `audit_events` (spec-aligned target columns).
2. Added auth token APIs:
   - `POST /auth/token`
   - `GET /auth/tokens`
   - `DELETE /auth/tokens/{tokenId}`
3. Added Bearer auth middleware in the Rust router:
   - enforces auth on mutating endpoints and `/auth/*`
   - keeps local dev bypass via `KROMA_API_AUTH_DEV_BYPASS` (default enabled)
4. Added audit writes and `audit_id` response fields for mutating handlers in scope:
   - `provider_accounts`
   - `secrets`
   - `projects` (project/storage upserts)
   - `agent_instructions` create/confirm/cancel
5. Added legacy script safety gates (`KROMA_ENABLE_LEGACY_SCRIPTS=1`) to:
   - `scripts/backend.py`
   - `scripts/agent_worker.py`
   - `scripts/agent_dispatch.py`
6. Added `scripts/README.md` deprecation note pointing to Rust runtime as the only supported backend/runtime.

### Remaining (Important)

1. Full removal of `scripts/image-lab.mjs` from the HTTP pipeline path is still not complete.
   - Rust owns more execution/planning/log shaping logic, but run-mode generation/post-process orchestration still uses script fallback.
2. Audit writes for export mutations are not implemented because the Rust HTTP surface currently has no export mutation endpoints.
3. OpenAPI security docs are only partially annotated (auth routes + bearer scheme added; endpoint-level security metadata still needs full pass).

## Latest Review Pass (2026-02-23)

### Scope (newly added Rust migration code reviewed)

1. `src-tauri/src/db/projects/pipeline_ingest.rs` (native Rust run-log ingest)
2. `src-tauri/src/pipeline/backend_ops.rs` (native ingest + Rust S3 sync backend ops)
3. Endpoint/runtime regression surface after recent trigger/runtime consolidation commits

### Issues Discovered (this pass)

1. Bug (fixed): fallback `candidate_index` generation in native ingest used a run-global counter instead of a per-job counter.
   - Impact: jobs with missing `candidate_index` fields could get incorrect candidate numbering (e.g. second job starting at `2` instead of `1`).
2. Edge-case bug (fixed): Rust S3 sync precheck only validated `project_root.exists()`.
   - Impact: a file at the configured `project_root` path would incorrectly pass precheck and fail later in `aws s3 sync` with a less clear runtime error.

### Fixes Implemented (this pass)

1. `pipeline_ingest`: fallback `candidate_index` now resets per job (`enumerate()` within each job candidate list).
2. Added ingest regression test covering two jobs with missing candidate indexes (`candidate_index` must start at `1` for each job).
3. `backend_ops`: Rust S3 sync precheck now requires `project_root` to be a directory (`is_dir()`).
4. Added backend ops regression test asserting file-backed `project_root` fails precheck without invoking AWS CLI.

### Validation (this pass)

1. `cargo test pipeline_ingest --lib`
2. `cargo test pipeline::backend_ops --lib`
3. `cargo fmt`
4. `cargo test --test pipeline_trigger_endpoints --test http_contract_surface`

Result: passing.

### Follow-up Hardening (2026-02-23)

1. Added native Rust ingest re-ingest/idempotency regression coverage for the same `run_log_path`.
2. Test verifies second ingest replaces the prior run state cleanly (single run row, refreshed job/candidate state, refreshed quality/cost rows).
3. File: `src-tauri/src/db/projects/pipeline_ingest.rs`

### Review Follow-up (2026-02-23, planning/runtime preflight)

Scope reviewed:
1. `src-tauri/src/pipeline/planning.rs` (typed planning manifest parsing/defaults)
2. `src-tauri/src/pipeline/runtime.rs` (Rust planning preflight + parity warning integration)
3. `src-tauri/src/api/runs_assets.rs` / `src-tauri/src/pipeline/trigger.rs` fallout from new runtime error variants/options

Issues discovered (fixed):
1. Regression: `ScriptPipelineOrchestrator::execute` ran Rust manifest planning preflight before `build_command`.
   - Impact: invalid manifest payloads could mask earlier validation/infrastructure errors (e.g. invalid project slug) and change error precedence.
2. Typed parser issue: planning manifest `scene_refs` / `style_refs` silently dropped empty-string entries.
   - Impact: Rust preflight could pass malformed arrays that the script would later fail on, weakening migration parity and early validation.

Fixes implemented:
1. Restored error precedence by building/validating the command (`build_command`) before running Rust planning preflight.
2. Added runtime regression test asserting invalid project slug is returned before manifest preflight errors.
3. Tightened planning manifest parser to reject empty-string entries in string arrays (new explicit error).
4. Added planning parser regression test for empty array entries.

Validation (review follow-up):
1. `cargo test pipeline::planning --lib`
2. `cargo test pipeline::runtime --lib`
3. `cargo test --test pipeline_trigger_endpoints --test http_contract_surface`

Result: passing.

Remaining risks / TODO (review follow-up):
1. Rust preflight still duplicates manifest parsing/planning work already performed by `image-lab.mjs` (temporary migration cost).
2. Rust now covers scene expansion parity for typed dry `input` runs, but run-mode execution/post-process remains script-owned.
3. Next migration step remains to replace script-owned generation/post-process execution with Rust execution modules.
4. Rust now passes planned jobs via `--jobs-file` for script-backed runs, but `image-lab.mjs` still owns the run-mode execution loop and final run-log writing on that path.

Review follow-up (2026-02-23, latest runtime slice):
1. Fixed regression in Rust preflight activation guard: typed dry `input` path runs were still delegating to script because the guard skipped `InputPath`.
2. Fixed temp-file leak edge case in script-backed `--jobs-file` path when the second `build_command` fails after writing the temp file.
3. Validation:
   - `cargo test pipeline::runtime --lib`
   - `cargo test --test pipeline_trigger_endpoints --test http_contract_surface`
4. Result: passing.

### Current Status / Open Tasks (updated)

1. Typed trigger post-run backend ops are Rust-owned end-to-end (Rust ingest + Rust S3 sync execution).
2. Remaining major script dependency on the app trigger path is `scripts/image-lab.mjs` generation/orchestration itself.
3. Next phase started: Rust `pipeline::planning` module now ports `image-lab` prompt composition + generation job planning.
4. Follow-up landed: typed planning manifest defaults + JSON parser (`scene_refs`, `style_refs`, `policy.default_no_invention`, `prompts`) with tests for script-subset parity.
5. Follow-up landed: `pipeline::runtime` now supports typed `manifest_path` and runs Rust planning preflight (manifest parse + job planning) before script execution when a manifest is provided.
6. Follow-up landed: runtime now compares Rust planned job count vs script summary job count (when available) and emits a warning on mismatch (parity signal during migration).
7. Next phase starter landed: runtime preflight now retains Rust-planned job IDs (not only count) and includes IDs in parity mismatch warnings.
8. Follow-up landed: Rust runtime now writes planned jobs to a temp JSON file and passes `--jobs-file` to `image-lab.mjs` for manifest-backed scene-ref runs, bypassing script-side planning on the typed app path.
9. Next phase starter landed: new `pipeline::runlog` Rust module provides typed summary-marker formatting and pretty JSON run-log writing helpers (foundation for replacing script run-log output).
10. Major follow-up landed: typed dry runs with `scene_refs` now execute in Rust (planning + run-log writing + summary marker) via `RustDryRunPipelineOrchestrator` and no longer require Node/script execution on that path.
11. Review+follow-up landed: typed dry runs with `input` path (file/dir) now also execute in Rust using Rust image-file discovery (recursive image scan parity) and no longer delegate to the script path.
12. Next phase starter landed: `pipeline::execution` Rust module (run-mode execution foundation) with script-parity candidate output filename logic and tests.
13. Follow-up landed: `pipeline::execution` now owns script-parity project directory layout conventions (`outputs`, `runs`, `upscaled`, `color_corrected`, `background_removed`, `archive/*`) with tests.
14. Continue extracting generation/orchestration stages from `scripts/image-lab.mjs` into Rust modules and remove the script fallback (next: move candidate generation/post-process loop pieces into `pipeline::execution` and reuse `pipeline::runlog` for final log output).

### Recommended Next Steps

1. Start a Rust orchestration module for run planning + job expansion currently handled in `scripts/image-lab.mjs`.
   - Started: `src-tauri/src/pipeline/planning.rs` (prompt composition + generation job planning parity helpers/tests)
   - Extended: planning manifest default values + typed JSON/file parsing for the planning subset used by `image-lab`
   - Wired (preflight): `ScriptPipelineOrchestrator::execute` validates manifest/planning in Rust before invoking `image-lab.mjs` when `manifest_path` is set
2. Move script stdout/run-log ownership into Rust (structured in-memory result + Rust log writer), then delete script summary parsing fallback.
3. Add re-ingest/idempotency tests for native Rust ingest (especially changed candidate/output paths across repeated `run_log_path` imports).
   - Started: same `run_log_path` replacement coverage added in `pipeline_ingest` tests; still expand for multi-job/missing-path edge cases

## Architecture Direction (Non-Negotiable)

1. Kroma is a desktop app and the target architecture is Rust-owned end to end (`src-tauri`).
2. `scripts/` are transitional migration scaffolding only, not a supported long-term runtime.
3. Temporary Rust wrappers around script behavior are allowed only while replacing that behavior with Rust modules.
4. Every script-backed runtime path should have:
   - a Rust replacement milestone
   - a script deprecation/removal milestone
   - script deletion in the same phase once Rust parity lands (no dormant legacy script retention)
5. Phase 1 is not complete until core runtime/orchestration, worker flows, and active backend integrations are owned by Rust modules.

## Runtime Consolidation Update (Newest)

1. Native Rust run-log ingest is now implemented in `src-tauri/src/db/projects/pipeline_ingest.rs` (`ProjectsStore::ingest_run_log`).
2. Default typed `runs/trigger` post-run path now uses:
   - Rust post-run wrapper (`pipeline::runtime`)
   - Rust post-run service (`pipeline::post_run`)
   - native Rust ingest + Rust AWS CLI sync execution (via `pipeline::backend_ops` hybrid adapter)
3. `backend.py ingest-run` and `backend.py sync-project-s3` are no longer used on the default Rust `runs/trigger` post-run path.
4. Remaining runtime script dependency on that path is `scripts/image-lab.mjs` generation/orchestration itself (post-run backend ops are Rust-owned).
5. Script handoff hardening is in place:
   - `scripts/image-lab.mjs` emits `KROMA_PIPELINE_SUMMARY_JSON: {...}`
   - Rust parser prefers the structured marker and keeps text-line fallback during migration
6. `scripts/image-lab.mjs` no longer executes backend ingest/S3 sync directly (post-run backend calls removed from the script run path).
7. Typed `runs/trigger` now injects `project_root` from Rust project storage when omitted, reducing dependency on script-side backend storage resolution for the app path.

## What Landed (Latest Relevant Backend Work)

Latest backend commit on `master`: `a620df7`  
Latest docs/handoff commit on `master`: `9f972e0`  
Commit messages:
- `fix(trigger): validate semantic errors before project lookup`
- `refactor(trigger): remove redundant validation clones`

### Implemented

1. Trigger validation regression fix (new):
   - extracted shared trigger semantic validation (`validate_trigger_input`) in `src-tauri/src/pipeline/trigger.rs`
   - handler now runs semantic validation before project lookup in `src-tauri/src/api/runs_assets.rs`
   - restores consistent `400` validation errors before `404` missing-project for invalid typed payloads
2. Trigger validation hardening (recent commits):
   - `ac769cd`: reject stage-incompatible `time`/`weather` typed filters
   - `e50bdf6`: reject empty typed list inputs (`scene_refs`, `style_refs`)
   - `d62c556`: tighten `runs/trigger` OpenAPI schema and response docs
3. Bootstrap endpoint OpenAPI response docs improved:
   - documented `GET /api/projects/{slug}/bootstrap-prompt` response body shape
   - documented `POST /api/projects/{slug}/bootstrap-import` response body shape (`200` / `400` / `404`)
4. Roadmap/handoff docs refreshed for recent Phase 1 progress (`6df48f7`, `4c283af`)
5. Voice legacy feature surface removed from Rust backend (previous commit `4c283af`):
   - deleted `src-tauri/src/api/voice.rs`
   - removed `/voice/*` routes from route catalog/router/OpenAPI parity expectations
   - deleted `src-tauri/tests/voice_endpoints.rs`
6. Secrets DB code split out of the old mixed voice/secrets module:
   - replaced `src-tauri/src/db/projects/voice_secrets.rs` with `src-tauri/src/db/projects/secrets.rs`
   - `projects.rs` schema setup now calls `ensure_secret_tables` / `ensure_secret_columns`
7. Legacy Python HTTP entrypoint removed:
   - deleted `scripts/backend_api.py`
   - removed `npm run backend:api` from `package.json`
8. Scope/docs/script cleanup:
   - removed voice schema remnants from `scripts/backend.py`
   - removed voice flows from `scripts/contract-smoke.sh` and `scripts/contract_smoke.py`
   - updated `README.md` and `docs/Kroma_—_Project_Spec_(Current_State_&_Roadmap).md` to reflect current scope
9. Prior commit (still relevant to current architecture): `6df48f7`
   - `runs/trigger` refactor is typed end-to-end and no longer accepts raw Rust-side `extra_args`

## Local Work In Progress (Uncommitted)

1. Next-phase runtime-consolidation kickoff (current local change)
   - added `src-tauri/src/pipeline/backend_ops.rs` typed Rust boundary for script-backed `backend.py` operations
   - covers `ingest-run` and `sync-project-s3` command building/execution shell with unit tests
2. `src-tauri/src/pipeline/mod.rs`
   - exports new `backend_ops` module
3. Unrelated local file change remains outside backend work:
   - `logo.png` modified (not part of backend/runtime work)

## Code Analysis (This Pass)

Scope reviewed:
1. Latest `runs/trigger` validation hardening in `src-tauri/src/pipeline/trigger.rs`
2. HTTP trigger handler validation/ordering in `src-tauri/src/api/runs_assets.rs`
3. Trigger endpoint coverage in `src-tauri/tests/pipeline_trigger_endpoints.rs`
4. Contract/mount parity after recent trigger/OpenAPI changes
5. Newly extracted shared trigger validation path for duplication/perf issues
6. Next-phase `backend.py` dependency surface in `scripts/image-lab.mjs` / `scripts/backend.py` (`ingest-run`, `sync-project-s3`)

Issues discovered:
1. Bug (fixed): `reference_sets` import accepted entries without an `items` field.
   - In merge mode, this could silently delete all items for a provided reference set because per-set item application is authoritative.
2. Regression (fixed): stage-aware trigger semantic validation ran only inside `PipelineTriggerService` after project lookup.
   - Invalid typed requests (e.g. `time` without compatible `stage`) could return `404 Project not found` before the expected `400` validation error.
3. Performance/maintenance issue (fixed): `PipelineTriggerService::trigger` reconstructed/cloned all `TriggerRunParams` fields just to call shared validation.
   - This added avoidable allocations and created a field-sync maintenance hazard for future trigger params.

Fixes implemented:
1. Added validation requiring `reference_sets[].items` to be explicitly present (use `[]` for an empty set).
2. Added bootstrap validation test coverage for the missing `reference_sets[].items` case.
3. Clarified bootstrap prompt rules and OpenAPI docs for `reference_sets` item-array semantics.
4. Extracted shared trigger semantic validation helper and reused it in both the HTTP handler and trigger service.
5. Restored pre-lookup validation precedence for stage-aware trigger semantic errors.
6. Updated endpoint tests to assert validation errors are returned before missing-project lookup for stage-aware invalid payloads.
7. Removed redundant param cloning by validating `TriggerRunParams` by reference before destructuring in the trigger service.
8. Started a typed Rust boundary (`pipeline::backend_ops`) for script-backed backend ingest/S3 sync operations.

Remaining risks / TODO:
1. `reference_sets` nested item behavior is authoritative per provided set (not per-item merge).
   - This is currently documented, but payloads do not yet include stable item IDs for fine-grained merge semantics.
2. Bootstrap `reference_sets` export item loading was optimized (batched query), but larger-dataset profiling is still TODO.
3. `scripts/backend.py` still exists and remains an active dependency of `scripts/image-lab.mjs`.
   - Do not delete it until a Rust replacement path is wired for the needed pipeline operations.
4. `runs/trigger` remains script-backed at execution time.
   - HTTP/API and service contracts are now typed; runtime internals still need script orchestration replacement.

## Validation Status

From the feature pass that produced `868b71d` (per prior handoff):
1. `cargo fmt`
2. `cargo test --test bootstrap_endpoints`
3. `cargo test --test bootstrap_endpoints --test contract_parity --test http_contract_surface`
4. `cargo test`

Reported result: all passing.

Local validation run for the uncommitted bootstrap extensions:
1. `cargo fmt`
2. `cargo test --test bootstrap_endpoints`
3. `cargo test --test contract_parity --test http_contract_surface`
4. `cargo test`

Result: passing.

Local validation run for next-phase runtime consolidation kickoff:
1. `cargo fmt`
2. `cargo test pipeline::runtime --lib`
3. `cargo test pipeline::trigger --lib`
4. `cargo test --test http_contract_surface`
5. `cargo test --test pipeline_trigger_endpoints`
6. `cargo test --test contract_parity --test http_contract_surface`

Result: passing.

Local validation run for voice/legacy-scope cleanup:
1. `cargo fmt`
2. `cargo test`
3. `python3 -m py_compile scripts/backend.py scripts/contract_smoke.py`
4. `bash -n scripts/contract-smoke.sh`

Result: passing.

Local validation run for bootstrap export `reference_sets` loader optimization:
1. `cargo fmt`
2. `cargo test --test bootstrap_endpoints`

Result: passing.

Local validation run for trigger semantic-validation regression fix:
1. `cargo test pipeline::trigger --lib`
2. `cargo test --test pipeline_trigger_endpoints`
3. `cargo test --test contract_parity --test http_contract_surface`

Result: passing.

Local validation run for trigger validation clone-removal follow-up:
1. `cargo test pipeline::trigger --lib`
2. `cargo test --test pipeline_trigger_endpoints`

Result: passing.

Local validation run for next-phase `pipeline::backend_ops` boundary kickoff:
1. `cargo fmt`
2. `cargo test pipeline::backend_ops --lib`

Result: passing.

Local validation run for runtime post-run ingest wiring (Rust-owned wrapper around script fallback):
1. `cargo fmt`
2. `cargo test pipeline::runtime --lib`
3. `cargo test pipeline::trigger --lib`
4. `cargo test pipeline::post_run --lib`
5. `cargo test --test pipeline_trigger_endpoints`
6. `cargo test --test contract_parity --test http_contract_surface`

Result: passing.

## Runtime Consolidation Update (Newest)

1. `pipeline::backend_ops` and `pipeline::post_run` are now wired into the default Rust pipeline runtime path for typed `runs/trigger`.
2. The default runtime now wraps `ScriptPipelineOrchestrator` with a Rust post-run wrapper:
   - disables script-side backend ingest (`--backend-db-ingest false`) to avoid duplicate ingest
   - keeps script-side S3 sync disabled (`--storage-sync-s3 false`) until Rust owns sync policy/options
   - runs Rust `PipelinePostRunService` ingest after successful script execution (best-effort; warning on failure)
3. Current transitional risk:
   - Rust post-run wrapper extracts `run_log_path` from the script stdout line `Run log: ...`
   - this is intentionally temporary and should be replaced by structured handoff (JSON stdout/metadata) or full Rust orchestration ownership

## Next Priority Work

1. Continue Phase 1 runtime consolidation (Rust app unification):
   - Rust pipeline orchestration replacement for `scripts/image-lab.mjs`
   - Replace `backend.py`-dependent pipeline operations with Rust modules behind the existing runtime boundary (in progress: `pipeline::backend_ops` + `pipeline::post_run`; backend ingest now routed through Rust wrapper on typed trigger path)
   - Rust worker/dispatcher replacement for script workers
   - typed Rust tool adapters for external tools/APIs
2. Decide whether `chat` / `agent instructions` belong in bootstrap scope and define rules before implementation.
3. Add desktop UI bootstrap flow:
   - export prompt action
   - paste/import modal
   - `dry_run` preview/diff confirmation
   - explicit merge vs replace UX
4. Expand bootstrap/OpenAPI nested response schemas/examples if client generation needs stricter typing.
5. Keep `docs/ROADMAP.md` and `NEXT_CHAT_HANDOFF.md` aligned when Phase 1 milestones or priorities shift.

## Suggested Starting Point For Next Chat

1. Wire `pipeline::backend_ops` into the Rust runtime/orchestration path as the boundary for backend ingest + S3 sync post-run operations (still script-backed initially).
   - Status: done for backend ingest on the default typed trigger path (best-effort Rust post-run wrapper)
2. Replace stdout `Run log:` parsing with structured handoff from the script runtime (JSON metadata output), or move run-log creation + post-run orchestration fully into Rust.
3. Extend Rust-owned post-run path to S3 sync once typed trigger/runtime policy/options are defined (keep sync disabled in script while migrating).
4. Use `docs/ROADMAP.md` as the first status check, then update `NEXT_CHAT_HANDOFF.md` after milestone-level changes.
5. Keep `scripts/` callable only behind explicit Rust interfaces/migration boundaries.
