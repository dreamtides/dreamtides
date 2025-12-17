# Tabula CLI Observations

- git-setup installs pre-commit, post-checkout, post-merge, and post-commit hooks that call `tabula git-hook <hook>` (falling back to a workspace cargo run); pre-commit runs build-toml, blocks when any Tabula/*.toml file is newer than Tabula.xlsm (suggesting build-xls), strips images in place, and stages the XLSM plus the TOML directory so LFS only sees placeholder media.
- Checkout/merge/commit hooks rebuild images with `rebuild-images --auto`, restoring from the cache then falling back to IMAGE() downloads so working copies retain images while the repo stores stripped XLSM files with cache-backed manifests in `.git/xlsm_image_cache`.
- git-setup populates `.gitattributes` to track `client/Assets/StreamingAssets/Tabula.xlsm` via Git LFS and creates the image cache directory under `.git/`.
- Validate keeps AppleScript as unimplemented; `--strip-images` validation strips to a temp copy, round-trips TOML, rebuilds from cache, then diffs workbook snapshots and xl/media bytes.
