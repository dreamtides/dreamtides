#!/usr/bin/env python3
"""Validate and export Dreamsign image/name assignments."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_EFFECTS_PATH = REPO_ROOT / "notes/dreamsigns.txt"
DEFAULT_ASSIGNMENTS_PATH = REPO_ROOT / "notes/dreamsign-assignments.json"
DEFAULT_WORKLIST_PATH = REPO_ROOT / "notes/dreamsign-assignment-worklist.jsonl"
DEFAULT_ART_MANIFEST_PATH = REPO_ROOT / "notes/dreamsign-art-manifest.json"
DEFAULT_ART_DIR = Path("/Users/dthurn/Documents/dreamsigns/filtered")
FILLER_WORDS = {"a", "an", "of", "the"}
MANUAL_ALT_TEXT = {
    "hair.png": "Bundled lock of brown hair tied at one end.",
}
CANONICAL_OBJECT_OVERRIDES = {
    "Champignon.png": "shell token",
    "Magicwand.png": "wand",
    "Ring_2.png": "badge",
    "White tortoise shell.png": "tortoise shell",
    "aertfact.png": "sigil panel",
    "bag_2.png": "satchel",
    "bag_3.png": "glow pouch",
    "bag_4.png": "charm pouch",
    "bat_wings.png": "bat wing",
    "bettle.png": "beetle",
    "black-acorn.png": "dark acorn",
    "black_widow.png": "spider jar",
    "bone.png": "bone flute",
    "book_10.png": "beast codex",
    "book_11.png": "tide codex",
    "book_2.png": "storm codex",
    "book_3.png": "eye codex",
    "book_5.png": "death codex",
    "book_6.png": "serpent codex",
    "book_8.png": "field journal",
    "book_9.png": "whorl codex",
    "bowl.png": "mortar",
    "cactus .png": "cactus",
    "dark-essense.png": "shadow droplet",
    "dead_rat.png": "dead rat",
    "dreamcatcher.png": "dreamcatcher",
    "egg_dragon.png": "ember egg",
    "eggs_2.png": "spotted egg",
    "eye_3.png": "gold eye",
    "ginger .png": "ginger root",
    "horn_rainbow .png": "rainbow horn",
    "key_2.png": "silver key",
    "key_3.png": "gold key",
    "magic-ball.png": "cracked orb",
    "magic_ball.png": "eye orb",
    "magic_ball_2.png": "prism orb",
    "magic_stick_2.png": "ribbon wand",
    "magic_stick_3.png": "root staff",
    "magic_stick_6.png": "charm staff",
    "mask_2.png": "theater mask",
    "mortar_pestle.png": "herb mortar",
    "note_2.png": "notes",
    "philosopher's_Stone.png": "philosopher stone",
    "pig_nouse.png": "hollow cap",
    "powder_magic.png": "powder bowl",
    "rabbit_tail.png": "swirl tuft",
    "rainbow_jellyfish.png": "jelly veil",
    "raven_skull.png": "raven skull",
    "red_cheese.png": "ember wedge",
    "rose_petals.png": "rose swirl",
    "scroll_3.png": "sealed scroll",
    "shell 2.png": "spiral shell",
    "skull_2.png": "human skull",
    "speaces.png": "green ooze",
    "star_anise.png": "star pod",
    "stone-red.png": "red shard",
    "stone_2.png": "glow shard",
    "strange_fruit.png": "spike fruit",
    "teeaths.png": "paired teeth",
    "white_rat.png": "white rat",
    "witch_coin.png": "witch coin",
}
GENERIC_ALT_PATTERNS = (
    "multiple fantasy item icons",
    "orb",
    "stone",
    "gem",
    "crystal sphere",
)
EFFECT_TAG_RULES = {
    "discard": ("discard",),
    "cost_reduction": ("costs", "cost "),
    "character": ("character", "characters"),
    "event": ("event", "events"),
    "fast": ("↯fast", "fast "),
    "materialize": ("materialize", "materialized"),
    "abandon": ("abandon", "abandoned"),
    "banish": ("banish", "banished"),
    "warrior": ("warrior", "warriors"),
    "figment": ("figment", "figments"),
    "judgment": ("judgment",),
    "copy": ("copy", "copied"),
    "void": ("void",),
    "reclaim": ("reclaim",),
    "kindle": ("kindle",),
    "foresee": ("foresee",),
    "draw": ("draw",),
    "spark": ("spark",),
    "dissolve": ("dissolve", "dissolved"),
    "energy": ("gain 1●", "gain 2●", "maximum ●", "your ●"),
    "score": ("score", "points"),
    "shop": ("shop", "purchase", "buy"),
    "transfigure": ("transfiguration", "transfigure"),
    "essence": ("essence",),
    "draft": ("draft", "picks"),
    "purge": ("purge",),
    "duplicate": ("duplicate", "duplication"),
    "dreamsign": ("dreamsign",),
    "enhance": ("enhanced",),
    "journey": ("dream journey",),
    "bane": ("bane", "nightmare"),
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--effects-path",
        type=Path,
        default=DEFAULT_EFFECTS_PATH,
    )
    parser.add_argument(
        "--assignments-path",
        type=Path,
        default=DEFAULT_ASSIGNMENTS_PATH,
    )
    parser.add_argument(
        "--art-dir",
        type=Path,
        default=DEFAULT_ART_DIR,
    )
    parser.add_argument(
        "--worklist-path",
        type=Path,
        default=DEFAULT_WORKLIST_PATH,
    )
    parser.add_argument(
        "--art-manifest-path",
        type=Path,
        default=DEFAULT_ART_MANIFEST_PATH,
    )
    return parser.parse_args()


def normalize_slug(text: str) -> str:
    return re.sub(r"-{2,}", "-", re.sub(r"[^a-z0-9]+", "-", text.lower())).strip("-")


def extract_name_tokens(name: str) -> list[str]:
    return re.findall(r"[A-Za-z0-9']+", name.lower())


def non_filler_word_count(name: str) -> int:
    return sum(1 for token in extract_name_tokens(name) if token not in FILLER_WORDS)


def validate_name(name: str) -> None:
    if not name.strip():
        raise ValueError("Dreamsign name cannot be blank")
    if non_filler_word_count(name) > 2:
        raise ValueError(f"Dreamsign name exceeds 2 non-filler words: {name}")


def load_effects(path: Path) -> list[dict[str, Any]]:
    effects = []
    for index, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        text = line.strip()
        if not text:
            raise ValueError(f"Blank effect line at {index}")
        effects.append(
            {
                "dreamsign_id": f"dreamsign_{index:03d}",
                "source_index": index,
                "effect_text": text,
            }
        )
    return effects


def load_alt_text_map(art_dir: Path) -> dict[str, str]:
    alt_path = art_dir / "alt_text.txt"
    alt_map: dict[str, str] = {}
    for line in alt_path.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        file_name, description = line.split("\t", 1)
        alt_map[file_name] = description.strip()
    alt_map.update(MANUAL_ALT_TEXT)
    return alt_map


def infer_canonical_object(file_name: str, description: str) -> str:
    if file_name in CANONICAL_OBJECT_OVERRIDES:
        return CANONICAL_OBJECT_OVERRIDES[file_name]
    stem = Path(file_name).stem
    stem = re.sub(r"[_\s-]*\d+$", "", stem)
    stem = stem.replace("_", " ").replace("-", " ").strip()
    if stem:
        return stem.lower()
    return description.split(" with ", 1)[0].split(" on ", 1)[0].lower()


def infer_family_key(file_name: str) -> str:
    name = Path(file_name).stem.lower().strip()
    name = name.replace(" ", "_").replace("-", "_")
    name = re.sub(r"_?\d+$", "", name)
    if name.startswith("magic_ball"):
        return "magic_ball"
    if name.startswith("magic_stick"):
        return "magic_stick"
    return name


def infer_motif_tags(description: str, canonical_object: str) -> list[str]:
    haystack = f"{canonical_object} {description}".lower()
    tags: set[str] = set()
    rules = {
        "animal": (
            "rat",
            "cat",
            "crow",
            "raven",
            "piranha",
            "toad",
            "scorpion",
            "spider",
            "beetle",
            "fly",
            "worm",
            "larva",
            "fish",
        ),
        "bone": ("bone", "skull", "teeth", "claw", "horn"),
        "plant": (
            "leaf",
            "flower",
            "herb",
            "mushroom",
            "fruit",
            "seed",
            "root",
            "garlic",
            "ginger",
            "rosemary",
            "petals",
            "cactus",
        ),
        "crystal": ("crystal", "gem", "shard", "stone", "opal", "orb"),
        "container": (
            "bag",
            "satchel",
            "pouch",
            "vial",
            "bottle",
            "cauldron",
            "mortar",
            "bowl",
            "goblet",
        ),
        "weapon": ("knife", "dagger", "sickle", "wand", "staff"),
        "token": (
            "coin",
            "badge",
            "rune",
            "amulet",
            "pendant",
            "talisman",
            "medallion",
        ),
        "text": ("book", "scroll", "parchment", "letter", "notes", "card"),
        "dream": ("dream", "eye", "mirror", "mask"),
        "shell": ("shell", "urchin", "clam", "mollusk"),
        "feather": ("feather", "wing"),
        "time": ("hourglass",),
        "essence": ("essence", "potion", "powder"),
    }
    for tag, needles in rules.items():
        if any(needle in haystack for needle in needles):
            tags.add(tag)
    if not tags:
        tags.add("oddity")
    return sorted(tags)


def infer_ambiguity_flags(
    file_name: str,
    description: str,
    family_key: str,
    has_manual_alt: bool,
) -> list[str]:
    flags = []
    normalized = file_name.strip()
    if normalized != file_name or " " in Path(file_name).stem:
        flags.append("filename_format")
    if file_name in MANUAL_ALT_TEXT and has_manual_alt:
        flags.append("manual_alt")
    if any(pattern in description.lower() for pattern in GENERIC_ALT_PATTERNS):
        flags.append("generic_alt")
    if family_key in {
        "amulet",
        "book",
        "feather",
        "magic_ball",
        "magic_stick",
        "shell",
        "stone",
    }:
        flags.append("duplicate_family")
    if file_name in {
        "aertfact.png",
        "bettle.png",
        "dark-essense.png",
        "pig_nouse.png",
        "speaces.png",
        "teeaths.png",
    }:
        flags.append("filename_typo")
    return flags


def load_art_manifest(art_dir: Path) -> list[dict[str, Any]]:
    alt_map = load_alt_text_map(art_dir)
    art = []
    for file_name in sorted(
        path.name
        for path in art_dir.iterdir()
        if path.is_file()
        and path.name != "alt_text.txt"
        and not path.name.startswith(".")
    ):
        if file_name not in alt_map:
            raise ValueError(f"Missing alt text for {file_name}")
        description = alt_map[file_name]
        canonical_object = infer_canonical_object(file_name, description)
        family_key = infer_family_key(file_name)
        art.append(
            {
                "art_id": normalize_slug(Path(file_name).stem),
                "source_filename": file_name,
                "raw_alt_text": description,
                "canonical_object": canonical_object,
                "motif_tags": infer_motif_tags(description, canonical_object),
                "family_key": family_key,
                "ambiguity_flags": infer_ambiguity_flags(
                    file_name=file_name,
                    description=description,
                    family_key=family_key,
                    has_manual_alt=file_name in MANUAL_ALT_TEXT,
                ),
            }
        )
    return art


def load_assignments(path: Path) -> list[dict[str, Any]]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if isinstance(payload, list):
        return payload
    if isinstance(payload, dict) and isinstance(payload.get("assignments"), list):
        return payload["assignments"]
    raise ValueError("Assignments JSON must be a list or an object with assignments")


def infer_effect_tags(effect_text: str) -> list[str]:
    haystack = effect_text.lower()
    tags = [
        tag
        for tag, needles in EFFECT_TAG_RULES.items()
        if any(needle in haystack for needle in needles)
    ]
    return sorted(tags)


def infer_confidence(ambiguity_flags: list[str], effect_tags: list[str]) -> str:
    if "manual_alt" in ambiguity_flags or len(ambiguity_flags) >= 2:
        return "medium"
    if len(effect_tags) <= 1:
        return "medium"
    return "high"


def validate_assignments(
    *,
    effects: list[dict[str, Any]],
    art_manifest: list[dict[str, Any]],
    assignments: list[dict[str, Any]],
) -> None:
    effect_by_id = {effect["dreamsign_id"]: effect for effect in effects}
    art_by_file = {entry["source_filename"]: entry for entry in art_manifest}
    if len(assignments) != len(effects):
        raise ValueError(
            f"Expected {len(effects)} assignments, found {len(assignments)}"
        )

    seen_ids: set[str] = set()
    seen_images: set[str] = set()
    seen_names: set[str] = set()
    for assignment in assignments:
        dreamsign_id = assignment["dreamsign_id"]
        image_file = assignment["image_file"]
        name = assignment["name"]
        effect_text = assignment["effect_text"]
        justification = assignment["justification"]

        if dreamsign_id not in effect_by_id:
            raise ValueError(f"Unknown dreamsign id: {dreamsign_id}")
        if image_file not in art_by_file:
            raise ValueError(f"Unknown image file: {image_file}")
        if effect_text != effect_by_id[dreamsign_id]["effect_text"]:
            raise ValueError(f"Effect text mismatch for {dreamsign_id}")
        if not justification.strip():
            raise ValueError(f"Blank justification for {dreamsign_id}")
        validate_name(name)
        normalized_name = name.lower().strip()
        if normalized_name in seen_names:
            raise ValueError(f"Duplicate dreamsign name: {name}")
        if dreamsign_id in seen_ids:
            raise ValueError(f"Duplicate dreamsign id: {dreamsign_id}")
        if image_file in seen_images:
            raise ValueError(f"Duplicate image assignment: {image_file}")
        seen_ids.add(dreamsign_id)
        seen_images.add(image_file)
        seen_names.add(normalized_name)

    if seen_ids != set(effect_by_id):
        raise ValueError("Assignments do not cover every dreamsign id")
    if seen_images != set(art_by_file):
        raise ValueError("Assignments do not cover every image file")


def build_worklist(
    *,
    effects: list[dict[str, Any]],
    art_manifest: list[dict[str, Any]],
    assignments: list[dict[str, Any]],
) -> list[dict[str, Any]]:
    effect_by_id = {effect["dreamsign_id"]: effect for effect in effects}
    art_by_file = {entry["source_filename"]: entry for entry in art_manifest}
    rows = []
    for assignment in assignments:
        effect = effect_by_id[assignment["dreamsign_id"]]
        art = art_by_file[assignment["image_file"]]
        effect_tags = infer_effect_tags(effect["effect_text"])
        rows.append(
            {
                "dreamsign_id": assignment["dreamsign_id"],
                "source_index": effect["source_index"],
                "effect_text": effect["effect_text"],
                "effect_tags": effect_tags,
                "image_file": assignment["image_file"],
                "image_alt_text": art["raw_alt_text"],
                "canonical_object": art["canonical_object"],
                "motif_tags": art["motif_tags"],
                "proposed_name": assignment["name"],
                "justification": assignment["justification"],
                "confidence": infer_confidence(art["ambiguity_flags"], effect_tags),
                "alternate_image_files": [],
                "review_status": "approved",
                "ambiguity_flags": art["ambiguity_flags"],
            }
        )
    return sorted(rows, key=lambda row: row["source_index"])


def write_json(path: Path, payload: Any) -> None:
    path.write_text(
        json.dumps(payload, indent=2, ensure_ascii=True) + "\n", encoding="utf-8"
    )


def write_jsonl(path: Path, rows: list[dict[str, Any]]) -> None:
    path.write_text(
        "".join(json.dumps(row, ensure_ascii=True) + "\n" for row in rows),
        encoding="utf-8",
    )


def main() -> int:
    args = parse_args()
    effects = load_effects(args.effects_path)
    art_manifest = load_art_manifest(args.art_dir)
    assignments = load_assignments(args.assignments_path)
    validate_assignments(
        effects=effects,
        art_manifest=art_manifest,
        assignments=assignments,
    )
    art_payload = {
        "art_directory": str(args.art_dir),
        "count": len(art_manifest),
        "art": art_manifest,
    }
    worklist = build_worklist(
        effects=effects,
        art_manifest=art_manifest,
        assignments=assignments,
    )
    write_json(args.art_manifest_path, art_payload)
    write_jsonl(args.worklist_path, worklist)
    return 0


if __name__ == "__main__":
    sys.exit(main())
