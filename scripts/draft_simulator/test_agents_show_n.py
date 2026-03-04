"""Tests for agent pick policies and Show-N strategies.

Covers agent state management, all five pick policies, AI noise,
openness estimation, and all four Show-N card selection strategies.
Stdlib-only, no external dependencies.
"""

import random
import unittest

import agents
import show_n
import commitment
from config import AgentsConfig, ScoringConfig
from draft_models import CardDesign, CardInstance


def _make_card(
    card_id: str,
    fitness: list[float],
    power: float = 0.5,
    commit: float = 0.5,
    flex: float = 0.5,
) -> CardDesign:
    return CardDesign(
        card_id=card_id,
        name=f"Card {card_id}",
        fitness=fitness,
        power=power,
        commit=commit,
        flex=flex,
    )


def _make_instance(
    instance_id: int,
    fitness: list[float],
    power: float = 0.5,
    commit: float = 0.5,
    flex: float = 0.5,
    card_id: str = "test",
) -> CardInstance:
    design = _make_card(card_id, fitness, power, commit, flex)
    return CardInstance(instance_id=instance_id, design=design)


class TestAgentState(unittest.TestCase):
    """Tests for agent state creation and updates."""

    def test_create_agent_uniform_w(self) -> None:
        agent = agents.create_agent(4)
        self.assertEqual(len(agent.w), 4)
        for v in agent.w:
            self.assertAlmostEqual(v, 0.25)

    def test_create_agent_uniform_openness(self) -> None:
        agent = agents.create_agent(4)
        self.assertEqual(len(agent.openness), 4)
        for v in agent.openness:
            self.assertAlmostEqual(v, 0.25)

    def test_create_agent_empty_collections(self) -> None:
        agent = agents.create_agent(8)
        self.assertEqual(agent.drafted, [])
        self.assertEqual(agent.pick_history, [])
        self.assertEqual(agent.openness_window, [])

    def test_update_appends_to_drafted(self) -> None:
        agent = agents.create_agent(2)
        card = _make_instance(0, [0.8, 0.2])
        agents.update_agent_after_pick(agent, card, [card], 0, 0, "pack1", 1.0, 3)
        self.assertEqual(len(agent.drafted), 1)
        self.assertEqual(agent.drafted[0].instance_id, 0)

    def test_update_records_pick_history(self) -> None:
        agent = agents.create_agent(2)
        card = _make_instance(0, [0.8, 0.2], card_id="c1")
        agents.update_agent_after_pick(agent, card, [card], 5, 1, "pack_abc", 1.0, 3)
        self.assertEqual(len(agent.pick_history), 1)
        self.assertEqual(agent.pick_history[0], (5, 1, "c1", "pack_abc"))

    def test_w_update_is_additive(self) -> None:
        agent = agents.create_agent(2)
        card = _make_instance(0, [0.6, 0.4])
        agents.update_agent_after_pick(agent, card, [card], 0, 0, "p1", 1.0, 3)
        self.assertAlmostEqual(agent.w[0], 0.5 + 0.6)
        self.assertAlmostEqual(agent.w[1], 0.5 + 0.4)

    def test_w_update_not_renormalized(self) -> None:
        agent = agents.create_agent(2)
        card = _make_instance(0, [1.0, 1.0])
        agents.update_agent_after_pick(agent, card, [card], 0, 0, "p1", 1.0, 3)
        self.assertAlmostEqual(sum(agent.w), 3.0)

    def test_openness_window_grows(self) -> None:
        agent = agents.create_agent(2)
        card = _make_instance(0, [0.8, 0.2])
        for i in range(3):
            agents.update_agent_after_pick(agent, card, [card], i, 0, f"p{i}", 1.0, 3)
        self.assertEqual(len(agent.openness_window), 3)

    def test_openness_window_trims(self) -> None:
        agent = agents.create_agent(2)
        card = _make_instance(0, [0.8, 0.2])
        for i in range(5):
            agents.update_agent_after_pick(agent, card, [card], i, 0, f"p{i}", 1.0, 3)
        self.assertEqual(len(agent.openness_window), 3)

    def test_openness_from_pack_signal(self) -> None:
        agent = agents.create_agent(2)
        card_a = _make_instance(0, [1.0, 0.0])
        card_b = _make_instance(1, [0.0, 1.0])
        agents.update_agent_after_pick(
            agent, card_a, [card_a, card_b], 0, 0, "p1", 1.0, 3
        )
        # Supply signal should be mean of [1.0, 0.0] and [0.0, 1.0] = [0.5, 0.5]
        self.assertAlmostEqual(agent.openness[0], 0.5)
        self.assertAlmostEqual(agent.openness[1], 0.5)


class TestPickPolicies(unittest.TestCase):
    """Tests for pick policy implementations."""

    def _agents_cfg(
        self,
        optimality: float = 1.0,
        signal_weight: float = 0.2,
    ) -> AgentsConfig:
        return AgentsConfig(
            ai_optimality=optimality,
            ai_signal_weight=signal_weight,
        )

    def test_greedy_maximizes_deck_value(self) -> None:
        agent = agents.create_agent(2)
        agent.w = [2.0, 0.5]
        # Card with high fitness for arch 0 should be preferred
        card_good = _make_instance(0, [0.9, 0.1], power=0.8)
        card_bad = _make_instance(1, [0.1, 0.9], power=0.8)
        cfg = self._agents_cfg()
        scoring = ScoringConfig()
        rng = random.Random(42)
        pick = agents.pick_card(
            [card_bad, card_good], agent, "greedy", cfg, scoring, rng
        )
        self.assertEqual(pick.instance_id, card_good.instance_id)

    def test_archetype_loyal_follows_w(self) -> None:
        agent = agents.create_agent(2)
        agent.w = [3.0, 1.0]
        card_arch0 = _make_instance(0, [0.9, 0.1])
        card_arch1 = _make_instance(1, [0.1, 0.9])
        cfg = self._agents_cfg()
        scoring = ScoringConfig()
        rng = random.Random(42)
        pick = agents.pick_card(
            [card_arch1, card_arch0],
            agent,
            "archetype_loyal",
            cfg,
            scoring,
            rng,
        )
        self.assertEqual(pick.instance_id, card_arch0.instance_id)

    def test_archetype_loyal_breaks_ties_by_power(self) -> None:
        agent = agents.create_agent(2)
        agent.w = [3.0, 1.0]
        card_low = _make_instance(0, [0.9, 0.1], power=0.3)
        card_high = _make_instance(1, [0.9, 0.1], power=0.8)
        cfg = self._agents_cfg()
        scoring = ScoringConfig()
        rng = random.Random(42)
        pick = agents.pick_card(
            [card_low, card_high],
            agent,
            "archetype_loyal",
            cfg,
            scoring,
            rng,
        )
        self.assertEqual(pick.instance_id, card_high.instance_id)

    def test_force_ignores_w(self) -> None:
        agent = agents.create_agent(2)
        agent.w = [10.0, 0.1]  # w strongly favors arch 0
        card_arch0 = _make_instance(0, [0.9, 0.1])
        card_arch1 = _make_instance(1, [0.1, 0.9])
        cfg = self._agents_cfg()
        scoring = ScoringConfig()
        rng = random.Random(42)
        # Force to archetype 1, should ignore w
        pick = agents.pick_card(
            [card_arch0, card_arch1],
            agent,
            "force",
            cfg,
            scoring,
            rng,
            force_archetype=1,
        )
        self.assertEqual(pick.instance_id, card_arch1.instance_id)

    def test_force_breaks_ties_by_power(self) -> None:
        agent = agents.create_agent(2)
        card_low = _make_instance(0, [0.9, 0.1], power=0.3)
        card_high = _make_instance(1, [0.9, 0.1], power=0.8)
        cfg = self._agents_cfg()
        scoring = ScoringConfig()
        rng = random.Random(42)
        pick = agents.pick_card(
            [card_low, card_high],
            agent,
            "force",
            cfg,
            scoring,
            rng,
            force_archetype=0,
        )
        self.assertEqual(pick.instance_id, card_high.instance_id)

    def test_force_requires_archetype(self) -> None:
        agent = agents.create_agent(2)
        card = _make_instance(0, [0.5, 0.5])
        cfg = self._agents_cfg()
        scoring = ScoringConfig()
        rng = random.Random(42)
        with self.assertRaises(ValueError):
            agents.pick_card([card], agent, "force", cfg, scoring, rng)

    def test_adaptive_combines_terms(self) -> None:
        agent = agents.create_agent(2)
        agent.w = [2.0, 0.5]
        agent.openness = [0.8, 0.2]
        # Card aligned with both w and openness
        card_good = _make_instance(0, [0.9, 0.1], power=0.8)
        card_bad = _make_instance(1, [0.1, 0.9], power=0.3)
        cfg = self._agents_cfg(signal_weight=0.2)
        scoring = ScoringConfig()
        rng = random.Random(42)
        pick = agents.pick_card(
            [card_bad, card_good],
            agent,
            "adaptive",
            cfg,
            scoring,
            rng,
        )
        self.assertEqual(pick.instance_id, card_good.instance_id)

    def test_adaptive_score_formula(self) -> None:
        agent = agents.create_agent(2)
        agent.w = [2.0, 1.0]
        agent.openness = [0.6, 0.4]
        card = _make_instance(0, [0.8, 0.2], power=0.5)
        cfg = AgentsConfig(
            ai_signal_weight=0.2, ai_power_weight=0.3, ai_pref_weight=0.5
        )
        score = agents.score_card_adaptive(card, agent, cfg)
        # power_weight=0.3, pref_weight=0.5, signal_weight=0.2
        # w_norm = [2/3, 1/3]
        # power_term = 0.3 * 0.5 = 0.15
        # pref_term = 0.5 * (0.8*2/3 + 0.2*1/3) = 0.5 * (0.5333 + 0.0667) = 0.3
        # signal_term = 0.2 * (0.8*0.6 + 0.2*0.4) = 0.2 * (0.48 + 0.08) = 0.112
        expected = 0.15 + 0.3 + 0.112
        self.assertAlmostEqual(score, expected, places=3)

    def test_signal_ignorant_ignores_openness(self) -> None:
        agent = agents.create_agent(2)
        agent.w = [2.0, 1.0]
        agent.openness = [0.9, 0.1]  # Skewed openness, should be ignored
        card = _make_instance(0, [0.8, 0.2], power=0.5)
        cfg = AgentsConfig(
            ai_signal_weight=0.2, ai_power_weight=0.3, ai_pref_weight=0.5
        )

        score_ignorant = agents.score_card_signal_ignorant(card, agent, cfg)

        # Signal-ignorant uses uniform openness = [0.5, 0.5]
        # signal_term = 0.2 * (0.8*0.5 + 0.2*0.5) = 0.2 * 0.5 = 0.1
        # Compare with adaptive that would use [0.9, 0.1]
        score_adaptive = agents.score_card_adaptive(card, agent, cfg)

        self.assertNotAlmostEqual(score_ignorant, score_adaptive)

    def test_signal_ignorant_constant_across_openness_changes(self) -> None:
        agent = agents.create_agent(2)
        agent.w = [1.0, 1.0]
        card = _make_instance(0, [0.5, 0.5], power=0.5)
        cfg = AgentsConfig(
            ai_signal_weight=0.2, ai_power_weight=0.3, ai_pref_weight=0.5
        )

        agent.openness = [0.1, 0.9]
        score1 = agents.score_card_signal_ignorant(card, agent, cfg)
        agent.openness = [0.9, 0.1]
        score2 = agents.score_card_signal_ignorant(card, agent, cfg)

        self.assertAlmostEqual(score1, score2)

    def test_noise_causes_random_picks(self) -> None:
        """With ai_optimality=0.0, picks should be random."""
        agent = agents.create_agent(2)
        agent.w = [10.0, 0.0]
        card_good = _make_instance(0, [1.0, 0.0], power=1.0)
        card_bad = _make_instance(1, [0.0, 1.0], power=0.0)
        cfg = self._agents_cfg(optimality=0.0)
        scoring = ScoringConfig()

        picks: set[int] = set()
        for seed in range(100):
            rng = random.Random(seed)
            pick = agents.pick_card(
                [card_good, card_bad],
                agent,
                "greedy",
                cfg,
                scoring,
                rng,
            )
            picks.add(pick.instance_id)

        # With 100 random trials on 2 cards, both should be picked
        self.assertEqual(picks, {0, 1})

    def test_full_optimality_never_random(self) -> None:
        """With ai_optimality=1.0, should always pick optimally."""
        agent = agents.create_agent(2)
        agent.w = [10.0, 0.0]
        card_good = _make_instance(0, [1.0, 0.0], power=1.0)
        card_bad = _make_instance(1, [0.0, 1.0], power=0.0)
        cfg = self._agents_cfg(optimality=1.0)
        scoring = ScoringConfig()

        for seed in range(20):
            rng = random.Random(seed)
            pick = agents.pick_card(
                [card_good, card_bad],
                agent,
                "greedy",
                cfg,
                scoring,
                rng,
            )
            self.assertEqual(pick.instance_id, card_good.instance_id)


class TestShowN(unittest.TestCase):
    """Tests for Show-N card selection strategies."""

    def _make_pack(self, n: int) -> list[CardInstance]:
        cards = []
        for i in range(n):
            fitness = [0.1] * 8
            fitness[i % 8] = 0.8
            cards.append(
                _make_instance(
                    i,
                    fitness,
                    power=0.2 + 0.05 * i,
                    commit=0.3 + 0.03 * i,
                    card_id=f"card_{i}",
                )
            )
        return cards

    def test_uniform_returns_n_cards(self) -> None:
        pack = self._make_pack(10)
        rng = random.Random(42)
        result = show_n.select_cards(pack, 4, "uniform", rng)
        self.assertEqual(len(result), 4)

    def test_power_biased_returns_n_cards(self) -> None:
        pack = self._make_pack(10)
        rng = random.Random(42)
        result = show_n.select_cards(pack, 4, "power_biased", rng)
        self.assertEqual(len(result), 4)

    def test_curated_returns_n_cards(self) -> None:
        pack = self._make_pack(10)
        rng = random.Random(42)
        w = [3.0, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1]
        result = show_n.select_cards(pack, 4, "curated", rng, human_w=w)
        self.assertEqual(len(result), 4)

    def test_signal_rich_returns_n_cards(self) -> None:
        pack = self._make_pack(10)
        rng = random.Random(42)
        result = show_n.select_cards(pack, 4, "signal_rich", rng)
        self.assertEqual(len(result), 4)

    def test_small_pack_returns_all(self) -> None:
        pack = self._make_pack(3)
        rng = random.Random(42)
        result = show_n.select_cards(pack, 4, "uniform", rng)
        self.assertEqual(len(result), 3)

    def test_no_duplicate_instance_ids(self) -> None:
        pack = self._make_pack(15)
        rng = random.Random(42)
        for strategy in ["uniform", "power_biased", "signal_rich", "top_scored"]:
            result = show_n.select_cards(pack, 4, strategy, rng, human_w=[1.0] * 8)
            ids = [c.instance_id for c in result]
            self.assertEqual(len(ids), len(set(ids)), f"Duplicates in {strategy}")

    def test_curated_no_duplicates(self) -> None:
        pack = self._make_pack(15)
        rng = random.Random(42)
        w = [5.0, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1]
        result = show_n.select_cards(pack, 4, "curated", rng, human_w=w)
        ids = [c.instance_id for c in result]
        self.assertEqual(len(ids), len(set(ids)))

    def test_curated_includes_on_plan(self) -> None:
        """Curated should include at least 1 on-plan card when available."""
        # Create pack with 1 clear on-plan card and several off-plan
        on_plan = _make_instance(0, [0.9, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1], power=0.5)
        off_plan_cards = [
            _make_instance(
                i + 1,
                [0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1],
                power=0.6,
                card_id=f"off_{i}",
            )
            for i in range(9)
        ]
        pack = [on_plan] + off_plan_cards
        w = [5.0, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1]
        rng = random.Random(42)
        result = show_n.select_cards(pack, 4, "curated", rng, human_w=w)
        # The on-plan card should be included
        result_ids = {c.instance_id for c in result}
        self.assertIn(on_plan.instance_id, result_ids)

    def test_curated_includes_off_plan_strong(self) -> None:
        """Curated should include an off-plan strong card when available."""
        # All cards on-plan except one strong off-plan card
        on_plan_cards = [
            _make_instance(
                i,
                [0.8, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1],
                power=0.3,
                card_id=f"on_{i}",
            )
            for i in range(8)
        ]
        off_plan_strong = _make_instance(
            99,
            [0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1],
            power=0.8,
            card_id="off_strong",
        )
        pack = on_plan_cards + [off_plan_strong]
        w = [5.0, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1]
        rng = random.Random(42)
        result = show_n.select_cards(pack, 4, "curated", rng, human_w=w)
        result_ids = {c.instance_id for c in result}
        self.assertIn(off_plan_strong.instance_id, result_ids)

    def test_curated_falls_back_without_w(self) -> None:
        """Curated without human_w should fall back to power-biased."""
        pack = self._make_pack(10)
        rng = random.Random(42)
        result = show_n.select_cards(pack, 4, "curated", rng, human_w=None)
        self.assertEqual(len(result), 4)

    def test_curated_small_n_does_not_exceed(self) -> None:
        """Curated with n=1 must not return more than 1 card."""
        on_plan = _make_instance(0, [0.9, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1], power=0.5)
        off_plan_strong = _make_instance(
            1, [0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1], power=0.8
        )
        filler = [
            _make_instance(
                i + 2,
                [0.3, 0.3, 0.3, 0.3, 0.3, 0.3, 0.3, 0.3],
                power=0.4,
                card_id=f"fill_{i}",
            )
            for i in range(8)
        ]
        pack = [on_plan, off_plan_strong] + filler
        w = [5.0, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1]
        rng = random.Random(42)
        result = show_n.select_cards(pack, 1, "curated", rng, human_w=w)
        self.assertEqual(len(result), 1)

    def test_top_scored_returns_n_cards(self) -> None:
        pack = self._make_pack(10)
        rng = random.Random(42)
        w = [3.0, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1]
        result = show_n.select_cards(pack, 4, "top_scored", rng, human_w=w)
        self.assertEqual(len(result), 4)

    def test_top_scored_no_duplicates(self) -> None:
        pack = self._make_pack(15)
        rng = random.Random(42)
        w = [5.0, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1]
        result = show_n.select_cards(pack, 4, "top_scored", rng, human_w=w)
        ids = [c.instance_id for c in result]
        self.assertEqual(len(ids), len(set(ids)))

    def test_top_scored_includes_on_plan_when_concentrated(self) -> None:
        """Top-scored should include on-plan cards when w is concentrated."""
        on_plan = _make_instance(0, [0.9, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1], power=0.5)
        off_plan_cards = [
            _make_instance(
                i + 1,
                [0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1],
                power=0.5,
                card_id=f"off_{i}",
            )
            for i in range(9)
        ]
        pack = [on_plan] + off_plan_cards
        w = [5.0, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1]
        rng = random.Random(42)
        result = show_n.select_cards(pack, 4, "top_scored", rng, human_w=w)
        result_ids = {c.instance_id for c in result}
        self.assertIn(on_plan.instance_id, result_ids)

    def test_top_scored_falls_back_without_w(self) -> None:
        """Top-scored without human_w should fall back to power-biased."""
        pack = self._make_pack(10)
        rng = random.Random(42)
        result = show_n.select_cards(pack, 4, "top_scored", rng, human_w=None)
        self.assertEqual(len(result), 4)

    def test_curated_n2_with_both_guarantees(self) -> None:
        """Curated with n=2 returns exactly 2 when both guarantees apply."""
        on_plan = _make_instance(0, [0.9, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1], power=0.5)
        off_plan_strong = _make_instance(
            1, [0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1], power=0.8
        )
        filler = [
            _make_instance(
                i + 2,
                [0.3, 0.3, 0.3, 0.3, 0.3, 0.3, 0.3, 0.3],
                power=0.4,
                card_id=f"fill_{i}",
            )
            for i in range(8)
        ]
        pack = [on_plan, off_plan_strong] + filler
        w = [5.0, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1]
        rng = random.Random(42)
        result = show_n.select_cards(pack, 2, "curated", rng, human_w=w)
        self.assertEqual(len(result), 2)

    def test_force_out_of_range_raises(self) -> None:
        """Force policy with out-of-range archetype should raise ValueError."""
        agent = agents.create_agent(2)
        card = _make_instance(0, [0.5, 0.5])
        cfg = AgentsConfig(ai_optimality=1.0)
        scoring = ScoringConfig()
        rng = random.Random(42)
        with self.assertRaises(ValueError):
            agents.pick_card(
                [card], agent, "force", cfg, scoring, rng, force_archetype=99
            )
        with self.assertRaises(ValueError):
            agents.pick_card(
                [card], agent, "force", cfg, scoring, rng, force_archetype=-1
            )

    def test_unknown_strategy_raises(self) -> None:
        pack = self._make_pack(5)
        rng = random.Random(42)
        with self.assertRaises(ValueError):
            show_n.select_cards(pack, 4, "nonexistent", rng)


if __name__ == "__main__":
    unittest.main()
