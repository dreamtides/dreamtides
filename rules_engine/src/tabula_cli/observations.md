# Tabula CLI Observations

- git-setup now treats `Tabula.xlsm` as an ignored working copy and `TabulaData.xlsm` as the LFS-tracked stripped artifact: pre-commit validates TOML freshness, runs build-toml from `Tabula.xlsm`, strips images into `TabulaData.xlsm`, stages `TabulaData.xlsm` plus TOMLs, and verifies staged content is placeholder-only (skipping when the index holds an LFS pointer).
- Checkout/merge hooks copy `TabulaData.xlsm` to `Tabula.xlsm` and run `rebuild-images --auto`, restoring images from cache/URLs so local edits always work while the repo stores stripped data. Post-commit hook was removed.
- git-setup writes `.gitignore` to ignore `client/Assets/StreamingAssets/Tabula.xlsm` and `.gitattributes` to track `client/Assets/StreamingAssets/TabulaData.xlsm` via LFS, ensuring only stripped content hits the index.
- Validate no longer exposes an AppleScript option; `--strip-images` validation strips to a temp copy, round-trips TOML, rebuilds from cache, then diffs workbook snapshots and xl/media bytes.
- The AppleScriptTask helper `tabula_server_curl.scpt` is compiled from `tabula_server_curl.applescript`, exposes `tabula_server_request`, expects `url|request_path|response_path`, posts via `/usr/bin/curl --data-binary @request`, writes the response file, and returns `ok|<http_code>` or `error|<number>|<message>`.
