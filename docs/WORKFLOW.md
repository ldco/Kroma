# Workflow

See `docs/BACKEND_ARCHITECTURE_FREEZE.md` for the backend-first gate and frontend start criteria.
See `docs/USER_FLOW_JOURNEY_MAP.md` for canonical user journey steps and acceptance criteria.
See `docs/BACKEND_CONTRACT_FREEZE.md` for Step B contract/error taxonomy baseline.

## Journey Mapping Rule (Mandatory)

1. Every implementation task must map to a journey step ID.
2. Step type mapping:
   - `Jxx` for primary project flow
   - `Uxx` for secondary utility flow
   - `Rxx` for recovery/error flow
3. If there is no mapped journey step, do not implement the task yet.
4. PR/commit notes must include journey step ID(s), user outcome, and acceptance test evidence.

## Project Rule

- Always run with `--project <name>`.
- Every project writes only to `<project_root>/...`.
- Replaced outputs are auto-archived to `<project_root>/archive/replaced/`.
- Rejected outputs should be moved with `archive-bad` into `<project_root>/archive/bad/`.
- Run history is auto-ingested into backend DB (`var/backend/app.db`) unless disabled.

## Backend Bootstrap

Recommended (Rust primary path):

1. Start Rust backend API
- `npm run backend:rust`

2. Create project record
- `curl -s -X POST http://127.0.0.1:8788/api/projects -H 'Content-Type: application/json' -d '{"name":"my_project","slug":"my_project"}'`

3. Configure local storage
- `curl -s -X PUT http://127.0.0.1:8788/api/projects/my_project/storage/local -H 'Content-Type: application/json' -d '{"project_root":"/data/iat/my_project"}'`

4. Optional S3 storage config
- `curl -s -X PUT http://127.0.0.1:8788/api/projects/my_project/storage/s3 -H 'Content-Type: application/json' -d '{"enabled":true,"bucket":"my-art-bucket","prefix":"iat-prod","region":"us-east-1"}'`

Transitional/legacy scripts (still available):

- `npm run backend:init`
- direct `python3 scripts/backend.py ...` commands

## Staged Process

1. Style lock
- `npm run lab -- dry --project my_project --stage style`
- Optional: add `--candidates 3` to plan multi-candidate run.

2. Time/light lock
- `npm run lab -- dry --project my_project --stage time --time day`

3. Weather/effects lock
- `npm run lab -- dry --project my_project --stage weather --time day --weather rain`

4. Optional post chain
- Execution order is fixed: `bg_remove(rembg) -> bg_refine_openai -> upscale -> color`
- `npm run lab -- run --project my_project --confirm-spend --post-bg-remove --post-upscale --upscale-backend python --post-color --post-color-profile cinematic_warm`
- Multi-candidate production run: `npm run lab -- run --project my_project --confirm-spend --candidates 4`

Implementation note:
- the staged process maps directly to `J04 -> J07` in `docs/USER_FLOW_JOURNEY_MAP.md`

## Utility Commands

- Upscale only:
`npm run upscale -- --project my_project --input /data/iat/my_project/outputs --output /data/iat/my_project/upscaled --upscale-backend python --upscale-scale 2`

- Color only:
`npm run color -- --project my_project --input /data/iat/my_project/upscaled --output /data/iat/my_project/color_corrected --profile cinematic_warm`

- Background remove only:
`npm run bgremove -- --project my_project --input /data/iat/my_project/outputs --output /data/iat/my_project/background_removed --bg-remove-backends rembg --bg-refine-openai true`

- Archive bad files:
`npm run archivebad -- --project my_project --input /data/iat/my_project/background_removed`

- QA only (no generation):
`npm run qa -- --project my_project --input /data/iat/my_project/background_removed`
