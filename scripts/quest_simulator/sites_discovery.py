"""Discovery Draft and Specialty Shop site interactions.

Discovery Draft uses the tag system to select a theme, filters the pool
to cards matching that tag, and offers 4 cards to pick from. Enhanced
(Arcane biome) allows picking any number of offered cards.

Specialty Shop is the same as a regular Shop but items are filtered to
a tag-selected theme. One item gets a random discount.
"""

import random
from dataclasses import dataclass
from typing import Optional

from algorithm import select_cards
from input_handler import multi_select, single_select
from jsonl_log import SessionLogger
from models import (
    AlgorithmParams,
    Card,
    PoolEntry,
    Rarity,
    ResonanceProfile,
    TagProfile,
)
from pool import increment_staleness, remove_entry
from quest_state import QuestState
from render import (
    format_card,
    resonance_profile_footer,
    site_visit_header,
    BOLD,
    DIM,
    RESET,
)
from tags import filter_pool_by_tag, select_theme


@dataclass
class ShopItem:
    """A single item in a shop display with pricing."""

    entry: PoolEntry
    base_price: int
    discounted_price: Optional[int]


def _compute_price(rarity: Rarity, shop_config: dict[str, int]) -> int:
    """Return the base price for a card of the given rarity."""
    price_map: dict[Rarity, int] = {
        Rarity.COMMON: shop_config["price_common"],
        Rarity.UNCOMMON: shop_config["price_uncommon"],
        Rarity.RARE: shop_config["price_rare"],
        Rarity.LEGENDARY: shop_config["price_legendary"],
    }
    return price_map[rarity]


def _apply_discount(
    base_price: int,
    rng: random.Random,
    discount_min: int,
    discount_max: int,
) -> int:
    """Apply a random discount percentage and round to nearest 10."""
    discount_pct = rng.randint(discount_min, discount_max)
    discounted = base_price * (100 - discount_pct) / 100
    return max(10, round(discounted / 10) * 10)


def _select_discovery_cards(
    pool: list[PoolEntry],
    resonance_profile: ResonanceProfile,
    tag_profile: TagProfile,
    params: AlgorithmParams,
    rng: random.Random,
    cards_per_pick: int,
    tag_config: dict[str, float],
) -> Optional[tuple[list[tuple[PoolEntry, float]], Optional[str]]]:
    """Select cards for a discovery draft pick.

    Returns (selected_entries_with_weights, theme_tag) or None if pool
    is empty. Theme tag is None when falling back to unthemed selection.
    """
    if not pool:
        return None

    theme = select_theme(
        pool=pool,
        profile=tag_profile,
        rng=rng,
        min_theme_cards=int(tag_config["min_theme_cards"]),
        tag_scale=tag_config["scale"],
        relevance_boost=tag_config["relevance_boost"],
        depth_factor=tag_config["depth_factor"],
    )

    if theme is not None:
        filtered = filter_pool_by_tag(pool, theme)
        selected = select_cards(
            filtered, cards_per_pick, resonance_profile, params, rng
        )
        if selected:
            return (selected, theme)

    # Fallback to unthemed selection
    selected = select_cards(pool, cards_per_pick, resonance_profile, params, rng)
    if selected:
        return (selected, None)

    return None


def _apply_discovery_pick(
    offered: list[tuple[PoolEntry, float]],
    picked_indices: list[int],
) -> tuple[list[PoolEntry], list[PoolEntry]]:
    """Split offered entries into picked and unpicked based on indices.

    Returns (picked_entries, unpicked_entries).
    """
    picked_set = set(picked_indices)
    picked = [offered[i][0] for i in range(len(offered)) if i in picked_set]
    unpicked = [offered[i][0] for i in range(len(offered)) if i not in picked_set]
    return (picked, unpicked)


def _select_specialty_items(
    pool: list[PoolEntry],
    resonance_profile: ResonanceProfile,
    tag_profile: TagProfile,
    params: AlgorithmParams,
    rng: random.Random,
    items_count: int,
    tag_config: dict[str, float],
) -> Optional[tuple[list[tuple[PoolEntry, float]], Optional[str]]]:
    """Select items for a specialty shop.

    Returns (selected_entries_with_weights, theme_tag) or None if pool
    is empty. Theme tag is None when falling back to unthemed selection.
    """
    if not pool:
        return None

    theme = select_theme(
        pool=pool,
        profile=tag_profile,
        rng=rng,
        min_theme_cards=int(tag_config["min_theme_cards"]),
        tag_scale=tag_config["scale"],
        relevance_boost=tag_config["relevance_boost"],
        depth_factor=tag_config["depth_factor"],
    )

    if theme is not None:
        filtered = filter_pool_by_tag(pool, theme)
        selected = select_cards(
            filtered, items_count, resonance_profile, params, rng
        )
        if selected:
            return (selected, theme)

    # Fallback to unthemed selection
    selected = select_cards(pool, items_count, resonance_profile, params, rng)
    if selected:
        return (selected, None)

    return None


def _prepare_shop_items(
    selected: list[tuple[PoolEntry, float]],
    rng: random.Random,
    shop_config: dict[str, int],
) -> list[ShopItem]:
    """Prepare shop items with prices and apply one random discount."""
    items: list[ShopItem] = []
    for entry, _weight in selected:
        base_price = _compute_price(entry.card.rarity, shop_config)
        items.append(ShopItem(
            entry=entry,
            base_price=base_price,
            discounted_price=None,
        ))

    if items:
        discount_idx = rng.randrange(len(items))
        item = items[discount_idx]
        item.discounted_price = _apply_discount(
            item.base_price,
            rng,
            shop_config["discount_min"],
            shop_config["discount_max"],
        )

    return items


def _effective_price(item: ShopItem) -> int:
    """Return the effective price for a shop item (discounted if applicable)."""
    if item.discounted_price is not None:
        return item.discounted_price
    return item.base_price


def _total_cost(items: list[ShopItem], indices: list[int]) -> int:
    """Compute total cost for the selected item indices."""
    return sum(_effective_price(items[i]) for i in indices)


def run_discovery_draft(
    state: QuestState,
    params: AlgorithmParams,
    logger: SessionLogger,
    dreamscape_name: str,
    dreamscape_number: int,
    is_enhanced: bool,
    cards_per_pick: int,
    picks_per_site: int,
    tag_config: dict[str, float],
) -> None:
    """Run a Discovery Draft site interaction.

    Selects a theme tag, filters pool to matching cards, offers 4 per pick.
    Normal: pick 1. Enhanced (Arcane): pick any number.
    """
    for pick_num in range(1, picks_per_site + 1):
        result = _select_discovery_cards(
            pool=state.pool,
            resonance_profile=state.resonance_profile,
            tag_profile=state.tag_profile,
            params=params,
            rng=state.rng,
            cards_per_pick=cards_per_pick,
            tag_config=tag_config,
        )

        if result is None:
            print(f"  {DIM}No cards available in pool.{RESET}")
            break

        offered, theme_tag = result

        # Build header
        theme_label = f" [{theme_tag}]" if theme_tag else ""
        label = f"Discovery Draft{theme_label}"
        pick_info = f"Pick {pick_num}/{picks_per_site}"
        header = site_visit_header(
            dreamscape_name, label, pick_info, dreamscape_number
        )
        print(header)
        print()

        # Display offered cards
        card_names = [entry.card.name for entry, _ in offered]

        if is_enhanced:
            # Enhanced: multi-select any number
            def _render_multi(
                idx: int, option: str, highlighted: bool, checked: bool
            ) -> str:
                entry, _ = offered[idx]
                lines = format_card(entry.card, highlighted=highlighted)
                check = "[x]" if checked else "[ ]"
                marker = ">" if highlighted else " "
                lines[0] = lines[0].replace(
                    f"  {'>' if highlighted else ' '} ",
                    f"  {marker} {check} ",
                    1,
                )
                return "\n".join(lines)

            selected_indices = multi_select(card_names, render_fn=_render_multi)
            if not selected_indices:
                selected_indices = [0]  # Must pick at least 1
        else:
            # Normal: single-select pick 1
            def _render_single(
                idx: int, option: str, highlighted: bool
            ) -> str:
                entry, _ = offered[idx]
                lines = format_card(entry.card, highlighted=highlighted)
                return "\n".join(lines)

            selected_idx = single_select(card_names, render_fn=_render_single)
            selected_indices = [selected_idx]

        picked, unpicked = _apply_discovery_pick(offered, selected_indices)

        # Add picked cards to deck and remove from pool
        for entry in picked:
            state.add_card(entry.card)
            remove_entry(state.pool, entry)

        # Increment staleness on unpicked
        increment_staleness(unpicked)

        # Log each picked card
        offered_cards = [e.card for e, _ in offered]
        weights = [w for _, w in offered]
        for entry in picked:
            logger.log_draft_pick(
                offered_cards=offered_cards,
                weights=weights,
                picked_card=entry.card,
                profile_snapshot=state.resonance_profile.snapshot(),
            )

    # Log the overall site visit
    logger.log_site_visit(
        site_type="DiscoveryDraft",
        dreamscape=dreamscape_name,
        is_enhanced=is_enhanced,
        choices=[],
        choice_made=None,
        state_changes={
            "picks_completed": picks_per_site,
            "deck_size_after": state.deck_count(),
        },
        profile_snapshot=state.resonance_profile.snapshot(),
    )

    # Show resonance profile footer
    print(
        resonance_profile_footer(
            state.resonance_profile.snapshot(),
            state.deck_count(),
            state.essence,
        )
    )


def run_specialty_shop(
    state: QuestState,
    params: AlgorithmParams,
    logger: SessionLogger,
    dreamscape_name: str,
    dreamscape_number: int,
    is_enhanced: bool,
    shop_config: dict[str, int],
    tag_config: dict[str, float],
) -> None:
    """Run a Specialty Shop site interaction.

    Selects a theme tag, filters pool to matching cards, displays items
    with rarity-based prices and one random discount. Player buys via
    multi-select. Reroll costs 50 essence.
    """
    while True:
        items_count = shop_config["items_count"]
        result = _select_specialty_items(
            pool=state.pool,
            resonance_profile=state.resonance_profile,
            tag_profile=state.tag_profile,
            params=params,
            rng=state.rng,
            items_count=items_count,
            tag_config=tag_config,
        )

        if result is None:
            print(f"  {DIM}No items available.{RESET}")
            break

        selected, theme_tag = result
        items = _prepare_shop_items(selected, state.rng, shop_config)

        # Build header
        theme_label = f" [{theme_tag}]" if theme_tag else ""
        label = f"Specialty Shop{theme_label}"
        header = site_visit_header(
            dreamscape_name, label, "Browse", dreamscape_number
        )
        print(header)
        print()

        # Build option names including prices
        option_names: list[str] = []
        for item in items:
            if item.discounted_price is not None:
                price_str = f"{item.discounted_price}e (was {item.base_price}e)"
            else:
                price_str = f"{item.base_price}e"
            option_names.append(f"{item.entry.card.name} -- {price_str}")

        # Add reroll option
        reroll_cost = shop_config["reroll_cost"]
        can_afford_reroll = state.essence >= reroll_cost
        reroll_label = f"Reroll ({reroll_cost}e)"
        if not can_afford_reroll:
            reroll_label = f"{DIM}Reroll ({reroll_cost}e) -- not enough essence{RESET}"
        option_names.append(reroll_label)

        # Multi-select for purchasing
        def _render_shop(
            idx: int, option: str, highlighted: bool, checked: bool
        ) -> str:
            if idx < len(items):
                shop_item = items[idx]
                lines = format_card(
                    shop_item.entry.card, highlighted=highlighted
                )
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
            else:
                marker = ">" if highlighted else " "
                return f"\n  {marker} {option}"

        toggled = multi_select(option_names, render_fn=_render_shop)

        # Check if reroll was selected
        reroll_idx = len(items)
        if reroll_idx in toggled:
            if can_afford_reroll:
                state.spend_essence(reroll_cost)
                continue
            else:
                toggled = [i for i in toggled if i != reroll_idx]

        # Filter to item indices only
        purchase_indices = [i for i in toggled if i < len(items)]

        # Compute total cost
        total = _total_cost(items, purchase_indices)

        # Check affordability
        if total > state.essence:
            print(f"  Cannot afford {total}e (have {state.essence}e).")
            continue

        # Apply purchases
        purchased_cards: list[Card] = []
        for idx in purchase_indices:
            shop_item = items[idx]
            state.add_card(shop_item.entry.card)
            remove_entry(state.pool, shop_item.entry)
            purchased_cards.append(shop_item.entry.card)
            state.spend_essence(_effective_price(shop_item))

        # Log the shop interaction
        items_shown = [items[i].entry.card for i in range(len(items))]
        logger.log_shop_purchase(
            items_shown=items_shown,
            items_bought=purchased_cards,
            essence_spent=total,
        )
        logger.log_site_visit(
            site_type="SpecialtyShop",
            dreamscape=dreamscape_name,
            is_enhanced=is_enhanced,
            choices=[item.entry.card.name for item in items],
            choice_made=", ".join(c.name for c in purchased_cards)
            if purchased_cards
            else None,
            state_changes={
                "items_bought": [c.name for c in purchased_cards],
                "essence_spent": total,
                "deck_size_after": state.deck_count(),
            },
            profile_snapshot=state.resonance_profile.snapshot(),
        )
        break

    # Show resonance profile footer
    print(
        resonance_profile_footer(
            state.resonance_profile.snapshot(),
            state.deck_count(),
            state.essence,
        )
    )
