"""Metrics computation, table formatting, and all output modes."""

import statistics
from collections import Counter

from models import QuestResult, Resonance, ResonanceProfile


def format_table(headers: list[str], rows: list[list[str]], align: str = "") -> str:
    """Format a clean ASCII table with aligned columns.

    align: string of 'l' or 'r' per column (default left).
    """
    if not rows:
        return "(no data)"

    col_widths = [len(h) for h in headers]
    for row in rows:
        for i, cell in enumerate(row):
            if i < len(col_widths):
                col_widths[i] = max(col_widths[i], len(cell))

    if not align:
        align = "l" * len(headers)

    def fmt_row(cells: list[str]) -> str:
        parts = []
        for i, cell in enumerate(cells):
            w = col_widths[i] if i < len(col_widths) else len(cell)
            if i < len(align) and align[i] == "r":
                parts.append(cell.rjust(w))
            else:
                parts.append(cell.ljust(w))
        return "  ".join(parts)

    lines = [fmt_row(headers), fmt_row(["-" * col_widths[i] for i in range(len(headers))])]
    for row in rows:
        lines.append(fmt_row(row))
    return "\n".join(lines)


def classify_deck(profile: ResonanceProfile) -> str:
    """Classify a deck as mono, dual, or tri."""
    total = profile.total()
    if total == 0:
        return "empty"

    shares = sorted(
        [(c / total, r.value) for r, c in profile.counts.items()],
        reverse=True,
    )

    if shares[0][0] > 0.85:
        return "mono"
    if shares[0][0] + shares[1][0] > 0.75 and shares[0][0] <= 0.85:
        return "dual"
    if len([s for s, _ in shares if s > 0.15]) >= 3:
        return "tri"
    return "dual"  # Default to dual for edge cases


def has_splash(profile: ResonanceProfile) -> bool:
    """Check if deck has a deliberate splash (3rd resonance with 1-3 cards)."""
    sorted_counts = sorted(profile.counts.values(), reverse=True)
    if len(sorted_counts) >= 3:
        third = sorted_counts[2]
        return 1 <= third <= 3
    return False


def convergence_pick(result: QuestResult) -> int:
    """First pick where top-2 share of drafted cards (excluding dreamcaller bonus) exceeds 75%."""
    deck_profile = ResonanceProfile()
    for pick in result.picks:
        if pick.bought:
            for card in pick.bought:
                for r in card.resonances:
                    deck_profile.add(r)
        else:
            for r in pick.picked.resonances:
                deck_profile.add(r)
        # Need at least 5 resonance-bearing cards before checking
        if deck_profile.total() >= 5 and deck_profile.top2_share() > 0.75:
            return pick.pick_number
    return result.picks[-1].pick_number if result.picks else 0


def off_color_offered_pct(result: QuestResult) -> float:
    """Fraction of offered cards that were off-color (not in top-2, not neutral)."""
    if not result.picks:
        return 0.0

    profile = ResonanceProfile()
    # Add dreamcaller bonus from first pick's profile
    if result.picks:
        first_profile = result.picks[0].profile_after
        for r in result.picks[0].picked.resonances:
            first_profile[r] = first_profile.get(r, 0) - 1

    total_offered = 0
    off_color = 0

    running = ResonanceProfile()
    for r, c in (result.picks[0].profile_after if result.picks else {}).items():
        # Reconstruct dreamcaller contribution
        pass

    # Simpler approach: use final profile's top-2
    top2 = {r for r, _ in result.final_profile.top_n(2)}

    for pick in result.picks:
        for card in pick.offered:
            total_offered += 1
            if card.resonances and not (card.resonances & top2):
                off_color += 1

    return off_color / total_offered if total_offered > 0 else 0.0


def print_trace(result: QuestResult):
    """Print pick-by-pick detail for a single quest."""
    dc_str = "+".join(r.value for r in sorted(result.dreamcaller_resonances, key=lambda r: r.value))
    print(f"\n=== Quest Trace (Dreamcaller: {dc_str}) ===\n")

    for pick in result.picks:
        is_shop = pick.bought is not None
        label = "Shop" if is_shop else "Pick"
        print(f"--- {label} {pick.pick_number} ---")

        bought_ids = {c.id for c in pick.bought} if pick.bought else set()

        # Show offered cards with weights
        for i, (card, weight) in enumerate(zip(pick.offered, pick.weights)):
            res = "+".join(r.value for r in sorted(card.resonances, key=lambda r: r.value))
            if not res:
                res = "neutral"
            if is_shop:
                marker = " <-- bought" if card.id in bought_ids else ""
            else:
                marker = " <--" if card.id == pick.picked.id else ""
            rarity_tag = card.rarity.value[0]
            print(f"  [{rarity_tag}] power={card.power:2d}  {res:14s}  w={weight:6.2f}{marker}")

        if is_shop:
            print(f"  Bought {len(pick.bought)} cards")
            for reason in (pick.buy_reasons or []):
                print(f"    {reason}")
        else:
            print(f"  Picked: {pick.pick_reason}")

        # Profile snapshot
        profile_str = ", ".join(
            f"{r.value}:{c}" for r, c in sorted(pick.profile_after.items(), key=lambda x: x[0].value) if c > 0
        )
        print(f"  Profile: {profile_str}")
        print()

    # Final summary
    print("=== Final Deck ===")
    print(f"  Cards: {len(result.deck)}")
    total = result.final_profile.total()
    for r, c in result.final_profile.top_n(5):
        if c > 0:
            pct = c / total * 100 if total > 0 else 0
            print(f"  {r.value:8s}: {c:2d} ({pct:4.1f}%)")
    print(f"  Top-2 share: {result.final_profile.top2_share():.1%}")
    print(f"  HHI: {result.final_profile.hhi():.3f}")
    print(f"  Classification: {classify_deck(result.final_profile)}")


def print_aggregate(results: list[QuestResult]):
    """Print convergence stats, archetype distribution, and splash analysis."""
    n = len(results)
    print(f"\n=== Aggregate Results ({n} quests) ===\n")

    # Convergence
    conv_picks = [convergence_pick(r) for r in results]
    print("Convergence (top-2 share > 75%):")
    print(f"  Mean pick: {statistics.mean(conv_picks):.1f}")
    print(f"  Median:    {statistics.median(conv_picks):.1f}")
    print(f"  Std dev:   {statistics.stdev(conv_picks):.1f}" if n > 1 else "")
    print()

    # Final deck composition
    top2_shares = [r.final_profile.top2_share() for r in results]
    hhis = [r.final_profile.hhi() for r in results]
    print("Final Deck Composition:")
    print(f"  Mean top-2 share: {statistics.mean(top2_shares):.1%}")
    print(f"  Mean HHI:         {statistics.mean(hhis):.3f}")
    print()

    # Archetype distribution
    classifications = Counter(classify_deck(r.final_profile) for r in results)
    print("Archetype Distribution:")
    headers = ["Type", "Count", "Pct"]
    rows = []
    for cls in ["mono", "dual", "tri"]:
        count = classifications.get(cls, 0)
        rows.append([cls, str(count), f"{count / n:.1%}"])
    print(format_table(headers, rows, "lrr"))
    print()

    # Splash analysis
    splash_count = sum(1 for r in results if has_splash(r.final_profile))
    print(f"Splash (3rd color 1-3 cards): {splash_count}/{n} ({splash_count / n:.1%})")

    # Off-color offered
    off_color_pcts = [off_color_offered_pct(r) for r in results]
    print(f"Off-color offered:            {statistics.mean(off_color_pcts):.1%} (mean)")
    print()

    # Resonance pair distribution
    pair_counts: Counter = Counter()
    for r in results:
        top2 = tuple(sorted([res.value for res, _ in r.final_profile.top_n(2)]))
        pair_counts[top2] += 1

    print("Top Resonance Pairs:")
    headers = ["Pair", "Count", "Pct"]
    rows = []
    for pair, count in pair_counts.most_common(10):
        rows.append([f"{pair[0]}+{pair[1]}", str(count), f"{count / n:.1%}"])
    print(format_table(headers, rows, "lrr"))


def print_sweep(
    sweep_results: list[tuple[str, str, list[QuestResult]]],
    param_name: str,
):
    """Print comparison table across parameter values.

    sweep_results: list of (param_value_str, strategy_name, results)
    """
    print(f"\n=== Parameter Sweep: {param_name} ===\n")

    headers = [
        param_name,
        "Strategy",
        "Conv.Pick",
        "Top2%",
        "HHI",
        "Mono%",
        "Dual%",
        "Tri%",
        "Splash%",
        "OffColor%",
    ]
    rows = []

    for val_str, strat_name, results in sweep_results:
        n = len(results)
        conv = statistics.mean([convergence_pick(r) for r in results])
        top2 = statistics.mean([r.final_profile.top2_share() for r in results])
        hhi = statistics.mean([r.final_profile.hhi() for r in results])
        classes = Counter(classify_deck(r.final_profile) for r in results)
        splash = sum(1 for r in results if has_splash(r.final_profile)) / n
        off_color = statistics.mean([off_color_offered_pct(r) for r in results])

        rows.append([
            val_str,
            strat_name,
            f"{conv:.1f}",
            f"{top2:.1%}",
            f"{hhi:.3f}",
            f"{classes.get('mono', 0) / n:.0%}",
            f"{classes.get('dual', 0) / n:.0%}",
            f"{classes.get('tri', 0) / n:.0%}",
            f"{splash:.0%}",
            f"{off_color:.1%}",
        ])

    print(format_table(headers, rows, "llrrrrrrrr"))


def print_evolution(results: list[QuestResult]):
    """Show how metrics evolve pick-by-pick, averaged over runs."""
    print(f"\n=== Metric Evolution ({len(results)} quests) ===\n")

    # Find max pick number
    max_pick = max(
        (p.pick_number for r in results for p in r.picks),
        default=0,
    )

    checkpoints = [5, 10, 15, 20, 25, 30]
    checkpoints = [c for c in checkpoints if c <= max_pick]

    headers = ["Pick", "Top2%", "Eff.Colors", "HHI", "OffColor%"]
    rows = []

    for target_pick in checkpoints:
        top2_vals = []
        eff_colors = []
        hhi_vals = []
        off_color_vals = []

        for result in results:
            # Find the profile at this pick number
            profile = ResonanceProfile()
            # Reconstruct dreamcaller bonus
            if result.picks:
                first = result.picks[0]
                for r, c in first.profile_after.items():
                    found_in_pick = 1 if r in first.picked.resonances else 0
                    dc_bonus = c - found_in_pick
                    if dc_bonus > 0:
                        profile.add(r, dc_bonus)

            off_total = 0
            off_count = 0

            # Reset profile and replay
            profile = ResonanceProfile()
            # Re-add dreamcaller bonus
            if result.picks:
                first = result.picks[0]
                # Count resonances added by the first pick's cards
                first_res_counts: dict = {}
                if first.bought:
                    for card in first.bought:
                        for r in card.resonances:
                            first_res_counts[r] = first_res_counts.get(r, 0) + 1
                else:
                    for r in first.picked.resonances:
                        first_res_counts[r] = first_res_counts.get(r, 0) + 1
                for r, c in first.profile_after.items():
                    dc_bonus = c - first_res_counts.get(r, 0)
                    if dc_bonus > 0:
                        profile.add(r, dc_bonus)

            for pick in result.picks:
                if pick.pick_number > target_pick:
                    break
                if pick.bought:
                    for card in pick.bought:
                        for r in card.resonances:
                            profile.add(r)
                else:
                    for r in pick.picked.resonances:
                        profile.add(r)

                # Track off-color at this point
                top2_res = {r for r, _ in profile.top_n(2)}
                for card in pick.offered:
                    off_total += 1
                    if card.resonances and not (card.resonances & top2_res):
                        off_count += 1

            if profile.total() > 0:
                top2_vals.append(profile.top2_share())
                eff_colors.append(profile.effective_colors())
                hhi_vals.append(profile.hhi())
            if off_total > 0:
                off_color_vals.append(off_count / off_total)

        if top2_vals:
            rows.append([
                str(target_pick),
                f"{statistics.mean(top2_vals):.1%}",
                f"{statistics.mean(eff_colors):.2f}",
                f"{statistics.mean(hhi_vals):.3f}",
                f"{statistics.mean(off_color_vals):.1%}" if off_color_vals else "N/A",
            ])

    print(format_table(headers, rows, "rrrrr"))
