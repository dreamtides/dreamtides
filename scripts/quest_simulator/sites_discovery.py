"""Discovery Draft and Specialty Shop site interactions.

Discovery Draft draws cards from the CubeManager, filters by archetype
fitness for the player's top archetype(s), and offers 4 cards to pick
from. Enhanced (Arcane biome) allows picking any number of offered
cards. Neither site advances the draft pick counter.

Specialty Shop draws from the CubeManager, filters by archetype
fitness, and uses power-based pricing. Reroll re-draws from the cube.
"""

import random
from dataclasses import dataclass
from typing import Optional

import agents
from draft_models import CardInstance
from input_handler import multi_select, single_select
from jsonl_log import SessionLogger
from models import Dreamsign
from quest_state import QuestState
from render import (
    format_card,
    site_visit_header,
    BOLD,
    DIM,
    RESET,
)
import render_status

_FITNESS_THRESHOLD = 0.7
_DRAW_BATCH_SIZE = 30
_CARDS_PER_PICK = 4
_PICKS_PER_SITE = 2
_MIN_PRICE = 5
_MAX_PRICE = 100


@dataclass
class ShopItem:
    """A single item in a shop display with pricing."""

    instance: CardInstance
    base_price: int
    discounted_price: Optional[int]

    @property
    def effective_price(self) -> int:
        """Return the effective price (discounted if applicable)."""
        if self.discounted_price is not None:
            return self.discounted_price
        return self.base_price


def _top_archetypes(w: list[float], max_count: int = 2) -> list[int]:
    """Return indices of the top archetype(s) from the preference vector.

    Always returns the top archetype. Includes the second-best if it
    is within 50% of the top weight.
    """
    if not w:
        return []
    indexed = sorted(range(len(w)), key=lambda i: w[i], reverse=True)
    result = [indexed[0]]
    if max_count >= 2 and len(indexed) >= 2:
        top_val = w[indexed[0]]
        second_val = w[indexed[1]]
        if top_val > 0 and second_val >= top_val * 0.5:
            result.append(indexed[1])
    return result


def _has_high_fitness(
    card: CardInstance,
    archetype_indices: list[int],
    threshold: float,
) -> bool:
    """Return True if card has fitness >= threshold for any of the given archetypes."""
    fitness = card.design.fitness
    for idx in archetype_indices:
        if idx < len(fitness) and fitness[idx] >= threshold:
            return True
    return False


def draw_and_filter(
    state: QuestState,
    count: int,
    batch_size: int = _DRAW_BATCH_SIZE,
) -> list[CardInstance]:
    """Draw cards from the cube and filter by archetype fitness.

    Draws a batch from the CubeManager, filters to cards with
    fitness >= 0.7 for the player's top archetype(s), and returns
    up to `count` filtered cards. If not enough high-fitness cards
    are found, relaxes the threshold or returns what is available.
    """
    top_archs = _top_archetypes(state.human_agent.w)
    available = state.cube.remaining
    first_draw = min(batch_size, available)
    if first_draw <= 0:
        return []

    drawn = state.cube.draw(first_draw, state.rng)

    # Filter to high-fitness cards
    filtered = [
        card for card in drawn if _has_high_fitness(card, top_archs, _FITNESS_THRESHOLD)
    ]

    if len(filtered) >= count:
        return filtered[:count]

    # Not enough: try drawing more
    second_draw = min(batch_size, state.cube.remaining)
    extra: list[CardInstance] = []
    if second_draw > 0:
        extra = state.cube.draw(second_draw, state.rng)
    for card in extra:
        if _has_high_fitness(card, top_archs, _FITNESS_THRESHOLD):
            filtered.append(card)
        if len(filtered) >= count:
            break

    if len(filtered) >= count:
        return filtered[:count]

    # Still not enough: relax threshold to 0.4
    relaxed_threshold = 0.4
    relaxed = [
        card
        for card in drawn + extra
        if _has_high_fitness(card, top_archs, relaxed_threshold)
    ]

    if relaxed:
        return relaxed[:count]

    # Last resort: return whatever was drawn
    return (drawn + extra)[:count]


def compute_power_price(power: float) -> int:
    """Compute price based on card power: round(power * 25) clamped to [5, 100]."""
    raw = round(power * 25)
    return max(_MIN_PRICE, min(_MAX_PRICE, raw))


def _apply_discount(
    base_price: int,
    rng: random.Random,
    discount_min: int,
    discount_max: int,
) -> int:
    """Apply a random discount percentage and round to nearest 10.

    The result is clamped so it never exceeds the base price.
    """
    discount_pct = rng.randint(discount_min, discount_max)
    discounted = base_price * (100 - discount_pct) / 100
    rounded = max(_MIN_PRICE, round(discounted / 10) * 10)
    return min(rounded, base_price)


def prepare_shop_items(
    instances: list[CardInstance],
    rng: random.Random,
    shop_config: dict[str, int],
) -> list[ShopItem]:
    """Prepare shop items with power-based prices and per-item discount chance."""
    items: list[ShopItem] = []
    for inst in instances:
        base_price = compute_power_price(inst.design.power)
        items.append(
            ShopItem(
                instance=inst,
                base_price=base_price,
                discounted_price=None,
            )
        )

    if items:
        discount_chance = shop_config.get("discount_chance", 30) / 100.0
        for item in items:
            if rng.random() < discount_chance:
                item.discounted_price = _apply_discount(
                    item.base_price,
                    rng,
                    shop_config["discount_min"],
                    shop_config["discount_max"],
                )

    return items


def _update_human_agent(state: QuestState, card: CardInstance) -> None:
    """Update the human agent after picking a card.

    Calls update_agent_after_pick with a synthetic pack context since
    Discovery Draft and Specialty Shop operate outside the draft loop.
    """
    cfg = state.draft_cfg
    agents.update_agent_after_pick(
        state.human_agent,
        card,
        [],  # No visible remaining cards from a pack
        state.global_pick_index,
        state.round_index,
        "discovery",  # Synthetic pack_id
        cfg.agents.learning_rate,
        cfg.agents.openness_window,
    )


def run_discovery_draft(
    state: QuestState,
    logger: Optional[SessionLogger],
    dreamscape_name: str,
    dreamscape_number: int,
    is_enhanced: bool,
    picks_per_site: int = _PICKS_PER_SITE,
    cards_per_pick: int = _CARDS_PER_PICK,
) -> None:
    """Run a Discovery Draft site interaction.

    Draws cards from the CubeManager, filters by archetype fitness,
    and offers cards to pick from. Does NOT advance the draft pick
    counter. Normal: pick 1. Enhanced (Arcane): pick any number.
    """
    for pick_num in range(1, picks_per_site + 1):
        offered = draw_and_filter(state, count=cards_per_pick)

        if not offered:
            print(f"  {DIM}No cards available.{RESET}")
            break

        # Build header
        label = "Discovery Draft"
        pick_info = f"Pick {pick_num}/{picks_per_site}"
        header = site_visit_header(dreamscape_name, label, pick_info, dreamscape_number)
        print(header)
        print()

        card_names = [inst.design.name for inst in offered]

        if is_enhanced:

            def _render_multi(
                idx: int, option: str, highlighted: bool, checked: bool
            ) -> str:
                lines = format_card(offered[idx], highlighted=highlighted)
                check = "[x]" if checked else "[ ]"
                marker = ">" if highlighted else " "
                lines[0] = lines[0].replace(
                    f"  {'>' if highlighted else ' '} ",
                    f"  {marker} {check} ",
                    1,
                )
                return "\n".join(lines)

            selected_indices = multi_select(card_names, render_fn=_render_multi)
        else:

            def _render_single(idx: int, option: str, highlighted: bool) -> str:
                lines = format_card(offered[idx], highlighted=highlighted)
                return "\n".join(lines)

            selected_idx = single_select(card_names, render_fn=_render_single)
            selected_indices = [selected_idx]

        # Process picks
        for idx in selected_indices:
            card = offered[idx]
            state.add_card(card)
            _update_human_agent(state, card)

        # Log picks
        if logger is not None:
            if selected_indices:
                for idx in selected_indices:
                    card = offered[idx]
                    logger.log_site_visit(
                        site_type="DiscoveryDraft",
                        dreamscape=dreamscape_name,
                        is_enhanced=is_enhanced,
                        choices=[inst.design.name for inst in offered],
                        choice_made=card.design.name,
                        state_changes={
                            "pick_num": pick_num,
                            "deck_size_after": state.deck_count(),
                        },
                    )
            else:
                logger.log_site_visit(
                    site_type="DiscoveryDraft",
                    dreamscape=dreamscape_name,
                    is_enhanced=is_enhanced,
                    choices=[inst.design.name for inst in offered],
                    choice_made=None,
                    state_changes={
                        "pick_num": pick_num,
                        "deck_size_after": state.deck_count(),
                    },
                )

    # Show archetype preference footer
    print(
        render_status.archetype_preference_footer(
            w=state.human_agent.w,
            deck_count=state.deck_count(),
            essence=state.essence,
        )
    )


def run_specialty_shop(
    state: QuestState,
    logger: Optional[SessionLogger],
    dreamscape_name: str,
    dreamscape_number: int,
    is_enhanced: bool,
    shop_config: dict[str, int],
    all_dreamsigns: Optional[list[Dreamsign]] = None,
) -> None:
    """Run a Specialty Shop site interaction.

    Draws from the CubeManager, filters by archetype fitness, displays
    items with power-based prices. Includes a dreamsign offering if
    dreamsigns are available. Does NOT advance the draft pick counter.
    Reroll re-draws from the cube.
    """
    items_count = shop_config.get("items_count", 4)
    reroll_cost = shop_config.get("reroll_cost", 50)

    # Select a dreamsign offering if dreamsigns are available
    dreamsign_offer: Optional[Dreamsign] = None
    if all_dreamsigns:
        non_bane = [ds for ds in all_dreamsigns if not ds.is_bane]
        if non_bane:
            dreamsign_offer = state.rng.choice(non_bane)

    while True:
        drawn = draw_and_filter(state, count=items_count)

        if not drawn:
            print(f"  {DIM}No items available.{RESET}")
            break

        items = prepare_shop_items(drawn, state.rng, shop_config)

        # Build header
        label = "Specialty Shop"
        header = site_visit_header(dreamscape_name, label, "Browse", dreamscape_number)
        print(header)
        print()

        # Build option names including prices
        option_names: list[str] = []
        for item in items:
            if item.discounted_price is not None:
                price_str = f"{item.discounted_price}e (was {item.base_price}e)"
            else:
                price_str = f"{item.base_price}e"
            option_names.append(f"{item.instance.design.name} -- {price_str}")

        # Add dreamsign offering if available
        dreamsign_idx: Optional[int] = None
        if dreamsign_offer is not None:
            dreamsign_idx = len(option_names)
            option_names.append(f"Dreamsign: {dreamsign_offer.name} (free)")

        # Add reroll option
        reroll_idx = len(option_names)
        can_afford_reroll = state.essence >= reroll_cost
        reroll_label = f"Reroll ({reroll_cost}e)"
        if not can_afford_reroll:
            reroll_label = f"{DIM}Reroll ({reroll_cost}e) -- not enough essence{RESET}"
        option_names.append(reroll_label)

        # Multi-select for purchasing
        def _render_shop(
            idx: int,
            option: str,
            highlighted: bool,
            checked: bool,
            _items: list[ShopItem] = items,
            _dreamsign_idx: Optional[int] = dreamsign_idx,
            _dreamsign_offer: Optional[Dreamsign] = dreamsign_offer,
        ) -> str:
            if idx < len(_items):
                shop_item = _items[idx]
                lines = format_card(shop_item.instance, highlighted=highlighted)
                check = "[x]" if checked else "[ ]"
                marker = ">" if highlighted else " "

                if shop_item.discounted_price is not None:
                    price_line = (
                        f"  {BOLD}{shop_item.discounted_price}e{RESET}"
                        f" {DIM}(was {shop_item.base_price}e){RESET}"
                    )
                else:
                    price_line = f"  {shop_item.base_price}e"

                lines[0] = lines[0].replace(
                    f"  {'>' if highlighted else ' '} ",
                    f"  {marker} {check} ",
                    1,
                )
                return "\n".join(lines) + f"\n    {price_line}"
            elif _dreamsign_idx is not None and idx == _dreamsign_idx:
                marker = ">" if highlighted else " "
                check = "[x]" if checked else "[ ]"
                ds_name = _dreamsign_offer.name if _dreamsign_offer else ""
                line1 = f"  {marker} {check} {BOLD}Dreamsign:{RESET} {ds_name}"
                line2 = (
                    f'      "{_dreamsign_offer.effect_text}"'
                    if _dreamsign_offer
                    else ""
                )
                line3 = f"      {DIM}(free){RESET}"
                return "\n".join([line1, line2, line3])
            else:
                marker = ">" if highlighted else " "
                return f"\n  {marker} {option}"

        toggled = multi_select(option_names, render_fn=_render_shop)

        # Check if reroll was selected
        if reroll_idx in toggled:
            if can_afford_reroll:
                state.spend_essence(reroll_cost)
                continue
            else:
                toggled = [i for i in toggled if i != reroll_idx]

        # Handle dreamsign selection
        acquired_dreamsign: Optional[Dreamsign] = None
        if dreamsign_idx is not None and dreamsign_idx in toggled:
            acquired_dreamsign = dreamsign_offer
            toggled = [i for i in toggled if i != dreamsign_idx]

        # Filter to item indices only
        purchase_indices = [i for i in toggled if i < len(items)]

        # Compute total cost
        total = sum(items[i].effective_price for i in purchase_indices)

        # Check affordability
        if total > state.essence:
            print(f"  Cannot afford {total}e (have {state.essence}e).")
            continue

        # Apply purchases
        purchased: list[CardInstance] = []
        for idx in purchase_indices:
            shop_item = items[idx]
            state.add_card(shop_item.instance)
            _update_human_agent(state, shop_item.instance)
            purchased.append(shop_item.instance)
            state.spend_essence(shop_item.effective_price)

        # Apply dreamsign acquisition
        if acquired_dreamsign is not None:
            state.add_dreamsign(acquired_dreamsign)

        # Log the shop interaction
        if logger is not None:
            logger.log_site_visit(
                site_type="SpecialtyShop",
                dreamscape=dreamscape_name,
                is_enhanced=is_enhanced,
                choices=[item.instance.design.name for item in items],
                choice_made=(
                    ", ".join(c.design.name for c in purchased) if purchased else None
                ),
                state_changes={
                    "items_bought": [c.design.name for c in purchased],
                    "essence_spent": total,
                    "deck_size_after": state.deck_count(),
                    "dreamsign_acquired": (
                        acquired_dreamsign.name
                        if acquired_dreamsign is not None
                        else None
                    ),
                },
            )
        break

    # Show archetype preference footer
    print(
        render_status.archetype_preference_footer(
            w=state.human_agent.w,
            deck_count=state.deck_count(),
            essence=state.essence,
        )
    )
