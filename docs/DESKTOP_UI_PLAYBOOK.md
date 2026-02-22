# Kroma Desktop UI Playbook (No Codex Required)

Last updated: 2026-02-22
Status: Product/UX implementation guide

## 1. Why this UI exists

Raw GPT image prompting is not enough for long-form consistent work.

Main failure mode:
1. You get one nice image.
2. Next image drifts in style, composition, or character identity.
3. You start prompt tweaking manually and lose reproducibility.

Kroma desktop UI should solve this by forcing a structured production workflow:
1. lock style first,
2. then apply controlled variations,
3. then add character detail,
4. then post-process locally.

No Codex required. No prompt engineering marathon required.

## 2. Core UX rule

The app should not be "chat-first". It should be "pipeline-first".

Chat can exist as optional assistant UI, but the main path must be:
1. Form-driven setup
2. Repeatable run configurations
3. Visual review + accept/reject
4. Audit trail for every decision

## 3. Desktop screen map (exact)

1. `Onboarding`
2. `Projects`
3. `Project Dashboard`
4. `References`
5. `Style Guides`
6. `Prompt Templates`
7. `Run Composer`
8. `Run Review`
9. `Post-Process`
10. `Assets Library`
11. `Exports`
12. `Settings`

## 4. Exact user flow in UI

### Step 1: Onboarding

User action:
1. Open app.
2. Paste OpenAI API key.
3. Choose local project root directory.
4. Click `Save`.

App action:
1. Writes secret to encrypted project secret storage.
2. Validates key with lightweight health/test request.
3. Creates default local user/project context.

Success indicator:
1. Green status: `Provider connected`.
2. `Continue` button unlocked.

### Step 2: Create project

User action:
1. Click `New Project`.
2. Enter `Name` and `Slug`.
3. Confirm storage root.
4. Click `Create`.

App action:
1. Creates project record.
2. Creates project folders (`outputs`, `runs`, `archive`, etc.).
3. Initializes default project settings.

Success indicator:
1. Project appears in `Projects` list.
2. Dashboard shows zero-state cards (`Runs 0`, `Assets 0`).

### Step 3: Build reference packs

User action:
1. Open `References`.
2. Add style anchors to a `style` reference set.
3. Add scene anchors to `scene` reference set.
4. Add character references to `character` sets (one set per character).

App action:
1. Imports files into project asset registry.
2. Stores reference-to-asset links.
3. Validates missing/duplicate assets.

Success indicator:
1. Every planned stage shows `Ready`.
2. UI shows reference counts per set.

### Step 3.5: Bootstrap settings via AI (new project accelerator)

Goal:
1. Generate a project-specific prompt that the user can send to any AI tool.
2. Import the AI response back into Kroma as structured settings.

User action:
1. Open project dashboard.
2. Click `Export Bootstrap Prompt`.
3. Copy prompt and paste into external AI tool.
4. Copy AI JSON output.
5. Click `Import Bootstrap Settings` and paste AI response.
6. Choose mode:
   - `merge`: update/add only provided sections.
   - `replace`: replace only the sections included in payload; omitted sections stay unchanged.
7. Confirm import.

App action:
1. Calls `GET /api/projects/{slug}/bootstrap-prompt`.
2. Shows returned `bootstrap.prompt` text to the user.
3. Accepts raw AI output text (including fenced JSON).
4. Calls `POST /api/projects/{slug}/bootstrap-import` with:
   - `ai_response_text` for raw paste, or
   - `settings` for already-parsed JSON.
5. Refreshes project settings panels using returned snapshot.

Success indicator:
1. Toast: `Bootstrap settings imported`.
2. Style guides/provider accounts/prompt templates update immediately.
3. Run Composer defaults reflect the imported templates/style guide choices.

### Step 4: Define style lock

User action:
1. Open `Style Guides`.
2. Create a guide with explicit style rules:
   - line quality
   - palette
   - texture/brush handling
   - forbidden artifacts
3. Save as default style guide.

App action:
1. Persists style guide JSON/rules.
2. Connects guide to project run composer defaults.

Success indicator:
1. `Default style guide` badge visible.

### Step 5: Run style stage (only style)

User action:
1. Open `Run Composer`.
2. Set Stage = `style`.
3. Set Candidates = `3` (or chosen value).
4. Select scene refs + style refs.
5. Click `Run`.

App action:
1. Executes style-only run.
2. Stores all candidates and ranking metadata.
3. Applies QA guard and archives hard failures.

Success indicator:
1. Run appears as `Completed`.
2. `Run Review` shows candidate grid with rank reasons.

### Step 6: Review and lock winners

User action:
1. Open `Run Review`.
2. Compare candidates side by side.
3. Approve one candidate per scene.
4. Click `Lock as baseline`.

App action:
1. Stores selected candidate indices.
2. Creates baseline links (`derived_from`/`reference_of`).
3. Freezes these assets as next-stage inputs.

Success indicator:
1. Baseline badge on selected assets.
2. Next-stage wizard enabled.

### Step 7: Controlled variation stages

User action:
1. New run, Stage = `time` (or `weather`).
2. Keep style guide and baseline fixed.
3. Run and review.
4. Repeat for other controlled stage.

App action:
1. Reuses locked references and templates.
2. Tracks drift metrics between stages.

Success indicator:
1. Drift indicator remains within threshold.
2. Approved chain visible in asset lineage graph.

### Step 8: Character stage

User action:
1. Select character reference set(s).
2. Stage = `character`.
3. Run and review identity stability.

App action:
1. Uses character refs + baseline style refs.
2. Scores outputs for identity consistency signals.

Success indicator:
1. Character pass rate shown per scene.
2. Low-quality outputs auto-flagged for retry.

### Step 9: Post-process locally

User action:
1. Open `Post-Process`.
2. Toggle needed steps:
   - background removal (`rembg`)
   - upscaling (Real-ESRGAN)
   - color profile
3. Click `Process`.

App action:
1. Runs tools locally in fixed order.
2. Writes outputs + metadata.
3. Links derived assets to source assets.

Success indicator:
1. Post-process job status `Done`.
2. New derived assets visible in library.

### Step 10: Export

User action:
1. Open `Exports`.
2. Select run(s) / approved assets.
3. Click `Export`.

App action:
1. Builds package with manifest + checksums.
2. Stores export record.

Success indicator:
1. Download/open folder action available.
2. Export appears with checksum and timestamp.

## 5. How this UI improves style consistency without Codex

1. Stage gating
- User cannot skip directly to "everything at once".
- This prevents uncontrolled prompt drift.

2. Locked references
- Style and baseline assets are pinned between stages.
- Variation stages reuse the same anchors.

3. Template-driven prompts
- Prompt templates are edited once and versioned.
- Runs store snapshots, so successful runs are repeatable.

4. Multi-candidate + objective ranking
- UI promotes best candidate using quality scores.
- Human still approves final selection.

5. Local post-process standardization
- Upscaling/background/color are deterministic local tools.
- This reduces random output differences.

## 6. Required controls in each run form

Minimum controls:
1. Stage selector (`style`, `time`, `weather`, `character`)
2. Candidate count
3. Style guide selector
4. Reference set selectors
5. Prompt template selector
6. Cost estimate and explicit confirmation
7. Post-process toggles

If any of these are missing, the app falls back to ad-hoc prompting and consistency drops.

## 7. Non-negotiable UX safeguards

1. Never hide which references were used.
2. Never overwrite approved baseline without explicit confirmation.
3. Never run paid generation without a visible cost confirmation step.
4. Always show lineage (`source -> derived`) for every output.
5. Always allow rerun from saved config snapshot.

## 8. What "good final result" means in this UI

A good result is not "one pretty image". It is:
1. consistent look across a set,
2. reproducible from saved run config,
3. traceable through stages and assets,
4. acceptable quality after local post-processing.

That is exactly what this desktop UI is designed to enforce.
