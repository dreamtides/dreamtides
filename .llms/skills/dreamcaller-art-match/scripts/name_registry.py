#!/usr/bin/env python3

import argparse
import datetime as dt
import json
import re
import sys
from pathlib import Path

import fcntl


REGISTRY_PATH = (
    Path("/tmp")
    / "dreamcaller_art_match_registry.json"
)
WORD_PATTERN = re.compile(r"[A-Za-z0-9]+(?:'[A-Za-z0-9]+)?")
DEFAULT_SOFT_CAP = 3
DEFAULT_HARD_CAP = 5
STOP_WORDS = {
    "a",
    "an",
    "and",
    "for",
    "of",
    "the",
    "to",
}


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Track unique dreamcaller names for the dreamcaller-art-match skill."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    check_parser = subparsers.add_parser("check", help="Check whether a name is unused.")
    check_parser.add_argument("--name", required=True, help='Candidate like "Proper Name, Title".')

    ability_check_parser = subparsers.add_parser(
        "check-ability",
        help="Check whether an ability is below the configured usage caps.",
    )
    ability_check_parser.add_argument(
        "--ability",
        required=True,
        help='Exact or near-exact ability text to evaluate.',
    )
    ability_check_parser.add_argument(
        "--soft-cap",
        type=int,
        default=DEFAULT_SOFT_CAP,
        help="Preferred maximum number of prior uses before agents should avoid the ability.",
    )
    ability_check_parser.add_argument(
        "--hard-cap",
        type=int,
        default=DEFAULT_HARD_CAP,
        help="Maximum number of prior uses before the ability is banned.",
    )

    claim_parser = subparsers.add_parser("claim", help="Reserve a unique dreamcaller name.")
    claim_parser.add_argument("--name", required=True, help='Final choice like "Proper Name, Title".')
    claim_parser.add_argument("--image", default="", help="Image path or label for provenance.")
    claim_parser.add_argument("--ability", default="", help="Short ability excerpt for provenance.")

    status_parser = subparsers.add_parser("status", help="Show registry stats.")
    status_parser.add_argument(
        "--limit",
        type=int,
        default=10,
        help="How many recent claimed names to print.",
    )
    status_parser.add_argument(
        "--ability-limit",
        type=int,
        default=10,
        help="How many ability counts to print.",
    )

    args = parser.parse_args()
    if args.command == "check":
        return run_check(args.name)
    if args.command == "check-ability":
        return run_check_ability(args.ability, args.soft_cap, args.hard_cap)
    if args.command == "claim":
        return run_claim(args.name, args.image, args.ability)
    return run_status(args.limit, args.ability_limit)


def run_check(name: str) -> int:
    with locked_registry() as registry:
        return print_check_result(name, registry)


def run_check_ability(ability: str, soft_cap: int, hard_cap: int) -> int:
    with locked_registry() as registry:
        return print_ability_check_result(ability, registry, soft_cap, hard_cap)


def run_claim(name: str, image: str, ability: str) -> int:
    with locked_registry() as registry:
        if print_check_result(name, registry) != 0:
            return 1
        if print_ability_check_result(ability, registry, DEFAULT_SOFT_CAP, DEFAULT_HARD_CAP) != 0:
            return 1
        registry["entries"].append(
            {
                "timestamp_utc": dt.datetime.now(dt.UTC).isoformat().replace("+00:00", "Z"),
                "name": collapse_whitespace(name),
                "normalized_name": normalize_name(name),
                "words": extract_words(name),
                "image": image,
                "normalized_image": normalize_image_label(image),
                "ability": ability,
                "normalized_ability": normalize_ability(ability),
            }
        )
        print(f'claimed: "{collapse_whitespace(name)}"')
        return 0


def run_status(limit: int, ability_limit: int) -> int:
    with locked_registry() as registry:
        entries = registry["entries"]
        used_words = sorted({word for entry in entries for word in entry_words(entry)})
        print(f"registry: {REGISTRY_PATH}")
        print(f"claimed names: {len(entries)}")
        print(f"used words: {len(used_words)}")
        print("top abilities:")
        for ability, count in ability_usage_counts(entries)[:ability_limit]:
            print(f"{count}x {ability}")
        print("recent names:")
        for entry in entries[-limit:]:
            print(entry["name"])
        return 0


def print_check_result(name: str, registry: dict) -> int:
    candidate_words = extract_words(name)
    if not candidate_words:
        print("conflict: candidate has no usable words after normalization")
        return 1

    normalized_name = normalize_name(name)
    existing_names = {entry_normalized_name(entry) for entry in registry["entries"]}
    if normalized_name in existing_names:
        print(f'conflict: full name already claimed: "{collapse_whitespace(name)}"')
        return 1

    used_words = {word for entry in registry["entries"] for word in entry_words(entry)}
    overlapping_words = sorted(set(candidate_words) & used_words)
    if overlapping_words:
        print("conflict: reused words: " + ", ".join(overlapping_words))
        return 1

    print("ok: name is unique")
    print("words: " + ", ".join(candidate_words))
    return 0


def print_ability_check_result(
    ability: str,
    registry: dict,
    soft_cap: int,
    hard_cap: int,
) -> int:
    normalized_ability = normalize_ability(ability)
    if not normalized_ability:
        print("conflict: ability has no usable words after normalization")
        return 2

    prior_uses = ability_usage_count(registry["entries"], normalized_ability)
    print(f"ability prior uses: {prior_uses}")
    print(f"soft cap: {soft_cap}")
    print(f"hard cap: {hard_cap}")

    if prior_uses >= hard_cap:
        print("conflict: ability has reached the hard cap and is banned")
        return 2

    if prior_uses >= soft_cap:
        print("warning: ability has reached the soft cap and should be avoided")
        return 0

    print("ok: ability is below the soft cap")
    return 0


class locked_registry:
    def __enter__(self) -> dict:
        REGISTRY_PATH.parent.mkdir(parents=True, exist_ok=True)
        self.handle = REGISTRY_PATH.open("a+", encoding="utf-8")
        fcntl.flock(self.handle.fileno(), fcntl.LOCK_EX)
        self.handle.seek(0)
        content = self.handle.read()
        self.registry = json.loads(content) if content.strip() else {"version": 1, "entries": []}
        self.registry.setdefault("version", 1)
        self.registry.setdefault("entries", [])
        return self.registry

    def __exit__(self, exc_type, exc, tb) -> None:
        self.handle.seek(0)
        self.handle.truncate()
        json.dump(self.registry, self.handle, indent=2, ensure_ascii=True)
        self.handle.write("\n")
        self.handle.flush()
        fcntl.flock(self.handle.fileno(), fcntl.LOCK_UN)
        self.handle.close()


def extract_words(name: str) -> list[str]:
    sanitized = collapse_whitespace(name).replace("-", " ").lower()
    return [word for word in WORD_PATTERN.findall(sanitized) if word not in STOP_WORDS]


def extract_ability_words(ability: str) -> list[str]:
    return WORD_PATTERN.findall(collapse_whitespace(ability).lower())


def normalize_name(name: str) -> str:
    return " ".join(extract_words(name))


def entry_words(entry: dict) -> list[str]:
    return extract_words(entry.get("name", ""))


def entry_normalized_name(entry: dict) -> str:
    return normalize_name(entry.get("name", ""))


def normalize_ability(ability: str) -> str:
    return " ".join(extract_ability_words(ability))


def ability_usage_count(entries: list[dict], normalized_ability: str) -> int:
    return len(
        {
            entry_normalized_image(entry)
            for entry in entries
            if entry_normalized_ability(entry) == normalized_ability and entry_normalized_image(entry)
        }
    )


def ability_usage_counts(entries: list[dict]) -> list[tuple[str, int]]:
    counts: dict[str, tuple[str, set[str]]] = {}
    for entry in entries:
        ability = collapse_whitespace(entry.get("ability", ""))
        normalized_ability = entry_normalized_ability(entry)
        normalized_image = entry_normalized_image(entry)
        if not normalized_ability:
            continue
        existing = counts.get(normalized_ability)
        if existing is None:
            counts[normalized_ability] = (ability, {normalized_image} if normalized_image else set())
        else:
            if normalized_image:
                existing[1].add(normalized_image)
    return sorted(
        ((ability, len(images)) for ability, images in counts.values()),
        key=lambda item: (-item[1], item[0]),
    )


def entry_normalized_ability(entry: dict) -> str:
    return entry.get("normalized_ability", normalize_ability(entry.get("ability", "")))


def entry_normalized_image(entry: dict) -> str:
    return entry.get("normalized_image", normalize_image_label(entry.get("image", "")))


def normalize_image_label(image: str) -> str:
    collapsed = collapse_whitespace(image)
    if not collapsed:
        return ""
    return Path(collapsed).name.lower()


def collapse_whitespace(value: str) -> str:
    return " ".join(value.split())


if __name__ == "__main__":
    sys.exit(main())
