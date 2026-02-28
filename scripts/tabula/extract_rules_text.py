#!/usr/bin/env python3

import json
import tomllib
import sys
from pathlib import Path


def main():
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    toml_path = project_root / "Assets" / "StreamingAssets" / "Tabula" / "cards.toml"
    output_path = project_root / "rules_text.json"

    if not toml_path.exists():
        print(f"Error: {toml_path} not found", file=sys.stderr)
        sys.exit(1)

    with open(toml_path, "rb") as f:
        data = tomllib.load(f)

    rules_texts = []
    if "cards" in data:
        for card in data["cards"]:
            if "rules-text" in card:
                rules_text = card["rules-text"]
                if rules_text:
                    rules_texts.append(rules_text.strip())

    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(rules_texts, f, indent=2, ensure_ascii=False)

    print(f"Extracted {len(rules_texts)} rules-text entries to {output_path}")


if __name__ == "__main__":
    main()
