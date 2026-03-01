#!/usr/bin/env bash
set -euo pipefail

# =========================================================
# CONFIG
# =========================================================

REAL_HOME="$HOME"
UNIVERSES_ROOT="${REAL_HOME}/.vscode-universes"
PROJECT_DIR="$(pwd)"
VSCODE_DIR="${PROJECT_DIR}/.vscode"
MARKER_FILE="${VSCODE_DIR}/universe.json"

QWEN_EXTENSION_ID="qwenlm.qwen-code-vscode-ide-companion"

mkdir -p "$UNIVERSES_ROOT"
mkdir -p "$VSCODE_DIR"

log() {
  echo -e "🚀 $*"
}

# =========================================================
# RESOLVE UNIVERSE NAME
# =========================================================

if [[ $# -ge 1 ]]; then
  UNIVERSE_NAME="$1"

  log "Saving universe marker"

  cat > "$MARKER_FILE" <<EOF
{
  "universe": "$UNIVERSE_NAME"
}
EOF

elif [[ -f "$MARKER_FILE" ]]; then
  UNIVERSE_NAME="$(grep -oP '"universe"\s*:\s*"\K[^"]+' "$MARKER_FILE" || true)"

  if [[ -z "${UNIVERSE_NAME}" ]]; then
    echo "❌ Failed to read universe name from .vscode/universe.json"
    exit 1
  fi
else
  echo "❌ Universe not set."
  echo "👉 First run: ./codeu.sh <universe-name>"
  exit 1
fi

# =========================================================
# PATHS
# =========================================================

UNIVERSE_DIR="${UNIVERSES_ROOT}/${UNIVERSE_NAME}"
ISOLATED_HOME="${UNIVERSE_DIR}/home"
EXT_DIR="${UNIVERSE_DIR}/extensions"
USER_DATA_DIR="${UNIVERSE_DIR}"

log "Universe: $UNIVERSE_NAME"
log "Project: $PROJECT_DIR"

mkdir -p "$UNIVERSE_DIR"
mkdir -p "$ISOLATED_HOME"
mkdir -p "$EXT_DIR"

# =========================================================
# 🔥 Mirror core zsh entry files
# =========================================================

for f in .zshrc .zprofile .zlogin .zshenv .p10k.zsh; do
  if [[ -f "$REAL_HOME/$f" && ! -e "$ISOLATED_HOME/$f" ]]; then
    ln -s "$REAL_HOME/$f" "$ISOLATED_HOME/$f"
  fi
done

# =========================================================
# 🔥 Bridge selected user resources (SAFE SHARED ZONE)
# =========================================================

BRIDGE_DIRS=(
  ".oh-my-zsh"
  ".config"
  ".local"
  ".cache/starship"
  ".zinit"
  ".zplug"
  ".zgen"
  ".zgenom"
  ".zsh_plugins"
  ".zim"
  ".nvm"
  ".pyenv"
  ".codex"        # ⭐ THIS is what you need
)

for dir in "${BRIDGE_DIRS[@]}"; do
  if [[ -e "$REAL_HOME/$dir" && ! -e "$ISOLATED_HOME/$dir" ]]; then
    ln -s "$REAL_HOME/$dir" "$ISOLATED_HOME/$dir" 2>/dev/null || true
  fi
done

# =========================================================
# 🔧 Seed VS Code settings (project-local)
# =========================================================

SETTINGS_FILE="${VSCODE_DIR}/settings.json"

if [[ ! -f "$SETTINGS_FILE" ]]; then
  log "Creating project VS Code settings"

  cat > "$SETTINGS_FILE" <<'EOF'
{
  "terminal.integrated.defaultProfile.linux": "zsh",
  "terminal.integrated.profiles.linux": {
    "zsh": {
      "path": "/usr/bin/zsh",
      "args": ["-l"]
    }
  },
  "window.newWindowDimensions": "maximized"
}
EOF
fi

# =========================================================
# 📦 Install Qwen extension if missing
# =========================================================

if ! ls "${EXT_DIR}/${QWEN_EXTENSION_ID}-"* >/dev/null 2>&1; then
  log "Installing Qwen extension into universe"

  code \
    --user-data-dir "$USER_DATA_DIR" \
    --extensions-dir "$EXT_DIR" \
    --install-extension "$QWEN_EXTENSION_ID" \
    --force >/dev/null 2>&1 || true
else
  log "Qwen extension already present"
fi

# =========================================================
# 🚀 Launch VS Code (fully isolated)
# =========================================================

log "Launching VS Code"

exec env HOME="$ISOLATED_HOME" \
  ELECTRON_USER_DATA_DIR="$USER_DATA_DIR" \
  code \
  --new-window \
  --user-data-dir "$USER_DATA_DIR" \
  --extensions-dir "$EXT_DIR" \
  "$PROJECT_DIR"
