# Workflow

## Project Rule

- Always run with `--project <name>`.
- Every project writes only to `generated/projects/<name>/...`.
- Replaced outputs are auto-archived to `generated/projects/<name>/archive/replaced/`.
- Rejected outputs should be moved with `archive-bad` into `generated/projects/<name>/archive/bad/`.
- Run history is auto-ingested into backend DB (`generated/backend/app.db`) unless disabled.

## Backend Bootstrap

1. Initialize DB and default user
- `npm run backend:init`

2. Create project record once
- `python3 scripts/backend.py create-project --name "my_project" --slug my_project`

2.1 Configure project local storage (optional)
- `python3 scripts/backend.py set-project-storage-local --project-slug my_project --project-root /data/iat/my_project`

2.2 Configure project S3 storage (optional)
- `python3 scripts/backend.py set-project-storage-s3 --project-slug my_project --enabled true --bucket my-art-bucket --prefix iat-prod --region us-east-1`

3. Export only this project when needed
- `python3 scripts/backend.py export-project --project-slug my_project --output generated/exports/my_project.tar.gz`

4. Start backend API for future GUI integration
- `npm run backend:api`

5. Run agent instruction worker (queue execution)
- `npm run backend:worker`

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

## Utility Commands

- Upscale only:
`npm run upscale -- --project my_project --input generated/projects/my_project/outputs --output generated/projects/my_project/upscaled --upscale-backend python --upscale-scale 2`

- Color only:
`npm run color -- --project my_project --input generated/projects/my_project/upscaled --output generated/projects/my_project/color_corrected --profile cinematic_warm`

- Background remove only:
`npm run bgremove -- --project my_project --input generated/projects/my_project/outputs --output generated/projects/my_project/background_removed --bg-remove-backends rembg --bg-refine-openai true`

- Archive bad files:
`npm run archivebad -- --project my_project --input generated/projects/my_project/background_removed`

- QA only (no generation):
`npm run qa -- --project my_project --input generated/projects/my_project/background_removed`
