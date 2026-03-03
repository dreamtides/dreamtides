"""
Resonance V5 — Agent 1 Simulation: Pair-Based Threshold Auto-Spend with Scaling Bonus

One-sentence algorithm:
"Each card you draft with 2+ symbols adds 1 to its ordered pair count; when any
pair reaches 3, your next pack gets a bonus card sharing that pair and the count
resets to 0; if the pair has already triggered before, add 2 bonus cards instead."

This simulation implements:
- Card pool generation (360 cards, ~40 per archetype, 36 generic)
- The 8 archetypes on a circle with correct resonance assignments
- Fitness assignment (S-tier home, A-tier adjacent sharing primary, B-tier sharing secondary, C/F distant)
- Symbol counting (primary=2 weight, secondary/tertiary=1)
- Pair-Based Threshold Auto-Spend draft algorithm
- 3 player strategies (archetype-committed, power-chaser, signal-reader)
- 1000 drafts of 30 picks each
- ALL 8 metrics at archetype level
- V3 Lane Locking baseline
- V4 Pack Widening baseline (auto-spend)
- Parameter sensitivity sweeps
- Symbol distribution sensitivity
- 3 detailed draft traces
"""

import random
import math
from collections import defaultdict, Counter
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional
import statistics

# ============================================================
# Data Model
# ============================================================

class Resonance(Enum):
    EMBER = "Ember"
    STONE = "Stone"
    TIDE = "Tide"
    ZEPHYR = "Zephyr"

class Tier(Enum):
    S = "S"
    A = "A"
    B = "B"
    C = "C"
    F = "F"

# 8 archetypes on a circle
ARCHETYPES = [
    {"name": "Flash/Tempo/Prison",   "primary": Resonance.ZEPHYR, "secondary": Resonance.EMBER,  "pos": 0},
    {"name": "Blink/Flicker",        "primary": Resonance.EMBER,  "secondary": Resonance.ZEPHYR, "pos": 1},
    {"name": "Storm/Spellslinger",   "primary": Resonance.EMBER,  "secondary": Resonance.STONE,  "pos": 2},
    {"name": "Self-Discard",         "primary": Resonance.STONE,  "secondary": Resonance.EMBER,  "pos": 3},
    {"name": "Self-Mill/Reanimator", "primary": Resonance.STONE,  "secondary": Resonance.TIDE,   "pos": 4},
    {"name": "Sacrifice/Abandon",    "primary": Resonance.TIDE,   "secondary": Resonance.STONE,  "pos": 5},
    {"name": "Warriors/Midrange",    "primary": Resonance.TIDE,   "secondary": Resonance.ZEPHYR, "pos": 6},
    {"name": "Ramp/Spirit Animals",  "primary": Resonance.ZEPHYR, "secondary": Resonance.TIDE,   "pos": 7},
]

ARCHETYPE_NAMES = [a["name"] for a in ARCHETYPES]

def circle_distance(pos1, pos2):
    """Distance on the 8-position circle."""
    d = abs(pos1 - pos2)
    return min(d, 8 - d)

@dataclass
class SimCard:
    id: int
    symbols: list  # list of Resonance, ordered, 0-3 elements
    archetype: Optional[str]  # None for generics
    archetype_fitness: dict = field(default_factory=dict)  # archetype_name -> Tier
    power: float = 5.0

    def ordered_pair(self):
        """Return (primary, secondary) if 2+ symbols, else None."""
        if len(self.symbols) >= 2:
            return (self.symbols[0], self.symbols[1])
        return None

    def primary_resonance(self):
        if len(self.symbols) >= 1:
            return self.symbols[0]
        return None


# ============================================================
# Card Pool Generation
# ============================================================

def generate_pool(sym_dist=None, seed=None):
    """
    Generate 360 cards: ~40 per archetype (320) + 36 generic.
    sym_dist: dict with keys 1,2,3 mapping to fraction of non-generic cards.
    Default: {1: 0.15, 2: 0.65, 3: 0.20}
    """
    if seed is not None:
        random.seed(seed)

    if sym_dist is None:
        sym_dist = {1: 0.15, 2: 0.65, 3: 0.20}

    cards = []
    card_id = 0

    # Per archetype: 40 cards each
    cards_per_archetype = 40
    for arch in ARCHETYPES:
        pri = arch["primary"]
        sec = arch["secondary"]

        n1 = round(cards_per_archetype * sym_dist[1])
        n3 = round(cards_per_archetype * sym_dist[3])
        n2 = cards_per_archetype - n1 - n3

        # 1-symbol cards: all primary
        for _ in range(n1):
            c = SimCard(id=card_id, symbols=[pri], archetype=arch["name"],
                        power=random.uniform(3, 8))
            card_id += 1
            cards.append(c)

        # 2-symbol cards: [primary, secondary]
        for _ in range(n2):
            c = SimCard(id=card_id, symbols=[pri, sec], archetype=arch["name"],
                        power=random.uniform(3, 8))
            card_id += 1
            cards.append(c)

        # 3-symbol cards: [primary, primary, secondary] or [primary, secondary, primary]
        for i in range(n3):
            if i % 2 == 0:
                syms = [pri, sec, pri]
            else:
                syms = [pri, pri, sec]
            c = SimCard(id=card_id, symbols=syms, archetype=arch["name"],
                        power=random.uniform(3, 8))
            card_id += 1
            cards.append(c)

    # 36 generic cards
    for _ in range(36):
        c = SimCard(id=card_id, symbols=[], archetype=None,
                    power=random.uniform(4, 9))
        card_id += 1
        cards.append(c)

    # Assign fitness
    for c in cards:
        c.archetype_fitness = {}
        if c.archetype is None:
            # Generic: B-tier in all archetypes
            for a in ARCHETYPES:
                c.archetype_fitness[a["name"]] = Tier.B
        else:
            home_arch = next(a for a in ARCHETYPES if a["name"] == c.archetype)
            for a in ARCHETYPES:
                if a["name"] == c.archetype:
                    c.archetype_fitness[a["name"]] = Tier.S
                elif a["primary"] == home_arch["primary"] and a["name"] != c.archetype:
                    # Adjacent sharing primary resonance -> A-tier
                    c.archetype_fitness[a["name"]] = Tier.A
                elif a["primary"] == home_arch["secondary"] or a["secondary"] == home_arch["primary"]:
                    # Shares secondary resonance -> B-tier
                    c.archetype_fitness[a["name"]] = Tier.B
                else:
                    dist = circle_distance(home_arch["pos"], a["pos"])
                    if dist <= 2:
                        c.archetype_fitness[a["name"]] = Tier.C
                    else:
                        c.archetype_fitness[a["name"]] = Tier.F

    return cards


def get_pair_matched_cards(pool, pair):
    """Get all cards from pool whose ordered pair matches."""
    return [c for c in pool if c.ordered_pair() == pair]


def get_resonance_matched_cards(pool, resonance):
    """Get cards from pool whose primary resonance matches."""
    return [c for c in pool if c.primary_resonance() == resonance]


# ============================================================
# Draft Algorithms
# ============================================================

def generate_base_pack(pool, size=4):
    """Draw `size` random cards from pool (with replacement)."""
    return random.choices(pool, k=size)


# --- Algorithm 1: Pair-Based Threshold Auto-Spend with Scaling Bonus ---

def pair_threshold_draft(pool, player_strategy, target_archetype, threshold=3,
                          base_bonus=1, scaling=True, num_picks=30):
    """
    Each card you draft with 2+ symbols adds 1 to its ordered pair count;
    when any pair reaches `threshold`, your next pack gets bonus cards sharing
    that pair and the count resets to 0. If scaling=True and the pair has
    triggered before, add 2 bonus cards instead of 1.
    """
    pair_counts = defaultdict(int)
    pair_trigger_history = defaultdict(int)  # how many times each pair has triggered
    drafted = []
    pack_log = []  # for tracing

    # Track resonance counts for player strategy
    resonance_counts = defaultdict(int)

    for pick_num in range(num_picks):
        # Determine bonus cards for this pack
        bonus_cards = []
        # Check highest pair count
        if pair_counts:
            max_pair = max(pair_counts, key=pair_counts.get)
            if pair_counts[max_pair] >= threshold:
                # Determine bonus count
                if scaling and pair_trigger_history[max_pair] > 0:
                    num_bonus = 2
                else:
                    num_bonus = base_bonus

                # Find pair-matched cards
                matched = get_pair_matched_cards(pool, max_pair)
                if matched:
                    bonus_cards = random.choices(matched, k=num_bonus)

                pair_trigger_history[max_pair] += 1
                pair_counts[max_pair] = 0

        # Generate pack
        base_pack = generate_base_pack(pool, 4)
        full_pack = base_pack + bonus_cards

        # Player picks
        pick = pick_card(full_pack, player_strategy, target_archetype,
                         drafted, resonance_counts, pick_num)

        drafted.append(pick)
        pack_log.append({
            "pick_num": pick_num,
            "pack": full_pack,
            "picked": pick,
            "bonus_count": len(bonus_cards),
            "pair_counts": dict(pair_counts),
        })

        # Update pair counts
        pair = pick.ordered_pair()
        if pair is not None:
            pair_counts[pair] += 1

        # Update resonance counts for strategy
        for i, sym in enumerate(pick.symbols):
            if i == 0:
                resonance_counts[sym] += 2
            else:
                resonance_counts[sym] += 1

    return drafted, pack_log


# --- V3 Lane Locking Baseline ---

def lane_locking_draft(pool, player_strategy, target_archetype, threshold1=3,
                        threshold2=8, num_picks=30):
    """
    V3 Lane Locking: When symbol count for a resonance reaches threshold1,
    lock 1 pack slot to that resonance. At threshold2, lock a second slot.
    Primary symbol counts as 2, secondary/tertiary as 1.
    """
    resonance_counts = defaultdict(int)
    locked_resonance = None
    locks = 0
    drafted = []
    pack_log = []

    for pick_num in range(num_picks):
        # Generate pack with locked slots
        pack = []
        for slot in range(4):
            if slot < locks and locked_resonance is not None:
                # Locked slot: draw from resonance-matched cards
                matched = get_resonance_matched_cards(pool, locked_resonance)
                if matched:
                    pack.append(random.choice(matched))
                else:
                    pack.append(random.choice(pool))
            else:
                pack.append(random.choice(pool))

        pick = pick_card(pack, player_strategy, target_archetype,
                         drafted, resonance_counts, pick_num)
        drafted.append(pick)
        pack_log.append({
            "pick_num": pick_num,
            "pack": pack,
            "picked": pick,
            "locks": locks,
        })

        # Update resonance counts
        for i, sym in enumerate(pick.symbols):
            if i == 0:
                resonance_counts[sym] += 2
            else:
                resonance_counts[sym] += 1

        # Check for lock thresholds
        if resonance_counts:
            top_res = max(resonance_counts, key=resonance_counts.get)
            top_count = resonance_counts[top_res]
            if top_count >= threshold2:
                locked_resonance = top_res
                locks = 2
            elif top_count >= threshold1:
                locked_resonance = top_res
                locks = max(locks, 1)

    return drafted, pack_log


# --- V4 Pack Widening Baseline (auto-spend) ---

def pack_widening_draft(pool, player_strategy, target_archetype, cost=3,
                         bonus=1, num_picks=30):
    """
    V4 Pack Widening with auto-spend: Each symbol adds tokens (primary=2,
    secondary/tertiary=1). When highest resonance >= cost, auto-spend:
    deduct cost tokens, add bonus card(s) of that resonance.
    """
    resonance_counts = defaultdict(int)
    drafted = []
    pack_log = []

    for pick_num in range(num_picks):
        # Check for auto-spend
        bonus_cards = []
        if resonance_counts:
            top_res = max(resonance_counts, key=resonance_counts.get)
            if resonance_counts[top_res] >= cost:
                resonance_counts[top_res] -= cost
                matched = get_resonance_matched_cards(pool, top_res)
                if matched:
                    bonus_cards = random.choices(matched, k=bonus)

        base_pack = generate_base_pack(pool, 4)
        full_pack = base_pack + bonus_cards

        pick = pick_card(full_pack, player_strategy, target_archetype,
                         drafted, resonance_counts, pick_num)
        drafted.append(pick)
        pack_log.append({
            "pick_num": pick_num,
            "pack": full_pack,
            "picked": pick,
            "bonus_count": len(bonus_cards),
        })

        # Update resonance counts
        for i, sym in enumerate(pick.symbols):
            if i == 0:
                resonance_counts[sym] += 2
            else:
                resonance_counts[sym] += 1

    return drafted, pack_log


# ============================================================
# Player Strategies
# ============================================================

def pick_card(pack, strategy, target_archetype, drafted, resonance_counts, pick_num):
    """Player picks a card from the pack based on strategy."""
    if strategy == "committed":
        return pick_committed(pack, target_archetype, pick_num)
    elif strategy == "power":
        return pick_power(pack)
    elif strategy == "signal":
        return pick_signal(pack, drafted, resonance_counts, pick_num)
    else:
        return random.choice(pack)


def pick_committed(pack, target_archetype, pick_num):
    """
    Archetype-committed: Picks card with highest fitness for target archetype.
    Early picks (1-5) have some randomness to simulate exploration.
    """
    tier_values = {Tier.S: 10, Tier.A: 7, Tier.B: 4, Tier.C: 2, Tier.F: 0}

    if pick_num < 5:
        # Early: mix of fitness and power (simulate exploration)
        def score(c):
            fitness = tier_values.get(c.archetype_fitness.get(target_archetype, Tier.F), 0)
            return fitness * 0.5 + c.power * 0.5
    else:
        # Late: strongly prefer fitness
        def score(c):
            fitness = tier_values.get(c.archetype_fitness.get(target_archetype, Tier.F), 0)
            return fitness * 0.9 + c.power * 0.1

    return max(pack, key=score)


def pick_power(pack):
    """Power-chaser: always pick highest raw power."""
    return max(pack, key=lambda c: c.power)


def pick_signal(pack, drafted, resonance_counts, pick_num):
    """
    Signal-reader: evaluates which resonance appears most available in packs
    and drafts toward the open archetype. Uses resonance counts to identify
    commitment, then picks S/A for the emerging archetype.
    """
    tier_values = {Tier.S: 10, Tier.A: 7, Tier.B: 4, Tier.C: 2, Tier.F: 0}

    if pick_num < 5:
        # Early: pick highest power with slight resonance preference
        def score(c):
            res_bonus = 0
            if c.primary_resonance() and resonance_counts:
                top_res = max(resonance_counts, key=resonance_counts.get) if resonance_counts else None
                if top_res and c.primary_resonance() == top_res:
                    res_bonus = 1
            return c.power + res_bonus
        return max(pack, key=score)
    else:
        # Late: identify best archetype from drafted cards and pick for it
        if resonance_counts:
            top_res = max(resonance_counts, key=resonance_counts.get)
            # Find archetypes with this primary
            candidate_archetypes = [a["name"] for a in ARCHETYPES if a["primary"] == top_res]
            # Pick the archetype we have more S/A cards for
            best_arch = None
            best_count = -1
            for arch_name in candidate_archetypes:
                count = sum(1 for c in drafted
                           if c.archetype_fitness.get(arch_name, Tier.F) in (Tier.S, Tier.A))
                if count > best_count:
                    best_count = count
                    best_arch = arch_name
            if best_arch:
                def score(c):
                    fitness = tier_values.get(c.archetype_fitness.get(best_arch, Tier.F), 0)
                    return fitness * 0.85 + c.power * 0.15
                return max(pack, key=score)

        return max(pack, key=lambda c: c.power)


# ============================================================
# Metrics Computation
# ============================================================

def compute_metrics(drafted_cards, pack_logs, target_archetype):
    """Compute all 8 measurable targets + variance at archetype level."""

    metrics = {}

    # Helper: is card S/A for target archetype?
    def is_sa(card):
        return card.archetype_fitness.get(target_archetype, Tier.F) in (Tier.S, Tier.A)

    def is_cf(card):
        return card.archetype_fitness.get(target_archetype, Tier.F) in (Tier.C, Tier.F)

    # Picks 1-5 metrics
    early_unique_archetypes = []
    early_sa_for_target = []
    for log in pack_logs[:5]:
        pack = log["pack"]
        # Unique archetypes with S/A cards in this pack
        archs_with_sa = set()
        for c in pack:
            for a in ARCHETYPE_NAMES:
                if c.archetype_fitness.get(a, Tier.F) in (Tier.S, Tier.A):
                    archs_with_sa.add(a)
        early_unique_archetypes.append(len(archs_with_sa))
        # S/A cards for target archetype
        sa_count = sum(1 for c in pack if is_sa(c))
        early_sa_for_target.append(sa_count)

    metrics["early_unique_archetypes"] = statistics.mean(early_unique_archetypes) if early_unique_archetypes else 0
    metrics["early_sa_for_target"] = statistics.mean(early_sa_for_target) if early_sa_for_target else 0

    # Picks 6+ metrics
    late_sa_counts = []
    late_cf_counts = []
    for log in pack_logs[5:]:
        pack = log["pack"]
        sa_count = sum(1 for c in pack if is_sa(c))
        cf_count = sum(1 for c in pack if is_cf(c))
        late_sa_counts.append(sa_count)
        late_cf_counts.append(cf_count)

    metrics["late_sa_per_pack"] = statistics.mean(late_sa_counts) if late_sa_counts else 0
    metrics["late_cf_per_pack"] = statistics.mean(late_cf_counts) if late_cf_counts else 0
    metrics["late_sa_stddev"] = statistics.stdev(late_sa_counts) if len(late_sa_counts) > 1 else 0

    # Convergence pick: first pick where trailing-3 average of S/A >= 2.0
    convergence_pick = 30  # default: never converges
    window = 3
    for i in range(window - 1, len(pack_logs)):
        recent_sa = []
        for log in pack_logs[max(0, i - window + 1):i + 1]:
            sa = sum(1 for c in log["pack"] if is_sa(c))
            recent_sa.append(sa)
        if statistics.mean(recent_sa) >= 2.0:
            convergence_pick = pack_logs[max(0, i - window + 1)]["pick_num"] + 1  # 1-indexed
            break
    metrics["convergence_pick"] = convergence_pick

    # Deck archetype concentration
    sa_in_deck = sum(1 for c in drafted_cards if is_sa(c))
    metrics["deck_concentration"] = sa_in_deck / len(drafted_cards) if drafted_cards else 0

    return metrics, late_sa_counts


def compute_run_variety(all_drafted_ids_list):
    """Compute average card overlap between runs with same starting conditions."""
    if len(all_drafted_ids_list) < 2:
        return 0.0
    overlaps = []
    for i in range(len(all_drafted_ids_list)):
        for j in range(i + 1, len(all_drafted_ids_list)):
            set_i = set(all_drafted_ids_list[i])
            set_j = set(all_drafted_ids_list[j])
            if len(set_i | set_j) > 0:
                overlap = len(set_i & set_j) / len(set_i | set_j)
                overlaps.append(overlap)
    return statistics.mean(overlaps) if overlaps else 0.0


# ============================================================
# Simulation Runner
# ============================================================

def run_simulation(algorithm_fn, pool, n_runs=1000, num_picks=30, **algo_kwargs):
    """Run n_runs drafts and aggregate metrics."""
    all_metrics = defaultdict(list)
    all_late_sa = []
    archetype_freq = defaultdict(int)
    all_drafted_ids = []

    for run in range(n_runs):
        # Pick a random target archetype
        target_arch = random.choice(ARCHETYPE_NAMES)
        archetype_freq[target_arch] += 1

        # Choose strategy: 60% committed, 20% power, 20% signal
        r = random.random()
        if r < 0.6:
            strategy = "committed"
        elif r < 0.8:
            strategy = "power"
        else:
            strategy = "signal"

        drafted, pack_log = algorithm_fn(pool, strategy, target_arch,
                                          num_picks=num_picks, **algo_kwargs)
        metrics, late_sa = compute_metrics(drafted, pack_log, target_arch)
        all_late_sa.extend(late_sa)

        for k, v in metrics.items():
            all_metrics[k].append(v)

        all_drafted_ids.append([c.id for c in drafted])

    # Aggregate
    results = {}
    for k, v in all_metrics.items():
        results[k] = statistics.mean(v)

    results["late_sa_stddev_global"] = statistics.stdev(all_late_sa) if len(all_late_sa) > 1 else 0

    # Run variety: sample 50 pairs of runs with same archetype
    variety_overlaps = []
    for arch in ARCHETYPE_NAMES:
        arch_runs = [ids for i, ids in enumerate(all_drafted_ids)
                     if all_metrics["convergence_pick"][i] is not None]
        # Just compute pairwise overlap on a sample
        sample = random.sample(all_drafted_ids, min(20, len(all_drafted_ids)))
        if len(sample) >= 2:
            variety_overlaps.append(compute_run_variety(sample))
    results["run_variety"] = statistics.mean(variety_overlaps) if variety_overlaps else 0

    # Archetype frequency
    results["archetype_freq"] = {k: v / n_runs for k, v in archetype_freq.items()}

    return results


def run_per_archetype_convergence(algorithm_fn, pool, n_runs_per=200, num_picks=30, **algo_kwargs):
    """Run simulation for each archetype with committed players and report convergence."""
    convergence_table = {}
    for arch in ARCHETYPES:
        target = arch["name"]
        convergence_picks = []
        for _ in range(n_runs_per):
            drafted, pack_log = algorithm_fn(pool, "committed", target,
                                              num_picks=num_picks, **algo_kwargs)
            metrics, _ = compute_metrics(drafted, pack_log, target)
            convergence_picks.append(metrics["convergence_pick"])
        convergence_table[target] = statistics.mean(convergence_picks)
    return convergence_table


# ============================================================
# Main Simulation
# ============================================================

def print_separator():
    print("=" * 70)


def print_scorecard(results, label):
    """Print the target scorecard table."""
    print(f"\n{'='*70}")
    print(f"  SCORECARD: {label}")
    print(f"{'='*70}")

    targets = [
        ("Picks 1-5: unique archetypes w/ S/A per pack", "early_unique_archetypes", ">= 3", lambda x: x >= 3),
        ("Picks 1-5: S/A for target per pack", "early_sa_for_target", "<= 2", lambda x: x <= 2),
        ("Picks 6+: S/A for archetype per pack", "late_sa_per_pack", ">= 2.0", lambda x: x >= 2.0),
        ("Picks 6+: off-archetype (C/F) per pack", "late_cf_per_pack", ">= 0.5", lambda x: x >= 0.5),
        ("Convergence pick", "convergence_pick", "5-8", lambda x: 5 <= x <= 8),
        ("Deck concentration", "deck_concentration", "60-90%", lambda x: 0.60 <= x <= 0.90),
        ("Run-to-run variety (overlap)", "run_variety", "< 40%", lambda x: x < 0.40),
        ("Variance (stddev S/A picks 6+)", "late_sa_stddev_global", ">= 0.8", lambda x: x >= 0.8),
    ]

    print(f"{'Metric':<50} {'Target':<12} {'Actual':<12} {'Pass/Fail'}")
    print("-" * 86)
    for name, key, target_str, check_fn in targets:
        val = results.get(key, 0)
        if "concentration" in key or "variety" in key:
            val_str = f"{val:.1%}"
        else:
            val_str = f"{val:.2f}"
        passed = "PASS" if check_fn(val) else "FAIL"
        print(f"{name:<50} {target_str:<12} {val_str:<12} {passed}")


def print_convergence_table(conv_table, label):
    """Print per-archetype convergence table."""
    print(f"\n  Per-Archetype Convergence: {label}")
    print(f"  {'Archetype':<28} {'Avg Convergence Pick'}")
    print(f"  {'-'*50}")
    for arch in ARCHETYPES:
        name = arch["name"]
        pick = conv_table.get(name, 30)
        print(f"  {name:<28} {pick:.1f}")


def print_draft_trace(pack_logs, drafted, target_archetype, label):
    """Print an annotated draft trace."""
    print(f"\n{'='*70}")
    print(f"  DRAFT TRACE: {label} (targeting {target_archetype})")
    print(f"{'='*70}")

    for i, log in enumerate(pack_logs[:15]):  # First 15 picks
        pack = log["pack"]
        picked = log["picked"]
        bonus = log.get("bonus_count", 0)

        sa_in_pack = sum(1 for c in pack
                        if c.archetype_fitness.get(target_archetype, Tier.F) in (Tier.S, Tier.A))

        picked_tier = picked.archetype_fitness.get(target_archetype, Tier.F).value
        picked_pair = picked.ordered_pair()
        pair_str = f"({picked_pair[0].value},{picked_pair[1].value})" if picked_pair else "(none)"

        sym_str = "/".join(s.value for s in picked.symbols) if picked.symbols else "generic"
        bonus_str = f" [+{bonus} bonus]" if bonus > 0 else ""

        pair_counts_str = ""
        if "pair_counts" in log and log["pair_counts"]:
            top_pairs = sorted(log["pair_counts"].items(), key=lambda x: -x[1])[:3]
            pair_counts_str = " | pairs: " + ", ".join(
                f"({p[0].value},{p[1].value})={c}" for (p, c) in top_pairs if c > 0
            )

        print(f"  Pick {i+1:2d}: pack={len(pack)} cards, {sa_in_pack} S/A for target{bonus_str} "
              f"| picked [{sym_str}] {picked.archetype or 'Generic'} ({picked_tier}){pair_str}"
              f"{pair_counts_str}")


def main():
    random.seed(42)

    print("=" * 70)
    print("  RESONANCE V5 SIMULATION — AGENT 1")
    print("  Pair-Based Threshold Auto-Spend with Scaling Bonus")
    print("=" * 70)

    # Generate pool
    pool = generate_pool(sym_dist={1: 0.15, 2: 0.65, 3: 0.20}, seed=42)
    print(f"\nPool generated: {len(pool)} cards")

    # Validate pool
    arch_counts = Counter(c.archetype for c in pool if c.archetype)
    generic_count = sum(1 for c in pool if c.archetype is None)
    sym_counts = Counter(len(c.symbols) for c in pool)
    print(f"  Archetypes: {dict(arch_counts)}")
    print(f"  Generics: {generic_count}")
    print(f"  Symbol counts: {dict(sym_counts)}")

    # Validate pair precision
    print("\n  Pair Precision Validation:")
    for arch in ARCHETYPES:
        pair = (arch["primary"], arch["secondary"])
        matched = get_pair_matched_cards(pool, pair)
        s_count = sum(1 for c in matched if c.archetype_fitness.get(arch["name"], Tier.F) == Tier.S)
        a_count = sum(1 for c in matched if c.archetype_fitness.get(arch["name"], Tier.F) == Tier.A)
        sa_pct = (s_count + a_count) / len(matched) * 100 if matched else 0
        print(f"    {arch['name']:<28} pair=({pair[0].value},{pair[1].value}): "
              f"{len(matched)} cards, {s_count}S + {a_count}A = {sa_pct:.0f}% S/A")

    # ============================================================
    # Run main algorithm: Pair-Based Threshold (threshold=3, scaling bonus)
    # ============================================================
    print_separator()
    print("\n>>> RUNNING: Pair-Based Threshold Auto-Spend (threshold=3, scaling bonus)")

    results_main = run_simulation(
        pair_threshold_draft, pool, n_runs=1000,
        threshold=3, base_bonus=1, scaling=True
    )
    print_scorecard(results_main, "Pair Threshold (T=3, Scaling Bonus)")

    conv_main = run_per_archetype_convergence(
        pair_threshold_draft, pool, n_runs_per=200,
        threshold=3, base_bonus=1, scaling=True
    )
    print_convergence_table(conv_main, "Pair Threshold (T=3, Scaling)")

    # ============================================================
    # V3 Lane Locking Baseline
    # ============================================================
    print_separator()
    print("\n>>> RUNNING: V3 Lane Locking Baseline (threshold 3/8)")

    results_ll = run_simulation(
        lane_locking_draft, pool, n_runs=1000,
        threshold1=3, threshold2=8
    )
    print_scorecard(results_ll, "V3 Lane Locking (3/8)")

    conv_ll = run_per_archetype_convergence(
        lane_locking_draft, pool, n_runs_per=200,
        threshold1=3, threshold2=8
    )
    print_convergence_table(conv_ll, "V3 Lane Locking")

    # ============================================================
    # V4 Pack Widening Baseline (auto-spend)
    # ============================================================
    print_separator()
    print("\n>>> RUNNING: V4 Pack Widening Baseline (cost=3, bonus=1, auto-spend)")

    results_pw = run_simulation(
        pack_widening_draft, pool, n_runs=1000,
        cost=3, bonus=1
    )
    print_scorecard(results_pw, "V4 Pack Widening (cost=3, bonus=1)")

    conv_pw = run_per_archetype_convergence(
        pack_widening_draft, pool, n_runs_per=200,
        cost=3, bonus=1
    )
    print_convergence_table(conv_pw, "V4 Pack Widening")

    # ============================================================
    # Parameter Sensitivity: Threshold 2 vs 3, Bonus 1 vs Scaling
    # ============================================================
    print_separator()
    print("\n>>> PARAMETER SENSITIVITY SWEEP")

    configs = [
        ("T=2, Bonus=1, No Scaling", {"threshold": 2, "base_bonus": 1, "scaling": False}),
        ("T=2, Bonus=1, Scaling", {"threshold": 2, "base_bonus": 1, "scaling": True}),
        ("T=3, Bonus=1, No Scaling", {"threshold": 3, "base_bonus": 1, "scaling": False}),
        ("T=3, Bonus=1, Scaling", {"threshold": 3, "base_bonus": 1, "scaling": True}),
        ("T=2, Bonus=2, No Scaling", {"threshold": 2, "base_bonus": 2, "scaling": False}),
        ("T=3, Bonus=2, No Scaling", {"threshold": 3, "base_bonus": 2, "scaling": False}),
    ]

    print(f"\n{'Config':<35} {'Late S/A':<10} {'Conv Pick':<10} {'StdDev':<10} {'Deck Conc':<10} {'C/F':<10}")
    print("-" * 85)

    for label, kwargs in configs:
        r = run_simulation(pair_threshold_draft, pool, n_runs=500, **kwargs)
        print(f"{label:<35} {r['late_sa_per_pack']:<10.2f} {r['convergence_pick']:<10.1f} "
              f"{r['late_sa_stddev_global']:<10.2f} {r['deck_concentration']:<10.1%} {r['late_cf_per_pack']:<10.2f}")

    # ============================================================
    # Symbol Distribution Sensitivity
    # ============================================================
    print_separator()
    print("\n>>> SYMBOL DISTRIBUTION SENSITIVITY")

    distributions = [
        ("15% 1-sym (default)", {1: 0.15, 2: 0.65, 3: 0.20}),
        ("30% 1-sym (stress)", {1: 0.30, 2: 0.50, 3: 0.20}),
        ("5% 1-sym (pair-heavy)", {1: 0.05, 2: 0.75, 3: 0.20}),
    ]

    print(f"\n{'Distribution':<30} {'Late S/A':<10} {'Conv Pick':<10} {'StdDev':<10} {'Deck Conc':<10}")
    print("-" * 70)

    for label, dist in distributions:
        test_pool = generate_pool(sym_dist=dist, seed=123)
        r = run_simulation(pair_threshold_draft, test_pool, n_runs=500,
                           threshold=3, base_bonus=1, scaling=True)
        print(f"{label:<30} {r['late_sa_per_pack']:<10.2f} {r['convergence_pick']:<10.1f} "
              f"{r['late_sa_stddev_global']:<10.2f} {r['deck_concentration']:<10.1%}")

    # ============================================================
    # Draft Traces
    # ============================================================
    print_separator()
    print("\n>>> DRAFT TRACES")

    # Trace 1: Early committer (Warriors)
    random.seed(100)
    drafted1, log1 = pair_threshold_draft(
        pool, "committed", "Warriors/Midrange",
        threshold=3, base_bonus=1, scaling=True, num_picks=30
    )
    print_draft_trace(log1, drafted1, "Warriors/Midrange", "Early Committer")

    # Trace 2: Flexible player (stays open 8+ picks)
    random.seed(200)
    drafted2, log2 = pair_threshold_draft(
        pool, "signal", "Storm/Spellslinger",
        threshold=3, base_bonus=1, scaling=True, num_picks=30
    )
    print_draft_trace(log2, drafted2, "Storm/Spellslinger", "Signal Reader / Flexible")

    # Trace 3: Power chaser
    random.seed(300)
    drafted3, log3 = pair_threshold_draft(
        pool, "power", "Blink/Flicker",
        threshold=3, base_bonus=1, scaling=True, num_picks=30
    )
    print_draft_trace(log3, drafted3, "Blink/Flicker", "Power Chaser")

    # ============================================================
    # Side-by-side Comparison Table
    # ============================================================
    print_separator()
    print("\n>>> SIDE-BY-SIDE COMPARISON")
    print(f"\n{'Metric':<50} {'Pair Thresh':<14} {'Lane Lock':<14} {'Pack Widen':<14}")
    print("-" * 92)

    comparison_metrics = [
        ("Picks 1-5: unique archs w/ S/A", "early_unique_archetypes"),
        ("Picks 1-5: S/A for target", "early_sa_for_target"),
        ("Picks 6+: S/A per pack", "late_sa_per_pack"),
        ("Picks 6+: C/F per pack", "late_cf_per_pack"),
        ("Convergence pick", "convergence_pick"),
        ("Deck concentration", "deck_concentration"),
        ("Run-to-run variety (overlap)", "run_variety"),
        ("Variance (stddev S/A 6+)", "late_sa_stddev_global"),
    ]

    for name, key in comparison_metrics:
        v1 = results_main.get(key, 0)
        v2 = results_ll.get(key, 0)
        v3 = results_pw.get(key, 0)
        if "concentration" in key or "variety" in key:
            print(f"{name:<50} {v1:<14.1%} {v2:<14.1%} {v3:<14.1%}")
        else:
            print(f"{name:<50} {v1:<14.2f} {v2:<14.2f} {v3:<14.2f}")

    # ============================================================
    # S/A Distribution for committed player (picks 6+)
    # ============================================================
    print_separator()
    print("\n>>> S/A DISTRIBUTION (Picks 6+, Committed Player)")

    dist_counts = defaultdict(int)
    random.seed(42)
    for _ in range(500):
        target = random.choice(ARCHETYPE_NAMES)
        drafted_d, log_d = pair_threshold_draft(
            pool, "committed", target,
            threshold=3, base_bonus=1, scaling=True, num_picks=30
        )
        for log_entry in log_d[5:]:
            sa = sum(1 for c in log_entry["pack"]
                    if c.archetype_fitness.get(target, Tier.F) in (Tier.S, Tier.A))
            dist_counts[sa] += 1

    total_packs = sum(dist_counts.values())
    print(f"\n  {'S/A Count':<12} {'Frequency':<12} {'Percentage'}")
    print(f"  {'-'*36}")
    for sa_count in sorted(dist_counts.keys()):
        freq = dist_counts[sa_count]
        pct = freq / total_packs * 100
        print(f"  {sa_count:<12} {freq:<12} {pct:.1f}%")

    # ============================================================
    # Archetype Frequency
    # ============================================================
    print_separator()
    print("\n>>> ARCHETYPE FREQUENCY (from main run)")
    for name in ARCHETYPE_NAMES:
        freq = results_main.get("archetype_freq", {}).get(name, 0)
        print(f"  {name:<28} {freq:.1%}")

    print("\n" + "=" * 70)
    print("  SIMULATION COMPLETE")
    print("=" * 70)


if __name__ == "__main__":
    main()
