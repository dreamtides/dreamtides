"""Tests for resonance heuristic scoring, assignment, and tagging."""

from pathlib import Path

from resonance_heuristic import (
    CONFIDENCE_THRESHOLD,
    DUAL_THRESHOLD,
    KEYWORD_RESONANCE,
    RESONANCES,
    SINGLE_THRESHOLD,
    assign_resonance,
    assign_tags,
    find_keywords,
    load_allocations,
    score_resonances,
)


class TestScoreResonances:
    def test_subtype_affinity_warrior(self) -> None:
        card: dict[str, object] = {"subtype": "Warrior"}
        scores = score_resonances(card)
        assert scores["Ember"] > 0
        assert scores["Stone"] > 0
        assert scores["Tide"] == 0

    def test_subtype_affinity_mage(self) -> None:
        card: dict[str, object] = {"subtype": "Mage"}
        scores = score_resonances(card)
        assert scores["Tide"] > 0
        assert scores["Ember"] == 0

    def test_keyword_matching(self) -> None:
        card: dict[str, object] = {
            "rules_text": "Draw 2 cards and dissolve a character.",
        }
        scores = score_resonances(card)
        assert scores["Tide"] > 0
        assert scores["Ruin"] > 0

    def test_keyword_case_insensitive(self) -> None:
        card: dict[str, object] = {"rules_text": "FORESEE 3"}
        scores = score_resonances(card)
        assert scores["Tide"] > 0

    def test_fast_flag_adds_zephyr(self) -> None:
        card: dict[str, object] = {"is_fast": True}
        scores = score_resonances(card)
        assert scores["Zephyr"] > 0

    def test_low_cost_character_adds_ember(self) -> None:
        card: dict[str, object] = {"energy_cost": 1, "card_type": "Character"}
        scores = score_resonances(card)
        assert scores["Ember"] > 0

    def test_high_cost_character_adds_stone(self) -> None:
        card: dict[str, object] = {"energy_cost": 6, "card_type": "Character"}
        scores = score_resonances(card)
        assert scores["Stone"] > 0

    def test_cost_heuristic_only_for_characters(self) -> None:
        card: dict[str, object] = {"energy_cost": 1, "card_type": "Event"}
        scores = score_resonances(card)
        assert scores["Ember"] == 0

    def test_empty_card_all_zeros(self) -> None:
        card: dict[str, object] = {}
        scores = score_resonances(card)
        for r in RESONANCES:
            assert scores[r] == 0.0

    def test_unknown_subtype_ignored(self) -> None:
        card: dict[str, object] = {"subtype": "UnknownType"}
        scores = score_resonances(card)
        for r in RESONANCES:
            assert scores[r] == 0.0


class TestAssignResonance:
    def _make_scores(self, **kwargs: float) -> dict[str, float]:
        scores = {r: 0.0 for r in RESONANCES}
        scores.update(kwargs)
        return scores

    def test_neutral_when_no_signal(self) -> None:
        scores = self._make_scores()
        assert assign_resonance(scores) == []

    def test_neutral_when_below_confidence(self) -> None:
        scores = self._make_scores(Tide=1.0)
        assert sum(scores.values()) < CONFIDENCE_THRESHOLD
        assert assign_resonance(scores) == []

    def test_single_resonance(self) -> None:
        scores = self._make_scores(Ember=2.0, Tide=1.0)
        result = assign_resonance(scores)
        assert result == ["Ember"]

    def test_dual_resonance(self) -> None:
        scores = self._make_scores(Ember=3.0, Stone=3.0)
        result = assign_resonance(scores)
        assert len(result) == 2
        assert "Ember" in result
        assert "Stone" in result

    def test_dual_resonance_sorted(self) -> None:
        scores = self._make_scores(Zephyr=3.0, Ember=3.0)
        result = assign_resonance(scores)
        assert result == sorted(result)

    def test_top_score_below_single_threshold_is_neutral(self) -> None:
        scores = self._make_scores(Tide=0.5, Ember=0.5, Stone=0.5, Ruin=0.5)
        assert assign_resonance(scores) == []


class TestFindKeywords:
    def test_finds_matching_keywords(self) -> None:
        keywords = find_keywords("Draw 2 cards then foresee 3.")
        assert "draw" in keywords
        assert "foresee" in keywords

    def test_case_insensitive(self) -> None:
        keywords = find_keywords("DISSOLVE a character")
        assert "dissolve" in keywords

    def test_empty_text(self) -> None:
        assert find_keywords("") == set()

    def test_no_match(self) -> None:
        assert find_keywords("This card does nothing special.") == set()

    def test_word_boundary_respected(self) -> None:
        keywords = find_keywords("undiscovered territory")
        assert "discover" not in keywords


class TestAssignTags:
    def test_tribal_tag_from_subtype(self) -> None:
        card: dict[str, object] = {"subtype": "Warrior", "rules_text": ""}
        tags = assign_tags(card)
        assert "tribal:warrior" in tags

    def test_tribal_tag_hyphenated(self) -> None:
        card: dict[str, object] = {"subtype": "Spirit Animal", "rules_text": ""}
        tags = assign_tags(card)
        assert "tribal:spirit-animal" in tags

    def test_mechanic_tag_from_rules_text(self) -> None:
        card: dict[str, object] = {"rules_text": "Foresee 3."}
        tags = assign_tags(card)
        assert "mechanic:foresee" in tags

    def test_role_finisher_from_high_spark(self) -> None:
        card: dict[str, object] = {"spark": 5, "rules_text": ""}
        tags = assign_tags(card)
        assert "role:finisher" in tags

    def test_role_removal_from_dissolve(self) -> None:
        card: dict[str, object] = {"rules_text": "Dissolve target character."}
        tags = assign_tags(card)
        assert "role:removal" in tags

    def test_role_engine_from_draw(self) -> None:
        card: dict[str, object] = {"rules_text": "Draw 2 cards."}
        tags = assign_tags(card)
        assert "role:engine" in tags

    def test_fallback_event_tag(self) -> None:
        card: dict[str, object] = {"card_type": "Event", "rules_text": ""}
        tags = assign_tags(card)
        assert "mechanic:event" in tags

    def test_fallback_general_tag(self) -> None:
        card: dict[str, object] = {"card_type": "Character", "rules_text": ""}
        tags = assign_tags(card)
        assert "mechanic:general" in tags

    def test_no_duplicate_tags(self) -> None:
        card: dict[str, object] = {"subtype": "Mage", "rules_text": "Draw a card."}
        tags = assign_tags(card)
        assert len(tags) == len(set(tags))

    def test_archetype_tags_included(self) -> None:
        card: dict[str, object] = {"rules_text": "Draw a card."}
        tags = assign_tags(card, archetypes=["Tempest", "Mirage"])
        assert "archetype:tempest" in tags
        assert "archetype:mirage" in tags

    def test_archetype_tags_come_first(self) -> None:
        card: dict[str, object] = {"subtype": "Warrior", "rules_text": "Draw 2."}
        tags = assign_tags(card, archetypes=["Crucible"])
        assert tags[0] == "archetype:crucible"

    def test_no_archetypes_produces_standard_tags(self) -> None:
        card: dict[str, object] = {"subtype": "Warrior", "rules_text": ""}
        tags = assign_tags(card)
        assert "tribal:warrior" in tags
        assert not any(t.startswith("archetype:") for t in tags)


class TestLoadAllocations:
    def test_loads_real_allocations_file(self) -> None:
        path = Path(__file__).parent / "data" / "card_allocations.toml"
        allocations = load_allocations(path)
        assert len(allocations) == 249

    def test_mono_resonance_card(self) -> None:
        path = Path(__file__).parent / "data" / "card_allocations.toml"
        allocations = load_allocations(path)
        entry = allocations["The Calling Night"]
        assert entry["resonance"] == ["Tide"]
        assert entry["archetypes"] == ["Tempest"]

    def test_dual_resonance_card(self) -> None:
        path = Path(__file__).parent / "data" / "card_allocations.toml"
        allocations = load_allocations(path)
        entry = allocations["Tideweaver Adept"]
        assert entry["resonance"] == ["Tide", "Zephyr"]
        assert entry["archetypes"] == ["Mirage"]

    def test_neutral_card(self) -> None:
        path = Path(__file__).parent / "data" / "card_allocations.toml"
        allocations = load_allocations(path)
        entry = allocations["Nexus of Passing"]
        assert entry["resonance"] == []
        assert entry["archetypes"] == []

    def test_curated_overrides_heuristic(self) -> None:
        """Data Pulse was Ember by heuristic but Tide by curation."""
        path = Path(__file__).parent / "data" / "card_allocations.toml"
        allocations = load_allocations(path)
        entry = allocations["Data Pulse"]
        assert entry["resonance"] == ["Tide"]

    def test_dreamtide_cartographer_moved_to_zephyr(self) -> None:
        path = Path(__file__).parent / "data" / "card_allocations.toml"
        allocations = load_allocations(path)
        entry = allocations["Dreamtide Cartographer"]
        assert entry["resonance"] == ["Zephyr"]
