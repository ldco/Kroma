#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TOOLS_DIR="$ROOT_DIR/tools/realesrgan"
BIN_PATH="$TOOLS_DIR/realesrgan-ncnn-vulkan"

mkdir -p "$TOOLS_DIR"

echo "Checking Real-ESRGAN release metadata..."
RELEASES_JSON="$(curl -fsSL 'https://api.github.com/repos/xinntao/Real-ESRGAN/releases?per_page=20')"

readarray -t RELEASE_INFO < <(RELEASES_JSON="$RELEASES_JSON" python3 - <<'PY'
import json
import os
import sys
rels=json.loads(os.environ["RELEASES_JSON"])
chosen=None
for r in rels:
    for a in r.get("assets", []):
        n=a.get("name", "").lower()
        if "realesrgan-ncnn-vulkan" in n and "ubuntu" in n:
            chosen=(r.get("tag_name", ""), r.get("published_at", ""), a.get("name", ""), a.get("browser_download_url", ""))
            break
    if chosen:
        break
if not chosen:
    sys.exit(1)
for value in chosen:
    print(value)
PY
)

if [[ ${#RELEASE_INFO[@]} -ne 4 ]]; then
  echo "Could not resolve a Linux ncnn release asset for Real-ESRGAN"
  exit 1
fi

TAG="${RELEASE_INFO[0]}"
PUBLISHED="${RELEASE_INFO[1]}"
ASSET_NAME="${RELEASE_INFO[2]}"
ASSET_URL="${RELEASE_INFO[3]}"

echo "Resolved release: $TAG ($PUBLISHED)"
echo "Asset: $ASSET_NAME"

if [[ -x "$BIN_PATH" ]]; then
  echo "Binary already exists: $BIN_PATH"
  "$BIN_PATH" -h | sed -n '1,2p' || true
  exit 0
fi

ARCHIVE_PATH="/tmp/${ASSET_NAME}"

echo "Downloading $ASSET_NAME ..."
curl -L "$ASSET_URL" -o "$ARCHIVE_PATH"

echo "Installing to $TOOLS_DIR ..."
unzip -o "$ARCHIVE_PATH" -d "$TOOLS_DIR" >/tmp/realesrgan_setup_unzip.log
chmod +x "$BIN_PATH"

echo "Installed: $BIN_PATH"
"$BIN_PATH" -h | sed -n '1,2p' || true
