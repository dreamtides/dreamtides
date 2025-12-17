# Tabula CLI Observations

- `rebuild-images` now has two modes: default restores `xl/media/*` from `.git/xlsm_image_cache/_xlsm_manifest.json` (manifest v1 required), while `--from-urls` ignores the cache and downloads images from `IMAGE()` formulas.
- URL mode maps IMAGE metadata via `xl/metadata.xml` value metadata → `xl/richData/rdrichvalue.xml` identifiers → `xl/richData/rdRichValueWebImage.xml` entries; each entry pairs an address rel (URL) with a media rel, and every identifier must produce a URL or the command fails.
- Shared IMAGE formulas use the anchor cell’s text and adjust relative references by the row/column offset of each consuming cell, respecting `$`-fixed references; cell values come from calamine ranges.
- Downloads use blocking reqwest and rewrite both `xl/media/*` bytes and `xl/richData/_rels/rdRichValueWebImage.xml.rels` hyperlink targets; the `.git` image cache is unchanged in URL mode.
- Keep running `just fmt`, `just check`, `just clippy`, and `just review` before handoff.
