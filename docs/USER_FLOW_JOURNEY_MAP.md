# Kroma User Flow and Journey Map (Canonical)

Last updated: 2026-02-27
Status: Active implementation contract

## Why this file exists

Kroma implements a project-first comic/graphic-novel production flow.
If a feature is not mapped to a journey step in this file, it is out of scope.

## Product Intent

1. Main mode: build long-form project universes with stable style and stable faces.
2. Secondary mode: quick utility actions without project setup.
3. Future mode: continuity-preserving video, built on project identity foundations.

## Personas

1. Solo Artist (primary): creates one or more long-running story projects.
2. Studio Operator (secondary): manages many projects and repeatable production runs.
3. Utility User (secondary): uses fast one-off tools (for example background removal).

## Primary Journey: Project-First Comic Creation

### `J00` Onboarding and Provider Setup

User outcome:
- App is ready to generate.

Backend dependencies:
- provider accounts CRUD
- project secrets CRUD (encrypted at rest)
- auth token bootstrap (if auth enabled)

Done criteria:
- user can save provider config and pass provider health check

### `J01` Create or Select Project Universe

User outcome:
- project exists as isolated continuity boundary.

Backend dependencies:
- projects CRUD
- project storage configuration APIs

Done criteria:
- project appears in list and has storage root

### `J02` Build Continuity References

User outcome:
- style, scene, and character anchors are ready.

Backend dependencies:
- assets APIs
- reference sets/items CRUD
- characters CRUD
- style guides CRUD

Done criteria:
- required reference sets are populated and visible

### `J03` Bootstrap Story Settings (Optional Accelerator)

User outcome:
- imported project settings from AI-generated structured response.

Backend dependencies:
- `GET /api/projects/{slug}/bootstrap-prompt`
- `POST /api/projects/{slug}/bootstrap-import`

Done criteria:
- imported entities visible in project settings and defaults

### `J04` Lock Style Baseline

User outcome:
- approved style baseline for downstream stages.

Backend dependencies:
- run trigger endpoint
- run logs and candidate persistence
- run review data APIs

Done criteria:
- one baseline winner approved per scene (or explicit retry recorded)

### `J05` Controlled Variation (Time and Weather)

User outcome:
- same composition/style preserved under controlled changes.

Backend dependencies:
- staged run semantics
- candidate ranking
- asset lineage links

Done criteria:
- drift remains within accepted threshold or run marked for retry

### `J06` Character Identity Stage

User outcome:
- recurring heroes keep stable facial and identity traits.

Backend dependencies:
- character reference linkage
- quality report storage
- retry workflow APIs

Done criteria:
- identity checks pass for approved outputs

### `J07` Local Post-Process Chain

User outcome:
- production-ready assets (bg removal/upscale/color).

Backend dependencies:
- post-process orchestration
- derived asset links
- QA guard + archive behavior

Done criteria:
- derived assets are linked and traceable to source run outputs

### `J08` Review, Curate, and Export

User outcome:
- chapter/page-ready asset package with reproducibility metadata.

Backend dependencies:
- runs/assets read APIs
- exports APIs
- checksums/manifest metadata

Done criteria:
- export record exists and package is reproducible from metadata

## Secondary Journey: Utility Mode

### `U01` Quick One-Off Operation

User outcome:
- run utility task quickly without full project setup.

Scope:
- background removal
- upscaling
- one-off generation

Guardrails:
- utility mode must not bypass project data isolation for project-scoped records
- utility mode must not consume roadmap priority over `J00-J08`

## Failure and Recovery Journey

### `R01` Failed Run Recovery

User outcome:
- user can identify failure reason and retry safely.

Required behavior:
- stable error taxonomy
- retry guidance
- preserved audit/run log trail

### `R02` Provider/Credential Recovery

User outcome:
- user restores generation ability without data loss.

Required behavior:
- clear provider error surfacing
- credential update flow
- no plaintext secret exposure

## Implementation Gate (Mandatory)

Every new backend/frontend feature must include:

1. Journey step ID (`Jxx`, `Uxx`, or `Rxx`) in spec/PR notes.
2. Acceptance criteria tied to that step.
3. Test evidence for that step (unit/integration/e2e as applicable).

If no journey mapping exists, do not implement the feature yet.

## Frontend Start Rule

Frontend implementation begins from `J00 -> J08` in order.
UI work may start only after backend contracts for each targeted step are stable.
