#!/usr/bin/env python3

"""Tests for the dreamcaller batch runner."""

from __future__ import annotations

import asyncio
import io
import json
import tempfile
import time
import sys
import unittest
from unittest import mock
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

    def test_run_job_ignores_codex_stderr_when_exit_is_zero(self) -> None:
        async def fake_run_codex(**_: object) -> tuple[str, str, int]:
            return json.dumps(VALID_RESULT), "OpenAI Codex v0.118.0", 0

        with tempfile.TemporaryDirectory() as temp_dir:
            with mock.patch.object(
                dreamcaller_batch,
                "_run_codex",
                side_effect=fake_run_codex,
            ):
                result = asyncio.run(
                    dreamcaller_batch._run_job(
                        dreamcaller_batch.AgentJob(
                            agent_name="codex",
                            prompt="tempo",
                        ),
                        skill_text="skill",
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


class SchedulerTests(unittest.TestCase):
    """Tests for live scheduling choices."""

    def test_choose_next_agent_balances_provider_mix(self) -> None:
        self.assertEqual(
            dreamcaller_batch.choose_next_agent(
                pending_counts={"codex": 5, "claude": 5},
                active_counts={"codex": 0, "claude": 3},
                max_concurrency=4,
            ),
            "codex",
        )
        self.assertEqual(
            dreamcaller_batch.choose_next_agent(
                pending_counts={"codex": 5, "claude": 5},
                active_counts={"codex": 2, "claude": 1},
                max_concurrency=4,
            ),
            "claude",
        )
        self.assertEqual(
            dreamcaller_batch.choose_next_agent(
                pending_counts={"codex": 0, "claude": 5},
                active_counts={"codex": 0, "claude": 0},
                max_concurrency=4,
            ),
            "claude",
        )

    def test_choose_next_agent_reserves_capacity_for_other_provider(self) -> None:
        self.assertEqual(
            dreamcaller_batch.choose_next_agent(
                pending_counts={"codex": 4, "claude": 10},
                active_counts={"codex": 1, "claude": 2},
                max_concurrency=4,
            ),
            "codex",
        )


class PersistenceTests(unittest.TestCase):
    """Tests for incremental persistence and stdout progress."""

    def test_batch_reporter_writes_attempt_log_and_partial_synthesis(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = Path(temp_dir)
            output_path = temp_path / "dreamcaller.json"
            attempt_log_path = temp_path / "dreamcaller.attempts.jsonl"
            stream = io.StringIO()
            reporter = dreamcaller_batch.BatchReporter(
                prompts=["tempo"],
                output_path=output_path,
                attempt_log_path=attempt_log_path,
                stream=stream,
            )

            reporter.write_snapshot({"tempo": {}})
            reporter.record_start(
                dreamcaller_batch.AgentJob(
                    agent_name="codex",
                    prompt="tempo",
                    attempt=1,
                ),
                active_counts={"codex": 1, "claude": 0},
                pending_counts={"codex": 0, "claude": 1},
            )
            result = dreamcaller_batch.AgentAttemptResult(
                agent_name="codex",
                prompt="tempo",
                success=True,
                parsed_json=VALID_RESULT,
                errors=[],
                used_retry=False,
                attempts=1,
                exit_code=0,
            )
            reporter.record_finish(
                result,
                active_counts={"codex": 0, "claude": 0},
                pending_counts={"codex": 0, "claude": 1},
            )
            reporter.write_snapshot({"tempo": {"codex": result}})

            synthesis = json.loads(output_path.read_text(encoding="utf-8"))
            log_lines = attempt_log_path.read_text(encoding="utf-8").splitlines()

        self.assertIn("codex", synthesis["tempo"]["agents"])
        self.assertEqual(len(log_lines), 2)
        self.assertIn("start", log_lines[0])
        self.assertIn("finish", log_lines[1])
        self.assertIn("START codex", stream.getvalue())
        self.assertIn("DONE  codex", stream.getvalue())


class RecoveryTests(unittest.TestCase):
    """Tests for recovering completed work from provider logs."""

    def test_recover_claude_results_matches_short_logged_prompt(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            log_path = Path(temp_dir) / "claude-session.jsonl"
            log_path.write_text(
                "\n".join(
                    [
                        json.dumps(
                            {
                                "type": "queue-operation",
                                "operation": "enqueue",
                                "timestamp": "2026-04-11T05:00:00Z",
                                "content": (
                                    "Theme prompt: Permanent Ramp, increasing max "
                                    "energy, ramping out big threats\n\nSkill "
                                    "specification:"
                                ),
                            }
                        ),
                        json.dumps(
                            {
                                "type": "attachment",
                                "timestamp": "2026-04-11T05:05:00Z",
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

            recovered = dreamcaller_batch.recover_claude_results(
                prompts=[
                    (
                        "Permanent Ramp: Design a dreamcaller for lasting energy "
                        "acceleration so oversized cards arrive ahead of pace."
                    )
                ],
                claude_dir=Path(temp_dir),
            )

        self.assertEqual(len(recovered), 1)
        recovered_prompt = next(iter(recovered))
        self.assertTrue(recovered_prompt.startswith("Permanent Ramp:"))
        self.assertIn("claude", recovered[recovered_prompt])

    def test_recover_claude_results_extracts_structured_output(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            log_path = Path(temp_dir) / "claude-session.jsonl"
            log_path.write_text(
                "\n".join(
                    [
                        json.dumps(
                            {
                                "type": "queue-operation",
                                "operation": "enqueue",
                                "timestamp": "2026-04-11T05:00:00Z",
                                "content": "Theme prompt: tempo\n\nSkill specification:",
                            }
                        ),
                        json.dumps(
                            {
                                "type": "attachment",
                                "timestamp": "2026-04-11T05:05:00Z",
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

            recovered = dreamcaller_batch.recover_claude_results(
                prompts=["tempo"],
                claude_dir=Path(temp_dir),
            )

        self.assertIn("tempo", recovered)
        self.assertIn("claude", recovered["tempo"])
        self.assertEqual(recovered["tempo"]["claude"].parsed_json, VALID_RESULT)

    def test_recover_codex_results_extracts_last_agent_message(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            sessions_dir = Path(temp_dir)
            log_path = sessions_dir / "rollout.jsonl"
            log_path.write_text(
                "\n".join(
                    [
                        json.dumps(
                            {
                                "timestamp": "2026-04-11T05:00:00Z",
                                "type": "event_msg",
                                "payload": {
                                    "type": "user_message",
                                    "message": "Theme prompt: tempo\n\nSkill specification:",
                                },
                            }
                        ),
                        json.dumps(
                            {
                                "timestamp": "2026-04-11T05:10:00Z",
                                "type": "event_msg",
                                "payload": {
                                    "type": "task_complete",
                                    "last_agent_message": json.dumps(VALID_RESULT),
                                },
                            }
                        ),
                    ]
                )
                + "\n",
                encoding="utf-8",
            )

            recovered = dreamcaller_batch.recover_codex_results(
                prompts=["tempo"],
                codex_sessions_dir=sessions_dir,
            )

        self.assertIn("tempo", recovered)
        self.assertIn("codex", recovered["tempo"])
        self.assertEqual(recovered["tempo"]["codex"].parsed_json, VALID_RESULT)


if __name__ == "__main__":
    unittest.main()
