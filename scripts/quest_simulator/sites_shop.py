"""Shop site interaction for the quest simulator.

Implements the Shop site: 3 priced cards from the draft pack at seat 0,
2 random dreamsign offerings, and a reroll option. Cards are priced based
on power value. Each shop visit consumes at least 1 pick step from the
draft loop; rerolls cost 1 additional pick step each.
"""

from dataclasses import dataclass
from typing import Any, Optional

import colors
import input_handler
import log_helpers
import render
import render_cards
import render_status
import round_manager
import show_n
from draft_models import CardInstance
from jsonl_log import SessionLogger
from models import Dreamsign
from quest_state import QuestState

SHOP_SHOW_N = 3


@dataclass(frozen=True)
class ShopItem:
    """A card offered for sale in the shop."""

    card_instance: CardInstance
    price: int


def compute_price(power: float) -> int:
    """Compute the shop price for a card based on its power value.

    Price is ``round(power * 25)``, clamped to the range [5, 100].
    """
    return max(5, min(100, round(power * 25)))


def _select_dreamsigns(
    all_dreamsigns: list[Dreamsign],
    count: int,
    state: QuestState,
) -> list[Dreamsign]:
    """Select random dreamsign offerings from the available pool."""
    if not all_dreamsigns or count <= 0:
        return []
    n = min(count, len(all_dreamsigns))
    indices = list(range(len(all_dreamsigns)))
    state.rng.shuffle(indices)
    return [all_dreamsigns[i] for i in indices[:n]]


def _build_shop_items(shown_cards: list[CardInstance]) -> list[ShopItem]:
    """Build priced shop items from the filtered card list."""
    return [
        ShopItem(
            card_instance=card,
            price=compute_price(card.design.power),
        )
        for card in shown_cards
    ]


def _render_option(
    index: int,
    option: str,
    is_selected: bool,
    items: list[ShopItem],
    dreamsign_offerings: list[Dreamsign],
    available_essence: int,
    reroll_cost: int,
    free_rerolls: int,
) -> str:
    """Render a single shop menu option."""
    marker = ">" if is_selected else " "
    card_count = len(items)
    dreamsign_count = len(dreamsign_offerings)

    if index < card_count:
        item = items[index]
        card_lines = render_cards.format_card_display(
            item.card_instance, highlighted=is_selected
        )
        affordable = available_essence >= item.price
        price_color = "accent" if affordable else "comment"
        price_str = colors.c(f"{item.price}e", price_color, bold=affordable)
        price_line = f"         Price: {price_str}"
        return "\n".join(card_lines + [price_line])

    dreamsign_start = card_count
    if index < dreamsign_start + dreamsign_count:
        ds = dreamsign_offerings[index - dreamsign_start]
        bane_label = f" {colors.c('[BANE]', 'error', bold=True)}" if ds.is_bane else ""
        name_line = f"  {marker} {colors.c(ds.name, 'accent')}{bane_label}"
        detail_line = f"      {colors.dim(ds.effect_text)}"
        price_line = f"      Price: {colors.c('FREE', 'accent', bold=True)}"
        return "\n".join([name_line, detail_line, price_line])

    reroll_idx = dreamsign_start + dreamsign_count
    if index == reroll_idx:
        if free_rerolls > 0:
            label = f"Reroll ({colors.c('FREE', 'accent', bold=True)})"
        else:
            reroll_affordable = available_essence >= reroll_cost
            cost_color = "accent" if reroll_affordable else "comment"
            label = f"Reroll ({colors.c(f'{reroll_cost} essence', cost_color)})"
        return f"  {marker} {label}"

    # Leave option
    return f"  {marker} {colors.dim('Leave shop')}"


def run_shop(
    state: QuestState,
    shop_config: dict[str, Any],
    dreamscape_name: str,
    dreamscape_number: int,
    logger: Optional[SessionLogger],
    is_enhanced: bool = False,
    all_dreamsigns: Optional[list[Dreamsign]] = None,
) -> None:
    """Run a Shop site interaction.

    Displays 3 cards from the draft pack at seat 0 (filtered via
    show_n=3), priced by power value, plus 2 dreamsign offerings and
    a reroll option. Each visit consumes at least 1 draft pick step.
    Rerolls advance the draft by 1 additional pick step each.
    Enhanced shops (Verdant biome) get the first reroll free.
    """
    reroll_cost: int = shop_config.get("reroll_cost", 50)
    free_rerolls = 1 if is_enhanced else 0
    dreamsigns_pool = all_dreamsigns or []

    # Display header
    header = render.site_visit_header(
        dreamscape_name=dreamscape_name,
        site_type_label="Shop",
        pick_info="Buy items",
        dreamscape_number=dreamscape_number,
    )
    print(header)
    print()

    # Get the initial pack at seat 0 (AI picks at seats 1-5 run first).
    # This is called once before the menu loop. Only rerolls (which
    # consume a pick step) trigger a fresh advance_to_human_pick.
    pack = round_manager.advance_to_human_pick(state, logger=logger)

    while True:
        # Filter to SHOP_SHOW_N cards
        shown_cards = show_n.select_cards(
            pack.cards,
            SHOP_SHOW_N,
            "sharpened_preference",
            state.rng,
            human_w=state.human_agent.w,
            human_drafted=state.human_agent.drafted,
            scoring_cfg=state.draft_cfg.scoring,
        )

        # Log show-N filtering
        if logger is not None and shown_cards:
            scores = log_helpers.compute_show_n_scores(
                shown_cards, state.human_agent.w, "sharpened_preference"
            )
            shown_with_scores = []
            for card, score in zip(shown_cards, scores):
                entry = log_helpers.card_instance_dict(card)
                entry["score"] = score
                shown_with_scores.append(entry)

            filtered_out = [c for c in pack.cards if c not in shown_cards]
            filtered_scores = log_helpers.compute_show_n_scores(
                filtered_out, state.human_agent.w, "sharpened_preference"
            )
            filtered_top3 = []
            paired = list(zip(filtered_out, filtered_scores))
            paired.sort(key=lambda t: t[1], reverse=True)
            for card, score in paired[:3]:
                entry = log_helpers.card_instance_dict(card)
                entry["score"] = score
                filtered_top3.append(entry)

            logger.log_show_n_filter(
                strategy="sharpened_preference",
                pack_size=len(pack.cards),
                shown_count=len(shown_cards),
                shown_cards_with_scores=shown_with_scores,
                filtered_out_top3=filtered_top3,
                context="shop",
                global_pick_index=state.global_pick_index,
                round_index=state.round_index,
            )

        if not shown_cards:
            print(f"  {colors.dim('No cards available.')}")
            round_manager.advance_pick_no_card(state)
            break

        items = _build_shop_items(shown_cards)
        dreamsign_offerings = _select_dreamsigns(dreamsigns_pool, 2, state)

        # Display shop contents
        print(
            f"  {colors.section('Shop Items')}  |  Essence: {colors.num(state.essence)}"
        )
        print()

        for item in items:
            card_lines = render_cards.format_card_display(
                item.card_instance, show_images=True
            )
            for line in card_lines:
                print(line)
            price_str = colors.c(f"{item.price}e", "accent", bold=True)
            print(f"         Price: {price_str}")
            print()

        if dreamsign_offerings:
            print(f"  {colors.section('Dreamsign Offerings')}")
            for ds in dreamsign_offerings:
                bane_label = (
                    f" {colors.c('[BANE]', 'error', bold=True)}" if ds.is_bane else ""
                )
                print(f"    {colors.c(ds.name, 'accent')}{bane_label}")
                print(f"      {colors.dim(ds.effect_text)}")
            print()

        # Build single-select options:
        # [card0, card1, card2, dreamsign0, dreamsign1, reroll, leave]
        option_labels: list[str] = []
        for item in items:
            option_labels.append(
                f"Buy {item.card_instance.design.name} ({item.price}e)"
            )
        for ds in dreamsign_offerings:
            option_labels.append(f"Buy dreamsign: {ds.name}")
        if free_rerolls > 0:
            option_labels.append("Reroll (FREE)")
        else:
            option_labels.append(f"Reroll ({reroll_cost} essence)")
        option_labels.append("Leave shop")

        card_count = len(items)
        ds_count = len(dreamsign_offerings)
        reroll_index = card_count + ds_count
        leave_index = reroll_index + 1

        def _make_render_fn(
            _items: list[ShopItem],
            _ds: list[Dreamsign],
            _essence: int,
            _reroll_cost: int,
            _free_rerolls: int,
        ):
            def _fn(index: int, option: str, is_selected: bool) -> str:
                return _render_option(
                    index,
                    option,
                    is_selected,
                    _items,
                    _ds,
                    _essence,
                    _reroll_cost,
                    _free_rerolls,
                )

            return _fn

        render_fn = _make_render_fn(
            items,
            dreamsign_offerings,
            state.essence,
            reroll_cost,
            free_rerolls,
        )

        # Build structured card data for web UI (card options show TCG images;
        # dreamsign/reroll/leave remain plain-text).
        web_options_data: list[dict | None] = [
            input_handler.make_card_option_data(
                name=item.card_instance.design.name,
                energy_cost=item.card_instance.design.energy_cost,
                card_type=item.card_instance.design.card_type,
                rules_text=item.card_instance.design.rules_text,
                spark=item.card_instance.design.spark,
                price=item.price,
            )
            for item in items
        ] + [None] * (ds_count + 2)

        chosen_index = input_handler.single_select(
            options=option_labels,
            render_fn=render_fn,
            options_data=web_options_data,
        )

        # Handle the choice
        if chosen_index < card_count:
            # Buy a card
            item = items[chosen_index]
            if state.essence >= item.price:
                state.spend_essence(item.price)
                state.add_card(item.card_instance)
                round_manager.complete_human_pick(
                    state, item.card_instance, shown_cards
                )

                # Log preference snapshot after shop purchase
                if logger is not None:
                    logger.log_preference_snapshot(
                        global_pick_index=state.global_pick_index,
                        preference_vector=state.human_agent.w,
                        top_archetype_index=log_helpers.top_n_w(state.human_agent.w, 1)[
                            0
                        ][0],
                        concentration=log_helpers.w_concentration(state.human_agent.w),
                    )

                print()
                card_name = colors.card(item.card_instance.design.name)
                print(f"  Purchased {card_name} for {item.price} essence.")
                print(f"  Essence remaining: {colors.num(state.essence)}")

                if logger is not None:
                    logger.log_site_visit(
                        site_type="Shop",
                        dreamscape=dreamscape_name,
                        is_enhanced=is_enhanced,
                        choices=[c.design.name for c in shown_cards],
                        choice_made=item.card_instance.design.name,
                        state_changes={
                            "essence_spent": item.price,
                            "deck_size_after": state.deck_count(),
                        },
                    )

                # Advance draft and continue shopping with fresh cards.
                pack = round_manager.advance_to_human_pick(state, logger=logger)
                continue
            else:
                print()
                print(f"  {colors.dim('Not enough essence.')}")
                round_manager.advance_pick_no_card(state)

                if logger is not None:
                    logger.log_site_visit(
                        site_type="Shop",
                        dreamscape=dreamscape_name,
                        is_enhanced=is_enhanced,
                        choices=[c.design.name for c in shown_cards],
                        choice_made=None,
                        state_changes={
                            "essence_spent": 0,
                            "deck_size_after": state.deck_count(),
                        },
                    )
                break

        elif chosen_index < card_count + ds_count:
            # Buy a dreamsign (does NOT consume a draft pick).
            # The player is returned to the shop menu to pick a card
            # or leave. No new advance_to_human_pick is needed because
            # the same pack is still pending resolution.
            ds = dreamsign_offerings[chosen_index - card_count]
            state.add_dreamsign(ds)

            print()
            print(f"  Acquired dreamsign: {colors.c(ds.name, 'accent')}")
            continue

        elif chosen_index == reroll_index:
            # Reroll: advance 1 pick step (no card taken)
            reroll_essence_cost = 0
            if free_rerolls > 0:
                free_rerolls -= 1
            elif state.essence >= reroll_cost:
                reroll_essence_cost = reroll_cost
                state.spend_essence(reroll_cost)
            else:
                print()
                print(f"  {colors.dim(f'Not enough essence to reroll '
                    f'({reroll_cost} needed, {state.essence} available).')}")
                continue

            print()
            print(f"  {colors.c('Rerolling shop...', 'accent', bold=True)}")
            print()

            # Consume 1 pick step for the reroll, then get the new pack
            round_manager.advance_pick_no_card(state)

            if logger is not None:
                logger.log_site_visit(
                    site_type="Shop",
                    dreamscape=dreamscape_name,
                    is_enhanced=is_enhanced,
                    choices=[c.design.name for c in shown_cards],
                    choice_made="reroll",
                    state_changes={
                        "essence_spent": reroll_essence_cost,
                        "deck_size_after": state.deck_count(),
                    },
                )

            pack = round_manager.advance_to_human_pick(state, logger=logger)
            continue

        else:
            # Leave shop (buy nothing) -- consumes 1 pick step
            round_manager.advance_pick_no_card(state)
            print()
            print(f"  {colors.dim('No items purchased.')}")

            if logger is not None:
                logger.log_site_visit(
                    site_type="Shop",
                    dreamscape=dreamscape_name,
                    is_enhanced=is_enhanced,
                    choices=[c.design.name for c in shown_cards],
                    choice_made=None,
                    state_changes={
                        "essence_spent": 0,
                        "deck_size_after": state.deck_count(),
                    },
                )
            break

    # Show the archetype preference footer
    footer = render_status.archetype_preference_footer(
        w=state.human_agent.w,
        deck_count=state.deck_count(),
        essence=state.essence,
    )
    print(footer)
