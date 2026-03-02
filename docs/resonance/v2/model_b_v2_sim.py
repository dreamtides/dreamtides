#!/usr/bin/env python3
"""
Model B v2 Simulation: Clustered 8 with Suppression and Soft Floor

Revised from Round 2 (N=10) based on Round 3 debate findings:
- N=8 archetypes (down from 10)
- 2 suppressed per run (from Model D)
- ~27% multi-archetype cards (down from 62%)
- Soft floor guarantee (debate consensus)
- Delayed commitment detection (pick >= 6 + clear lead)
- Clustered neighbor topology (retained from original Model B)
- No depletion (debate consensus: hardest to explain, least validated)
"""

import random
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from typing import Optional

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

NUM_ARCHETYPES = 8
TOTAL_UNIQUE_CARDS = 360
PICKS_PER_DRAFT = 30
PACK_SIZE = 4
NUM_DRAFTS = 1000
SUPPRESSED_PER_RUN = 2
SEED = 42

# Rarity distribution
RARITY_DIST = {"common": 0.55, "uncommon": 0.25, "rare": 0.15, "legendary": 0.05}
RARITY_COPIES = {"common": 4, "uncommon": 3, "rare": 2, "legendary": 1}

# Default card type distribution (~27% multi-archetype)
DEFAULT_CARD_TYPE_DIST = {
    "narrow_specialist": 0.45,
    "specialist_splash": 0.25,
    "multi_star": 0.08,
    "broad_generalist": 0.12,
    "universal_star": 0.03,
    "pure_filler": 0.07,
}

TIER_VALUES = {"S": 5, "A": 4, "B": 2, "C": 1, "F": 0}

# Neighbor topology: ring of 8 archetypes
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


# ---------------------------------------------------------------------------
# Weight ramp for committed archetype
# ---------------------------------------------------------------------------

def get_weight_multiplier(pick_num):
    if pick_num <= 5:
        return 1.0
    elif pick_num <= 10:
        return 5.0
    elif pick_num <= 20:
        return 6.0
    else:
        return 7.0


# ---------------------------------------------------------------------------
# Card data structure
# ---------------------------------------------------------------------------

@dataclass
class SimCard:
    id: int
    rarity: str
    power: float
    archetype_fitness: dict  # archetype_id -> tier string (S/A/B/C/F)
    card_type: str

    def fitness_in(self, arch):
        return self.archetype_fitness.get(arch, "F")

    def is_fitting(self, arch):
        return self.fitness_in(arch) in ("S", "A")

    def fitting_archetypes(self):
        return {a for a, t in self.archetype_fitness.items() if t in ("S", "A")}

    def s_tier_archetypes(self):
        return {a for a, t in self.archetype_fitness.items() if t == "S"}

    def fitness_score(self, arch):
        return TIER_VALUES[self.fitness_in(arch)] + self.power * 0.1

    def best_archetype(self):
        best_a, best_v = 0, -1
        for a, t in self.archetype_fitness.items():
            v = TIER_VALUES[t]
            if v > best_v:
                best_v = v
                best_a = a
        return best_a


# ---------------------------------------------------------------------------
# Card pool generation
# ---------------------------------------------------------------------------

def generate_card_pool(rng, card_type_dist=None):
    """Generate 360 unique cards with per-archetype fitness scores."""
    if card_type_dist is None:
        card_type_dist = DEFAULT_CARD_TYPE_DIST

    cards = []
    s_tier_counts = defaultdict(int)

    def pick_primary():
        min_count = min(s_tier_counts.get(a, 0) for a in range(NUM_ARCHETYPES))
        candidates = [a for a in range(NUM_ARCHETYPES)
                      if s_tier_counts.get(a, 0) == min_count]
        return rng.choice(candidates)

    # Assign rarities
    rarities = []
    for rarity_name, frac in RARITY_DIST.items():
        rarities.extend([rarity_name] * round(TOTAL_UNIQUE_CARDS * frac))
    while len(rarities) < TOTAL_UNIQUE_CARDS:
        rarities.append("common")
    while len(rarities) > TOTAL_UNIQUE_CARDS:
        rarities.pop()
    rng.shuffle(rarities)

    # Compute counts per card type
    type_counts = {}
    remaining = TOTAL_UNIQUE_CARDS
    for ctype, frac in card_type_dist.items():
        count = round(TOTAL_UNIQUE_CARDS * frac)
        type_counts[ctype] = count
        remaining -= count
    # Adjust narrow_specialist to absorb rounding
    type_counts["narrow_specialist"] += remaining

    card_id = 0

    # 1. Narrow specialists
    for _ in range(type_counts.get("narrow_specialist", 0)):
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
        cards.append(SimCard(card_id, rarities[card_id], power, fitness, "narrow_specialist"))
        card_id += 1

    # 2. Specialists with splash
    for _ in range(type_counts.get("specialist_splash", 0)):
        primary = pick_primary()
        s_tier_counts[primary] += 1
        fitness = {}
        # A-tier in 1-2 neighbors (80% chance neighbor, 20% non-neighbor)
        a_count = rng.choice([1, 2])
        a_targets = []
        for _ in range(a_count):
            if rng.random() < 0.8:
                a_targets.append(rng.choice(NEIGHBORS[primary]))
            else:
                non_neighbors = [x for x in range(NUM_ARCHETYPES)
                                 if x != primary and x not in NEIGHBORS[primary]]
                if non_neighbors:
                    a_targets.append(rng.choice(non_neighbors))
                else:
                    a_targets.append(rng.choice(NEIGHBORS[primary]))
        a_targets = list(set(a_targets))  # dedupe

        for a in range(NUM_ARCHETYPES):
            if a == primary:
                fitness[a] = "S"
            elif a in a_targets:
                fitness[a] = "A"
            elif a in NEIGHBORS[primary] and rng.random() < 0.25:
                fitness[a] = "B"
            elif rng.random() < 0.2:
                fitness[a] = "C"
            else:
                fitness[a] = rng.choice(["C", "F"])
        power = rng.uniform(4.0, 7.0)
        cards.append(SimCard(card_id, rarities[card_id], power, fitness, "specialist_splash"))
        card_id += 1

    # 3. Multi-archetype stars
    for _ in range(type_counts.get("multi_star", 0)):
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
        cards.append(SimCard(card_id, rarities[card_id], power, fitness, "multi_star"))
        card_id += 1

    # 4. Broad generalists
    for _ in range(type_counts.get("broad_generalist", 0)):
        fitness = {}
        a_count = rng.randint(2, 3)
        b_count = rng.randint(3, 4)
        a_targets = rng.sample(range(NUM_ARCHETYPES), k=a_count)
        remaining_archs = [a for a in range(NUM_ARCHETYPES) if a not in a_targets]
        b_targets = rng.sample(remaining_archs, k=min(b_count, len(remaining_archs)))
        for a in range(NUM_ARCHETYPES):
            if a in a_targets:
                fitness[a] = "A"
            elif a in b_targets:
                fitness[a] = "B"
            else:
                fitness[a] = "C"
        power = rng.uniform(5.0, 7.5)
        cards.append(SimCard(card_id, rarities[card_id], power, fitness, "broad_generalist"))
        card_id += 1

    # 5. Universal stars
    for _ in range(type_counts.get("universal_star", 0)):
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
        cards.append(SimCard(card_id, rarities[card_id], power, fitness, "universal_star"))
        card_id += 1

    # 6. Pure filler
    for _ in range(type_counts.get("pure_filler", 0)):
        fitness = {}
        b_count = rng.randint(2, 3)
        b_targets = rng.sample(range(NUM_ARCHETYPES), k=b_count)
        for a in range(NUM_ARCHETYPES):
            if a in b_targets:
                fitness[a] = "B"
            else:
                fitness[a] = rng.choice(["C", "F"])
        power = rng.uniform(2.0, 5.0)
        cards.append(SimCard(card_id, rarities[card_id], power, fitness, "pure_filler"))
        card_id += 1

    return cards


# ---------------------------------------------------------------------------
# Pool construction (per-run with suppression)
# ---------------------------------------------------------------------------

def build_run_pool(cards, rng):
    """Build draft pool for a single run. Suppress 2 archetypes by removing
    ~50% of copies whose primary archetype is suppressed."""
    suppressed = set(rng.sample(range(NUM_ARCHETYPES), k=SUPPRESSED_PER_RUN))
    active = [a for a in range(NUM_ARCHETYPES) if a not in suppressed]

    pool = []
    for card in cards:
        copies = RARITY_COPIES[card.rarity]
        primary = card.best_archetype()
        if primary in suppressed:
            # Remove ~50% of copies for suppressed archetype specialists
            copies = max(1, int(copies * 0.5))
        for _ in range(copies):
            pool.append(card)

    rng.shuffle(pool)
    return pool, active, suppressed


# ---------------------------------------------------------------------------
# Commitment detection
# ---------------------------------------------------------------------------

def detect_commitment(picked_cards, pick_num, active_archetypes):
    """Detect commitment: requires pick >= 6, 3+ S/A picks in one archetype,
    and a strict lead over the runner-up (best count > runner-up count)."""
    if pick_num < 6:
        return None

    arch_counts = Counter()
    for card in picked_cards:
        for a in active_archetypes:
            if card.is_fitting(a):
                arch_counts[a] += 1

    if not arch_counts:
        return None

    top_two = arch_counts.most_common(2)
    best_arch, best_count = top_two[0]
    runner_up_count = top_two[1][1] if len(top_two) > 1 else 0

    if best_count >= 3 and best_count > runner_up_count:
        return best_arch
    return None


# ---------------------------------------------------------------------------
# Pack construction
# ---------------------------------------------------------------------------

def build_pack(pool, pick_num, committed_arch, active_archetypes, rng):
    """Build a 4-card pack using adaptive weighted sampling with soft floor."""
    if len(pool) < PACK_SIZE:
        return list(range(len(pool)))

    multiplier = get_weight_multiplier(pick_num)
    apply_bias = committed_arch is not None and pick_num >= 6

    if apply_bias:
        # 3 archetype-weighted slots + 1 splash slot
        selected_indices = []
        used = set()

        # Compute weights for archetype slots
        weights = []
        for i, card in enumerate(pool):
            w = 1.0
            if card.is_fitting(committed_arch):
                w *= multiplier
            weights.append(w)

        # Draw 3 archetype-biased cards
        for _ in range(3):
            avail = [(i, weights[i]) for i in range(len(pool)) if i not in used]
            if not avail:
                break
            total_w = sum(w for _, w in avail)
            if total_w <= 0:
                break
            r = rng.random() * total_w
            cumulative = 0.0
            chosen = avail[0][0]
            for idx, w in avail:
                cumulative += w
                if cumulative >= r:
                    chosen = idx
                    break
            selected_indices.append(chosen)
            used.add(chosen)

        # Draw 1 splash card: prefer high power or S-tier in other active archetype
        splash_candidates = [i for i in range(len(pool))
                             if i not in used and not pool[i].is_fitting(committed_arch)]
        if not splash_candidates:
            splash_candidates = [i for i in range(len(pool)) if i not in used]

        if splash_candidates:
            splash_weights = []
            for i in splash_candidates:
                c = pool[i]
                sw = c.power
                for a in active_archetypes:
                    if a != committed_arch and c.fitness_in(a) == "S":
                        sw += 5.0
                        break
                splash_weights.append(sw)
            total_sw = sum(splash_weights)
            if total_sw > 0:
                r = rng.random() * total_sw
                cumulative = 0.0
                chosen = splash_candidates[0]
                for idx, sw in zip(splash_candidates, splash_weights):
                    cumulative += sw
                    if cumulative >= r:
                        chosen = idx
                        break
                selected_indices.append(chosen)
            elif splash_candidates:
                selected_indices.append(rng.choice(splash_candidates))

        # Soft floor: if 0 fitting cards in the 3 archetype slots, replace weakest
        arch_slot_cards = [pool[i] for i in selected_indices[:3]]
        fitting_count = sum(1 for c in arch_slot_cards if c.is_fitting(committed_arch))
        if fitting_count == 0:
            fitting_pool = [i for i in range(len(pool))
                            if i not in used and pool[i].is_fitting(committed_arch)]
            if fitting_pool:
                replacement = rng.choice(fitting_pool)
                # Replace the weakest archetype slot card
                if selected_indices:
                    weakest_pos = min(range(min(3, len(selected_indices))),
                                      key=lambda j: pool[selected_indices[j]].fitness_score(committed_arch))
                    selected_indices[weakest_pos] = replacement

        return selected_indices
    else:
        # Uniform random for early picks
        return rng.sample(range(len(pool)), k=min(PACK_SIZE, len(pool)))


# ---------------------------------------------------------------------------
# Player strategies
# ---------------------------------------------------------------------------

def strategy_committed(pack_cards, picked_cards, pick_num, committed_arch,
                       active_archetypes, seen_cards):
    """Pick highest fitness in committed archetype. Before commitment,
    pick highest single-archetype fitness among active archetypes."""
    if committed_arch is not None:
        return max(range(len(pack_cards)),
                   key=lambda i: pack_cards[i].fitness_score(committed_arch))
    else:
        def best_active(i):
            return max(pack_cards[i].fitness_score(a) for a in active_archetypes)
        return max(range(len(pack_cards)), key=best_active)


def strategy_power_chaser(pack_cards, picked_cards, pick_num, committed_arch,
                          active_archetypes, seen_cards):
    """Pick highest raw power card."""
    return max(range(len(pack_cards)), key=lambda i: pack_cards[i].power)


def strategy_signal_reader(pack_cards, picked_cards, pick_num, committed_arch,
                           active_archetypes, seen_cards):
    """Track which archetypes appear most frequently and draft toward the open one."""
    if committed_arch is not None:
        return max(range(len(pack_cards)),
                   key=lambda i: pack_cards[i].fitness_score(committed_arch))

    # Count archetype frequency across all seen cards
    arch_freq = Counter()
    for card in seen_cards:
        for a in active_archetypes:
            if card.is_fitting(a):
                arch_freq[a] += 1
    for card in pack_cards:
        for a in active_archetypes:
            if card.is_fitting(a):
                arch_freq[a] += 1

    if not arch_freq:
        return max(range(len(pack_cards)), key=lambda i: pack_cards[i].power)

    target_arch = max(arch_freq, key=lambda a: arch_freq[a])
    return max(range(len(pack_cards)),
               key=lambda i: pack_cards[i].fitness_score(target_arch))


STRATEGIES = {
    "committed": strategy_committed,
    "power_chaser": strategy_power_chaser,
    "signal_reader": strategy_signal_reader,
}


# ---------------------------------------------------------------------------
# Run a single draft
# ---------------------------------------------------------------------------

def run_draft(cards, strategy_name, rng, trace=False):
    """Run one complete draft and return detailed metrics."""
    pool, active, suppressed = build_run_pool(cards, rng)

    picked_cards = []
    seen_cards = []
    committed_arch = None
    pick_details = []

    # Starting card signal: show 3 active-archetype S/A cards, keep best
    starting_candidates = [c for c in pool
                           if any(c.is_fitting(a) for a in active)]
    if len(starting_candidates) >= 3:
        starting_options = rng.sample(starting_candidates, k=3)
    else:
        starting_options = rng.sample(pool, k=min(3, len(pool)))

    starting_card = max(starting_options,
                        key=lambda c: max(c.fitness_score(a) for a in active))
    picked_cards.append(starting_card)
    # Remove starting card from pool
    for i, c in enumerate(pool):
        if c.id == starting_card.id:
            pool.pop(i)
            break

    trace_log = []

    for pick_num in range(1, PICKS_PER_DRAFT + 1):
        if len(pool) < PACK_SIZE:
            break

        # Detect commitment
        committed_arch = detect_commitment(picked_cards, pick_num, active)

        pack_indices = build_pack(pool, pick_num, committed_arch, active, rng)
        pack_cards = [pool[i] for i in pack_indices]
        seen_cards.extend(pack_cards)

        # Apply strategy
        strategy_fn = STRATEGIES[strategy_name]
        choice = strategy_fn(pack_cards, picked_cards, pick_num,
                             committed_arch, active, seen_cards)
        chosen_card = pack_cards[choice]
        picked_cards.append(chosen_card)

        # Measure metrics for this pick
        # Determine the "leading archetype" for early metrics
        arch_counts = Counter()
        for card in picked_cards:
            for a in active:
                if card.is_fitting(a):
                    arch_counts[a] += 1
        leading_arch = arch_counts.most_common(1)[0][0] if arch_counts else None

        # Unique archetypes in pack (S or A in any active archetype)
        archs_in_pack = set()
        for c in pack_cards:
            for a in active:
                if c.is_fitting(a):
                    archs_in_pack.add(a)

        # Fitting cards for leading/committed archetype
        measure_arch = committed_arch if committed_arch is not None else leading_arch
        fitting_count = 0
        if measure_arch is not None:
            fitting_count = sum(1 for c in pack_cards if c.is_fitting(measure_arch))

        # Off-archetype strong cards
        off_arch_strong = 0
        if measure_arch is not None:
            for c in pack_cards:
                if not c.is_fitting(measure_arch):
                    if c.power >= 7.0 or any(c.fitness_in(a) == "S"
                                              for a in active if a != measure_arch):
                        off_arch_strong += 1

        detail = {
            "pick_num": pick_num,
            "pack_archetypes": len(archs_in_pack),
            "fitting_count": fitting_count,
            "off_arch_strong": off_arch_strong,
            "committed_arch": committed_arch,
            "leading_arch": leading_arch,
            "chosen_id": chosen_card.id,
            "chosen_type": chosen_card.card_type,
            "chosen_power": chosen_card.power,
            "chosen_fitness": chosen_card.fitness_in(measure_arch) if measure_arch is not None else "N/A",
        }
        pick_details.append(detail)

        if trace:
            pack_str = " | ".join(
                f"[{'*' if c.id == chosen_card.id else ' '}] id={c.id} "
                f"fit={c.fitness_in(measure_arch) if measure_arch is not None else '?'} "
                f"pwr={c.power:.1f}"
                for c in pack_cards
            )
            print(f"  Pick {pick_num:2d} (arch={committed_arch}): {pack_str}")

        # Remove chosen card from pool
        chosen_pool_idx = pack_indices[choice]
        pool.pop(chosen_pool_idx)

    if trace:
        print(f"  Starting card: id={starting_card.id}, "
              f"archetypes={starting_card.fitting_archetypes()}")
        if committed_arch is not None:
            fitting = sum(1 for c in picked_cards if c.is_fitting(committed_arch))
            print(f"  Final: arch={committed_arch}, "
                  f"{fitting}/{len(picked_cards)} fitting "
                  f"({fitting/len(picked_cards)*100:.0f}%)")
        print(f"  Active: {active}, Suppressed: {list(suppressed)}")

    return {
        "strategy": strategy_name,
        "picked_cards": picked_cards,
        "pick_details": pick_details,
        "active": active,
        "suppressed": suppressed,
        "committed_arch": committed_arch,
    }


# ---------------------------------------------------------------------------
# Aggregate metrics
# ---------------------------------------------------------------------------

def compute_metrics(results):
    """Compute all 8 measurable target metrics from a list of draft results."""
    metrics = {}

    # Filter by strategy for certain metrics
    committed_results = [r for r in results if r["strategy"] == "committed"]
    if not committed_results:
        committed_results = results  # fallback

    # 1. Picks 1-5: unique archetypes per pack
    early_archs = []
    for r in committed_results:
        for d in r["pick_details"]:
            if d["pick_num"] <= 5:
                early_archs.append(d["pack_archetypes"])
    metrics["early_archetypes_per_pack"] = (
        sum(early_archs) / len(early_archs) if early_archs else 0)

    # 2. Picks 1-5: fitting cards per pack (emerging archetype)
    early_fitting = []
    for r in committed_results:
        for d in r["pick_details"]:
            if d["pick_num"] <= 5 and d["leading_arch"] is not None:
                early_fitting.append(d["fitting_count"])
    metrics["early_fitting_per_pack"] = (
        sum(early_fitting) / len(early_fitting) if early_fitting else 0)

    # 3. Picks 6+: fitting cards per pack (committed archetype)
    late_fitting = []
    for r in committed_results:
        for d in r["pick_details"]:
            if d["pick_num"] >= 6 and d["committed_arch"] is not None:
                late_fitting.append(d["fitting_count"])
    metrics["late_fitting_per_pack"] = (
        sum(late_fitting) / len(late_fitting) if late_fitting else 0)

    # 4. Picks 6+: off-archetype strong cards per pack
    late_off_arch = []
    for r in committed_results:
        for d in r["pick_details"]:
            if d["pick_num"] >= 6 and d["committed_arch"] is not None:
                late_off_arch.append(d["off_arch_strong"])
    metrics["late_off_arch_per_pack"] = (
        sum(late_off_arch) / len(late_off_arch) if late_off_arch else 0)

    # 5. Convergence pick: first pick >= 6 where 2+ fitting in pack
    convergence_picks = []
    for r in committed_results:
        for d in r["pick_details"]:
            if d["pick_num"] >= 6 and d["committed_arch"] is not None and d["fitting_count"] >= 2:
                convergence_picks.append(d["pick_num"])
                break
    metrics["convergence_pick"] = (
        sum(convergence_picks) / len(convergence_picks) if convergence_picks else float('inf'))
    metrics["convergence_rate"] = (
        len(convergence_picks) / len(committed_results) * 100 if committed_results else 0)

    # 6. Deck archetype concentration
    concentrations = []
    for r in committed_results:
        if r["committed_arch"] is not None:
            arch = r["committed_arch"]
            fitting = sum(1 for c in r["picked_cards"] if c.is_fitting(arch))
            conc = fitting / len(r["picked_cards"]) * 100 if r["picked_cards"] else 0
            concentrations.append(conc)
    metrics["deck_concentration"] = (
        sum(concentrations) / len(concentrations) if concentrations else 0)

    # Power-chaser concentration
    power_results = [r for r in results if r["strategy"] == "power_chaser"]
    if power_results:
        pc_concs = []
        for r in power_results:
            if r["committed_arch"] is not None:
                arch = r["committed_arch"]
                fitting = sum(1 for c in r["picked_cards"] if c.is_fitting(arch))
                pc_concs.append(fitting / len(r["picked_cards"]) * 100 if r["picked_cards"] else 0)
            else:
                # For power chasers without commitment, use best archetype
                arch_counts = Counter()
                for c in r["picked_cards"]:
                    for a in r["active"]:
                        if c.is_fitting(a):
                            arch_counts[a] += 1
                if arch_counts:
                    best = arch_counts.most_common(1)[0][0]
                    fitting = sum(1 for c in r["picked_cards"] if c.is_fitting(best))
                    pc_concs.append(fitting / len(r["picked_cards"]) * 100)
        metrics["power_chaser_concentration"] = (
            sum(pc_concs) / len(pc_concs) if pc_concs else 0)

    # 7. Run-to-run variety (card overlap)
    overlaps = []
    for strat in STRATEGIES:
        strat_results = [r for r in results if r["strategy"] == strat]
        id_sets = [set(c.id for c in r["picked_cards"]) for r in strat_results]
        for i in range(0, min(100, len(id_sets) - 1), 2):
            if id_sets[i] and id_sets[i + 1]:
                overlap = len(id_sets[i] & id_sets[i + 1]) / max(len(id_sets[i] | id_sets[i + 1]), 1) * 100
                overlaps.append(overlap)
    metrics["run_overlap"] = sum(overlaps) / len(overlaps) if overlaps else 0

    # 8. Archetype frequency across runs
    arch_freq = Counter()
    total_committed = 0
    for r in committed_results:
        if r["committed_arch"] is not None:
            arch_freq[r["committed_arch"]] += 1
            total_committed += 1
    if total_committed > 0:
        freqs = {a: arch_freq.get(a, 0) / total_committed * 100
                 for a in range(NUM_ARCHETYPES)}
        metrics["arch_max_freq"] = max(freqs.values())
        metrics["arch_min_freq"] = min(freqs.values())
        metrics["arch_freq_detail"] = freqs
    else:
        metrics["arch_max_freq"] = 0
        metrics["arch_min_freq"] = 0
        metrics["arch_freq_detail"] = {}

    return metrics


# ---------------------------------------------------------------------------
# Sensitivity analysis
# ---------------------------------------------------------------------------

def make_multi_arch_dist(target_pct):
    """Create a card type distribution targeting a specific % of multi-archetype cards.
    Multi-archetype = specialist_splash + multi_star + broad_generalist + universal_star."""
    # Base narrow + filler = 1 - target_pct
    non_multi = 1.0 - target_pct
    narrow = max(0.1, non_multi - 0.07)
    filler = non_multi - narrow

    # Distribute multi-arch budget
    splash = target_pct * 0.55
    star = target_pct * 0.15
    generalist = target_pct * 0.25
    universal = target_pct * 0.05

    return {
        "narrow_specialist": narrow,
        "specialist_splash": splash,
        "multi_star": star,
        "broad_generalist": generalist,
        "universal_star": universal,
        "pure_filler": filler,
    }


def run_sensitivity_multi_arch():
    """Sweep multi-archetype card percentage."""
    print("\n" + "=" * 75)
    print("SENSITIVITY: Multi-Archetype Card Percentage")
    print("=" * 75)

    pcts = [0.10, 0.15, 0.20, 0.27, 0.35, 0.45]
    print(f"{'Multi-Arch %':<14} {'Late Fit':<10} {'Early Fit':<10} "
          f"{'Deck Conc':<10} {'Conv Pick':<10} {'Conv Rate':<10} {'Overlap':<10}")
    print("-" * 75)

    sensitivity_results = {}
    for pct in pcts:
        test_rng = random.Random(SEED + 2000)
        dist = make_multi_arch_dist(pct)
        test_cards = generate_card_pool(test_rng, card_type_dist=dist)

        results = []
        for _ in range(300):
            r = run_draft(test_cards, "committed", test_rng)
            results.append(r)

        m = compute_metrics(results)
        sensitivity_results[pct] = m
        print(f"{pct:<14.0%} {m['late_fitting_per_pack']:<10.2f} "
              f"{m['early_fitting_per_pack']:<10.2f} "
              f"{m['deck_concentration']:<10.1f} "
              f"{m['convergence_pick']:<10.1f} "
              f"{m['convergence_rate']:<10.1f} "
              f"{m['run_overlap']:<10.1f}")

    return sensitivity_results


def run_sensitivity_weight_ramp():
    """Sweep weight multiplier intensity."""
    print("\n" + "=" * 75)
    print("SENSITIVITY: Weight Ramp Intensity")
    print("=" * 75)

    ramps = [
        ("Gentle (2/3/4)", lambda p: 2.0 if p <= 10 else 3.0 if p <= 20 else 4.0),
        ("Moderate (3.5/4.5/5.5)", get_weight_multiplier),
        ("Aggressive (5/6/7)", lambda p: 1.0 if p <= 5 else 5.0 if p <= 10 else 6.0 if p <= 20 else 7.0),
        ("Very aggressive (6/8/10)", lambda p: 1.0 if p <= 5 else 6.0 if p <= 10 else 8.0 if p <= 20 else 10.0),
    ]

    print(f"{'Ramp':<25} {'Late Fit':<10} {'Deck Conc':<10} {'Conv Pick':<10}")
    print("-" * 55)

    # We need to temporarily override the weight function
    global get_weight_multiplier_override
    original_fn = get_weight_multiplier

    for name, ramp_fn in ramps:
        test_rng = random.Random(SEED + 3000)
        test_cards = generate_card_pool(test_rng)

        # Monkey-patch the weight function for this run
        results = []
        for _ in range(300):
            r = run_draft_with_ramp(test_cards, "committed", test_rng, ramp_fn)
            results.append(r)

        m = compute_metrics(results)
        print(f"{name:<25} {m['late_fitting_per_pack']:<10.2f} "
              f"{m['deck_concentration']:<10.1f} "
              f"{m['convergence_pick']:<10.1f}")


def run_draft_with_ramp(cards, strategy_name, rng, ramp_fn, trace=False):
    """Run a draft with a custom weight ramp function."""
    pool, active, suppressed = build_run_pool(cards, rng)

    picked_cards = []
    seen_cards = []
    committed_arch = None
    pick_details = []

    # Starting card
    starting_candidates = [c for c in pool if any(c.is_fitting(a) for a in active)]
    if len(starting_candidates) >= 3:
        starting_options = rng.sample(starting_candidates, k=3)
    else:
        starting_options = rng.sample(pool, k=min(3, len(pool)))
    starting_card = max(starting_options,
                        key=lambda c: max(c.fitness_score(a) for a in active))
    picked_cards.append(starting_card)
    for i, c in enumerate(pool):
        if c.id == starting_card.id:
            pool.pop(i)
            break

    for pick_num in range(1, PICKS_PER_DRAFT + 1):
        if len(pool) < PACK_SIZE:
            break

        committed_arch = detect_commitment(picked_cards, pick_num, active)

        # Custom pack construction with ramp
        multiplier = ramp_fn(pick_num) if (committed_arch is not None and pick_num >= 6) else 1.0
        apply_bias = committed_arch is not None and pick_num >= 6

        if apply_bias:
            selected_indices = []
            used = set()
            weights = []
            for i, card in enumerate(pool):
                w = 1.0
                if card.is_fitting(committed_arch):
                    w *= multiplier
                weights.append(w)

            for _ in range(3):
                avail = [(i, weights[i]) for i in range(len(pool)) if i not in used]
                if not avail:
                    break
                total_w = sum(w for _, w in avail)
                if total_w <= 0:
                    break
                r = rng.random() * total_w
                cumulative = 0.0
                chosen = avail[0][0]
                for idx, w in avail:
                    cumulative += w
                    if cumulative >= r:
                        chosen = idx
                        break
                selected_indices.append(chosen)
                used.add(chosen)

            splash_cands = [i for i in range(len(pool))
                            if i not in used and not pool[i].is_fitting(committed_arch)]
            if not splash_cands:
                splash_cands = [i for i in range(len(pool)) if i not in used]
            if splash_cands:
                selected_indices.append(rng.choice(splash_cands))

            # Soft floor
            arch_cards = [pool[i] for i in selected_indices[:3]]
            if not any(c.is_fitting(committed_arch) for c in arch_cards):
                fit_pool = [i for i in range(len(pool))
                            if i not in used and pool[i].is_fitting(committed_arch)]
                if fit_pool and selected_indices:
                    replacement = rng.choice(fit_pool)
                    weakest = min(range(min(3, len(selected_indices))),
                                  key=lambda j: pool[selected_indices[j]].fitness_score(committed_arch))
                    selected_indices[weakest] = replacement

            pack_indices = selected_indices
        else:
            pack_indices = rng.sample(range(len(pool)), k=min(PACK_SIZE, len(pool)))

        pack_cards = [pool[i] for i in pack_indices]
        seen_cards.extend(pack_cards)

        strategy_fn = STRATEGIES[strategy_name]
        choice = strategy_fn(pack_cards, picked_cards, pick_num,
                             committed_arch, active, seen_cards)
        chosen_card = pack_cards[choice]
        picked_cards.append(chosen_card)

        arch_counts = Counter()
        for card in picked_cards:
            for a in active:
                if card.is_fitting(a):
                    arch_counts[a] += 1
        leading_arch = arch_counts.most_common(1)[0][0] if arch_counts else None
        measure_arch = committed_arch if committed_arch is not None else leading_arch

        archs_in_pack = set()
        for c in pack_cards:
            for a in active:
                if c.is_fitting(a):
                    archs_in_pack.add(a)

        fitting_count = 0
        if measure_arch is not None:
            fitting_count = sum(1 for c in pack_cards if c.is_fitting(measure_arch))

        off_arch_strong = 0
        if measure_arch is not None:
            for c in pack_cards:
                if not c.is_fitting(measure_arch):
                    if c.power >= 7.0 or any(c.fitness_in(a) == "S"
                                              for a in active if a != measure_arch):
                        off_arch_strong += 1

        pick_details.append({
            "pick_num": pick_num,
            "pack_archetypes": len(archs_in_pack),
            "fitting_count": fitting_count,
            "off_arch_strong": off_arch_strong,
            "committed_arch": committed_arch,
            "leading_arch": leading_arch,
            "chosen_id": chosen_card.id,
            "chosen_type": chosen_card.card_type,
            "chosen_power": chosen_card.power,
            "chosen_fitness": chosen_card.fitness_in(measure_arch) if measure_arch is not None else "N/A",
        })

        chosen_pool_idx = pack_indices[choice]
        pool.pop(chosen_pool_idx)

    return {
        "strategy": strategy_name,
        "picked_cards": picked_cards,
        "pick_details": pick_details,
        "active": active,
        "suppressed": suppressed,
        "committed_arch": committed_arch,
    }


def run_sensitivity_suppression():
    """Sweep number of suppressed archetypes."""
    print("\n" + "=" * 75)
    print("SENSITIVITY: Suppression Count")
    print("=" * 75)

    print(f"{'Suppressed':<12} {'Late Fit':<10} {'Deck Conc':<10} "
          f"{'Conv Pick':<10} {'Early Archs':<12} {'Overlap':<10}")
    print("-" * 65)

    global SUPPRESSED_PER_RUN
    original = SUPPRESSED_PER_RUN

    for num_suppressed in [0, 1, 2, 3]:
        SUPPRESSED_PER_RUN = num_suppressed
        test_rng = random.Random(SEED + 4000)
        test_cards = generate_card_pool(test_rng)

        results = []
        for _ in range(300):
            r = run_draft(test_cards, "committed", test_rng)
            results.append(r)

        m = compute_metrics(results)
        print(f"{num_suppressed:<12} {m['late_fitting_per_pack']:<10.2f} "
              f"{m['deck_concentration']:<10.1f} "
              f"{m['convergence_pick']:<10.1f} "
              f"{m['early_archetypes_per_pack']:<12.2f} "
              f"{m['run_overlap']:<10.1f}")

    SUPPRESSED_PER_RUN = original


# ---------------------------------------------------------------------------
# Draft story traces
# ---------------------------------------------------------------------------

def run_story_traces(cards):
    """Produce 3 story traces: early committer, flexible player, signal reader."""
    print("\n" + "=" * 75)
    print("DRAFT STORY TRACE 1: Early Committer (commits by pick 4-5)")
    print("=" * 75)

    # Run drafts until we find one where commitment happens early
    trace_rng = random.Random(SEED + 5000)
    for attempt in range(100):
        r = run_draft(cards, "committed", trace_rng, trace=False)
        # Check if commitment was detected early (by pick 7, since detection requires pick >= 6)
        early_commit = False
        for d in r["pick_details"]:
            if d["committed_arch"] is not None and d["pick_num"] <= 7:
                early_commit = True
                break
        if early_commit:
            # Re-run with trace
            trace_rng2 = random.Random(SEED + 5000 + attempt)
            # Replay
            print(f"  (Found early commit on attempt {attempt + 1})")
            r2 = run_draft(cards, "committed", random.Random(trace_rng.randint(0, 2**32)), trace=True)
            # If this one also commits early, use it
            for d in r2["pick_details"]:
                if d["committed_arch"] is not None and d["pick_num"] <= 7:
                    print_story_summary(r2)
                    break
            else:
                # Retry with trace on original
                print_story_summary(r)
            break

    print("\n" + "=" * 75)
    print("DRAFT STORY TRACE 2: Flexible Player (stays open 8+ picks)")
    print("=" * 75)

    # Power chaser stays flexible by nature
    trace_rng = random.Random(SEED + 6000)
    r = run_draft(cards, "power_chaser", trace_rng, trace=True)
    print_story_summary(r)

    print("\n" + "=" * 75)
    print("DRAFT STORY TRACE 3: Signal Reader (identifies open archetype)")
    print("=" * 75)

    trace_rng = random.Random(SEED + 7000)
    r = run_draft(cards, "signal_reader", trace_rng, trace=True)
    print_story_summary(r)


def print_story_summary(result):
    """Print summary of a draft story."""
    committed = result["committed_arch"]
    active = result["active"]
    suppressed = result["suppressed"]

    print(f"\n  Strategy: {result['strategy']}")
    print(f"  Active archetypes: {active}")
    print(f"  Suppressed archetypes: {list(suppressed)}")

    if committed is not None:
        fitting = sum(1 for c in result["picked_cards"] if c.is_fitting(committed))
        total = len(result["picked_cards"])
        print(f"  Committed to: Archetype {committed}")
        print(f"  Deck: {fitting}/{total} fitting ({fitting/total*100:.0f}%)")

        # Find commitment pick
        for d in result["pick_details"]:
            if d["committed_arch"] is not None:
                print(f"  Committed at: Pick {d['pick_num']}")
                break

        # Average fitting per pack after commitment
        post_commit = [d["fitting_count"] for d in result["pick_details"]
                       if d["committed_arch"] is not None]
        if post_commit:
            print(f"  Avg fitting/pack after commitment: {sum(post_commit)/len(post_commit):.2f}")
    else:
        print("  Never committed to a single archetype")
        total = len(result["picked_cards"])
        types = Counter(c.card_type for c in result["picked_cards"])
        print(f"  Deck size: {total}")
        print(f"  Card types: {dict(types)}")


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    print("=" * 75)
    print("MODEL B v2: Clustered 8 with Suppression and Soft Floor")
    print("=" * 75)
    print(f"  N={NUM_ARCHETYPES} archetypes, {SUPPRESSED_PER_RUN} suppressed per run")
    print(f"  {TOTAL_UNIQUE_CARDS} unique cards, {PICKS_PER_DRAFT} picks, "
          f"{PACK_SIZE} per pack")
    print(f"  Commitment detection: pick >= 6, 3+ S/A, strict lead")
    print()

    rng = random.Random(SEED)
    cards = generate_card_pool(rng)

    # Verify pool composition
    multi_count = sum(1 for c in cards if len(c.fitting_archetypes()) >= 2)
    print(f"Pool: {len(cards)} cards, {multi_count} multi-archetype "
          f"({multi_count/len(cards)*100:.1f}%)")

    for a in range(NUM_ARCHETYPES):
        s_count = sum(1 for c in cards if c.fitness_in(a) == "S")
        sa_count = sum(1 for c in cards if c.is_fitting(a))
        print(f"  Archetype {a}: {s_count} S-tier, {sa_count} S/A-tier")

    type_counts = Counter(c.card_type for c in cards)
    print(f"Card types: {dict(type_counts)}")

    # --- Run 1000 drafts per strategy ---
    all_results = []
    for strategy_name in STRATEGIES:
        print(f"\nRunning 1000 drafts: {strategy_name}...")
        strat_rng = random.Random(SEED + hash(strategy_name) % (2**31))
        for _ in range(NUM_DRAFTS):
            r = run_draft(cards, strategy_name, strat_rng)
            all_results.append(r)

    # --- Compute and display metrics ---
    print("\n" + "=" * 75)
    print("OVERALL METRICS (all strategies)")
    print("=" * 75)
    overall_m = compute_metrics(all_results)
    print_metrics(overall_m)

    for strategy_name in STRATEGIES:
        strat_results = [r for r in all_results if r["strategy"] == strategy_name]
        print(f"\n--- {strategy_name} ---")
        m = compute_metrics(strat_results)
        print_metrics(m)

    # --- Target scorecard ---
    committed_results = [r for r in all_results if r["strategy"] == "committed"]
    cm = compute_metrics(committed_results)

    print("\n" + "=" * 75)
    print("TARGET SCORECARD (committed strategy)")
    print("=" * 75)

    scorecard = [
        ("Picks 1-5: unique archs/pack (>=3)", ">= 3.0",
         cm["early_archetypes_per_pack"], cm["early_archetypes_per_pack"] >= 3.0),
        ("Picks 1-5: fitting/pack (<=2)", "<= 2.0",
         cm["early_fitting_per_pack"], cm["early_fitting_per_pack"] <= 2.0),
        ("Picks 6+: fitting/pack (>=2)", ">= 2.0",
         cm["late_fitting_per_pack"], cm["late_fitting_per_pack"] >= 2.0),
        ("Picks 6+: off-arch strong (>=0.5)", ">= 0.5",
         cm["late_off_arch_per_pack"], cm["late_off_arch_per_pack"] >= 0.5),
        ("Convergence pick (5-8)", "5-8",
         cm["convergence_pick"], 5 <= cm["convergence_pick"] <= 8),
        ("Deck concentration (85-95%)", "85-95%",
         cm["deck_concentration"], 85 <= cm["deck_concentration"] <= 95),
        ("Run-to-run overlap (<40%)", "< 40%",
         cm["run_overlap"], cm["run_overlap"] < 40),
        ("Arch freq (none >20% or <5%)", "5-20%",
         f"{cm['arch_min_freq']:.1f}-{cm['arch_max_freq']:.1f}",
         cm["arch_max_freq"] <= 20 and cm["arch_min_freq"] >= 5),
    ]

    print(f"\n{'Metric':<42} {'Target':<10} {'Actual':<12} {'Pass/Fail'}")
    print("-" * 75)
    passes = 0
    for name, target, actual, passed in scorecard:
        if isinstance(actual, float):
            actual_str = f"{actual:.2f}"
        else:
            actual_str = str(actual)
        status = "PASS" if passed else "FAIL"
        if passed:
            passes += 1
        print(f"{name:<42} {target:<10} {actual_str:<12} {status}")
    print(f"\nTotal: {passes}/{len(scorecard)} targets met")

    # Also show power-chaser concentration
    if "power_chaser_concentration" in cm:
        print(f"\nPower-chaser deck concentration: {cm.get('power_chaser_concentration', 0):.1f}%")

    # --- Sensitivity analyses ---
    run_sensitivity_multi_arch()
    run_sensitivity_weight_ramp()
    run_sensitivity_suppression()

    # --- Story traces ---
    run_story_traces(cards)

    print("\n" + "=" * 75)
    print("Simulation complete.")
    print("=" * 75)


def print_metrics(metrics):
    print(f"  Early archetypes/pack (picks 1-5): {metrics['early_archetypes_per_pack']:.2f}")
    print(f"  Early fitting/pack (picks 1-5):    {metrics['early_fitting_per_pack']:.2f}")
    print(f"  Late fitting/pack (picks 6+):      {metrics['late_fitting_per_pack']:.2f}")
    print(f"  Late off-arch strong/pack:         {metrics['late_off_arch_per_pack']:.2f}")
    print(f"  Convergence pick (avg):            {metrics['convergence_pick']:.1f}")
    print(f"  Convergence rate:                  {metrics['convergence_rate']:.1f}%")
    print(f"  Deck concentration:                {metrics['deck_concentration']:.1f}%")
    print(f"  Run-to-run overlap:                {metrics['run_overlap']:.1f}%")
    print(f"  Arch freq range:                   {metrics['arch_min_freq']:.1f}% - {metrics['arch_max_freq']:.1f}%")
    if metrics.get("arch_freq_detail"):
        detail = metrics["arch_freq_detail"]
        freqs = [f"A{a}:{detail[a]:.1f}%" for a in sorted(detail.keys())]
        print(f"  Per-archetype: {', '.join(freqs)}")


if __name__ == "__main__":
    main()
