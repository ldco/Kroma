# Next Chat Handoff

Date: 2026-02-21
Branch: `master`
Last commit before this handoff update: `5929cce`

## Current Architecture Decisions

1. Rust backend (`src-tauri`) is the primary implementation and test path.
2. Contract-first routing and parity tests remain mandatory guardrails.
3. Implemented domains now include:
- projects/storage
- runs/assets
- asset-links
- analytics
- exports
- prompt templates
- provider accounts
- style guides
- characters
- reference sets + items
- chat sessions + messages
4. Repository layer continues to own validation, normalization, and persistence contracts.
5. Each implemented route group has dedicated integration tests and updated contract-surface expectations.

## Completed Work In This Pass

1. Implemented chat session/message routes:
- `GET /api/projects/{slug}/chat/sessions`
- `POST /api/projects/{slug}/chat/sessions`
- `GET /api/projects/{slug}/chat/sessions/{sessionId}`
- `GET /api/projects/{slug}/chat/sessions/{sessionId}/messages`
- `POST /api/projects/{slug}/chat/sessions/{sessionId}/messages`
2. Added typed repository models + inputs:
- `ChatSessionSummary`
- `ChatMessageSummary`
- `CreateChatSessionInput`
- `CreateChatMessageInput`
3. Added schema support:
- `chat_sessions`
- `chat_messages`
4. Added API module + route wiring:
- `src-tauri/src/api/chat.rs`
- router dispatch updates in `src-tauri/src/api/server.rs`
5. Added integration tests:
- `src-tauri/tests/chat_endpoints.rs`
6. Updated HTTP contract-surface status expectations for chat routes.
7. Stabilized chat message ordering by using insertion order (`rowid`) for message listing.

## Major Refactors / Rewrites

1. Maintained nested-resource pattern for chat while keeping handler logic thin.
2. Added strict role validation (`user|assistant|system|tool`) and required content checks.
3. Preserved deterministic not-found semantics for missing project/session paths.

## Key Issues Found

1. Chat routes were contract-mounted but unimplemented.
2. No chat schema existed prior to this pass.
3. Created-at second resolution caused unstable message ordering in tests; resolved with insertion-order query.

## Remaining Technical Debt

1. `db/projects.rs` is large and needs physical decomposition by domain.
2. Candidate table overlap remains (`run_job_candidates` and `run_candidates`).
3. Remaining unimplemented contract domains:
- agent instructions
- voice
- secrets

## Next Phase Goals (Immediate)

1. Implement agent-instructions list/create routes.
2. Implement voice route set baseline.
3. Begin module split for repository domains to reduce coupling.

## Validation Snapshot

1. `cargo fmt --all`
2. `cargo test`
3. `npm run backend:rust:test --silent`
4. Passing suites include:
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
