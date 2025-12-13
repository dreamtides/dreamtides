#!/bin/bash
set -e

REPO_ROOT=$(git rev-parse --show-toplevel)
SCRIPT_PATH="$REPO_ROOT/client/scripts/git_xls_filter.py"

if [ ! -f "$SCRIPT_PATH" ]; then
    echo "Error: Filter script not found at $SCRIPT_PATH"
    exit 1
fi

cd "$REPO_ROOT"

RELATIVE_SCRIPT_PATH="client/scripts/git_xls_filter.py"

echo "Setting up Git filter for .xlsm files..."

git config filter.excelimg.clean "python3 $RELATIVE_SCRIPT_PATH clean %f"
git config filter.excelimg.smudge "python3 $RELATIVE_SCRIPT_PATH smudge %f"
git config filter.excelimg.required true

echo "Git filter configured successfully!"
echo ""
echo "To test, add a .xlsm file:"
echo "  git add path/to/file.xlsm"
echo ""
echo "The filter will:"
echo "  - Strip images when committing (stored in Git)"
echo "  - Restore images in your working directory (from cache)"

