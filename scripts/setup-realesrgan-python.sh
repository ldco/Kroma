#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VENV_DIR="$ROOT_DIR/tools/realesrgan-python/.venv"
SRC_DIR="$ROOT_DIR/tools/realesrgan-python/src"
BASICSR_DIR="$SRC_DIR/BasicSR"
REALESRGAN_DIR="$SRC_DIR/Real-ESRGAN"
PYTHON_BIN="${PYTHON_BIN:-python3}"
TORCH_INDEX_URL="${TORCH_INDEX_URL:-}"

mkdir -p "$ROOT_DIR/tools/realesrgan-python" "$SRC_DIR"

if [[ ! -d "$VENV_DIR" ]]; then
  echo "Creating virtual environment: $VENV_DIR"
  "$PYTHON_BIN" -m venv "$VENV_DIR"
fi

PIP_BIN="$VENV_DIR/bin/pip"
PY_BIN="$VENV_DIR/bin/python"

echo "Upgrading pip/setuptools/wheel ..."
"$PIP_BIN" install --upgrade pip setuptools wheel

if ! "$PY_BIN" - <<'PY' >/dev/null 2>&1
import torch
import torchvision
PY
then
  echo "Installing torch + torchvision ..."
  if [[ -n "$TORCH_INDEX_URL" ]]; then
    "$PIP_BIN" install torch torchvision --index-url "$TORCH_INDEX_URL"
  else
    "$PIP_BIN" install torch torchvision
  fi
fi

echo "Installing runtime dependencies for BasicSR/Real-ESRGAN source mode ..."
"$PIP_BIN" install --upgrade \
  numpy opencv-python pillow scipy tqdm requests pyyaml addict future lmdb scikit-image tb-nightly yapf

if [[ ! -d "$BASICSR_DIR/.git" ]]; then
  echo "Cloning BasicSR ..."
  git clone https://github.com/XPixelGroup/BasicSR.git "$BASICSR_DIR"
fi

if [[ ! -d "$REALESRGAN_DIR/.git" ]]; then
  echo "Cloning Real-ESRGAN ..."
  git clone https://github.com/xinntao/Real-ESRGAN.git "$REALESRGAN_DIR"
fi

if [[ ! -f "$BASICSR_DIR/basicsr/version.py" ]]; then
  cat > "$BASICSR_DIR/basicsr/version.py" <<'PY'
# Auto-generated for runtime import on Python 3.14 setup workaround.
__version__ = '1.4.2'
__gitsha__ = 'unknown'
version_info = (1, 4, 2)
PY
fi

if [[ ! -f "$REALESRGAN_DIR/realesrgan/version.py" ]]; then
  cat > "$REALESRGAN_DIR/realesrgan/version.py" <<'PY'
# Auto-generated for runtime import on Python 3.14 setup workaround.
__version__ = '0.3.0'
__gitsha__ = 'unknown'
version_info = (0, 3, 0)
PY
fi

echo "Validating runtime imports via source trees ..."
"$PY_BIN" - <<PY
import sys
from pathlib import Path
root = Path(r"$SRC_DIR")
sys.path.insert(0, str(root / 'Real-ESRGAN'))
sys.path.insert(0, str(root / 'BasicSR'))
mods = ['torch', 'torchvision', 'cv2', 'PIL', 'basicsr', 'realesrgan']
for m in mods:
    __import__(m)
print('OK: python backend ready (source mode)')
PY

echo "Python backend installed."
echo "Interpreter: $PY_BIN"
echo "Test command: $PY_BIN scripts/realesrgan-python-upscale.py --help"
