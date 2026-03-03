#!/usr/bin/env python3
"""
Resonance V4 — Agent 1 Simulation: Exile Pressure

Algorithm (one-sentence):
"When you pass a card, add 2 to its primary resonance's exile counter and 1
per secondary/tertiary symbol; all counters decay by 1 each pick; each pack
card is independently skipped with probability (its primary resonance's counter
/ 20), rerolling on a skip."

Also implements V3 Lane Locking baseline for comparison.
"""

import random
import math
from dataclasses import dataclass, field
from enum import Enum
from collections import defaultdict
from typing import Optional

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

NUM_CARDS = 360
NUM_GENERIC = 36
NUM_ARCHETYPE_CARDS = NUM_CARDS - NUM_GENERIC  # 324
PACK_SIZE = 4
NUM_PICKS = 30
NUM_DRAFTS = 1000
EXILE_DIVISOR = 20
EXILE_CAP = 20
EXILE_DECAY = 1
MAX_REROLLS = 20  # safety valve for rejection sampling

RESONANCES = ["Ember", "Stone", "Tide", "Zephyr"]

# Archetypes on the circle, each with (primary_res, secondary_res)
ARCHETYPES = [
    ("Flash/Tempo/Prison",   "Zephyr", "Ember"),
    ("Blink/Flicker",        "Ember",  "Zephyr"),
    ("Storm/Spellslinger",   "Ember",  "Stone"),
    ("Self-Discard",         "Stone",  "Ember"),
    ("Self-Mill/Reanimator", "Stone",  "Tide"),
    ("Sacrifice/Abandon",    "Tide",   "Stone"),
    ("Warriors/Midrange",    "Tide",   "Zephyr"),
    ("Ramp/Spirit Animals",  "Zephyr", "Tide"),
]

ARCHETYPE_NAMES = [a[0] for a in ARCHETYPES]

# Build adjacency: archetypes at positions i and i+1 (mod 8) are adjacent
def _build_adjacency():
    adj = defaultdict(set)
    for i in range(8):
        j = (i + 1) % 8
        adj[ARCHETYPE_NAMES[i]].add(ARCHETYPE_NAMES[j])
        adj[ARCHETYPE_NAMES[j]].add(ARCHETYPE_NAMES[i])
    return dict(adj)

ADJACENCY = _build_adjacency()

# Map resonance -> archetypes using it as primary
RES_TO_PRIMARY_ARCHETYPES = defaultdict(list)
RES_TO_SECONDARY_ARCHETYPES = defaultdict(list)
for name, pri, sec in ARCHETYPES:
    RES_TO_PRIMARY_ARCHETYPES[pri].append(name)
    RES_TO_SECONDARY_ARCHETYPES[sec].append(name)

# ---------------------------------------------------------------------------
# Data model
# ---------------------------------------------------------------------------

@dataclass
class SimCard:
    id: int
    symbols: list  # ordered list of resonance strings, 0-3 elements
    archetype: str  # primary archetype
    archetype_fitness: dict = field(default_factory=dict)  # archetype -> tier
    power: float = 5.0

    @property
    def primary_resonance(self):
        return self.symbols[0] if self.symbols else None


# ---------------------------------------------------------------------------
# Card pool construction
# ---------------------------------------------------------------------------

def build_card_pool(symbol_dist=(0.30, 0.50, 0.20)):
    """
    Build 360 cards.  symbol_dist = (frac_1sym, frac_2sym, frac_3sym) among
    non-generic cards.
    """
    cards = []
    card_id = 0

    # Generic cards (36)
    for _ in range(NUM_GENERIC):
        c = SimCard(id=card_id, symbols=[], archetype="Generic",
                    power=round(random.uniform(3, 8), 1))
        # Generic: B-tier in all archetypes
        c.archetype_fitness = {a: "B" for a in ARCHETYPE_NAMES}
        cards.append(c)
        card_id += 1

    # Per-archetype cards: ~40.5 each (324 / 8 = 40.5, use 40 + distribute remainder)
    per_arch = NUM_ARCHETYPE_CARDS // 8  # 40
    remainder = NUM_ARCHETYPE_CARDS % 8  # 4
    arch_counts = [per_arch] * 8
    for i in range(remainder):
        arch_counts[i] += 1

    frac_1, frac_2, frac_3 = symbol_dist

    for arch_idx, (arch_name, pri_res, sec_res) in enumerate(ARCHETYPES):
        count = arch_counts[arch_idx]
        n1 = round(count * frac_1)
        n3 = round(count * frac_3)
        n2 = count - n1 - n3

        for i in range(count):
            if i < n1:
                symbols = [pri_res]
            elif i < n1 + n2:
                # 2-symbol: mostly [pri, sec], sometimes [pri, pri]
                if random.random() < 0.75:
                    symbols = [pri_res, sec_res]
                else:
                    symbols = [pri_res, pri_res]
            else:
                # 3-symbol: [pri, sec, pri] or [pri, pri, sec]
                if random.random() < 0.5:
                    symbols = [pri_res, sec_res, pri_res]
                else:
                    symbols = [pri_res, pri_res, sec_res]

            c = SimCard(id=card_id, symbols=symbols, archetype=arch_name,
                        power=round(random.uniform(2, 9), 1))

            # Fitness assignment
            fitness = {}
            for other_idx, (other_name, other_pri, other_sec) in enumerate(ARCHETYPES):
                if other_name == arch_name:
                    fitness[other_name] = "S"
                elif other_name in ADJACENCY.get(arch_name, set()):
                    # Adjacent archetype — check if they share primary resonance
                    if other_pri == pri_res or other_sec == pri_res:
                        fitness[other_name] = "A"
                    else:
                        fitness[other_name] = "B"
                else:
                    # Check for secondary resonance overlap
                    if other_pri == sec_res or other_sec == sec_res:
                        fitness[other_name] = "B"
                    else:
                        # Distance check: further away = C or F
                        dist = min(abs(arch_idx - other_idx), 8 - abs(arch_idx - other_idx))
                        if dist <= 2:
                            fitness[other_name] = "C"
                        else:
                            fitness[other_name] = "F"
                fitness[other_name] = fitness.get(other_name, "F")
            c.archetype_fitness = fitness
            cards.append(c)
            card_id += 1

    random.shuffle(cards)
    return cards


def is_sa_for(card, archetype):
    """Return True if card is S or A tier for the given archetype."""
    tier = card.archetype_fitness.get(archetype, "F")
    return tier in ("S", "A")


# ---------------------------------------------------------------------------
# Exile Pressure draft algorithm
# ---------------------------------------------------------------------------

def exile_pressure_draft(pool, strategy, target_archetype=None, trace=False):
    """
    Run one 30-pick draft using the Exile Pressure algorithm.

    strategy: "committed", "power", "signal"
    target_archetype: for committed strategy, the target archetype name
    """
    exile_counters = {r: 0.0 for r in RESONANCES}
    drafted = []
    pack_log = []  # for traces

    # For signal reader: track resonance counts seen
    res_seen = {r: 0 for r in RESONANCES}

    # Copy pool IDs for tracking
    available_ids = set(c.id for c in pool)
    pool_by_id = {c.id: c for c in pool}

    for pick_num in range(1, NUM_PICKS + 1):
        # Generate pack of 4 using rejection sampling
        pack = _generate_exile_pack(pool, exile_counters)

        # Track resonances seen (for signal reader)
        for c in pack:
            if c.primary_resonance:
                res_seen[c.primary_resonance] += 1

        # Choose a card based on strategy
        if strategy == "committed":
            chosen = _pick_committed(pack, target_archetype, drafted)
        elif strategy == "power":
            chosen = _pick_power(pack)
        elif strategy == "signal":
            chosen, target_archetype = _pick_signal(
                pack, drafted, res_seen, target_archetype, pick_num)
        else:
            chosen = random.choice(pack)

        passed = [c for c in pack if c.id != chosen.id]
        drafted.append(chosen)

        if trace:
            sa_count = sum(1 for c in pack if is_sa_for(c, target_archetype)) if target_archetype else 0
            pack_log.append({
                "pick": pick_num,
                "pack": [(c.archetype, c.symbols, c.archetype_fitness.get(target_archetype, "?")) for c in pack],
                "chosen": (chosen.archetype, chosen.symbols),
                "sa_in_pack": sa_count,
                "exile": dict(exile_counters),
            })

        # Update exile counters from passed cards
        for c in passed:
            if c.symbols:
                exile_counters[c.symbols[0]] = min(
                    EXILE_CAP, exile_counters[c.symbols[0]] + 2)
                for sym in c.symbols[1:]:
                    exile_counters[sym] = min(
                        EXILE_CAP, exile_counters[sym] + 1)

        # Decay all counters
        for r in RESONANCES:
            exile_counters[r] = max(0, exile_counters[r] - EXILE_DECAY)

    return drafted, target_archetype, pack_log


def _generate_exile_pack(pool, exile_counters):
    """Generate a 4-card pack using rejection sampling with exile counters."""
    pack = []
    used_ids = set()
    for _ in range(PACK_SIZE):
        for _attempt in range(MAX_REROLLS):
            card = random.choice(pool)
            if card.id in used_ids:
                continue
            pri = card.primary_resonance
            if pri and exile_counters.get(pri, 0) > 0:
                skip_prob = min(exile_counters[pri] / EXILE_DIVISOR, 1.0)
                if random.random() < skip_prob:
                    continue  # rejected, reroll
            used_ids.add(card.id)
            pack.append(card)
            break
        else:
            # Fallback: just pick any unused card
            candidates = [c for c in pool if c.id not in used_ids]
            if candidates:
                card = random.choice(candidates)
                used_ids.add(card.id)
                pack.append(card)
    return pack


# ---------------------------------------------------------------------------
# V3 Lane Locking baseline
# ---------------------------------------------------------------------------

def lane_locking_draft(pool, strategy, target_archetype=None, trace=False):
    """
    V3 Lane Locking: threshold 3 locks 1 slot, threshold 8 locks a 2nd slot.
    Locked slots always show a card of that resonance (primary=2 weighted counting).
    """
    res_counts = {r: 0 for r in RESONANCES}
    drafted = []
    pack_log = []
    res_seen = {r: 0 for r in RESONANCES}

    for pick_num in range(1, NUM_PICKS + 1):
        # Determine locked resonances
        locked = []
        sorted_res = sorted(RESONANCES, key=lambda r: res_counts[r], reverse=True)
        for r in sorted_res:
            if res_counts[r] >= 3 and len(locked) < 1:
                locked.append(r)
            elif res_counts[r] >= 8 and len(locked) < 2:
                locked.append(r)

        # Generate pack: locked slots + random slots
        pack = []
        used_ids = set()
        for r in locked:
            candidates = [c for c in pool if c.primary_resonance == r and c.id not in used_ids]
            if candidates:
                card = random.choice(candidates)
                used_ids.add(card.id)
                pack.append(card)

        # Fill remaining slots randomly
        remaining = PACK_SIZE - len(pack)
        for _ in range(remaining):
            for _attempt in range(50):
                card = random.choice(pool)
                if card.id not in used_ids:
                    used_ids.add(card.id)
                    pack.append(card)
                    break

        for c in pack:
            if c.primary_resonance:
                res_seen[c.primary_resonance] += 1

        # Choose card
        if strategy == "committed":
            chosen = _pick_committed(pack, target_archetype, drafted)
        elif strategy == "power":
            chosen = _pick_power(pack)
        elif strategy == "signal":
            chosen, target_archetype = _pick_signal(
                pack, drafted, res_seen, target_archetype, pick_num)
        else:
            chosen = random.choice(pack)

        drafted.append(chosen)

        if trace:
            sa_count = sum(1 for c in pack if is_sa_for(c, target_archetype)) if target_archetype else 0
            pack_log.append({
                "pick": pick_num,
                "pack": [(c.archetype, c.symbols, c.archetype_fitness.get(target_archetype, "?")) for c in pack],
                "chosen": (chosen.archetype, chosen.symbols),
                "sa_in_pack": sa_count,
                "exile": {},
                "locked": list(locked),
            })

        # Update resonance counts (primary=2, secondary/tertiary=1)
        if chosen.symbols:
            res_counts[chosen.symbols[0]] += 2
            for sym in chosen.symbols[1:]:
                res_counts[sym] += 1

    return drafted, target_archetype, pack_log


# ---------------------------------------------------------------------------
# Player strategies
# ---------------------------------------------------------------------------

def _pick_committed(pack, target_archetype, drafted):
    """Pick the card with best fitness for target archetype."""
    tier_order = {"S": 0, "A": 1, "B": 2, "C": 3, "F": 4}
    best = min(pack, key=lambda c: (
        tier_order.get(c.archetype_fitness.get(target_archetype, "F"), 4),
        -c.power))
    return best


def _pick_power(pack):
    """Pick highest raw power card."""
    return max(pack, key=lambda c: c.power)


def _pick_signal(pack, drafted, res_seen, current_target, pick_num):
    """
    Signal reader: for first 5 picks, pick best card from most-seen resonance.
    After pick 5, commit to the archetype with most S/A cards drafted.
    """
    if pick_num <= 5:
        # Pick from most-seen resonance
        best_res = max(RESONANCES, key=lambda r: res_seen[r])
        # Find archetypes with this as primary
        candidate_archs = RES_TO_PRIMARY_ARCHETYPES.get(best_res, [])
        if candidate_archs:
            # Pick card with best fitness for any of these archetypes
            best_card = None
            best_score = 999
            for c in pack:
                for arch in candidate_archs:
                    tier_order = {"S": 0, "A": 1, "B": 2, "C": 3, "F": 4}
                    score = tier_order.get(c.archetype_fitness.get(arch, "F"), 4)
                    if score < best_score or (score == best_score and c.power > (best_card.power if best_card else 0)):
                        best_score = score
                        best_card = c
                        current_target = arch
            return best_card, current_target
        return max(pack, key=lambda c: c.power), current_target
    else:
        # Commit: find best archetype from drafted cards
        if current_target is None:
            arch_sa_counts = defaultdict(int)
            for c in drafted:
                for arch in ARCHETYPE_NAMES:
                    if is_sa_for(c, arch):
                        arch_sa_counts[arch] += 1
            if arch_sa_counts:
                current_target = max(arch_sa_counts, key=arch_sa_counts.get)
            else:
                current_target = random.choice(ARCHETYPE_NAMES)
        return _pick_committed(pack, current_target, drafted), current_target


# ---------------------------------------------------------------------------
# Metrics computation
# ---------------------------------------------------------------------------

def compute_metrics(draft_func, pool, num_drafts=NUM_DRAFTS):
    """Run many drafts and compute all 8+ metrics at archetype level."""

    all_results = {
        "committed": defaultdict(list),
        "power": defaultdict(list),
        "signal": defaultdict(list),
    }

    # For per-archetype convergence
    per_arch_convergence = {a: [] for a in ARCHETYPE_NAMES}

    for draft_i in range(num_drafts):
        for strategy in ["committed", "power", "signal"]:
            if strategy == "committed":
                target = random.choice(ARCHETYPE_NAMES)
            else:
                target = None

            drafted, final_target, _ = draft_func(pool, strategy, target)

            if final_target is None:
                final_target = random.choice(ARCHETYPE_NAMES)

            # Measure pack metrics by re-running with pack logging
            # Instead, let's compute from the draft itself
            # We need to re-run to get pack data
            # For efficiency, embed metric collection in a wrapper
            pass

    # More efficient approach: run drafts with metric collection
    metrics = {
        "early_unique_archetypes": [],
        "early_sa_for_emerging": [],
        "late_sa_for_committed": [],
        "late_off_archetype": [],
        "convergence_pick": [],
        "deck_concentration": [],
        "sa_per_pack_late_all": [],  # for variance
    }

    archetype_freq = defaultdict(int)
    run_cards = []  # for run-to-run variety

    for draft_i in range(num_drafts):
        target = random.choice(ARCHETYPE_NAMES)
        archetype_freq[target] += 1

        # Run draft with pack tracking
        exile_counters = {r: 0.0 for r in RESONANCES}
        drafted = []
        pack_sa_counts = []  # sa count per pack for target archetype
        pack_unique_archs = []  # unique archetypes with S/A cards per pack
        pack_off_arch = []  # off-archetype cards per pack

        res_seen = {r: 0 for r in RESONANCES}

        for pick_num in range(1, NUM_PICKS + 1):
            if draft_func == exile_pressure_draft:
                pack = _generate_exile_pack(pool, exile_counters)
            else:
                # Lane locking
                res_counts = {r: 0 for r in RESONANCES}
                for c in drafted:
                    if c.symbols:
                        res_counts[c.symbols[0]] += 2
                        for sym in c.symbols[1:]:
                            res_counts[sym] += 1
                locked = []
                sorted_res = sorted(RESONANCES, key=lambda r: res_counts[r], reverse=True)
                for r in sorted_res:
                    if res_counts[r] >= 3 and len(locked) < 1:
                        locked.append(r)
                    elif res_counts[r] >= 8 and len(locked) < 2:
                        locked.append(r)
                pack = []
                used_ids = set()
                for r in locked:
                    candidates = [c for c in pool if c.primary_resonance == r and c.id not in used_ids]
                    if candidates:
                        card = random.choice(candidates)
                        used_ids.add(card.id)
                        pack.append(card)
                remaining = PACK_SIZE - len(pack)
                for _ in range(remaining):
                    for _attempt in range(50):
                        card = random.choice(pool)
                        if card.id not in used_ids:
                            used_ids.add(card.id)
                            pack.append(card)
                            break

            # Measure pack quality
            sa_count = sum(1 for c in pack if is_sa_for(c, target))
            unique_archs_with_sa = len(set(
                arch for c in pack for arch in ARCHETYPE_NAMES
                if is_sa_for(c, arch)
            ))
            off_arch = sum(1 for c in pack if not is_sa_for(c, target))

            pack_sa_counts.append(sa_count)
            pack_unique_archs.append(unique_archs_with_sa)
            pack_off_arch.append(off_arch)

            # Pick (committed strategy for metrics)
            chosen = _pick_committed(pack, target, drafted)
            passed = [c for c in pack if c.id != chosen.id]
            drafted.append(chosen)

            # Update exile counters
            if draft_func == exile_pressure_draft:
                for c in passed:
                    if c.symbols:
                        exile_counters[c.symbols[0]] = min(
                            EXILE_CAP, exile_counters[c.symbols[0]] + 2)
                        for sym in c.symbols[1:]:
                            exile_counters[sym] = min(
                                EXILE_CAP, exile_counters[sym] + 1)
                for r in RESONANCES:
                    exile_counters[r] = max(0, exile_counters[r] - EXILE_DECAY)

        # Compute per-draft metrics
        # Early (picks 1-5)
        early_unique = sum(pack_unique_archs[:5]) / 5.0
        early_sa = sum(pack_sa_counts[:5]) / 5.0
        metrics["early_unique_archetypes"].append(early_unique)
        metrics["early_sa_for_emerging"].append(early_sa)

        # Late (picks 6+)
        late_sa_values = pack_sa_counts[5:]
        late_sa_avg = sum(late_sa_values) / len(late_sa_values) if late_sa_values else 0
        late_off_values = pack_off_arch[5:]
        late_off_avg = sum(late_off_values) / len(late_off_values) if late_off_values else 0
        metrics["late_sa_for_committed"].append(late_sa_avg)
        metrics["late_off_archetype"].append(late_off_avg)
        metrics["sa_per_pack_late_all"].extend(late_sa_values)

        # Convergence pick: first pick where rolling avg of 3 picks has >= 2.0 S/A
        convergence = NUM_PICKS  # default if never converges
        for p in range(2, NUM_PICKS):
            window = pack_sa_counts[max(0, p - 2):p + 1]
            if len(window) >= 3 and sum(window) / len(window) >= 2.0:
                convergence = p + 1  # 1-indexed
                break
        metrics["convergence_pick"].append(convergence)

        # Per-archetype convergence
        per_arch_convergence[target].append(convergence)

        # Deck concentration
        sa_in_deck = sum(1 for c in drafted if is_sa_for(c, target))
        metrics["deck_concentration"].append(sa_in_deck / NUM_PICKS)

        # Run-to-run variety: store drafted card IDs
        run_cards.append(set(c.id for c in drafted))

    # Compute aggregate metrics
    result = {}
    result["early_unique_archetypes"] = sum(metrics["early_unique_archetypes"]) / len(metrics["early_unique_archetypes"])
    result["early_sa_for_emerging"] = sum(metrics["early_sa_for_emerging"]) / len(metrics["early_sa_for_emerging"])
    result["late_sa_for_committed"] = sum(metrics["late_sa_for_committed"]) / len(metrics["late_sa_for_committed"])
    result["late_off_archetype"] = sum(metrics["late_off_archetype"]) / len(metrics["late_off_archetype"])
    result["convergence_pick"] = sum(metrics["convergence_pick"]) / len(metrics["convergence_pick"])
    result["deck_concentration"] = sum(metrics["deck_concentration"]) / len(metrics["deck_concentration"])

    # Variance of S/A per pack (late)
    sa_late = metrics["sa_per_pack_late_all"]
    mean_sa = sum(sa_late) / len(sa_late)
    variance = sum((x - mean_sa) ** 2 for x in sa_late) / len(sa_late)
    result["sa_stddev"] = math.sqrt(variance)

    # Run-to-run variety: average pairwise overlap
    overlaps = []
    sample_pairs = min(500, len(run_cards) * (len(run_cards) - 1) // 2)
    for _ in range(sample_pairs):
        i, j = random.sample(range(len(run_cards)), 2)
        intersection = len(run_cards[i] & run_cards[j])
        union = len(run_cards[i] | run_cards[j])
        if union > 0:
            overlaps.append(intersection / len(run_cards[i]))
    result["run_overlap"] = sum(overlaps) / len(overlaps) if overlaps else 0

    # Archetype frequency
    total = sum(archetype_freq.values())
    result["archetype_freq"] = {a: archetype_freq[a] / total for a in ARCHETYPE_NAMES}

    # Per-archetype convergence table
    result["per_arch_convergence"] = {}
    for arch in ARCHETYPE_NAMES:
        vals = per_arch_convergence[arch]
        if vals:
            result["per_arch_convergence"][arch] = sum(vals) / len(vals)
        else:
            result["per_arch_convergence"][arch] = float("nan")

    # S/A distribution for picks 6+
    from collections import Counter
    sa_dist = Counter(sa_late)
    total_packs = len(sa_late)
    result["sa_distribution"] = {k: sa_dist[k] / total_packs for k in sorted(sa_dist.keys())}

    return result


# ---------------------------------------------------------------------------
# Parameter sensitivity sweep
# ---------------------------------------------------------------------------

def parameter_sweep(pool):
    """Sweep key parameters and report convergence + variance."""
    print("\n=== Parameter Sensitivity Sweep ===\n")

    # 1. Divisor sweep
    print("--- Divisor Sweep (exile_divisor) ---")
    global EXILE_DIVISOR
    for divisor in [15, 20, 25, 30]:
        EXILE_DIVISOR = divisor
        r = compute_metrics(exile_pressure_draft, pool, num_drafts=300)
        print(f"  Divisor={divisor}: late_SA={r['late_sa_for_committed']:.2f}, "
              f"stddev={r['sa_stddev']:.2f}, conv_pick={r['convergence_pick']:.1f}")
    EXILE_DIVISOR = 20  # reset

    # 2. Decay sweep
    print("\n--- Decay Sweep (exile_decay) ---")
    global EXILE_DECAY
    for decay in [0.5, 1.0, 1.5, 2.0]:
        EXILE_DECAY = decay
        r = compute_metrics(exile_pressure_draft, pool, num_drafts=300)
        print(f"  Decay={decay}: late_SA={r['late_sa_for_committed']:.2f}, "
              f"stddev={r['sa_stddev']:.2f}, conv_pick={r['convergence_pick']:.1f}")
    EXILE_DECAY = 1  # reset

    # 3. Cap sweep
    print("\n--- Cap Sweep (exile_cap) ---")
    global EXILE_CAP
    for cap in [10, 15, 20, 30]:
        EXILE_CAP = cap
        r = compute_metrics(exile_pressure_draft, pool, num_drafts=300)
        print(f"  Cap={cap}: late_SA={r['late_sa_for_committed']:.2f}, "
              f"stddev={r['sa_stddev']:.2f}, conv_pick={r['convergence_pick']:.1f}")
    EXILE_CAP = 20  # reset


def symbol_distribution_sweep():
    """Sweep symbol distribution and report metrics."""
    print("\n=== Symbol Distribution Sweep ===\n")
    distributions = [
        ("Mostly 1-sym", (0.60, 0.30, 0.10)),
        ("Balanced",     (0.30, 0.50, 0.20)),
        ("Mostly 2-sym", (0.15, 0.65, 0.20)),
        ("Mostly 3-sym", (0.15, 0.30, 0.55)),
    ]
    for name, dist in distributions:
        pool = build_card_pool(dist)
        r = compute_metrics(exile_pressure_draft, pool, num_drafts=300)
        print(f"  {name} {dist}: late_SA={r['late_sa_for_committed']:.2f}, "
              f"stddev={r['sa_stddev']:.2f}, conv_pick={r['convergence_pick']:.1f}, "
              f"deck_conc={r['deck_concentration']:.2f}")


# ---------------------------------------------------------------------------
# Draft traces
# ---------------------------------------------------------------------------

def run_traces(pool):
    """Run 3 detailed draft traces."""
    print("\n=== Draft Trace 1: Early Committer (Warriors, committed by pick 3) ===\n")
    _, _, log1 = exile_pressure_draft(pool, "committed", "Warriors/Midrange", trace=True)
    _print_trace(log1, "Warriors/Midrange", picks_to_show=15)

    print("\n=== Draft Trace 2: Flexible Player (power-chaser for 10 picks) ===\n")
    # Simulate a player who power-picks first 10, then commits
    exile_counters = {r: 0.0 for r in RESONANCES}
    drafted = []
    log2 = []
    chosen_target = None
    for pick_num in range(1, NUM_PICKS + 1):
        pack = _generate_exile_pack(pool, exile_counters)
        if pick_num <= 10:
            chosen = _pick_power(pack)
        else:
            if chosen_target is None:
                # Find best archetype from drafted
                arch_sa = defaultdict(int)
                for c in drafted:
                    for a in ARCHETYPE_NAMES:
                        if is_sa_for(c, a):
                            arch_sa[a] += 1
                chosen_target = max(arch_sa, key=arch_sa.get) if arch_sa else "Warriors/Midrange"
            chosen = _pick_committed(pack, chosen_target, drafted)

        passed = [c for c in pack if c.id != chosen.id]
        drafted.append(chosen)
        sa_count = sum(1 for c in pack if is_sa_for(c, chosen_target)) if chosen_target else 0
        log2.append({
            "pick": pick_num,
            "pack": [(c.archetype, c.symbols, c.archetype_fitness.get(chosen_target or "Warriors/Midrange", "?")) for c in pack],
            "chosen": (chosen.archetype, chosen.symbols),
            "sa_in_pack": sa_count,
            "exile": dict(exile_counters),
        })
        for c in passed:
            if c.symbols:
                exile_counters[c.symbols[0]] = min(EXILE_CAP, exile_counters[c.symbols[0]] + 2)
                for sym in c.symbols[1:]:
                    exile_counters[sym] = min(EXILE_CAP, exile_counters[sym] + 1)
        for r in RESONANCES:
            exile_counters[r] = max(0, exile_counters[r] - EXILE_DECAY)

    print(f"  (Committed to {chosen_target} at pick 11)")
    _print_trace(log2, chosen_target or "Warriors/Midrange", picks_to_show=15)

    print("\n=== Draft Trace 3: Signal Reader ===\n")
    _, final_target, log3 = exile_pressure_draft(pool, "signal", None, trace=True)
    print(f"  (Final archetype: {final_target})")
    _print_trace(log3, final_target, picks_to_show=15)


def _print_trace(log, target, picks_to_show=15):
    for entry in log[:picks_to_show]:
        p = entry["pick"]
        pack_desc = []
        for arch, syms, tier in entry["pack"]:
            sym_str = "/".join(syms) if syms else "none"
            pack_desc.append(f"{arch}({sym_str})[{tier}]")
        ch_arch, ch_syms = entry["chosen"]
        ch_str = f"{ch_arch}({'/'.join(ch_syms) if ch_syms else 'none'})"
        exile_str = ", ".join(f"{r}:{v:.0f}" for r, v in entry["exile"].items() if v > 0)
        print(f"  Pick {p:2d}: Pack=[{', '.join(pack_desc)}]")
        print(f"          Chose={ch_str} | SA={entry['sa_in_pack']} | Exile=[{exile_str}]")


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    random.seed(42)

    print("Building card pool (30/50/20 symbol distribution)...")
    pool = build_card_pool((0.30, 0.50, 0.20))
    print(f"  Pool size: {len(pool)} cards")
    print(f"  Generic: {sum(1 for c in pool if not c.symbols)}")
    print(f"  1-sym: {sum(1 for c in pool if len(c.symbols) == 1)}")
    print(f"  2-sym: {sum(1 for c in pool if len(c.symbols) == 2)}")
    print(f"  3-sym: {sum(1 for c in pool if len(c.symbols) == 3)}")

    # Verify SA distribution for a sample archetype
    test_arch = "Warriors/Midrange"
    sa_cards = sum(1 for c in pool if is_sa_for(c, test_arch))
    print(f"  S/A cards for {test_arch}: {sa_cards} / {len(pool)} ({sa_cards/len(pool)*100:.1f}%)")

    print("\n" + "=" * 60)
    print("EXILE PRESSURE — 1000 drafts (committed players)")
    print("=" * 60)
    exile_results = compute_metrics(exile_pressure_draft, pool, NUM_DRAFTS)
    _print_results("Exile Pressure", exile_results)

    print("\n" + "=" * 60)
    print("V3 LANE LOCKING BASELINE — 1000 drafts (committed players)")
    print("=" * 60)
    lane_results = compute_metrics(lane_locking_draft, pool, NUM_DRAFTS)
    _print_results("Lane Locking", lane_results)

    print("\n" + "=" * 60)
    print("SIDE-BY-SIDE COMPARISON")
    print("=" * 60)
    _print_comparison(exile_results, lane_results)

    # Parameter sweeps
    parameter_sweep(pool)
    symbol_distribution_sweep()

    # Draft traces
    run_traces(pool)

    print("\n" + "=" * 60)
    print("SIMULATION COMPLETE")
    print("=" * 60)


def _print_results(name, r):
    print(f"\n--- {name} ---")
    print(f"  Picks 1-5 unique archetypes w/ S/A per pack: {r['early_unique_archetypes']:.2f} (target >= 3)")
    print(f"  Picks 1-5 S/A for emerging archetype per pack: {r['early_sa_for_emerging']:.2f} (target <= 2)")
    print(f"  Picks 6+ S/A for committed archetype per pack: {r['late_sa_for_committed']:.2f} (target >= 2)")
    print(f"  Picks 6+ off-archetype cards per pack: {r['late_off_archetype']:.2f} (target >= 0.5)")
    print(f"  Convergence pick: {r['convergence_pick']:.1f} (target 5-8)")
    print(f"  Deck concentration: {r['deck_concentration']:.2f} (target 0.60-0.90)")
    print(f"  S/A stddev (picks 6+): {r['sa_stddev']:.2f} (target >= 0.8)")
    print(f"  Run-to-run card overlap: {r['run_overlap']:.2f} (target < 0.40)")

    print(f"\n  S/A distribution per pack (picks 6+):")
    for k, v in sorted(r['sa_distribution'].items()):
        print(f"    {k} S/A cards: {v*100:.1f}%")

    print(f"\n  Per-archetype convergence:")
    for arch, pick in r['per_arch_convergence'].items():
        print(f"    {arch:30s}: pick {pick:.1f}")

    print(f"\n  Archetype frequency:")
    for arch, freq in sorted(r['archetype_freq'].items(), key=lambda x: -x[1]):
        flag = " !!!" if freq > 0.20 or freq < 0.05 else ""
        print(f"    {arch:30s}: {freq*100:.1f}%{flag}")


def _print_comparison(exile, lane):
    headers = [
        ("Picks 1-5 unique archs", "early_unique_archetypes", ">= 3"),
        ("Picks 1-5 S/A emerging", "early_sa_for_emerging", "<= 2"),
        ("Picks 6+ S/A committed", "late_sa_for_committed", ">= 2"),
        ("Picks 6+ off-archetype", "late_off_archetype", ">= 0.5"),
        ("Convergence pick", "convergence_pick", "5-8"),
        ("Deck concentration", "deck_concentration", "0.60-0.90"),
        ("S/A stddev (late)", "sa_stddev", ">= 0.8"),
        ("Run overlap", "run_overlap", "< 0.40"),
    ]
    print(f"\n  {'Metric':<30s} {'Target':>10s} {'Exile':>10s} {'Lane Lock':>10s}")
    print("  " + "-" * 65)
    for label, key, target in headers:
        ev = exile[key]
        lv = lane[key]
        print(f"  {label:<30s} {target:>10s} {ev:>10.2f} {lv:>10.2f}")


if __name__ == "__main__":
    main()
