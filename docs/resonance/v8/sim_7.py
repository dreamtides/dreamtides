#!/usr/bin/env python3
"""
Simulation Agent 7: CSCT (Commitment-Scaled Continuous Targeting)
Jittered+Bias variant: multiplier=5, bias=1.5x, 15% slot jitter, floor=1 from pick 3.

The number of pair-matched slots scales continuously with commitment ratio
C = pair_count / total_picks. targeted_slots = floor(min(C * multiplier, 3)).
Remaining slots drawn with bias toward primary resonance. Per-slot jitter adds
organic variance.
"""

import random
import math
from collections import defaultdict
from dataclasses import dataclass, field
import statistics

# ── Constants ──

RESONANCES = ["Ember", "Stone", "Tide", "Zephyr"]

ARCHETYPES = [
    {"name": "Flash",         "primary": "Zephyr", "secondary": "Ember"},
    {"name": "Blink",         "primary": "Ember",  "secondary": "Zephyr"},
    {"name": "Storm",         "primary": "Ember",  "secondary": "Stone"},
    {"name": "Self-Discard",  "primary": "Stone",  "secondary": "Ember"},
    {"name": "Self-Mill",     "primary": "Stone",  "secondary": "Tide"},
    {"name": "Sacrifice",     "primary": "Tide",   "secondary": "Stone"},
    {"name": "Warriors",      "primary": "Tide",   "secondary": "Zephyr"},
    {"name": "Ramp",          "primary": "Zephyr", "secondary": "Tide"},
]

ARCHETYPE_NAMES = [a["name"] for a in ARCHETYPES]
ARCH_BY_NAME = {a["name"]: a for a in ARCHETYPES}

# Co-primary sibling pairs (share primary resonance)
CO_PRIMARY_PAIRS = {
    "Flash": "Ramp", "Ramp": "Flash",
    "Blink": "Storm", "Storm": "Blink",
    "Self-Discard": "Self-Mill", "Self-Mill": "Self-Discard",
    "Sacrifice": "Warriors", "Warriors": "Sacrifice",
}

# ── Fitness Models ──

def make_fitness_model(warriors_sac, discard_mill, blink_storm, flash_ramp):
    """Build per-pair fitness dict from the four co-primary pair rates."""
    return {
        ("Warriors", "Sacrifice"): warriors_sac,
        ("Sacrifice", "Warriors"): warriors_sac,
        ("Self-Discard", "Self-Mill"): discard_mill,
        ("Self-Mill", "Self-Discard"): discard_mill,
        ("Blink", "Storm"): blink_storm,
        ("Storm", "Blink"): blink_storm,
        ("Flash", "Ramp"): flash_ramp,
        ("Ramp", "Flash"): flash_ramp,
    }

FITNESS_MODELS = {
    "Optimistic": make_fitness_model(1.0, 1.0, 1.0, 1.0),
    "Graduated_Realistic": make_fitness_model(0.50, 0.40, 0.30, 0.25),
    "Pessimistic": make_fitness_model(0.35, 0.25, 0.15, 0.10),
    "Hostile": make_fitness_model(0.08, 0.08, 0.08, 0.08),
}

# ── Card and Pool ──

@dataclass
class Card:
    id: int
    symbols: list          # ordered resonance strings
    home_archetype: str    # primary archetype
    power: float           # 0-10 raw strength
    is_generic: bool = False
    # Pre-computed archetype tiers: archetype_name -> "S"/"A"/"B"/"F"
    tiers: dict = field(default_factory=dict)

    @property
    def primary_res(self):
        return self.symbols[0] if self.symbols else None

    @property
    def pair(self):
        if len(self.symbols) >= 2:
            return (self.symbols[0], self.symbols[1])
        return None

    def is_sa(self, archetype_name):
        return self.tiers.get(archetype_name, "F") in ("S", "A")


def assign_tiers(card, fitness_model, rng):
    """Pre-compute card tiers for all archetypes given a fitness model."""
    tiers = {}
    for arch in ARCHETYPES:
        aname = arch["name"]
        if card.is_generic:
            # Generic cards: ~25% chance of being B-tier (usable but not S/A).
            # They don't count as S/A for any archetype.
            tiers[aname] = "B"
            continue

        if card.home_archetype == aname:
            tiers[aname] = "S"
        elif card.home_archetype == CO_PRIMARY_PAIRS.get(aname):
            # Sibling archetype: A-tier with probability = fitness rate
            pair_key = (aname, card.home_archetype)
            rate = fitness_model.get(pair_key, 0.0)
            if rng.random() < rate:
                tiers[aname] = "A"
            else:
                tiers[aname] = "B"
        else:
            # Non-sibling: check if secondary resonance matches
            # Cards from adjacent archetypes on the circle get a small chance
            # of being B-tier (usable). Everyone else is F.
            arch_info = ARCH_BY_NAME[aname]
            card_arch_info = ARCH_BY_NAME.get(card.home_archetype)
            if card_arch_info and (
                card_arch_info["primary"] == arch_info["secondary"] or
                card_arch_info["secondary"] == arch_info["primary"]
            ):
                tiers[aname] = "B"  # Adjacent on circle
            else:
                tiers[aname] = "F"
    card.tiers = tiers


def build_pool(dual_res_pct=0.40, total_cards=360, generic_count=40,
               fitness_model=None, seed=42):
    """Build a card pool with pre-computed tiers."""
    pool_rng = random.Random(seed)
    cards = []
    card_id = 0

    # Generic cards
    for _ in range(generic_count):
        c = Card(id=card_id, symbols=[], home_archetype="Generic",
                 power=pool_rng.uniform(4.0, 7.0), is_generic=True)
        cards.append(c)
        card_id += 1

    resonance_cards = total_cards - generic_count
    dual_res_count = int(resonance_cards * dual_res_pct)
    single_res_count = resonance_cards - dual_res_count

    # Single-resonance: distribute evenly across archetypes
    single_per_arch = single_res_count // 8
    single_rem = single_res_count - single_per_arch * 8

    for i, arch in enumerate(ARCHETYPES):
        count = single_per_arch + (1 if i < single_rem else 0)
        for _ in range(count):
            c = Card(id=card_id, symbols=[arch["primary"]],
                     home_archetype=arch["name"],
                     power=pool_rng.uniform(3.0, 8.0))
            cards.append(c)
            card_id += 1

    # Dual-resonance: distribute evenly across archetypes
    dual_per_arch = dual_res_count // 8
    dual_rem = dual_res_count - dual_per_arch * 8

    for i, arch in enumerate(ARCHETYPES):
        count = dual_per_arch + (1 if i < dual_rem else 0)
        for _ in range(count):
            if pool_rng.random() < 0.60:
                remaining = [r for r in RESONANCES
                             if r != arch["primary"] and r != arch["secondary"]]
                third = pool_rng.choice(remaining)
                syms = [arch["primary"], arch["secondary"], third]
            else:
                syms = [arch["primary"], arch["secondary"]]
            c = Card(id=card_id, symbols=syms, home_archetype=arch["name"],
                     power=pool_rng.uniform(3.0, 8.0))
            cards.append(c)
            card_id += 1

    # Assign tiers
    if fitness_model:
        tier_rng = random.Random(seed + 999)
        for c in cards:
            assign_tiers(c, fitness_model, tier_rng)

    pool_rng.shuffle(cards)
    return cards


def build_indexes(pool):
    """Pre-compute lookup indexes."""
    by_pair = defaultdict(list)
    by_primary = defaultdict(list)
    all_resonance = []

    for card in pool:
        if card.is_generic:
            continue
        all_resonance.append(card)
        if card.primary_res:
            by_primary[card.primary_res].append(card)
        if card.pair:
            by_pair[card.pair].append(card)

    return by_pair, by_primary, all_resonance


# ── CSCT Algorithm ──

PACK_SIZE = 4
NUM_PICKS = 30
CSCT_MULTIPLIER = 5.0
CSCT_BIAS = 1.5
CSCT_JITTER = 0.15
CSCT_FLOOR_START = 3
CSCT_MAX_PAIR_SLOTS = 3


def detect_leading_archetype(drafted_cards):
    """
    Detect the archetype the player is most committed to.
    Returns (archetype_name, pair_tuple, pair_count).
    pair_count: number of drafted cards that match this archetype's resonance pair.
    """
    if not drafted_cards:
        return None, None, 0

    # Score each archetype by how many drafted cards are S/A for it
    arch_scores = defaultdict(float)
    for card in drafted_cards:
        for aname in ARCHETYPE_NAMES:
            if card.is_sa(aname):
                arch_scores[aname] += 1.0

    if not arch_scores:
        return None, None, 0

    best_arch = max(arch_scores, key=arch_scores.get)
    arch_info = ARCH_BY_NAME[best_arch]
    pair = (arch_info["primary"], arch_info["secondary"])

    # Count drafted cards matching this pair (for commitment ratio)
    pair_count = 0
    for card in drafted_cards:
        if card.is_sa(best_arch):
            pair_count += 1

    return best_arch, pair, pair_count


def generate_pack_csct(pick_num, drafted_cards, pool, indexes, rng,
                       multiplier=CSCT_MULTIPLIER, bias=CSCT_BIAS,
                       jitter=CSCT_JITTER, floor_start=CSCT_FLOOR_START,
                       max_pair_slots=CSCT_MAX_PAIR_SLOTS):
    """Generate a 4-card pack using the CSCT algorithm."""
    by_pair, by_primary, all_resonance = indexes

    arch_name, pair, pair_count = detect_leading_archetype(drafted_cards)
    total_picks = len(drafted_cards)

    # Commitment ratio
    C = pair_count / total_picks if total_picks > 0 else 0.0

    # Base targeted slots
    raw_slots = min(C * multiplier, max_pair_slots)
    base_pair_slots = int(math.floor(raw_slots))

    # Floor: from pick floor_start+, at least 1 pair-matched slot
    if pick_num >= floor_start and arch_name is not None:
        base_pair_slots = max(base_pair_slots, 1)

    pair_pool = by_pair.get(pair, []) if pair else []
    primary_res = pair[0] if pair else None

    pack = []
    pair_slots_used = 0

    for slot_idx in range(PACK_SIZE):
        is_pair_slot = slot_idx < base_pair_slots

        # Jitter: 15% chance to flip
        if rng.random() < jitter:
            is_pair_slot = not is_pair_slot

        # Don't exceed max
        if is_pair_slot and pair_slots_used >= max_pair_slots:
            is_pair_slot = False

        if is_pair_slot and pair_pool:
            card = rng.choice(pair_pool)
            pack.append(card)
            pair_slots_used += 1
        else:
            # Random slot with optional bias toward primary resonance
            if primary_res and bias > 1.0:
                candidates = all_resonance
                weights = []
                for c in candidates:
                    w = bias if c.primary_res == primary_res else 1.0
                    weights.append(w)
                # Weighted random choice
                total_w = sum(weights)
                r = rng.random() * total_w
                cumul = 0.0
                chosen = candidates[-1]
                for c, w in zip(candidates, weights):
                    cumul += w
                    if r <= cumul:
                        chosen = c
                        break
                pack.append(chosen)
            else:
                pack.append(rng.choice(all_resonance if all_resonance else pool))

    return pack


# ── Player Strategies ──

def strategy_committed(pack, drafted_cards, pick_num, rng):
    """Picks highest fitness for strongest archetype. Commits around pick 5-6."""
    arch_name, _, _ = detect_leading_archetype(drafted_cards)

    if arch_name is None or pick_num <= 2:
        return max(pack, key=lambda c: c.power)

    def score(card):
        tier = card.tiers.get(arch_name, "F")
        tier_val = {"S": 4, "A": 3, "B": 1, "F": 0}[tier]
        return (tier_val, card.power)

    return max(pack, key=score)


def strategy_power_chaser(pack, drafted_cards, pick_num, rng):
    """Picks highest raw power regardless of archetype."""
    return max(pack, key=lambda c: c.power)


def strategy_signal_reader(pack, drafted_cards, pick_num, rng):
    """Evaluates open archetypes and drafts toward strongest signals."""
    if pick_num <= 3:
        return max(pack, key=lambda c: c.power)

    arch_name, _, _ = detect_leading_archetype(drafted_cards)
    if arch_name is None:
        return max(pack, key=lambda c: c.power)

    def score(card):
        primary_val = {"S": 4, "A": 3, "B": 1, "F": 0}[card.tiers.get(arch_name, "F")]
        best_alt = max(
            ({"S": 4, "A": 3, "B": 1, "F": 0}[card.tiers.get(a, "F")]
             for a in ARCHETYPE_NAMES if a != arch_name),
            default=0,
        )
        weight = min(pick_num / 15.0, 1.0)
        return weight * primary_val + (1 - weight) * max(primary_val, best_alt) + card.power * 0.05

    return max(pack, key=score)


STRATEGIES = {
    "committed": strategy_committed,
    "power_chaser": strategy_power_chaser,
    "signal_reader": strategy_signal_reader,
}


# ── Simulation ──

def simulate_draft(pool, indexes, strategy_fn, rng,
                   multiplier=CSCT_MULTIPLIER, bias=CSCT_BIAS,
                   jitter=CSCT_JITTER):
    """Simulate a single 30-pick draft."""
    drafted = []
    pick_data = []

    for pick_num in range(1, NUM_PICKS + 1):
        pack = generate_pack_csct(pick_num, drafted, pool, indexes, rng,
                                  multiplier=multiplier, bias=bias, jitter=jitter)
        chosen = strategy_fn(pack, drafted, pick_num, rng)
        drafted.append(chosen)

        arch_name, pair, pair_count = detect_leading_archetype(drafted)
        sa_count = 0
        off_count = 0
        unique_archs = set()

        if arch_name:
            for card in pack:
                if card.is_sa(arch_name):
                    sa_count += 1
                if card.tiers.get(arch_name, "F") == "F":
                    off_count += 1
            for a in ARCHETYPE_NAMES:
                if any(c.is_sa(a) for c in pack):
                    unique_archs.add(a)

        C = pair_count / pick_num if pick_num > 0 else 0

        pick_data.append({
            "pick_num": pick_num,
            "pack": pack,
            "chosen": chosen,
            "committed_arch": arch_name,
            "sa_count": sa_count,
            "off_arch_count": off_count,
            "unique_archs": len(unique_archs),
            "commitment_ratio": C,
        })

    return pick_data, drafted


def compute_metrics(all_drafts_data):
    """Compute all 10 metrics from simulation data."""
    early_unique = []
    early_sa = []
    post_sa = []
    post_off = []
    convergence_picks = []
    deck_conc = []
    per_arch_m3 = defaultdict(list)
    post_sa_all = []
    pack_sequences = []

    for draft_data, drafted in all_drafts_data:
        final_arch = draft_data[-1]["committed_arch"] if draft_data else None
        if final_arch is None:
            continue

        # M1, M2
        for pd in draft_data[:5]:
            early_unique.append(pd["unique_archs"])
            early_sa.append(pd["sa_count"])

        # Re-evaluate all post-commit packs against final archetype
        draft_seq = []
        for pd in draft_data[5:]:
            sa = sum(1 for c in pd["pack"] if c.is_sa(final_arch))
            off = sum(1 for c in pd["pack"]
                      if c.tiers.get(final_arch, "F") == "F")
            post_sa.append(sa)
            post_off.append(off)
            per_arch_m3[final_arch].append(sa)
            post_sa_all.append(sa)
            draft_seq.append(sa)
        pack_sequences.append(draft_seq)

        # M5: convergence pick
        conv = NUM_PICKS
        for i in range(4, len(draft_data)):
            window = []
            for j in range(max(0, i - 4), i + 1):
                sa_val = sum(1 for c in draft_data[j]["pack"]
                             if c.is_sa(final_arch))
                window.append(sa_val)
            if statistics.mean(window) >= 2.0:
                conv = i + 1
                break
        convergence_picks.append(conv)

        # M6: deck concentration
        sa_deck = sum(1 for c in drafted if c.is_sa(final_arch))
        deck_conc.append(sa_deck / len(drafted))

    # M7: run-to-run variety
    overlaps = []
    all_drafted = [d[1] for d in all_drafts_data]
    sample_count = min(500, len(all_drafted))
    for _ in range(sample_count):
        if len(all_drafted) < 2:
            break
        i, j = random.sample(range(len(all_drafted)), 2)
        ids_i = set(c.id for c in all_drafted[i])
        ids_j = set(c.id for c in all_drafted[j])
        union = len(ids_i | ids_j)
        if union > 0:
            overlaps.append(len(ids_i & ids_j) / union)

    # M8: archetype frequency
    arch_freq = defaultdict(int)
    total = len(all_drafts_data)
    for data, _ in all_drafts_data:
        final = data[-1]["committed_arch"] if data else None
        if final:
            arch_freq[final] += 1

    # M9
    m9 = statistics.stdev(post_sa_all) if len(post_sa_all) > 1 else 0

    # M10: consecutive bad packs
    max_consec = 0
    avg_consec_list = []
    for seq in pack_sequences:
        run = 0
        worst = 0
        runs = []
        for s in seq:
            if s < 1.5:
                run += 1
                worst = max(worst, run)
            else:
                if run > 0:
                    runs.append(run)
                run = 0
        if run > 0:
            runs.append(run)
            worst = max(worst, run)
        max_consec = max(max_consec, worst)
        avg_consec_list.append(statistics.mean(runs) if runs else 0)

    # Pack quality percentiles
    sorted_sa = sorted(post_sa_all)
    n = len(sorted_sa)
    def pctl(p):
        return sorted_sa[min(int(p / 100.0 * n), n - 1)] if n > 0 else 0

    return {
        "M1": statistics.mean(early_unique) if early_unique else 0,
        "M2": statistics.mean(early_sa) if early_sa else 0,
        "M3": statistics.mean(post_sa) if post_sa else 0,
        "M4": statistics.mean(post_off) if post_off else 0,
        "M5": statistics.mean(convergence_picks) if convergence_picks else 0,
        "M6": statistics.mean(deck_conc) if deck_conc else 0,
        "M7": statistics.mean(overlaps) if overlaps else 0,
        "M8": {a: arch_freq.get(a, 0) / max(total, 1) for a in ARCHETYPE_NAMES},
        "M9": m9,
        "M10_max": max_consec,
        "M10_avg": statistics.mean(avg_consec_list) if avg_consec_list else 0,
        "per_arch_m3": {a: statistics.mean(v) if v else 0
                        for a, v in per_arch_m3.items()},
        "pctl": {"p10": pctl(10), "p25": pctl(25), "p50": pctl(50),
                 "p75": pctl(75), "p90": pctl(90)},
    }


def run_sim(num_drafts=1000, fitness_name="Graduated_Realistic",
            dual_res_pct=0.40, strategy_name="committed",
            multiplier=CSCT_MULTIPLIER, bias=CSCT_BIAS,
            jitter=CSCT_JITTER, seed=42):
    """Run a full simulation batch."""
    fitness_model = FITNESS_MODELS[fitness_name]
    pool = build_pool(dual_res_pct=dual_res_pct, fitness_model=fitness_model, seed=seed)
    indexes = build_indexes(pool)
    strategy_fn = STRATEGIES[strategy_name]

    all_drafts = []
    for i in range(num_drafts):
        drng = random.Random(seed * 1000 + i)
        data, drafted = simulate_draft(pool, indexes, strategy_fn, drng,
                                        multiplier=multiplier, bias=bias,
                                        jitter=jitter)
        all_drafts.append((data, drafted))

    return compute_metrics(all_drafts), all_drafts, pool, indexes


def print_metrics(m, label):
    """Pretty-print metrics."""
    print(f"\n{'─'*60}")
    print(f"  {label}")
    print(f"{'─'*60}")
    tgt = lambda v, op, t: "PASS" if (v >= t if op == ">=" else v <= t if op == "<=" else v < t) else "FAIL"
    print(f"  M1  Unique archs (early):    {m['M1']:.2f}  {tgt(m['M1'],'>=',3)}")
    print(f"  M2  SA emerging (early):     {m['M2']:.2f}  {tgt(m['M2'],'<=',2)}")
    print(f"  M3  SA post-commit:          {m['M3']:.2f}  {tgt(m['M3'],'>=',2.0)}")
    print(f"  M4  Off-archetype (post):    {m['M4']:.2f}  {tgt(m['M4'],'>=',0.5)}")
    print(f"  M5  Convergence pick:        {m['M5']:.1f}  {tgt(m['M5'],'<=',8)}")
    print(f"  M6  Deck concentration:      {m['M6']:.1%}  {'PASS' if 0.60 <= m['M6'] <= 0.90 else 'FAIL'}")
    print(f"  M7  Run overlap:             {m['M7']:.1%}  {tgt(m['M7'],'<',0.40)}")
    print(f"  M9  SA StdDev:               {m['M9']:.2f}  {tgt(m['M9'],'>=',0.8)}")
    print(f"  M10 Max consec < 1.5:        {m['M10_max']}  {tgt(m['M10_max'],'<=',2)}")
    p = m["pctl"]
    print(f"  Pack pctls: P10={p['p10']} P25={p['p25']} P50={p['p50']} P75={p['p75']} P90={p['p90']}")
    print(f"  M8 Archetype frequency:")
    for a in ARCHETYPE_NAMES:
        print(f"    {a:16s}: {m['M8'].get(a, 0):.1%}")
    print(f"  Per-archetype M3:")
    for a in ARCHETYPE_NAMES:
        v = m['per_arch_m3'].get(a, 0)
        print(f"    {a:16s}: {v:.2f}")


def run_trace(pool, indexes, strategy_fn, seed, label,
              multiplier=CSCT_MULTIPLIER, bias=CSCT_BIAS, jitter=CSCT_JITTER):
    """Run and print a detailed draft trace."""
    rng = random.Random(seed)
    data, drafted = simulate_draft(pool, indexes, strategy_fn, rng,
                                    multiplier=multiplier, bias=bias, jitter=jitter)
    print(f"\n{'='*70}")
    print(f"  DRAFT TRACE: {label}")
    print(f"{'='*70}")
    for pd in data:
        p = pd["pick_num"]
        arch = pd["committed_arch"] or "?"
        sa = pd["sa_count"]
        cr = pd["commitment_ratio"]
        ch = pd["chosen"]
        syms = "/".join(ch.symbols) if ch.symbols else "generic"
        if p <= 10 or p % 5 == 0 or p == NUM_PICKS:
            print(f"  Pick {p:2d}: C={cr:.2f} SA={sa} Arch={arch:14s} "
                  f"Chose: {ch.home_archetype} ({syms}) pw={ch.power:.1f}")
    final = data[-1]["committed_arch"]
    sa_post = [sum(1 for c in pd["pack"] if c.is_sa(final))
               for pd in data[5:]]
    print(f"  Final: {final}, Avg S/A (6+): {statistics.mean(sa_post):.2f}")
    return data, drafted


# ── Main ──

def main():
    print("=" * 70)
    print("  CSCT Simulation — Jittered+Bias (mult=5, bias=1.5x, jitter=15%)")
    print("=" * 70)

    # ── Core results: all fitness × 2 pools × committed strategy ──
    core_results = {}
    for pool_label, dpct in [("V7_15pct", 0.15), ("Enriched_40pct", 0.40)]:
        for fname in ["Optimistic", "Graduated_Realistic", "Pessimistic", "Hostile"]:
            key = f"{pool_label}|{fname}"
            m, drafts, _, _ = run_sim(num_drafts=1000, fitness_name=fname,
                                       dual_res_pct=dpct, strategy_name="committed")
            core_results[key] = m
            print_metrics(m, f"{pool_label} | {fname} | committed")

    # ── Summary table ──
    print("\n\n" + "=" * 70)
    print("  SUMMARY: M3 across all conditions (committed strategy)")
    print("=" * 70)
    print(f"  {'Pool':16s} {'Fitness':22s} {'M3':>5s} {'M4':>5s} {'M5':>5s} "
          f"{'M6':>6s} {'M9':>5s} {'M10':>4s}")
    print("  " + "-" * 65)
    for pool_label, dpct in [("V7_15pct", 0.15), ("Enriched_40pct", 0.40)]:
        for fname in ["Optimistic", "Graduated_Realistic", "Pessimistic", "Hostile"]:
            key = f"{pool_label}|{fname}"
            m = core_results[key]
            print(f"  {pool_label:16s} {fname:22s} {m['M3']:5.2f} {m['M4']:5.2f} "
                  f"{m['M5']:5.1f} {m['M6']:6.1%} {m['M9']:5.2f} {m['M10_max']:4d}")

    # ── Strategy comparison ──
    print("\n\n" + "=" * 70)
    print("  STRATEGY COMPARISON: Graduated Realistic + 40% Enriched")
    print("=" * 70)
    for strat in ["committed", "power_chaser", "signal_reader"]:
        m, _, _, _ = run_sim(num_drafts=1000, fitness_name="Graduated_Realistic",
                              dual_res_pct=0.40, strategy_name=strat)
        print(f"  {strat:15s}: M3={m['M3']:.2f} M4={m['M4']:.2f} M5={m['M5']:.1f} "
              f"M6={m['M6']:.1%} M9={m['M9']:.2f} M10={m['M10_max']}")

    # ── Parameter sensitivity: multiplier ──
    print("\n\n" + "=" * 70)
    print("  PARAM SWEEP: multiplier (Grad. Realistic + 40%)")
    print("=" * 70)
    for mult in [3.0, 4.0, 5.0, 6.0, 7.0]:
        m, _, _, _ = run_sim(num_drafts=500, fitness_name="Graduated_Realistic",
                              dual_res_pct=0.40, multiplier=mult)
        print(f"  mult={mult:.0f}: M3={m['M3']:.2f} M5={m['M5']:.1f} "
              f"M9={m['M9']:.2f} M10={m['M10_max']}")

    # ── Parameter sensitivity: bias ──
    print("\n" + "=" * 70)
    print("  PARAM SWEEP: bias (Grad. Realistic + 40%)")
    print("=" * 70)
    for b in [1.0, 1.3, 1.5, 2.0, 2.5]:
        m, _, _, _ = run_sim(num_drafts=500, fitness_name="Graduated_Realistic",
                              dual_res_pct=0.40, bias=b)
        print(f"  bias={b:.1f}: M3={m['M3']:.2f} M5={m['M5']:.1f} "
              f"M9={m['M9']:.2f} M10={m['M10_max']}")

    # ── Parameter sensitivity: jitter ──
    print("\n" + "=" * 70)
    print("  PARAM SWEEP: jitter (Grad. Realistic + 40%)")
    print("=" * 70)
    for j in [0.0, 0.05, 0.10, 0.15, 0.20, 0.30]:
        m, _, _, _ = run_sim(num_drafts=500, fitness_name="Graduated_Realistic",
                              dual_res_pct=0.40, jitter=j)
        print(f"  jitter={j:.0%}: M3={m['M3']:.2f} M5={m['M5']:.1f} "
              f"M9={m['M9']:.2f} M10={m['M10_max']}")

    # ── Draft traces ──
    print("\n\n" + "=" * 70)
    print("  DRAFT TRACES (Graduated Realistic + 40% Enriched)")
    print("=" * 70)
    fm = FITNESS_MODELS["Graduated_Realistic"]
    pool = build_pool(dual_res_pct=0.40, fitness_model=fm, seed=42)
    idx = build_indexes(pool)

    run_trace(pool, idx, strategy_committed, 100,
              "Trace 1: Early Committer (committed)")
    run_trace(pool, idx, strategy_power_chaser, 200,
              "Trace 2: Power Chaser")
    run_trace(pool, idx, strategy_signal_reader, 300,
              "Trace 3: Signal Reader")

    # ── Fitness degradation curve ──
    print("\n\n" + "=" * 70)
    print("  FITNESS DEGRADATION (committed, 40% Enriched, uniform rate)")
    print("=" * 70)
    for rate in [1.0, 0.80, 0.60, 0.50, 0.40, 0.30, 0.20, 0.10, 0.05, 0.0]:
        fm_sweep = make_fitness_model(rate, rate, rate, rate)
        FITNESS_MODELS["_sweep"] = fm_sweep
        m, _, _, _ = run_sim(num_drafts=500, fitness_name="_sweep",
                              dual_res_pct=0.40)
        print(f"  F={rate:5.0%}: M3={m['M3']:.2f} M4={m['M4']:.2f} M9={m['M9']:.2f}")
    del FITNESS_MODELS["_sweep"]

    # ── Per-archetype convergence ──
    print("\n\n" + "=" * 70)
    print("  PER-ARCHETYPE CONVERGENCE (Grad. Realistic + 40% Enriched)")
    print("=" * 70)
    fm = FITNESS_MODELS["Graduated_Realistic"]
    pool = build_pool(dual_res_pct=0.40, fitness_model=fm, seed=42)
    idx = build_indexes(pool)

    arch_conv = defaultdict(list)
    for i in range(1000):
        drng = random.Random(42000 + i)
        data, drafted = simulate_draft(pool, idx, strategy_committed, drng)
        final = data[-1]["committed_arch"]
        if not final:
            continue
        for pick_i in range(4, len(data)):
            window = [sum(1 for c in data[j]["pack"] if c.is_sa(final))
                      for j in range(max(0, pick_i - 4), pick_i + 1)]
            if statistics.mean(window) >= 2.0:
                arch_conv[final].append(pick_i + 1)
                break
        else:
            arch_conv[final].append(NUM_PICKS)

    print(f"  {'Archetype':16s} {'AvgConv':>8s} {'N':>5s} {'%<8':>6s}")
    print("  " + "-" * 38)
    for a in ARCHETYPE_NAMES:
        vals = arch_conv.get(a, [])
        if vals:
            avg = statistics.mean(vals)
            pct = sum(1 for v in vals if v <= 8) / len(vals)
            print(f"  {a:16s} {avg:8.1f} {len(vals):5d} {pct:6.1%}")

    # ── Consecutive bad pack analysis ──
    print("\n\n" + "=" * 70)
    print("  CONSECUTIVE BAD PACK ANALYSIS (committed, picks 6+)")
    print("=" * 70)
    for plbl, dpct in [("V7_15%", 0.15), ("40%_Enr", 0.40)]:
        for fname in ["Graduated_Realistic", "Pessimistic", "Hostile"]:
            m, drafts, _, _ = run_sim(num_drafts=1000, fitness_name=fname,
                                       dual_res_pct=dpct)
            worst_list = []
            avg_list = []
            for data, drafted in drafts:
                final = data[-1]["committed_arch"]
                if not final:
                    continue
                run_len = 0
                worst = 0
                runs = []
                for pd in data[5:]:
                    sa = sum(1 for c in pd["pack"] if c.is_sa(final))
                    if sa < 1.5:
                        run_len += 1
                        worst = max(worst, run_len)
                    else:
                        if run_len > 0:
                            runs.append(run_len)
                        run_len = 0
                if run_len > 0:
                    runs.append(run_len)
                worst_list.append(worst)
                avg_list.append(statistics.mean(runs) if runs else 0)
            aw = statistics.mean(worst_list) if worst_list else 0
            mw = max(worst_list) if worst_list else 0
            ar = statistics.mean(avg_list) if avg_list else 0
            print(f"  {plbl:8s} {fname:22s}: avg_worst={aw:.1f} max_worst={mw} avg_run={ar:.1f}")

    print("\n\nSimulation complete.")


if __name__ == "__main__":
    main()
