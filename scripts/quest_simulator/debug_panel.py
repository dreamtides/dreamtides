"""Debug panel for AI draft bot state during quest mode.

Renders a multi-line display showing draft status, seating layout,
and per-bot archetype preferences with commitment detection.
"""

import colors
import commitment
import render
from quest_state import QuestState

_BOT_COLORS = ["entity", "keyword", "constant", "tag", "string"]


def render_debug_panel(state: QuestState) -> str:
    """Build a multi-line debug panel string."""
    lines: list[str] = []

    sep = render.draw_double_separator()
    lines.append(sep)
    lines.append(f"  {colors.header('DEBUG PANEL')}")
    lines.append(sep)

    # Draft status
    round_num = state.round_index + 1
    pick_in_round = state.round_pick_count
    global_pick = state.global_pick_index

    status = (
        f"  Round {colors.num(round_num)}  "
        f"Pick {colors.num(pick_in_round)}  "
        f"Global pick {colors.num(global_pick)}"
    )

    if state.packs is not None and len(state.packs) > 0:
        seat0_pack_size = len(state.packs[0].cards)
        status += f"  Pack[0]: {colors.num(seat0_pack_size)} cards"

    pool_size = state.cube.total_size
    status += f"  Pool: {colors.num(pool_size)}"

    lines.append(status)
    lines.append("")

    # Seating layout
    # pass_left=True: seat s passes to (s+1)%6
    # Seat 5 passes to human (seat 0) = upstream/left
    # Seat 1 receives from human (seat 0) = downstream/right
    seat_order = [3, 4, 5, 0, 1, 2]
    seat_labels = {
        3: "3 upstream",
        4: "2 upstream",
        5: "LEFT (passes to you)",
        1: "RIGHT (receives from you)",
        2: "2 downstream",
    }

    for seat in seat_order:
        if seat == 0:
            lines.append(f"  {render.draw_separator()}")
            lines.append(f"  {colors.header('YOU (Seat 0)')}")
            lines.append(f"  {render.draw_separator()}")
            continue

        bot_idx = seat - 1
        color_role = _BOT_COLORS[bot_idx % len(_BOT_COLORS)]
        bot_name = colors.c(f"AI Agent {seat}", color_role, bold=True)
        position = seat_labels[seat]

        agent = state.ai_agents[bot_idx]
        w = agent.w

        # Top archetype preferences
        total = sum(w)
        if total > 0:
            indexed = sorted(enumerate(w), key=lambda x: -x[1])
            top = indexed[:3]
            pref_parts = []
            for arch_idx, val in top:
                pct = val / total * 100
                if pct < 5:
                    continue
                name = render.ARCHETYPE_NAMES[arch_idx]
                pref_parts.append(f"{name} {pct:.0f}%")
            pref_str = ", ".join(pref_parts) if pref_parts else "uniform"
        else:
            pref_str = "uniform"

        # Commitment detection
        conc = commitment.concentration(w)
        if conc >= 0.25:
            top_arch = max(range(len(w)), key=lambda i: w[i])
            arch_name = render.ARCHETYPE_NAMES[top_arch]
            commit_str = colors.c(f"Committed to {arch_name}", "accent", bold=True)
        else:
            commit_str = colors.dim("Exploring")

        # Resonance commitment
        if agent.committed_resonance is not None:
            p, s = agent.committed_resonance
            res_str = colors.c(f"{p}/{s}", "tag")
        else:
            res_str = colors.dim("undecided")

        # Cards drafted count
        drafted = len(agent.drafted)

        lines.append(f"  {bot_name}  <- {position}")
        lines.append(
            f"    {pref_str}  |  {commit_str}  |  "
            f"Res: {res_str}  |  {colors.num(drafted)} drafted"
        )

    lines.append("")
    lines.append(sep)
    return "\n".join(lines)
