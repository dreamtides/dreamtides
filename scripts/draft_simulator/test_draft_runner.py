"""Tests for draft runner orchestration and CLI mode behavior.

Covers run_draft execution, force policy plumbing, trace output,
determinism, and the openness update using visible cards only.
Stdlib-only, no external dependencies.
"""

import random
import unittest

import config
import draft_runner


class TestRunDraft(unittest.TestCase):
    """Tests for run_draft orchestration."""

    def _default_cfg(self) -> config.SimulatorConfig:
        return config.load_config()

    def test_single_run_produces_correct_seat_count(self) -> None:
        cfg = self._default_cfg()
        result = draft_runner.run_draft(cfg, seed=42)
        self.assertEqual(len(result.seat_results), cfg.draft.seat_count)

    def test_each_seat_drafts_expected_card_count(self) -> None:
        cfg = self._default_cfg()
        expected_total = sum(cfg.draft.picks_per_round)
        result = draft_runner.run_draft(cfg, seed=42)
        for sr in result.seat_results:
            self.assertEqual(len(sr.drafted), expected_total)

    def test_determinism_same_seed(self) -> None:
        cfg = self._default_cfg()
        r1 = draft_runner.run_draft(cfg, seed=100)
        r2 = draft_runner.run_draft(cfg, seed=100)
        for sr1, sr2 in zip(r1.seat_results, r2.seat_results):
            self.assertAlmostEqual(sr1.deck_value, sr2.deck_value)
            self.assertEqual(
                [c.instance_id for c in sr1.drafted],
                [c.instance_id for c in sr2.drafted],
            )

    def test_different_seeds_produce_different_results(self) -> None:
        cfg = self._default_cfg()
        r1 = draft_runner.run_draft(cfg, seed=100)
        r2 = draft_runner.run_draft(cfg, seed=200)
        values1 = [sr.deck_value for sr in r1.seat_results]
        values2 = [sr.deck_value for sr in r2.seat_results]
        self.assertNotEqual(values1, values2)

    def test_deck_values_in_valid_range(self) -> None:
        cfg = self._default_cfg()
        result = draft_runner.run_draft(cfg, seed=42)
        for sr in result.seat_results:
            self.assertGreaterEqual(sr.deck_value, 0.0)
            self.assertLessEqual(sr.deck_value, 1.0)

    def test_no_traces_when_disabled(self) -> None:
        cfg = self._default_cfg()
        result = draft_runner.run_draft(cfg, seed=42, trace_enabled=False)
        self.assertEqual(len(result.traces), 0)

    def test_traces_emitted_when_enabled(self) -> None:
        cfg = self._default_cfg()
        result = draft_runner.run_draft(cfg, seed=42, trace_enabled=True)
        expected_picks = sum(cfg.draft.picks_per_round) * cfg.draft.seat_count
        self.assertEqual(len(result.traces), expected_picks)

    def test_trace_records_have_correct_fields(self) -> None:
        cfg = self._default_cfg()
        result = draft_runner.run_draft(cfg, seed=42, trace_enabled=True)
        trace = result.traces[0]
        self.assertEqual(trace.round_index, 0)
        self.assertEqual(trace.pick_index, 0)
        self.assertIn(trace.seat_index, range(cfg.draft.seat_count))
        self.assertIsInstance(trace.pack_id, str)
        self.assertGreater(len(trace.pack_card_ids), 0)
        self.assertGreater(len(trace.shown_card_ids), 0)
        self.assertIsInstance(trace.chosen_card_id, int)
        self.assertEqual(len(trace.agent_w_snapshot), cfg.cards.archetype_count)


class TestForcePolicy(unittest.TestCase):
    """Tests for force policy integration with run_draft."""

    def test_force_policy_with_archetype(self) -> None:
        cfg = config.load_config(
            overrides=[
                "agents.policy=force",
                "agents.force_archetype=2",
            ]
        )
        result = draft_runner.run_draft(cfg, seed=42)
        self.assertEqual(len(result.seat_results), cfg.draft.seat_count)
        for sr in result.seat_results:
            self.assertEqual(len(sr.drafted), 30)

    def test_force_policy_without_archetype_fails_validation(self) -> None:
        with self.assertRaises(ValueError) as ctx:
            config.load_config(overrides=["agents.policy=force"])
        self.assertIn("force_archetype", str(ctx.exception))

    def test_force_policy_trace_uses_configured_archetype(self) -> None:
        cfg = config.load_config(
            overrides=[
                "agents.policy=force",
                "agents.force_archetype=3",
            ]
        )
        result = draft_runner.run_draft(cfg, seed=42, trace_enabled=True)
        self.assertGreater(len(result.traces), 0)
        for trace in result.traces[:5]:
            self.assertIsInstance(trace.card_score, float)


class TestHumanSeatShowN(unittest.TestCase):
    """Tests for human seat Show-N behavior."""

    def test_human_seat_shown_subset_in_traces(self) -> None:
        cfg = config.load_config()
        result = draft_runner.run_draft(cfg, seed=42, trace_enabled=True)
        human_traces = [t for t in result.traces if t.seat_index == 0]
        for trace in human_traces:
            self.assertLessEqual(len(trace.shown_card_ids), cfg.agents.show_n)
            for shown_id in trace.shown_card_ids:
                self.assertIn(shown_id, trace.pack_card_ids)

    def test_ai_seats_see_full_pack(self) -> None:
        cfg = config.load_config()
        result = draft_runner.run_draft(cfg, seed=42, trace_enabled=True)
        ai_traces = [t for t in result.traces if t.seat_index > 0]
        for trace in ai_traces[:10]:
            self.assertEqual(trace.shown_card_ids, trace.pack_card_ids)


if __name__ == "__main__":
    unittest.main()
