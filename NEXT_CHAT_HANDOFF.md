# Next Chat Handoff

Date: 2026-02-21
Branch: `master`
Last commit before this handoff update: `eb76b55`

## Current Architecture Decisions

1. Rust backend (`src-tauri`) remains the active implementation path.
2. Contract-first routing and parity checks are preserved as guardrails.
3. Implemented API domains now include:
- projects/storage
- runs/assets
- asset-links
- analytics
- exports
- prompt templates
- provider accounts
- style guides
- characters
- reference sets/items
- chat sessions/messages
- agent instructions/events/actions
4. Repository layer is still the single authority for validation and state transitions.
5. Integration tests exist for each implemented domain and match mounted contract behavior.

## Completed Work In This Pass

1. Implemented agent instruction routes:
- `GET /api/projects/{slug}/agent/instructions`
- `POST /api/projects/{slug}/agent/instructions`
- `GET /api/projects/{slug}/agent/instructions/{instructionId}`
- `GET /api/projects/{slug}/agent/instructions/{instructionId}/events`
- `POST /api/projects/{slug}/agent/instructions/{instructionId}/confirm`
- `POST /api/projects/{slug}/agent/instructions/{instructionId}/cancel`
2. Added typed repository models + inputs:
- `AgentInstructionSummary`
- `AgentInstructionEventSummary`
- `CreateAgentInstructionInput`
- `AgentInstructionActionInput`
3. Added schema support:
- `agent_instructions`
- `agent_instruction_events`
4. Added API module and route wiring:
- `src-tauri/src/api/agent_instructions.rs`
- dispatch updates in `src-tauri/src/api/server.rs`
5. Added integration tests:
- `src-tauri/tests/agent_instructions_endpoints.rs`
6. Updated contract-surface status expectations for agent-instruction routes.

## Major Refactors / Rewrites

1. Added explicit instruction state-transition rules in repository:
- `pending -> confirmed`
- `pending -> canceled`
- reject `confirm` on canceled
- reject `cancel` on confirmed
2. Added instruction event recording helper to keep event trails deterministic.
3. Kept handlers thin by centralizing transition/business rules in persistence layer.

## Key Issues Found

1. Agent-instruction routes were mounted but unimplemented.
2. No persistence schema existed for instructions/events.
3. Transition behavior was undefined; explicit rules and errors are now enforced.

## Remaining Technical Debt

1. `db/projects.rs` is very large and should be split into domain modules.
2. Candidate overlap remains (`run_job_candidates` and `run_candidates`).
3. Remaining unimplemented contract domains:
- voice (`stt`, `tts`, `voice request detail`)
- secrets (`list`, `upsert`, `delete`)

## Next Phase Goals (Immediate)

1. Implement voice request persistence + endpoints as baseline (stub processing, real persistence/status).
2. Implement secrets CRUD endpoints with safe response shape (no secret value echo).
3. Begin physical repository split by domain to improve maintainability.

## Validation Snapshot

1. `cargo fmt --all`
2. `cargo test`
3. `npm run backend:rust:test --silent`
4. Passing suites now include:
- contract parity
- HTTP contract-surface
- projects/storage
- runs/assets
- asset-links
- analytics
- exports
- prompt templates
- provider accounts
- style guides
- characters
- reference sets/items
- chat
- agent instructions
