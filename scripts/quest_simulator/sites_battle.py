"""Battle site interaction for the quest simulator.

Implements the Battle site: auto-win combat, essence reward scaled by
completion level, and post-battle rare draft pick (1 of 3 rare+ cards
from the pool). Opponent type varies by completion level: Dream Guardian
for most battles, random miniboss at the miniboss battle, and random
final boss at the last battle.
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
    Boss,
    Card,
)
from quest_state import QuestState


def determine_opponent(
    completion_level: int,
    bosses: list[Boss],
    rng: random.Random,
    quest_config: dict[str, int],
) -> tuple[str, Optional[Boss]]:
    """Determine the opponent for the current battle.

    Returns a tuple of (opponent_name, boss_info). For Dream Guardian
    battles, boss_info is None. For miniboss and final boss battles,
    boss_info is the randomly selected Boss object.
    """
    miniboss_level = quest_config["miniboss_battle"] - 1
    final_level = quest_config["total_battles"] - 1

    if completion_level == final_level:
        final_bosses = [b for b in bosses if b.is_final]
        if final_bosses:
            boss = rng.choice(final_bosses)
            return boss.name, boss
        return "Dream Guardian", None

    if completion_level == miniboss_level:
        minibosses = [b for b in bosses if not b.is_final]
        if minibosses:
            boss = rng.choice(minibosses)
            return boss.name, boss
        return "Dream Guardian", None

    return "Dream Guardian", None


def compute_essence_reward(
    completion_level: int,
    battle_config: dict[str, int],
) -> int:
    """Compute the essence reward for a battle at the given completion level."""
    return battle_config["base_essence"] + battle_config["per_level"] * completion_level


def run_battle(
    state: QuestState,
    battle_config: dict[str, int],
    quest_config: dict[str, int],
    algorithm_params: AlgorithmParams,
    bosses: list[Boss],
    dreamscape_name: str,
    dreamscape_number: int,
    logger: Optional[SessionLogger],
) -> None:
    """Run a Battle site interaction.

    Determines the opponent, displays the battle, auto-wins, awards
    essence, offers a rare draft pick, increments completion level,
    and logs the result.
    """
    # Determine opponent
    opponent_name, boss_info = determine_opponent(
        state.completion_level, bosses, state.rng, quest_config
    )

    # Display battle header
    battle_number = state.completion_level + 1
    total = quest_config["total_battles"]
    header = render.site_visit_header(
        dreamscape_name=dreamscape_name,
        site_type_label="Battle",
        pick_info=f"Battle {battle_number}/{total}",
        dreamscape_number=dreamscape_number,
    )
    print(header)
    print()

    # Display opponent details
    if boss_info is not None:
        res_str = render.color_resonances(boss_info.resonances)
        print(f"  {render.BOLD}Opponent: {opponent_name}{render.RESET}")
        print(f"  Archetype: {boss_info.archetype}  ({res_str})")
        print(f"  Ability: {boss_info.ability_text}")
    else:
        print(f"  {render.BOLD}Opponent: {opponent_name}{render.RESET}")
    print()

    # Auto-win message
    print(f"  {render.BOLD}Victory!{render.RESET}")
    print()

    # Essence reward
    essence_reward = compute_essence_reward(state.completion_level, battle_config)
    print(f"  You gain {render.BOLD}{essence_reward}{render.RESET} essence!")
    print()

    # Wait for player to continue after battle display
    input_handler.wait_for_continue()

    # Apply essence reward
    state.gain_essence(essence_reward)

    # Post-battle rare draft pick
    rare_pick_count = battle_config["rare_pick_count"]
    selections = algorithm.select_cards(
        pool=state.pool,
        n=rare_pick_count,
        profile=state.resonance_profile,
        params=algorithm_params,
        rng=state.rng,
        rare_only=True,
    )

    picked_card: Optional[Card] = None

    if selections:
        offered_entries = [entry for entry, _ in selections]
        offered_cards = [entry.card for entry in offered_entries]

        # Display rare draft pick header
        print()
        print(f"  {render.BOLD}Rare Draft Pick{render.RESET} -- Choose 1 card:")
        print()

        option_labels = [card.name for card in offered_cards]

        def _render_card_option(
            index: int,
            option: str,
            is_selected: bool,
            _cards: list[Card] = offered_cards,
        ) -> str:
            card = _cards[index]
            lines = render.format_card(card, highlighted=is_selected)
            return "\n".join(lines)

        chosen_index = input_handler.single_select(
            options=option_labels,
            render_fn=_render_card_option,
        )

        picked_entry = offered_entries[chosen_index]
        picked_card = picked_entry.card

        # Add picked card to deck
        state.add_card(picked_card)

        # Remove picked entry from pool
        pool_module.remove_entry(state.pool, picked_entry)

        print()
        print(f"  Added {render.BOLD}{picked_card.name}{render.RESET} to your deck.")
        print()
    else:
        print()
        print(f"  {render.DIM}No rare cards available in the pool.{render.RESET}")
        print()

    # Log the battle
    if logger is not None:
        logger.log_battle_complete(
            opponent_name=opponent_name,
            essence_reward=essence_reward,
            rare_pick=picked_card,
        )

    # Show resonance profile footer
    footer = render.resonance_profile_footer(
        counts=state.resonance_profile.snapshot(),
        deck_count=state.deck_count(),
        essence=state.essence,
    )
    print(footer)
