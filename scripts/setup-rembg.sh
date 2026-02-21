#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VENV_DIR="$ROOT_DIR/tools/rembg/.venv"
PYTHON_BIN="${PYTHON_BIN:-python3}"

mkdir -p "$ROOT_DIR/tools/rembg"

if [[ ! -d "$VENV_DIR" ]]; then
  echo "Creating rembg venv: $VENV_DIR"
  "$PYTHON_BIN" -m venv "$VENV_DIR"
fi

PIP_BIN="$VENV_DIR/bin/pip"
PY_BIN="$VENV_DIR/bin/python"

echo "Upgrading pip/setuptools/wheel ..."
"$PIP_BIN" install --upgrade pip setuptools wheel

echo "Installing rembg runtime ..."
"$PIP_BIN" install --upgrade rembg onnxruntime pillow

echo "Validating imports ..."
"$PY_BIN" - <<'PY'
mods = ['rembg', 'onnxruntime', 'PIL']
for m in mods:
    __import__(m)
print('OK: rembg runtime ready')
PY

echo "Interpreter: $PY_BIN"
echo "Test command: $PY_BIN scripts/rembg-remove.py --help"
