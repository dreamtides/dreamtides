#!/usr/bin/env python3
"""
Model B Simulation: 10 Archetypes with Tight Card Pools

Simulates a draft system with N=10 archetypes, adaptive weighted sampling,
soft floor guarantees, and per-run archetype weighting for variety.
"""

import random
import math
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional
from collections import Counter

# ─── Constants ────────────────────────────────────────────────────────────────

NUM_ARCHETYPES = 10
NUM_CARDS = 360
PACK_SIZE = 4
NUM_PICKS = 30
NUM_DRAFTS = 1000
SEED = 42

# Rarity distribution
RARITY_DIST = {"common": 0.55, "uncommon": 0.25, "rare": 0.15, "legendary": 0.05}
RARITY_COPIES = {"common": 4, "uncommon": 3, "rare": 2, "legendary": 1}

# Card type distribution
CARD_TYPE_DIST = {
    "narrow_specialist": 0.30,
    "specialist_splash": 0.35,
    "dual_star": 0.10,
    "broad_generalist": 0.15,
    "universal_star": 0.02,
    "pure_filler": 0.08,
}

# Adaptive weight ramp (pick_number -> weight multiplier for fitting cards)
def get_weight_multiplier(pick_num):
    if pick_num <= 3:
        return 1.0
    elif pick_num <= 5:
        return 1.8
    elif pick_num <= 8:
        return 3.0
    elif pick_num <= 14:
        return 4.5
    else:
        return 6.0

# Archetype ring topology: each archetype neighbors the 2 adjacent ones
# Archetypes 0-9 in a ring: 0-1, 1-2, ..., 8-9, 9-0
def get_neighbors(arch_id):
    return [(arch_id - 1) % NUM_ARCHETYPES, (arch_id + 1) % NUM_ARCHETYPES]

# ─── Fitness Tier ─────────────────────────────────────────────────────────────

class Tier(Enum):
    S = 5
    A = 4
    B = 3
    C = 2
    F = 1

TIER_POWER = {Tier.S: 9.0, Tier.A: 7.0, Tier.B: 5.0, Tier.C: 3.0, Tier.F: 1.0}

# ─── Card ─────────────────────────────────────────────────────────────────────

@dataclass
class SimCard:
    id: int
    rarity: str
    power: float
    archetype_fitness: dict  # archetype_id -> Tier
    card_type: str

    def fitness_in(self, arch_id):
        return self.archetype_fitness.get(arch_id, Tier.F)

    def is_fitting(self, arch_id):
        """S or A tier in the given archetype."""
        return self.fitness_in(arch_id) in (Tier.S, Tier.A)

    def fitting_archetypes(self):
        """Return set of archetypes where this card is S or A tier."""
        return {a for a, t in self.archetype_fitness.items() if t in (Tier.S, Tier.A)}

    def s_tier_archetypes(self):
        return {a for a, t in self.archetype_fitness.items() if t == Tier.S}

# ─── Card Pool Generation ────────────────────────────────────────────────────

def assign_rarity(rng):
    r = rng.random()
    cumulative = 0.0
    for rarity, prob in RARITY_DIST.items():
        cumulative += prob
        if r < cumulative:
            return rarity
    return "common"

def assign_card_type(rng, dist=None):
    if dist is None:
        dist = CARD_TYPE_DIST
    r = rng.random()
    cumulative = 0.0
    for ctype, prob in dist.items():
        cumulative += prob
        if r < cumulative:
            return ctype
    return "pure_filler"

def generate_card_pool(rng, card_type_dist=None):
    """Generate 360 unique cards with per-archetype fitness scores."""
    cards = []

    # Pre-assign S-tier archetype for each card (round-robin for balance)
    # Each archetype gets 36 S-tier cards
    s_tier_assignments = []
    for i in range(NUM_CARDS):
        s_tier_assignments.append(i % NUM_ARCHETYPES)
    rng.shuffle(s_tier_assignments)

    for card_id in range(NUM_CARDS):
        rarity = assign_rarity(rng)
        card_type = assign_card_type(rng, card_type_dist)
        primary_arch = s_tier_assignments[card_id]
        neighbors = get_neighbors(primary_arch)

        fitness = {}

        if card_type == "narrow_specialist":
            # S in 1, B in 1-2 neighbors, C/F elsewhere
            fitness[primary_arch] = Tier.S
            # B in 1 neighbor
            b_count = rng.choice([1, 2])
            b_targets = rng.sample(neighbors, min(b_count, len(neighbors)))
            for n in b_targets:
                fitness[n] = Tier.B
            for a in range(NUM_ARCHETYPES):
                if a not in fitness:
                    fitness[a] = rng.choice([Tier.C, Tier.F])

        elif card_type == "specialist_splash":
            # S in 1, A in 1-2, B in 1-2, C/F elsewhere
            fitness[primary_arch] = Tier.S
            a_count = rng.choice([1, 2])
            # A-tier goes to neighbors preferentially (clustered overlap)
            a_candidates = list(neighbors)
            # Sometimes reach beyond neighbors
            if rng.random() < 0.3:
                non_neighbors = [x for x in range(NUM_ARCHETYPES)
                                 if x != primary_arch and x not in neighbors]
                a_candidates.extend(rng.sample(non_neighbors, min(1, len(non_neighbors))))
            a_targets = rng.sample(a_candidates, min(a_count, len(a_candidates)))
            for n in a_targets:
                fitness[n] = Tier.A
            # B in 1-2 others
            remaining = [x for x in range(NUM_ARCHETYPES) if x not in fitness]
            b_count = rng.choice([1, 2])
            b_targets = rng.sample(remaining, min(b_count, len(remaining)))
            for n in b_targets:
                fitness[n] = Tier.B
            for a in range(NUM_ARCHETYPES):
                if a not in fitness:
                    fitness[a] = rng.choice([Tier.C, Tier.F])

        elif card_type == "dual_star":
            # S in 2, A in 1, B in 1-2, C/F elsewhere
            fitness[primary_arch] = Tier.S
            # Second S goes to a neighbor
            second_s = rng.choice(neighbors)
            fitness[second_s] = Tier.S
            # A in 1 other
            remaining = [x for x in range(NUM_ARCHETYPES) if x not in fitness]
            a_target = rng.choice(remaining)
            fitness[a_target] = Tier.A
            # B in 1-2
            remaining = [x for x in range(NUM_ARCHETYPES) if x not in fitness]
            b_count = rng.choice([1, 2])
            b_targets = rng.sample(remaining, min(b_count, len(remaining)))
            for n in b_targets:
                fitness[n] = Tier.B
            for a in range(NUM_ARCHETYPES):
                if a not in fitness:
                    fitness[a] = rng.choice([Tier.C, Tier.F])

        elif card_type == "broad_generalist":
            # A in 2-3, B in 3-4, no S
            a_count = rng.choice([2, 3])
            a_targets = rng.sample(range(NUM_ARCHETYPES), a_count)
            for n in a_targets:
                fitness[n] = Tier.A
            remaining = [x for x in range(NUM_ARCHETYPES) if x not in fitness]
            b_count = rng.choice([3, 4])
            b_targets = rng.sample(remaining, min(b_count, len(remaining)))
            for n in b_targets:
                fitness[n] = Tier.B
            for a in range(NUM_ARCHETYPES):
                if a not in fitness:
                    fitness[a] = rng.choice([Tier.C, Tier.F])

        elif card_type == "universal_star":
            # S in 3+, high raw power
            s_count = rng.choice([3, 4])
            s_targets = rng.sample(range(NUM_ARCHETYPES), s_count)
            for n in s_targets:
                fitness[n] = Tier.S
            remaining = [x for x in range(NUM_ARCHETYPES) if x not in fitness]
            for n in remaining:
                fitness[n] = rng.choice([Tier.A, Tier.B])

        elif card_type == "pure_filler":
            # B in 2-3, C/F elsewhere, no S or A
            b_count = rng.choice([2, 3])
            b_targets = rng.sample(range(NUM_ARCHETYPES), b_count)
            for n in b_targets:
                fitness[n] = Tier.B
            for a in range(NUM_ARCHETYPES):
                if a not in fitness:
                    fitness[a] = rng.choice([Tier.C, Tier.F])

        # Power is based on best tier + noise
        best_tier = max(fitness.values(), key=lambda t: t.value)
        base_power = TIER_POWER[best_tier]
        if card_type == "universal_star":
            base_power = 9.5
        power = max(1.0, min(10.0, base_power + rng.gauss(0, 0.8)))

        cards.append(SimCard(
            id=card_id,
            rarity=rarity,
            power=power,
            archetype_fitness=fitness,
            card_type=card_type,
        ))

    return cards


# ─── Pool Construction (per-run) ─────────────────────────────────────────────

def build_run_pool(cards, rng):
    """
    Build the draft pool for a single run with archetype weighting.
    Returns list of (card, copies) and the archetype status dict.
    """
    # Assign archetype statuses: 3 boosted, 3 normal, 4 suppressed
    arch_ids = list(range(NUM_ARCHETYPES))
    rng.shuffle(arch_ids)
    arch_status = {}
    for i, a in enumerate(arch_ids):
        if i < 3:
            arch_status[a] = "boosted"
        elif i < 6:
            arch_status[a] = "normal"
        else:
            arch_status[a] = "suppressed"

    # Copy count multipliers
    status_multiplier = {"boosted": 1.5, "normal": 1.0, "suppressed": 0.6}

    pool_entries = []
    for card in cards:
        base_copies = RARITY_COPIES[card.rarity]

        # Card's copy count is influenced by the best status of its S-tier archetypes
        s_archs = card.s_tier_archetypes()
        if s_archs:
            best_status_mult = max(status_multiplier[arch_status[a]] for a in s_archs)
        else:
            # Generalists/filler: use average of A-tier archetypes
            a_archs = card.fitting_archetypes()
            if a_archs:
                best_status_mult = max(status_multiplier[arch_status[a]] for a in a_archs)
            else:
                best_status_mult = 1.0

        # Additional per-card random variance
        card_variance = rng.uniform(0.8, 1.2)
        effective_copies = max(1, round(base_copies * best_status_mult * card_variance))

        for _ in range(effective_copies):
            pool_entries.append(card)

    return pool_entries, arch_status


# ─── Pack Construction ────────────────────────────────────────────────────────

def detect_archetype(picked_cards):
    """
    Detect the player's most likely archetype(s) based on picked cards.
    Returns (primary_archetype, commitment_strength).
    """
    if not picked_cards:
        return None, 0

    arch_counts = Counter()
    for card in picked_cards:
        for a, t in card.archetype_fitness.items():
            if t in (Tier.S, Tier.A):
                arch_counts[a] += 1

    if not arch_counts:
        return None, 0

    best_arch = arch_counts.most_common(1)[0]
    return best_arch[0], best_arch[1]


def build_pack(pool, pick_num, picked_cards, rng):
    """
    Build a 4-card pack using adaptive weighted sampling with soft floor.
    """
    primary_arch, commitment = detect_archetype(picked_cards)
    weight_mult = get_weight_multiplier(pick_num)

    # Only apply bias if player has committed (2+ S/A picks in an archetype)
    apply_bias = primary_arch is not None and commitment >= 2

    # Calculate weights
    weights = []
    for card in pool:
        w = 1.0
        if apply_bias and card.is_fitting(primary_arch):
            w *= weight_mult
        weights.append(w)

    # Normalize
    total_w = sum(weights)
    if total_w == 0:
        return rng.sample(range(len(pool)), min(PACK_SIZE, len(pool)))

    # Weighted sample without replacement
    selected_indices = []
    available = list(range(len(pool)))
    avail_weights = list(weights)

    for _ in range(min(PACK_SIZE, len(available))):
        total = sum(avail_weights[i] for i in range(len(available)))
        if total <= 0:
            break
        r = rng.random() * total
        cumulative = 0.0
        chosen_pos = 0
        for pos in range(len(available)):
            cumulative += avail_weights[pos]
            if cumulative >= r:
                chosen_pos = pos
                break
        selected_indices.append(available[chosen_pos])
        available.pop(chosen_pos)
        avail_weights.pop(chosen_pos)

    # Soft floor guarantee: after pick 6, if 0 fitting cards, replace one
    if apply_bias and pick_num >= 6:
        pack_cards = [pool[i] for i in selected_indices]
        fitting_count = sum(1 for c in pack_cards if c.is_fitting(primary_arch))
        if fitting_count == 0:
            # Find a fitting card not in the pack
            fitting_candidates = [
                i for i in range(len(pool))
                if pool[i].is_fitting(primary_arch) and i not in selected_indices
            ]
            if fitting_candidates:
                replacement = rng.choice(fitting_candidates)
                # Replace the weakest card in the pack
                weakest_idx = min(range(len(selected_indices)),
                                  key=lambda j: pool[selected_indices[j]].power)
                selected_indices[weakest_idx] = replacement

    return selected_indices


# ─── Player Strategies ────────────────────────────────────────────────────────

def strategy_archetype_committed(pack_cards, picked_cards, pick_num):
    """Pick the card with highest fitness in the player's strongest archetype.
    Will take a high-power off-archetype card if no good fitting option exists."""
    primary_arch, commitment = detect_archetype(picked_cards)

    if primary_arch is None:
        # Early picks: pick the highest power card
        return max(range(len(pack_cards)), key=lambda i: pack_cards[i].power)

    # Check if any fitting card is available with decent power
    fitting_cards = [(i, c) for i, c in enumerate(pack_cards) if c.is_fitting(primary_arch)]
    best_fitting_power = max((c.power for _, c in fitting_cards), default=0)

    # Realistic behavior: sometimes take powerful off-archetype cards
    best_power_idx = max(range(len(pack_cards)), key=lambda i: pack_cards[i].power)
    best_power = pack_cards[best_power_idx].power

    # Take off-archetype if: (a) no good fitting card, or (b) power difference is huge
    if best_fitting_power < 6.5 and best_power >= 7.5:
        return best_power_idx
    if best_power - best_fitting_power >= 2.5:
        return best_power_idx

    # Score by fitness in primary archetype, with power as tiebreaker
    def score(card):
        tier_val = card.fitness_in(primary_arch).value
        return tier_val * 100 + card.power
    return max(range(len(pack_cards)), key=lambda i: score(pack_cards[i]))


def strategy_power_chaser(pack_cards, picked_cards, pick_num):
    """Pick the highest raw power card."""
    return max(range(len(pack_cards)), key=lambda i: pack_cards[i].power)


def strategy_signal_reader(pack_cards, picked_cards, pick_num, seen_cards):
    """
    Evaluate which archetype seems most available and draft toward it.
    In early picks, track which archetypes appear most.
    After enough data, commit to the most available one.
    """
    if pick_num <= 5:
        # Track archetype frequency in seen cards
        arch_freq = Counter()
        for card in seen_cards:
            for a in card.s_tier_archetypes():
                arch_freq[a] += 1

        if arch_freq:
            open_arch = arch_freq.most_common(1)[0][0]
            # Pick the card that best fits the open archetype, with power tiebreak
            def score(card):
                tier_val = card.fitness_in(open_arch).value
                return tier_val * 100 + card.power
            return max(range(len(pack_cards)), key=lambda i: score(pack_cards[i]))
        else:
            return max(range(len(pack_cards)), key=lambda i: pack_cards[i].power)
    else:
        # After pick 5, behave like archetype-committed
        return strategy_archetype_committed(pack_cards, picked_cards, pick_num)


# ─── Draft Simulation ────────────────────────────────────────────────────────

@dataclass
class DraftResult:
    strategy: str
    picked_cards: list
    seen_cards: list  # all cards seen across all packs
    pick_details: list  # list of dicts with per-pick info
    arch_status: dict
    primary_archetype: Optional[int] = None
    commitment_pick: Optional[int] = None  # pick at which 2+ fitting regularly seen


def simulate_draft(cards, strategy_name, rng, trace=False):
    """Simulate a single 30-pick draft."""
    pool_entries, arch_status = build_run_pool(cards, rng)
    pool = list(pool_entries)  # mutable copy

    picked_cards = []
    seen_cards = []
    pick_details = []

    for pick_num in range(1, NUM_PICKS + 1):
        if len(pool) < PACK_SIZE:
            break

        pack_indices = build_pack(pool, pick_num, picked_cards, rng)
        pack_cards = [pool[i] for i in pack_indices]
        seen_cards.extend(pack_cards)

        # Apply strategy
        if strategy_name == "archetype_committed":
            choice = strategy_archetype_committed(pack_cards, picked_cards, pick_num)
        elif strategy_name == "power_chaser":
            choice = strategy_power_chaser(pack_cards, picked_cards, pick_num)
        elif strategy_name == "signal_reader":
            choice = strategy_signal_reader(pack_cards, picked_cards, pick_num, seen_cards)
        else:
            raise ValueError(f"Unknown strategy: {strategy_name}")

        chosen_card = pack_cards[choice]
        picked_cards.append(chosen_card)

        # Detect current archetype
        primary_arch, commitment = detect_archetype(picked_cards)

        # Count fitting cards in this pack
        fitting_in_pack = 0
        if primary_arch is not None:
            fitting_in_pack = sum(1 for c in pack_cards if c.is_fitting(primary_arch))

        # Count unique archetypes in pack
        pack_archetypes = set()
        for c in pack_cards:
            for a in c.s_tier_archetypes():
                pack_archetypes.add(a)

        # Count strong off-archetype cards
        off_arch_strong = 0
        if primary_arch is not None:
            for c in pack_cards:
                if not c.is_fitting(primary_arch):
                    # Strong if power >= 7.0 or S-tier in any other archetype
                    if c.power >= 7.0 or any(
                        t == Tier.S for a, t in c.archetype_fitness.items()
                        if a != primary_arch
                    ):
                        off_arch_strong += 1

        detail = {
            "pick_num": pick_num,
            "pack_card_ids": [c.id for c in pack_cards],
            "pack_fitting": fitting_in_pack,
            "pack_archetypes": len(pack_archetypes),
            "chosen_card_id": chosen_card.id,
            "chosen_fitness": chosen_card.fitness_in(primary_arch).name if primary_arch is not None else "N/A",
            "primary_arch": primary_arch,
            "commitment": commitment,
            "off_arch_strong": off_arch_strong,
        }
        pick_details.append(detail)

        # Remove chosen card's pool entry
        chosen_pool_idx = pack_indices[choice]
        pool.pop(chosen_pool_idx)

        if trace:
            archs_str = ", ".join(
                f"Arch{a}:{c.fitness_in(a).name}" for a in sorted(c.s_tier_archetypes())
            ) if c.s_tier_archetypes() else "none"
            print(f"  Pick {pick_num}: Pack archetypes={len(pack_archetypes)}, "
                  f"fitting={fitting_in_pack}, "
                  f"chose Card#{chosen_card.id} (power={chosen_card.power:.1f}, "
                  f"type={chosen_card.card_type}, "
                  f"fit={detail['chosen_fitness']}), "
                  f"arch={primary_arch}, commitment={commitment}")

    result = DraftResult(
        strategy=strategy_name,
        picked_cards=picked_cards,
        seen_cards=seen_cards,
        pick_details=pick_details,
        arch_status=arch_status,
    )

    # Determine primary archetype and convergence pick
    if picked_cards:
        primary_arch, _ = detect_archetype(picked_cards)
        result.primary_archetype = primary_arch

        # Convergence pick: first pick where player has seen 2+ fitting cards
        # in 3 of the last 4 packs
        window = []
        for detail in pick_details:
            if detail["primary_arch"] is not None:
                window.append(detail["pack_fitting"] >= 2)
            else:
                window.append(False)
            if len(window) > 4:
                window.pop(0)
            if len(window) >= 4 and sum(window) >= 3:
                if result.commitment_pick is None:
                    result.commitment_pick = detail["pick_num"]

    return result


# ─── Metrics ──────────────────────────────────────────────────────────────────

def compute_metrics(results):
    """Compute all 8 measurable target metrics across a list of DraftResults."""
    metrics = {}

    # 1. Picks 1-5: unique archetypes represented per pack (>= 3 of 4)
    early_arch_counts = []
    for r in results:
        for d in r.pick_details:
            if d["pick_num"] <= 5:
                early_arch_counts.append(d["pack_archetypes"])
    metrics["early_archetypes_per_pack"] = (
        sum(early_arch_counts) / len(early_arch_counts) if early_arch_counts else 0
    )

    # 2. Picks 1-5: cards fitting player's emerging archetype per pack (<= 2 of 4)
    early_fitting = []
    for r in results:
        for d in r.pick_details:
            if d["pick_num"] <= 5 and d["primary_arch"] is not None:
                early_fitting.append(d["pack_fitting"])
    metrics["early_fitting_per_pack"] = (
        sum(early_fitting) / len(early_fitting) if early_fitting else 0
    )

    # 3. Picks 6+: cards fitting committed archetype per pack (>= 2 of 4)
    late_fitting = []
    for r in results:
        for d in r.pick_details:
            if d["pick_num"] >= 6 and d["primary_arch"] is not None and d["commitment"] >= 2:
                late_fitting.append(d["pack_fitting"])
    metrics["late_fitting_per_pack"] = (
        sum(late_fitting) / len(late_fitting) if late_fitting else 0
    )

    # 4. Picks 6+: strong off-archetype cards per pack (>= 0.5 of 4)
    late_off_arch = []
    for r in results:
        for d in r.pick_details:
            if d["pick_num"] >= 6 and d["primary_arch"] is not None and d["commitment"] >= 2:
                late_off_arch.append(d["off_arch_strong"])
    metrics["late_off_arch_per_pack"] = (
        sum(late_off_arch) / len(late_off_arch) if late_off_arch else 0
    )

    # 5. Convergence pick (pick 5-8)
    convergence_picks = [r.commitment_pick for r in results if r.commitment_pick is not None]
    metrics["convergence_pick"] = (
        sum(convergence_picks) / len(convergence_picks) if convergence_picks else float('inf')
    )
    metrics["convergence_pct"] = (
        len(convergence_picks) / len(results) * 100 if results else 0
    )

    # 6. Deck archetype concentration (60-80% S/A-tier cards for committed player)
    concentrations = []
    for r in results:
        if r.primary_archetype is not None and r.strategy == "archetype_committed":
            fitting = sum(1 for c in r.picked_cards if c.is_fitting(r.primary_archetype))
            concentrations.append(fitting / len(r.picked_cards) * 100 if r.picked_cards else 0)
    metrics["deck_concentration"] = (
        sum(concentrations) / len(concentrations) if concentrations else 0
    )

    # 7. Run-to-run variety (< 40% card overlap)
    # Compare pairs of runs with same strategy
    by_strategy = {}
    for r in results:
        by_strategy.setdefault(r.strategy, []).append(r)

    overlaps = []
    for strat, strat_results in by_strategy.items():
        for i in range(min(50, len(strat_results) - 1)):
            cards_a = set(c.id for c in strat_results[i].picked_cards)
            cards_b = set(c.id for c in strat_results[i + 1].picked_cards)
            if cards_a and cards_b:
                overlap = len(cards_a & cards_b) / len(cards_a | cards_b) * 100
                overlaps.append(overlap)
    metrics["run_overlap"] = sum(overlaps) / len(overlaps) if overlaps else 0

    # 8. Archetype frequency (no archetype > 20% or < 5%)
    arch_freq = Counter()
    total_runs = 0
    for r in results:
        if r.primary_archetype is not None:
            arch_freq[r.primary_archetype] += 1
            total_runs += 1

    if total_runs > 0:
        freqs = {a: arch_freq.get(a, 0) / total_runs * 100 for a in range(NUM_ARCHETYPES)}
        metrics["arch_max_freq"] = max(freqs.values())
        metrics["arch_min_freq"] = min(freqs.values())
        metrics["arch_freq_detail"] = freqs
    else:
        metrics["arch_max_freq"] = 0
        metrics["arch_min_freq"] = 0
        metrics["arch_freq_detail"] = {}

    return metrics


# ─── Sensitivity Analysis ────────────────────────────────────────────────────

def run_sensitivity_analysis(base_cards, rng):
    """
    Vary the % of multi-archetype cards and measure key metrics.
    """
    print("\n" + "=" * 70)
    print("MULTI-ARCHETYPE CARD SENSITIVITY ANALYSIS")
    print("=" * 70)

    # We vary by adjusting how many specialist_splash and dual_star cards exist
    # Original: 35% splash + 10% dual + 15% generalist + 2% universal = 62% multi
    # Test: 10%, 20%, 30%, 40%, 50%, 62% (original), 75%
    configs = [
        ("10% multi", {"narrow_specialist": 0.72, "specialist_splash": 0.05,
                        "dual_star": 0.02, "broad_generalist": 0.03,
                        "universal_star": 0.00, "pure_filler": 0.18}),
        ("20% multi", {"narrow_specialist": 0.60, "specialist_splash": 0.10,
                        "dual_star": 0.04, "broad_generalist": 0.06,
                        "universal_star": 0.00, "pure_filler": 0.20}),
        ("30% multi", {"narrow_specialist": 0.52, "specialist_splash": 0.15,
                        "dual_star": 0.06, "broad_generalist": 0.09,
                        "universal_star": 0.00, "pure_filler": 0.18}),
        ("40% multi", {"narrow_specialist": 0.44, "specialist_splash": 0.22,
                        "dual_star": 0.07, "broad_generalist": 0.11,
                        "universal_star": 0.01, "pure_filler": 0.15}),
        ("50% multi", {"narrow_specialist": 0.38, "specialist_splash": 0.28,
                        "dual_star": 0.08, "broad_generalist": 0.13,
                        "universal_star": 0.01, "pure_filler": 0.12}),
        ("62% multi (baseline)", CARD_TYPE_DIST),
        ("75% multi", {"narrow_specialist": 0.17, "specialist_splash": 0.42,
                        "dual_star": 0.13, "broad_generalist": 0.18,
                        "universal_star": 0.02, "pure_filler": 0.08}),
    ]

    print(f"\n{'Config':<25} {'Late Fit/Pack':<15} {'Early Fit/Pack':<15} "
          f"{'Deck Conc %':<12} {'Convergence':<12} {'Overlap %':<10}")
    print("-" * 90)

    for config_name, dist in configs:
        test_rng = random.Random(SEED + 1000)
        test_cards = generate_card_pool(test_rng, card_type_dist=dist)

        # Run 200 drafts with archetype-committed strategy
        test_results = []
        for _ in range(200):
            result = simulate_draft(test_cards, "archetype_committed", test_rng)
            test_results.append(result)

        m = compute_metrics(test_results)
        print(f"{config_name:<25} {m['late_fitting_per_pack']:<15.2f} "
              f"{m['early_fitting_per_pack']:<15.2f} "
              f"{m['deck_concentration']:<12.1f} "
              f"{m['convergence_pick']:<12.1f} "
              f"{m['run_overlap']:<10.1f}")


# ─── Main ────────────────────────────────────────────────────────────────────

def main():
    rng = random.Random(SEED)
    print("Generating card pool (360 cards, 10 archetypes)...")
    cards = generate_card_pool(rng)

    # Verify distribution
    multi_count = sum(1 for c in cards if len(c.fitting_archetypes()) >= 2)
    print(f"Cards with S/A in 2+ archetypes: {multi_count} ({multi_count/len(cards)*100:.1f}%)")

    type_counts = Counter(c.card_type for c in cards)
    print(f"Card type distribution: {dict(type_counts)}")

    # Per-archetype S-tier and S/A-tier counts
    for a in range(NUM_ARCHETYPES):
        s_count = sum(1 for c in cards if c.fitness_in(a) == Tier.S)
        sa_count = sum(1 for c in cards if c.is_fitting(a))
        if a < 3:
            print(f"  Archetype {a}: {s_count} S-tier, {sa_count} S/A-tier")

    # ─── Run 1000 drafts per strategy ─────────────────────────────────────
    strategies = ["archetype_committed", "power_chaser", "signal_reader"]
    all_results = []

    for strategy in strategies:
        print(f"\nRunning 1000 drafts with strategy: {strategy}...")
        strat_rng = random.Random(SEED + hash(strategy))
        for i in range(NUM_DRAFTS):
            result = simulate_draft(cards, strategy, strat_rng)
            all_results.append(result)

    # ─── Compute and display metrics ──────────────────────────────────────
    print("\n" + "=" * 70)
    print("OVERALL METRICS (all strategies combined)")
    print("=" * 70)
    overall_metrics = compute_metrics(all_results)
    print_metrics(overall_metrics)

    # Per-strategy metrics
    for strategy in strategies:
        strat_results = [r for r in all_results if r.strategy == strategy]
        print(f"\n{'=' * 70}")
        print(f"METRICS: {strategy}")
        print(f"{'=' * 70}")
        m = compute_metrics(strat_results)
        print_metrics(m)

    # ─── Draft traces ─────────────────────────────────────────────────────
    print("\n" + "=" * 70)
    print("DETAILED DRAFT TRACES")
    print("=" * 70)

    trace_rng = random.Random(SEED + 999)

    print("\n--- Trace 1: Archetype-Committed Player ---")
    result1 = simulate_draft(cards, "archetype_committed", trace_rng, trace=True)
    print_trace_summary(result1)

    print("\n--- Trace 2: Power-Chaser Player ---")
    result2 = simulate_draft(cards, "power_chaser", trace_rng, trace=True)
    print_trace_summary(result2)

    print("\n--- Trace 3: Signal-Reader Player ---")
    result3 = simulate_draft(cards, "signal_reader", trace_rng, trace=True)
    print_trace_summary(result3)

    # ─── Sensitivity analysis ─────────────────────────────────────────────
    run_sensitivity_analysis(cards, rng)

    # ─── Target Scorecard ─────────────────────────────────────────────────
    print("\n" + "=" * 70)
    print("TARGET SCORECARD")
    print("=" * 70)

    committed_results = [r for r in all_results if r.strategy == "archetype_committed"]
    committed_metrics = compute_metrics(committed_results)

    scorecard = [
        ("Early archetypes/pack (>=3)", ">= 3.0",
         f"{overall_metrics['early_archetypes_per_pack']:.2f}",
         overall_metrics['early_archetypes_per_pack'] >= 3.0),
        ("Early fitting/pack (<=2)", "<= 2.0",
         f"{overall_metrics['early_fitting_per_pack']:.2f}",
         overall_metrics['early_fitting_per_pack'] <= 2.0),
        ("Late fitting/pack (>=2)", ">= 2.0",
         f"{committed_metrics['late_fitting_per_pack']:.2f}",
         committed_metrics['late_fitting_per_pack'] >= 2.0),
        ("Late off-arch strong (>=0.5)", ">= 0.5",
         f"{committed_metrics['late_off_arch_per_pack']:.2f}",
         committed_metrics['late_off_arch_per_pack'] >= 0.5),
        ("Convergence pick (5-8)", "5-8",
         f"{committed_metrics['convergence_pick']:.1f}",
         5 <= committed_metrics['convergence_pick'] <= 8),
        ("Deck concentration (60-80%)", "60-80%",
         f"{committed_metrics['deck_concentration']:.1f}%",
         60 <= committed_metrics['deck_concentration'] <= 80),
        ("Run overlap (<40%)", "< 40%",
         f"{committed_metrics['run_overlap']:.1f}%",
         committed_metrics['run_overlap'] < 40),
        ("Arch freq (none >20%, <5%)", "5-20%",
         f"{committed_metrics['arch_min_freq']:.1f}-{committed_metrics['arch_max_freq']:.1f}%",
         committed_metrics['arch_max_freq'] <= 20 and committed_metrics['arch_min_freq'] >= 5),
    ]

    print(f"\n{'Metric':<40} {'Target':<12} {'Actual':<12} {'Pass/Fail':<10}")
    print("-" * 75)
    passes = 0
    for name, target, actual, passed in scorecard:
        status = "PASS" if passed else "FAIL"
        if passed:
            passes += 1
        print(f"{name:<40} {target:<12} {actual:<12} {status:<10}")
    print(f"\nTotal: {passes}/{len(scorecard)} targets met")


def print_metrics(metrics):
    print(f"  Early archetypes per pack (picks 1-5): {metrics['early_archetypes_per_pack']:.2f}")
    print(f"  Early fitting per pack (picks 1-5):    {metrics['early_fitting_per_pack']:.2f}")
    print(f"  Late fitting per pack (picks 6+):      {metrics['late_fitting_per_pack']:.2f}")
    print(f"  Late off-arch strong per pack:         {metrics['late_off_arch_per_pack']:.2f}")
    print(f"  Convergence pick (avg):                {metrics['convergence_pick']:.1f}")
    print(f"  Convergence rate:                      {metrics['convergence_pct']:.1f}%")
    print(f"  Deck concentration:                    {metrics['deck_concentration']:.1f}%")
    print(f"  Run-to-run overlap:                    {metrics['run_overlap']:.1f}%")
    print(f"  Arch freq max:                         {metrics['arch_max_freq']:.1f}%")
    print(f"  Arch freq min:                         {metrics['arch_min_freq']:.1f}%")
    if metrics.get('arch_freq_detail'):
        detail = metrics['arch_freq_detail']
        freqs = [f"A{a}:{detail[a]:.1f}%" for a in sorted(detail.keys())]
        print(f"  Arch frequencies: {', '.join(freqs)}")


def print_trace_summary(result):
    print(f"  Strategy: {result.strategy}")
    print(f"  Primary archetype: {result.primary_archetype}")
    print(f"  Convergence pick: {result.commitment_pick}")
    if result.primary_archetype is not None:
        fitting = sum(1 for c in result.picked_cards
                      if c.is_fitting(result.primary_archetype))
        print(f"  Deck: {fitting}/{len(result.picked_cards)} fitting "
              f"({fitting/len(result.picked_cards)*100:.0f}%)")
    print(f"  Arch status: {result.arch_status}")

    # Card type breakdown
    type_counts = Counter(c.card_type for c in result.picked_cards)
    print(f"  Picked card types: {dict(type_counts)}")


if __name__ == "__main__":
    main()
