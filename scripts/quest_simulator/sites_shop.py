"""Shop site interaction for the quest simulator.

Implements the Shop site: 6 items with rarity-based prices, one random
discount, multi-select purchasing, and reroll support. Items are
selected from the draft pool via resonance-weighted selection.
"""

from dataclasses import dataclass
from typing import Any, Optional

import algorithm
import input_handler
import pool as pool_module
import render
from jsonl_log import SessionLogger
from models import (
    AlgorithmParams,
    Card,
    PoolEntry,
    PoolParams,
    Rarity,
    Resonance,
)
from quest_state import QuestState


_RARITY_PRICE_KEYS: dict[Rarity, str] = {
    Rarity.COMMON: "price_common",
    Rarity.UNCOMMON: "price_uncommon",
    Rarity.RARE: "price_rare",
    Rarity.LEGENDARY: "price_legendary",
}


@dataclass(frozen=True)
class ShopItem:
    """A card offered for sale in the shop."""

    card: Card
    pool_entry: PoolEntry
    base_price: int
    discounted_price: Optional[int]

    @property
    def effective_price(self) -> int:
        """The actual price the player pays."""
        if self.discounted_price is not None:
            return self.discounted_price
        return self.base_price


def get_price(rarity: Rarity, shop_config: dict[str, Any]) -> int:
    """Return the base essence price for a card of the given rarity."""
    key = _RARITY_PRICE_KEYS[rarity]
    result: int = shop_config[key]
    return result


def apply_discount(base_price: int, discount_percent: int) -> int:
    """Apply a percentage discount to a base price, rounded to nearest 10.

    The result is clamped to a minimum of 10 essence.
    """
    discount_amount = base_price * discount_percent / 100
    discounted = base_price - discount_amount
    rounded = max(10, round(discounted / 10) * 10)
    return rounded


def generate_shop_items(
    state: QuestState,
    params: AlgorithmParams,
    shop_config: dict[str, Any],
) -> list[ShopItem]:
    """Select items for the shop from the draft pool.

    Selects up to items_count cards via resonance-weighted selection,
    assigns rarity-based prices, and gives one random item a discount.
    """
    items_count: int = shop_config["items_count"]

    selections = algorithm.select_cards(
        pool=state.pool,
        n=items_count,
        profile=state.resonance_profile,
        params=params,
        rng=state.rng,
    )

    if not selections:
        return []

    items: list[ShopItem] = []
    for entry, _weight in selections:
        base_price = get_price(entry.card.rarity, shop_config)
        items.append(ShopItem(
            card=entry.card,
            pool_entry=entry,
            base_price=base_price,
            discounted_price=None,
        ))

    # Apply a random discount to one item
    if items:
        discount_min: int = shop_config["discount_min"]
        discount_max: int = shop_config["discount_max"]
        discount_index = state.rng.randrange(len(items))
        discount_percent = state.rng.randint(discount_min, discount_max)
        old_item = items[discount_index]
        discounted = apply_discount(old_item.base_price, discount_percent)
        items[discount_index] = ShopItem(
            card=old_item.card,
            pool_entry=old_item.pool_entry,
            base_price=old_item.base_price,
            discounted_price=discounted,
        )

    return items


def _format_price(item: ShopItem) -> str:
    """Format a shop item's price display."""
    if item.discounted_price is not None:
        return (
            f"{render.DIM}{render.STRIKETHROUGH}{item.base_price}e"
            f"{render.RESET} "
            f"{render.BOLD}{item.discounted_price}e{render.RESET}"
        )
    return f"{item.base_price}e"


def _render_shop_item(
    item: ShopItem,
    highlighted: bool,
    checked: bool,
    affordable: bool,
) -> str:
    """Render a single shop item as display lines."""
    card_lines = render.format_card(item.card, highlighted=highlighted)
    price_str = _format_price(item)
    check = "[x]" if checked else "[ ]"
    if not affordable and not checked:
        check = f"{render.DIM}[-]{render.RESET}"

    # Replace the default marker with our check marker
    line1 = card_lines[0]
    if highlighted:
        line1 = f"  > {check} " + line1.lstrip(" >")
    else:
        line1 = f"    {check} " + line1.lstrip(" ")

    price_line = f"         Price: {price_str}"
    return "\n".join([line1, card_lines[1], price_line])


def run_shop(
    state: QuestState,
    algorithm_params: AlgorithmParams,
    pool_params: PoolParams,
    shop_config: dict[str, Any],
    dreamscape_name: str,
    dreamscape_number: int,
    logger: Optional[SessionLogger],
    is_enhanced: bool = False,
) -> None:
    """Run a Shop site interaction.

    Displays 6 items with rarity-based prices, one with a random discount.
    The player can multi-select items to purchase or reroll the shop.
    Enhanced shops (Verdant biome) get the first reroll free.
    """
    reroll_cost: int = shop_config["reroll_cost"]
    free_rerolls = 1 if is_enhanced else 0

    # Display header
    header = render.site_visit_header(
        dreamscape_name=dreamscape_name,
        site_type_label="Shop",
        pick_info="Buy items",
        dreamscape_number=dreamscape_number,
    )
    print(header)
    print()

    while True:
        # Attempt pool refill if empty
        if not state.pool:
            pool_module.refill_pool(state.pool, state.all_cards, pool_params)

        items = generate_shop_items(state, algorithm_params, shop_config)

        if not items:
            print(f"  {render.DIM}No items available.{render.RESET}")
            break

        # Build the options list: items + reroll
        option_labels: list[str] = [item.card.name for item in items]

        # Add reroll option
        if free_rerolls > 0:
            reroll_label = f"Reroll (free - {free_rerolls} remaining)"
        else:
            reroll_label = f"Reroll ({reroll_cost} essence)"
        option_labels.append(reroll_label)
        reroll_index = len(option_labels) - 1

        # Display items in 2 rows of 3
        print(f"  {render.BOLD}Shop Items:{render.RESET}")
        print(f"  Essence: {state.essence}")
        print()

        # Track running total for the render callback; reset at each
        # render pass (index 0) and accumulate for checked items.
        running_total = [0]

        def _render_fn(
            index: int,
            option: str,
            is_highlighted: bool,
            is_checked: bool,
            _items: list[ShopItem] = items,
            _reroll_idx: int = reroll_index,
            _running: list[int] = running_total,
        ) -> str:
            # Reset running total at the start of each render pass
            if index == 0:
                _running[0] = 0

            if index == _reroll_idx:
                marker = ">" if is_highlighted else " "
                check = "[x]" if is_checked else "[ ]"
                reroll_affordable = (
                    free_rerolls > 0 or state.essence >= reroll_cost
                )
                if not reroll_affordable and not is_checked:
                    check = f"{render.DIM}[-]{render.RESET}"
                line = f"  {marker} {check} {option}"
                # Show running total on the last line
                if _running[0] > 0:
                    line += (
                        f"\n         {render.BOLD}"
                        f"Total: {_running[0]}e{render.RESET}"
                        f"  (Remaining: {state.essence - _running[0]}e)"
                    )
                return line

            item = _items[index]
            affordable = (
                state.essence - _running[0] >= item.effective_price
            )
            result = _render_shop_item(
                item, is_highlighted, is_checked, affordable
            )

            # Update running total for checked items
            if is_checked:
                _running[0] += item.effective_price

            # Add row separator after every 3rd item
            if index == 2 and len(_items) > 3:
                result += "\n"

            return result

        selected_indices = input_handler.multi_select(
            options=option_labels,
            render_fn=_render_fn,
        )

        # Filter out unaffordable selections: ensure total cost of
        # selected items does not exceed available essence.
        affordable_indices: list[int] = []
        remaining_essence = state.essence
        for i in selected_indices:
            if i == reroll_index:
                affordable_indices.append(i)
                continue
            if i < len(items):
                cost = items[i].effective_price
                if remaining_essence >= cost:
                    remaining_essence -= cost
                    affordable_indices.append(i)
        selected_indices = affordable_indices

        # Check if reroll was selected
        if reroll_index in selected_indices:
            if free_rerolls > 0:
                free_rerolls -= 1
            elif state.essence >= reroll_cost:
                state.spend_essence(reroll_cost)
            else:
                print()
                print(
                    f"  {render.DIM}Not enough essence to reroll "
                    f"({reroll_cost} needed, {state.essence} available)."
                    f"{render.RESET}"
                )
                continue
            print()
            print(f"  {render.BOLD}Rerolling shop...{render.RESET}")
            print()
            continue

        # Process purchases
        bought_items: list[ShopItem] = [
            items[i] for i in selected_indices if i < len(items)
        ]

        total_cost = sum(item.effective_price for item in bought_items)

        # Spend essence first to ensure atomicity -- if the player cannot
        # afford the total, no state mutations occur.
        if total_cost > 0:
            state.spend_essence(total_cost)

        for item in bought_items:
            state.add_card(item.card)
            pool_module.remove_entry(state.pool, item.pool_entry)

        # Log the interaction
        if logger is not None:
            logger.log_shop_purchase(
                items_shown=[item.card for item in items],
                items_bought=[item.card for item in bought_items],
                essence_spent=total_cost,
            )

        if bought_items:
            print()
            print(
                f"  Purchased {len(bought_items)} card(s) "
                f"for {total_cost} essence."
            )
            print(f"  Essence remaining: {state.essence}")

        break

    # Show resonance profile footer
    footer = render.resonance_profile_footer(
        counts=state.resonance_profile.snapshot(),
        deck_count=state.deck_count(),
        essence=state.essence,
    )
    print(footer)
