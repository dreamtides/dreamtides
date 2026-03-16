"""Battle site interaction for the quest simulator.

Implements the Battle site: auto-win combat, essence reward scaled by
completion level, and post-battle card pick from the draft pack via the
draft strategy. Opponent type varies by completion level: Dream Guardian
for most battles, random miniboss at the miniboss battle, and random
final boss at the last battle.
"""

import random
from typing import Optional

import colors
import input_handler
import log_helpers
import render
import render_cards
import render_status
from jsonl_log import SessionLogger
from models import Boss
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
    bosses: list[Boss],
    dreamscape_name: str,
    dreamscape_number: int,
    logger: Optional[SessionLogger],
) -> None:
    """Run a Battle site interaction.

    Determines the opponent, displays the battle, auto-wins, awards
    essence, offers a post-battle card pick via the draft strategy,
    increments completion level, and logs the result.
    """
    strategy = state.draft_strategy

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

    # Post-battle card pick via draft strategy
    rare_pick_count = battle_config["rare_pick_count"]

    result = strategy.generate_pick(n=rare_pick_count, logger=logger, context="battle")
    shown_cards = result.shown_cards

    picked_card = None

    if shown_cards:
        # Display reward summary with essence
        reward_summary = render_status.battle_reward_summary(
            essence_reward=essence_reward,
        )
        print(reward_summary)
        print()

        # Show card columns (images + text) above interactive selector
        render_cards.render_card_columns(shown_cards)
        print(render.draw_separator())

        option_labels = [render_cards.card_name(card) for card in shown_cards]

        chosen_index = input_handler.single_select(
            options=option_labels,
        )

        picked_card = shown_cards[chosen_index]

        # Signal the draft strategy
        strategy.complete_pick(picked_card, shown_cards)

        # Add picked card to deck
        state.add_card(picked_card)

        # Log preference snapshot after battle pick
        if logger is not None:
            logger.log_preference_snapshot(
                global_pick_index=strategy.pick_index,
                preference_vector=strategy.preference_vector,
                top_archetype_index=log_helpers.top_n_w(strategy.preference_vector, 1)[
                    0
                ][0],
                concentration=log_helpers.w_concentration(strategy.preference_vector),
            )

        print()
        print(
            f"  Added {colors.card(render_cards.card_name(picked_card))} to your deck."
        )
        print()
    else:
        # No cards available in the pack; still advance draft state
        strategy.skip_pick()

        print(
            f"  Essence reward: {colors.c(f'+{essence_reward}', 'accent', bold=True)}"
        )
        print()
        print(f"  {colors.dim('No cards available in the pack.')}")
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
            choice_made=(
                render_cards.card_name(picked_card) if picked_card is not None else None
            ),
            state_changes={
                "opponent": opponent_name,
                "essence_reward": essence_reward,
                "rare_pick": (
                    render_cards.card_name(picked_card)
                    if picked_card is not None
                    else None
                ),
                "deck_size_after": state.deck_count(),
            },
        )

    # Show archetype preference footer
    footer = render_status.archetype_preference_footer(
        w=strategy.preference_vector,
        deck_count=len(state.deck),
        essence=state.essence,
        archetype_draft=state.simple_draft,
    )
    print(footer)
