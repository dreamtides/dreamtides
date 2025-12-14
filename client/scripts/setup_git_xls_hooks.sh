#!/bin/bash
set -e

REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT"

echo "Setting up Git hooks for Tabula.xlsm..."

git config core.hooksPath .githooks

chmod +x .githooks/* client/scripts/*.py 2>/dev/null || true

XLSM_DIR="client/Assets/StreamingAssets/Tabula.xlsm.d"
XLSM_FILE="client/Assets/StreamingAssets/Tabula.xlsm"

if [[ -d "$XLSM_DIR" ]]; then
    echo "Generating Tabula.xlsm from extracted directory..."
    python3 client/scripts/xlsm_pack.py "$XLSM_DIR" "$XLSM_FILE" --overwrite
    echo "Tabula.xlsm generated successfully!"
fi

echo ""
echo "Git hooks configured successfully!"
echo ""
echo "Workflow:"
echo "  - Edit Tabula.xlsm"
echo "  - git commit → pre-commit hook extracts to Tabula.xlsm.d/ and stages changes"
echo "  - git checkout/pull → post-* hooks regenerate Tabula.xlsm from Tabula.xlsm.d/"

