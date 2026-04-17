#!/usr/bin/env python3

"""Tests for the Dreamsign assignment exporter."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
SCRIPTS_DIR = SCRIPT_DIR.parent.parent
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

import llms.dreamsign_assignment_export as exporter


class NameValidationTests(unittest.TestCase):
    """Tests for Dreamsign name validation."""

    def test_validate_name_allows_two_non_filler_words(self) -> None:
        exporter.validate_name("Bell of Ash")
        exporter.validate_name("Golden Acorn")

    def test_validate_name_rejects_three_non_filler_words(self) -> None:
        with self.assertRaisesRegex(ValueError, "exceeds 2 non-filler words"):
            exporter.validate_name("Bell of Ash Smoke")


class ManifestTests(unittest.TestCase):
    """Tests for art manifest loading."""

    def test_load_art_manifest_backfills_manual_alt_text(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            art_dir = Path(temp_dir)
            (art_dir / "alt_text.txt").write_text(
                "apple.png\tShiny red apple.\n",
                encoding="utf-8",
            )
            (art_dir / "apple.png").write_bytes(b"")
            (art_dir / "hair.png").write_bytes(b"")

            art_manifest = exporter.load_art_manifest(art_dir)

        art_by_file = {entry["source_filename"]: entry for entry in art_manifest}
        self.assertEqual(len(art_manifest), 2)
        self.assertEqual(
            art_by_file["hair.png"]["raw_alt_text"],
            exporter.MANUAL_ALT_TEXT["hair.png"],
        )
        self.assertIn("manual_alt", art_by_file["hair.png"]["ambiguity_flags"])


class ExportTests(unittest.TestCase):
    """Tests for end-to-end export helpers."""

    def test_build_worklist_preserves_source_order(self) -> None:
        effects = [
            {
                "dreamsign_id": "dreamsign_001",
                "source_index": 1,
                "effect_text": "When you discard a card, draw a card.",
            },
            {
                "dreamsign_id": "dreamsign_002",
                "source_index": 2,
                "effect_text": "At each Shop, gain 50 essence.",
            },
        ]
        art_manifest = [
            {
                "source_filename": "bag.png",
                "raw_alt_text": "Brown satchel.",
                "canonical_object": "satchel",
                "motif_tags": ["container"],
                "ambiguity_flags": [],
            },
            {
                "source_filename": "feather.png",
                "raw_alt_text": "Black feather.",
                "canonical_object": "feather",
                "motif_tags": ["feather"],
                "ambiguity_flags": [],
            },
        ]
        assignments = [
            {
                "dreamsign_id": "dreamsign_002",
                "effect_text": "At each Shop, gain 50 essence.",
                "name": "Brown Satchel",
                "image_file": "bag.png",
                "justification": "A satchel carries bargains.",
            },
            {
                "dreamsign_id": "dreamsign_001",
                "effect_text": "When you discard a card, draw a card.",
                "name": "Black Feather",
                "image_file": "feather.png",
                "justification": "A feather drifts out and another follows.",
            },
        ]

        rows = exporter.build_worklist(
            effects=effects,
            art_manifest=art_manifest,
            assignments=assignments,
        )

        self.assertEqual(
            [row["dreamsign_id"] for row in rows], ["dreamsign_001", "dreamsign_002"]
        )
        self.assertEqual(rows[0]["effect_tags"], ["discard", "draw"])
        self.assertEqual(rows[1]["effect_tags"], ["essence", "shop"])

    def test_validate_assignments_requires_unique_images(self) -> None:
        effects = [
            {
                "dreamsign_id": "dreamsign_001",
                "source_index": 1,
                "effect_text": "One.",
            },
            {
                "dreamsign_id": "dreamsign_002",
                "source_index": 2,
                "effect_text": "Two.",
            },
        ]
        art_manifest = [
            {
                "source_filename": "one.png",
                "raw_alt_text": "One.",
                "canonical_object": "one",
                "motif_tags": ["oddity"],
                "ambiguity_flags": [],
            },
            {
                "source_filename": "two.png",
                "raw_alt_text": "Two.",
                "canonical_object": "two",
                "motif_tags": ["oddity"],
                "ambiguity_flags": [],
            },
        ]
        assignments = [
            {
                "dreamsign_id": "dreamsign_001",
                "effect_text": "One.",
                "name": "First One",
                "image_file": "one.png",
                "justification": "One.",
            },
            {
                "dreamsign_id": "dreamsign_002",
                "effect_text": "Two.",
                "name": "Second One",
                "image_file": "one.png",
                "justification": "Two.",
            },
        ]

        with self.assertRaisesRegex(ValueError, "Duplicate image assignment"):
            exporter.validate_assignments(
                effects=effects,
                art_manifest=art_manifest,
                assignments=assignments,
            )


class SerializationTests(unittest.TestCase):
    """Tests for output serialization helpers."""

    def test_write_jsonl_emits_one_object_per_line(self) -> None:
        rows = [{"name": "A"}, {"name": "B"}]
        with tempfile.TemporaryDirectory() as temp_dir:
            path = Path(temp_dir) / "rows.jsonl"
            exporter.write_jsonl(path, rows)
            lines = path.read_text(encoding="utf-8").splitlines()

        self.assertEqual([json.loads(line) for line in lines], rows)


if __name__ == "__main__":
    unittest.main()
