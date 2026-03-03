#!/usr/bin/env python3
"""
Pool Synthesis Simulation for Lane Locking Draft System.

Reconciles findings from 5 parallel investigations:
  1. Symbol count ratios (70/20/10 vs 25/55/20 etc.)
  2. Rarity system (TCG distribution, orthogonal to locking)
  3. Archetype breakdown (40 per archetype + 40 generic)
  4. Symbol patterns (7 pattern types for 83% genuine choice)
  5. Threshold tuning ((5,12) vs (3,8) vs (4,10))

Tests the RECONCILED pool design against the original default.
"""

import random
import statistics
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional

# ============================================================
# Core types
# ============================================================

class Resonance(Enum):
    EMBER = "Ember"
    STONE = "Stone"
    TIDE = "Tide"
    ZEPHYR = "Zephyr"

class Rarity(Enum):
    COMMON = "Common"
    UNCOMMON = "Uncommon"
    RARE = "Rare"
    LEGENDARY = "Legendary"

class Tier(Enum):
    S = "S"
    A = "A"
    B = "B"
    C = "C"
    F = "F"

POWER_RANGES = {
    Rarity.COMMON: (2.0, 5.0),
    Rarity.UNCOMMON: (4.0, 7.0),
    Rarity.RARE: (6.0, 9.0),
    Rarity.LEGENDARY: (8.0, 10.0),
}

# ============================================================
# Archetype circle
# ============================================================

# (name, primary_resonance, secondary_resonance)
ARCHETYPES = [
    ("Flash", Resonance.ZEPHYR, Resonance.EMBER),       # 1
    ("Blink", Resonance.EMBER, Resonance.ZEPHYR),        # 2
    ("Storm", Resonance.EMBER, Resonance.STONE),          # 3
    ("SelfDiscard", Resonance.STONE, Resonance.EMBER),    # 4
    ("SelfMill", Resonance.STONE, Resonance.TIDE),        # 5
    ("Sacrifice", Resonance.TIDE, Resonance.STONE),       # 6
    ("Warriors", Resonance.TIDE, Resonance.ZEPHYR),       # 7
    ("Ramp", Resonance.ZEPHYR, Resonance.TIDE),           # 8
]

# Build adjacency. On a circle, neighbors are +1 and -1 (mod 8).
ARCHETYPE_NAMES = [a[0] for a in ARCHETYPES]
ARCHETYPE_INDEX = {a[0]: i for i, a in enumerate(ARCHETYPES)}


def adjacent_indices(idx: int) -> list[int]:
    return [(idx - 1) % 8, (idx + 1) % 8]


def archetype_fitness(card_arch_idx: int, target_arch_idx: int) -> Tier:
    """Compute fitness tier of a card from card_arch for target_arch."""
    if card_arch_idx == target_arch_idx:
        return Tier.S
    if target_arch_idx in adjacent_indices(card_arch_idx):
        return Tier.A
    # Check if they share any resonance
    card_pri = ARCHETYPES[card_arch_idx][1]
    card_sec = ARCHETYPES[card_arch_idx][2]
    target_pri = ARCHETYPES[target_arch_idx][1]
    target_sec = ARCHETYPES[target_arch_idx][2]
    shared = set()
    if card_pri == target_pri or card_pri == target_sec:
        shared.add(card_pri)
    if card_sec == target_pri or card_sec == target_sec:
        shared.add(card_sec)
    if shared:
        return Tier.B
    return Tier.F


def generic_fitness() -> Tier:
    return Tier.B


# ============================================================
# Card generation
# ============================================================

@dataclass
class SimCard:
    id: int
    symbols: list[Resonance]
    archetype: Optional[str]  # None for generic
    rarity: Rarity
    power: float
    _fitness_cache: dict = field(default_factory=dict, repr=False)

    def primary_resonance(self) -> Optional[Resonance]:
        return self.symbols[0] if self.symbols else None

    def fitness_for(self, target_arch: str) -> Tier:
        if target_arch in self._fitness_cache:
            return self._fitness_cache[target_arch]
        if self.archetype is None:
            tier = generic_fitness()
        else:
            tier = archetype_fitness(
                ARCHETYPE_INDEX[self.archetype],
                ARCHETYPE_INDEX[target_arch],
            )
        self._fitness_cache[target_arch] = tier
        return tier


def assign_rarity_tcg(cards: list[SimCard], rarity_counts: dict[Rarity, int]):
    """Assign rarities following TCG distribution."""
    slots = []
    for r, count in rarity_counts.items():
        slots.extend([r] * count)
    random.shuffle(slots)
    for i, card in enumerate(cards):
        card.rarity = slots[i % len(slots)]
        lo, hi = POWER_RANGES[card.rarity]
        card.power = random.uniform(lo, hi)


def make_symbols(primary: Resonance, secondary: Resonance,
                 pattern: str) -> list[Resonance]:
    """Generate symbol list from pattern template and archetype resonances."""
    mapping = {
        "mono_primary": [primary],
        "standard_dual": [primary, secondary],
        "double_primary": [primary, primary],
        "secondary_led": [secondary, primary],
        "deep_commit": [primary, primary, secondary],
        "balanced_triple": [primary, secondary, secondary],
        "cross_bridge": None,  # handled specially
    }
    if pattern == "cross_bridge":
        # Primary + a resonance that's NOT primary or secondary
        others = [r for r in Resonance if r != primary and r != secondary]
        return [primary, random.choice(others)]
    return mapping[pattern]


# ============================================================
# Pool designs
# ============================================================

# Pattern mix: fraction of each archetype's ~40 cards
RECONCILED_PATTERN_MIX = {
    # Agent 4's recommended mix, reconciled with Agent 3:
    # - 25% mono_primary: clean identity, slow accumulation (Agent 1 influence)
    # - 25% standard_dual: signature pattern, most archetype-identifying
    # - 10% double_primary: deep commit, acceleration cards
    # - 10% secondary_led: bridge pivot cards
    # - 15% deep_commit_triple: strong commit signal (PPS)
    # - 10% balanced_triple: breadth option (PSS)
    # - 5% cross_bridge: off-archetype splash
    "mono_primary": 0.25,
    "standard_dual": 0.25,
    "double_primary": 0.10,
    "secondary_led": 0.10,
    "deep_commit": 0.15,
    "balanced_triple": 0.10,
    "cross_bridge": 0.05,
}

DEFAULT_PATTERN_MIX = {
    # Original default: 25/55/20 symbol counts, mapped to patterns
    # 25% 1-symbol = mono_primary
    # 55% 2-symbol = split across standard_dual, double_primary, secondary_led
    # 20% 3-symbol = split across deep_commit, balanced_triple
    "mono_primary": 0.25,
    "standard_dual": 0.40,
    "double_primary": 0.075,
    "secondary_led": 0.075,
    "deep_commit": 0.10,
    "balanced_triple": 0.10,
    "cross_bridge": 0.00,
}


def generate_pool(
    cards_per_archetype: int = 40,
    generic_count: int = 40,
    pattern_mix: dict[str, float] = None,
    rarity_distribution: dict[Rarity, int] = None,
) -> list[SimCard]:
    """Generate a card pool."""
    if pattern_mix is None:
        pattern_mix = RECONCILED_PATTERN_MIX
    if rarity_distribution is None:
        rarity_distribution = {
            Rarity.COMMON: 180,
            Rarity.UNCOMMON: 100,
            Rarity.RARE: 60,
            Rarity.LEGENDARY: 20,
        }

    cards: list[SimCard] = []
    card_id = 0

    for arch_name, arch_pri, arch_sec in ARCHETYPES:
        # Determine count per pattern
        pattern_cards: list[tuple[str, int]] = []
        remaining = cards_per_archetype
        patterns = list(pattern_mix.items())
        for i, (pat, frac) in enumerate(patterns):
            if i == len(patterns) - 1:
                count = remaining
            else:
                count = round(cards_per_archetype * frac)
                remaining -= count
            if count > 0:
                pattern_cards.append((pat, count))

        for pat, count in pattern_cards:
            for _ in range(count):
                syms = make_symbols(arch_pri, arch_sec, pat)
                cards.append(SimCard(
                    id=card_id,
                    symbols=syms,
                    archetype=arch_name,
                    rarity=Rarity.COMMON,  # placeholder
                    power=3.0,
                ))
                card_id += 1

    # Generic cards
    for _ in range(generic_count):
        cards.append(SimCard(
            id=card_id,
            symbols=[],
            archetype=None,
            rarity=Rarity.COMMON,
            power=3.0,
        ))
        card_id += 1

    # Assign rarities
    total = len(cards)
    # Scale rarity distribution to actual pool size
    rarity_total = sum(rarity_distribution.values())
    scaled = {}
    assigned = 0
    rarities = list(rarity_distribution.items())
    for i, (r, c) in enumerate(rarities):
        if i == len(rarities) - 1:
            scaled[r] = total - assigned
        else:
            scaled[r] = round(c * total / rarity_total)
            assigned += scaled[r]
    assign_rarity_tcg(cards, scaled)

    return cards


# ============================================================
# Lane Locking draft algorithm
# ============================================================

@dataclass
class LaneLockState:
    """Track lock state for a single draft."""
    counters: dict  # Resonance -> int (weighted symbol count)
    locks: list  # list of (Resonance or None) for 4 slots
    threshold_fired: dict  # Resonance -> set of threshold indices fired
    thresholds: tuple[int, ...]  # e.g. (3, 8) or (5, 12)
    primary_weight: int = 2

    @staticmethod
    def new(thresholds: tuple[int, ...], primary_weight: int = 2) -> "LaneLockState":
        return LaneLockState(
            counters={r: 0 for r in Resonance},
            locks=[None, None, None, None],
            threshold_fired={r: set() for r in Resonance},
            thresholds=thresholds,
            primary_weight=primary_weight,
        )

    def locked_count(self) -> int:
        return sum(1 for s in self.locks if s is not None)

    def open_slots(self) -> list[int]:
        return [i for i, s in enumerate(self.locks) if s is None]

    def update(self, card: SimCard):
        """Update counters and fire locks after picking a card."""
        for i, sym in enumerate(card.symbols):
            weight = self.primary_weight if i == 0 else 1
            self.counters[sym] += weight

        # Check thresholds, highest count first
        resonances_by_count = sorted(
            Resonance, key=lambda r: self.counters[r], reverse=True
        )
        for res in resonances_by_count:
            for ti, thresh in enumerate(self.thresholds):
                if ti in self.threshold_fired[res]:
                    continue
                if self.counters[res] >= thresh:
                    self.threshold_fired[res].add(ti)
                    open_s = self.open_slots()
                    if open_s:
                        slot = random.choice(open_s)
                        self.locks[slot] = res


def build_pack(
    pool: list[SimCard],
    state: LaneLockState,
    pack_size: int = 4,
) -> list[SimCard]:
    """Build a pack of cards using Lane Locking."""
    pack = []
    used_ids = set()

    for slot_idx in range(pack_size):
        locked_res = state.locks[slot_idx]
        if locked_res is not None:
            # Locked slot: pick a card whose primary resonance matches
            candidates = [
                c for c in pool
                if c.id not in used_ids
                and c.primary_resonance() == locked_res
            ]
        else:
            # Open slot: any card from pool
            candidates = [c for c in pool if c.id not in used_ids]

        if not candidates:
            # Fallback: any unused card
            candidates = [c for c in pool if c.id not in used_ids]

        if candidates:
            card = random.choice(candidates)
            pack.append(card)
            used_ids.add(card.id)

    return pack


# ============================================================
# Player strategies
# ============================================================

def pick_committed(pack: list[SimCard], target_arch: str,
                   pick_num: int) -> SimCard:
    """Archetype-committed: pick highest fitness, break ties by power."""
    tier_order = {Tier.S: 0, Tier.A: 1, Tier.B: 2, Tier.C: 3, Tier.F: 4}
    return min(pack, key=lambda c: (tier_order[c.fitness_for(target_arch)],
                                    -c.power))


def pick_power_chaser(pack: list[SimCard], **_) -> SimCard:
    """Power-chaser: pick highest raw power."""
    return max(pack, key=lambda c: c.power)


def pick_signal_reader(
    pack: list[SimCard],
    counters: dict[Resonance, int],
    pick_num: int,
) -> SimCard:
    """Signal-reader: evaluate which resonance is most available, draft toward
    the open archetype. In early picks, pick the resonance most represented
    in the pack. After pick 5, commit to the strongest resonance."""
    if pick_num <= 5:
        # Count resonances in pack
        res_count: dict[Resonance, int] = {r: 0 for r in Resonance}
        for card in pack:
            for i, sym in enumerate(card.symbols):
                w = 2 if i == 0 else 1
                res_count[sym] += w
        # Combine with accumulated counters
        combined = {r: counters[r] + res_count[r] for r in Resonance}
        best_res = max(combined, key=lambda r: combined[r])
        # Pick the card that contributes most to best_res
        def card_contribution(c: SimCard) -> int:
            return sum(
                (2 if i == 0 else 1) for i, s in enumerate(c.symbols)
                if s == best_res
            )
        return max(pack, key=lambda c: (card_contribution(c), c.power))
    else:
        # Committed: pick based on top resonance
        best_res = max(counters, key=lambda r: counters[r])
        # Find archetype with this as primary
        target_archs = [
            name for name, pri, sec in ARCHETYPES if pri == best_res
        ]
        if target_archs:
            target = target_archs[0]
            return pick_committed(pack, target, pick_num)
        return max(pack, key=lambda c: c.power)


# ============================================================
# Draft simulation
# ============================================================

@dataclass
class DraftMetrics:
    """Collected metrics for a single draft."""
    early_unique_archs: list[int] = field(default_factory=list)  # per pack, picks 1-5
    early_sa_per_pack: list[int] = field(default_factory=list)
    late_sa_per_pack: list[int] = field(default_factory=list)
    late_cf_per_pack: list[float] = field(default_factory=list)
    convergence_pick: int = 30  # first pick where 2+ S/A
    deck_sa_count: int = 0
    deck_total: int = 0
    picked_archetypes: list[str] = field(default_factory=list)
    picked_card_ids: list[int] = field(default_factory=list)

    # Extended metrics
    first_lock_pick: int = 30
    second_lock_pick: int = 30
    all_locked_pick: int = 30
    genuine_choice_packs: int = 0
    total_post_commit_packs: int = 0
    unwanted_locks: int = 0
    total_locks: int = 0
    pick1_lock: bool = False
    sa_by_phase: dict = field(default_factory=lambda: {
        "p1_5": [], "p6_10": [], "p11_15": [],
        "p16_20": [], "p21_25": [], "p26_30": [],
    })


def phase_key(pick: int) -> str:
    if pick <= 5: return "p1_5"
    if pick <= 10: return "p6_10"
    if pick <= 15: return "p11_15"
    if pick <= 20: return "p16_20"
    if pick <= 25: return "p21_25"
    return "p26_30"


def is_sa(card: SimCard, target_arch: str) -> bool:
    t = card.fitness_for(target_arch)
    return t in (Tier.S, Tier.A)


def is_cf(card: SimCard, target_arch: str) -> bool:
    t = card.fitness_for(target_arch)
    return t in (Tier.C, Tier.F)


def count_unique_archs_with_sa(pack: list[SimCard]) -> int:
    """Count how many archetypes have at least one S/A card in the pack."""
    archs = set()
    for card in pack:
        for arch_name in ARCHETYPE_NAMES:
            if card.fitness_for(arch_name) in (Tier.S, Tier.A):
                archs.add(arch_name)
    return len(archs)


def has_genuine_choice(pack: list[SimCard], target_arch: str) -> bool:
    """Does the pack have 2+ S/A cards with different symbol patterns?"""
    sa_cards = [c for c in pack if is_sa(c, target_arch)]
    if len(sa_cards) < 2:
        return False
    # Check for different pattern types (by tuple of symbols)
    patterns = set()
    for c in sa_cards:
        patterns.add(tuple(c.symbols))
    return len(patterns) >= 2


def run_draft(
    pool: list[SimCard],
    thresholds: tuple[int, ...],
    strategy: str,
    target_arch: str,
    primary_weight: int = 2,
    num_picks: int = 30,
) -> DraftMetrics:
    """Run a single draft and collect metrics."""
    state = LaneLockState.new(thresholds, primary_weight)
    metrics = DraftMetrics()
    found_convergence = False
    prev_lock_count = 0

    for pick_num in range(1, num_picks + 1):
        pack = build_pack(pool, state)

        # Measure pre-pick metrics
        sa_count = sum(1 for c in pack if is_sa(c, target_arch))
        cf_count = sum(1 for c in pack if is_cf(c, target_arch))

        # Phase-based S/A tracking
        metrics.sa_by_phase[phase_key(pick_num)].append(sa_count)

        if pick_num <= 5:
            metrics.early_unique_archs.append(count_unique_archs_with_sa(pack))
            metrics.early_sa_per_pack.append(sa_count)
        else:
            metrics.late_sa_per_pack.append(sa_count)
            metrics.late_cf_per_pack.append(cf_count)
            # Genuine choice tracking
            metrics.total_post_commit_packs += 1
            if has_genuine_choice(pack, target_arch):
                metrics.genuine_choice_packs += 1

        if sa_count >= 2 and not found_convergence:
            metrics.convergence_pick = pick_num
            found_convergence = True

        # Pick a card
        if strategy == "committed":
            picked = pick_committed(pack, target_arch, pick_num)
        elif strategy == "power":
            picked = pick_power_chaser(pack)
        elif strategy == "signal":
            picked = pick_signal_reader(pack, state.counters, pick_num)
        else:
            picked = random.choice(pack)

        # Track deck composition
        if is_sa(picked, target_arch):
            metrics.deck_sa_count += 1
        metrics.deck_total += 1
        metrics.picked_card_ids.append(picked.id)
        if picked.archetype:
            metrics.picked_archetypes.append(picked.archetype)

        # Update lock state
        old_locks = list(state.locks)
        state.update(picked)
        new_locks = list(state.locks)

        # Track lock timing
        new_lock_count = state.locked_count()
        if new_lock_count > prev_lock_count:
            for i in range(prev_lock_count + 1, new_lock_count + 1):
                if i == 1:
                    metrics.first_lock_pick = pick_num
                    if pick_num == 1:
                        metrics.pick1_lock = True
                if i == 2:
                    metrics.second_lock_pick = pick_num
                if i == 4 and metrics.all_locked_pick == 30:
                    metrics.all_locked_pick = pick_num

            # Track unwanted locks
            target_pri = ARCHETYPES[ARCHETYPE_INDEX[target_arch]][1]
            target_sec = ARCHETYPES[ARCHETYPE_INDEX[target_arch]][2]
            for slot_i in range(4):
                if old_locks[slot_i] is None and new_locks[slot_i] is not None:
                    metrics.total_locks += 1
                    locked_res = new_locks[slot_i]
                    if locked_res != target_pri and locked_res != target_sec:
                        metrics.unwanted_locks += 1

        prev_lock_count = new_lock_count

    return metrics


# ============================================================
# Batch simulation
# ============================================================

@dataclass
class AggregateResults:
    early_diversity: float = 0.0
    early_sa: float = 0.0
    late_sa: float = 0.0
    late_cf: float = 0.0
    convergence_pick: float = 0.0
    deck_concentration: float = 0.0
    card_overlap: float = 0.0
    archetype_freq_min: float = 0.0
    archetype_freq_max: float = 0.0
    # Extended
    first_lock: float = 0.0
    second_lock: float = 0.0
    all_locked: float = 0.0
    pick1_lock_pct: float = 0.0
    genuine_choice_pct: float = 0.0
    unwanted_lock_pct: float = 0.0
    sa_by_phase: dict = field(default_factory=dict)

    def pass_fail(self) -> dict[str, tuple[str, bool]]:
        return {
            "Early diversity (>=3)": (f"{self.early_diversity:.2f}",
                                      self.early_diversity >= 3),
            "Early S/A (<=2)": (f"{self.early_sa:.2f}",
                                self.early_sa <= 2),
            "Late S/A (>=2)": (f"{self.late_sa:.2f}",
                               self.late_sa >= 2),
            "Late C/F (>=0.5)": (f"{self.late_cf:.2f}",
                                 self.late_cf >= 0.5),
            "Conv pick (5-8)": (f"{self.convergence_pick:.1f}",
                                5 <= self.convergence_pick <= 8),
            "Deck conc (60-80%)": (f"{self.deck_concentration:.1f}%",
                                   60 <= self.deck_concentration <= 80),
            "Card overlap (<40%)": (f"{self.card_overlap:.1f}%",
                                    self.card_overlap < 40),
            "Arch freq (5-20%)": (
                f"{self.archetype_freq_min:.1f}-{self.archetype_freq_max:.1f}%",
                self.archetype_freq_min >= 5 and self.archetype_freq_max <= 20,
            ),
        }


def run_batch(
    pool: list[SimCard],
    thresholds: tuple[int, ...],
    primary_weight: int,
    n_drafts: int = 1000,
    strategy: str = "committed",
) -> AggregateResults:
    """Run n_drafts and aggregate metrics."""
    all_metrics: list[DraftMetrics] = []
    arch_counts: dict[str, int] = {a: 0 for a in ARCHETYPE_NAMES}
    card_id_sets: list[set[int]] = []

    for _ in range(n_drafts):
        # Random target archetype
        target_idx = random.randint(0, 7)
        target_arch = ARCHETYPE_NAMES[target_idx]
        arch_counts[target_arch] += 1

        m = run_draft(pool, thresholds, strategy, target_arch, primary_weight)
        all_metrics.append(m)
        card_id_sets.append(set(m.picked_card_ids))

    res = AggregateResults()

    # Early diversity
    all_early_div = []
    for m in all_metrics:
        all_early_div.extend(m.early_unique_archs)
    res.early_diversity = statistics.mean(all_early_div) if all_early_div else 0

    # Early S/A
    all_early_sa = []
    for m in all_metrics:
        all_early_sa.extend(m.early_sa_per_pack)
    res.early_sa = statistics.mean(all_early_sa) if all_early_sa else 0

    # Late S/A
    all_late_sa = []
    for m in all_metrics:
        all_late_sa.extend(m.late_sa_per_pack)
    res.late_sa = statistics.mean(all_late_sa) if all_late_sa else 0

    # Late C/F
    all_late_cf = []
    for m in all_metrics:
        all_late_cf.extend(m.late_cf_per_pack)
    res.late_cf = statistics.mean(all_late_cf) if all_late_cf else 0

    # Convergence pick
    res.convergence_pick = statistics.mean(
        [m.convergence_pick for m in all_metrics]
    )

    # Deck concentration
    concentrations = [
        m.deck_sa_count / m.deck_total * 100
        for m in all_metrics if m.deck_total > 0
    ]
    res.deck_concentration = statistics.mean(concentrations) if concentrations else 0

    # Card overlap (average pairwise Jaccard among same-archetype drafts)
    overlaps = []
    sample_size = min(200, len(card_id_sets))
    indices = random.sample(range(len(card_id_sets)), sample_size)
    for i in range(0, len(indices) - 1, 2):
        s1 = card_id_sets[indices[i]]
        s2 = card_id_sets[indices[i + 1]]
        if s1 or s2:
            overlap = len(s1 & s2) / len(s1 | s2) * 100
            overlaps.append(overlap)
    res.card_overlap = statistics.mean(overlaps) if overlaps else 0

    # Archetype frequency
    total_drafts = sum(arch_counts.values())
    freqs = {a: arch_counts[a] / total_drafts * 100 for a in ARCHETYPE_NAMES}
    # But we want to measure how often each archetype is CHOSEN, not assigned.
    # Since we assign randomly, this measures the algorithm's balance.
    # For signal-reader, we should track what they end up drafting.
    res.archetype_freq_min = min(freqs.values())
    res.archetype_freq_max = max(freqs.values())

    # Extended metrics
    res.first_lock = statistics.mean([m.first_lock_pick for m in all_metrics])
    res.second_lock = statistics.mean([m.second_lock_pick for m in all_metrics])
    res.all_locked = statistics.mean([m.all_locked_pick for m in all_metrics])
    res.pick1_lock_pct = (
        sum(1 for m in all_metrics if m.pick1_lock) / len(all_metrics) * 100
    )
    genuine_total = sum(m.genuine_choice_packs for m in all_metrics)
    post_commit_total = sum(m.total_post_commit_packs for m in all_metrics)
    res.genuine_choice_pct = (
        genuine_total / post_commit_total * 100 if post_commit_total > 0 else 0
    )
    total_locks = sum(m.total_locks for m in all_metrics)
    unwanted_locks = sum(m.unwanted_locks for m in all_metrics)
    res.unwanted_lock_pct = (
        unwanted_locks / total_locks * 100 if total_locks > 0 else 0
    )

    # S/A by phase
    for phase in ["p1_5", "p6_10", "p11_15", "p16_20", "p21_25", "p26_30"]:
        vals = []
        for m in all_metrics:
            vals.extend(m.sa_by_phase[phase])
        res.sa_by_phase[phase] = statistics.mean(vals) if vals else 0

    return res


# ============================================================
# Main: run all configurations and print results
# ============================================================

def format_results(name: str, res: AggregateResults) -> str:
    lines = [f"\n{'='*70}", f"  {name}", f"{'='*70}"]

    lines.append("\n--- 8 Archetype-Level Targets ---")
    pf = res.pass_fail()
    passes = 0
    for metric, (val, passed) in pf.items():
        status = "PASS" if passed else "FAIL"
        if passed:
            passes += 1
        lines.append(f"  {metric:30s}  {val:>15s}  {status}")
    lines.append(f"  {'Targets passed':30s}  {passes}/8")

    lines.append("\n--- Lock Timing ---")
    lines.append(f"  First lock pick:    {res.first_lock:.1f}")
    lines.append(f"  Second lock pick:   {res.second_lock:.1f}")
    lines.append(f"  All-4 locked pick:  {res.all_locked:.1f}")
    lines.append(f"  Pick-1 lock rate:   {res.pick1_lock_pct:.1f}%")

    lines.append("\n--- Decision Quality ---")
    lines.append(f"  Genuine choice rate: {res.genuine_choice_pct:.1f}%")
    lines.append(f"  Unwanted lock rate:  {res.unwanted_lock_pct:.1f}%")

    lines.append("\n--- Convergence Curve (S/A per pack by phase) ---")
    for phase in ["p1_5", "p6_10", "p11_15", "p16_20", "p21_25", "p26_30"]:
        val = res.sa_by_phase.get(phase, 0)
        lines.append(f"  {phase:8s}: {val:.2f}")
    sa_trend = res.sa_by_phase.get("p26_30", 0) - res.sa_by_phase.get("p6_10", 0)
    lines.append(f"  SA Trend (p26_30 - p6_10): {sa_trend:+.2f}")

    return "\n".join(lines)


def main():
    random.seed(42)
    N_DRAFTS = 1000

    print("=" * 70)
    print("  POOL SYNTHESIS SIMULATION")
    print("  Reconciled Lane Locking Pool Design")
    print("=" * 70)

    # ---- Generate pools ----
    reconciled_pool = generate_pool(
        cards_per_archetype=40,
        generic_count=40,
        pattern_mix=RECONCILED_PATTERN_MIX,
    )
    print(f"\nReconciled pool: {len(reconciled_pool)} cards")
    # Count symbol distribution
    sym_counts = {0: 0, 1: 0, 2: 0, 3: 0}
    for c in reconciled_pool:
        sym_counts[len(c.symbols)] += 1
    print(f"  Symbol distribution: {sym_counts}")

    default_pool = generate_pool(
        cards_per_archetype=40,
        generic_count=40,
        pattern_mix=DEFAULT_PATTERN_MIX,
    )
    print(f"Default pool: {len(default_pool)} cards")
    sym_counts_d = {0: 0, 1: 0, 2: 0, 3: 0}
    for c in default_pool:
        sym_counts_d[len(c.symbols)] += 1
    print(f"  Symbol distribution: {sym_counts_d}")

    # ---- Configurations to test ----
    configs = [
        # (name, pool, thresholds, primary_weight)
        ("DEFAULT: (3,8) W2 + default pool", default_pool, (3, 8), 2),
        ("RECONCILED: (3,8) W2", reconciled_pool, (3, 8), 2),
        ("RECONCILED: (4,10) W2", reconciled_pool, (4, 10), 2),
        ("RECONCILED: (5,12) W2", reconciled_pool, (5, 12), 2),
    ]

    strategies = ["committed", "power", "signal"]

    all_results: dict[str, dict[str, AggregateResults]] = {}

    for config_name, pool, thresholds, pw in configs:
        all_results[config_name] = {}
        for strat in strategies:
            print(f"\nRunning: {config_name} / {strat} ({N_DRAFTS} drafts)...")
            res = run_batch(pool, thresholds, pw, N_DRAFTS, strat)
            all_results[config_name][strat] = res

    # ---- Print results ----
    print("\n\n")
    print("#" * 70)
    print("  RESULTS")
    print("#" * 70)

    for config_name in all_results:
        for strat in strategies:
            res = all_results[config_name][strat]
            print(format_results(f"{config_name} [{strat}]", res))

    # ---- Comparison table ----
    print("\n\n")
    print("#" * 70)
    print("  COMPARISON: DEFAULT vs RECONCILED (committed strategy)")
    print("#" * 70)

    default_key = "DEFAULT: (3,8) W2 + default pool"
    compare_configs = [
        ("DEFAULT (3,8)", default_key),
        ("RECON (3,8)", "RECONCILED: (3,8) W2"),
        ("RECON (4,10)", "RECONCILED: (4,10) W2"),
        ("RECON (5,12)", "RECONCILED: (5,12) W2"),
    ]

    header = f"{'Metric':30s}"
    for short_name, _ in compare_configs:
        header += f"  {short_name:>15s}"
    print(f"\n{header}")
    print("-" * len(header))

    metrics_list = [
        ("Late S/A", lambda r: f"{r.late_sa:.2f}"),
        ("Early diversity", lambda r: f"{r.early_diversity:.2f}"),
        ("Early S/A", lambda r: f"{r.early_sa:.2f}"),
        ("Late C/F", lambda r: f"{r.late_cf:.2f}"),
        ("Conv pick", lambda r: f"{r.convergence_pick:.1f}"),
        ("Deck conc %", lambda r: f"{r.deck_concentration:.1f}%"),
        ("Card overlap %", lambda r: f"{r.card_overlap:.1f}%"),
        ("Arch freq range", lambda r: f"{r.archetype_freq_min:.1f}-{r.archetype_freq_max:.1f}%"),
        ("1st lock pick", lambda r: f"{r.first_lock:.1f}"),
        ("2nd lock pick", lambda r: f"{r.second_lock:.1f}"),
        ("All-4 locked", lambda r: f"{r.all_locked:.1f}"),
        ("Pick-1 lock %", lambda r: f"{r.pick1_lock_pct:.1f}%"),
        ("Genuine choice %", lambda r: f"{r.genuine_choice_pct:.1f}%"),
        ("Unwanted lock %", lambda r: f"{r.unwanted_lock_pct:.1f}%"),
        ("SA Trend", lambda r: f"{r.sa_by_phase.get('p26_30', 0) - r.sa_by_phase.get('p6_10', 0):+.2f}"),
    ]

    for metric_name, fmt_fn in metrics_list:
        row = f"{metric_name:30s}"
        for _, config_key in compare_configs:
            res = all_results[config_key]["committed"]
            row += f"  {fmt_fn(res):>15s}"
        print(row)

    # ---- Strategy comparison for recommended config ----
    print("\n\n")
    print("#" * 70)
    print("  STRATEGY COMPARISON: RECONCILED (4,10) W2")
    print("#" * 70)

    rec_key = "RECONCILED: (4,10) W2"
    header2 = f"{'Metric':30s}"
    for strat in strategies:
        header2 += f"  {strat:>15s}"
    print(f"\n{header2}")
    print("-" * len(header2))

    for metric_name, fmt_fn in metrics_list:
        row = f"{metric_name:30s}"
        for strat in strategies:
            res = all_results[rec_key][strat]
            row += f"  {fmt_fn(res):>15s}"
        print(row)

    print("\n\nDone.")


if __name__ == "__main__":
    main()
