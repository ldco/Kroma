#!/usr/bin/env python3
"""
Cross-platform runtime setup helper.

Replaces bash-based setup scripts so Linux/macOS/Windows can bootstrap the
local toolchain with Python only.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import stat
import subprocess
import sys
import tempfile
import urllib.request
import zipfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
TOOLS_DIR = ROOT / "tools"


def _is_windows() -> bool:
    return os.name == "nt"


def _venv_python(venv_dir: Path) -> Path:
    if _is_windows():
        return venv_dir / "Scripts" / "python.exe"
    return venv_dir / "bin" / "python"


def _run(cmd: list[str], *, cwd: Path | None = None, env: dict[str, str] | None = None):
    print("+", " ".join(cmd))
    subprocess.run(cmd, cwd=str(cwd) if cwd else None, env=env, check=True)


def _ensure_venv(venv_dir: Path, python_bin: str):
    if not venv_dir.exists():
        venv_dir.parent.mkdir(parents=True, exist_ok=True)
        _run([python_bin, "-m", "venv", str(venv_dir)])


def _pip_install(venv_dir: Path, packages: list[str], *, upgrade: bool = True):
    py = _venv_python(venv_dir)
    cmd = [str(py), "-m", "pip", "install"]
    if upgrade:
        cmd.append("--upgrade")
    cmd.extend(packages)
    _run(cmd)


def _ensure_git_available():
    if shutil.which("git"):
        return
    raise SystemExit("git is required for realesrgan-python setup but was not found in PATH")


def _write_if_missing(path: Path, content: str):
    if not path.exists():
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")


def _download_json(url: str) -> dict | list:
    req = urllib.request.Request(url, headers={"User-Agent": "kroma-setup-tools"})
    with urllib.request.urlopen(req, timeout=30) as resp:  # noqa: S310
        raw = resp.read().decode("utf-8")
    return json.loads(raw)


def _download_file(url: str, dest: Path):
    req = urllib.request.Request(url, headers={"User-Agent": "kroma-setup-tools"})
    with urllib.request.urlopen(req, timeout=120) as resp:  # noqa: S310
        dest.write_bytes(resp.read())


def _pick_ncnn_asset(assets: list[dict]) -> dict | None:
    platform_tokens: list[str]
    if sys.platform.startswith("linux"):
        platform_tokens = ["ubuntu", "linux"]
    elif sys.platform == "darwin":
        platform_tokens = ["macos", "osx", "mac"]
    elif _is_windows():
        platform_tokens = ["windows", "win"]
    else:
        platform_tokens = []

    candidates = []
    for a in assets:
        name = str(a.get("name", "")).lower()
        if "realesrgan-ncnn-vulkan" not in name:
            continue
        if not name.endswith(".zip"):
            continue
        score = 0
        for token in platform_tokens:
            if token in name:
                score += 1
        candidates.append((score, a))

    if not candidates:
        return None
    candidates.sort(key=lambda x: x[0], reverse=True)
    return candidates[0][1]


def setup_realesrgan_ncnn(args):
    target_dir = TOOLS_DIR / "realesrgan"
    target_dir.mkdir(parents=True, exist_ok=True)
    exe_name = "realesrgan-ncnn-vulkan.exe" if _is_windows() else "realesrgan-ncnn-vulkan"
    existing = target_dir / exe_name
    if existing.exists():
        print(f"Real-ESRGAN ncnn already present: {existing}")
        return

    releases = _download_json("https://api.github.com/repos/xinntao/Real-ESRGAN/releases?per_page=30")
    if not isinstance(releases, list):
        raise SystemExit("Unexpected GitHub API response for releases")

    chosen_asset = None
    chosen_tag = ""
    for rel in releases:
        assets = rel.get("assets", [])
        picked = _pick_ncnn_asset(assets if isinstance(assets, list) else [])
        if picked:
            chosen_asset = picked
            chosen_tag = str(rel.get("tag_name", "unknown"))
            break
    if not chosen_asset:
        raise SystemExit(f"Could not find matching Real-ESRGAN ncnn asset for platform={sys.platform}")

    asset_name = str(chosen_asset.get("name", "asset.zip"))
    asset_url = str(chosen_asset.get("browser_download_url", ""))
    if not asset_url:
        raise SystemExit("Chosen release asset has no download URL")
    print(f"Resolved Real-ESRGAN release {chosen_tag}: {asset_name}")

    with tempfile.TemporaryDirectory(prefix="kroma_realesrgan_") as td:
        archive = Path(td) / asset_name
        _download_file(asset_url, archive)
        with zipfile.ZipFile(archive, "r") as zf:
            zf.extractall(target_dir)

    found = list(target_dir.rglob(exe_name))
    if not found:
        raise SystemExit(f"Installed archive but executable '{exe_name}' not found under {target_dir}")

    exe = found[0]
    final = target_dir / exe_name
    if exe.resolve() != final.resolve():
        if final.exists():
            final.unlink()
        shutil.move(str(exe), str(final))

    if not _is_windows():
        mode = final.stat().st_mode
        final.chmod(mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    print(f"Installed Real-ESRGAN ncnn: {final}")


def setup_realesrgan_python(args):
    _ensure_git_available()
    venv_dir = TOOLS_DIR / "realesrgan-python" / ".venv"
    src_dir = TOOLS_DIR / "realesrgan-python" / "src"
    basicsr_dir = src_dir / "BasicSR"
    realesrgan_dir = src_dir / "Real-ESRGAN"
    _ensure_venv(venv_dir, args.python_bin)

    _pip_install(venv_dir, ["pip", "setuptools", "wheel"])
    env = os.environ.copy()
    py = _venv_python(venv_dir)

    try:
        _run([str(py), "-c", "import torch, torchvision"])
    except subprocess.CalledProcessError:
        torch_cmd = [str(py), "-m", "pip", "install", "--upgrade", "torch", "torchvision"]
        torch_index = (args.torch_index_url or os.environ.get("TORCH_INDEX_URL") or "").strip()
        if torch_index:
            torch_cmd.extend(["--index-url", torch_index])
        _run(torch_cmd)

    _pip_install(
        venv_dir,
        [
            "numpy",
            "opencv-python",
            "pillow",
            "scipy",
            "tqdm",
            "requests",
            "pyyaml",
            "addict",
            "future",
            "lmdb",
            "scikit-image",
            "tb-nightly",
            "yapf",
        ],
    )

    if not (basicsr_dir / ".git").exists():
        basicsr_dir.parent.mkdir(parents=True, exist_ok=True)
        _run(["git", "clone", "https://github.com/XPixelGroup/BasicSR.git", str(basicsr_dir)])
    if not (realesrgan_dir / ".git").exists():
        realesrgan_dir.parent.mkdir(parents=True, exist_ok=True)
        _run(["git", "clone", "https://github.com/xinntao/Real-ESRGAN.git", str(realesrgan_dir)])

    _write_if_missing(
        basicsr_dir / "basicsr" / "version.py",
        "__version__='1.4.2'\n__gitsha__='unknown'\nversion_info=(1,4,2)\n",
    )
    _write_if_missing(
        realesrgan_dir / "realesrgan" / "version.py",
        "__version__='0.3.0'\n__gitsha__='unknown'\nversion_info=(0,3,0)\n",
    )

    validate = (
        "import sys;"
        f"sys.path.insert(0, r'{str((src_dir / 'Real-ESRGAN').resolve())}');"
        f"sys.path.insert(0, r'{str((src_dir / 'BasicSR').resolve())}');"
        "import torch, torchvision, cv2, PIL, basicsr, realesrgan; "
        "print('OK: realesrgan python runtime ready')"
    )
    _run([str(py), "-c", validate], env=env)
    print(f"Interpreter: {py}")


def setup_rembg(args):
    venv_dir = TOOLS_DIR / "rembg" / ".venv"
    _ensure_venv(venv_dir, args.python_bin)
    _pip_install(venv_dir, ["pip", "setuptools", "wheel"])
    _pip_install(venv_dir, ["rembg", "onnxruntime", "pillow"])
    py = _venv_python(venv_dir)
    _run([str(py), "-c", "import rembg, onnxruntime, PIL; print('OK: rembg runtime ready')"])
    print(f"Interpreter: {py}")


def build_parser():
    p = argparse.ArgumentParser(description="Cross-platform local runtime setup")
    p.add_argument("--python-bin", default=os.environ.get("PYTHON_BIN", "python3"), help="Python interpreter to bootstrap venvs")
    p.add_argument("--torch-index-url", default="", help="Optional torch wheel index URL")
    sub = p.add_subparsers(dest="cmd", required=True)

    sub.add_parser("realesrgan-ncnn", help="Install Real-ESRGAN ncnn binary + models for current platform").set_defaults(
        func=setup_realesrgan_ncnn
    )
    sub.add_parser("realesrgan-python", help="Install Real-ESRGAN python runtime in tools/").set_defaults(
        func=setup_realesrgan_python
    )
    sub.add_parser("rembg", help="Install rembg runtime in tools/").set_defaults(func=setup_rembg)

    def _setup_all(args):
        setup_rembg(args)
        setup_realesrgan_python(args)
        setup_realesrgan_ncnn(args)

    sub.add_parser("all", help="Install rembg + realesrgan runtimes").set_defaults(func=_setup_all)
    return p


def main():
    args = build_parser().parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
