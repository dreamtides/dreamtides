"""Main quest loop and atlas navigation for the quest simulator.

Orchestrates the entire quest from atlas initialization through victory:
atlas display, dreamscape selection, site visit loop, battle completion,
deck limit enforcement, and victory detection.
"""

import traceback
from typing import Optional

import atlas
import input_handler
import render
import render_atlas
import render_status
from jsonl_log import SessionLogger
from models import (
    DreamscapeNode,
    NodeState,
    Site,
    SiteType,
)
from draft_strategy import SixSeatDraftStrategy
from quest_state import QuestState
from site_dispatch import SiteData, VisitContext

_VIEW_DECK_LABEL = "View Deck"


def _get_selectable_sites(sites: list[Site]) -> list[Site]:
    """Return the list of sites the player can currently visit.

    Battle is locked until all other sites have been visited.
    Already-visited sites are excluded.
    """
    all_non_battle_visited = all(
        s.is_visited for s in sites if s.site_type != SiteType.BATTLE
    )
    selectable: list[Site] = []
    for site in sites:
        if site.is_visited:
            continue
        if site.site_type == SiteType.BATTLE and not all_non_battle_visited:
            continue
        selectable.append(site)
    return selectable


def _enforce_deck_limits(
    state: QuestState,
    logger: Optional[SessionLogger],
) -> None:
    """Enforce deck size limits before the battle site becomes available.

    If over max_deck, triggers the forced purge interaction.
    If under min_deck, duplicates the whole deck repeatedly until it
    exceeds the minimum.
    """
    import sites_purge

    if state.is_over_deck_limit():
        sites_purge.forced_deck_limit_purge(state, logger)

    if state.is_under_deck_limit() and state.deck_count() > 0:
        before = state.deck_count()
        state.auto_fill_deck()
        after = state.deck_count()
        copies = after // before
        print()
        print(
            f"  {render.BOLD}Auto-fill:{render.RESET} "
            f"Deck had {before} cards (minimum {state.min_deck}). "
            f"Duplicated to {copies} copies ({after} cards)."
        )
        print()


def _handle_post_battle(
    state: QuestState,
    nodes: list[DreamscapeNode],
    node_id: int,
    total_battles: int,
    logger: Optional[SessionLogger],
) -> bool:
    """Handle post-battle bookkeeping.

    Marks the dreamscape as completed, generates new atlas nodes,
    increments completion level, and checks for victory.

    Returns True if the quest is won.
    """
    state.increment_completion()
    atlas.complete_node(nodes, node_id, state.rng)

    return state.completion_level >= total_battles


def _show_victory(
    state: QuestState,
    total_battles: int,
    dreamscapes_visited: int,
    logger: Optional[SessionLogger],
    log_path: Optional[str] = None,
) -> None:
    """Display the victory screen and log session end."""
    dreamcaller_name = (
        state.dreamcaller.name if state.dreamcaller is not None else "None"
    )

    screen = render_status.victory_screen(
        battles_won=state.completion_level,
        total_battles=total_battles,
        dreamscapes_visited=dreamscapes_visited,
        dreamcaller_name=dreamcaller_name,
        deck_size=state.deck_count(),
        dreamsign_count=state.dreamsign_count(),
        essence=state.essence,
        w=state.draft_strategy.preference_vector,
        log_path=log_path,
        archetype_draft=state.archetype_draft,
    )
    print(screen)

    if logger is not None:
        # Log post-hoc commitment detection for 6-seat drafts
        if isinstance(state.draft_strategy, SixSeatDraftStrategy):
            commit_results = state.draft_strategy.detect_all_commitments()
            seat_commitments: list[dict[str, object]] = []
            for seat_idx, result in enumerate(commit_results):
                seat_commitments.append(
                    {
                        "seat_index": seat_idx,
                        "commitment_pick": result.commitment_pick,
                        "committed_archetype": result.committed_archetype,
                        "entropy_commitment_pick": result.entropy_commitment_pick,
                        "entropy_committed_archetype": result.entropy_committed_archetype,
                    }
                )
            logger.log_draft_commitment(seat_commitments)

        logger.log_session_end(
            deck=state.deck,
            essence=state.essence,
            completion_level=state.completion_level,
            dreamsigns=state.dreamsigns,
            dreamcaller=state.dreamcaller,
            preference_vector=state.draft_strategy.preference_vector,
        )


def _show_deck_view(state: QuestState) -> None:
    """Display the full deck viewer and wait for dismissal."""
    import render_cards

    output = render_cards.render_full_deck_view(
        deck_cards=state.deck,
        dreamsigns=state.dreamsigns,
        dreamcaller=state.dreamcaller,
        essence=state.essence,
    )
    print(output)
    input_handler.wait_for_continue()


def _dreamscape_loop(
    node: DreamscapeNode,
    state: QuestState,
    data: SiteData,
    logger: Optional[SessionLogger],
    dreamscape_number: int,
) -> None:
    """Run the site visit loop within a single dreamscape.

    Displays the site list, lets the player select sites (with battle
    locked until all others are visited), dispatches to handlers, and
    enforces deck limits before battle. A "View Deck" option is
    available at the top of the menu to inspect the current deck
    without consuming a site visit.
    """
    from site_dispatch import visit_site

    context = VisitContext(
        dreamscape_name=node.name,
        dreamscape_number=dreamscape_number,
    )

    while True:
        selectable = _get_selectable_sites(node.sites)
        if not selectable:
            break

        # Check if we are about to unlock battle (only battle left)
        if len(selectable) == 1 and selectable[0].site_type == SiteType.BATTLE:
            _enforce_deck_limits(state, logger)

        # Show visited/locked sites as non-interactive header
        all_non_battle_visited = all(
            s.is_visited for s in node.sites if s.site_type != SiteType.BATTLE
        )
        header_lines: list[str] = []
        for site in node.sites:
            name = render_atlas.site_type_name(site.site_type)
            if site.is_enhanced:
                name += "*"
            if site.is_visited:
                header_lines.append(f"  {render.DIM}\u2713 {name}{render.RESET}")
            elif site.site_type == SiteType.BATTLE and not all_non_battle_visited:
                header_lines.append(f"      {name} [locked]")
        if header_lines:
            print("\n".join(header_lines))

        # Build selection options: View Deck first, then selectable sites
        option_labels = [_VIEW_DECK_LABEL] + [
            render_atlas.site_type_name(s.site_type) + ("*" if s.is_enhanced else "")
            for s in selectable
        ]

        def _render_site_option(
            index: int,
            option: str,
            is_selected: bool,
            _selectable: list[Site] = selectable,
        ) -> str:
            marker = ">" if is_selected else " "
            if index == 0:
                return f"  {marker}   {render.DIM}{_VIEW_DECK_LABEL}{render.RESET}"
            site = _selectable[index - 1]
            name = render_atlas.site_type_name(site.site_type)
            if site.is_enhanced:
                name += "*"
            return f"  {marker}   {name}"

        chosen_idx = input_handler.single_select(
            options=option_labels,
            render_fn=_render_site_option,
            initial=1,
        )

        if chosen_idx == 0:
            _show_deck_view(state)
            continue

        chosen_site = selectable[chosen_idx - 1]
        try:
            visit_site(chosen_site, state, data, logger, context)
        except KeyboardInterrupt:
            raise
        except Exception:
            error_msg = traceback.format_exc()
            site_name = chosen_site.site_type.value
            print()
            print(
                f"  {render.BOLD}Error:{render.RESET} "
                f"Site '{site_name}' encountered an error and was skipped."
            )
            print()
            print(error_msg)
            print()
            if logger is not None:
                logger.log_error(
                    site_type=site_name,
                    error_message=error_msg,
                )
            chosen_site.is_visited = True


def _atlas_loop(
    nodes: list[DreamscapeNode],
    state: QuestState,
    data: SiteData,
    total_battles: int,
    logger: Optional[SessionLogger],
) -> None:
    """Run the main atlas navigation loop.

    Displays the atlas, lets the player select a dreamscape, generates
    sites if needed, runs the dreamscape loop, handles post-battle,
    and checks for victory.
    """
    dreamscapes_visited = 0
    is_first_dreamscape = state.completion_level == 0

    while True:
        available = atlas.get_available_nodes(nodes)
        if not available:
            break

        # Display atlas
        atlas_display = render_atlas.render_full_atlas(
            available_nodes=available,
            all_nodes=nodes,
            selected_index=0,
            essence=state.essence,
            completion=state.completion_level,
            total_battles=total_battles,
            deck_count=state.deck_count(),
            dreamsign_count=state.dreamsign_count(),
        )
        print(atlas_display)

        # Build option labels
        option_labels = [n.name for n in available]

        def _render_atlas_option(
            index: int,
            option: str,
            is_selected: bool,
            _available: list[DreamscapeNode] = available,
            _nodes: list[DreamscapeNode] = nodes,
        ) -> str:
            return render_atlas.render_available_dreamscapes(
                _available, index if is_selected else -1
            )

        chosen_idx = input_handler.single_select(
            options=option_labels,
        )

        chosen_node = available[chosen_idx]
        dreamscapes_visited += 1

        # Generate sites if not yet generated
        if not chosen_node.sites:
            atlas.generate_sites(
                chosen_node,
                completion_level=state.completion_level,
                rng=state.rng,
                is_first_dreamscape=is_first_dreamscape,
            )
            is_first_dreamscape = False

        # Log dreamscape entry
        if logger is not None:
            logger.log_dreamscape_enter(
                dreamscape_name=chosen_node.name,
                completion_level=state.completion_level,
                sites=chosen_node.sites,
            )

        # Run the dreamscape site loop
        _dreamscape_loop(
            node=chosen_node,
            state=state,
            data=data,
            logger=logger,
            dreamscape_number=dreamscapes_visited,
        )

        # Post-battle handling
        victory = _handle_post_battle(
            state=state,
            nodes=nodes,
            node_id=chosen_node.node_id,
            total_battles=total_battles,
            logger=logger,
        )

        if victory:
            _show_victory(
                state=state,
                total_battles=total_battles,
                dreamscapes_visited=dreamscapes_visited,
                logger=logger,
                log_path=str(logger.path) if logger is not None else None,
            )
            break


def run_quest(
    state: QuestState,
    nodes: list[DreamscapeNode],
    data: SiteData,
    total_battles: int,
    logger: Optional[SessionLogger] = None,
) -> None:
    """Run the entire quest from atlas initialization through victory.

    This is the main entry point for the quest flow. It runs the atlas
    loop until the player achieves victory or there are no more
    available dreamscapes.
    """
    _atlas_loop(
        nodes=nodes,
        state=state,
        data=data,
        total_battles=total_battles,
        logger=logger,
    )
