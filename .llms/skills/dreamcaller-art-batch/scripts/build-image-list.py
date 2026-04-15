#!/usr/bin/env python3
"""Write the list of dreamcaller image IDs (basenames without .png) to /tmp."""

from pathlib import Path

SRC = Path.home() / "Documents" / "synty" / "dreamcallers"
OUT = Path("/tmp/dreamcaller-batch-images.txt")

ids = sorted(p.stem for p in SRC.glob("*.png"))
OUT.write_text("\n".join(ids) + "\n")
print(f"Wrote {len(ids)} image IDs to {OUT}")
