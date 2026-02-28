"""Battle site interaction for the quest simulator.

Implements the Battle site: auto-win combat, essence reward scaled by
completion level, and post-battle rare draft pick (1 of 3 rare+ cards
from the pool). Opponent type varies by completion level: Dream Guardian
for most battles, random miniboss at the miniboss battle, and random
final boss at the last battle.
"""

import logging
import random
from typing import Optional

import algorithm
import input_handler
import pool as pool_module
import render
import render_status
from jsonl_log import SessionLogger
from models import (
    AlgorithmParams,
    Boss,
    Card,
)
from quest_state import QuestState

_log = logging.getLogger(__name__)


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

    # Display dramatic battle header
    battle_number = state.completion_level + 1
    total = quest_config["total_battles"]
    header = render_status.battle_header(
        battle_number=battle_number,
        total_battles=total,
        boss_info=boss_info,
    )
    print(header)
    print()

    # Wait for player to continue after battle introduction
    input_handler.wait_for_continue()

    # Auto-win: show visually distinct victory message
    victory_msg = render_status.battle_victory_message()
    print(victory_msg)
    print()

    # Essence reward
    essence_reward = compute_essence_reward(state.completion_level, battle_config)

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

    # Fall back to any-rarity pool when no rare cards are available
    if not selections and state.pool:
        _log.warning(
            "No rare cards in pool for post-battle draft; falling back to any-rarity pool"
        )
        selections = algorithm.select_cards(
            pool=state.pool,
            n=rare_pick_count,
            profile=state.resonance_profile,
            params=algorithm_params,
            rng=state.rng,
            rare_only=False,
        )

    if selections:
        offered_entries = [entry for entry, _ in selections]
        offered_cards = [entry.card for entry in offered_entries]

        # Display reward summary with essence and rare pick framing
        reward_summary = render_status.battle_reward_summary(
            essence_reward=essence_reward,
            rare_pick_count=len(offered_cards),
        )
        print(reward_summary)
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
        # Pool is completely empty: no cards available at all
        print(f"  Essence reward: {render.BOLD}+{essence_reward}{render.RESET}")
        print()
        print(f"  {render.DIM}No cards available in the pool.{render.RESET}")
        print()

    # Show completion progress
    new_completion = state.completion_level + 1
    progress = render_status.battle_completion_progress(
        new_completion=new_completion,
        total_battles=total,
    )
    print(progress)
    print()

    # Log the battle
    if logger is not None:
        logger.log_battle_complete(
            opponent_name=opponent_name,
            essence_reward=essence_reward,
            rare_pick=picked_card,
        )
        logger.log_site_visit(
            site_type="Battle",
            dreamscape=dreamscape_name,
            choices=[opponent_name],
            choice_made=picked_card.name if picked_card is not None else None,
            state_changes={
                "opponent": opponent_name,
                "essence_reward": essence_reward,
                "rare_pick": picked_card.name if picked_card is not None else None,
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
