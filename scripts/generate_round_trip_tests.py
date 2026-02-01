#!/usr/bin/env python3
"""
Generates round-trip tests for cards.toml with hardcoded rules text strings.

This script reads cards.toml and generates a Rust test file where each test
has the rules_text and variables embedded directly as string literals,
avoiding any runtime dependency on cards.toml.
"""

import re
import tomllib
from pathlib import Path


def sanitize_name(name: str) -> str:
    """Convert card name to a valid Rust function name."""
    # Replace non-alphanumeric with underscore
    sanitized = re.sub(r'[^a-zA-Z0-9]', '_', name.lower())
    # Collapse multiple underscores
    sanitized = re.sub(r'_+', '_', sanitized)
    # Strip leading/trailing underscores
    return sanitized.strip('_')


def escape_rust_string(s: str) -> str:
    """Escape a string for use in a Rust string literal."""
    s = s.replace('\\', '\\\\')
    s = s.replace('"', '\\"')
    s = s.replace('\n', '\\n')
    return s


def generate_test(card_name: str, rules_text: str, variables: str) -> str:
    """Generate a single test function."""
    fn_name = f"test_round_trip_card_{sanitize_name(card_name)}"

    # Handle multiline strings
    rules_text = rules_text.strip()
    variables = variables.strip() if variables else ""

    # Split rules_text into ability blocks (separated by blank lines)
    ability_blocks = [block.strip() for block in rules_text.split('\n\n') if block.strip()]

    lines = ["#[test]", f"fn {fn_name}() {{"]

    vars_escaped = escape_rust_string(variables)

    for block in ability_blocks:
        rules_escaped = escape_rust_string(block)
        lines.append(f'    assert_round_trip("{rules_escaped}", "{vars_escaped}");')

    lines.append("}")
    return '\n'.join(lines)


def main():
    project_root = Path(__file__).parent.parent
    cards_toml_path = project_root / "rules_engine" / "tabula" / "cards.toml"
    output_path = project_root / "rules_engine" / "tests" / "parser_v2_tests" / "tests" / "cards_toml_round_trip_tests.rs"

    # Read cards.toml
    with open(cards_toml_path, "rb") as f:
        data = tomllib.load(f)

    cards = data.get("cards", [])

    tests = []
    for card in cards:
        name = card.get("name", "").strip()
        rules_text = card.get("rules-text", "")
        variables = card.get("variables", "") or ""

        if not name or not rules_text:
            continue

        test_code = generate_test(name, rules_text, variables)
        tests.append(test_code)

    # Generate the output file
    output = '''//! Per-card round-trip tests for cards.toml.
//!
//! Each test verifies that a card's ability text round-trips
//! correctly through parse -> serialize.
//!
//! GENERATED FILE - Do not edit manually.
//! Regenerate with: python scripts/generate_round_trip_tests.py

use parser_v2_tests::test_helpers::*;

'''

    output += '\n\n'.join(tests)
    output += '\n'

    with open(output_path, "w") as f:
        f.write(output)

    print(f"Generated {len(tests)} tests to {output_path}")


if __name__ == "__main__":
    main()
