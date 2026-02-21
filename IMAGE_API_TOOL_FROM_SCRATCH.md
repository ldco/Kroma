# Image API Tool From Scratch

Last update: 2026-02-19  
Purpose: blueprint for a separate project that generates images via API with predictable quality, cost control, and reproducible runs.

## 1. Scope

Build a standalone tool that:

- takes one or more input images;
- applies prompt-driven transformations through an image API;
- runs in explicit stages (style -> time/light -> weather/effects);
- stores outputs and machine-readable run logs;
- enforces safety gates before paid calls.

This document captures practical lessons learned from the current Geograf graphics pipeline so you can reboot cleanly without repeating expensive mistakes.

## 2. Tech Stack (Recommended)

- Runtime: Node.js 20+
- API: OpenAI Images Edits endpoint (`/v1/images/edits`)
- Config: JSON manifests in `settings/`
- Output + logs: filesystem (`generated/`)
- Optional QA helper: Python 3 + Pillow for grayscale/chroma checks

## 3. Project Layout

Create a new repo with this baseline structure:

```text
image-api-tool/
├── .env.example
├── package.json
├── README.md
├── scripts/
│   └── image-lab.mjs
├── settings/
│   ├── manifest.json
│   └── presets.json
├── knowledge/
│   └── WORKFLOW.md
├── references/
│   └── .gitkeep
├── scenes/
│   └── .gitkeep
└── generated/
    ├── outputs/
    └── runs/
```

Rule: keep runner logic in `scripts/`, all prompts/paths/defaults in `settings/`.

## 4. Initialization

```bash
mkdir image-api-tool && cd image-api-tool
npm init -y
npm pkg set type=module
npm pkg set scripts.lab="node scripts/image-lab.mjs"
mkdir -p scripts settings knowledge references scenes generated/outputs generated/runs
```

## 5. Environment

Create `.env.example`:

```dotenv
OPENAI_API_KEY=your_key_here
OPENAI_IMAGE_MODEL=gpt-image-1
OPENAI_IMAGE_SIZE=1024x1536
OPENAI_IMAGE_QUALITY=high
```

Copy to `.env` and set real values.

## 6. Single Source of Truth Config

Create `settings/manifest.json`:

```json
{
  "style_refs": [],
  "scene_refs": [],
  "safe_batch_limit": 20,
  "output_guard": {
    "enforce_grayscale": false,
    "max_chroma_delta": 2.0,
    "fail_on_chroma_exceed": false
  },
  "prompts": {
    "style_base": "Preserve geometry and perspective. Apply one coherent noir drawing hand. No text, no logos, no watermark.",
    "time_day": "Daylight scene with clear readability and stable contrast family.",
    "time_night": "Night scene with controlled deep shadows, no topology changes.",
    "weather_clear": "Dry clear atmosphere. No rain streaks, no snow particles, no diagonal sky hatching.",
    "weather_rain": "Visible wet surfaces, puddles, and reflection cues; no style drift."
  }
}
```

Then add your own files into `references/` and `scenes/`, and populate:

- `style_refs` with your style anchors
- `scene_refs` with your scene inputs

Design rule:

- prompts are data;
- runner composes prompts;
- no hardcoded project text in script.

## 7. Minimal Runner Skeleton

Create `scripts/image-lab.mjs`:

```js
#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const manifestPath = path.join(root, "settings/manifest.json");
const dotenvPath = path.join(root, ".env");

function loadEnv() {
  if (!fs.existsSync(dotenvPath)) return;
  for (const raw of fs.readFileSync(dotenvPath, "utf8").split("\n")) {
    const line = raw.trim();
    if (!line || line.startsWith("#")) continue;
    const i = line.indexOf("=");
    if (i < 1) continue;
    const k = line.slice(0, i).trim();
    const v = line.slice(i + 1).trim().replace(/^['"]|['"]$/g, "");
    if (!process.env[k]) process.env[k] = v;
  }
}

function readArg(name, fallback = "") {
  const args = process.argv.slice(2);
  const i = args.indexOf(name);
  return i >= 0 ? String(args[i + 1] || fallback) : fallback;
}

function hasFlag(name) {
  return process.argv.slice(2).includes(name);
}

function loadManifest() {
  return JSON.parse(fs.readFileSync(manifestPath, "utf8"));
}

function mimeFor(fp) {
  const ext = path.extname(fp).toLowerCase();
  if (ext === ".png") return "image/png";
  if (ext === ".webp") return "image/webp";
  return "image/jpeg";
}

async function callImagesEdits({ apiKey, model, size, quality, prompt, inputImages }) {
  const form = new FormData();
  form.append("model", model);
  form.append("size", size);
  form.append("quality", quality);
  form.append("prompt", prompt);
  form.append("output_format", "png");
  form.append("input_fidelity", "high");
  for (const rel of inputImages) {
    const abs = path.join(root, rel);
    const buf = fs.readFileSync(abs);
    form.append("image[]", new Blob([buf], { type: mimeFor(abs) }), path.basename(abs));
  }
  const res = await fetch("https://api.openai.com/v1/images/edits", {
    method: "POST",
    headers: { Authorization: `Bearer ${apiKey}` },
    body: form
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}: ${await res.text()}`);
  const json = await res.json();
  return json?.data?.[0]?.b64_json;
}

async function main() {
  loadEnv();
  const mode = process.argv[2] || "dry";
  const confirmSpend = hasFlag("--confirm-spend");
  const dry = mode === "dry";
  const manifest = loadManifest();

  const model = readArg("--model", process.env.OPENAI_IMAGE_MODEL || "gpt-image-1");
  const size = readArg("--size", process.env.OPENAI_IMAGE_SIZE || "1024x1536");
  const quality = readArg("--quality", process.env.OPENAI_IMAGE_QUALITY || "high");

  const jobs = manifest.scene_refs.map((scene, i) => ({
    id: `style_${i + 1}`,
    mode: "style",
    input_images: [scene, ...manifest.style_refs],
    prompt: manifest.prompts.style_base
  }));

  if (!dry && !confirmSpend) throw new Error("Spending is locked. Add --confirm-spend.");
  if (!dry && jobs.length > (manifest.safe_batch_limit || 20)) throw new Error("Batch exceeds safety limit.");

  const stamp = new Date().toISOString().replace(/[:.]/g, "-");
  const runLogPath = path.join(root, "generated/runs", `run_${stamp}.json`);
  const runLog = [];
  fs.mkdirSync(path.join(root, "generated/outputs"), { recursive: true });
  fs.mkdirSync(path.join(root, "generated/runs"), { recursive: true });

  for (const job of jobs) {
    if (dry) {
      runLog.push({ ...job, status: "planned" });
      continue;
    }
    const b64 = await callImagesEdits({
      apiKey: process.env.OPENAI_API_KEY,
      model,
      size,
      quality,
      prompt: job.prompt,
      inputImages: job.input_images
    });
    const outPath = path.join(root, "generated/outputs", `${job.id}.png`);
    fs.writeFileSync(outPath, Buffer.from(b64, "base64"));
    runLog.push({ ...job, status: "done", output: path.relative(root, outPath) });
  }

  fs.writeFileSync(runLogPath, JSON.stringify(runLog, null, 2) + "\n");
  console.log(`Run log: ${path.relative(root, runLogPath)}`);
}

main().catch((e) => {
  console.error(e.message || String(e));
  process.exit(1);
});
```

## 8. Commands

```bash
# dry run: builds queue, no spend
npm run lab -- dry

# paid run: requires explicit spend confirmation
npm run lab -- run --confirm-spend
```

Add more modes as you grow (`style`, `time`, `weather`, `character_insert`, etc.).

## 9. Staged Workflow (Critical)

Do not tune everything at once.

1. Stage 1: Style lock  
Goal: one consistent artistic hand, geometry preserved.
2. Stage 2: Time/light lock  
Goal: day/night variation while style remains stable.
3. Stage 3: Weather/effects lock  
Goal: weather cues are explicit and never replace structural drawing language.
4. Stage 4+: Characters/extras/transport (optional)  
Goal: add content only after scene style controls are stable.

## 10. Lessons Learned (Hard Rules)

- Keep geometry lock explicit in every prompt.
- Separate structural hatching from weather marks.
- Never use location outputs as style anchors by default.
- Prevent style-anchor contamination with allowed roots policy.
- Add dry-run and `--confirm-spend` guard from day one.
- Enforce safe batch limits (default 20).
- Write run logs for every job; reproducibility saves money.
- Archive old outputs/logs between real runs to keep active folders clean.
- Default to "no invention": do not add new object categories unless explicitly requested.

## 11. QA Gate Checklist

Pass/fail each image before promoting it:

- source topology preserved (camera/perspective/object layout);
- style consistency across multiple scenes;
- requested time/weather is unambiguous;
- no accidental text/logos/watermarks;
- no border artifacts (white frame/perimeter halo);
- if grayscale project: chroma delta within threshold.

## 12. Cost and Safety Controls

Mandatory controls:

- `dry` mode for every new prompt pack;
- explicit `--confirm-spend` for real runs;
- queue cap + manual override flag;
- per-run manifest (`run_*.json`) and metadata;
- optional per-stage budget envelope.

Recommended policy:

- test on 1-3 scenes first;
- promote only winning prompt profiles to default;
- avoid large batches until QA is stable.

## 13. Scaling Plan

When MVP is stable, add:

- `presets.json` with reusable style/time/weather bundles;
- command families (`style-dry`, `style-run`, `weather-dry`, `weather-run`);
- policy checks (allowed style anchor directories);
- automatic grayscale enforcement + chroma QA;
- optional two-step insert pipeline for characters (clean insert -> style pass).

## 14. Definition of Done for v1

Your separate tool is v1-ready when all are true:

- one command can run dry and real modes;
- prompts and paths live in manifest config, not code;
- outputs and run logs are written for every job;
- spend protection and batch limits are enforced;
- staged workflow is documented and repeatable by another person.
