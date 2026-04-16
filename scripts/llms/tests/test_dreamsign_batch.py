#!/usr/bin/env python3

"""Tests for the Dreamsign batch runner."""

from __future__ import annotations

import asyncio
import argparse
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

SCRIPT_DIR = Path(__file__).resolve().parent
SCRIPTS_DIR = SCRIPT_DIR.parent.parent
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

import llms.dreamsign_batch as dreamsign_batch

VALID_RESULT = {
    "dreamsign": "Ashwake Compass",
    "type": "Battle",
    "ability_text": "Your first tide-linked event each turn costs 1 less.",
    "justification": "This sharpens tide bridge decks without becoming a full engine.",
    "rejected_alternatives": [
        "A void-recursion rebate was cleaner on paper but duplicated Dreamcaller scale payoffs.",
        "A quest shop discount line created a second idea instead of reinforcing the battle plan.",
    ],
}


class PlanningTests(unittest.TestCase):
    """Tests for deterministic job planning."""

    def test_build_saturated_chunks_uses_all_items_once_when_possible(self) -> None:
        items = [f"Item {index}" for index in range(7)]

        chunks = dreamsign_batch._build_saturated_chunks(
            items,
            chunk_count=3,
            min_per_chunk=1,
            rng=dreamsign_batch.random.Random("test"),
        )

        self.assertEqual(sum(len(chunk) for chunk in chunks), 7)
        self.assertEqual(
            sorted(item for chunk in chunks for item in chunk), sorted(items)
        )
        self.assertEqual(sorted(len(chunk) for chunk in chunks), [2, 2, 3])

    def test_build_saturated_chunks_duplicates_minimally_when_needed(self) -> None:
        items = [f"Item {index}" for index in range(5)]

        chunks = dreamsign_batch._build_saturated_chunks(
            items,
            chunk_count=7,
            min_per_chunk=1,
            rng=dreamsign_batch.random.Random("test"),
        )

        flattened = [item for chunk in chunks for item in chunk]
        self.assertEqual(len(chunks), 7)
        self.assertEqual(len(flattened), 7)
        self.assertEqual(set(flattened), set(items))
        self.assertTrue(all(len(chunk) == 1 for chunk in chunks))

    def test_provider_counts_for_mode_require_even_split(self) -> None:
        with self.assertRaisesRegex(ValueError, "divide evenly"):
            dreamsign_batch.provider_counts_for_mode(55)

    def test_build_jobs_balances_modes_and_providers(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = Path(temp_dir)
            dreamcallers_path = temp_path / "dreamcallers.toml"
            dreamcallers_path.write_text(
                "\n".join(
                    [
                        "[[dreamcaller]]",
                        'name = "A"',
                        'title = "One"',
                        'rendered-text = "Alpha"',
                        'mandatory-tides = ["tide_a"]',
                        'optional-tides = ["tide_b"]',
                        "",
                        "[[dreamcaller]]",
                        'name = "B"',
                        'title = "Two"',
                        'rendered-text = "Beta"',
                        'mandatory-tides = ["tide_c"]',
                        'optional-tides = ["tide_d"]',
                        "",
                    ]
                )
                + "\n",
                encoding="utf-8",
            )
            commanders_path = temp_path / "commanders.txt"
            commanders_path.write_text(
                "\n".join(f"Commander line {index}" for index in range(1, 17)) + "\n",
                encoding="utf-8",
            )
            sts_fight_path = temp_path / "relics-fight.txt"
            sts_fight_path.write_text(
                "\n".join(f"STS fight {index}" for index in range(1, 17)) + "\n",
                encoding="utf-8",
            )
            mt_battle_path = temp_path / "artifacts-battle.txt"
            mt_battle_path.write_text(
                "\n".join(f"MT battle {index}" for index in range(1, 17)) + "\n",
                encoding="utf-8",
            )
            mt_map_path = temp_path / "artifacts-map.txt"
            mt_map_path.write_text(
                "\n".join(f"MT map {index}" for index in range(1, 9)) + "\n",
                encoding="utf-8",
            )
            sts_other_path = temp_path / "relics-other.txt"
            sts_other_path.write_text(
                "\n".join(f"STS other {index}" for index in range(1, 17)) + "\n",
                encoding="utf-8",
            )

            args = argparse.Namespace(
                seed=7,
                codex_model="gpt-5.2-codex",
                claude_model="opus",
                mode1_count=2,
                mode2_count=4,
                mode3a_count=4,
                mode3b_count=4,
                mode2_pool_size=2,
                mode3a_sts_pool_size=1,
                mode3a_mt_pool_size=1,
                mode3b_sts_pool_size=2,
                mode3b_mt_pool_size=1,
                dreamcallers_path=dreamcallers_path,
                commanders_path=commanders_path,
                sts_fight_path=sts_fight_path,
                mt_battle_path=mt_battle_path,
                mt_map_path=mt_map_path,
                sts_other_path=sts_other_path,
            )

            jobs = dreamsign_batch.build_jobs(args)

        self.assertEqual(len(jobs), 14)
        self.assertEqual(sum(1 for job in jobs if job.agent_name == "codex"), 7)
        self.assertEqual(sum(1 for job in jobs if job.agent_name == "claude"), 7)
        self.assertEqual(sum(1 for job in jobs if job.mode == "mode1"), 2)
        self.assertEqual(sum(1 for job in jobs if job.mode == "mode2"), 4)
        self.assertEqual(sum(1 for job in jobs if job.mode == "mode3a"), 4)
        self.assertEqual(sum(1 for job in jobs if job.mode == "mode3b"), 4)
        self.assertEqual(len({job.job_key for job in jobs}), 14)

    def test_build_mode_specs_requires_exact_dreamcaller_count(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = Path(temp_dir)
            dreamcallers_path = temp_path / "dreamcallers.toml"
            dreamcallers_path.write_text(
                "\n".join(
                    [
                        "[[dreamcaller]]",
                        'name = "A"',
                        'title = "One"',
                        'rendered-text = "Alpha"',
                        'mandatory-tides = ["tide_a"]',
                        'optional-tides = ["tide_b"]',
                        "",
                    ]
                )
                + "\n",
                encoding="utf-8",
            )
            filler = temp_path / "filler.txt"
            filler.write_text(
                "\n".join(f"Line {index}" for index in range(1, 10)) + "\n"
            )

            args = argparse.Namespace(
                seed=1,
                mode1_count=2,
                mode2_count=2,
                mode3a_count=2,
                mode3b_count=2,
                mode2_pool_size=1,
                mode3a_sts_pool_size=1,
                mode3a_mt_pool_size=1,
                mode3b_sts_pool_size=1,
                mode3b_mt_pool_size=1,
                dreamcallers_path=dreamcallers_path,
                commanders_path=filler,
                sts_fight_path=filler,
                mt_battle_path=filler,
                mt_map_path=filler,
                sts_other_path=filler,
            )

            with self.assertRaisesRegex(ValueError, "must match dreamcaller count"):
                dreamsign_batch.build_mode_specs(args)

    def test_build_jobs_allows_mode3b_only_run(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = Path(temp_dir)
            dreamcallers_path = temp_path / "dreamcallers.toml"
            dreamcallers_path.write_text(
                "\n".join(
                    [
                        "[[dreamcaller]]",
                        'name = "A"',
                        'title = "One"',
                        'rendered-text = "Alpha"',
                        'mandatory-tides = ["tide_a"]',
                        'optional-tides = ["tide_b"]',
                        "",
                    ]
                )
                + "\n",
                encoding="utf-8",
            )
            commanders_path = temp_path / "commanders.txt"
            commanders_path.write_text("Commander 1\nCommander 2\n", encoding="utf-8")
            sts_fight_path = temp_path / "relics-fight.txt"
            sts_fight_path.write_text("STS fight 1\n", encoding="utf-8")
            mt_battle_path = temp_path / "artifacts-battle.txt"
            mt_battle_path.write_text("MT battle 1\n", encoding="utf-8")
            mt_map_path = temp_path / "artifacts-map.txt"
            mt_map_path.write_text("MT map 1\nMT map 2\n", encoding="utf-8")
            sts_other_path = temp_path / "relics-other.txt"
            sts_other_path.write_text(
                "STS other 1\nSTS other 2\nSTS other 3\n",
                encoding="utf-8",
            )

            args = argparse.Namespace(
                seed=5,
                codex_model="gpt-5.4",
                claude_model="opus",
                mode1_count=0,
                mode2_count=0,
                mode3a_count=0,
                mode3b_count=4,
                mode2_min_per_job=1,
                mode3a_sts_min_per_job=1,
                mode3a_mt_min_per_job=1,
                mode3b_sts_min_per_job=1,
                mode3b_mt_min_per_job=1,
                dreamcallers_path=dreamcallers_path,
                commanders_path=commanders_path,
                sts_fight_path=sts_fight_path,
                mt_battle_path=mt_battle_path,
                mt_map_path=mt_map_path,
                sts_other_path=sts_other_path,
                avoid_ability_texts_path=None,
            )

            jobs = dreamsign_batch.build_jobs(args)

        self.assertEqual(len(jobs), 4)
        self.assertTrue(all(job.mode == "mode3b" for job in jobs))
        self.assertEqual(sum(1 for job in jobs if job.agent_name == "codex"), 2)
        self.assertEqual(sum(1 for job in jobs if job.agent_name == "claude"), 2)


class ValidationTests(unittest.TestCase):
    """Tests for output validation."""

    def test_validate_result_accepts_expected_shape(self) -> None:
        errors = dreamsign_batch.validate_dreamsign_result(VALID_RESULT)
        self.assertEqual(errors, [])

    def test_parse_and_validate_agent_output_extracts_structured_output(self) -> None:
        raw_text = (
            '{"type":"result","subtype":"success","structured_output":'
            + dreamsign_batch.json.dumps(VALID_RESULT)
            + "}"
        )

        parsed_json, errors = dreamsign_batch.parse_and_validate_agent_output(raw_text)

        self.assertEqual(parsed_json, VALID_RESULT)
        self.assertEqual(errors, [])


class CommandTests(unittest.TestCase):
    """Tests for command construction."""

    def test_build_agent_prompt_includes_avoid_list(self) -> None:
        prompt = dreamsign_batch.build_agent_prompt(
            dreamsign_batch.AgentJob(
                agent_name="codex",
                model_name="gpt-5.4",
                job_key="mode3b-001-codex-test",
                mode="mode3b",
                source_paths=("map.txt", "other.txt"),
                source_items=("Map effect", "Other effect"),
                avoid_ability_texts=("Existing line one.", "Existing line two."),
            ),
            "skill text",
        )

        self.assertIn(
            "Mode: Mode 3B: Monster Train / Slay the Spire Inspired, Quest-Level",
            prompt,
        )
        self.assertIn("Existing Dreamsign ability texts to avoid", prompt)
        self.assertIn("Existing line one.", prompt)
        self.assertIn("Existing line two.", prompt)

    def test_build_codex_command_includes_model_and_schema(self) -> None:
        command = dreamsign_batch.build_codex_command(
            prompt_text="Return JSON only.",
            output_path=Path("/tmp/codex-test.json"),
            output_schema_path=Path("/tmp/schema.json"),
            codex_bin="codex",
            model_name="gpt-5.2-codex",
        )

        self.assertIn("--model", command)
        self.assertIn("gpt-5.2-codex", command)
        self.assertIn("--output-schema", command)
        self.assertIn("/tmp/schema.json", command)

    def test_build_claude_command_includes_model(self) -> None:
        command = dreamsign_batch.build_claude_command(
            prompt_text="Return JSON only.",
            claude_bin="claude",
            model_name="opus",
        )

        self.assertIn("--model", command)
        self.assertIn("opus", command)


class JobRunTests(unittest.TestCase):
    """Tests for provider execution handling."""

    def test_run_job_ignores_codex_stderr_when_exit_is_zero(self) -> None:
        async def fake_run_codex(**_: object) -> tuple[str, str, int]:
            return json.dumps(VALID_RESULT), "OpenAI Codex v0.118.0", 0

        with tempfile.TemporaryDirectory() as temp_dir:
            with mock.patch.object(
                dreamsign_batch,
                "_run_codex",
                side_effect=fake_run_codex,
            ):
                result = asyncio.run(
                    dreamsign_batch._run_job(
                        dreamsign_batch.AgentJob(
                            agent_name="codex",
                            model_name="gpt-5.2-codex",
                            job_key="mode2-001-codex-test",
                            mode="mode2",
                            source_paths=("commanders.txt",),
                            source_items=("Commander A", "Commander B"),
                        ),
                        skill_text="skill",
                        output_schema_path=Path(temp_dir) / "schema.json",
                        temp_dir=Path(temp_dir),
                        codex_timeout_seconds=10,
                        claude_timeout_seconds=10,
                        codex_bin="codex",
                        claude_bin="claude",
                    )
                )

        self.assertTrue(result.success)
        self.assertEqual(result.errors, [])
        self.assertEqual(result.parsed_json, VALID_RESULT)


class RecoveryTests(unittest.TestCase):
    """Tests for recovering completed work from provider logs."""

    def test_extract_job_key(self) -> None:
        self.assertEqual(
            dreamsign_batch.extract_job_key("Job key: mode2-001-codex-test\nMore text"),
            "mode2-001-codex-test",
        )

    def test_recover_claude_results_matches_job_key(self) -> None:
        job = dreamsign_batch.AgentJob(
            agent_name="claude",
            model_name="opus",
            job_key="mode3b-001-claude-test",
            mode="mode3b",
            source_paths=("a.txt", "b.txt"),
            source_items=("One", "Two"),
        )
        with tempfile.TemporaryDirectory() as temp_dir:
            log_path = Path(temp_dir) / "claude-session.jsonl"
            log_path.write_text(
                "\n".join(
                    [
                        json.dumps(
                            {
                                "type": "queue-operation",
                                "operation": "enqueue",
                                "content": (
                                    "Job key: mode3b-001-claude-test\n"
                                    "Mode: Mode 3B: Monster Train / Slay the Spire Inspired, Quest-Level"
                                ),
                            }
                        ),
                        json.dumps(
                            {
                                "type": "attachment",
                                "attachment": {
                                    "type": "structured_output",
                                    "data": VALID_RESULT,
                                },
                            }
                        ),
                    ]
                )
                + "\n",
                encoding="utf-8",
            )

            recovered = dreamsign_batch.recover_claude_results(
                jobs=[job],
                claude_dir=Path(temp_dir),
            )

        self.assertIn(job.job_key, recovered)
        self.assertEqual(recovered[job.job_key].parsed_json, VALID_RESULT)


if __name__ == "__main__":
    unittest.main()
