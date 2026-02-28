"""Duplication, Reward, and Cleanse site interactions.

Implements three smaller site types that round out the quest experience:
- Duplication: copies cards in the deck
- Reward: grants pre-determined rewards based on completion level
- Cleanse: removes bane cards and bane dreamsigns
"""

import random
from typing import Any, Optional

import input_handler
import render
from jsonl_log import SessionLogger
from models import Card, DeckCard, Dreamsign, Rarity, Resonance
from quest_state import QuestState

SKIP_LABEL = "Skip"
DUPLICATION_NORMAL_COUNT = 3
DUPLICATION_MIN_COPIES = 1
DUPLICATION_MAX_COPIES = 4
CLEANSE_MAX_BANES = 3
REWARD_ESSENCE_MIN = 150
REWARD_ESSENCE_MAX = 250
REWARD_LOW_THRESHOLD = 2
REWARD_HIGH_THRESHOLD = 5


def select_duplication_candidates(
    deck: list[DeckCard],
    rng: random.Random,
    enhanced: bool = False,
) -> tuple[list[DeckCard], list[int]]:
    """Select cards from the deck as duplication candidates.

    Normal mode: selects up to 3 random cards from the deck, each with
    a random copy count (1-4).
    Enhanced (Mirrored): returns the full deck with copy counts for all.

    Returns a tuple of (candidates, copy_counts) where both lists have
    the same length.
    """
    if not deck:
        return [], []

    if enhanced:
        candidates = list(deck)
    else:
        n = min(DUPLICATION_NORMAL_COUNT, len(deck))
        candidates = rng.sample(deck, n)

    copy_counts = [
        rng.randint(DUPLICATION_MIN_COPIES, DUPLICATION_MAX_COPIES) for _ in candidates
    ]
    return candidates, copy_counts


def run_duplication(
    state: QuestState,
    dreamscape_name: str,
    dreamscape_number: int,
    logger: Optional[SessionLogger],
    is_enhanced: bool = False,
) -> None:
    """Run a Duplication site interaction.

    Normal: show 3 random cards from deck with random copy counts (1-4).
    Pick 1 or skip. Adds copies to deck.
    Enhanced (Mirrored): choose any card from the full deck.
    """
    if not state.deck:
        print(f"  {render.DIM}Deck is empty -- nothing to duplicate.{render.RESET}")
        footer = render.resonance_profile_footer(
            counts=state.resonance_profile.snapshot(),
            deck_count=state.deck_count(),
            essence=state.essence,
        )
        print(footer)
        return

    candidates, copy_counts = select_duplication_candidates(
        state.deck, state.rng, enhanced=is_enhanced
    )

    # Display header
    enhanced_label = " (Enhanced)" if is_enhanced else ""
    header = render.site_visit_header(
        dreamscape_name=dreamscape_name,
        site_type_label=f"Duplication{enhanced_label}",
        pick_info="Choose a card to duplicate",
        dreamscape_number=dreamscape_number,
    )
    print(header)
    print()

    if is_enhanced:
        print(f"  {render.BOLD}Mirrored:{render.RESET} Choose any card to duplicate.")
    else:
        print("  Select a card to duplicate or skip:")
    print()

    # Build option labels with copy counts
    option_labels = [f"{dc.card.name} x{cc}" for dc, cc in zip(candidates, copy_counts)]
    option_labels.append(SKIP_LABEL)

    def _render_option(
        index: int,
        option: str,
        is_selected: bool,
        _candidates: list[DeckCard] = candidates,
        _copy_counts: list[int] = copy_counts,
    ) -> str:
        if index < len(_candidates):
            dc = _candidates[index]
            cc = _copy_counts[index]
            card_lines = render.format_card(dc.card, highlighted=is_selected)
            marker = ">" if is_selected else " "
            line1 = f"  {marker} {dc.card.name} x{cc}"
            # Replace the first line with the card format but add copy info
            formatted_line1 = card_lines[0]
            copy_label = f" x{cc}"
            if is_selected:
                formatted_line1 = (
                    formatted_line1 + f"  {render.BOLD}{copy_label}{render.RESET}"
                )
            else:
                formatted_line1 = (
                    formatted_line1 + f"  {render.DIM}{copy_label}{render.RESET}"
                )
            return "\n".join([formatted_line1, card_lines[1]])
        marker = ">" if is_selected else " "
        return f"  {marker} {render.DIM}{SKIP_LABEL}{render.RESET}"

    selected_index = input_handler.single_select(
        options=option_labels,
        render_fn=_render_option,
    )

    chosen_card: Optional[Card] = None
    chosen_copies: int = 0

    if selected_index < len(candidates):
        chosen_dc = candidates[selected_index]
        chosen_card = chosen_dc.card
        chosen_copies = copy_counts[selected_index]

        # Add copies to deck
        for _ in range(chosen_copies):
            state.add_card(chosen_card)

        print()
        print(
            f"  {render.BOLD}Duplicated:{render.RESET} {chosen_card.name} "
            f"x{chosen_copies} added to deck."
        )
    else:
        print()
        print(f"  {render.DIM}Skipped.{render.RESET}")

    # Log the interaction
    if logger is not None:
        logger.log_site_visit(
            site_type="Duplication",
            dreamscape=dreamscape_name,
            is_enhanced=is_enhanced,
            choices=[dc.card.name for dc in candidates],
            choice_made=(
                f"{chosen_card.name} x{chosen_copies}"
                if chosen_card is not None
                else None
            ),
            state_changes={
                "cards_added": chosen_copies,
                "card_duplicated": (
                    chosen_card.name if chosen_card is not None else None
                ),
                "deck_size_after": state.deck_count(),
            },
            profile_snapshot=state.resonance_profile.snapshot(),
        )

    # Show resonance profile footer
    footer = render.resonance_profile_footer(
        counts=state.resonance_profile.snapshot(),
        deck_count=state.deck_count(),
        essence=state.essence,
    )
    print(footer)


def generate_reward(
    completion_level: int,
    rng: random.Random,
    all_cards: list[Card],
    all_dreamsigns: Optional[list[Dreamsign]] = None,
) -> dict[str, Any]:
    """Generate a pre-determined reward based on completion level.

    Low levels (0-1): essence reward (150-250).
    Mid levels (2-4): card reward (weighted rare+ from pool).
    High levels (5+): dreamsign reward (random non-bane).

    Returns a dict with 'type', 'value', and optionally 'card' or 'dreamsign'.
    """
    if completion_level < REWARD_LOW_THRESHOLD:
        # Low level: essence
        amount = rng.randint(REWARD_ESSENCE_MIN, REWARD_ESSENCE_MAX)
        return {"type": "essence", "value": amount}
    elif completion_level < REWARD_HIGH_THRESHOLD:
        # Mid level: card (weighted toward rare+)
        rare_plus = [
            c for c in all_cards if c.rarity in (Rarity.RARE, Rarity.LEGENDARY)
        ]
        if rare_plus:
            card = rng.choice(rare_plus)
            return {"type": "card", "value": 0, "card": card}
        # Fallback to essence if no rare+ cards available
        amount = rng.randint(REWARD_ESSENCE_MIN, REWARD_ESSENCE_MAX)
        return {"type": "essence", "value": amount}
    else:
        # High level: dreamsign
        available = all_dreamsigns or []
        non_bane = [ds for ds in available if not ds.is_bane]
        if non_bane:
            ds = rng.choice(non_bane)
            return {"type": "dreamsign", "value": 0, "dreamsign": ds}
        # Fallback to essence if no dreamsigns available
        amount = rng.randint(REWARD_ESSENCE_MIN, REWARD_ESSENCE_MAX)
        return {"type": "essence", "value": amount}


def _format_reward_description(reward: dict[str, Any]) -> str:
    """Format a reward for display."""
    reward_type = reward["type"]
    if reward_type == "essence":
        return f"{render.BOLD}{reward['value']}{render.RESET} essence"
    elif reward_type == "card":
        card: Card = reward["card"]
        res_str = render.color_resonances(card.resonances)
        badge = render.rarity_badge(card.rarity)
        return f"{card.name} {res_str} {badge}"
    elif reward_type == "dreamsign":
        ds: Dreamsign = reward["dreamsign"]
        res_str = render.color_resonance(ds.resonance)
        return f"{ds.name} ({res_str})"
    return "Unknown reward"


def run_reward(
    state: QuestState,
    dreamscape_name: str,
    dreamscape_number: int,
    logger: Optional[SessionLogger],
    all_dreamsigns: list[Dreamsign],
) -> None:
    """Run a Reward site interaction.

    Generates a pre-determined reward based on completion level.
    Displays the reward and lets the player accept or decline.
    """
    reward = generate_reward(
        completion_level=state.completion_level,
        rng=state.rng,
        all_cards=state.all_cards,
        all_dreamsigns=all_dreamsigns,
    )

    # Display header
    header = render.site_visit_header(
        dreamscape_name=dreamscape_name,
        site_type_label="Reward Site",
        pick_info="Accept or decline",
        dreamscape_number=dreamscape_number,
    )
    print(header)
    print()

    # Display the reward
    description = _format_reward_description(reward)
    print(f"  Reward: {description}")
    print()

    # Confirm/decline
    accepted = input_handler.confirm_decline()

    if accepted:
        reward_type = reward["type"]
        if reward_type == "essence":
            state.gain_essence(reward["value"])
            print()
            print(f"  {render.BOLD}Gained {reward['value']} essence!{render.RESET}")
        elif reward_type == "card":
            card = reward["card"]
            state.add_card(card)
            print()
            print(f"  {render.BOLD}Added {card.name} to deck!{render.RESET}")
        elif reward_type == "dreamsign":
            ds = reward["dreamsign"]
            state.add_dreamsign(ds)
            print()
            print(f"  {render.BOLD}Acquired dreamsign: {ds.name}!{render.RESET}")
    else:
        print()
        print(f"  {render.DIM}Declined.{render.RESET}")

    # Log the interaction
    if logger is not None:
        choice_str = None
        if accepted:
            if reward["type"] == "essence":
                choice_str = f"essence:{reward['value']}"
            elif reward["type"] == "card":
                choice_str = reward["card"].name
            elif reward["type"] == "dreamsign":
                choice_str = reward["dreamsign"].name

        logger.log_site_visit(
            site_type="RewardSite",
            dreamscape=dreamscape_name,
            choices=[_format_reward_description(reward)],
            choice_made=choice_str,
            state_changes={
                "reward_type": reward["type"],
                "accepted": accepted,
                "essence_after": state.essence,
                "deck_size_after": state.deck_count(),
            },
            profile_snapshot=state.resonance_profile.snapshot(),
        )

    # Show resonance profile footer
    footer = render.resonance_profile_footer(
        counts=state.resonance_profile.snapshot(),
        deck_count=state.deck_count(),
        essence=state.essence,
    )
    print(footer)


def find_bane_items(
    state: QuestState,
) -> tuple[list[DeckCard], list[Dreamsign]]:
    """Find all bane cards and bane dreamsigns in the player's collection.

    Returns a tuple of (bane_deck_cards, bane_dreamsigns).
    """
    bane_deck_cards = [dc for dc in state.deck if dc.is_bane]
    bane_dreamsigns = [ds for ds in state.dreamsigns if ds.is_bane]
    return bane_deck_cards, bane_dreamsigns


def run_cleanse(
    state: QuestState,
    dreamscape_name: str,
    dreamscape_number: int,
    logger: Optional[SessionLogger],
) -> None:
    """Run a Cleanse site interaction.

    Finds bane cards and bane dreamsigns. If none, auto-completes.
    If banes exist, shows up to 3 random bane items and lets the
    player confirm or decline removal of all shown banes.
    """
    bane_cards, bane_dreamsigns = find_bane_items(state)

    # Display header
    header = render.site_visit_header(
        dreamscape_name=dreamscape_name,
        site_type_label="Cleanse",
        pick_info="Remove bane effects",
        dreamscape_number=dreamscape_number,
    )
    print(header)
    print()

    if not bane_cards and not bane_dreamsigns:
        # No banes: auto-complete
        print(f"  {render.DIM}No bane effects found.{render.RESET}")
        print()
        input_handler.wait_for_continue()

        if logger is not None:
            logger.log_site_visit(
                site_type="Cleanse",
                dreamscape=dreamscape_name,
                choices=[],
                choice_made=None,
                state_changes={
                    "banes_removed": 0,
                    "deck_size_after": state.deck_count(),
                },
                profile_snapshot=state.resonance_profile.snapshot(),
            )

        footer = render.resonance_profile_footer(
            counts=state.resonance_profile.snapshot(),
            deck_count=state.deck_count(),
            essence=state.essence,
        )
        print(footer)
        return

    # Combine all bane items and limit to 3
    all_bane_items: list[tuple[str, DeckCard | None, Dreamsign | None]] = []
    for dc in bane_cards:
        all_bane_items.append((f"Card: {dc.card.name}", dc, None))
    for ds in bane_dreamsigns:
        all_bane_items.append((f"Dreamsign: {ds.name}", None, ds))

    # Select up to 3 random bane items
    n = min(CLEANSE_MAX_BANES, len(all_bane_items))
    shown_items = state.rng.sample(all_bane_items, n)

    print("  The following bane effects can be removed:")
    print()
    for label, _dc, _ds in shown_items:
        print(f"    - {label}")
    print()

    # Confirm/decline
    accepted = input_handler.confirm_decline(
        accept_label="Cleanse",
        decline_label="Decline",
    )

    removed_names: list[str] = []

    if accepted:
        for label, dc, ds in shown_items:
            if dc is not None:
                state.remove_card(dc)
                removed_names.append(dc.card.name)
            if ds is not None:
                state.remove_dreamsign(ds)
                removed_names.append(ds.name)

        print()
        print(
            f"  {render.BOLD}Cleansed {len(removed_names)} bane effect(s).{render.RESET}"
        )
    else:
        print()
        print(f"  {render.DIM}Declined.{render.RESET}")

    # Log the interaction
    if logger is not None:
        logger.log_site_visit(
            site_type="Cleanse",
            dreamscape=dreamscape_name,
            choices=[label for label, _, _ in shown_items],
            choice_made=", ".join(removed_names) if removed_names else None,
            state_changes={
                "banes_removed": len(removed_names),
                "items_removed": removed_names,
                "deck_size_after": state.deck_count(),
                "dreamsign_count_after": state.dreamsign_count(),
            },
            profile_snapshot=state.resonance_profile.snapshot(),
        )

    # Show resonance profile footer
    footer = render.resonance_profile_footer(
        counts=state.resonance_profile.snapshot(),
        deck_count=state.deck_count(),
        essence=state.essence,
    )
    print(footer)
