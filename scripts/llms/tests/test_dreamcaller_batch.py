#!/usr/bin/env python3

"""Tests for the dreamcaller batch runner."""

from __future__ import annotations

import asyncio
import tempfile
import time
import sys
import unittest
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
LLMS_DIR = SCRIPT_DIR.parent
if str(LLMS_DIR) not in sys.path:
    sys.path.insert(0, str(LLMS_DIR))

import dreamcaller_batch

VALID_RESULT = {
    "theme": "tempo",
    "brainstorm_pool": [
        {
            "id": 1,
            "ability_idea": "Once per turn, when you attack, draw a card.",
            "interesting_note": "Simple test concept.",
            "support_estimate": {
                "approximate_cards": 8,
                "bucket": "Medium",
                "basis": "Enough attack payoffs exist.",
            },
            "novelty_test": {"passes": False, "notes": "Obvious design."},
            "quality_gates": {
                "theme_fit": "High",
                "draft_pull": "Draft attackers.",
                "simplicity": "High",
            },
            "is_obvious_design": True,
            "uses_battlefield_position": False,
            "hearthstone_source": None,
            "selected_for_final": True,
        }
    ],
    "final_designs": [
        {
            "id": 1,
            "source_brainstorm_id": 1,
            "ability_text": "Once per turn, when you attack, draw a card.",
            "ability_type": "Triggered",
            "design_rationale": "Supports attacking decks.",
            "synergy_citations": [{"card": "Card A", "note": "Attack payoff."}],
            "support_estimate": {
                "approximate_cards": 8,
                "bucket": "Medium",
                "basis": "Enough attack payoffs exist.",
            },
            "novelty_statement": "No existing card does this as a dreamcaller.",
            "inspiration_source": "Test fixture",
            "tags": {
                "obvious_design": True,
                "hearthstone_inspired": False,
                "positional": False,
            },
        }
    ],
    "selection_notes": {
        "selected_brainstorm_ids": [1],
        "cut_brainstorm_ids": [],
        "constraints_satisfied": {
            "obvious_design_count": 1,
            "novel_design_count": 0,
            "has_hearthstone_inspired_design": False,
            "has_positional_design": False,
            "ability_type_mix": ["Triggered"],
        },
    },
}


class PromptFileTests(unittest.TestCase):
    """Tests for parsing newline-delimited prompts."""

    def test_load_prompts_skips_blank_lines(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            prompt_path = Path(temp_dir) / "prompts.txt"
            prompt_path.write_text("tempo\n\n discard matters \n", encoding="utf-8")

            prompts = dreamcaller_batch.load_prompts(prompt_path)

        self.assertEqual(prompts, ["tempo", "discard matters"])

    def test_load_prompts_rejects_duplicates(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            prompt_path = Path(temp_dir) / "prompts.txt"
            prompt_path.write_text("tempo\ntempo\n", encoding="utf-8")

            with self.assertRaisesRegex(ValueError, "Duplicate prompt"):
                dreamcaller_batch.load_prompts(prompt_path)


class ValidationTests(unittest.TestCase):
    """Tests for surface-level JSON validation."""

    def test_validate_result_accepts_expected_shape(self) -> None:
        errors = dreamcaller_batch.validate_dreamcaller_result(VALID_RESULT)
        self.assertEqual(errors, [])

    def test_validate_result_reports_missing_top_level_key(self) -> None:
        invalid_result = dict(VALID_RESULT)
        invalid_result.pop("selection_notes")

        errors = dreamcaller_batch.validate_dreamcaller_result(invalid_result)

        self.assertIn("Missing top-level key: selection_notes", errors)

    def test_parse_and_validate_agent_output_extracts_claude_structured_output(
        self,
    ) -> None:
        raw_text = (
            '{"type":"result","subtype":"success","structured_output":'
            + dreamcaller_batch.json.dumps(VALID_RESULT)
            + "}"
        )

        parsed_json, errors = dreamcaller_batch.parse_and_validate_agent_output(
            raw_text
        )

        self.assertEqual(parsed_json, VALID_RESULT)
        self.assertEqual(errors, [])


class SynthesisTests(unittest.TestCase):
    """Tests for synthesized output shape."""

    def test_synthesize_results_omits_failed_agent_payloads(self) -> None:
        synthesis = dreamcaller_batch.synthesize_results(
            prompts=["tempo"],
            results_by_prompt={
                "tempo": {
                    "codex": dreamcaller_batch.AgentAttemptResult(
                        agent_name="codex",
                        prompt="tempo",
                        success=True,
                        parsed_json=VALID_RESULT,
                        errors=[],
                        used_retry=False,
                    ),
                    "claude": dreamcaller_batch.AgentAttemptResult(
                        agent_name="claude",
                        prompt="tempo",
                        success=False,
                        parsed_json=None,
                        errors=["Invalid JSON"],
                        used_retry=True,
                    ),
                }
            },
        )

        self.assertEqual(synthesis["tempo"]["prompt"], "tempo")
        self.assertIn("codex", synthesis["tempo"]["agents"])
        self.assertNotIn("claude", synthesis["tempo"]["agents"])
        self.assertEqual(
            synthesis["tempo"]["verification"]["claude"]["errors"], ["Invalid JSON"]
        )
        self.assertTrue(synthesis["tempo"]["verification"]["claude"]["used_retry"])


class SubprocessTests(unittest.TestCase):
    """Tests for subprocess handling edge cases."""

    def test_run_command_returns_after_parent_exit(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            helper_path = Path(temp_dir) / "spawn_child.py"
            helper_path.write_text(
                "\n".join(
                    [
                        "#!/usr/bin/env python3",
                        "import subprocess",
                        "import sys",
                        'print("parent-finished")',
                        "sys.stdout.flush()",
                        "subprocess.Popen(",
                        "    [sys.executable, '-c', 'import time; time.sleep(2)'],",
                        "    stdout=sys.stdout,",
                        "    stderr=sys.stderr,",
                        ")",
                    ]
                )
                + "\n",
                encoding="utf-8",
            )

            start = time.monotonic()
            stdout, stderr, exit_code = asyncio.run(
                dreamcaller_batch._run_command(
                    [sys.executable, str(helper_path)],
                    timeout_seconds=1,
                )
            )
            elapsed = time.monotonic() - start

        self.assertEqual(exit_code, 0)
        self.assertEqual(stderr, "")
        self.assertIn("parent-finished", stdout)
        self.assertLess(elapsed, 1.0)

    def test_build_codex_command_avoids_schema_flag(self) -> None:
        command = dreamcaller_batch.build_codex_command(
            prompt_text="Return JSON only.",
            output_path=Path("/tmp/codex-test.json"),
            codex_bin="codex",
        )

        self.assertNotIn("--output-schema", command)
        self.assertIn("-o", command)


if __name__ == "__main__":
    unittest.main()
