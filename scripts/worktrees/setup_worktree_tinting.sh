#!/usr/bin/env bash

set -euo pipefail

# Sets up visual tinting for worktree Unity Editor windows.
#
# What this does:
#   1. Installs Hammerspoon via Homebrew if not present
#   2. Installs the Hammerspoon CLI (hs) if not present
#   3. Creates ~/.hammerspoon/ if needed
#   4. Ensures init.lua has require("hs.ipc") for CLI support
#   5. Adds dofile() to load unity_tint.lua from this repo
#   6. Reloads Hammerspoon config
#
# The C# editor script (WorktreeWindowTitle.cs) is already in the Unity
# project and requires no setup — it activates automatically when Unity
# opens a worktree project.
#
# Safe to run multiple times; idempotent.

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TINT_SCRIPT="$REPO_ROOT/scripts/abu/unity_tint.lua"
HS_DIR="$HOME/.hammerspoon"
HS_INIT="$HS_DIR/init.lua"
DOFILE_LINE="dofile(os.getenv(\"HOME\") .. \"/Documents/GoogleDrive/dreamtides/scripts/abu/unity_tint.lua\")"

# --- Hammerspoon installation ---

if ! brew list --cask hammerspoon &>/dev/null; then
  echo "Installing Hammerspoon..."
  brew install --cask hammerspoon
else
  echo "Hammerspoon already installed."
fi

# Launch Hammerspoon if not running (needed for CLI and IPC)
if ! pgrep -q Hammerspoon; then
  echo "Starting Hammerspoon..."
  open -a Hammerspoon
  sleep 2
fi

# --- Hammerspoon CLI (hs) ---

HS_CLI="/usr/local/bin/hs"
if [ ! -f "$HS_CLI" ]; then
  echo "Installing Hammerspoon CLI..."
  HS_APP="$(mdfind 'kMDItemCFBundleIdentifier == "org.hammerspoon.Hammerspoon"' | head -1)"
  if [ -z "$HS_APP" ]; then
    echo "Warning: Could not find Hammerspoon.app to install CLI. Install manually via Hammerspoon preferences." >&2
  else
    HS_CLI_SOURCE="$HS_APP/Contents/Frameworks/hs/hs"
    if [ -f "$HS_CLI_SOURCE" ]; then
      sudo ln -sf "$HS_CLI_SOURCE" "$HS_CLI"
      echo "Installed hs CLI to $HS_CLI"
    else
      echo "Warning: hs binary not found at $HS_CLI_SOURCE. Install CLI manually via Hammerspoon preferences." >&2
    fi
  fi
else
  echo "Hammerspoon CLI already installed."
fi

# --- init.lua setup ---

mkdir -p "$HS_DIR"

if [ ! -f "$HS_INIT" ]; then
  echo "Creating $HS_INIT..."
  cat > "$HS_INIT" <<'EOF'
-- Enable IPC so the `hs` CLI tool can communicate with Hammerspoon
require("hs.ipc")
EOF
fi

# Ensure require("hs.ipc") is present
if ! grep -q 'require.*hs\.ipc' "$HS_INIT"; then
  echo "Adding hs.ipc require to init.lua..."
  printf '\n-- Enable IPC so the `hs` CLI tool can communicate with Hammerspoon\nrequire("hs.ipc")\n' >> "$HS_INIT"
fi

# Ensure dofile for unity_tint.lua is present
if ! grep -q 'unity_tint\.lua' "$HS_INIT"; then
  echo "Adding unity_tint.lua dofile to init.lua..."
  printf '\n-- Tint worktree Unity Editor windows with a colored strip\n%s\n' "$DOFILE_LINE" >> "$HS_INIT"
else
  echo "unity_tint.lua already loaded in init.lua."
fi

# Verify the tint script exists
if [ ! -f "$TINT_SCRIPT" ]; then
  echo "Error: $TINT_SCRIPT not found. Is the repo cloned correctly?" >&2
  exit 1
fi

# --- Reload ---

if command -v hs &>/dev/null; then
  echo "Reloading Hammerspoon config..."
  hs -c 'hs.reload()' 2>/dev/null || echo "Warning: Could not reload Hammerspoon. Reload manually from the menu bar icon."
else
  echo "Reload Hammerspoon manually from the menu bar icon."
fi

echo ""
echo "Setup complete. Worktree Unity editors will show a colored strip at the top."
echo "Main repo editors are unaffected."
