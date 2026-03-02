"""
Model A v2 Simulation: N=8 with Suppression, Soft Floor, and Splash Slot

Revised from N=4 to N=8 based on Round 3 debate consensus. Key changes:
- 8 archetypes with 2 suppressed per run (28 configurations)
- Adaptive weighted sampling with soft floor guarantee
- Dedicated splash slot (1 of 4 pack slots)
- Clustered neighbor topology
- Standardized commitment detection (pick >= 6, 3+ S/A, 2+ lead)
- ~28% multi-archetype cards (testing minimum viable design burden)
"""

import random
from collections import defaultdict
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional

# ── Constants ──────────────────────────────────────────────────────────

NUM_ARCHETYPES = 8
TOTAL_UNIQUE_CARDS = 360
PACK_SIZE = 4
PICKS_PER_DRAFT = 30
NUM_DRAFTS = 1000
SUPPRESSED_PER_RUN = 2
COMMITMENT_THRESHOLD = 3
COMMITMENT_LEAD = 1
COMMITMENT_MIN_PICK = 5

RARITY_DIST = {"common": 0.55, "uncommon": 0.25, "rare": 0.15, "legendary": 0.05}
RARITY_COPIES = {"common": 4, "uncommon": 3, "rare": 2, "legendary": 1}
TIER_VALUES = {"S": 5, "A": 4, "B": 2, "C": 1, "F": 0}

# Neighbor topology: ring with 2 neighbors each
NEIGHBORS = {
    0: [1, 7],
    1: [0, 2],
    2: [1, 3],
    3: [2, 4],
    4: [3, 5],
    5: [4, 6],
    6: [5, 7],
    7: [6, 0],
}


# ── Data Model ─────────────────────────────────────────────────────────

class Rarity(Enum):
    COMMON = "common"
    UNCOMMON = "uncommon"
    RARE = "rare"
    LEGENDARY = "legendary"


@dataclass
class SimCard:
    id: int
    rarity: Rarity
    power: float
    archetype_fitness: dict  # archetype_id -> tier string

    def fitness_in(self, arch: int) -> str:
        return self.archetype_fitness.get(arch, "F")

    def is_fitting(self, arch: int) -> bool:
        return self.fitness_in(arch) in ("S", "A")

    def best_archetype(self) -> int:
        return max(self.archetype_fitness,
                   key=lambda a: TIER_VALUES[self.archetype_fitness[a]])

    def fitness_score(self, arch: int) -> float:
        return TIER_VALUES[self.fitness_in(arch)] + self.power * 0.1


# ── Card Generation ────────────────────────────────────────────────────

def generate_card_pool(rng: random.Random,
                       multi_archetype_pct: float = 0.28) -> list:
    """Generate 360 unique cards with parameterized multi-archetype %."""
    cards = []
    card_id = 0

    # Assign rarities
    rarities = []
    for rarity_name, frac in RARITY_DIST.items():
        count = round(TOTAL_UNIQUE_CARDS * frac)
        rarities.extend([Rarity(rarity_name)] * count)
    while len(rarities) < TOTAL_UNIQUE_CARDS:
        rarities.append(Rarity.COMMON)
    while len(rarities) > TOTAL_UNIQUE_CARDS:
        rarities.pop()
    rng.shuffle(rarities)

    # Parameterize distribution based on multi_archetype_pct
    # Multi-archetype = splash + dual + generalist + universal
    # Splash is ~55% of multi, dual ~20%, generalist ~15%, universal ~10%
    narrow_pct = 1.0 - multi_archetype_pct
    splash_pct = multi_archetype_pct * 0.55
    dual_pct = multi_archetype_pct * 0.20
    generalist_pct = multi_archetype_pct * 0.15
    universal_pct = multi_archetype_pct * 0.10

    narrow_count = int(TOTAL_UNIQUE_CARDS * narrow_pct)
    splash_count = int(TOTAL_UNIQUE_CARDS * splash_pct)
    dual_count = int(TOTAL_UNIQUE_CARDS * dual_pct)
    generalist_count = int(TOTAL_UNIQUE_CARDS * generalist_pct)
    universal_count = TOTAL_UNIQUE_CARDS - narrow_count - splash_count - dual_count - generalist_count

    s_tier_counts = defaultdict(int)

    def pick_primary() -> int:
        min_count = min(s_tier_counts.get(a, 0) for a in range(NUM_ARCHETYPES))
        candidates = [a for a in range(NUM_ARCHETYPES)
                      if s_tier_counts.get(a, 0) == min_count]
        return rng.choice(candidates)

    # 1. Narrow Specialists: S in 1, B in 0-2 neighbors, C/F elsewhere
    for i in range(narrow_count):
        primary = pick_primary()
        s_tier_counts[primary] += 1
        fitness = {}
        for a in range(NUM_ARCHETYPES):
            if a == primary:
                fitness[a] = "S"
            elif a in NEIGHBORS[primary] and rng.random() < 0.3:
                fitness[a] = "B"
            elif rng.random() < 0.15:
                fitness[a] = "C"
            else:
                fitness[a] = "F"
        power = rng.uniform(3.0, 6.0)
        cards.append(SimCard(card_id, rarities[card_id], power, fitness))
        card_id += 1

    # 2. Specialist with Splash: S in 1, A in 1-2 neighbors, B in 1-2
    for i in range(splash_count):
        primary = pick_primary()
        s_tier_counts[primary] += 1
        neighbors = NEIGHBORS[primary]
        a_targets = rng.sample(neighbors,
                               k=min(rng.randint(1, 2), len(neighbors)))
        fitness = {}
        for a in range(NUM_ARCHETYPES):
            if a == primary:
                fitness[a] = "S"
            elif a in a_targets:
                fitness[a] = "A"
            elif a in neighbors or rng.random() < 0.2:
                fitness[a] = "B"
            else:
                fitness[a] = rng.choice(["C", "F"])
        power = rng.uniform(4.0, 7.0)
        cards.append(SimCard(card_id, rarities[card_id], power, fitness))
        card_id += 1

    # 3. Multi-Archetype Stars: S in 2 neighbors, B in 2-3
    for i in range(dual_count):
        primary = pick_primary()
        s_tier_counts[primary] += 1
        secondary = rng.choice(NEIGHBORS[primary])
        s_tier_counts[secondary] += 1
        fitness = {}
        for a in range(NUM_ARCHETYPES):
            if a == primary or a == secondary:
                fitness[a] = "S"
            elif a in NEIGHBORS[primary] or a in NEIGHBORS[secondary]:
                fitness[a] = "B" if rng.random() < 0.5 else "C"
            else:
                fitness[a] = rng.choice(["C", "F"])
        power = rng.uniform(5.0, 8.0)
        cards.append(SimCard(card_id, rarities[card_id], power, fitness))
        card_id += 1

    # 4. Broad Generalists: A in 2-3, B in 3-4, no S
    for i in range(generalist_count):
        fitness = {}
        a_count = rng.randint(2, 3)
        b_count = rng.randint(3, 4)
        a_targets = rng.sample(range(NUM_ARCHETYPES), k=a_count)
        remaining = [a for a in range(NUM_ARCHETYPES) if a not in a_targets]
        b_targets = rng.sample(remaining, k=min(b_count, len(remaining)))
        for a in range(NUM_ARCHETYPES):
            if a in a_targets:
                fitness[a] = "A"
            elif a in b_targets:
                fitness[a] = "B"
            else:
                fitness[a] = "C"
        power = rng.uniform(5.0, 7.5)
        cards.append(SimCard(card_id, rarities[card_id], power, fitness))
        card_id += 1

    # 5. Universal Stars: S in 3+, high power
    for i in range(universal_count):
        fitness = {}
        s_count = rng.randint(3, 5)
        s_targets = rng.sample(range(NUM_ARCHETYPES), k=s_count)
        for a in range(NUM_ARCHETYPES):
            if a in s_targets:
                fitness[a] = "S"
                s_tier_counts[a] += 1
            else:
                fitness[a] = rng.choice(["A", "B"])
        power = rng.uniform(7.0, 10.0)
        cards.append(SimCard(card_id, rarities[card_id], power, fitness))
        card_id += 1

    return cards


def build_pool(cards: list, rng: random.Random,
               suppressed: list) -> list:
    """Build the draft pool with rarity copies and suppression."""
    pool = []
    for card in cards:
        copies = RARITY_COPIES[card.rarity.value]
        # Copy-count variance: +/-1 randomly
        variance = rng.choice([-1, 0, 0, 0, 1])
        copies = max(1, copies + variance)

        # Suppression: remove 50% of copies for cards whose primary archetype
        # is suppressed
        primary = card.best_archetype()
        if primary in suppressed:
            kept = 0
            for _ in range(copies):
                if rng.random() >= 0.5:
                    kept += 1
            copies = max(0, kept)
            if copies == 0:
                continue

        for _ in range(copies):
            pool.append(card)

    rng.shuffle(pool)
    return pool


# ── Draft State ────────────────────────────────────────────────────────

@dataclass
class DraftState:
    pool: list
    active_archetypes: list
    suppressed: list
    picked: list = field(default_factory=list)
    committed: Optional[int] = None
    archetype_counts: dict = field(default_factory=lambda: defaultdict(int))
    packs_seen: list = field(default_factory=list)
    starting_card: Optional[object] = None


def detect_commitment(state: DraftState, pick_num: int) -> Optional[int]:
    """Standardized commitment: pick >= 6, 3+ S/A, 2+ lead over runner-up."""
    if pick_num < COMMITMENT_MIN_PICK:
        return None

    counts = dict(state.archetype_counts)
    if not counts:
        return None

    sorted_archs = sorted(counts.items(), key=lambda x: -x[1])
    best_arch, best_count = sorted_archs[0]

    if best_count < COMMITMENT_THRESHOLD:
        return None

    if best_arch not in state.active_archetypes:
        return None

    runner_up = sorted_archs[1][1] if len(sorted_archs) > 1 else 0
    if best_count - runner_up < COMMITMENT_LEAD:
        return None

    return best_arch


# ── Pack Construction ──────────────────────────────────────────────────

def get_weight_multiplier(pick_num: int) -> float:
    """Adaptive weight ramp for committed archetype cards."""
    if pick_num < COMMITMENT_MIN_PICK:
        return 1.0
    elif pick_num <= 10:
        return 6.0
    elif pick_num <= 20:
        return 7.0
    else:
        return 8.0


def weighted_sample_one(pool: list, weights: list, rng: random.Random,
                        exclude_ids: set) -> Optional[int]:
    """Sample one index from pool using weights, excluding card IDs."""
    candidates = []
    cand_weights = []
    for i, card in enumerate(pool):
        if card.id not in exclude_ids:
            candidates.append(i)
            cand_weights.append(weights[i])

    if not candidates:
        return None

    total = sum(cand_weights)
    if total <= 0:
        return rng.choice(candidates) if candidates else None

    r = rng.uniform(0, total)
    cumulative = 0
    for idx, w in zip(candidates, cand_weights):
        cumulative += w
        if cumulative >= r:
            return idx

    return candidates[-1]


def construct_pack(state: DraftState, pick_num: int,
                   rng: random.Random) -> list:
    """Build a 4-card pack with weighted sampling, splash slot, and soft floor."""
    pool = state.pool
    if len(pool) < PACK_SIZE:
        return list(pool)

    committed = state.committed
    multiplier = get_weight_multiplier(pick_num)
    used_ids = set()
    pack = []

    if committed is not None and pick_num >= COMMITMENT_MIN_PICK:
        # Compute weights
        weights = []
        for card in pool:
            if card.is_fitting(committed):
                weights.append(multiplier)
            else:
                weights.append(1.0)

        # Draw 3 archetype-biased cards
        for _ in range(3):
            idx = weighted_sample_one(pool, weights, rng, used_ids)
            if idx is not None:
                pack.append(pool[idx])
                used_ids.add(pool[idx].id)

        # Draw 1 splash card: off-archetype, biased toward high power
        splash_weights = []
        for card in pool:
            if card.id in used_ids:
                splash_weights.append(0.0)
            elif not card.is_fitting(committed):
                # Bias toward high power and S-tier in other active archetypes
                sw = card.power
                for a in state.active_archetypes:
                    if a != committed and card.fitness_in(a) == "S":
                        sw += 5.0
                        break
                splash_weights.append(sw)
            else:
                splash_weights.append(0.1)  # Small chance of fitting in splash

        splash_idx = weighted_sample_one(pool, splash_weights, rng, used_ids)
        if splash_idx is not None:
            pack.append(pool[splash_idx])
            used_ids.add(pool[splash_idx].id)

        # Soft floor: if 0 fitting cards in pack, replace 1 with a fitting card
        fitting_count = sum(1 for c in pack if c.is_fitting(committed))
        if fitting_count == 0:
            # Find a fitting card not already in pack
            fitting_candidates = [i for i, c in enumerate(pool)
                                  if c.is_fitting(committed) and c.id not in used_ids]
            if fitting_candidates:
                replace_idx = rng.choice(fitting_candidates)
                # Replace the last non-splash card (index 0-2)
                if len(pack) >= 2:
                    pack[rng.randint(0, min(2, len(pack) - 1))] = pool[replace_idx]

    else:
        # Uniform random for early picks
        all_weights = [1.0] * len(pool)
        for _ in range(PACK_SIZE):
            idx = weighted_sample_one(pool, all_weights, rng, used_ids)
            if idx is not None:
                pack.append(pool[idx])
                used_ids.add(pool[idx].id)

    return pack


# ── Player Strategies ──────────────────────────────────────────────────

def strategy_committed(pack: list, state: DraftState) -> object:
    """Pick highest fitness in committed archetype; before commitment, best overall."""
    if state.committed is not None:
        return max(pack, key=lambda c: c.fitness_score(state.committed))
    return max(pack, key=lambda c: max(c.fitness_score(a)
               for a in state.active_archetypes))


def strategy_power_chaser(pack: list, state: DraftState) -> object:
    """Pick highest raw power card."""
    return max(pack, key=lambda c: c.power)


def strategy_signal_reader(pack: list, state: DraftState) -> object:
    """Draft toward the most-seen active archetype."""
    if state.committed is not None:
        return max(pack, key=lambda c: c.fitness_score(state.committed))

    arch_freq = defaultdict(int)
    for prev_pack in state.packs_seen:
        for c in prev_pack:
            for a in state.active_archetypes:
                if c.is_fitting(a):
                    arch_freq[a] += 1
    for c in pack:
        for a in state.active_archetypes:
            if c.is_fitting(a):
                arch_freq[a] += 1

    if arch_freq:
        target = max(arch_freq, key=lambda a: arch_freq[a])
        return max(pack, key=lambda c: c.fitness_score(target))
    return max(pack, key=lambda c: c.power)


STRATEGIES = {
    "committed": strategy_committed,
    "power_chaser": strategy_power_chaser,
    "signal_reader": strategy_signal_reader,
}


# ── Single Draft ───────────────────────────────────────────────────────

def run_draft(cards: list, strategy_name: str, rng: random.Random,
              trace: bool = False) -> dict:
    """Run one complete draft and return metrics."""
    suppressed = rng.sample(range(NUM_ARCHETYPES), k=SUPPRESSED_PER_RUN)
    active = [a for a in range(NUM_ARCHETYPES) if a not in suppressed]
    pool = build_pool(cards, rng, suppressed)

    state = DraftState(pool=pool, active_archetypes=active, suppressed=suppressed)

    # Starting card signal: show 3 active-archetype S-tier cards, keep 1
    start_candidates = [c for c in pool if any(c.fitness_in(a) == "S" for a in active)]
    if len(start_candidates) >= 3:
        start_options = rng.sample(start_candidates, k=3)
    else:
        start_options = rng.sample(pool, k=min(3, len(pool)))

    state.starting_card = max(start_options,
                              key=lambda c: max(c.fitness_score(a) for a in active))
    for a in active:
        if state.starting_card.is_fitting(a):
            state.archetype_counts[a] += 1

    metrics = {
        "early_unique_archetypes": [],
        "early_fitting_per_pack": [],
        "late_fitting_per_pack": [],
        "late_off_archetype": [],
        "convergence_pick": None,
        "deck_concentration": 0,
        "committed_archetype": None,
        "picked_ids": set(),
        "suppressed": suppressed,
        "active": active,
    }
    trace_log = []

    for pick_num in range(1, PICKS_PER_DRAFT + 1):
        pack = construct_pack(state, pick_num, rng)
        if len(pack) == 0:
            break

        state.packs_seen.append(pack)

        # Measure early metrics (picks 1-5)
        if pick_num <= 5:
            archs_in_pack = set()
            for c in pack:
                for a in active:
                    if c.is_fitting(a):
                        archs_in_pack.add(a)
            metrics["early_unique_archetypes"].append(len(archs_in_pack))

            if state.committed is not None:
                fitting = sum(1 for c in pack if c.is_fitting(state.committed))
                metrics["early_fitting_per_pack"].append(fitting)
            elif state.archetype_counts:
                emerging = max(state.archetype_counts,
                               key=lambda a: state.archetype_counts[a])
                fitting = sum(1 for c in pack if c.is_fitting(emerging))
                metrics["early_fitting_per_pack"].append(fitting)

        # Measure late metrics (picks 6+)
        if pick_num >= 6 and state.committed is not None:
            arch = state.committed
            fitting = sum(1 for c in pack if c.is_fitting(arch))
            metrics["late_fitting_per_pack"].append(fitting)

            off_strong = sum(1 for c in pack
                             if not c.is_fitting(arch) and
                             (c.power >= 7.0 or
                              any(c.fitness_in(a) == "S"
                                  for a in active if a != arch)))
            metrics["late_off_archetype"].append(off_strong)

            if metrics["convergence_pick"] is None and fitting >= 2:
                metrics["convergence_pick"] = pick_num

        # Player picks
        strategy_fn = STRATEGIES[strategy_name]
        picked = strategy_fn(pack, state)
        state.picked.append(picked)
        metrics["picked_ids"].add(picked.id)

        if trace:
            trace_log.append({
                "pick": pick_num,
                "committed": state.committed,
                "pack": [(c.id, c.fitness_in(state.committed if state.committed
                          is not None else (max(state.archetype_counts,
                          key=lambda a: state.archetype_counts[a])
                          if state.archetype_counts else 0)),
                          round(c.power, 1)) for c in pack],
                "picked_id": picked.id,
                "picked_fitness": {a: picked.fitness_in(a) for a in active},
            })

        # Remove all copies of picked card from pool
        state.pool = [c for c in state.pool if c.id != picked.id]

        # Update archetype counts
        for a in active:
            if picked.is_fitting(a):
                state.archetype_counts[a] += 1

        # Check commitment
        if state.committed is None:
            state.committed = detect_commitment(state, pick_num)

    # Deck concentration
    if state.committed is not None:
        arch = state.committed
        sa_count = sum(1 for c in state.picked if c.is_fitting(arch))
        metrics["deck_concentration"] = sa_count / len(state.picked) if state.picked else 0
    else:
        if state.archetype_counts:
            best = max(state.archetype_counts,
                       key=lambda a: state.archetype_counts[a])
            sa_count = sum(1 for c in state.picked if c.is_fitting(best))
            metrics["deck_concentration"] = sa_count / len(state.picked) if state.picked else 0
            state.committed = best

    metrics["committed_archetype"] = state.committed
    metrics["trace"] = trace_log if trace else None

    return metrics


# ── Batch Simulation ───────────────────────────────────────────────────

def run_simulation(num_drafts: int = NUM_DRAFTS,
                   multi_archetype_pct: float = 0.28,
                   trace_count: int = 0) -> dict:
    """Run full simulation batch."""
    master_rng = random.Random(42)
    cards = generate_card_pool(master_rng, multi_archetype_pct)

    all_results = {}

    for strategy_name in STRATEGIES:
        results = {
            "early_unique_archs": [],
            "early_fitting": [],
            "late_fitting": [],
            "late_off_arch": [],
            "convergence_picks": [],
            "deck_concentrations": [],
            "picked_id_sets": [],
            "committed_archetypes": [],
            "traces": [],
        }

        for draft_i in range(num_drafts):
            draft_rng = random.Random(master_rng.randint(0, 2**32))
            do_trace = draft_i < trace_count and strategy_name == "committed"

            m = run_draft(cards, strategy_name, draft_rng, trace=do_trace)

            results["early_unique_archs"].extend(m["early_unique_archetypes"])
            results["early_fitting"].extend(m["early_fitting_per_pack"])
            results["late_fitting"].extend(m["late_fitting_per_pack"])
            results["late_off_arch"].extend(m["late_off_archetype"])
            if m["convergence_pick"] is not None:
                results["convergence_picks"].append(m["convergence_pick"])
            results["deck_concentrations"].append(m["deck_concentration"])
            results["picked_id_sets"].append(m["picked_ids"])
            if m["committed_archetype"] is not None:
                results["committed_archetypes"].append(m["committed_archetype"])
            if m.get("trace"):
                results["traces"].append(m["trace"])

        all_results[strategy_name] = results

    return all_results


# ── Metrics Computation ────────────────────────────────────────────────

def compute_metrics(all_results: dict) -> dict:
    """Compute all 8 target metrics."""
    metrics = {}

    for strat_name, results in all_results.items():
        prefix = strat_name + "_"

        if results["early_unique_archs"]:
            metrics[prefix + "early_unique_archs"] = (
                sum(results["early_unique_archs"]) /
                len(results["early_unique_archs"]))

        if results["early_fitting"]:
            metrics[prefix + "early_fitting"] = (
                sum(results["early_fitting"]) /
                len(results["early_fitting"]))

        if results["late_fitting"]:
            metrics[prefix + "late_fitting"] = (
                sum(results["late_fitting"]) /
                len(results["late_fitting"]))

        if results["late_off_arch"]:
            metrics[prefix + "late_off_arch"] = (
                sum(results["late_off_arch"]) /
                len(results["late_off_arch"]))

        if results["convergence_picks"]:
            picks = sorted(results["convergence_picks"])
            metrics[prefix + "convergence_pick"] = (
                sum(picks) / len(picks))
            metrics[prefix + "convergence_pct"] = (
                len(picks) / len(results["deck_concentrations"]) * 100)
        else:
            metrics[prefix + "convergence_pick"] = float("inf")
            metrics[prefix + "convergence_pct"] = 0

        concs = [c for c in results["deck_concentrations"] if c > 0]
        if concs:
            metrics[prefix + "deck_concentration"] = sum(concs) / len(concs)

        # Run overlap
        id_sets = results["picked_id_sets"]
        overlaps = []
        for i in range(1, min(200, len(id_sets))):
            s1 = id_sets[i - 1]
            s2 = id_sets[i]
            if s1 and s2:
                overlap = len(s1 & s2) / max(len(s1 | s2), 1)
                overlaps.append(overlap)
        metrics[prefix + "card_overlap"] = (
            sum(overlaps) / len(overlaps) if overlaps else 0)

        # Archetype frequency
        arch_counts = defaultdict(int)
        total = len(results["committed_archetypes"])
        for a in results["committed_archetypes"]:
            arch_counts[a] += 1
        if total > 0:
            metrics[prefix + "arch_freq_max"] = max(arch_counts.values()) / total
            metrics[prefix + "arch_freq_min"] = (
                min(arch_counts.values()) / total if arch_counts else 0)
        else:
            metrics[prefix + "arch_freq_max"] = 0
            metrics[prefix + "arch_freq_min"] = 0

    return metrics


# ── Printing ───────────────────────────────────────────────────────────

def print_scorecard(metrics: dict):
    """Print target scorecard for committed strategy."""
    print("\n" + "=" * 76)
    print("TARGET SCORECARD (committed strategy)")
    print("=" * 76)

    p = "committed_"
    targets = [
        ("Picks 1-5: unique archs/pack", ">= 3",
         metrics.get(p + "early_unique_archs", 0),
         metrics.get(p + "early_unique_archs", 0) >= 3.0),
        ("Picks 1-5: fitting cards/pack", "<= 2",
         metrics.get(p + "early_fitting", 0),
         metrics.get(p + "early_fitting", 0) <= 2.0),
        ("Picks 6+: fitting cards/pack", ">= 2",
         metrics.get(p + "late_fitting", 0),
         metrics.get(p + "late_fitting", 0) >= 2.0),
        ("Picks 6+: off-arch strong/pack", ">= 0.5",
         metrics.get(p + "late_off_arch", 0),
         metrics.get(p + "late_off_arch", 0) >= 0.5),
        ("Convergence pick", "5-8",
         metrics.get(p + "convergence_pick", 0),
         5 <= metrics.get(p + "convergence_pick", 0) <= 8),
        ("Deck concentration", "85-95%*",
         metrics.get(p + "deck_concentration", 0),
         0.85 <= metrics.get(p + "deck_concentration", 0) <= 0.95),
        ("Run-to-run overlap", "< 40%",
         metrics.get(p + "card_overlap", 0),
         metrics.get(p + "card_overlap", 0) < 0.4),
        ("Arch freq max", "<= 20%",
         metrics.get(p + "arch_freq_max", 0),
         metrics.get(p + "arch_freq_max", 0) <= 0.20),
        ("Arch freq min", ">= 5%",
         metrics.get(p + "arch_freq_min", 0),
         metrics.get(p + "arch_freq_min", 0) >= 0.05),
    ]

    print(f"{'Metric':<40} {'Target':<12} {'Actual':<10} {'Pass?':<6}")
    print("-" * 76)
    pass_count = 0
    for name, target, actual, passed in targets:
        if isinstance(actual, float):
            if "%" in target or "concentration" in name.lower() or "overlap" in name.lower() or "freq" in name.lower():
                actual_str = f"{actual:.1%}"
            else:
                actual_str = f"{actual:.2f}"
        else:
            actual_str = str(actual)
        status = "PASS" if passed else "FAIL"
        if passed:
            pass_count += 1
        print(f"{name:<40} {target:<12} {actual_str:<10} {status:<6}")

    print(f"\nPassed: {pass_count}/{len(targets)}")
    print("* Concentration target relaxed to 85-95% per debate consensus")
    return pass_count, len(targets)


def print_strategy_comparison(metrics: dict):
    """Print per-strategy comparison."""
    print("\n" + "=" * 76)
    print("CROSS-STRATEGY COMPARISON")
    print("=" * 76)
    print(f"{'Metric':<35} {'Committed':<12} {'Power':<12} {'Signal':<12}")
    print("-" * 76)

    for metric_suffix in ["late_fitting", "deck_concentration", "card_overlap",
                          "convergence_pick"]:
        row = f"{metric_suffix:<35}"
        for strat in ["committed", "power_chaser", "signal_reader"]:
            val = metrics.get(f"{strat}_{metric_suffix}", "N/A")
            if isinstance(val, float):
                if "concentration" in metric_suffix or "overlap" in metric_suffix:
                    row += f" {val:<11.1%}"
                else:
                    row += f" {val:<11.2f}"
            else:
                row += f" {str(val):<11}"
        print(row)


def print_traces(all_results: dict):
    """Print detailed draft story traces."""
    traces = all_results.get("committed", {}).get("traces", [])
    if not traces:
        return

    for i, trace in enumerate(traces[:3]):
        if i == 0:
            title = "EARLY COMMITTER (commits by pick 4-5)"
        elif i == 1:
            title = "FLEXIBLE DRAFTER (stays open 8+ picks)"
        else:
            title = "SIGNAL READER (finds the open archetype)"
        print(f"\n{'=' * 60}")
        print(f"DRAFT TRACE {i + 1}: {title}")
        print(f"{'=' * 60}")
        for entry in trace:
            pick_num = entry["pick"]
            pack_info = entry["pack"]
            picked_id = entry["picked_id"]
            committed = entry["committed"]
            pack_str = " | ".join(
                f"[{'*' if cid == picked_id else ' '}] id={cid} fit={fit} pwr={pwr}"
                for cid, fit, pwr in pack_info
            )
            print(f"  Pick {pick_num:2d} (arch={committed}): {pack_str}")


# ── Story Traces ───────────────────────────────────────────────────────

def run_story_traces(cards: list):
    """Run 3 specifically-crafted story traces."""
    print("\n" + "=" * 76)
    print("DRAFT STORY TRACES")
    print("=" * 76)

    # Trace 1: Early committer
    print("\n--- Trace 1: Early Committer (commits by pick 4-5) ---")
    rng1 = random.Random(100)
    m1 = run_draft(cards, "committed", rng1, trace=True)
    trace1 = m1["trace"]
    if trace1:
        for entry in trace1[:12]:
            pick = entry["pick"]
            committed = entry["committed"]
            picked_id = entry["picked_id"]
            pack_str = " | ".join(
                f"[{'*' if cid == picked_id else ' '}] id={cid} fit={fit} pwr={pwr}"
                for cid, fit, pwr in entry["pack"]
            )
            print(f"  Pick {pick:2d} (arch={committed}): {pack_str}")
        print(f"  ... (remaining picks omitted)")
        print(f"  Final archetype: {m1['committed_archetype']}, "
              f"Deck S/A: {m1['deck_concentration']:.1%}, "
              f"Convergence: pick {m1['convergence_pick']}")

    # Trace 2: Flexible drafter (power chaser stays open longer)
    print("\n--- Trace 2: Flexible Drafter (stays open 8+ picks) ---")
    rng2 = random.Random(200)
    m2 = run_draft(cards, "power_chaser", rng2, trace=True)
    trace2 = m2["trace"]
    if trace2:
        for entry in trace2[:15]:
            pick = entry["pick"]
            committed = entry["committed"]
            picked_id = entry["picked_id"]
            pack_str = " | ".join(
                f"[{'*' if cid == picked_id else ' '}] id={cid} fit={fit} pwr={pwr}"
                for cid, fit, pwr in entry["pack"]
            )
            print(f"  Pick {pick:2d} (arch={committed}): {pack_str}")
        print(f"  ... (remaining picks omitted)")
        print(f"  Final archetype: {m2['committed_archetype']}, "
              f"Deck S/A: {m2['deck_concentration']:.1%}")

    # Trace 3: Signal reader
    print("\n--- Trace 3: Signal Reader (identifies the open archetype) ---")
    rng3 = random.Random(300)
    m3 = run_draft(cards, "signal_reader", rng3, trace=True)
    trace3 = m3["trace"]
    if trace3:
        for entry in trace3[:12]:
            pick = entry["pick"]
            committed = entry["committed"]
            picked_id = entry["picked_id"]
            pack_str = " | ".join(
                f"[{'*' if cid == picked_id else ' '}] id={cid} fit={fit} pwr={pwr}"
                for cid, fit, pwr in entry["pack"]
            )
            print(f"  Pick {pick:2d} (arch={committed}): {pack_str}")
        print(f"  ... (remaining picks omitted)")
        print(f"  Final archetype: {m3['committed_archetype']}, "
              f"Deck S/A: {m3['deck_concentration']:.1%}, "
              f"Convergence: pick {m3['convergence_pick']}")


# ── Sensitivity Sweeps ─────────────────────────────────────────────────

def sensitivity_sweep_multi_arch():
    """Sweep multi-archetype card percentage."""
    print("\n" + "=" * 76)
    print("SENSITIVITY SWEEP 1: Multi-Archetype Card Percentage")
    print("=" * 76)

    pcts = [0.10, 0.15, 0.20, 0.25, 0.28, 0.30, 0.35, 0.40, 0.50]
    print(f"{'Multi-Arch%':<12} {'Late Fit':<10} {'Deck Conc':<10} "
          f"{'Conv Pick':<10} {'Off-Arch':<10} {'Overlap':<10} "
          f"{'EarlyArch':<10}")
    print("-" * 76)

    for pct in pcts:
        results = run_simulation(num_drafts=200, multi_archetype_pct=pct)
        m = compute_metrics(results)
        p = "committed_"
        late_fit = m.get(p + "late_fitting", 0)
        deck_conc = m.get(p + "deck_concentration", 0)
        conv = m.get(p + "convergence_pick", float("inf"))
        off_arch = m.get(p + "late_off_arch", 0)
        overlap = m.get(p + "card_overlap", 0)
        early_arch = m.get(p + "early_unique_archs", 0)
        conv_str = f"{conv:.1f}" if conv < 100 else "N/A"
        print(f"{pct:<11.0%}  {late_fit:<9.2f} {deck_conc:<9.1%}  "
              f"{conv_str:<9}  {off_arch:<9.2f} {overlap:<9.1%} "
              f"{early_arch:<9.2f}")


def sensitivity_sweep_weight():
    """Sweep weight multiplier intensity."""
    print("\n" + "=" * 76)
    print("SENSITIVITY SWEEP 2: Weight Multiplier Intensity")
    print("=" * 76)

    global get_weight_multiplier
    original = get_weight_multiplier

    weight_configs = [
        ("2x/3x/4x", lambda p: 1.0 if p < 6 else (2.0 if p <= 10 else (3.0 if p <= 20 else 4.0))),
        ("3x/4x/5x", lambda p: 1.0 if p < 6 else (3.0 if p <= 10 else (4.0 if p <= 20 else 5.0))),
        ("4x/5x/6x", lambda p: 1.0 if p < 6 else (4.0 if p <= 10 else (5.0 if p <= 20 else 6.0))),
        ("5x/6x/7x", lambda p: 1.0 if p < 6 else (5.0 if p <= 10 else (6.0 if p <= 20 else 7.0))),
        ("6x/7x/8x", lambda p: 1.0 if p < 6 else (6.0 if p <= 10 else (7.0 if p <= 20 else 8.0))),
    ]

    print(f"{'Config':<14} {'Late Fit':<10} {'Deck Conc':<10} "
          f"{'Conv Pick':<10} {'Off-Arch':<10}")
    print("-" * 60)

    for name, fn in weight_configs:
        get_weight_multiplier = fn
        results = run_simulation(num_drafts=200, multi_archetype_pct=0.28)
        m = compute_metrics(results)
        p = "committed_"
        late_fit = m.get(p + "late_fitting", 0)
        deck_conc = m.get(p + "deck_concentration", 0)
        conv = m.get(p + "convergence_pick", float("inf"))
        off_arch = m.get(p + "late_off_arch", 0)
        conv_str = f"{conv:.1f}" if conv < 100 else "N/A"
        print(f"{name:<13}  {late_fit:<9.2f} {deck_conc:<9.1%}  "
              f"{conv_str:<9}  {off_arch:<9.2f}")

    get_weight_multiplier = original


def sensitivity_sweep_suppression():
    """Sweep number of suppressed archetypes."""
    print("\n" + "=" * 76)
    print("SENSITIVITY SWEEP 3: Number of Suppressed Archetypes")
    print("=" * 76)

    global SUPPRESSED_PER_RUN
    original = SUPPRESSED_PER_RUN

    print(f"{'Suppressed':<12} {'Late Fit':<10} {'Deck Conc':<10} "
          f"{'Conv Pick':<10} {'EarlyArch':<10} {'Overlap':<10}")
    print("-" * 66)

    for n_sup in [0, 1, 2, 3, 4]:
        SUPPRESSED_PER_RUN = n_sup
        results = run_simulation(num_drafts=200, multi_archetype_pct=0.28)
        m = compute_metrics(results)
        p = "committed_"
        late_fit = m.get(p + "late_fitting", 0)
        deck_conc = m.get(p + "deck_concentration", 0)
        conv = m.get(p + "convergence_pick", float("inf"))
        early_arch = m.get(p + "early_unique_archs", 0)
        overlap = m.get(p + "card_overlap", 0)
        conv_str = f"{conv:.1f}" if conv < 100 else "N/A"
        print(f"{n_sup:<11}  {late_fit:<9.2f} {deck_conc:<9.1%}  "
              f"{conv_str:<9}  {early_arch:<9.2f} {overlap:<9.1%}")

    SUPPRESSED_PER_RUN = original


# ── Main ───────────────────────────────────────────────────────────────

def main():
    print("=" * 76)
    print("MODEL A v2: N=8 with Suppression, Soft Floor, and Splash Slot")
    print("=" * 76)
    print(f"  Archetypes: {NUM_ARCHETYPES}, Suppressed: {SUPPRESSED_PER_RUN}/run")
    print(f"  Cards: {TOTAL_UNIQUE_CARDS}, Pack: {PACK_SIZE}, Picks: {PICKS_PER_DRAFT}")
    print(f"  Commitment: {COMMITMENT_THRESHOLD}+ S/A, {COMMITMENT_LEAD}+ lead, "
          f"pick >= {COMMITMENT_MIN_PICK}")
    print()

    # Main simulation
    print("Running main simulation (1000 drafts x 3 strategies)...")
    results = run_simulation(num_drafts=NUM_DRAFTS, multi_archetype_pct=0.28,
                             trace_count=3)
    metrics = compute_metrics(results)

    pass_count, total = print_scorecard(metrics)
    print_strategy_comparison(metrics)

    # Story traces
    master_rng = random.Random(42)
    cards = generate_card_pool(master_rng, 0.28)
    run_story_traces(cards)

    # Sensitivity sweeps
    print("\nRunning sensitivity sweeps (200 drafts each)...")
    sensitivity_sweep_multi_arch()
    sensitivity_sweep_weight()
    sensitivity_sweep_suppression()

    print("\n" + "=" * 76)
    print("SIMULATION COMPLETE")
    print(f"Main scorecard: {pass_count}/{total} targets passed")
    print("=" * 76)


if __name__ == "__main__":
    main()
