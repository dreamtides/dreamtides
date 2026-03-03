#!/usr/bin/env python3
"""
Agent 2 Pool Investigation: Rarity x Pair-Escalation Slots

Investigates how rarity should interact with Pair-Escalation Slots and
resonance symbols.

Tests 5 rarity models:
  A: Flat rarity (cosmetic only, uniform power)
  B: Standard TCG (180C/100U/60R/20L, power scales with rarity)
  C: Roguelike-skewed (120C/100U/90R/50L, power scales)
  D: Rarity-symbol correlation (rares have more symbols, accelerate pairs)
  E: Inverse correlation (rares have fewer symbols, stall pair accumulation)

All models use Pair-Escalation Slots (K=6, cap=0.50, 4 slots, 30 picks).

Key insight: In Pair-Escalation, only 2+ symbol cards contribute pairs.
If rares tend to have 1 symbol (Model E), drafting a rare STALLS pair
accumulation. If rares have 2-3 symbols (Model D), drafting a rare
ACCELERATES convergence. This creates a meaningful tension: "do I pick the
powerful rare that doesn't help my pairs, or the weaker common that does?"
"""

import random
import statistics
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

NUM_CARDS = 360
NUM_GENERIC = 36
NUM_ARCHETYPE_CARDS = NUM_CARDS - NUM_GENERIC  # 324
NUM_ARCHETYPES = 8
CARDS_PER_ARCHETYPE = NUM_ARCHETYPE_CARDS // NUM_ARCHETYPES  # 40
PACK_SIZE = 4
NUM_PICKS = 30
NUM_DRAFTS = 1200

# Pair-Escalation parameters
PE_K = 6.0
PE_CAP = 0.50

RESONANCES = ["Ember", "Stone", "Tide", "Zephyr"]

ARCHETYPES = [
    ("Flash",        "Zephyr", "Ember"),   # 0
    ("Blink",        "Ember",  "Zephyr"),  # 1
    ("Storm",        "Ember",  "Stone"),   # 2
    ("Self-Discard", "Stone",  "Ember"),   # 3
    ("Self-Mill",    "Stone",  "Tide"),    # 4
    ("Sacrifice",    "Tide",   "Stone"),   # 5
    ("Warriors",     "Tide",   "Zephyr"),  # 6
    ("Ramp",         "Zephyr", "Tide"),    # 7
]

ARCHETYPE_NAMES = [a[0] for a in ARCHETYPES]

# ---------------------------------------------------------------------------
# Rarity
# ---------------------------------------------------------------------------

class Rarity(Enum):
    COMMON = 0
    UNCOMMON = 1
    RARE = 2
    LEGENDARY = 3

RARITY_SHORT = {Rarity.COMMON: "C", Rarity.UNCOMMON: "U",
                Rarity.RARE: "R", Rarity.LEGENDARY: "L"}

# ---------------------------------------------------------------------------
# Fitness
# ---------------------------------------------------------------------------

class Tier(Enum):
    S = 5
    A = 4
    B = 3
    C = 1
    F = 0

TIER_VAL = {"S": 5, "A": 4, "B": 3, "C": 1, "F": 0}


def circle_distance(i: int, j: int) -> int:
    d = abs(i - j)
    return min(d, NUM_ARCHETYPES - d)


def compute_fitness(card_archetype: Optional[str], card_symbols: list) -> dict:
    fitness = {}
    if card_archetype is None:
        for name, _, _ in ARCHETYPES:
            fitness[name] = "B"
        return fitness

    home_idx = ARCHETYPE_NAMES.index(card_archetype)
    home_pri = ARCHETYPES[home_idx][1]
    home_sec = ARCHETYPES[home_idx][2]

    for i, (name, arch_pri, arch_sec) in enumerate(ARCHETYPES):
        if name == card_archetype:
            fitness[name] = "S"
        elif circle_distance(home_idx, i) == 1 and home_pri in (arch_pri, arch_sec):
            fitness[name] = "A"
        elif home_sec in (arch_pri, arch_sec) and i != home_idx:
            fitness[name] = "B"
        elif home_pri in (arch_pri, arch_sec):
            fitness[name] = "B"
        else:
            fitness[name] = "F"

    return fitness


# ---------------------------------------------------------------------------
# Card model
# ---------------------------------------------------------------------------

@dataclass
class SimCard:
    id: int
    symbols: list
    archetype: Optional[str]
    archetype_fitness: dict
    rarity: Rarity
    power: float

    def ordered_pair(self):
        if len(self.symbols) >= 2:
            return (self.symbols[0], self.symbols[1])
        return None

    def primary_resonance(self):
        return self.symbols[0] if self.symbols else None


# ---------------------------------------------------------------------------
# Power scaling
# ---------------------------------------------------------------------------

def flat_power(_rarity: Rarity) -> float:
    return random.uniform(4.5, 5.5)


def scaled_power(rarity: Rarity) -> float:
    ranges = {
        Rarity.COMMON:    (2.0, 5.0),
        Rarity.UNCOMMON:  (4.0, 7.0),
        Rarity.RARE:      (6.0, 9.0),
        Rarity.LEGENDARY: (8.0, 10.0),
    }
    lo, hi = ranges[rarity]
    return random.uniform(lo, hi)


# ---------------------------------------------------------------------------
# Symbol generation
# ---------------------------------------------------------------------------

def make_symbols(num_symbols: int, primary: str, secondary: str) -> list:
    if num_symbols == 1:
        return [primary] if random.random() < 0.70 else [secondary]
    elif num_symbols == 2:
        first = primary if random.random() < 0.75 else secondary
        other = secondary if first == primary else primary
        second = other if random.random() < 0.80 else first
        return [first, second]
    else:  # 3
        patterns = [
            [primary, secondary, primary],
            [primary, primary, secondary],
            [primary, secondary, secondary],
        ]
        return random.choice(patterns)


# ---------------------------------------------------------------------------
# Pool generation helpers
# ---------------------------------------------------------------------------

def _assign_symbol_count_default(index: int, n_cards: int) -> int:
    """Default 15/60/25 distribution."""
    n1 = round(n_cards * 0.15)
    n3 = round(n_cards * 0.25)
    if index < n1:
        return 1
    elif index < n1 + (n_cards - n1 - n3):
        return 2
    else:
        return 3


def _build_rarity_list(rarity_counts: dict, total: int) -> list:
    result = []
    for r, count in rarity_counts.items():
        result.extend([r] * count)
    while len(result) < total:
        result.append(Rarity.COMMON)
    random.shuffle(result)
    return result[:total]


def _generate_pool_standard(rarity_counts: dict, power_fn) -> list:
    """Generate pool with standard symbol distribution, rarity independent of symbols."""
    cards = []
    card_id = 0

    rarity_list = _build_rarity_list(rarity_counts, NUM_CARDS)
    rarity_idx = 0

    for arch_idx, (arch_name, primary, secondary) in enumerate(ARCHETYPES):
        n = CARDS_PER_ARCHETYPE
        for i in range(n):
            num_sym = _assign_symbol_count_default(i, n)
            syms = make_symbols(num_sym, primary, secondary)
            rarity = rarity_list[rarity_idx]
            rarity_idx += 1
            fitness = compute_fitness(arch_name, syms)
            cards.append(SimCard(
                id=card_id, symbols=syms, archetype=arch_name,
                archetype_fitness=fitness, rarity=rarity,
                power=power_fn(rarity),
            ))
            card_id += 1

    for _ in range(NUM_GENERIC):
        rarity = rarity_list[rarity_idx] if rarity_idx < len(rarity_list) else Rarity.COMMON
        rarity_idx += 1
        cards.append(SimCard(
            id=card_id, symbols=[], archetype=None,
            archetype_fitness=compute_fitness(None, []),
            rarity=rarity, power=power_fn(rarity),
        ))
        card_id += 1

    random.shuffle(cards)
    return cards


def _generate_pool_symbol_correlated(rarity_counts: dict, power_fn,
                                      symbol_count_fn) -> list:
    """Generate pool where symbol count depends on rarity."""
    cards = []
    card_id = 0

    for arch_idx, (arch_name, primary, secondary) in enumerate(ARCHETYPES):
        n = CARDS_PER_ARCHETYPE
        # Proportional rarity assignment per archetype
        arch_rarities = []
        for r, total_count in rarity_counts.items():
            share = round(total_count * n / NUM_CARDS)
            arch_rarities.extend([r] * share)
        while len(arch_rarities) < n:
            arch_rarities.append(Rarity.COMMON)
        arch_rarities = arch_rarities[:n]
        random.shuffle(arch_rarities)

        for i in range(n):
            rarity = arch_rarities[i]
            num_sym = symbol_count_fn(rarity)
            syms = make_symbols(num_sym, primary, secondary)
            fitness = compute_fitness(arch_name, syms)
            cards.append(SimCard(
                id=card_id, symbols=syms, archetype=arch_name,
                archetype_fitness=fitness, rarity=rarity,
                power=power_fn(rarity),
            ))
            card_id += 1

    # Generic cards
    generic_rarities = []
    for r, total_count in rarity_counts.items():
        share = round(total_count * NUM_GENERIC / NUM_CARDS)
        generic_rarities.extend([r] * share)
    while len(generic_rarities) < NUM_GENERIC:
        generic_rarities.append(Rarity.COMMON)
    generic_rarities = generic_rarities[:NUM_GENERIC]
    random.shuffle(generic_rarities)

    for i in range(NUM_GENERIC):
        rarity = generic_rarities[i]
        cards.append(SimCard(
            id=card_id, symbols=[], archetype=None,
            archetype_fitness=compute_fitness(None, []),
            rarity=rarity, power=power_fn(rarity),
        ))
        card_id += 1

    random.shuffle(cards)
    return cards


# ---------------------------------------------------------------------------
# 5 Pool models
# ---------------------------------------------------------------------------

STANDARD_RARITY = {Rarity.COMMON: 180, Rarity.UNCOMMON: 100,
                   Rarity.RARE: 60, Rarity.LEGENDARY: 20}

ROGUELIKE_RARITY = {Rarity.COMMON: 120, Rarity.UNCOMMON: 100,
                    Rarity.RARE: 90, Rarity.LEGENDARY: 50}


def generate_pool_A() -> list:
    """Model A: Flat rarity (cosmetic only, uniform power)."""
    return _generate_pool_standard(STANDARD_RARITY, flat_power)


def generate_pool_B() -> list:
    """Model B: Standard TCG (180C/100U/60R/20L, power scales)."""
    return _generate_pool_standard(STANDARD_RARITY, scaled_power)


def generate_pool_C() -> list:
    """Model C: Roguelike-skewed (120C/100U/90R/50L, power scales)."""
    return _generate_pool_standard(ROGUELIKE_RARITY, scaled_power)


def generate_pool_D() -> list:
    """Model D: Rarity-symbol correlation (rares have more symbols)."""
    def sym_fn(rarity: Rarity) -> int:
        if rarity == Rarity.COMMON:
            return random.choices([1, 2], weights=[0.60, 0.40])[0]
        elif rarity == Rarity.UNCOMMON:
            return random.choices([1, 2, 3], weights=[0.15, 0.60, 0.25])[0]
        elif rarity == Rarity.RARE:
            return random.choices([2, 3], weights=[0.55, 0.45])[0]
        else:  # Legendary
            return random.choices([2, 3], weights=[0.30, 0.70])[0]

    return _generate_pool_symbol_correlated(STANDARD_RARITY, scaled_power, sym_fn)


def generate_pool_E() -> list:
    """Model E: Inverse correlation (rares have fewer symbols)."""
    def sym_fn(rarity: Rarity) -> int:
        if rarity == Rarity.COMMON:
            return random.choices([2, 3], weights=[0.65, 0.35])[0]
        elif rarity == Rarity.UNCOMMON:
            return random.choices([1, 2, 3], weights=[0.20, 0.55, 0.25])[0]
        elif rarity == Rarity.RARE:
            return random.choices([1, 2], weights=[0.70, 0.30])[0]
        else:  # Legendary
            return 1  # Legendaries always single-symbol

    return _generate_pool_symbol_correlated(STANDARD_RARITY, scaled_power, sym_fn)


# ---------------------------------------------------------------------------
# Pair-Escalation Slots algorithm
# ---------------------------------------------------------------------------

def get_top_pair(pair_counts: dict):
    """Return (top_pair, count) or (None, 0)."""
    if not pair_counts:
        return None, 0
    top = max(pair_counts.items(), key=lambda x: x[1])
    return top[0], top[1]


def generate_pack_pair_escalation(pool: list, pair_counts: dict,
                                   pair_index: dict) -> list:
    """
    Pair-Escalation Slots: Each pack slot independently shows a card matching
    the player's most common pair with probability min(top_count / K, cap),
    otherwise a random card.
    """
    top_pair, top_count = get_top_pair(pair_counts)
    prob = min(top_count / PE_K, PE_CAP) if top_pair else 0.0

    pair_matched = pair_index.get(top_pair, []) if top_pair else []

    pack = []
    used_ids = set()

    for _ in range(PACK_SIZE):
        if pair_matched and random.random() < prob:
            chosen = random.choice(pair_matched)
        else:
            candidates = [c for c in pool if c.id not in used_ids]
            if not candidates:
                candidates = pool
            chosen = random.choice(candidates)
        pack.append(chosen)
        used_ids.add(chosen.id)

    return pack


def build_pair_index(pool: list) -> dict:
    """Pre-build index: ordered_pair -> list of cards."""
    index = defaultdict(list)
    for c in pool:
        p = c.ordered_pair()
        if p:
            index[p].append(c)
    return dict(index)


# ---------------------------------------------------------------------------
# Player strategies
# ---------------------------------------------------------------------------

def evaluate_archetype_strength(drafted: list) -> dict:
    scores = {name: 0 for name in ARCHETYPE_NAMES}
    for card in drafted:
        for arch, tier in card.archetype_fitness.items():
            scores[arch] += TIER_VAL[tier]
    return scores


def card_is_sa(card: SimCard, arch: str) -> bool:
    return card.archetype_fitness.get(arch, "F") in ("S", "A")


def card_is_cf(card: SimCard, arch: str) -> bool:
    return card.archetype_fitness.get(arch, "F") in ("C", "F")


def pick_committed(pack: list, drafted: list, committed_arch: Optional[str],
                   pick_num: int) -> tuple:
    """Archetype-committed player. Commits at pick 5."""
    if committed_arch is None and pick_num >= 5 and drafted:
        scores = evaluate_archetype_strength(drafted)
        committed_arch = max(scores, key=scores.get)

    if committed_arch is None:
        if drafted:
            scores = evaluate_archetype_strength(drafted)
            best = max(scores, key=scores.get)
        else:
            best = random.choice(ARCHETYPE_NAMES)
        chosen = max(pack, key=lambda c: (
            TIER_VAL.get(c.archetype_fitness.get(best, "F"), 0), c.power))
    else:
        chosen = max(pack, key=lambda c: (
            TIER_VAL.get(c.archetype_fitness.get(committed_arch, "F"), 0), c.power))

    return chosen, committed_arch


def pick_power_chaser(pack: list) -> SimCard:
    """Always picks highest raw power."""
    return max(pack, key=lambda c: c.power)


def pick_signal_reader(pack: list, drafted: list, pair_counts: dict,
                       committed_arch: Optional[str], pick_num: int) -> tuple:
    """Signal reader: reads pairs and resonance from pack offerings."""
    if committed_arch is None and pick_num >= 5 and drafted:
        # Commit based on pair counts first, then resonance fallback
        pair_to_arch = {}
        for i, (name, pri, sec) in enumerate(ARCHETYPES):
            pair_to_arch[(pri, sec)] = name
        arch_scores = defaultdict(float)
        for (p, s), cnt in pair_counts.items():
            arch = pair_to_arch.get((p, s))
            if arch:
                arch_scores[arch] += cnt
        if arch_scores:
            committed_arch = max(arch_scores, key=arch_scores.get)
        else:
            scores = evaluate_archetype_strength(drafted)
            committed_arch = max(scores, key=scores.get)

    if pick_num < 5:
        # Early: mix of power and resonance signal
        return max(pack, key=lambda c: c.power + random.uniform(-1, 1)), committed_arch
    else:
        if committed_arch:
            chosen = max(pack, key=lambda c: (
                TIER_VAL.get(c.archetype_fitness.get(committed_arch, "F"), 0) * 0.85 +
                c.power * 0.15))
            return chosen, committed_arch
        return max(pack, key=lambda c: c.power), committed_arch


# ---------------------------------------------------------------------------
# Draft metrics
# ---------------------------------------------------------------------------

@dataclass
class DraftMetrics:
    # Standard targets
    early_unique_archetypes: list = field(default_factory=list)
    early_sa_for_target: list = field(default_factory=list)
    late_sa_per_pack: list = field(default_factory=list)
    late_cf_per_pack: list = field(default_factory=list)
    convergence_pick: int = 30
    deck_concentration: float = 0.0
    drafted_ids: list = field(default_factory=list)

    # Rarity-specific metrics
    total_power: float = 0.0
    rarity_counts_drafted: dict = field(default_factory=lambda: {r: 0 for r in Rarity})
    pairs_contributed_by_rarity: dict = field(default_factory=lambda: {r: 0 for r in Rarity})
    pair_count_at_pick: list = field(default_factory=list)  # top pair count after each pick

    # Tension metrics: packs where highest-power card differs from highest-fitness card
    tension_packs_late: int = 0
    total_packs_late: int = 0

    # Rarity of the card with highest power vs highest fitness
    tension_rare_power_common_fitness: int = 0  # rare has highest power, common has highest fitness
    total_tension_eligible: int = 0

    final_archetype: Optional[str] = None
    strategy: str = ""


# ---------------------------------------------------------------------------
# Single draft runner
# ---------------------------------------------------------------------------

def run_single_draft(pool: list, pair_index: dict, strategy: str) -> DraftMetrics:
    pair_counts = defaultdict(int)
    drafted = []
    metrics = DraftMetrics(strategy=strategy)
    committed_arch = None
    convergence_streak = 0
    convergence_found = False

    for pick_num in range(NUM_PICKS):
        pack = generate_pack_pair_escalation(pool, dict(pair_counts), pair_index)

        if not pack:
            break

        # Current best archetype for evaluation
        if drafted:
            scores = evaluate_archetype_strength(drafted)
            current_best = max(scores, key=scores.get)
        else:
            current_best = None

        # Early metrics (picks 0-4)
        if pick_num < 5:
            unique_archs = set()
            for c in pack:
                for a in ARCHETYPE_NAMES:
                    if card_is_sa(c, a):
                        unique_archs.add(a)
            metrics.early_unique_archetypes.append(len(unique_archs))
            if current_best:
                sa = sum(1 for c in pack if card_is_sa(c, current_best))
                metrics.early_sa_for_target.append(sa)

        # Late metrics (picks 5+)
        if pick_num >= 5 and current_best:
            metrics.total_packs_late += 1
            sa = sum(1 for c in pack if card_is_sa(c, current_best))
            cf = sum(1 for c in pack if card_is_cf(c, current_best))
            metrics.late_sa_per_pack.append(sa)
            metrics.late_cf_per_pack.append(cf)

            # Convergence: 3 consecutive packs with sa >= 2
            if sa >= 2:
                convergence_streak += 1
            else:
                convergence_streak = 0
            if convergence_streak >= 3 and not convergence_found:
                metrics.convergence_pick = pick_num - 2 + 1  # 1-indexed
                convergence_found = True

            # Tension: highest-power card vs highest-fitness card differ
            highest_power_card = max(pack, key=lambda c: c.power)
            highest_fitness_card = max(pack, key=lambda c: (
                TIER_VAL.get(c.archetype_fitness.get(current_best, "F"), 0), c.power))

            if highest_power_card.id != highest_fitness_card.id:
                # Check if power gap is meaningful (>= 1.5)
                power_gap = highest_power_card.power - highest_fitness_card.power
                if power_gap >= 1.5:
                    metrics.tension_packs_late += 1

                    # Is the powerful card rare and the fit card common?
                    if (highest_power_card.rarity in (Rarity.RARE, Rarity.LEGENDARY) and
                            highest_fitness_card.rarity in (Rarity.COMMON, Rarity.UNCOMMON)):
                        metrics.tension_rare_power_common_fitness += 1
                    metrics.total_tension_eligible += 1

        # Pick card
        if strategy == "committed":
            chosen, committed_arch = pick_committed(pack, drafted, committed_arch, pick_num)
        elif strategy == "power":
            chosen = pick_power_chaser(pack)
            if drafted:
                scores = evaluate_archetype_strength(drafted)
                committed_arch = max(scores, key=scores.get)
        elif strategy == "signal":
            chosen, committed_arch = pick_signal_reader(
                pack, drafted, dict(pair_counts), committed_arch, pick_num)
        else:
            chosen = random.choice(pack)

        drafted.append(chosen)
        metrics.drafted_ids.append(chosen.id)
        metrics.total_power += chosen.power
        metrics.rarity_counts_drafted[chosen.rarity] += 1

        # Update pair counts
        pair = chosen.ordered_pair()
        if pair:
            pair_counts[pair] += 1
            metrics.pairs_contributed_by_rarity[chosen.rarity] += 1

        # Track top pair count after each pick
        _, top_count = get_top_pair(dict(pair_counts))
        metrics.pair_count_at_pick.append(top_count)

    # Final archetype
    if drafted:
        scores = evaluate_archetype_strength(drafted)
        best = max(scores, key=scores.get)
        sa_count = sum(1 for c in drafted if card_is_sa(c, best))
        metrics.deck_concentration = sa_count / len(drafted)
        metrics.final_archetype = best

    return metrics


# ---------------------------------------------------------------------------
# Aggregate metrics
# ---------------------------------------------------------------------------

def compute_aggregate(all_metrics: list) -> dict:
    s = {}

    all_early_archs = [v for m in all_metrics for v in m.early_unique_archetypes]
    s["early_unique_archs"] = statistics.mean(all_early_archs) if all_early_archs else 0

    all_early_sa = [v for m in all_metrics for v in m.early_sa_for_target]
    s["early_sa_target"] = statistics.mean(all_early_sa) if all_early_sa else 0

    all_late_sa = [v for m in all_metrics for v in m.late_sa_per_pack]
    s["late_sa_committed"] = statistics.mean(all_late_sa) if all_late_sa else 0

    all_late_cf = [v for m in all_metrics for v in m.late_cf_per_pack]
    s["late_off_arch"] = statistics.mean(all_late_cf) if all_late_cf else 0

    conv_picks = [m.convergence_pick for m in all_metrics]
    s["convergence_pick"] = statistics.mean(conv_picks)

    s["deck_concentration"] = statistics.mean([m.deck_concentration for m in all_metrics])

    # Late S/A stddev
    s["late_sa_stddev"] = statistics.stdev(all_late_sa) if len(all_late_sa) > 1 else 0

    # Card overlap (sample pairs)
    if len(all_metrics) >= 2:
        overlaps = []
        sample_size = min(200, len(all_metrics))
        sampled = random.sample(all_metrics, sample_size)
        for i in range(0, sample_size - 1, 2):
            s1 = set(sampled[i].drafted_ids)
            s2 = set(sampled[i + 1].drafted_ids)
            if s1 | s2:
                overlaps.append(len(s1 & s2) / len(s1 | s2))
        s["card_overlap"] = statistics.mean(overlaps) if overlaps else 0
    else:
        s["card_overlap"] = 0

    # Power stats
    all_powers = [m.total_power / NUM_PICKS for m in all_metrics]
    s["avg_power"] = statistics.mean(all_powers)
    s["power_stdev"] = statistics.stdev(all_powers) if len(all_powers) > 1 else 0

    # Archetype frequency
    arch_counts = Counter(m.final_archetype for m in all_metrics if m.final_archetype)
    total = sum(arch_counts.values())
    s["arch_freq"] = {k: v / total for k, v in arch_counts.items()} if total else {}

    # Tension rate
    total_tension = sum(m.tension_packs_late for m in all_metrics)
    total_late = sum(m.total_packs_late for m in all_metrics)
    s["tension_rate"] = total_tension / total_late * 100 if total_late else 0

    # Rare-power-vs-common-fitness tension breakdown
    total_rp_cf = sum(m.tension_rare_power_common_fitness for m in all_metrics)
    total_eligible = sum(m.total_tension_eligible for m in all_metrics)
    s["tension_rare_vs_common_pct"] = total_rp_cf / total_eligible * 100 if total_eligible else 0

    # Pair contribution by rarity
    rarity_pair_totals = {r: 0 for r in Rarity}
    for m in all_metrics:
        for r, cnt in m.pairs_contributed_by_rarity.items():
            rarity_pair_totals[r] += cnt
    total_pairs = sum(rarity_pair_totals.values())
    s["pair_contrib_by_rarity"] = {
        RARITY_SHORT[r]: cnt / total_pairs * 100 if total_pairs else 0
        for r, cnt in rarity_pair_totals.items()
    }

    # Average pair count by pick (convergence speed)
    max_picks = max(len(m.pair_count_at_pick) for m in all_metrics)
    s["avg_pair_curve"] = []
    for i in range(max_picks):
        vals = [m.pair_count_at_pick[i] for m in all_metrics if i < len(m.pair_count_at_pick)]
        s["avg_pair_curve"].append(statistics.mean(vals) if vals else 0)

    # Rarity drafted breakdown
    rarity_totals = {r: 0 for r in Rarity}
    for m in all_metrics:
        for r, cnt in m.rarity_counts_drafted.items():
            rarity_totals[r] += cnt
    total_drafted = sum(rarity_totals.values())
    s["rarity_drafted"] = {
        RARITY_SHORT[r]: cnt / total_drafted * 100 if total_drafted else 0
        for r, cnt in rarity_totals.items()
    }

    # Convergence speed: average top pair count at picks 5, 10, 15, 20
    s["pair_at_milestones"] = {}
    for milestone in [5, 10, 15, 20]:
        vals = [m.pair_count_at_pick[milestone - 1] for m in all_metrics
                if len(m.pair_count_at_pick) >= milestone]
        s["pair_at_milestones"][milestone] = statistics.mean(vals) if vals else 0

    return s


# ---------------------------------------------------------------------------
# Pool statistics
# ---------------------------------------------------------------------------

def pool_stats(pool: list, label: str) -> dict:
    stats = {}
    rarity_counts = Counter(c.rarity for c in pool)
    stats["rarity"] = {RARITY_SHORT[r]: rarity_counts.get(r, 0) for r in Rarity}
    stats["avg_power"] = statistics.mean(c.power for c in pool)

    stats["power_by_rarity"] = {}
    for r in Rarity:
        cards_r = [c for c in pool if c.rarity == r]
        if cards_r:
            stats["power_by_rarity"][RARITY_SHORT[r]] = statistics.mean(c.power for c in cards_r)

    sym_counts = Counter(len(c.symbols) for c in pool)
    stats["sym_dist"] = dict(sorted(sym_counts.items()))

    stats["sym_by_rarity"] = {}
    for r in Rarity:
        cards_r = [c for c in pool if c.rarity == r and c.symbols]
        if cards_r:
            stats["sym_by_rarity"][RARITY_SHORT[r]] = statistics.mean(len(c.symbols) for c in cards_r)

    # Cards with 2+ symbols (pair contributors) by rarity
    stats["pair_contributors_by_rarity"] = {}
    for r in Rarity:
        cards_r = [c for c in pool if c.rarity == r]
        pair_cards = [c for c in cards_r if c.ordered_pair() is not None]
        stats["pair_contributors_by_rarity"][RARITY_SHORT[r]] = (
            len(pair_cards) / len(cards_r) * 100 if cards_r else 0)

    print(f"\n  Pool: {label}")
    print(f"    Rarity counts: {stats['rarity']}")
    print(f"    Avg power: {stats['avg_power']:.2f}")
    pwr_str = ', '.join(f'{k}={v:.2f}' for k, v in stats['power_by_rarity'].items())
    print(f"    Power by rarity: {pwr_str}")
    print(f"    Symbol dist: {stats['sym_dist']}")
    sym_str = ', '.join(f'{k}={v:.2f}' for k, v in stats['sym_by_rarity'].items())
    print(f"    Avg symbols by rarity: {sym_str}")
    pair_str = ', '.join(f'{k}={v:.0f}%' for k, v in stats['pair_contributors_by_rarity'].items())
    print(f"    Pair contributors by rarity: {pair_str}")

    return stats


# ---------------------------------------------------------------------------
# Pass/fail evaluation
# ---------------------------------------------------------------------------

def count_passes(agg: dict) -> int:
    passes = 0
    if agg["early_unique_archs"] >= 3:
        passes += 1
    if agg["early_sa_target"] <= 2:
        passes += 1
    if agg["late_sa_committed"] >= 2:
        passes += 1
    if agg["late_off_arch"] >= 0.5:
        passes += 1
    if 5 <= agg["convergence_pick"] <= 8:
        passes += 1
    if 0.60 <= agg["deck_concentration"] <= 0.90:
        passes += 1
    if agg["card_overlap"] < 0.40:
        passes += 1
    if agg.get("late_sa_stddev", 0) >= 0.8:
        passes += 1
    freq = agg.get("arch_freq", {})
    if freq:
        max_f = max(freq.values())
        min_f = min(freq.values())
        if max_f <= 0.20 and min_f >= 0.05:
            passes += 1
    return passes


# ---------------------------------------------------------------------------
# Printing
# ---------------------------------------------------------------------------

def print_scorecard(agg: dict, label: str):
    print(f"\n{'='*70}")
    print(f"  SCORECARD: {label}")
    print(f"{'='*70}")

    targets = [
        ("Picks 1-5: unique archetypes w/ S/A", "early_unique_archs", ">= 3",
         lambda x: x >= 3),
        ("Picks 1-5: S/A for target per pack", "early_sa_target", "<= 2",
         lambda x: x <= 2),
        ("Picks 6+: S/A for committed per pack", "late_sa_committed", ">= 2",
         lambda x: x >= 2.0),
        ("Picks 6+: off-archetype (C/F) per pack", "late_off_arch", ">= 0.5",
         lambda x: x >= 0.5),
        ("Convergence pick", "convergence_pick", "5-8",
         lambda x: 5 <= x <= 8),
        ("Deck concentration", "deck_concentration", "60-90%",
         lambda x: 0.60 <= x <= 0.90),
        ("Run-to-run overlap", "card_overlap", "< 40%",
         lambda x: x < 0.40),
        ("Late S/A stddev", "late_sa_stddev", ">= 0.8",
         lambda x: x >= 0.8),
    ]

    print(f"  {'Metric':<48} {'Target':<10} {'Actual':<10} {'P/F'}")
    print(f"  {'-'*78}")
    for name, key, target_str, check_fn in targets:
        val = agg.get(key, 0)
        if "concentration" in key or "overlap" in key:
            val_str = f"{val:.1%}"
        else:
            val_str = f"{val:.2f}"
        passed = "PASS" if check_fn(val) else "FAIL"
        print(f"  {name:<48} {target_str:<10} {val_str:<10} {passed}")

    freq = agg.get("arch_freq", {})
    if freq:
        max_f = max(freq.values())
        min_f = min(freq.values())
        arch_pass = "PASS" if max_f <= 0.20 and min_f >= 0.05 else "FAIL"
        print(f"  {'Archetype frequency':<48} {'5-20%':<10} {f'{min_f:.1%}-{max_f:.1%}':<10} {arch_pass}")


def print_rarity_details(agg: dict, label: str):
    print(f"\n  --- Rarity Details: {label} ---")

    rd = agg["rarity_drafted"]
    print(f"  Rarity drafted (C/U/R/L): {rd['C']:.1f}% / {rd['U']:.1f}% / {rd['R']:.1f}% / {rd['L']:.1f}%")

    pc = agg["pair_contrib_by_rarity"]
    print(f"  Pair contributions (C/U/R/L): {pc['C']:.1f}% / {pc['U']:.1f}% / {pc['R']:.1f}% / {pc['L']:.1f}%")

    print(f"  Avg power per card: {agg['avg_power']:.2f}")
    print(f"  Power stdev across runs: {agg['power_stdev']:.3f}")
    print(f"  Tension rate (overall): {agg['tension_rate']:.1f}%")
    print(f"  Tension: rare-power vs common-fitness: {agg['tension_rare_vs_common_pct']:.1f}%")

    pm = agg["pair_at_milestones"]
    print(f"  Avg top pair count at picks 5/10/15/20: "
          f"{pm[5]:.1f} / {pm[10]:.1f} / {pm[15]:.1f} / {pm[20]:.1f}")


# ---------------------------------------------------------------------------
# Main simulation
# ---------------------------------------------------------------------------

def main():
    random.seed(42)

    print("=" * 70)
    print("  RARITY x PAIR-ESCALATION SLOTS SIMULATION")
    print(f"  K={PE_K}, Cap={PE_CAP}, {PACK_SIZE} slots, {NUM_PICKS} picks")
    print(f"  {NUM_DRAFTS} drafts per model")
    print("=" * 70)

    models = {
        "A: Flat Rarity (cosmetic, uniform power)": generate_pool_A,
        "B: Standard TCG (180C/100U/60R/20L, scaled power)": generate_pool_B,
        "C: Roguelike-Skewed (120C/100U/90R/50L, scaled power)": generate_pool_C,
        "D: Rarity-Symbol Correlation (rares = more symbols)": generate_pool_D,
        "E: Inverse Correlation (rares = fewer symbols)": generate_pool_E,
    }

    all_results = {}

    for model_name, gen_fn in models.items():
        print(f"\n\n{'='*70}")
        print(f"  Generating: {model_name}")

        random.seed(42)
        pool = gen_fn()
        pstats = pool_stats(pool, model_name)
        pair_index = build_pair_index(pool)

        # Run committed, power, and signal strategies
        random.seed(100)
        print(f"  Running {NUM_DRAFTS} committed drafts...")
        committed_metrics = []
        for _ in range(NUM_DRAFTS):
            m = run_single_draft(pool, pair_index, "committed")
            committed_metrics.append(m)
        agg_committed = compute_aggregate(committed_metrics)

        random.seed(200)
        print(f"  Running {NUM_DRAFTS} power-chaser drafts...")
        power_metrics = []
        for _ in range(NUM_DRAFTS):
            m = run_single_draft(pool, pair_index, "power")
            power_metrics.append(m)
        agg_power = compute_aggregate(power_metrics)

        random.seed(300)
        print(f"  Running {NUM_DRAFTS} signal-reader drafts...")
        signal_metrics = []
        for _ in range(NUM_DRAFTS):
            m = run_single_draft(pool, pair_index, "signal")
            signal_metrics.append(m)
        agg_signal = compute_aggregate(signal_metrics)

        print_scorecard(agg_committed, f"{model_name} (Committed)")
        print_rarity_details(agg_committed, "Committed")

        passes = count_passes(agg_committed)
        print(f"\n  TARGETS PASSED: {passes}/9")

        all_results[model_name] = {
            "committed": agg_committed,
            "power": agg_power,
            "signal": agg_signal,
            "pool_stats": pstats,
            "passes": passes,
        }

    # =========================================================================
    # Summary Comparison Table
    # =========================================================================
    print("\n\n" + "=" * 70)
    print("  SUMMARY COMPARISON TABLE (Committed Player)")
    print("=" * 70)

    short_names = ["A:Flat", "B:TCG", "C:Rogue", "D:SymCor", "E:InvCor"]
    model_keys = list(models.keys())

    def fmt_header():
        h = f"  {'Metric':<40} {'Target':<10}"
        for sn in short_names:
            h += f" {sn:>10}"
        return h

    print(fmt_header())
    print(f"  {'-'*40} {'-'*10}" + " " + " ".join(["-" * 10] * len(short_names)))

    def gc(mk, key):
        return all_results[mk]["committed"][key]

    metric_rows = [
        ("Early unique archs w/ S/A", ">= 3",
         lambda mk: f"{gc(mk, 'early_unique_archs'):.2f}"),
        ("Early S/A for target", "<= 2",
         lambda mk: f"{gc(mk, 'early_sa_target'):.2f}"),
        ("Late S/A committed", ">= 2",
         lambda mk: f"{gc(mk, 'late_sa_committed'):.2f}"),
        ("Late off-archetype (C/F)", ">= 0.5",
         lambda mk: f"{gc(mk, 'late_off_arch'):.2f}"),
        ("Convergence pick", "5-8",
         lambda mk: f"{gc(mk, 'convergence_pick'):.1f}"),
        ("Deck concentration", "60-90%",
         lambda mk: f"{gc(mk, 'deck_concentration')*100:.1f}%"),
        ("Card overlap", "< 40%",
         lambda mk: f"{gc(mk, 'card_overlap')*100:.1f}%"),
        ("Late S/A stddev", ">= 0.8",
         lambda mk: f"{gc(mk, 'late_sa_stddev'):.2f}"),
        ("Targets passed", "9/9",
         lambda mk: f"{all_results[mk]['passes']}/9"),
        ("---", "---", lambda mk: "---"),
        ("Tension rate (overall)", "",
         lambda mk: f"{gc(mk, 'tension_rate'):.1f}%"),
        ("Tension: rare-pwr vs common-fit", "",
         lambda mk: f"{gc(mk, 'tension_rare_vs_common_pct'):.1f}%"),
        ("Avg power (committed)", "",
         lambda mk: f"{gc(mk, 'avg_power'):.2f}"),
        ("Power stdev", "",
         lambda mk: f"{gc(mk, 'power_stdev'):.3f}"),
        ("Avg power (power chaser)", "",
         lambda mk: f"{all_results[mk]['power']['avg_power']:.2f}"),
        ("Power gap (chaser - committed)", "",
         lambda mk: f"{all_results[mk]['power']['avg_power'] - gc(mk, 'avg_power'):.2f}"),
        ("Pair contrib C%", "",
         lambda mk: f"{gc(mk, 'pair_contrib_by_rarity')['C']:.1f}%"),
        ("Pair contrib R+L%", "",
         lambda mk: f"{gc(mk, 'pair_contrib_by_rarity')['R'] + gc(mk, 'pair_contrib_by_rarity')['L']:.1f}%"),
        ("Top pair at pick 10", "",
         lambda mk: f"{gc(mk, 'pair_at_milestones')[10]:.1f}"),
        ("Top pair at pick 20", "",
         lambda mk: f"{gc(mk, 'pair_at_milestones')[20]:.1f}"),
    ]

    for name, target, fn in metric_rows:
        line = f"  {name:<40} {target:<10}"
        for mk in model_keys:
            val = fn(mk)
            line += f" {val:>10}"
        print(line)

    # =========================================================================
    # Convergence Speed Comparison
    # =========================================================================
    print("\n\n" + "=" * 70)
    print("  CONVERGENCE SPEED: TOP PAIR COUNT AT MILESTONES")
    print("=" * 70)

    print(f"\n  {'Pick':<10}", end="")
    for sn in short_names:
        print(f" {sn:>10}", end="")
    print()
    print(f"  {'-'*10}" + " " + " ".join(["-" * 10] * len(short_names)))

    for pick in [3, 5, 8, 10, 15, 20, 25]:
        line = f"  {pick:<10}"
        for mk in model_keys:
            curve = all_results[mk]["committed"]["avg_pair_curve"]
            val = curve[pick - 1] if pick - 1 < len(curve) else 0
            line += f" {val:>10.2f}"
        print(line)

    # =========================================================================
    # Strategy Comparison
    # =========================================================================
    print("\n\n" + "=" * 70)
    print("  STRATEGY COMPARISON ACROSS MODELS")
    print("=" * 70)

    for mk, mname in zip(model_keys, short_names):
        r = all_results[mk]
        print(f"\n  {mname}:")
        for strat_name, strat_key in [("Committed", "committed"), ("Power", "power"), ("Signal", "signal")]:
            ag = r[strat_key]
            print(f"    {strat_name:12s}: Late S/A={ag['late_sa_committed']:.2f}, "
                  f"DeckConc={ag['deck_concentration']:.1%}, "
                  f"AvgPwr={ag['avg_power']:.2f}, "
                  f"Tension={ag['tension_rate']:.1f}%")

    # =========================================================================
    # Pair Contribution Analysis
    # =========================================================================
    print("\n\n" + "=" * 70)
    print("  PAIR CONTRIBUTION ANALYSIS BY RARITY")
    print("=" * 70)

    print(f"\n  {'Model':<12} {'C pairs%':>10} {'U pairs%':>10} {'R pairs%':>10} {'L pairs%':>10} "
          f"{'C drafted%':>10} {'R+L drafted%':>12}")
    print(f"  {'-'*76}")
    for mk, sn in zip(model_keys, short_names):
        pc = all_results[mk]["committed"]["pair_contrib_by_rarity"]
        rd = all_results[mk]["committed"]["rarity_drafted"]
        print(f"  {sn:<12} {pc['C']:>10.1f} {pc['U']:>10.1f} {pc['R']:>10.1f} {pc['L']:>10.1f} "
              f"{rd['C']:>10.1f} {rd['R'] + rd['L']:>12.1f}")

    # =========================================================================
    # Key Finding: Rarity-Pair Interaction Effect Size
    # =========================================================================
    print("\n\n" + "=" * 70)
    print("  KEY FINDING: RARITY-PAIR INTERACTION EFFECT SIZES")
    print("=" * 70)

    # Compare D vs E (positive vs inverse correlation)
    d_agg = all_results[model_keys[3]]["committed"]
    e_agg = all_results[model_keys[4]]["committed"]
    b_agg = all_results[model_keys[1]]["committed"]

    print(f"\n  Baseline (B: Standard TCG):")
    print(f"    Late S/A: {b_agg['late_sa_committed']:.2f}, Conv: {b_agg['convergence_pick']:.1f}, "
          f"Tension: {b_agg['tension_rate']:.1f}%")

    print(f"\n  Model D (rares = more symbols, accelerate pairs):")
    print(f"    Late S/A: {d_agg['late_sa_committed']:.2f} ({d_agg['late_sa_committed'] - b_agg['late_sa_committed']:+.2f} vs B)")
    print(f"    Conv pick: {d_agg['convergence_pick']:.1f} ({d_agg['convergence_pick'] - b_agg['convergence_pick']:+.1f} vs B)")
    print(f"    Tension: {d_agg['tension_rate']:.1f}% ({d_agg['tension_rate'] - b_agg['tension_rate']:+.1f}pp vs B)")
    print(f"    R+L pair contrib: {d_agg['pair_contrib_by_rarity']['R'] + d_agg['pair_contrib_by_rarity']['L']:.1f}%")

    print(f"\n  Model E (rares = fewer symbols, stall pairs):")
    print(f"    Late S/A: {e_agg['late_sa_committed']:.2f} ({e_agg['late_sa_committed'] - b_agg['late_sa_committed']:+.2f} vs B)")
    print(f"    Conv pick: {e_agg['convergence_pick']:.1f} ({e_agg['convergence_pick'] - b_agg['convergence_pick']:+.1f} vs B)")
    print(f"    Tension: {e_agg['tension_rate']:.1f}% ({e_agg['tension_rate'] - b_agg['tension_rate']:+.1f}pp vs B)")
    print(f"    R+L pair contrib: {e_agg['pair_contrib_by_rarity']['R'] + e_agg['pair_contrib_by_rarity']['L']:.1f}%")

    d_e_diff = d_agg['late_sa_committed'] - e_agg['late_sa_committed']
    d_e_conv_diff = d_agg['convergence_pick'] - e_agg['convergence_pick']
    d_e_tension_diff = d_agg['tension_rate'] - e_agg['tension_rate']
    print(f"\n  D vs E delta:")
    print(f"    Late S/A: {d_e_diff:+.2f}")
    print(f"    Conv pick: {d_e_conv_diff:+.1f}")
    print(f"    Tension rate: {d_e_tension_diff:+.1f}pp")

    print("\n\n  Simulation complete.")


if __name__ == "__main__":
    main()
