#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";

const root = process.cwd();
const args = process.argv.slice(2);
const IMAGE_EXTS = new Set([".jpg", ".jpeg", ".png", ".webp", ".bmp", ".tif", ".tiff"]);

function readArg(name, fallback = "") {
  const idx = args.indexOf(name);
  return idx >= 0 ? String(args[idx + 1] || fallback) : fallback;
}

function hasFlag(name) {
  return args.includes(name);
}

function isImagePath(filePath) {
  return IMAGE_EXTS.has(path.extname(filePath).toLowerCase());
}

function listImageFiles(inputAbs) {
  const stat = fs.statSync(inputAbs);
  if (stat.isFile()) {
    return isImagePath(inputAbs) ? [inputAbs] : [];
  }

  const out = [];
  for (const entry of fs.readdirSync(inputAbs, { withFileTypes: true })) {
    const abs = path.join(inputAbs, entry.name);
    if (entry.isDirectory()) {
      out.push(...listImageFiles(abs));
      continue;
    }
    if (entry.isFile() && isImagePath(abs)) out.push(abs);
  }
  return out;
}

function parseBoolArg(name, fallback = false) {
  const idx = args.indexOf(name);
  if (idx < 0) return fallback;
  const rawValue = args[idx + 1];
  if (!rawValue || rawValue.startsWith("--")) return true;
  const raw = String(rawValue);
  if (["1", "true", "yes", "on"].includes(raw.toLowerCase())) return true;
  if (["0", "false", "no", "off"].includes(raw.toLowerCase())) return false;
  throw new Error(`Invalid boolean for ${name}: '${raw}'`);
}

function parseCsvArg(name, fallbackList) {
  const raw = readArg(name, "");
  if (!raw) return fallbackList;
  return raw
    .split(",")
    .map((x) => x.trim().toLowerCase())
    .filter(Boolean);
}

function resolvePath(inputPath) {
  if (!inputPath) return "";
  return path.isAbsolute(inputPath) ? inputPath : path.resolve(root, inputPath);
}

function toRel(absPath) {
  const abs = String(absPath || "");
  const rel = normalizeRel(path.relative(root, abs));
  if (!rel || rel === ".") return ".";
  if (!rel.startsWith("../") && rel !== "..") return rel;
  return normalizeRel(abs);
}

function runCommand(command, cmdArgs, label) {
  const proc = spawnSync(command, cmdArgs, { encoding: "utf8" });
  if (proc.status !== 0) {
    const stderr = (proc.stderr || "").trim();
    const stdout = (proc.stdout || "").trim();
    throw new Error(`${label} failed (${command}): ${stderr || stdout || "exit code " + String(proc.status)}`);
  }
  return {
    stdout: (proc.stdout || "").trim(),
    stderr: (proc.stderr || "").trim()
  };
}

function loadEnv() {
  const dotenvPath = path.join(root, ".env");
  if (!fs.existsSync(dotenvPath)) return;

  const lines = fs.readFileSync(dotenvPath, "utf8").split("\n");
  for (const raw of lines) {
    const line = raw.trim();
    if (!line || line.startsWith("#")) continue;
    const i = line.indexOf("=");
    if (i < 1) continue;

    const key = line.slice(0, i).trim();
    const val = line.slice(i + 1).trim().replace(/^['"]|['"]$/g, "");
    if (!process.env[key]) process.env[key] = val;
  }
}

function loadJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function loadManifestIfExists() {
  const manifestPath = path.join(root, "settings/manifest.json");
  if (!fs.existsSync(manifestPath)) return {};
  return loadJson(manifestPath);
}

function mustExist(relativePath, label = "file") {
  const abs = path.resolve(root, relativePath);
  if (!fs.existsSync(abs)) {
    throw new Error(`Missing ${label}: ${relativePath}`);
  }
}

function normalizeRel(p) {
  return String(p).replace(/\\/g, "/");
}

function ensureUnderAllowedRoots(relPath, allowedRoots) {
  const rel = normalizeRel(relPath);
  const ok = allowedRoots.some((r) => {
    const rootRel = normalizeRel(r);
    return rel === rootRel || rel.startsWith(rootRel.endsWith("/") ? rootRel : `${rootRel}/`);
  });
  if (!ok) {
    throw new Error(`Style anchor not allowed by policy: ${relPath}`);
  }
}

function mimeFor(filePath) {
  const ext = path.extname(filePath).toLowerCase();
  if (ext === ".png") return "image/png";
  if (ext === ".webp") return "image/webp";
  return "image/jpeg";
}

function sanitizeId(value) {
  return String(value)
    .replace(/\.[a-z0-9]+$/i, "")
    .replace(/[^a-zA-Z0-9_-]+/g, "_")
    .replace(/^_+|_+$/g, "")
    .toLowerCase();
}

function makeStamp() {
  return new Date().toISOString().replace(/[:.]/g, "-");
}

function ensureDir(absDir) {
  fs.mkdirSync(absDir, { recursive: true });
}

function resolveBackendCommonConfig() {
  const pythonArg = readArg("--backend-python-bin", "python3");
  const pythonBin = pythonArg.includes("/") || pythonArg.startsWith(".") ? resolvePath(pythonArg) : pythonArg;
  const scriptPath = path.join(root, "scripts/backend.py");
  const dbPath = resolvePath(readArg("--backend-db", "generated/backend/app.db"));
  return {
    pythonBin,
    scriptPath,
    dbPath
  };
}

function resolveProjectStorageFromBackend(projectId) {
  const enabled = parseBoolArg("--backend-storage-resolve", true);
  const required = parseBoolArg("--backend-storage-required", false);
  if (!enabled) {
    return { enabled, required, resolved: false, reason: "disabled", localRoot: null, storage: null };
  }

  const backend = resolveBackendCommonConfig();
  if (!fs.existsSync(backend.scriptPath)) {
    const msg = `Backend storage resolve script missing: ${toRel(backend.scriptPath)}`;
    if (required) throw new Error(msg);
    return { enabled, required, resolved: false, reason: msg, localRoot: null, storage: null };
  }

  try {
    const proc = runCommand(
      backend.pythonBin,
      [backend.scriptPath, "--db", backend.dbPath, "get-project-storage", "--project-slug", projectId],
      "Backend project storage resolve"
    );
    const json = JSON.parse(proc.stdout || "{}");
    const localRootRaw = json?.storage?.local?.project_root || "";
    if (!localRootRaw) {
      return { enabled, required, resolved: false, reason: "missing_project_root", localRoot: null, storage: json?.storage || null };
    }
    return {
      enabled,
      required,
      resolved: true,
      reason: "",
      localRoot: resolvePath(localRootRaw),
      storage: json?.storage || null
    };
  } catch (err) {
    const msg = err?.message || String(err);
    if (required) throw err;
    return { enabled, required, resolved: false, reason: msg, localRoot: null, storage: null };
  }
}

function getProjectContext() {
  const raw = readArg("--project", process.env.IMAGE_LAB_PROJECT || "default");
  const project = sanitizeId(raw) || "default";
  const projectRootArg = readArg("--project-root", "").trim();
  const backendStorage = resolveProjectStorageFromBackend(project);
  const projectRoot = projectRootArg
    ? resolvePath(projectRootArg)
    : backendStorage.resolved
      ? backendStorage.localRoot
      : path.join(root, "generated", "projects", project);
  const dirs = {
    root: projectRoot,
    outputs: path.join(projectRoot, "outputs"),
    runs: path.join(projectRoot, "runs"),
    upscaled: path.join(projectRoot, "upscaled"),
    color: path.join(projectRoot, "color_corrected"),
    bgRemove: path.join(projectRoot, "background_removed"),
    archiveBad: path.join(projectRoot, "archive", "bad"),
    archiveReplaced: path.join(projectRoot, "archive", "replaced")
  };
  return { id: project, dirs, storage: backendStorage.storage, storage_resolved_from_backend: backendStorage.resolved };
}

function archiveExistingTarget(targetAbs, archiveDirAbs, tag = "replaced") {
  if (!fs.existsSync(targetAbs)) return null;
  if (!fs.statSync(targetAbs).isFile()) return null;
  ensureDir(archiveDirAbs);
  const ext = path.extname(targetAbs);
  const base = sanitizeId(path.basename(targetAbs, ext)) || "file";
  const archivedAbs = path.join(archiveDirAbs, `${base}_${tag}_${makeStamp()}${ext}`);
  fs.renameSync(targetAbs, archivedAbs);
  return archivedAbs;
}

function maybeArchiveExisting(targetAbs, projectCtx) {
  if (hasFlag("--no-archive-replaced")) return null;
  return archiveExistingTarget(targetAbs, projectCtx.dirs.archiveReplaced, "replaced");
}

function composePrompt({ prompts, stage, time, weather, noInvention }) {
  const chunks = [];

  if (!prompts.style_base) {
    throw new Error("Missing prompts.style_base in settings/manifest.json");
  }
  chunks.push(prompts.style_base);

  if (stage === "time" || stage === "weather") {
    const key = `time_${time}`;
    if (!prompts[key]) throw new Error(`Missing prompts.${key}`);
    chunks.push(prompts[key]);
  }

  if (stage === "weather") {
    const key = `weather_${weather}`;
    if (!prompts[key]) throw new Error(`Missing prompts.${key}`);
    chunks.push(prompts[key]);
  }

  if (noInvention) {
    chunks.push("Do not invent new object categories.");
  }

  return chunks.join(" ");
}

function buildJobs({ manifest, stage, time, weather }) {
  const noInvention = Boolean(manifest?.policy?.default_no_invention);
  const prompt = composePrompt({
    prompts: manifest.prompts || {},
    stage,
    time,
    weather,
    noInvention
  });

  const sceneRefs = Array.isArray(manifest.scene_refs) ? manifest.scene_refs : [];
  const styleRefs = Array.isArray(manifest.style_refs) ? manifest.style_refs : [];

  if (!sceneRefs.length) {
    throw new Error("No scene_refs configured in settings/manifest.json");
  }

  return sceneRefs.map((scene, i) => {
    const idSuffix = sanitizeId(path.basename(scene)) || String(i + 1);
    return {
      id: `${stage}_${i + 1}_${idSuffix}`,
      mode: stage,
      time,
      weather,
      input_images: [scene, ...styleRefs],
      prompt
    };
  });
}

function loadPostprocessConfig() {
  const fallback = {
    upscale: {
      backend: "python",
      binary: "tools/realesrgan/realesrgan-ncnn-vulkan",
      model_dir: "tools/realesrgan/models",
      model_name: "realesrgan-x4plus",
      scale: 2,
      tile: 0,
      format: "png",
      output_dir: "generated/upscaled",
      python: {
        python_bin: "tools/realesrgan-python/.venv/bin/python",
        model_name: "RealESRGAN_x4plus",
        tile: 0,
        tile_pad: 10,
        pre_pad: 0,
        fp32: false,
        gpu_id: null
      }
    },
    color: {
      settings_file: "settings/color-correction.json",
      default_profile: "neutral",
      output_dir: "generated/color_corrected"
    },
    bg_remove: {
      output_dir: "generated/background_removed",
      backends: ["rembg"],
      format: "png",
      size: "auto",
      crop: false,
      rembg: {
        python_bin: "tools/rembg/.venv/bin/python",
        model: "u2net"
      },
      photoroom: {
        endpoint: "https://sdk.photoroom.com/v1/segment",
        api_key_env: "PHOTOROOM_API_KEY"
      },
      removebg: {
        endpoint: "https://api.remove.bg/v1.0/removebg",
        api_key_env: "REMOVE_BG_API_KEY"
      },
      openai: {
        enabled: true,
        required: true,
        api_key_env: "OPENAI_API_KEY",
        model: "gpt-image-1",
        quality: "high",
        input_fidelity: "high",
        output_format: "png",
        background: "transparent",
        prompt:
          "Refine this subject cutout for production compositing. Keep identity, pose, outfit, proportions and colors unchanged. Clean jagged edges and halos, recover natural hair strands and semi-transparent details, preserve transparent background. Do not add or remove objects, do not retouch skin, and do not change framing."
      }
    }
  };

  const configPath = path.join(root, "settings/postprocess.json");
  if (!fs.existsSync(configPath)) return fallback;

  const cfg = loadJson(configPath);
  return {
    ...fallback,
    ...cfg,
    upscale: {
      ...fallback.upscale,
      ...(cfg?.upscale || {}),
      python: {
        ...fallback.upscale.python,
        ...(cfg?.upscale?.python || {})
      }
    },
    color: { ...fallback.color, ...(cfg?.color || {}) },
    bg_remove: {
      ...fallback.bg_remove,
      ...(cfg?.bg_remove || {}),
      rembg: {
        ...fallback.bg_remove.rembg,
        ...(cfg?.bg_remove?.rembg || {})
      },
      photoroom: {
        ...fallback.bg_remove.photoroom,
        ...(cfg?.bg_remove?.photoroom || {})
      },
      removebg: {
        ...fallback.bg_remove.removebg,
        ...(cfg?.bg_remove?.removebg || {})
      },
      openai: {
        ...fallback.bg_remove.openai,
        ...(cfg?.bg_remove?.openai || {})
      }
    }
  };
}

function parseUpscaleBackend(postCfg) {
  const backend = String(readArg("--upscale-backend", postCfg?.upscale?.backend || "python")).toLowerCase();
  if (!["ncnn", "python"].includes(backend)) {
    throw new Error(`Invalid --upscale-backend '${backend}'. Expected ncnn|python.`);
  }
  return backend;
}

function ensureNcnnUpscaleReady(postCfg) {
  const binArg = readArg("--realesrgan-bin", "");
  const modelDirArg = readArg("--realesrgan-model-dir", "");

  const binPath = resolvePath(binArg || postCfg.upscale.binary);
  const modelDir = resolvePath(modelDirArg || postCfg.upscale.model_dir);

  if (!fs.existsSync(binPath)) {
    throw new Error(`Real-ESRGAN binary not found: ${toRel(binPath)}. Run: bash scripts/setup-realesrgan.sh`);
  }
  if (!fs.existsSync(modelDir)) {
    throw new Error(`Real-ESRGAN models dir not found: ${toRel(modelDir)}`);
  }

  return {
    binPath,
    modelDir,
    modelName: String(readArg("--realesrgan-model", postCfg.upscale.model_name || "realesrgan-x4plus")),
    scale: Number(readArg("--upscale-scale", String(postCfg.upscale.scale || 2))),
    tile: Number(readArg("--upscale-tile", String(postCfg.upscale.tile || 0))),
    format: String(readArg("--upscale-format", postCfg.upscale.format || "png"))
  };
}

function ensurePythonUpscaleReady(postCfg) {
  const pythonArg = readArg("--realesrgan-python-bin", "");
  const configured = pythonArg || postCfg?.upscale?.python?.python_bin || "python3";
  const pythonBin =
    configured.includes("/") || configured.startsWith(".") ? resolvePath(configured) : configured;
  if ((pythonBin.includes("/") || pythonBin.startsWith(".")) && !fs.existsSync(pythonBin)) {
    throw new Error(
      `Real-ESRGAN python runtime not found: ${toRel(pythonBin)}. Run: bash scripts/setup-realesrgan-python.sh`
    );
  }

  const scriptPath = path.join(root, "scripts/realesrgan-python-upscale.py");
  if (!fs.existsSync(scriptPath)) {
    throw new Error(`Missing script: ${toRel(scriptPath)}`);
  }

  const gpuIdRaw = readArg("--upscale-gpu-id", String(postCfg?.upscale?.python?.gpu_id ?? ""));
  const gpuId = gpuIdRaw === "" ? null : Number(gpuIdRaw);

  return {
    pythonBin,
    scriptPath,
    modelName: String(
      readArg("--realesrgan-model", postCfg?.upscale?.python?.model_name || "RealESRGAN_x4plus")
    ),
    outscale: Number(readArg("--upscale-scale", String(postCfg.upscale.scale || 2))),
    tile: Number(
      readArg("--upscale-tile", String(postCfg?.upscale?.python?.tile ?? postCfg?.upscale?.tile ?? 0))
    ),
    tilePad: Number(readArg("--upscale-tile-pad", String(postCfg?.upscale?.python?.tile_pad ?? 10))),
    prePad: Number(readArg("--upscale-pre-pad", String(postCfg?.upscale?.python?.pre_pad ?? 0))),
    fp32: hasFlag("--upscale-fp32") || Boolean(postCfg?.upscale?.python?.fp32),
    gpuId,
    format: String(readArg("--upscale-format", postCfg.upscale.format || "png"))
  };
}

function runUpscalePassNcnn({ inputPath, outputPath, postCfg, projectCtx }) {
  const inputAbs = resolvePath(inputPath);
  const outputAbs = resolvePath(outputPath);
  if (!fs.existsSync(inputAbs)) {
    throw new Error(`Upscale input does not exist: ${inputPath}`);
  }

  const cfg = ensureNcnnUpscaleReady(postCfg);

  const inputStat = fs.statSync(inputAbs);
  if (inputStat.isDirectory()) {
    fs.mkdirSync(outputAbs, { recursive: true });
  } else {
    fs.mkdirSync(path.dirname(outputAbs), { recursive: true });
    maybeArchiveExisting(outputAbs, projectCtx);
  }

  const cmdArgs = [
    "-i",
    inputAbs,
    "-o",
    outputAbs,
    "-s",
    String(cfg.scale),
    "-m",
    cfg.modelDir,
    "-n",
    cfg.modelName,
    "-f",
    cfg.format
  ];

  if (Number.isFinite(cfg.tile) && cfg.tile >= 0) {
    cmdArgs.push("-t", String(cfg.tile));
  }

  runCommand(cfg.binPath, cmdArgs, "Real-ESRGAN upscale");
  return {
    backend: "ncnn",
    input: toRel(inputAbs),
    output: toRel(outputAbs),
    scale: cfg.scale,
    model: cfg.modelName
  };
}

function runUpscalePassPython({ inputPath, outputPath, postCfg, projectCtx }) {
  const inputAbs = resolvePath(inputPath);
  const outputAbs = resolvePath(outputPath);
  if (!fs.existsSync(inputAbs)) {
    throw new Error(`Upscale input does not exist: ${inputPath}`);
  }

  const cfg = ensurePythonUpscaleReady(postCfg);

  const inputStat = fs.statSync(inputAbs);
  if (inputStat.isDirectory()) {
    fs.mkdirSync(outputAbs, { recursive: true });
  } else {
    fs.mkdirSync(path.dirname(outputAbs), { recursive: true });
    maybeArchiveExisting(outputAbs, projectCtx);
  }

  const cmdArgs = [
    cfg.scriptPath,
    "--input",
    inputAbs,
    "--output",
    outputAbs,
    "--model-name",
    cfg.modelName,
    "--outscale",
    String(cfg.outscale),
    "--tile",
    String(cfg.tile),
    "--tile-pad",
    String(cfg.tilePad),
    "--pre-pad",
    String(cfg.prePad),
    "--ext",
    cfg.format
  ];
  if (cfg.fp32) cmdArgs.push("--fp32");
  if (cfg.gpuId !== null && Number.isFinite(cfg.gpuId)) {
    cmdArgs.push("--gpu-id", String(cfg.gpuId));
  }

  runCommand(cfg.pythonBin, cmdArgs, "Real-ESRGAN upscale");
  return {
    backend: "python",
    input: toRel(inputAbs),
    output: toRel(outputAbs),
    scale: cfg.outscale,
    model: cfg.modelName
  };
}

function runUpscalePass({ inputPath, outputPath, postCfg, projectCtx }) {
  const backend = parseUpscaleBackend(postCfg);
  if (backend === "python") {
    return runUpscalePassPython({ inputPath, outputPath, postCfg, projectCtx });
  }
  return runUpscalePassNcnn({ inputPath, outputPath, postCfg, projectCtx });
}

function runColorPass({ inputPath, outputPath, postCfg, profile, projectCtx }) {
  const inputAbs = resolvePath(inputPath);
  const outputAbs = resolvePath(outputPath);
  if (!fs.existsSync(inputAbs)) {
    throw new Error(`Color-correction input does not exist: ${inputPath}`);
  }

  const pythonBin = readArg("--python-bin", "python3");
  const scriptPath = path.join(root, "scripts/apply-color-correction.py");
  const settingsPath = resolvePath(readArg("--color-settings", postCfg.color.settings_file));

  if (!fs.existsSync(scriptPath)) {
    throw new Error(`Missing script: ${toRel(scriptPath)}`);
  }
  if (!fs.existsSync(settingsPath)) {
    throw new Error(`Missing color settings file: ${toRel(settingsPath)}`);
  }

  const inputStat = fs.statSync(inputAbs);
  if (inputStat.isDirectory()) {
    fs.mkdirSync(outputAbs, { recursive: true });
  } else {
    fs.mkdirSync(path.dirname(outputAbs), { recursive: true });
    maybeArchiveExisting(outputAbs, projectCtx);
  }

  const chosenProfile = profile || readArg("--profile", postCfg.color.default_profile || "neutral");
  const cmdArgs = [
    scriptPath,
    "--settings",
    settingsPath,
    "--profile",
    chosenProfile,
    "--input",
    inputAbs,
    "--output",
    outputAbs
  ];

  runCommand(pythonBin, cmdArgs, "Color correction");
  return {
    input: toRel(inputAbs),
    output: toRel(outputAbs),
    profile: chosenProfile,
    settings: toRel(settingsPath)
  };
}

function parseBgRemoveBackends(postCfg) {
  const defaults = Array.isArray(postCfg?.bg_remove?.backends) ? postCfg.bg_remove.backends : ["rembg"];
  const list = parseCsvArg("--bg-remove-backends", defaults);
  const allowed = new Set(["rembg", "photoroom", "removebg"]);
  for (const item of list) {
    if (!allowed.has(item)) {
      throw new Error(`Invalid background-remove backend '${item}'. Expected rembg|photoroom|removebg.`);
    }
  }
  return list;
}

function ensureRembgReady(postCfg) {
  const pythonArg = readArg("--rembg-python-bin", "");
  const configured = pythonArg || postCfg?.bg_remove?.rembg?.python_bin || "python3";
  const pythonBin =
    configured.includes("/") || configured.startsWith(".") ? resolvePath(configured) : configured;
  if ((pythonBin.includes("/") || pythonBin.startsWith(".")) && !fs.existsSync(pythonBin)) {
    throw new Error(`rembg python runtime not found: ${toRel(pythonBin)}. Run: bash scripts/setup-rembg.sh`);
  }
  const scriptPath = path.join(root, "scripts/rembg-remove.py");
  if (!fs.existsSync(scriptPath)) {
    throw new Error(`Missing script: ${toRel(scriptPath)}`);
  }
  return {
    pythonBin,
    scriptPath,
    model: String(readArg("--rembg-model", postCfg?.bg_remove?.rembg?.model || "u2net"))
  };
}

function resolveBgFormat(postCfg) {
  const format = String(readArg("--bg-remove-format", postCfg?.bg_remove?.format || "png")).toLowerCase();
  if (!["png", "jpg", "jpeg", "webp"].includes(format)) {
    throw new Error(`Invalid --bg-remove-format '${format}'. Expected png|jpg|jpeg|webp.`);
  }
  return format === "jpeg" ? "jpg" : format;
}

function resolveBgRefineOpenAi(postCfg, format) {
  const cfg = postCfg?.bg_remove?.openai || {};
  const outputFormatRaw = String(readArg("--bg-refine-format", cfg.output_format || format || "png")).toLowerCase();
  if (!["png", "jpg", "jpeg", "webp"].includes(outputFormatRaw)) {
    throw new Error(`Invalid --bg-refine-format '${outputFormatRaw}'. Expected png|jpg|jpeg|webp.`);
  }
  const outputFormat = outputFormatRaw === "jpeg" ? "jpg" : outputFormatRaw;
  if (format && outputFormat !== format) {
    throw new Error(
      `--bg-refine-format '${outputFormat}' must match --bg-remove-format '${format}' for in-place refine output`
    );
  }
  return {
    enabled: parseBoolArg("--bg-refine-openai", Boolean(cfg.enabled)),
    required: parseBoolArg("--bg-refine-openai-required", Boolean(cfg.required)),
    apiKeyEnv: readArg("--bg-refine-api-key-env", cfg.api_key_env || "OPENAI_API_KEY"),
    model: readArg("--bg-refine-model", cfg.model || process.env.OPENAI_IMAGE_MODEL || "gpt-image-1"),
    quality: readArg("--bg-refine-quality", cfg.quality || process.env.OPENAI_IMAGE_QUALITY || "high"),
    inputFidelity: readArg("--bg-refine-input-fidelity", cfg.input_fidelity || "high"),
    outputFormat,
    background: readArg("--bg-refine-background", cfg.background || "transparent"),
    prompt: readArg(
      "--bg-refine-prompt",
      cfg.prompt ||
        "Refine this subject cutout for production compositing. Keep identity and details unchanged. Clean edge artifacts and preserve transparency."
    )
  };
}

function resolveBgOutputPath({ inputAbs, inputRootAbs, outputAbs, inputIsDir, format }) {
  if (!inputIsDir) {
    const outStat = fs.existsSync(outputAbs) ? fs.statSync(outputAbs) : null;
    const asDir = outputAbs.endsWith(path.sep) || outputAbs.endsWith("/") || (outStat && outStat.isDirectory());
    if (asDir || !isImagePath(outputAbs)) {
      fs.mkdirSync(outputAbs, { recursive: true });
      const base = sanitizeId(path.basename(inputAbs, path.extname(inputAbs))) || "image";
      return path.join(outputAbs, `${base}.${format}`);
    }
    fs.mkdirSync(path.dirname(outputAbs), { recursive: true });
    return outputAbs;
  }

  const rel = path.relative(inputRootAbs, inputAbs);
  const relNoExt = rel.replace(/\.[^.]+$/, "");
  const dst = path.join(outputAbs, `${relNoExt}.${format}`);
  fs.mkdirSync(path.dirname(dst), { recursive: true });
  return dst;
}

function runBackgroundRemoveRembg({ inputAbs, outputAbs, postCfg, format }) {
  const cfg = ensureRembgReady(postCfg);
  const cmdArgs = [
    cfg.scriptPath,
    "--input",
    inputAbs,
    "--output",
    outputAbs,
    "--model",
    cfg.model,
    "--format",
    format
  ];
  runCommand(cfg.pythonBin, cmdArgs, "Background remove (rembg)");
}

async function runBackgroundRemovePhotoRoom({ inputAbs, outputAbs, postCfg, format }) {
  const endpoint = readArg("--photoroom-endpoint", postCfg?.bg_remove?.photoroom?.endpoint);
  const keyEnv = postCfg?.bg_remove?.photoroom?.api_key_env || "PHOTOROOM_API_KEY";
  const apiKey = process.env[keyEnv];
  if (!apiKey) {
    throw new Error(`Missing ${keyEnv} for PhotoRoom background remove`);
  }

  const size = String(readArg("--bg-remove-size", postCfg?.bg_remove?.size || "auto"));
  const crop = parseBoolArg("--bg-remove-crop", Boolean(postCfg?.bg_remove?.crop));
  const buffer = fs.readFileSync(inputAbs);
  const form = new FormData();
  form.append("image_file", new Blob([buffer], { type: mimeFor(inputAbs) }), path.basename(inputAbs));
  if (size && size !== "auto") form.append("size", size);
  if (format) form.append("format", format);
  if (crop) form.append("crop", "true");

  const res = await fetch(endpoint, {
    method: "POST",
    headers: { "x-api-key": apiKey },
    body: form
  });
  if (!res.ok) {
    throw new Error(`HTTP ${res.status}: ${await res.text()}`);
  }
  fs.mkdirSync(path.dirname(outputAbs), { recursive: true });
  fs.writeFileSync(outputAbs, Buffer.from(await res.arrayBuffer()));
}

async function runBackgroundRemoveRemoveBg({ inputAbs, outputAbs, postCfg, format }) {
  const endpoint = readArg("--removebg-endpoint", postCfg?.bg_remove?.removebg?.endpoint);
  const keyEnv = postCfg?.bg_remove?.removebg?.api_key_env || "REMOVE_BG_API_KEY";
  const apiKey = process.env[keyEnv];
  if (!apiKey) {
    throw new Error(`Missing ${keyEnv} for remove.bg background remove`);
  }

  const size = String(readArg("--bg-remove-size", postCfg?.bg_remove?.size || "auto"));
  const crop = parseBoolArg("--bg-remove-crop", Boolean(postCfg?.bg_remove?.crop));
  const buffer = fs.readFileSync(inputAbs);
  const form = new FormData();
  form.append("image_file", new Blob([buffer], { type: mimeFor(inputAbs) }), path.basename(inputAbs));
  if (size && size !== "auto") form.append("size", size);
  if (format) form.append("format", format);
  if (crop) form.append("crop", "true");

  const res = await fetch(endpoint, {
    method: "POST",
    headers: { "X-Api-Key": apiKey },
    body: form
  });
  if (!res.ok) {
    throw new Error(`HTTP ${res.status}: ${await res.text()}`);
  }
  fs.mkdirSync(path.dirname(outputAbs), { recursive: true });
  fs.writeFileSync(outputAbs, Buffer.from(await res.arrayBuffer()));
}

async function runBackgroundRefineOpenAi({ inputAbs, outputAbs, refineCfg }) {
  const apiKey = process.env[refineCfg.apiKeyEnv];
  if (!apiKey) {
    throw new Error(`Missing ${refineCfg.apiKeyEnv} for OpenAI background refinement`);
  }

  const buffer = fs.readFileSync(inputAbs);
  const form = new FormData();
  form.append("model", refineCfg.model);
  form.append("prompt", refineCfg.prompt);
  form.append("quality", refineCfg.quality);
  form.append("input_fidelity", refineCfg.inputFidelity);
  form.append("output_format", refineCfg.outputFormat);
  form.append("background", refineCfg.background);
  form.append("image[]", new Blob([buffer], { type: mimeFor(inputAbs) }), path.basename(inputAbs));

  const res = await fetch("https://api.openai.com/v1/images/edits", {
    method: "POST",
    headers: { Authorization: `Bearer ${apiKey}` },
    body: form
  });
  if (!res.ok) {
    throw new Error(`HTTP ${res.status}: ${await res.text()}`);
  }

  const json = await res.json();
  const b64 = json?.data?.[0]?.b64_json;
  if (!b64) {
    throw new Error("OpenAI refine returned no image payload");
  }

  fs.mkdirSync(path.dirname(outputAbs), { recursive: true });
  fs.writeFileSync(outputAbs, Buffer.from(b64, "base64"));
  return { model: refineCfg.model, output_format: refineCfg.outputFormat };
}

async function removeBackgroundWithFallback({ inputAbs, outputAbs, postCfg, backends, format }) {
  const failures = [];
  for (const backend of backends) {
    try {
      if (backend === "rembg") {
        runBackgroundRemoveRembg({ inputAbs, outputAbs, postCfg, format });
      } else if (backend === "photoroom") {
        await runBackgroundRemovePhotoRoom({ inputAbs, outputAbs, postCfg, format });
      } else if (backend === "removebg") {
        await runBackgroundRemoveRemoveBg({ inputAbs, outputAbs, postCfg, format });
      } else {
        throw new Error(`Unsupported backend '${backend}'`);
      }
      return backend;
    } catch (err) {
      failures.push(`${backend}: ${err?.message || String(err)}`);
    }
  }
  throw new Error(`Background remove failed for ${toRel(inputAbs)}. ${failures.join(" | ")}`);
}

async function runBackgroundRemovePass({ inputPath, outputPath, postCfg, projectCtx }) {
  const inputAbs = resolvePath(inputPath);
  const outputAbs = resolvePath(outputPath);
  if (!fs.existsSync(inputAbs)) {
    throw new Error(`Background-remove input does not exist: ${inputPath}`);
  }

  const backends = parseBgRemoveBackends(postCfg);
  const format = resolveBgFormat(postCfg);
  const refineCfg = resolveBgRefineOpenAi(postCfg, format);
  const files = listImageFiles(inputAbs);
  if (!files.length) {
    throw new Error("No image files found for background remove");
  }

  const inputIsDir = fs.statSync(inputAbs).isDirectory();
  if (inputIsDir) fs.mkdirSync(outputAbs, { recursive: true });

  const results = [];
  for (const fileAbs of files) {
    const dstAbs = resolveBgOutputPath({
      inputAbs: fileAbs,
      inputRootAbs: inputAbs,
      outputAbs,
      inputIsDir,
      format
    });
    maybeArchiveExisting(dstAbs, projectCtx);
    const backendUsed = await removeBackgroundWithFallback({
      inputAbs: fileAbs,
      outputAbs: dstAbs,
      postCfg,
      backends,
      format
    });
    let refineApplied = false;
    let refineError = null;
    if (refineCfg.enabled) {
      const ext = path.extname(dstAbs);
      const tmpAbs = path.join(path.dirname(dstAbs), `${path.basename(dstAbs, ext)}_openai_tmp_${makeStamp()}${ext}`);
      try {
        await runBackgroundRefineOpenAi({ inputAbs: dstAbs, outputAbs: tmpAbs, refineCfg });
        maybeArchiveExisting(dstAbs, projectCtx);
        fs.renameSync(tmpAbs, dstAbs);
        refineApplied = true;
      } catch (err) {
        if (fs.existsSync(tmpAbs)) fs.unlinkSync(tmpAbs);
        refineError = err?.message || String(err);
        if (refineCfg.required) {
          throw new Error(`OpenAI refine failed for ${toRel(fileAbs)}: ${refineError}`);
        }
        console.warn(`OpenAI refine skipped for ${toRel(fileAbs)}: ${refineError}`);
      }
    }
    results.push({
      input: toRel(fileAbs),
      output: toRel(dstAbs),
      backend: backendUsed,
      refine_openai: refineApplied,
      refine_error: refineError
    });
  }

  return {
    input: toRel(inputAbs),
    output: toRel(outputAbs),
    backends,
    refine_openai: refineCfg.enabled,
    refine_openai_required: refineCfg.required,
    format,
    processed: results.length,
    results
  };
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
    const abs = path.resolve(root, rel);
    const buf = fs.readFileSync(abs);
    const blob = new Blob([buf], { type: mimeFor(abs) });
    form.append("image[]", blob, path.basename(abs));
  }

  const res = await fetch("https://api.openai.com/v1/images/edits", {
    method: "POST",
    headers: { Authorization: `Bearer ${apiKey}` },
    body: form
  });

  if (!res.ok) {
    throw new Error(`HTTP ${res.status}: ${await res.text()}`);
  }

  const json = await res.json();
  const b64 = json?.data?.[0]?.b64_json;
  if (!b64) {
    throw new Error("API returned no image payload");
  }

  return b64;
}

function resolveOutputGuardConfig(manifest) {
  const cfg = manifest?.output_guard || {};
  const enabled = parseBoolArg("--output-guard-enabled", true);
  const enforceGrayscale = parseBoolArg("--enforce-grayscale", Boolean(cfg.enforce_grayscale));
  const failOnChromaExceed = parseBoolArg("--fail-on-chroma-exceed", Boolean(cfg.fail_on_chroma_exceed));
  const maxChromaDelta = Number(readArg("--max-chroma-delta", String(cfg.max_chroma_delta ?? 2)));
  if (!Number.isFinite(maxChromaDelta) || maxChromaDelta < 0) {
    throw new Error(`Invalid --max-chroma-delta '${maxChromaDelta}'. Expected a non-negative number.`);
  }

  const pythonArg = readArg("--qa-python-bin", "python3");
  const pythonBin = pythonArg.includes("/") || pythonArg.startsWith(".") ? resolvePath(pythonArg) : pythonArg;
  if ((pythonBin.includes("/") || pythonBin.startsWith(".")) && !fs.existsSync(pythonBin)) {
    throw new Error(`QA python runtime not found: ${toRel(pythonBin)}`);
  }

  return {
    enabled,
    enforceGrayscale,
    failOnChromaExceed,
    maxChromaDelta,
    pythonBin
  };
}

function runOutputGuard({ inputPath, guardCfg }) {
  const inputAbs = resolvePath(inputPath);
  if (!fs.existsSync(inputAbs)) {
    throw new Error(`Output-guard input does not exist: ${inputPath}`);
  }

  const scriptPath = path.join(root, "scripts/output-guard.py");
  if (!fs.existsSync(scriptPath)) {
    throw new Error(`Missing script: ${toRel(scriptPath)}`);
  }

  const cmdArgs = [scriptPath, "--input", inputAbs, "--max-chroma-delta", String(guardCfg.maxChromaDelta)];
  if (guardCfg.enforceGrayscale) cmdArgs.push("--enforce-grayscale");
  if (guardCfg.failOnChromaExceed) cmdArgs.push("--fail-on-chroma-exceed");
  const proc = runCommand(guardCfg.pythonBin, cmdArgs, "Output guard");

  let report = null;
  try {
    report = JSON.parse(proc.stdout || "{}");
  } catch {
    throw new Error(`Output guard returned invalid JSON for ${toRel(inputAbs)}`);
  }
  if (!report || typeof report !== "object") {
    throw new Error(`Output guard returned no report for ${toRel(inputAbs)}`);
  }
  return report;
}

function parseCandidateCount(manifest, dry) {
  const configured = Number(manifest?.generation?.candidates ?? 1);
  const raw = readArg("--candidates", String(Number.isFinite(configured) ? configured : 1));
  const count = Number(raw);
  if (!Number.isInteger(count) || count < 1) {
    throw new Error(`Invalid --candidates '${raw}'. Expected integer >= 1.`);
  }
  const hardMax = Number(readArg("--max-candidates", String(manifest?.generation?.max_candidates ?? 6)));
  if (!Number.isInteger(hardMax) || hardMax < 1) {
    throw new Error(`Invalid --max-candidates '${hardMax}'. Expected integer >= 1.`);
  }
  if (count > hardMax && !dry && !hasFlag("--allow-many-candidates")) {
    throw new Error(
      `Candidate count ${count} exceeds limit ${hardMax}. Use --allow-many-candidates to override.`
    );
  }
  return { count, hardMax };
}

function summarizeGuardReport(report, threshold) {
  const summary = report?.summary || {};
  const hardFailures = Number(summary.hard_failures || 0);
  const softWarnings = Number(summary.soft_warnings || 0);
  const files = Array.isArray(report?.files) ? report.files : [];
  const exceeds = files
    .map((f) => Number(f?.chroma_delta))
    .filter((v) => Number.isFinite(v))
    .map((v) => Math.max(0, v - threshold));
  const avgChromaExceed = exceeds.length
    ? Number((exceeds.reduce((acc, v) => acc + v, 0) / exceeds.length).toFixed(4))
    : 0;
  return { hardFailures, softWarnings, avgChromaExceed };
}

function pickBestCandidate(candidates) {
  const viable = candidates.filter((c) => c.status === "done");
  if (!viable.length) return null;
  viable.sort((a, b) => {
    if (a.rank.hard_failures !== b.rank.hard_failures) return a.rank.hard_failures - b.rank.hard_failures;
    if (a.rank.soft_warnings !== b.rank.soft_warnings) return a.rank.soft_warnings - b.rank.soft_warnings;
    if (a.rank.avg_chroma_exceed !== b.rank.avg_chroma_exceed) {
      return a.rank.avg_chroma_exceed - b.rank.avg_chroma_exceed;
    }
    return a.candidate_index - b.candidate_index;
  });
  return viable[0];
}

function resolveBackendIngestConfig() {
  const enabled = parseBoolArg("--backend-db-ingest", true);
  const required = parseBoolArg("--backend-db-required", false);
  const backend = resolveBackendCommonConfig();
  return {
    enabled,
    required,
    ...backend
  };
}

function maybeIngestRunToBackend({ backendCfg, projectCtx, runLogPath }) {
  if (!backendCfg.enabled) {
    return { enabled: false, attempted: false, ok: true, reason: "disabled" };
  }
  if (!fs.existsSync(backendCfg.scriptPath)) {
    const msg = `Backend ingest script missing: ${toRel(backendCfg.scriptPath)}`;
    if (backendCfg.required) throw new Error(msg);
    console.warn(msg);
    return { enabled: true, attempted: false, ok: false, reason: "missing_script" };
  }

  const cmdArgs = [
    backendCfg.scriptPath,
    "--db",
    backendCfg.dbPath,
    "ingest-run",
    "--run-log",
    resolvePath(runLogPath),
    "--project-slug",
    projectCtx.id,
    "--project-name",
    projectCtx.id
  ];

  try {
    const proc = runCommand(backendCfg.pythonBin, cmdArgs, "Backend run ingest");
    let payload = null;
    try {
      payload = JSON.parse(proc.stdout || "{}");
    } catch {
      payload = null;
    }
    return { enabled: true, attempted: true, ok: true, payload };
  } catch (err) {
    const msg = err?.message || String(err);
    if (backendCfg.required) throw err;
    console.warn(`Backend ingest skipped: ${msg}`);
    return { enabled: true, attempted: true, ok: false, reason: msg };
  }
}

function resolveStorageSyncConfig() {
  return {
    enabled: parseBoolArg("--storage-sync-s3", false),
    required: parseBoolArg("--storage-sync-required", false),
    dryRun: parseBoolArg("--storage-sync-dry-run", false),
    delete: parseBoolArg("--storage-sync-delete", false),
    allowMissingLocal: parseBoolArg("--storage-sync-allow-missing-local", true)
  };
}

function maybeSyncProjectS3({ backendCfg, projectCtx, syncCfg }) {
  if (!syncCfg.enabled) {
    return { enabled: false, attempted: false, ok: true, reason: "disabled" };
  }
  if (!fs.existsSync(backendCfg.scriptPath)) {
    const msg = `Backend S3 sync script missing: ${toRel(backendCfg.scriptPath)}`;
    if (syncCfg.required) throw new Error(msg);
    console.warn(msg);
    return { enabled: true, attempted: false, ok: false, reason: msg };
  }

  const cmdArgs = [
    backendCfg.scriptPath,
    "--db",
    backendCfg.dbPath,
    "sync-project-s3",
    "--project-slug",
    projectCtx.id
  ];
  if (syncCfg.dryRun) cmdArgs.push("--dry-run");
  if (syncCfg.delete) cmdArgs.push("--delete");
  if (syncCfg.allowMissingLocal) cmdArgs.push("--allow-missing-local");

  try {
    const proc = runCommand(backendCfg.pythonBin, cmdArgs, "Backend S3 sync");
    let payload = null;
    try {
      payload = JSON.parse(proc.stdout || "{}");
    } catch {
      payload = null;
    }
    return { enabled: true, attempted: true, ok: true, payload };
  } catch (err) {
    const msg = err?.message || String(err);
    if (syncCfg.required) throw err;
    console.warn(`S3 sync skipped: ${msg}`);
    return { enabled: true, attempted: true, ok: false, reason: msg };
  }
}

function usage() {
  return [
    "Usage:",
    "  npm run lab -- dry [--project NAME] [--stage style|time|weather] [--time day|night] [--weather clear|rain] [--candidates N]",
    "  npm run lab -- run --project NAME --confirm-spend [--stage style|time|weather] [--time day|night] [--weather clear|rain] [--candidates N] [--project-root PATH] [--post-upscale] [--upscale-backend ncnn|python] [--post-color] [--post-color-profile PROFILE] [--post-bg-remove]",
    "  npm run lab -- upscale [--project NAME] [--input PATH] [--output PATH] [--upscale-backend ncnn|python] [--upscale-scale 2|3|4]",
    "  npm run lab -- color [--project NAME] [--input PATH] [--output PATH] [--profile PROFILE] [--color-settings FILE]",
    "  npm run lab -- bgremove [--project NAME] [--input PATH] [--output PATH] [--bg-remove-backends rembg,photoroom,removebg] [--bg-refine-openai true|false]",
    "  npm run lab -- qa [--project NAME] [--input PATH] [--output-guard-enabled true|false]",
    "  npm run lab -- archive-bad --project NAME --input PATH",
    "",
    "Defaults:",
    "  mode: dry",
    "  project: default",
    "  stage: style",
    "  time: day",
    "  weather: clear",
    "  candidates: 1",
    "  output guard: enabled",
    "  backend run ingest: enabled",
    "  s3 sync after run: disabled",
    "  auto-archive on overwrite: enabled (disable with --no-archive-replaced)"
  ].join("\n");
}

function buildFileOutputPath({ outputDir, fileName, suffix = "", extension = ".png" }) {
  const base = sanitizeId(path.basename(fileName, path.extname(fileName))) || "image";
  const safeSuffix = suffix ? `_${sanitizeId(suffix)}` : "";
  return path.join(outputDir, `${base}${safeSuffix}${extension}`);
}

async function runGenerationMode(mode) {
  const projectCtx = getProjectContext();
  const projectDirs = projectCtx.dirs;
  ensureDir(projectDirs.outputs);
  ensureDir(projectDirs.runs);
  ensureDir(projectDirs.archiveBad);
  ensureDir(projectDirs.archiveReplaced);

  const stage = readArg("--stage", "style");
  if (!["style", "time", "weather"].includes(stage)) {
    throw new Error(`Invalid --stage '${stage}'. Expected style|time|weather.`);
  }

  const time = readArg("--time", "day");
  if (!["day", "night"].includes(time)) {
    throw new Error(`Invalid --time '${time}'. Expected day|night.`);
  }

  const weather = readArg("--weather", "clear");
  if (!["clear", "rain"].includes(weather)) {
    throw new Error(`Invalid --weather '${weather}'. Expected clear|rain.`);
  }

  const confirmSpend = hasFlag("--confirm-spend");
  const dry = mode === "dry";

  mustExist("settings/manifest.json", "manifest");
  const manifest = loadJson(path.join(root, "settings/manifest.json"));

  const allowedStyleRoots = Array.isArray(manifest?.policy?.allowed_style_roots)
    ? manifest.policy.allowed_style_roots
    : ["references/"];

  const styleRefs = Array.isArray(manifest.style_refs) ? manifest.style_refs : [];
  for (const rel of styleRefs) {
    ensureUnderAllowedRoots(rel, allowedStyleRoots);
    mustExist(rel, "style reference");
  }

  const sceneRefs = Array.isArray(manifest.scene_refs) ? manifest.scene_refs : [];
  for (const rel of sceneRefs) {
    mustExist(rel, "scene reference");
  }

  const jobs = buildJobs({ manifest, stage, time, weather });
  const safeBatchLimit = Number(manifest.safe_batch_limit || 20);
  if (jobs.length > safeBatchLimit && !hasFlag("--allow-large-batch")) {
    throw new Error(`Batch exceeds safety limit (${safeBatchLimit}). Use --allow-large-batch to override.`);
  }

  if (!dry && !confirmSpend) {
    throw new Error("Spending is locked. Add --confirm-spend for paid calls.");
  }

  const model = readArg("--model", process.env.OPENAI_IMAGE_MODEL || "gpt-image-1");
  const size = readArg("--size", process.env.OPENAI_IMAGE_SIZE || "1024x1536");
  const quality = readArg("--quality", process.env.OPENAI_IMAGE_QUALITY || "high");

  if (!dry && !process.env.OPENAI_API_KEY) {
    throw new Error("Missing OPENAI_API_KEY in environment or .env");
  }

  const outputsDir = projectDirs.outputs;
  const runsDir = projectDirs.runs;

  const postCfg = loadPostprocessConfig();
  const postUpscale = hasFlag("--post-upscale");
  const postColor = hasFlag("--post-color");
  const postBgRemove = hasFlag("--post-bg-remove");
  const outputGuardCfg = resolveOutputGuardConfig(manifest);
  const upscaleBackend = postUpscale ? parseUpscaleBackend(postCfg) : null;
  const postColorProfile = readArg("--post-color-profile", "") || postCfg.color.default_profile;
  const bgRemoveBackends = postBgRemove ? parseBgRemoveBackends(postCfg) : [];
  const bgRefineCfg = postBgRemove ? resolveBgRefineOpenAi(postCfg, resolveBgFormat(postCfg)) : null;
  const pipelineOrder = ["generate"];
  if (postBgRemove) {
    pipelineOrder.push("bg_remove");
    if (bgRefineCfg?.enabled) pipelineOrder.push("bg_refine_openai");
  }
  if (postUpscale) pipelineOrder.push("upscale");
  if (postColor) pipelineOrder.push("color");
  const upscaleOutputDir = resolvePath(readArg("--upscale-output", projectDirs.upscaled));
  const colorOutputDir = resolvePath(readArg("--color-output", projectDirs.color));
  const bgRemoveOutputDir = resolvePath(readArg("--bg-remove-output", projectDirs.bgRemove));
  const candidateCfg = parseCandidateCount(manifest, dry);
  const backendCfg = resolveBackendIngestConfig();
  const storageSyncCfg = resolveStorageSyncConfig();

  const stamp = makeStamp();
  const runLogPath = path.join(runsDir, `run_${stamp}.json`);

  const runMeta = {
    timestamp: new Date().toISOString(),
    project: projectCtx.id,
    mode,
    stage,
    time,
    weather,
    model,
    size,
    quality,
    generation: {
      candidates: candidateCfg.count,
      max_candidates: candidateCfg.hardMax
    },
    postprocess: {
      upscale: postUpscale,
      upscale_backend: upscaleBackend,
      color: postColor,
      color_profile: postColor ? postColorProfile : null,
      bg_remove: postBgRemove,
      bg_remove_backends: postBgRemove ? bgRemoveBackends : [],
      bg_refine_openai: postBgRemove ? Boolean(bgRefineCfg?.enabled) : false,
      bg_refine_openai_required: postBgRemove ? Boolean(bgRefineCfg?.required) : false,
      pipeline_order: pipelineOrder
    },
    output_guard: {
      enabled: outputGuardCfg.enabled,
      enforce_grayscale: outputGuardCfg.enforceGrayscale,
      max_chroma_delta: outputGuardCfg.maxChromaDelta,
      fail_on_chroma_exceed: outputGuardCfg.failOnChromaExceed
    },
    backend: {
      ingest_enabled: backendCfg.enabled,
      db: toRel(backendCfg.dbPath)
    },
    storage: {
      project_root: toRel(projectDirs.root),
      resolved_from_backend: Boolean(projectCtx.storage_resolved_from_backend),
      s3_sync_enabled: storageSyncCfg.enabled,
      s3_sync_dry_run: storageSyncCfg.dryRun
    },
    jobs: []
  };

  let failedOutputGuardJobs = 0;

  for (const job of jobs) {
    if (dry) {
      runMeta.jobs.push({
        ...job,
        status: "planned",
        planned_generation: {
          candidates: candidateCfg.count
        },
        planned_postprocess: {
          upscale: postUpscale,
          upscale_backend: upscaleBackend,
          color: postColor,
          color_profile: postColor ? postColorProfile : null,
          bg_remove: postBgRemove,
          bg_remove_backends: postBgRemove ? bgRemoveBackends : [],
          bg_refine_openai: postBgRemove ? Boolean(bgRefineCfg?.enabled) : false,
          bg_refine_openai_required: postBgRemove ? Boolean(bgRefineCfg?.required) : false,
          pipeline_order: pipelineOrder
        },
        planned_output_guard: {
          enabled: outputGuardCfg.enabled,
          enforce_grayscale: outputGuardCfg.enforceGrayscale,
          max_chroma_delta: outputGuardCfg.maxChromaDelta,
          fail_on_chroma_exceed: outputGuardCfg.failOnChromaExceed
        }
      });
      continue;
    }

    const jobMeta = {
      ...job,
      status: "running",
      selected_candidate: null,
      final_output: null,
      candidates: []
    };

    for (let candidateIndex = 1; candidateIndex <= candidateCfg.count; candidateIndex += 1) {
      const b64 = await callImagesEdits({
        apiKey: process.env.OPENAI_API_KEY,
        model,
        size,
        quality,
        prompt: job.prompt,
        inputImages: job.input_images
      });

      const candidateSuffix = candidateCfg.count > 1 ? `__c${candidateIndex}` : "";
      const outPath = path.join(outputsDir, `${job.id}${candidateSuffix}.png`);
      maybeArchiveExisting(outPath, projectCtx);
      fs.writeFileSync(outPath, Buffer.from(b64, "base64"));

      const candidateMeta = {
        candidate_index: candidateIndex,
        output: toRel(outPath),
        status: "generated",
        rank: {
          hard_failures: 0,
          soft_warnings: 0,
          avg_chroma_exceed: 0
        }
      };

      let currentPath = outPath;

      if (postBgRemove) {
        const bgFormat = resolveBgFormat(postCfg);
        const bgOutPath = buildFileOutputPath({
          outputDir: bgRemoveOutputDir,
          fileName: path.basename(currentPath),
          suffix: "nobg",
          extension: `.${bgFormat}`
        });
        const bgInfo = await runBackgroundRemovePass({
          inputPath: currentPath,
          outputPath: bgOutPath,
          postCfg,
          projectCtx
        });
        const single = bgInfo.results[0];
        candidateMeta.bg_remove = {
          input: single?.input || bgInfo.input,
          output: single?.output || bgInfo.output,
          backend: single?.backend || null,
          backends_tried: bgInfo.backends,
          refine_openai: Boolean(single?.refine_openai),
          refine_error: single?.refine_error || null
        };
        currentPath = resolvePath(candidateMeta.bg_remove.output);
      }

      if (postUpscale) {
        const upscaleOutPath = buildFileOutputPath({
          outputDir: upscaleOutputDir,
          fileName: path.basename(currentPath),
          suffix: `x${readArg("--upscale-scale", String(postCfg.upscale.scale || 2))}`,
          extension: `.${readArg("--upscale-format", postCfg.upscale.format || "png")}`
        });
        candidateMeta.upscale = runUpscalePass({
          inputPath: currentPath,
          outputPath: upscaleOutPath,
          postCfg,
          projectCtx
        });
        currentPath = resolvePath(candidateMeta.upscale.output);
      }

      if (postColor) {
        const colorOutPath = buildFileOutputPath({
          outputDir: colorOutputDir,
          fileName: path.basename(currentPath),
          suffix: postColorProfile,
          extension: ".png"
        });
        candidateMeta.color = runColorPass({
          inputPath: currentPath,
          outputPath: colorOutPath,
          postCfg,
          profile: postColorProfile,
          projectCtx
        });
        currentPath = resolvePath(candidateMeta.color.output);
      }

      if (outputGuardCfg.enabled) {
        const guardReport = runOutputGuard({ inputPath: currentPath, guardCfg: outputGuardCfg });
        const rank = summarizeGuardReport(guardReport, outputGuardCfg.maxChromaDelta);
        candidateMeta.rank = {
          hard_failures: rank.hardFailures,
          soft_warnings: rank.softWarnings,
          avg_chroma_exceed: rank.avgChromaExceed
        };
        candidateMeta.output_guard = {
          checked_input: toRel(resolvePath(currentPath)),
          summary: {
            total_files: Number(guardReport?.summary?.total_files || 0),
            hard_failures: rank.hardFailures,
            soft_warnings: rank.softWarnings
          },
          files: Array.isArray(guardReport?.files)
            ? guardReport.files.map((f) => ({
                ...f,
                file: normalizeRel(path.relative(root, String(f?.file || "")))
              }))
            : []
        };

        if (rank.hardFailures > 0) {
          const archivedBadAbs = archiveExistingTarget(resolvePath(currentPath), projectCtx.dirs.archiveBad, "bad");
          candidateMeta.status = "failed_output_guard";
          candidateMeta.output_guard.bad_archive = archivedBadAbs ? toRel(archivedBadAbs) : null;
          jobMeta.candidates.push(candidateMeta);
          continue;
        }
      }

      candidateMeta.status = "done";
      candidateMeta.final_output = toRel(resolvePath(currentPath));
      jobMeta.candidates.push(candidateMeta);
    }

    const winner = pickBestCandidate(jobMeta.candidates);
    if (!winner) {
      jobMeta.status = "failed_output_guard";
      jobMeta.failure_reason = "all_candidates_failed_output_guard";
      failedOutputGuardJobs += 1;
      runMeta.jobs.push(jobMeta);
      continue;
    }

    jobMeta.status = "done";
    jobMeta.selected_candidate = winner.candidate_index;
    jobMeta.final_output = winner.final_output;
    jobMeta.output = winner.final_output;
    if (winner.bg_remove) jobMeta.bg_remove = winner.bg_remove;
    if (winner.upscale) jobMeta.upscale = winner.upscale;
    if (winner.color) jobMeta.color = winner.color;
    if (winner.output_guard) jobMeta.output_guard = winner.output_guard;
    runMeta.jobs.push(jobMeta);
  }

  fs.writeFileSync(runLogPath, `${JSON.stringify(runMeta, null, 2)}\n`);
  const backendIngest = maybeIngestRunToBackend({
    backendCfg,
    projectCtx,
    runLogPath
  });
  const storageSync = maybeSyncProjectS3({
    backendCfg,
    projectCtx,
    syncCfg: storageSyncCfg
  });
  console.log(`Run log: ${path.relative(root, runLogPath)}`);
  console.log(`Project: ${projectCtx.id}`);
  console.log(`Project root: ${toRel(projectDirs.root)}`);
  console.log(`Jobs: ${jobs.length} (${dry ? "dry/planned" : "run/completed"})`);
  if (backendIngest.enabled && backendIngest.attempted) {
    console.log(`Backend ingest: ${backendIngest.ok ? "ok" : "failed"}`);
  }
  if (storageSync.enabled && storageSync.attempted) {
    console.log(`S3 sync: ${storageSync.ok ? "ok" : "failed"}`);
  }
  if (failedOutputGuardJobs > 0) {
    throw new Error(
      `Output guard failed for ${failedOutputGuardJobs} job(s). Bad outputs moved to ${toRel(projectCtx.dirs.archiveBad)}`
    );
  }
}

function runUpscaleOnlyMode() {
  const projectCtx = getProjectContext();
  const postCfg = loadPostprocessConfig();
  const inputPath = readArg("--input", projectCtx.dirs.outputs);
  const outputPath = readArg("--output", projectCtx.dirs.upscaled);
  const info = runUpscalePass({ inputPath, outputPath, postCfg, projectCtx });
  console.log(
    `Upscale done: ${info.input} -> ${info.output} (backend ${info.backend}, scale x${info.scale}, model ${info.model})`
  );
}

function runColorOnlyMode() {
  const projectCtx = getProjectContext();
  const postCfg = loadPostprocessConfig();
  const inputPath = readArg("--input", projectCtx.dirs.outputs);
  const outputPath = readArg("--output", projectCtx.dirs.color);
  const profile = readArg("--profile", postCfg.color.default_profile || "neutral");
  const info = runColorPass({ inputPath, outputPath, postCfg, profile, projectCtx });
  console.log(`Color correction done: ${info.input} -> ${info.output} (profile ${info.profile})`);
}

async function runBgRemoveOnlyMode() {
  const projectCtx = getProjectContext();
  const postCfg = loadPostprocessConfig();
  const inputPath = readArg("--input", projectCtx.dirs.outputs);
  const outputPath = readArg("--output", projectCtx.dirs.bgRemove);
  const info = await runBackgroundRemovePass({ inputPath, outputPath, postCfg, projectCtx });
  console.log(
    `Background remove done: ${info.input} -> ${info.output} (${info.processed} file(s), backends ${info.backends.join(" -> ")}, openai refine ${info.refine_openai ? "on" : "off"})`
  );
}

function runQaMode() {
  const projectCtx = getProjectContext();
  const manifest = loadManifestIfExists();
  const outputGuardCfg = resolveOutputGuardConfig(manifest);
  if (!outputGuardCfg.enabled) {
    console.log("Output guard is disabled (--output-guard-enabled false).");
    return;
  }

  const inputPath = readArg("--input", projectCtx.dirs.outputs);
  const report = runOutputGuard({ inputPath, guardCfg: outputGuardCfg });
  const summary = report?.summary || {};
  const hardFailures = Number(summary.hard_failures || 0);
  const softWarnings = Number(summary.soft_warnings || 0);
  const totalFiles = Number(summary.total_files || 0);

  console.log(
    `QA checked ${totalFiles} file(s): hard_failures=${hardFailures}, soft_warnings=${softWarnings}, input=${toRel(resolvePath(inputPath))}`
  );
  if (hardFailures > 0) {
    const failedFiles = (Array.isArray(report?.files) ? report.files : [])
      .filter((f) => Array.isArray(f?.hard_fail_reasons) && f.hard_fail_reasons.length > 0)
      .map((f) => normalizeRel(path.relative(root, String(f.file || ""))));
    if (failedFiles.length) {
      console.log(`Failed files: ${failedFiles.join(", ")}`);
    }
    throw new Error("QA mode found hard failures.");
  }
}

function runArchiveBadMode() {
  const projectCtx = getProjectContext();
  const inputPath = readArg("--input", "");
  if (!inputPath) {
    throw new Error("archive-bad requires --input PATH");
  }

  const inputAbs = resolvePath(inputPath);
  if (!fs.existsSync(inputAbs)) {
    throw new Error(`archive-bad input not found: ${inputPath}`);
  }

  ensureDir(projectCtx.dirs.archiveBad);
  const files = listImageFiles(inputAbs);
  if (!files.length) {
    throw new Error("archive-bad found no image files to move");
  }

  const moved = [];
  for (const abs of files) {
    const archivedAbs = archiveExistingTarget(abs, projectCtx.dirs.archiveBad, "bad");
    if (archivedAbs) {
      moved.push({ from: toRel(abs), to: toRel(archivedAbs) });
    }
  }
  console.log(`Archived bad files: ${moved.length} -> ${toRel(projectCtx.dirs.archiveBad)}`);
}

async function main() {
  loadEnv();

  const mode = args[0] || "dry";
  if (mode === "--help" || mode === "-h") {
    console.log(usage());
    return;
  }

  if (mode === "upscale") {
    runUpscaleOnlyMode();
    return;
  }

  if (mode === "color") {
    runColorOnlyMode();
    return;
  }

  if (mode === "bgremove") {
    await runBgRemoveOnlyMode();
    return;
  }

  if (mode === "qa") {
    runQaMode();
    return;
  }

  if (mode === "archive-bad") {
    runArchiveBadMode();
    return;
  }

  if (!["dry", "run"].includes(mode)) {
    throw new Error(`Unknown mode: ${mode}\n\n${usage()}`);
  }

  await runGenerationMode(mode);
}

main().catch((err) => {
  console.error(err?.message || String(err));
  process.exit(1);
});
