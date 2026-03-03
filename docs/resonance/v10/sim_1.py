#!/usr/bin/env python3
"""
Monte Carlo Simulation: D1 Open Table Algorithm
Resonance Draft System V10 — Simulation Agent 1

5 AI drafters (from 8 archetypes), 4 cards each per round,
10 lowest-power market culling per round, Level 0 reactivity.

Level 0 means AIs have a fixed preference function (not reactive to player),
but they pick from the shared pool each round — cards taken by one AI are
unavailable to others. The *logic* is predetermined; the *outcomes* depend on
the shared pool state.
"""

import random
import math
from collections import defaultdict, Counter
from dataclasses import dataclass, field
from typing import Optional

# ─── Constants ───────────────────────────────────────────────────────────────

NUM_DRAFTS = 1000
NUM_PICKS = 30
PACK_SIZE = 4
NUM_AIS = 5
AI_CARDS_PER_ROUND = 4
MARKET_CULL_COUNT = 10
POOL_MIN_FOR_CULL = 40
IMPERFECT_PICK_RATE = 0.10  # 10% off-archetype picks
NUM_ARCHETYPES = 8
CARDS_PER_ARCHETYPE = 40
GENERIC_CARDS = 40
TOTAL_CARDS = CARDS_PER_ARCHETYPE * NUM_ARCHETYPES + GENERIC_CARDS  # 360

ARCHETYPES = [
    "Flash",        # 0: Zephyr primary, Ember secondary
    "Blink",        # 1: Ember primary, Zephyr secondary
    "Storm",        # 2: Ember primary, Stone secondary
    "Self-Discard", # 3: Stone primary, Ember secondary
    "Self-Mill",    # 4: Stone primary, Tide secondary
    "Sacrifice",    # 5: Tide primary, Stone secondary
    "Warriors",     # 6: Tide primary, Zephyr secondary
    "Ramp",         # 7: Zephyr primary, Tide secondary
]

RESONANCES = ["Zephyr", "Ember", "Stone", "Tide"]

# Primary and secondary resonance for each archetype
ARCHETYPE_RESONANCES = {
    "Flash":        ("Zephyr", "Ember"),
    "Blink":        ("Ember", "Zephyr"),
    "Storm":        ("Ember", "Stone"),
    "Self-Discard": ("Stone", "Ember"),
    "Self-Mill":    ("Stone", "Tide"),
    "Sacrifice":    ("Tide", "Stone"),
    "Warriors":     ("Tide", "Zephyr"),
    "Ramp":         ("Zephyr", "Tide"),
}

# Graduated Realistic sibling rates (from orchestration plan)
# Each archetype has exactly two siblings: the adjacent archetypes on the circle
# that share the same primary or secondary resonance.
GRADUATED_SIBLING_RATES = {
    frozenset({"Warriors", "Sacrifice"}): 0.50,
    frozenset({"Self-Discard", "Self-Mill"}): 0.40,
    frozenset({"Blink", "Storm"}): 0.30,
    frozenset({"Flash", "Ramp"}): 0.25,
    frozenset({"Flash", "Blink"}): 0.30,
    frozenset({"Storm", "Self-Discard"}): 0.40,
    frozenset({"Self-Mill", "Sacrifice"}): 0.50,
    frozenset({"Warriors", "Ramp"}): 0.25,
}

# Pessimistic: reduce all sibling rates by 10pp
PESSIMISTIC_SIBLING_RATES = {k: max(0.0, v - 0.10) for k, v in GRADUATED_SIBLING_RATES.items()}


# ─── Data Structures ────────────────────────────────────────────────────────

@dataclass
class SimCard:
    id: int
    visible_symbols: list
    archetype: Optional[str]  # None for generic
    power: float              # 1-10
    pair_affinity: dict       # archetype -> float (0-1)
    fitness: dict             # archetype -> 'S'|'A'|'C'|'F'

    def is_sa_for(self, arch: str) -> bool:
        return self.fitness.get(arch, 'F') in ('S', 'A')


# ─── Card Pool Generation ───────────────────────────────────────────────────

def get_sibling_rate(arch: str, rates: dict) -> dict:
    """Return {sibling_archetype: rate} for the given archetype."""
    result = {}
    for pair_set, rate in rates.items():
        if arch in pair_set:
            sibling = [a for a in pair_set if a != arch][0]
            result[sibling] = rate
    return result


def generate_pool(rng: random.Random, sibling_rates: dict) -> list:
    """Generate a 360-card pool with fitness assignments."""
    cards = []
    card_id = 0

    # Decide dual-symbol counts: ~4-5 per archetype = 36 total
    dual_count_per_arch = {}
    remaining_dual = 36
    archs = list(ARCHETYPES)
    rng.shuffle(archs)
    for i, arch in enumerate(archs):
        if i < len(archs) - 1:
            count = min(rng.randint(4, 5), remaining_dual)
            dual_count_per_arch[arch] = count
            remaining_dual -= count
        else:
            dual_count_per_arch[arch] = max(0, remaining_dual)

    # Generate archetype cards (40 per archetype)
    for arch in ARCHETYPES:
        primary_res, secondary_res = ARCHETYPE_RESONANCES[arch]
        n_dual = dual_count_per_arch.get(arch, 4)
        siblings = get_sibling_rate(arch, sibling_rates)

        for i in range(CARDS_PER_ARCHETYPE):
            power = rng.uniform(1.0, 10.0)
            visible = [primary_res, secondary_res] if i < n_dual else [primary_res]

            # Pair affinity
            pair_aff = {}
            for a in ARCHETYPES:
                if a == arch:
                    pair_aff[a] = rng.uniform(0.7, 1.0)
                elif a in siblings:
                    pair_aff[a] = rng.uniform(0.3, 0.6)
                else:
                    pair_aff[a] = rng.uniform(0.0, 0.2)

            # Fitness
            fitness = {}
            fitness[arch] = rng.choice(['S', 'A'])  # home: always S/A
            for sib_arch, rate in siblings.items():
                fitness[sib_arch] = rng.choice(['S', 'A']) if rng.random() < rate else rng.choice(['C', 'F'])
            for a in ARCHETYPES:
                if a not in fitness:
                    fitness[a] = rng.choice(['C', 'F'])

            cards.append(SimCard(
                id=card_id, visible_symbols=visible, archetype=arch,
                power=power, pair_affinity=pair_aff, fitness=fitness,
            ))
            card_id += 1

    # Generate generic cards (40 cards, 0 symbols)
    for _ in range(GENERIC_CARDS):
        power = rng.uniform(1.0, 10.0)
        pair_aff = {a: rng.uniform(0.1, 0.3) for a in ARCHETYPES}
        fitness = {a: rng.choice(['C', 'F']) for a in ARCHETYPES}
        cards.append(SimCard(
            id=card_id, visible_symbols=[], archetype=None,
            power=power, pair_affinity=pair_aff, fitness=fitness,
        ))
        card_id += 1

    return cards


# ─── AI Drafter Logic ────────────────────────────────────────────────────────

def ai_score_card(card: SimCard, ai_archetype: str) -> float:
    """Score a card for an AI drafter's archetype preference."""
    if card.archetype is None:
        return 0.3 * card.power / 10.0
    return card.pair_affinity.get(ai_archetype, 0.0)


def ai_pick_from_pool(ai_archetype: str, pool_ids: set, pool_by_id: dict,
                       rng: random.Random) -> list:
    """
    AI picks 4 cards from the shared pool this round.
    3 affinity picks + 1 power pick, with 10% imperfect rate on affinity picks.
    Returns list of card ids picked.
    """
    available = [pool_by_id[cid] for cid in pool_ids]
    if not available:
        return []

    picked_ids = []
    for pick_idx in range(AI_CARDS_PER_ROUND):
        if not available:
            break

        is_power_pick = (pick_idx == 3) or (rng.random() < IMPERFECT_PICK_RATE)

        if is_power_pick:
            best = max(available, key=lambda c: c.power)
        else:
            best = max(available, key=lambda c: ai_score_card(c, ai_archetype))

        picked_ids.append(best.id)
        available = [c for c in available if c.id != best.id]

    return picked_ids


# ─── Player Strategies ──────────────────────────────────────────────────────

def committed_player_pick(pack, picked_cards, committed_arch, pick_num):
    """Committed: pick highest fitness for strongest archetype, commit at pick 5-6."""
    if committed_arch is None and pick_num >= 5:
        sa_counts = Counter()
        for c in picked_cards:
            for a in ARCHETYPES:
                if c.is_sa_for(a):
                    sa_counts[a] += 1
        committed_arch = sa_counts.most_common(1)[0][0] if sa_counts else ARCHETYPES[0]

    if committed_arch:
        sa_cards = [c for c in pack if c.is_sa_for(committed_arch)]
        if sa_cards:
            return max(sa_cards, key=lambda c: c.power), committed_arch
        return max(pack, key=lambda c: c.power), committed_arch
    else:
        # Pre-commit: pick highest power among S/A for any archetype
        best = None
        for c in pack:
            if any(c.is_sa_for(a) for a in ARCHETYPES):
                if best is None or c.power > best.power:
                    best = c
        return (best or max(pack, key=lambda c: c.power)), committed_arch


def power_chaser_pick(pack, picked_cards, committed_arch, pick_num):
    """Always pick highest power card."""
    return max(pack, key=lambda c: c.power), None


def signal_reader_pick(pack, picked_cards, committed_arch, pick_num, pool_remaining):
    """Evaluate which archetype has most remaining cards, commit by pick 5."""
    if committed_arch is None and pick_num >= 5:
        arch_counts = Counter()
        for c in pool_remaining:
            if c.archetype:
                arch_counts[c.archetype] += 1
        committed_arch = arch_counts.most_common(1)[0][0] if arch_counts else ARCHETYPES[0]

    if committed_arch:
        sa_cards = [c for c in pack if c.is_sa_for(committed_arch)]
        if sa_cards:
            return max(sa_cards, key=lambda c: c.power), committed_arch
        return max(pack, key=lambda c: c.power), committed_arch
    else:
        best = None
        for c in pack:
            if any(c.is_sa_for(a) for a in ARCHETYPES):
                if best is None or c.power > best.power:
                    best = c
        return (best or max(pack, key=lambda c: c.power)), committed_arch


# ─── Single Draft ────────────────────────────────────────────────────────────

def run_single_draft(rng, strategy, sibling_rates, trace=False):
    """Run a single 30-pick draft and return metrics data."""
    pool = generate_pool(rng, sibling_rates)
    pool_by_id = {c.id: c for c in pool}

    # Select 5 random AI archetypes
    ai_archetypes = rng.sample(ARCHETYPES, NUM_AIS)
    open_lanes = [a for a in ARCHETYPES if a not in ai_archetypes]

    # Shared pool tracking
    remaining_ids = set(c.id for c in pool)
    player_picks = []
    committed_arch = None

    # Per-pack data for metrics
    all_pack_sa = []         # S/A count per pack for committed arch (all picks)
    all_pack_cards = []      # the actual pack each round

    # AI behavior tracking
    ai_pick_counts = {a: Counter() for a in ai_archetypes}

    # Trace data
    trace_rounds = []

    for round_num in range(NUM_PICKS):
        pool_size_before = len(remaining_ids)

        # Step 1: All 5 AIs pick from the shared pool (random order)
        ai_order = list(range(len(ai_archetypes)))
        rng.shuffle(ai_order)

        round_ai_picked = []
        for ai_idx in ai_order:
            arch = ai_archetypes[ai_idx]
            picked = ai_pick_from_pool(arch, remaining_ids, pool_by_id, rng)
            for cid in picked:
                remaining_ids.discard(cid)
                round_ai_picked.append(cid)
                card = pool_by_id[cid]
                ai_pick_counts[arch][card.archetype or "Generic"] += 1

        # Step 2: Market culling (10 lowest-power cards)
        culled_count = 0
        if len(remaining_ids) > POOL_MIN_FOR_CULL:
            remaining_list = sorted(
                [pool_by_id[cid] for cid in remaining_ids],
                key=lambda c: c.power
            )
            n_cull = min(MARKET_CULL_COUNT, len(remaining_list) - POOL_MIN_FOR_CULL)
            n_cull = max(0, n_cull)
            for c in remaining_list[:n_cull]:
                remaining_ids.discard(c.id)
            culled_count = n_cull

        # Step 3: Draw 4-card pack for player
        remaining_list = [pool_by_id[cid] for cid in remaining_ids]
        if len(remaining_list) < PACK_SIZE:
            pack = remaining_list[:]
        else:
            pack = rng.sample(remaining_list, PACK_SIZE)

        if not pack:
            break

        # Step 4: Player picks
        pool_for_signal = [pool_by_id[cid] for cid in remaining_ids]
        if strategy == "committed":
            chosen, committed_arch = committed_player_pick(pack, player_picks, committed_arch, round_num + 1)
        elif strategy == "power":
            chosen, committed_arch = power_chaser_pick(pack, player_picks, committed_arch, round_num + 1)
        elif strategy == "signal":
            chosen, committed_arch = signal_reader_pick(pack, player_picks, committed_arch, round_num + 1, pool_for_signal)
        else:
            raise ValueError(f"Unknown strategy: {strategy}")

        remaining_ids.discard(chosen.id)
        player_picks.append(chosen)

        # Record data
        all_pack_cards.append(pack)
        if committed_arch:
            sa_count = sum(1 for c in pack if c.is_sa_for(committed_arch))
            all_pack_sa.append(sa_count)
        else:
            all_pack_sa.append(None)  # not yet committed

        if trace:
            # Count remaining cards per archetype
            arch_remaining = Counter()
            for cid in remaining_ids:
                c = pool_by_id[cid]
                arch_remaining[c.archetype or "Generic"] += 1

            trace_rounds.append({
                "round": round_num + 1,
                "pool_before": pool_size_before,
                "ai_took": len(round_ai_picked),
                "culled": culled_count,
                "pool_after": len(remaining_ids),
                "pack": [(c.archetype or "Generic", f"{c.power:.1f}",
                         c.is_sa_for(committed_arch) if committed_arch else "?")
                        for c in pack],
                "chosen": (chosen.archetype or "Generic", f"{chosen.power:.1f}",
                          chosen.is_sa_for(committed_arch) if committed_arch else "?"),
                "committed_arch": committed_arch,
                "arch_remaining": dict(arch_remaining),
            })

    # ─── Compute Metrics ─────────────────────────────────────────────────

    result = {
        "committed_arch": committed_arch,
        "ai_archetypes": ai_archetypes,
        "open_lanes": open_lanes,
        "player_picks": player_picks,
        "ai_pick_counts": ai_pick_counts,
    }

    # M1: Picks 1-5, unique archetypes with S/A cards per pack (target >= 3)
    m1_vals = []
    for rnd in range(min(5, len(all_pack_cards))):
        archs_with_sa = set()
        for c in all_pack_cards[rnd]:
            for a in ARCHETYPES:
                if c.is_sa_for(a):
                    archs_with_sa.add(a)
        m1_vals.append(len(archs_with_sa))
    result["m1"] = sum(m1_vals) / max(len(m1_vals), 1)

    # M2: Picks 1-5, S/A cards for emerging archetype per pack (target <= 2)
    if committed_arch:
        m2_vals = []
        for rnd in range(min(5, len(all_pack_cards))):
            m2_vals.append(sum(1 for c in all_pack_cards[rnd] if c.is_sa_for(committed_arch)))
        result["m2"] = sum(m2_vals) / max(len(m2_vals), 1)
    else:
        result["m2"] = 0

    # M3: Picks 6+, S/A cards for committed archetype per pack (target >= 2.0)
    if committed_arch:
        m3_vals = []
        for rnd in range(5, len(all_pack_cards)):
            m3_vals.append(sum(1 for c in all_pack_cards[rnd] if c.is_sa_for(committed_arch)))
        result["m3"] = sum(m3_vals) / max(len(m3_vals), 1)
        result["m3_per_pack"] = m3_vals
    else:
        result["m3"] = 0
        result["m3_per_pack"] = []

    # M4: Picks 6+, off-archetype C/F cards per pack (target >= 0.5)
    if committed_arch:
        m4_vals = []
        for rnd in range(5, len(all_pack_cards)):
            m4_vals.append(sum(1 for c in all_pack_cards[rnd] if not c.is_sa_for(committed_arch)))
        result["m4"] = sum(m4_vals) / max(len(m4_vals), 1)
    else:
        result["m4"] = 0

    # M5: Convergence pick — first pick where 3-pick rolling avg S/A >= 2.0 (target 5-8)
    if committed_arch:
        sa_seq = []
        for rnd in range(len(all_pack_cards)):
            sa_seq.append(sum(1 for c in all_pack_cards[rnd] if c.is_sa_for(committed_arch)))
        convergence = None
        for i in range(2, len(sa_seq)):
            if sum(sa_seq[i-2:i+1]) / 3.0 >= 2.0:
                convergence = i + 1
                break
        result["m5"] = convergence if convergence else NUM_PICKS + 1
    else:
        result["m5"] = NUM_PICKS + 1

    # M6: Deck archetype concentration (target 60-90%)
    if committed_arch:
        sa_deck = sum(1 for c in player_picks if c.is_sa_for(committed_arch))
        result["m6"] = sa_deck / max(len(player_picks), 1)
    else:
        result["m6"] = 0

    # M9: StdDev of S/A cards per pack, picks 6+ (target >= 0.8)
    if committed_arch and result["m3_per_pack"]:
        mean = result["m3"]
        var = sum((x - mean) ** 2 for x in result["m3_per_pack"]) / len(result["m3_per_pack"])
        result["m9"] = math.sqrt(var)
    else:
        result["m9"] = 0

    # M10: Max consecutive packs below 1.5 S/A, picks 6+ (target <= 2)
    all_streaks = []
    if committed_arch and result["m3_per_pack"]:
        max_streak = 0
        cur = 0
        for sa in result["m3_per_pack"]:
            if sa < 1.5:
                cur += 1
                max_streak = max(max_streak, cur)
            else:
                if cur > 0:
                    all_streaks.append(cur)
                cur = 0
        if cur > 0:
            all_streaks.append(cur)
        result["m10"] = max_streak
    else:
        result["m10"] = 0
    result["m10_streaks"] = all_streaks

    # M11: Picks 15+, S/A for committed archetype per pack (target >= 3.0)
    if committed_arch:
        m11_vals = []
        for rnd in range(14, len(all_pack_cards)):
            m11_vals.append(sum(1 for c in all_pack_cards[rnd] if c.is_sa_for(committed_arch)))
        result["m11"] = sum(m11_vals) / max(len(m11_vals), 1)
    else:
        result["m11"] = 0

    result["deck_card_ids"] = set(c.id for c in player_picks)

    if trace:
        result["trace_rounds"] = trace_rounds
        result["trace_ai_archetypes"] = ai_archetypes
        result["trace_open_lanes"] = open_lanes

    return result


# ─── Aggregate Simulation ───────────────────────────────────────────────────

def run_simulation(strategy, sibling_rates, n_drafts=NUM_DRAFTS, collect_traces=0):
    """Run n_drafts and aggregate metrics."""
    results = []
    traces = []

    for i in range(n_drafts):
        rng = random.Random(42 + i)
        do_trace = (i < collect_traces)
        r = run_single_draft(rng, strategy, sibling_rates, trace=do_trace)
        results.append(r)
        if do_trace:
            traces.append(r)

    agg = {}

    # Simple means
    for key in ["m1", "m2", "m3", "m4", "m5", "m6", "m9", "m10", "m11"]:
        vals = [r[key] for r in results]
        agg[f"{key}_mean"] = sum(vals) / len(vals)

    # Per-archetype M3
    arch_m3 = defaultdict(list)
    for r in results:
        if r["committed_arch"]:
            arch_m3[r["committed_arch"]].append(r["m3"])
    agg["per_arch_m3"] = {a: (sum(v)/len(v) if v else 0) for a, v in arch_m3.items()}

    # M7: Run-to-run card overlap (target < 40%)
    arch_runs = defaultdict(list)
    for r in results:
        if r["committed_arch"]:
            arch_runs[r["committed_arch"]].append(r["deck_card_ids"])
    overlaps = []
    for arch, decks in arch_runs.items():
        for i in range(len(decks) - 1):
            union = decks[i] | decks[i+1]
            inter = decks[i] & decks[i+1]
            if union:
                overlaps.append(len(inter) / len(union))
    agg["m7_overlap"] = sum(overlaps) / max(len(overlaps), 1)

    # M8: Archetype frequency (target: none > 20%, none < 5%)
    arch_freq = Counter()
    for r in results:
        if r["committed_arch"]:
            arch_freq[r["committed_arch"]] += 1
    total = sum(arch_freq.values())
    agg["m8_freq"] = {a: arch_freq.get(a, 0) / max(total, 1) for a in ARCHETYPES}
    agg["m8_max"] = max(agg["m8_freq"].values()) if agg["m8_freq"] else 0
    agg["m8_min"] = min(agg["m8_freq"].values()) if agg["m8_freq"] else 0

    # M10 streak distribution
    all_streaks = []
    for r in results:
        all_streaks.extend(r["m10_streaks"])
    agg["m10_streak_dist"] = Counter(all_streaks)

    # Pack quality distribution (S/A per pack, picks 6+)
    all_late_sa = []
    for r in results:
        all_late_sa.extend(r["m3_per_pack"])
    all_late_sa.sort()
    n = len(all_late_sa)
    if n > 0:
        agg["pq_p10"] = all_late_sa[int(n * 0.10)]
        agg["pq_p25"] = all_late_sa[int(n * 0.25)]
        agg["pq_p50"] = all_late_sa[int(n * 0.50)]
        agg["pq_p75"] = all_late_sa[int(n * 0.75)]
        agg["pq_p90"] = all_late_sa[min(int(n * 0.90), n - 1)]
    else:
        for p in ["pq_p10", "pq_p25", "pq_p50", "pq_p75", "pq_p90"]:
            agg[p] = 0

    # AI behavior summary
    total_ai_picks = Counter()
    total_ai_count = 0
    for r in results:
        for ai_arch, picks in r["ai_pick_counts"].items():
            for card_arch, cnt in picks.items():
                total_ai_picks[card_arch] += cnt
                total_ai_count += cnt
    agg["ai_total_picks"] = total_ai_count
    agg["ai_picks_by_arch"] = dict(total_ai_picks)

    agg["traces"] = traces
    return agg


# ─── Output Formatting ──────────────────────────────────────────────────────

def format_trace(r):
    """Format a single draft trace."""
    lines = []
    lines.append(f"  Strategy: {r.get('trace_rounds', [{}])[0].get('committed_arch', 'N/A') if r.get('trace_rounds') else 'N/A'}")

    ai_archs = r.get("trace_ai_archetypes", [])
    open_lanes = r.get("trace_open_lanes", [])
    committed = r.get("committed_arch", "None")

    lines.append(f"  AI Archetypes: {', '.join(ai_archs)}")
    lines.append(f"  Open Lanes: {', '.join(open_lanes)}")
    lines.append(f"  Player committed to: {committed}")
    lines.append(f"  Player committed to open lane: {committed in open_lanes if committed else 'N/A'}")
    lines.append("")

    for rd in r.get("trace_rounds", []):
        rn = rd["round"]
        if rn <= 10 or rn in [15, 20, 25, 30]:
            pack_str = " | ".join(
                f"{a}({p},{'S/A' if sa else 'C/F' if sa is not None else '?'})"
                for a, p, sa in rd["pack"]
            )
            ch_a, ch_p, ch_sa = rd["chosen"]
            lines.append(
                f"  Pick {rn:2d} | Pool {rd['pool_before']:3d} -> {rd['pool_after']:3d} "
                f"(AI:{rd['ai_took']}, cull:{rd['culled']}) | "
                f"Pack: [{pack_str}] | "
                f"Chose: {ch_a}({ch_p},{'S/A' if ch_sa else 'C/F' if ch_sa is not None else '?'}) | "
                f"Arch: {rd['committed_arch'] or 'undecided'}"
            )

    # Show pool composition at key points
    for rd in r.get("trace_rounds", []):
        if rd["round"] in [1, 5, 10, 15, 20, 30]:
            ar = rd.get("arch_remaining", {})
            comp = ", ".join(f"{a}:{ar.get(a, 0)}" for a in ARCHETYPES)
            lines.append(f"    Pool at pick {rd['round']}: {comp}, Generic:{ar.get('Generic', 0)}")

    lines.append("")
    return "\n".join(lines)


def format_results(label, agg):
    """Format aggregated results."""
    lines = []
    lines.append(f"=== {label} ===")
    lines.append(f"  M1  (unique archs S/A, picks 1-5):  {agg['m1_mean']:.2f}  (target >= 3)")
    lines.append(f"  M2  (emerging S/A, picks 1-5):      {agg['m2_mean']:.2f}  (target <= 2)")
    lines.append(f"  M3  (committed S/A, picks 6+):      {agg['m3_mean']:.2f}  (target >= 2.0)")
    lines.append(f"  M4  (off-arch C/F, picks 6+):       {agg['m4_mean']:.2f}  (target >= 0.5)")
    lines.append(f"  M5  (convergence pick):              {agg['m5_mean']:.1f}   (target 5-8)")
    lines.append(f"  M6  (deck concentration):            {agg['m6_mean']:.1%}  (target 60-90%)")
    lines.append(f"  M7  (run-to-run overlap):            {agg['m7_overlap']:.1%}  (target < 40%)")
    lines.append(f"  M8  (arch freq max/min):             {agg['m8_max']:.1%}/{agg['m8_min']:.1%}  (none>20%, none<5%)")
    lines.append(f"  M9  (S/A StdDev, picks 6+):         {agg['m9_mean']:.2f}  (target >= 0.8)")
    lines.append(f"  M10 (max consec < 1.5 S/A):         {agg['m10_mean']:.1f}   (target <= 2)")
    lines.append(f"  M11 (committed S/A, picks 15+):     {agg['m11_mean']:.2f}  (target >= 3.0)")
    lines.append("")

    lines.append("  Per-Archetype M3:")
    for a in ARCHETYPES:
        v = agg["per_arch_m3"].get(a, 0)
        lines.append(f"    {a:15s}: {v:.2f}")
    lines.append("")

    lines.append("  Pack Quality Distribution (S/A per pack, picks 6+):")
    lines.append(f"    p10={agg['pq_p10']}  p25={agg['pq_p25']}  p50={agg['pq_p50']}  "
                 f"p75={agg['pq_p75']}  p90={agg['pq_p90']}")
    lines.append("")

    lines.append("  Archetype Frequencies:")
    for a in ARCHETYPES:
        lines.append(f"    {a:15s}: {agg['m8_freq'].get(a, 0):.1%}")
    lines.append("")

    if agg["m10_streak_dist"]:
        lines.append("  Bad Pack Streak Distribution (consecutive packs < 1.5 S/A, picks 6+):")
        for length in sorted(agg["m10_streak_dist"].keys()):
            lines.append(f"    Length {length}: {agg['m10_streak_dist'][length]} occurrences")
        lines.append("")

    lines.append(f"  AI Total Picks: {agg['ai_total_picks']} across {NUM_DRAFTS} drafts "
                 f"({agg['ai_total_picks'] / NUM_DRAFTS:.0f}/draft)")
    lines.append("  AI Picks by Card Archetype:")
    for a in ARCHETYPES + ["Generic"]:
        cnt = agg["ai_picks_by_arch"].get(a, 0)
        pct = cnt / max(agg["ai_total_picks"], 1)
        lines.append(f"    {a:15s}: {cnt:7d} ({pct:.1%})")
    lines.append("")

    return "\n".join(lines)


# ─── Main ────────────────────────────────────────────────────────────────────

def main():
    print("=" * 70)
    print("D1 Open Table — Monte Carlo Simulation (v2)")
    print("=" * 70)
    print(f"Drafts: {NUM_DRAFTS}, Picks: {NUM_PICKS}, AIs: {NUM_AIS}")
    print(f"AI cards/round: {AI_CARDS_PER_ROUND}, Market cull: {MARKET_CULL_COUNT}")
    print(f"Imperfect pick rate: {IMPERFECT_PICK_RATE}")
    print()

    # Graduated Realistic
    print("Running Committed Player (Graduated Realistic)...")
    committed_gr = run_simulation("committed", GRADUATED_SIBLING_RATES, collect_traces=2)
    print(format_results("Committed Player — Graduated Realistic", committed_gr))

    print("Running Power-Chaser (Graduated Realistic)...")
    power_gr = run_simulation("power", GRADUATED_SIBLING_RATES)
    print(format_results("Power-Chaser — Graduated Realistic", power_gr))

    print("Running Signal-Reader (Graduated Realistic)...")
    signal_gr = run_simulation("signal", GRADUATED_SIBLING_RATES, collect_traces=2)
    print(format_results("Signal-Reader — Graduated Realistic", signal_gr))

    # Pessimistic
    print("Running Committed Player (Pessimistic)...")
    committed_pess = run_simulation("committed", PESSIMISTIC_SIBLING_RATES)
    print(format_results("Committed Player — Pessimistic", committed_pess))

    # Comparison
    print("=" * 70)
    print("PESSIMISTIC COMPARISON (Committed):")
    print(f"  M3:  GR={committed_gr['m3_mean']:.2f}  Pess={committed_pess['m3_mean']:.2f}")
    print(f"  M10: GR={committed_gr['m10_mean']:.1f}   Pess={committed_pess['m10_mean']:.1f}")
    print(f"  M11: GR={committed_gr['m11_mean']:.2f}  Pess={committed_pess['m11_mean']:.2f}")
    print()

    # Traces
    print("=" * 70)
    print("EXAMPLE DRAFT TRACES")
    print("=" * 70)
    if committed_gr["traces"]:
        print("\n--- Committed Player Trace (Draft 1) ---")
        print(format_trace(committed_gr["traces"][0]))
    if signal_gr["traces"]:
        print("\n--- Signal-Reader Trace (Draft 1) ---")
        print(format_trace(signal_gr["traces"][0]))

    # V9 Comparison
    print("=" * 70)
    print("V9 HYBRID B COMPARISON (Committed, Graduated Realistic)")
    print("=" * 70)
    print(f"  {'Metric':<8} {'V9':>10} {'D1':>10} {'Delta':>10}")
    for name, v9, d1 in [
        ("M3", 2.70, committed_gr["m3_mean"]),
        ("M5", 9.6, committed_gr["m5_mean"]),
        ("M6", 0.86, committed_gr["m6_mean"]),
        ("M10", 3.8, committed_gr["m10_mean"]),
        ("M11", 3.25, committed_gr["m11_mean"]),
    ]:
        print(f"  {name:<8} {v9:>10.2f} {d1:>10.2f} {d1-v9:>+10.2f}")
    print()

    # Pass/Fail
    print("=" * 70)
    print("PASS/FAIL SCORECARD (Committed, Graduated Realistic)")
    print("=" * 70)
    g = committed_gr
    metrics = [
        ("M1",  g["m1_mean"],   ">=3",   g["m1_mean"] >= 3.0),
        ("M2",  g["m2_mean"],   "<=2",   g["m2_mean"] <= 2.0),
        ("M3",  g["m3_mean"],   ">=2.0", g["m3_mean"] >= 2.0),
        ("M4",  g["m4_mean"],   ">=0.5", g["m4_mean"] >= 0.5),
        ("M5",  g["m5_mean"],   "5-8",   5 <= g["m5_mean"] <= 8),
        ("M6",  g["m6_mean"],   "60-90%", 0.60 <= g["m6_mean"] <= 0.90),
        ("M7",  g["m7_overlap"],"<40%",  g["m7_overlap"] < 0.40),
        ("M8",  f"{g['m8_max']:.1%}/{g['m8_min']:.1%}", "<=20%/>=5%",
         g["m8_max"] <= 0.20 and g["m8_min"] >= 0.05),
        ("M9",  g["m9_mean"],   ">=0.8", g["m9_mean"] >= 0.8),
        ("M10", g["m10_mean"],  "<=2",   g["m10_mean"] <= 2.0),
        ("M11", g["m11_mean"],  ">=3.0", g["m11_mean"] >= 3.0),
    ]

    pass_count = 0
    for name, val, target, passed in metrics:
        status = "PASS" if passed else "FAIL"
        if passed:
            pass_count += 1
        if isinstance(val, float):
            print(f"  {name:<5} {val:>8.2f}  target {target:<12}  [{status}]")
        else:
            print(f"  {name:<5} {val:>8}  target {target:<12}  [{status}]")

    print(f"\n  Total: {pass_count}/{len(metrics)} metrics passed")
    print()


if __name__ == "__main__":
    main()
