"""Dream Journey and Tempting Offer site interactions.

Dream Journey presents 2 (or 3 if Twilight-enhanced) journey events to
pick from. Tempting Offer presents 2 (or 3 if Infernal-enhanced)
reward/cost pairs. Both use single-select with a skip option.
"""

import random
from typing import Optional

import algorithm
import input_handler
import pool as pool_module
import render
from jsonl_log import SessionLogger
from models import (
    AlgorithmParams,
    BaneCard,
    Card,
    Dreamsign,
    EffectType,
    Journey,
    PoolParams,
    Rarity,
    Resonance,
    TemptingOffer,
)
from quest_state import QuestState

JOURNEY_DEFAULT_COUNT = 2
JOURNEY_ENHANCED_COUNT = 3
OFFER_DEFAULT_COUNT = 2
OFFER_ENHANCED_COUNT = 3

_EFFECT_TYPE_LABELS: dict[EffectType, str] = {
    EffectType.ADD_CARDS: "Add cards",
    EffectType.ADD_ESSENCE: "Gain essence",
    EffectType.REMOVE_CARDS: "Remove cards",
    EffectType.ADD_DREAMSIGN: "Gain dreamsign",
    EffectType.GAIN_RESONANCE: "Boost resonance",
    EffectType.LARGE_ESSENCE: "Gain essence",
    EffectType.LOSE_ESSENCE: "Lose essence",
    EffectType.ADD_BANE_CARD: "Gain bane card",
    EffectType.ADD_BANE_DREAMSIGN: "Gain bane dreamsign",
}


def select_journeys(
    all_journeys: list[Journey],
    rng: random.Random,
    is_enhanced: bool,
) -> list[Journey]:
    """Select random journeys from the full list."""
    count = JOURNEY_ENHANCED_COUNT if is_enhanced else JOURNEY_DEFAULT_COUNT
    count = min(count, len(all_journeys))
    return rng.sample(all_journeys, count)


def select_offers(
    all_offers: list[TemptingOffer],
    rng: random.Random,
    is_enhanced: bool,
) -> list[TemptingOffer]:
    """Select random offers from the full list."""
    count = OFFER_ENHANCED_COUNT if is_enhanced else OFFER_DEFAULT_COUNT
    count = min(count, len(all_offers))
    return rng.sample(all_offers, count)


def _add_cards_from_pool(
    state: QuestState,
    count: int,
    algorithm_params: AlgorithmParams,
    pool_params: PoolParams,
) -> list[Card]:
    """Draw weighted cards from pool and add to deck. Returns added cards."""
    if not state.pool:
        pool_module.refill_pool(state.pool, state.all_cards, pool_params)

    selections = algorithm.select_cards(
        pool=state.pool,
        n=count,
        profile=state.resonance_profile,
        params=algorithm_params,
        rng=state.rng,
    )

    added: list[Card] = []
    for entry, _weight in selections:
        state.add_card(entry.card)
        pool_module.remove_entry(state.pool, entry)
        added.append(entry.card)
    return added


def _remove_random_cards(state: QuestState, count: int) -> list[str]:
    """Randomly remove cards from the deck. Returns names of removed cards."""
    n = min(count, len(state.deck))
    if n == 0:
        return []

    to_remove = state.rng.sample(state.deck, n)
    removed_names: list[str] = []
    for dc in to_remove:
        removed_names.append(dc.card.name)
        state.remove_card(dc)
    return removed_names


def _add_random_non_bane_dreamsign(
    state: QuestState,
    all_dreamsigns: list[Dreamsign],
) -> Optional[Dreamsign]:
    """Add a random non-bane dreamsign. Returns the added dreamsign or None."""
    if state.is_over_dreamsign_limit():
        return None

    non_bane = [ds for ds in all_dreamsigns if not ds.is_bane]
    if not non_bane:
        return None

    chosen = state.rng.choice(non_bane)
    state.add_dreamsign(chosen)
    return chosen


def _add_random_bane_dreamsign(
    state: QuestState,
    all_dreamsigns: list[Dreamsign],
) -> Optional[Dreamsign]:
    """Add a random bane dreamsign. Returns the added dreamsign or None."""
    bane = [ds for ds in all_dreamsigns if ds.is_bane]
    if not bane:
        return None

    chosen = state.rng.choice(bane)
    state.add_dreamsign(chosen)
    return chosen


def _add_random_bane_card(
    state: QuestState,
    all_banes: list[BaneCard],
) -> Optional[str]:
    """Add a random bane card to deck. Returns the bane card name or None."""
    if not all_banes:
        return None

    chosen = state.rng.choice(all_banes)
    bane_as_card = Card(
        name=chosen.name,
        card_number=-1,
        energy_cost=chosen.energy_cost,
        card_type=chosen.card_type,
        subtype=None,
        is_fast=False,
        spark=None,
        rarity=Rarity.COMMON,
        rules_text=chosen.rules_text,
        resonances=frozenset(),
        tags=frozenset(),
    )
    state.add_bane_card(bane_as_card)
    return chosen.name


def _gain_resonance(state: QuestState, amount: int) -> Optional[Resonance]:
    """Add amount to the player's top resonance. Returns the resonance or None."""
    top = state.resonance_profile.top_n(1, rng=state.rng)
    if not top:
        # All zeros, pick a random resonance
        chosen = state.rng.choice(list(Resonance))
        state.resonance_profile.add(chosen, amount)
        return chosen

    resonance, _count = top[0]
    state.resonance_profile.add(resonance, amount)
    return resonance


def apply_journey_effect(
    state: QuestState,
    journey: Journey,
    algorithm_params: AlgorithmParams,
    pool_params: PoolParams,
    all_dreamsigns: list[Dreamsign],
) -> dict[str, object]:
    """Apply a journey's effect to state. Returns a dict of state changes."""
    changes: dict[str, object] = {
        "effect_type": journey.effect_type.value,
        "effect_value": journey.effect_value,
    }

    if journey.effect_type == EffectType.ADD_CARDS:
        added = _add_cards_from_pool(
            state, journey.effect_value, algorithm_params, pool_params,
        )
        changes["cards_added"] = [c.name for c in added]

    elif journey.effect_type == EffectType.ADD_ESSENCE:
        state.gain_essence(journey.effect_value)
        changes["essence_delta"] = journey.effect_value

    elif journey.effect_type == EffectType.REMOVE_CARDS:
        removed = _remove_random_cards(state, journey.effect_value)
        changes["cards_removed"] = removed

    elif journey.effect_type == EffectType.ADD_DREAMSIGN:
        ds = _add_random_non_bane_dreamsign(state, all_dreamsigns)
        changes["dreamsign_added"] = ds.name if ds is not None else None

    elif journey.effect_type == EffectType.GAIN_RESONANCE:
        res = _gain_resonance(state, journey.effect_value)
        changes["resonance_boosted"] = res.value if res is not None else None
        changes["resonance_amount"] = journey.effect_value

    return changes


def apply_reward_effect(
    state: QuestState,
    effect_type: EffectType,
    value: int,
    algorithm_params: AlgorithmParams,
    pool_params: PoolParams,
    all_dreamsigns: list[Dreamsign],
) -> dict[str, object]:
    """Apply a reward effect to state. Returns a dict of state changes."""
    changes: dict[str, object] = {
        "reward_effect_type": effect_type.value,
        "reward_value": value,
    }

    if effect_type == EffectType.ADD_CARDS:
        added = _add_cards_from_pool(state, value, algorithm_params, pool_params)
        changes["cards_added"] = [c.name for c in added]

    elif effect_type in (EffectType.ADD_ESSENCE, EffectType.LARGE_ESSENCE):
        state.gain_essence(value)
        changes["essence_delta"] = value

    elif effect_type == EffectType.ADD_DREAMSIGN:
        ds = _add_random_non_bane_dreamsign(state, all_dreamsigns)
        changes["dreamsign_added"] = ds.name if ds is not None else None

    return changes


def apply_cost_effect(
    state: QuestState,
    effect_type: EffectType,
    value: int,
    all_banes: list[BaneCard],
    all_dreamsigns: list[Dreamsign],
) -> dict[str, object]:
    """Apply a cost effect to state. Returns a dict of state changes."""
    changes: dict[str, object] = {
        "cost_effect_type": effect_type.value,
        "cost_value": value,
    }

    if effect_type == EffectType.LOSE_ESSENCE:
        actual_loss = min(value, state.essence)
        state.essence = max(0, state.essence - value)
        changes["essence_delta"] = -actual_loss

    elif effect_type == EffectType.ADD_BANE_CARD:
        name = _add_random_bane_card(state, all_banes)
        changes["bane_card_added"] = name

    elif effect_type == EffectType.ADD_BANE_DREAMSIGN:
        ds = _add_random_bane_dreamsign(state, all_dreamsigns)
        changes["bane_dreamsign_added"] = ds.name if ds is not None else None

    elif effect_type == EffectType.REMOVE_CARDS:
        removed = _remove_random_cards(state, value)
        changes["cards_removed"] = removed

    return changes


def _format_journey_option(
    journey: Journey,
    highlighted: bool,
) -> list[str]:
    """Format a journey for display in the selection menu."""
    marker = ">" if highlighted else " "
    effect_label = _EFFECT_TYPE_LABELS.get(
        journey.effect_type, journey.effect_type.value,
    )
    line1 = (
        f"  {marker} {render.BOLD}{journey.name}{render.RESET}"
        f"  ({effect_label}: {journey.effect_value})"
    )
    line2 = f"      \"{journey.description}\""
    return [line1, line2]


def _format_offer_option(
    offer: TemptingOffer,
    highlighted: bool,
) -> list[str]:
    """Format an offer for display in the selection menu."""
    marker = ">" if highlighted else " "
    reward_label = _EFFECT_TYPE_LABELS.get(
        offer.reward_effect_type, offer.reward_effect_type.value,
    )
    cost_label = _EFFECT_TYPE_LABELS.get(
        offer.cost_effect_type, offer.cost_effect_type.value,
    )

    line1 = (
        f"  {marker} {render.BOLD}Reward:{render.RESET} {offer.reward_name}"
        f"  ({reward_label}: {offer.reward_value})"
    )
    line2 = f"      \"{offer.reward_description}\""
    sep = f"      {render.DIM}{'- ' * 20}{render.RESET}"
    line3 = (
        f"      {render.BOLD}Cost:{render.RESET} {offer.cost_name}"
        f"  ({cost_label}: {offer.cost_value})"
    )
    line4 = f"      \"{offer.cost_description}\""
    return [line1, line2, sep, line3, line4]


def run_dream_journey(
    state: QuestState,
    all_journeys: list[Journey],
    all_dreamsigns: list[Dreamsign],
    algorithm_params: AlgorithmParams,
    pool_params: PoolParams,
    dreamscape_name: str,
    dreamscape_number: int,
    logger: Optional[SessionLogger],
    is_enhanced: bool = False,
) -> None:
    """Run a Dream Journey site interaction.

    Selects 2 (or 3 if enhanced) random journeys, displays them for the
    player to choose via arrow-key navigation, applies the selected
    journey's effect, logs the selection, and shows the resonance profile
    footer.
    """
    choices = select_journeys(all_journeys, state.rng, is_enhanced)

    # Display header
    header = render.site_visit_header(
        dreamscape_name=dreamscape_name,
        site_type_label="Dream Journey",
        pick_info="Pick 1 or Close",
        dreamscape_number=dreamscape_number,
    )
    print(header)
    print()

    # Build options: journeys + close
    option_labels = [j.name for j in choices] + ["Close"]
    close_index = len(choices)

    def _render_fn(index: int, option: str, is_selected: bool) -> str:
        if index == close_index:
            marker = ">" if is_selected else " "
            return f"  {marker} {render.DIM}Close{render.RESET}"
        journey = choices[index]
        lines = _format_journey_option(journey, highlighted=is_selected)
        return "\n".join(lines)

    selected_index = input_handler.single_select(
        options=option_labels,
        render_fn=_render_fn,
    )

    choice_made: Optional[str] = None
    state_changes: dict[str, object] = {}

    if selected_index < close_index:
        selected = choices[selected_index]
        choice_made = selected.name

        state_changes = apply_journey_effect(
            state, selected, algorithm_params, pool_params, all_dreamsigns,
        )

        print()
        effect_label = _EFFECT_TYPE_LABELS.get(
            selected.effect_type, selected.effect_type.value,
        )
        print(
            f"  {render.BOLD}Selected:{render.RESET} {selected.name}"
            f"  ({effect_label}: {selected.effect_value})"
        )
    else:
        print()
        print(f"  {render.DIM}Closed without selecting a journey.{render.RESET}")

    # Log the interaction
    if logger is not None:
        logger.log_site_visit(
            site_type="DreamJourney",
            dreamscape=dreamscape_name,
            is_enhanced=is_enhanced,
            choices=[j.name for j in choices],
            choice_made=choice_made,
            state_changes=state_changes,
            profile_snapshot=state.resonance_profile.snapshot(),
        )

    # Show resonance profile footer
    print()
    footer = render.resonance_profile_footer(
        counts=state.resonance_profile.snapshot(),
        deck_count=state.deck_count(),
        essence=state.essence,
    )
    print(footer)


def run_tempting_offer(
    state: QuestState,
    all_offers: list[TemptingOffer],
    all_banes: list[BaneCard],
    all_dreamsigns: list[Dreamsign],
    algorithm_params: AlgorithmParams,
    pool_params: PoolParams,
    dreamscape_name: str,
    dreamscape_number: int,
    logger: Optional[SessionLogger],
    is_enhanced: bool = False,
) -> None:
    """Run a Tempting Offer site interaction.

    Selects 2 (or 3 if enhanced) random offer pairs, displays them for
    the player to choose via arrow-key navigation. If selected, applies
    the reward first then the cost, logs the interaction, and shows the
    resonance profile footer.
    """
    choices = select_offers(all_offers, state.rng, is_enhanced)

    # Display header
    header = render.site_visit_header(
        dreamscape_name=dreamscape_name,
        site_type_label="Tempting Offer",
        pick_info="Pick 1 or Decline",
        dreamscape_number=dreamscape_number,
    )
    print(header)
    print()

    # Build options: offers + decline
    option_labels = [o.reward_name for o in choices] + ["Decline All"]
    decline_index = len(choices)

    def _render_fn(index: int, option: str, is_selected: bool) -> str:
        if index == decline_index:
            marker = ">" if is_selected else " "
            return f"  {marker} {render.DIM}Decline All{render.RESET}"
        offer = choices[index]
        lines = _format_offer_option(offer, highlighted=is_selected)
        return "\n".join(lines)

    selected_index = input_handler.single_select(
        options=option_labels,
        render_fn=_render_fn,
    )

    choice_made: Optional[str] = None
    state_changes: dict[str, object] = {}

    if selected_index < decline_index:
        selected = choices[selected_index]
        choice_made = selected.reward_name

        # Apply reward first, then cost
        reward_changes = apply_reward_effect(
            state, selected.reward_effect_type, selected.reward_value,
            algorithm_params, pool_params, all_dreamsigns,
        )
        cost_changes = apply_cost_effect(
            state, selected.cost_effect_type, selected.cost_value,
            all_banes, all_dreamsigns,
        )

        state_changes = {**reward_changes, **cost_changes}

        print()
        reward_label = _EFFECT_TYPE_LABELS.get(
            selected.reward_effect_type, selected.reward_effect_type.value,
        )
        cost_label = _EFFECT_TYPE_LABELS.get(
            selected.cost_effect_type, selected.cost_effect_type.value,
        )
        print(
            f"  {render.BOLD}Selected:{render.RESET} {selected.reward_name}"
        )
        print(
            f"    Reward: {reward_label}: {selected.reward_value}"
        )
        print(
            f"    Cost: {cost_label}: {selected.cost_value}"
        )
    else:
        print()
        print(
            f"  {render.DIM}Declined all offers.{render.RESET}"
        )

    # Log the interaction
    if logger is not None:
        logger.log_site_visit(
            site_type="TemptingOffer",
            dreamscape=dreamscape_name,
            is_enhanced=is_enhanced,
            choices=[o.reward_name for o in choices],
            choice_made=choice_made,
            state_changes=state_changes,
            profile_snapshot=state.resonance_profile.snapshot(),
        )

    # Show resonance profile footer
    print()
    footer = render.resonance_profile_footer(
        counts=state.resonance_profile.snapshot(),
        deck_count=state.deck_count(),
        essence=state.essence,
    )
    print(footer)
