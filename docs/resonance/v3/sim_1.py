#!/usr/bin/env python3
"""
Weighted Lottery with Wildcard Slot — Resonance Draft Simulation (Agent 1)
CORRECTED: All metrics measured at ARCHETYPE level, not resonance level.

Algorithm: "Each resonance starts at weight 1; each drafted symbol adds to
weights (primary +2, others +1); 3 of 4 pack slots pick a resonance
proportionally to weights; the 4th slot is always a random card."
"""

import random
from dataclasses import dataclass
from enum import Enum
from collections import Counter, defaultdict

# ===========================================================================
# Data Model
# ===========================================================================

class Resonance(Enum):
    EMBER = "Ember"
    STONE = "Stone"
    TIDE = "Tide"
    ZEPHYR = "Zephyr"

class Rarity(Enum):
    COMMON = "common"
    UNCOMMON = "uncommon"
    RARE = "rare"
    LEGENDARY = "legendary"

class Tier(Enum):
    S = 5
    A = 4
    B = 3
    C = 2
    F = 1

@dataclass
class SimCard:
    id: int
    symbols: list  # list of Resonance, ordered, 0-3 elements
    archetype: str
    archetype_fitness: dict  # archetype_name -> Tier
    rarity: Rarity
    power: float

# ===========================================================================
# Archetypes on a Circle
# ===========================================================================

ARCHETYPES = [
    ("Flash",         Resonance.ZEPHYR, Resonance.EMBER),   # 0
    ("Blink",         Resonance.EMBER,  Resonance.ZEPHYR),  # 1
    ("Storm",         Resonance.EMBER,  Resonance.STONE),   # 2
    ("Self-Discard",  Resonance.STONE,  Resonance.EMBER),   # 3
    ("Self-Mill",     Resonance.STONE,  Resonance.TIDE),    # 4
    ("Sacrifice",     Resonance.TIDE,   Resonance.STONE),   # 5
    ("Warriors",      Resonance.TIDE,   Resonance.ZEPHYR),  # 6
    ("Ramp",          Resonance.ZEPHYR, Resonance.TIDE),    # 7
]

ARCHETYPE_NAMES = [a[0] for a in ARCHETYPES]
N_ARCH = len(ARCHETYPES)

def adjacent_indices(arch_idx):
    """Return indices of archetypes adjacent on the circle."""
    return [(arch_idx - 1) % N_ARCH, (arch_idx + 1) % N_ARCH]

def compute_fitness(card_arch_idx):
    """
    Compute fitness dict for a card belonging to archetype at card_arch_idx.

    Fitness tiers:
      S-tier: card's home archetype
      A-tier: adjacent archetype sharing card's PRIMARY resonance
      B-tier: archetypes sharing card's SECONDARY resonance (adjacent or not)
      C-tier: archetypes sharing neither resonance but not maximally distant
      F-tier: distant archetypes with no shared resonance
    """
    fitness = {}
    card_pri = ARCHETYPES[card_arch_idx][1]
    card_sec = ARCHETYPES[card_arch_idx][2]
    adj = adjacent_indices(card_arch_idx)

    for i, (name, other_pri, other_sec) in enumerate(ARCHETYPES):
        if i == card_arch_idx:
            fitness[name] = Tier.S
        elif i in adj and (card_pri == other_pri or card_pri == other_sec):
            # Adjacent and shares primary resonance -> A-tier
            fitness[name] = Tier.A
        elif card_sec == other_pri or card_sec == other_sec:
            # Shares secondary resonance -> B-tier
            fitness[name] = Tier.B
        elif card_pri == other_pri or card_pri == other_sec:
            # Shares primary resonance but not adjacent -> B-tier
            fitness[name] = Tier.B
        elif i in adj:
            # Adjacent but only shares secondary -> C-tier
            fitness[name] = Tier.C
        else:
            # No shared resonance -> check distance
            # If they share any resonance at all, C-tier; otherwise F-tier
            shares = (card_sec == other_pri or card_sec == other_sec or
                      card_pri == other_pri or card_pri == other_sec)
            fitness[name] = Tier.C if shares else Tier.F

    return fitness

# ===========================================================================
# Card Pool Construction
# ===========================================================================

def generate_symbol_pattern(primary, secondary, num_symbols, rng):
    """Generate a symbol list for a card with given primary/secondary resonances."""
    if num_symbols == 1:
        return [primary] if rng.random() < 0.80 else [secondary]
    elif num_symbols == 2:
        r = rng.random()
        if r < 0.55:
            return [primary, secondary]
        elif r < 0.80:
            return [primary, primary]
        elif r < 0.95:
            return [secondary, primary]
        else:
            return [secondary, secondary]
    elif num_symbols == 3:
        r = rng.random()
        if r < 0.50:
            return [primary, primary, secondary]
        elif r < 0.75:
            return [primary, secondary, secondary]
        elif r < 0.90:
            return [primary, secondary, primary]
        else:
            return [primary, primary, primary]
    return []

def build_card_pool(symbol_dist=(0.20, 0.60, 0.20), seed=42):
    """
    Build 360 cards: 8 archetypes x ~40 + 36 generic.
    symbol_dist: (frac_1sym, frac_2sym, frac_3sym) among non-generic cards.
    """
    rng = random.Random(seed)
    cards = []
    card_id = 0

    def pick_rarity():
        r = rng.random()
        if r < 0.50: return Rarity.COMMON
        elif r < 0.80: return Rarity.UNCOMMON
        elif r < 0.95: return Rarity.RARE
        else: return Rarity.LEGENDARY

    frac_1, frac_2, frac_3 = symbol_dist
    # First 4 archetypes get 41 cards, last 4 get 40 = 324 + 36 = 360
    arch_card_counts = [41, 41, 41, 41, 40, 40, 40, 40]

    for arch_idx, (arch_name, pri, sec) in enumerate(ARCHETYPES):
        cards_per_arch = arch_card_counts[arch_idx]
        n1 = round(cards_per_arch * frac_1)
        n3 = round(cards_per_arch * frac_3)
        n2 = cards_per_arch - n1 - n3

        for i in range(cards_per_arch):
            if i < n1:
                num_sym = 1
            elif i < n1 + n2:
                num_sym = 2
            else:
                num_sym = 3

            symbols = generate_symbol_pattern(pri, sec, num_sym, rng)
            rarity = pick_rarity()
            power_base = {Rarity.COMMON: 4, Rarity.UNCOMMON: 5.5,
                          Rarity.RARE: 7, Rarity.LEGENDARY: 9}[rarity]
            power = max(1.0, min(10.0, power_base + rng.gauss(0, 1.0)))

            fitness = compute_fitness(arch_idx)
            cards.append(SimCard(
                id=card_id, symbols=symbols, archetype=arch_name,
                archetype_fitness=fitness, rarity=rarity, power=power
            ))
            card_id += 1

    # 36 generic cards (B-tier in all archetypes)
    for i in range(36):
        rarity = pick_rarity()
        power_base = {Rarity.COMMON: 4, Rarity.UNCOMMON: 5.5,
                      Rarity.RARE: 7, Rarity.LEGENDARY: 9}[rarity]
        power = max(1.0, min(10.0, power_base + rng.gauss(0, 1.0)))
        fitness = {name: Tier.B for name in ARCHETYPE_NAMES}
        cards.append(SimCard(
            id=card_id, symbols=[], archetype="Generic",
            archetype_fitness=fitness, rarity=rarity, power=power
        ))
        card_id += 1

    assert len(cards) == 360, f"Expected 360 cards, got {len(cards)}"
    return cards

# ===========================================================================
# Archetype-Level Evaluation Helpers
# ===========================================================================

def card_fitness_score(card, archetype_name):
    """Numeric score for how well a card fits an archetype."""
    return card.archetype_fitness.get(archetype_name, Tier.F).value

def card_is_sa_tier(card, archetype_name):
    """True if card is S-tier or A-tier for the given archetype."""
    return card_fitness_score(card, archetype_name) >= Tier.A.value

def card_is_cf_tier(card, archetype_name):
    """True if card is C-tier or F-tier for the given archetype."""
    return card_fitness_score(card, archetype_name) <= Tier.C.value

def count_sa_cards(pack, archetype_name):
    """Count cards in pack with S or A tier fitness for the archetype."""
    return sum(1 for c in pack if card_is_sa_tier(c, archetype_name))

def count_cf_cards(pack, archetype_name):
    """Count cards in pack with C or F tier fitness for the archetype."""
    return sum(1 for c in pack if card_is_cf_tier(c, archetype_name))

def unique_archetypes_with_sa(pack):
    """Count how many distinct archetypes have at least one S or A tier card in pack."""
    archs_represented = set()
    for card in pack:
        for arch_name in ARCHETYPE_NAMES:
            if card_is_sa_tier(card, arch_name):
                archs_represented.add(arch_name)
    return len(archs_represented)

# ===========================================================================
# Draft Algorithm: Weighted Lottery with Wildcard Slot
# ===========================================================================

def weighted_lottery_pack(pool, weights, rng, use_wildcard=True):
    """
    Build a 4-card pack using the Weighted Lottery with Wildcard Slot algorithm.

    - 3 of 4 slots: pick a resonance proportionally to weights, then pick a
      random card from the pool that has that resonance as primary symbol.
    - 1 slot (the wildcard): pick a completely random card from the pool.

    If use_wildcard=False, all 4 slots use weighted selection.
    """
    pack = []
    used_ids = set()
    n_weighted = 3 if use_wildcard else 4
    n_random = 1 if use_wildcard else 0

    # Precompute resonance-indexed pools
    res_pools = {r: [] for r in Resonance}
    for c in pool:
        if len(c.symbols) > 0:
            res_pools[c.symbols[0]].append(c)

    total_weight = sum(weights.values())
    resonances = list(Resonance)

    # Weighted slots
    for _ in range(n_weighted):
        roll = rng.random() * total_weight
        cumulative = 0
        chosen_res = resonances[-1]
        for r in resonances:
            cumulative += weights[r]
            if roll <= cumulative:
                chosen_res = r
                break

        candidates = [c for c in res_pools[chosen_res] if c.id not in used_ids]
        if not candidates:
            candidates = [c for c in pool if c.id not in used_ids]
        if candidates:
            card = rng.choice(candidates)
            pack.append(card)
            used_ids.add(card.id)

    # Wildcard slot(s): random card from entire pool
    for _ in range(n_random):
        candidates = [c for c in pool if c.id not in used_ids]
        if candidates:
            card = rng.choice(candidates)
            pack.append(card)
            used_ids.add(card.id)

    return pack

def update_weights(weights, card, primary_mult=2):
    """Update resonance weights based on drafted card's symbols."""
    if not card.symbols:
        return
    weights[card.symbols[0]] += primary_mult
    for sym in card.symbols[1:]:
        weights[sym] += 1

# ===========================================================================
# Player Strategies
# ===========================================================================

def determine_best_archetype(drafted):
    """Determine which archetype the drafted cards best fit (by total fitness)."""
    if not drafted:
        return None
    scores = {a: 0 for a in ARCHETYPE_NAMES}
    for c in drafted:
        for a in ARCHETYPE_NAMES:
            scores[a] += card_fitness_score(c, a)
    return max(scores, key=scores.get)

def strategy_archetype_committed(pack, drafted, pick_num, committed_arch, weights):
    """Commits to best archetype around pick 5-6. Before: picks best power. After: picks best fitness."""
    if committed_arch is None and pick_num >= 5 and drafted:
        committed_arch = determine_best_archetype(drafted)

    if committed_arch:
        best = max(pack, key=lambda c: (card_fitness_score(c, committed_arch), c.power))
    else:
        best = max(pack, key=lambda c: c.power)

    return best, committed_arch

def strategy_power_chaser(pack, drafted, pick_num, committed_arch, weights):
    """Always picks highest raw power. Tracks archetype for measurement only."""
    best = max(pack, key=lambda c: c.power)
    if pick_num >= 5 and drafted:
        committed_arch = determine_best_archetype(drafted)
    return best, committed_arch

def strategy_signal_reader(pack, drafted, pick_num, committed_arch, weights):
    """Evaluates which resonance is most weighted, drafts toward the open archetype."""
    if committed_arch is None and pick_num >= 7 and drafted:
        sorted_res = sorted(weights.keys(), key=lambda r: weights[r], reverse=True)
        top_res = sorted_res[0]
        second_res = sorted_res[1] if len(sorted_res) > 1 else None
        candidates = [(i, a) for i, (a, pri, sec) in enumerate(ARCHETYPES) if pri == top_res]
        if candidates:
            for idx, arch_name in candidates:
                if ARCHETYPES[idx][2] == second_res:
                    committed_arch = arch_name
                    break
            if committed_arch is None:
                committed_arch = candidates[0][1]

    if committed_arch:
        best = max(pack, key=lambda c: (card_fitness_score(c, committed_arch), c.power))
    else:
        sorted_res = sorted(weights.keys(), key=lambda r: weights[r], reverse=True)
        top_res = sorted_res[0]
        def signal_score(c):
            s = 0
            if c.symbols and c.symbols[0] == top_res:
                s += 2
            for sym in c.symbols[1:]:
                if sym == top_res:
                    s += 1
            return (s, c.power)
        best = max(pack, key=signal_score)

    return best, committed_arch

STRATEGIES = {
    "archetype_committed": strategy_archetype_committed,
    "power_chaser": strategy_power_chaser,
    "signal_reader": strategy_signal_reader,
}

# ===========================================================================
# Simulation Core
# ===========================================================================

def run_single_draft(pool, strategy_fn, starting_weight=1, primary_mult=2,
                     use_wildcard=True, rng=None, trace=False):
    """Run a single 30-pick draft. Returns archetype-level metrics."""
    if rng is None:
        rng = random.Random()

    weights = {r: starting_weight for r in Resonance}
    drafted = []
    committed_arch = None
    trace_log = []

    # ARCHETYPE-LEVEL metric accumulators
    early_unique_archs = []         # picks 1-5: unique archetypes w/ S/A card per pack
    early_sa_per_pack = []          # picks 1-5: S/A-tier cards for emerging arch per pack
    late_sa_per_pack = []           # picks 6+: S/A-tier cards for committed arch per pack
    late_cf_per_pack = []           # picks 6+: C/F-tier cards for committed arch per pack
    convergence_pick = None         # first pick where 2+ S/A-tier arch cards in pack
    convergence_streak = 0          # track consecutive packs with 2+ for "regular" detection
    regular_convergence_pick = None # first pick starting a streak of 3+ packs with 2+

    for pick_num in range(30):
        pack = weighted_lottery_pack(pool, weights, rng, use_wildcard)
        while len(pack) < 4:
            c = rng.choice(pool)
            if c.id not in {p.id for p in pack}:
                pack.append(c)

        # Determine measuring archetype
        if committed_arch:
            measure_arch = committed_arch
        elif drafted:
            measure_arch = determine_best_archetype(drafted)
        else:
            measure_arch = None

        # ARCHETYPE-LEVEL METRICS
        pack_unique_archs = unique_archetypes_with_sa(pack)

        if measure_arch:
            sa_count = count_sa_cards(pack, measure_arch)
            cf_count = count_cf_cards(pack, measure_arch)
        else:
            sa_count = 0
            cf_count = 0

        if pick_num < 5:
            early_unique_archs.append(pack_unique_archs)
            if measure_arch:
                early_sa_per_pack.append(sa_count)
        else:
            late_sa_per_pack.append(sa_count)
            late_cf_per_pack.append(cf_count)

            # Track convergence: first pick where we see 2+ S/A cards
            if convergence_pick is None and sa_count >= 2:
                convergence_pick = pick_num + 1  # 1-indexed

            # Track "regular" convergence (3 consecutive packs with 2+)
            if sa_count >= 2:
                convergence_streak += 1
                if regular_convergence_pick is None and convergence_streak >= 3:
                    regular_convergence_pick = pick_num + 1 - 2  # start of streak
            else:
                convergence_streak = 0

        # Player picks
        chosen, committed_arch = strategy_fn(
            pack, drafted, pick_num, committed_arch, weights
        )
        drafted.append(chosen)
        update_weights(weights, chosen, primary_mult)

        if trace:
            trace_log.append({
                "pick": pick_num + 1,
                "weights": {r: weights[r] for r in Resonance},
                "pack": [(c.id, c.archetype,
                          [s.value for s in c.symbols],
                          round(c.power, 1),
                          {a: c.archetype_fitness[a].name
                           for a in ARCHETYPE_NAMES if card_is_sa_tier(c, a)})
                         for c in pack],
                "chosen": (chosen.id, chosen.archetype,
                           [s.value for s in chosen.symbols],
                           round(chosen.power, 1)),
                "committed": committed_arch,
                "pack_sa": sa_count if measure_arch else "N/A",
                "pack_cf": cf_count if measure_arch else "N/A",
                "pack_unique_archs": pack_unique_archs,
                "measure_arch": measure_arch,
            })

    # Deck composition (archetype concentration)
    if committed_arch:
        deck_sa = sum(1 for c in drafted if card_is_sa_tier(c, committed_arch))
        deck_concentration = deck_sa / len(drafted)
    else:
        deck_concentration = 0

    return {
        "early_unique_archs": (sum(early_unique_archs) / len(early_unique_archs)
                               if early_unique_archs else 0),
        "early_sa_per_pack": (sum(early_sa_per_pack) / len(early_sa_per_pack)
                              if early_sa_per_pack else 0),
        "late_sa_per_pack": (sum(late_sa_per_pack) / len(late_sa_per_pack)
                             if late_sa_per_pack else 0),
        "late_cf_per_pack": (sum(late_cf_per_pack) / len(late_cf_per_pack)
                             if late_cf_per_pack else 0),
        "convergence_pick": convergence_pick if convergence_pick else 30,
        "regular_convergence_pick": regular_convergence_pick if regular_convergence_pick else 30,
        "deck_concentration": deck_concentration,
        "drafted_ids": [c.id for c in drafted],
        "committed_arch": committed_arch,
        "trace": trace_log if trace else None,
    }

def run_simulation(n_drafts=1000, strategy_name="archetype_committed",
                   starting_weight=1, primary_mult=2, use_wildcard=True,
                   symbol_dist=(0.20, 0.60, 0.20), pool_seed=42):
    """Run n_drafts and aggregate archetype-level metrics."""
    pool = build_card_pool(symbol_dist, seed=pool_seed)
    strategy_fn = STRATEGIES[strategy_name]

    metrics = defaultdict(list)

    for i in range(n_drafts):
        rng = random.Random(i * 1000 + pool_seed)
        result = run_single_draft(
            pool, strategy_fn, starting_weight, primary_mult, use_wildcard, rng
        )
        for k in ["early_unique_archs", "early_sa_per_pack", "late_sa_per_pack",
                   "late_cf_per_pack", "convergence_pick", "regular_convergence_pick",
                   "deck_concentration"]:
            metrics[k].append(result[k])
        metrics["drafted_ids"].append(result["drafted_ids"])
        metrics["committed_arch"].append(result["committed_arch"])

    # Aggregate
    agg = {}
    for k in ["early_unique_archs", "early_sa_per_pack", "late_sa_per_pack",
              "late_cf_per_pack", "convergence_pick", "regular_convergence_pick",
              "deck_concentration"]:
        vals = metrics[k]
        agg[k] = sum(vals) / len(vals)

    # Run-to-run variety: card overlap between pairs of runs
    overlap_samples = min(200, n_drafts)
    overlaps = []
    rng_sample = random.Random(999)
    id_lists = metrics["drafted_ids"]
    for _ in range(overlap_samples):
        i, j = rng_sample.sample(range(n_drafts), 2)
        set_i = set(id_lists[i])
        set_j = set(id_lists[j])
        overlap = len(set_i & set_j) / 30.0
        overlaps.append(overlap)
    agg["card_overlap"] = sum(overlaps) / len(overlaps) if overlaps else 0

    # Archetype frequency
    arch_counts = Counter(metrics["committed_arch"])
    total = sum(arch_counts.values())
    agg["arch_freq"] = {a: arch_counts.get(a, 0) / total for a in ARCHETYPE_NAMES}
    agg["arch_freq_max"] = max(agg["arch_freq"].values()) if agg["arch_freq"] else 0
    agg["arch_freq_min"] = min(agg["arch_freq"].values()) if agg["arch_freq"] else 0

    return agg

# ===========================================================================
# Draft Traces
# ===========================================================================

def run_trace(strategy_name, pool, seed=12345, starting_weight=1, primary_mult=2,
              use_wildcard=True):
    """Run a single traced draft for detailed output."""
    rng = random.Random(seed)
    strategy_fn = STRATEGIES[strategy_name]
    return run_single_draft(pool, strategy_fn, starting_weight, primary_mult,
                            use_wildcard, rng, trace=True)

def print_trace(label, result, max_picks=15):
    """Print a readable draft trace with archetype-level info."""
    print(f"\n{'='*75}")
    print(f"TRACE: {label}")
    print(f"{'='*75}")
    trace = result["trace"]
    for entry in trace[:max_picks]:
        p = entry["pick"]
        w = entry["weights"]
        total_w = sum(w.values())
        w_str = ", ".join(f"{r.value}:{w[r]}" for r in Resonance)
        pct_str = ", ".join(f"{r.value}:{w[r]/total_w*100:.0f}%" for r in Resonance)
        ma = entry.get("measure_arch", "N/A")
        print(f"\n  Pick {p:2d} | Weights: [{w_str}] = {total_w}")
        print(f"         | Pct: [{pct_str}]")
        print(f"         | Committed: {entry['committed'] or '(none)'} | "
              f"Measuring: {ma}")
        print(f"         | Pack S/A for arch: {entry['pack_sa']} | "
              f"Pack C/F: {entry['pack_cf']} | "
              f"Unique archetypes w/ S/A: {entry['pack_unique_archs']}")
        print(f"         | Pack:")
        for item in entry["pack"]:
            cid, arch, syms, pwr, sa_archs = item
            sym_str = "/".join(syms) if syms else "(generic)"
            sa_str = ",".join(sa_archs.keys()) if sa_archs else "none"
            marker = " <-- PICKED" if cid == entry["chosen"][0] else ""
            print(f"         |   [{sym_str:20s}] {arch:15s} pwr={pwr} "
                  f"S/A:{sa_str}{marker}")
    if len(trace) > max_picks:
        print(f"\n  ... (picks {max_picks+1}-30 omitted)")
    print(f"\n  Final committed archetype: {result['committed_arch']}")
    print(f"  Deck concentration (S/A-tier): {result['deck_concentration']*100:.1f}%")
    print(f"  Convergence pick: {result['convergence_pick']}")
    print(f"  Regular convergence pick: {result['regular_convergence_pick']}")

# ===========================================================================
# Main: Run everything
# ===========================================================================

def print_scorecard(agg, label=""):
    """Print the full target scorecard for an aggregated result."""
    targets = [
        ("Picks 1-5: unique archs w/ S/A per pack",
         agg["early_unique_archs"], ">= 3",
         agg["early_unique_archs"] >= 3.0),
        ("Picks 1-5: S/A for emerging arch per pack",
         agg["early_sa_per_pack"], "<= 2",
         agg["early_sa_per_pack"] <= 2.0),
        ("Picks 6+: S/A for committed arch per pack",
         agg["late_sa_per_pack"], ">= 2",
         agg["late_sa_per_pack"] >= 2.0),
        ("Picks 6+: C/F-tier cards per pack",
         agg["late_cf_per_pack"], ">= 0.5",
         agg["late_cf_per_pack"] >= 0.5),
        ("Convergence pick (regular 2+)",
         agg["regular_convergence_pick"], "5-8",
         5 <= agg["regular_convergence_pick"] <= 8),
        ("Deck concentration (S/A)",
         agg["deck_concentration"] * 100, "60-80%",
         60 <= agg["deck_concentration"] * 100 <= 80),
        ("Card overlap between runs",
         agg["card_overlap"] * 100, "< 40%",
         agg["card_overlap"] * 100 < 40),
        ("Arch freq range",
         f"{agg['arch_freq_min']*100:.1f}%-{agg['arch_freq_max']*100:.1f}%",
         "5-20%",
         agg["arch_freq_min"] >= 0.05 and agg["arch_freq_max"] <= 0.20),
    ]

    if label:
        print(f"\n  {label}")
    print(f"\n  {'Metric':<45s} {'Actual':>10s} {'Target':>10s} {'Result':>8s}")
    print(f"  {'-'*45} {'-'*10} {'-'*10} {'-'*8}")
    for name, actual, target, passed in targets:
        if isinstance(actual, float):
            actual_str = f"{actual:.2f}"
        else:
            actual_str = str(actual)
        result_str = "PASS" if passed else "FAIL"
        print(f"  {name:<45s} {actual_str:>10s} {target:>10s} {result_str:>8s}")

    passes = sum(1 for _, _, _, p in targets if p)
    print(f"\n  Score: {passes}/{len(targets)} targets passed")
    return passes

def main():
    print("=" * 75)
    print("WEIGHTED LOTTERY WITH WILDCARD SLOT — CORRECTED ARCHETYPE-LEVEL SIM")
    print("=" * 75)
    print("\nAll metrics measured at ARCHETYPE level (S/A/B/C/F fitness),")
    print("NOT at resonance level. This is the critical correction from v1.")

    # ----- Verify card pool fitness distribution -----
    print("\n" + "=" * 75)
    print("SECTION 0: CARD POOL VALIDATION")
    print("=" * 75)
    pool = build_card_pool((0.20, 0.60, 0.20), seed=42)

    # Show fitness distribution for one archetype as sanity check
    target_arch = "Warriors"
    tier_counts = Counter()
    for c in pool:
        tier_counts[c.archetype_fitness[target_arch].name] += 1
    print(f"\n  Fitness distribution relative to {target_arch}:")
    for tier_name in ["S", "A", "B", "C", "F"]:
        print(f"    {tier_name}-tier: {tier_counts[tier_name]} cards "
              f"({tier_counts[tier_name]/360*100:.1f}%)")

    # Count average S/A cards available per resonance for Warriors
    tide_cards = [c for c in pool if c.symbols and c.symbols[0] == Resonance.TIDE]
    tide_sa = sum(1 for c in tide_cards if card_is_sa_tier(c, "Warriors"))
    print(f"\n  Tide-primary cards: {len(tide_cards)}, of which {tide_sa} are "
          f"S/A for Warriors ({tide_sa/max(1,len(tide_cards))*100:.0f}%)")

    # ----- Baseline results (all 3 strategies) -----
    print("\n" + "=" * 75)
    print("SECTION 1: BASELINE RESULTS")
    print("(starting_weight=1, primary_mult=2, wildcard=True)")
    print("Symbol distribution: 20% 1-sym, 60% 2-sym, 20% 3-sym")
    print("=" * 75)

    for strat in STRATEGIES:
        agg = run_simulation(n_drafts=1000, strategy_name=strat,
                             starting_weight=1, primary_mult=2, use_wildcard=True,
                             symbol_dist=(0.20, 0.60, 0.20))
        print(f"\n--- Strategy: {strat} ---")
        print(f"  Picks 1-5 unique archs w/ S/A per pack:  {agg['early_unique_archs']:.2f}  (target: >= 3)")
        print(f"  Picks 1-5 S/A for arch per pack:         {agg['early_sa_per_pack']:.2f}  (target: <= 2)")
        print(f"  Picks 6+ S/A for arch per pack:          {agg['late_sa_per_pack']:.2f}  (target: >= 2)")
        print(f"  Picks 6+ C/F cards per pack:             {agg['late_cf_per_pack']:.2f}  (target: >= 0.5)")
        print(f"  Convergence pick (first 2+):             {agg['convergence_pick']:.1f}  (target: 5-8)")
        print(f"  Regular convergence pick (3 streak):     {agg['regular_convergence_pick']:.1f}  (target: 5-8)")
        print(f"  Deck concentration (S/A):                {agg['deck_concentration']*100:.1f}%  (target: 60-80%)")
        print(f"  Card overlap between runs:               {agg['card_overlap']*100:.1f}%  (target: < 40%)")
        print(f"  Arch freq range:                         {agg['arch_freq_min']*100:.1f}% - {agg['arch_freq_max']*100:.1f}%  (target: 5-20%)")
        freq_str = ", ".join(f"{a}:{agg['arch_freq'][a]*100:.1f}%"
                             for a in ARCHETYPE_NAMES)
        print(f"  Archetype breakdown:                     {freq_str}")

    # ----- Parameter sensitivity: starting weight -----
    print("\n" + "=" * 75)
    print("SECTION 2: PARAMETER SENSITIVITY — Starting Weight")
    print("(archetype_committed, primary_mult=2, wildcard=True)")
    print("=" * 75)

    for sw in [1, 3, 5]:
        agg = run_simulation(n_drafts=1000, strategy_name="archetype_committed",
                             starting_weight=sw, primary_mult=2, use_wildcard=True)
        print(f"\n  starting_weight={sw}:")
        print(f"    Unique archs early: {agg['early_unique_archs']:.2f}, "
              f"S/A early: {agg['early_sa_per_pack']:.2f}, "
              f"S/A late: {agg['late_sa_per_pack']:.2f}, "
              f"C/F late: {agg['late_cf_per_pack']:.2f}, "
              f"Conv: {agg['regular_convergence_pick']:.1f}, "
              f"Deck: {agg['deck_concentration']*100:.1f}%")

    # ----- Parameter sensitivity: primary multiplier -----
    print("\n" + "=" * 75)
    print("SECTION 3: PARAMETER SENSITIVITY — Primary Multiplier")
    print("(archetype_committed, starting_weight=1, wildcard=True)")
    print("=" * 75)

    for pm in [1, 2, 3]:
        agg = run_simulation(n_drafts=1000, strategy_name="archetype_committed",
                             starting_weight=1, primary_mult=pm, use_wildcard=True)
        print(f"\n  primary_mult={pm}:")
        print(f"    Unique archs early: {agg['early_unique_archs']:.2f}, "
              f"S/A early: {agg['early_sa_per_pack']:.2f}, "
              f"S/A late: {agg['late_sa_per_pack']:.2f}, "
              f"C/F late: {agg['late_cf_per_pack']:.2f}, "
              f"Conv: {agg['regular_convergence_pick']:.1f}, "
              f"Deck: {agg['deck_concentration']*100:.1f}%")

    # ----- Parameter sensitivity: with/without wildcard -----
    print("\n" + "=" * 75)
    print("SECTION 4: PARAMETER SENSITIVITY — Wildcard Slot")
    print("(archetype_committed, starting_weight=1, primary_mult=2)")
    print("=" * 75)

    for wc in [True, False]:
        agg = run_simulation(n_drafts=1000, strategy_name="archetype_committed",
                             starting_weight=1, primary_mult=2, use_wildcard=wc)
        print(f"\n  wildcard={wc}:")
        print(f"    Unique archs early: {agg['early_unique_archs']:.2f}, "
              f"S/A early: {agg['early_sa_per_pack']:.2f}, "
              f"S/A late: {agg['late_sa_per_pack']:.2f}, "
              f"C/F late: {agg['late_cf_per_pack']:.2f}, "
              f"Conv: {agg['regular_convergence_pick']:.1f}, "
              f"Deck: {agg['deck_concentration']*100:.1f}%")

    # ----- Draft Traces -----
    print("\n" + "=" * 75)
    print("SECTION 5: DRAFT TRACES")
    print("=" * 75)

    pool = build_card_pool((0.20, 0.60, 0.20), seed=42)

    result1 = run_trace("archetype_committed", pool, seed=77)
    print_trace("Early Committer (archetype_committed, commits ~pick 5)", result1, max_picks=12)

    result2 = run_trace("power_chaser", pool, seed=88)
    print_trace("Flexible Player (power_chaser, stays flexible 8+ picks)", result2, max_picks=12)

    result3 = run_trace("signal_reader", pool, seed=99)
    print_trace("Signal Reader (signal_reader, commits ~pick 7)", result3, max_picks=12)

    # ----- Summary scorecard -----
    print("\n" + "=" * 75)
    print("SECTION 6: TARGET SCORECARD (archetype_committed, baseline params)")
    print("=" * 75)

    agg = run_simulation(n_drafts=1000, strategy_name="archetype_committed",
                         starting_weight=1, primary_mult=2, use_wildcard=True)
    print_scorecard(agg, "Archetype-Committed Strategy (baseline)")

    print("\n" + "=" * 75)
    print("SECTION 7: TARGET SCORECARD (signal_reader)")
    print("=" * 75)

    agg_signal = run_simulation(n_drafts=1000, strategy_name="signal_reader",
                                starting_weight=1, primary_mult=2, use_wildcard=True)
    print_scorecard(agg_signal, "Signal Reader Strategy (baseline)")

if __name__ == "__main__":
    main()
