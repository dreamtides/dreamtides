#!/usr/bin/env python3
"""
Simulation for the Lane Locking draft algorithm (Agent 3, Threshold/Progression).

Algorithm (one sentence):
"Your pack has 4 slots; when your symbol count in a resonance first reaches 3,
one open slot locks to that resonance; when it first reaches 8, a second slot locks."

Symbol counting: primary (leftmost) = 2, secondary = 1, tertiary = 1.
Max 4 total locked slots across all resonances.

CORRECTED VERSION: All evaluation metrics are measured at the ARCHETYPE level
(S/A-tier fitness), not the resonance level. A resonance (e.g. Tide) is shared
by multiple archetypes; an archetype (e.g. Warriors = Tide/Zephyr) is specific.
"""

import random
import statistics
from collections import Counter
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional

# ─── Constants ───────────────────────────────────────────────────────────────

NUM_CARDS = 360
NUM_GENERIC = 36
NUM_ARCHETYPE_CARDS = NUM_CARDS - NUM_GENERIC  # 324
NUM_ARCHETYPES = 8
PACK_SIZE = 4
NUM_PICKS = 30
NUM_SIMULATIONS = 1000

RESONANCES = ["Ember", "Stone", "Tide", "Zephyr"]

# Archetypes on the circle: (name, primary_resonance, secondary_resonance)
ARCHETYPES = [
    ("Flash",         "Zephyr", "Ember"),   # 0  (position 1)
    ("Blink",         "Ember",  "Zephyr"),  # 1  (position 2)
    ("Storm",         "Ember",  "Stone"),   # 2  (position 3)
    ("Self-Discard",  "Stone",  "Ember"),   # 3  (position 4)
    ("Self-Mill",     "Stone",  "Tide"),    # 4  (position 5)
    ("Sacrifice",     "Tide",   "Stone"),   # 5  (position 6)
    ("Warriors",      "Tide",   "Zephyr"),  # 6  (position 7)
    ("Ramp",          "Zephyr", "Tide"),    # 7  (position 8)
]

ARCHETYPE_NAMES = [a[0] for a in ARCHETYPES]

# ─── Data Classes ────────────────────────────────────────────────────────────

class Rarity(Enum):
    COMMON = "common"
    UNCOMMON = "uncommon"
    RARE = "rare"
    LEGENDARY = "legendary"


@dataclass
class SimCard:
    id: int
    symbols: list  # list of resonance strings, ordered
    archetype: Optional[str]  # None for generic
    archetype_fitness: dict  # archetype_name -> tier (S/A/B/C/F)
    rarity: Rarity
    power: float

    @property
    def primary_resonance(self) -> Optional[str]:
        return self.symbols[0] if self.symbols else None


# ─── Adjacency & Fitness ────────────────────────────────────────────────────

def get_archetype_index(name: str) -> int:
    for i, (n, _, _) in enumerate(ARCHETYPES):
        if n == name:
            return i
    raise ValueError(f"Unknown archetype: {name}")


def circle_distance(i: int, j: int) -> int:
    """Distance on the circular arrangement of 8 archetypes."""
    d = abs(i - j)
    return min(d, NUM_ARCHETYPES - d)


def are_adjacent(i: int, j: int) -> bool:
    return circle_distance(i, j) == 1


def compute_fitness(card_archetype: Optional[str]) -> dict:
    """
    Compute fitness tiers for a card given its home archetype.

    Corrected fitness model:
    - S-tier: home archetype
    - A-tier: adjacent archetype that shares the home's PRIMARY resonance
      (the adjacent archetype has home_primary as either its primary or secondary)
    - B-tier: non-S/A archetype that shares the home's SECONDARY resonance
      (the archetype has home_secondary as either its primary or secondary)
    - C-tier: remaining non-adjacent archetypes sharing any resonance (catch-all)
    - F-tier: no shared resonance at all
    - Generic cards: B-tier in all archetypes
    """
    fitness = {}
    if card_archetype is None:
        for name, _, _ in ARCHETYPES:
            fitness[name] = "B"
        return fitness

    home_idx = get_archetype_index(card_archetype)
    home_primary = ARCHETYPES[home_idx][1]
    home_secondary = ARCHETYPES[home_idx][2]

    for i, (name, arch_pri, arch_sec) in enumerate(ARCHETYPES):
        if name == card_archetype:
            fitness[name] = "S"
        elif are_adjacent(home_idx, i) and home_primary in (arch_pri, arch_sec):
            # Adjacent archetype sharing home's primary resonance -> A-tier
            fitness[name] = "A"
        elif home_secondary in (arch_pri, arch_sec) and name not in fitness:
            # Shares home's secondary resonance (not already S or A) -> B-tier
            fitness[name] = "B"
        elif home_primary in (arch_pri, arch_sec):
            # Non-adjacent but shares home primary -> C-tier
            fitness[name] = "C"
        elif home_secondary in (arch_pri, arch_sec):
            # Already handled above as B, but just in case
            fitness[name] = "B"
        else:
            # No shared resonance
            fitness[name] = "F"

    return fitness


# ─── Card Pool Construction ─────────────────────────────────────────────────

def generate_card_pool(symbol_dist: dict = None) -> list:
    """
    Generate 360 cards.
    symbol_dist: dict with keys 1, 2, 3 mapping to fractions of non-generic cards.
    Default: 25% 1-symbol, 55% 2-symbol, 20% 3-symbol.
    """
    if symbol_dist is None:
        symbol_dist = {1: 0.25, 2: 0.55, 3: 0.20}

    cards = []
    card_id = 0

    # Distribute archetype cards: ~40 per archetype, exact split
    cards_per = [NUM_ARCHETYPE_CARDS // NUM_ARCHETYPES] * NUM_ARCHETYPES
    leftover = NUM_ARCHETYPE_CARDS - sum(cards_per)
    for i in range(leftover):
        cards_per[i] += 1

    for arch_idx, (arch_name, primary, secondary) in enumerate(ARCHETYPES):
        n_cards = cards_per[arch_idx]

        # Distribute symbol counts
        n1 = round(n_cards * symbol_dist[1])
        n3 = round(n_cards * symbol_dist[3])
        n2 = n_cards - n1 - n3

        for i in range(n_cards):
            if i < n1:
                num_symbols = 1
            elif i < n1 + n2:
                num_symbols = 2
            else:
                num_symbols = 3

            # Generate symbols based on archetype's resonances
            if num_symbols == 1:
                # Mostly primary, sometimes secondary
                symbols = [primary] if random.random() < 0.80 else [secondary]
            elif num_symbols == 2:
                r = random.random()
                if r < 0.55:
                    symbols = [primary, secondary]
                elif r < 0.80:
                    symbols = [primary, primary]
                elif r < 0.95:
                    symbols = [secondary, primary]
                else:
                    symbols = [secondary, secondary]
            else:  # 3 symbols
                r = random.random()
                if r < 0.45:
                    symbols = [primary, primary, secondary]
                elif r < 0.75:
                    symbols = [primary, secondary, primary]
                elif r < 0.90:
                    symbols = [primary, secondary, secondary]
                else:
                    symbols = [secondary, primary, secondary]

            # Assign rarity
            rr = random.random()
            if rr < 0.50:
                rarity = Rarity.COMMON
            elif rr < 0.80:
                rarity = Rarity.UNCOMMON
            elif rr < 0.95:
                rarity = Rarity.RARE
            else:
                rarity = Rarity.LEGENDARY

            power = random.uniform(3.0, 9.0)

            card = SimCard(
                id=card_id,
                symbols=symbols,
                archetype=arch_name,
                archetype_fitness=compute_fitness(arch_name),
                rarity=rarity,
                power=power,
            )
            cards.append(card)
            card_id += 1

    # Generic cards (36 total)
    for _ in range(NUM_GENERIC):
        rr = random.random()
        if rr < 0.40:
            rarity = Rarity.COMMON
        elif rr < 0.70:
            rarity = Rarity.UNCOMMON
        elif rr < 0.90:
            rarity = Rarity.RARE
        else:
            rarity = Rarity.LEGENDARY

        power = random.uniform(4.0, 9.5)  # generics slightly higher average

        card = SimCard(
            id=card_id,
            symbols=[],
            archetype=None,
            archetype_fitness=compute_fitness(None),
            rarity=rarity,
            power=power,
        )
        cards.append(card)
        card_id += 1

    random.shuffle(cards)
    return cards


# ─── Lane Locking Algorithm ─────────────────────────────────────────────────

@dataclass
class LaneLockState:
    """Tracks the lane-locking draft state for one player."""
    symbol_counts: dict = field(default_factory=lambda: {r: 0 for r in RESONANCES})
    locked_slots: list = field(default_factory=list)  # list of resonance strings
    thresholds_triggered: dict = field(default_factory=lambda: {r: set() for r in RESONANCES})
    max_locks: int = 4  # total lock cap (cannot exceed PACK_SIZE)

    def add_card(self, card: SimCard, threshold_low: int = 3, threshold_high: int = 8):
        """Update state after drafting a card. Returns list of new locks triggered."""
        new_locks = []
        if not card.symbols:
            return new_locks

        # Count symbols: primary (first) = 2, secondary/tertiary = 1
        for i, sym in enumerate(card.symbols):
            weight = 2 if i == 0 else 1
            self.symbol_counts[sym] += weight

        # Effective max: cannot lock more than PACK_SIZE slots
        effective_max = min(self.max_locks, PACK_SIZE)

        # Check thresholds for all resonances
        for res in RESONANCES:
            count = self.symbol_counts[res]
            if len(self.locked_slots) >= effective_max:
                break
            # Check low threshold
            if count >= threshold_low and "low" not in self.thresholds_triggered[res]:
                self.thresholds_triggered[res].add("low")
                self.locked_slots.append(res)
                new_locks.append((res, "low"))
                if len(self.locked_slots) >= effective_max:
                    break
            # Check high threshold
            if count >= threshold_high and "high" not in self.thresholds_triggered[res]:
                self.thresholds_triggered[res].add("high")
                self.locked_slots.append(res)
                new_locks.append((res, "high"))
                if len(self.locked_slots) >= effective_max:
                    break

        return new_locks


def build_pack(pool: list, state: LaneLockState) -> list:
    """
    Build a pack of PACK_SIZE cards using the lane locking algorithm.
    Locked slots are filled with a random card whose primary resonance matches.
    Open slots are filled with random cards from the full pool.
    """
    pack = []
    used_ids = set()

    # Fill locked slots (up to PACK_SIZE)
    slots_to_fill = state.locked_slots[:PACK_SIZE]
    for res in slots_to_fill:
        matching = [c for c in pool if c.primary_resonance == res and c.id not in used_ids]
        if matching:
            chosen = random.choice(matching)
            pack.append(chosen)
            used_ids.add(chosen.id)
        else:
            # Fallback: any card with this resonance symbol
            matching = [c for c in pool if res in c.symbols and c.id not in used_ids]
            if matching:
                chosen = random.choice(matching)
                pack.append(chosen)
                used_ids.add(chosen.id)
            else:
                # Last resort: random card
                available = [c for c in pool if c.id not in used_ids]
                if available:
                    chosen = random.choice(available)
                    pack.append(chosen)
                    used_ids.add(chosen.id)

    # Fill open slots with random cards from full pool
    num_open = PACK_SIZE - len(pack)
    if num_open > 0:
        available = [c for c in pool if c.id not in used_ids]
        if len(available) >= num_open:
            chosen = random.sample(available, num_open)
            pack.extend(chosen)
        else:
            pack.extend(available)

    return pack


# ─── Player Strategies ───────────────────────────────────────────────────────

TIER_VALUES = {"S": 5, "A": 4, "B": 3, "C": 1, "F": 0}


def evaluate_archetype_strength(drafted: list) -> dict:
    """Evaluate how strong each archetype is for the player based on drafted cards."""
    scores = {name: 0 for name in ARCHETYPE_NAMES}
    for card in drafted:
        for arch, tier in card.archetype_fitness.items():
            scores[arch] += TIER_VALUES[tier]
    return scores


def pick_archetype_committed(pack: list, drafted: list, committed_arch: Optional[str]) -> tuple:
    """
    Archetype-committed strategy: pick best card for strongest archetype.
    Commits around pick 5-6.
    """
    if committed_arch is None and len(drafted) >= 5:
        scores = evaluate_archetype_strength(drafted)
        committed_arch = max(scores, key=scores.get)

    if committed_arch is None:
        # Not yet committed: pick highest fitness in current best archetype
        if drafted:
            scores = evaluate_archetype_strength(drafted)
            best_arch = max(scores, key=scores.get)
        else:
            best_arch = random.choice(ARCHETYPE_NAMES)
        best_card = max(pack, key=lambda c: (
            TIER_VALUES.get(c.archetype_fitness.get(best_arch, "F"), 0), c.power))
    else:
        best_card = max(pack, key=lambda c: (
            TIER_VALUES.get(c.archetype_fitness.get(committed_arch, "F"), 0), c.power))

    return best_card, committed_arch


def pick_power_chaser(pack: list, drafted: list) -> SimCard:
    """Power-chaser: pick highest raw power card."""
    return max(pack, key=lambda c: c.power)


def pick_signal_reader(pack: list, drafted: list, pick_num: int,
                       committed_arch: Optional[str]) -> tuple:
    """
    Signal-reader: in early picks, identify the most-represented resonance
    in offered packs and draft toward it. After pick 5, commit to best archetype.
    """
    if pick_num < 5:
        # Count resonance symbols in this pack to detect signal
        pack_res_count = Counter()
        for card in pack:
            for i, sym in enumerate(card.symbols):
                pack_res_count[sym] += 2 if i == 0 else 1

        if pack_res_count:
            best_res = max(pack_res_count, key=pack_res_count.get)
        else:
            best_res = random.choice(RESONANCES)

        # Pick card with most symbols matching the detected resonance
        def card_res_score(c):
            score = 0
            for i, sym in enumerate(c.symbols):
                if sym == best_res:
                    score += 2 if i == 0 else 1
            return score
        best_card = max(pack, key=lambda c: (card_res_score(c), c.power))
        return best_card, None
    else:
        # After early picks, commit to best archetype based on what we've drafted
        if committed_arch is None:
            scores = evaluate_archetype_strength(drafted)
            committed_arch = max(scores, key=scores.get)
        best_card = max(pack, key=lambda c: (
            TIER_VALUES.get(c.archetype_fitness.get(committed_arch, "F"), 0), c.power))
        return best_card, committed_arch


# ─── ARCHETYPE-LEVEL Evaluation Helpers ─────────────────────────────────────

def card_fits_archetype(card: SimCard, arch_name: str) -> bool:
    """Does this card fit the given archetype at S or A tier?"""
    return card.archetype_fitness.get(arch_name, "F") in ("S", "A")


def card_is_off_archetype(card: SimCard, arch_name: str) -> bool:
    """Is this card C or F tier for the given archetype? (off-archetype)"""
    return card.archetype_fitness.get(arch_name, "F") in ("C", "F")


def get_unique_archetypes_with_sa(pack: list) -> set:
    """
    Get the set of archetypes that have at least one S or A tier card in the pack.
    This is the ARCHETYPE-LEVEL openness metric (not resonance-level).
    """
    archetypes_present = set()
    for card in pack:
        for arch_name in ARCHETYPE_NAMES:
            if card.archetype_fitness.get(arch_name, "F") in ("S", "A"):
                archetypes_present.add(arch_name)
    return archetypes_present


# ─── Simulation Core ─────────────────────────────────────────────────────────

@dataclass
class DraftMetrics:
    # Picks 1-5: unique archetypes with S/A cards per pack
    early_unique_archetypes_per_pack: list = field(default_factory=list)
    # Picks 1-5: S/A-tier cards for emerging archetype per pack
    early_archetype_fit_per_pack: list = field(default_factory=list)
    # Picks 6+: S/A-tier cards for committed archetype per pack
    late_archetype_fit_per_pack: list = field(default_factory=list)
    # Picks 6+: C/F-tier cards per pack (off-archetype)
    late_off_archetype_per_pack: list = field(default_factory=list)
    # First pick where 2+ S/A cards appear for committed archetype
    convergence_pick: Optional[int] = None
    # Fraction of drafted cards that are S/A tier in best archetype
    deck_concentration: float = 0.0
    # Card IDs drafted (for overlap measurement)
    drafted_card_ids: list = field(default_factory=list)


def run_single_draft(pool: list, strategy: str, threshold_low: int = 3,
                     threshold_high: int = 8, max_locks: int = 4,
                     trace: bool = False) -> "DraftMetrics | tuple":
    """Run a single 30-pick draft with the given strategy."""
    state = LaneLockState(max_locks=max_locks)
    drafted = []
    metrics = DraftMetrics()
    committed_arch = None
    convergence_found = False
    trace_lines = []

    for pick_num in range(NUM_PICKS):
        pack = build_pack(pool, state)

        # Determine player's current best archetype for metric measurement
        if drafted:
            scores = evaluate_archetype_strength(drafted)
            current_best_arch = max(scores, key=scores.get)
        else:
            current_best_arch = None

        # Early metrics (picks 1-5, i.e. pick_num 0-4)
        if pick_num < 5:
            # ARCHETYPE-LEVEL: count unique archetypes with at least one S/A card
            unique_archs = get_unique_archetypes_with_sa(pack)
            metrics.early_unique_archetypes_per_pack.append(len(unique_archs))

            if current_best_arch:
                fit_count = sum(1 for c in pack if card_fits_archetype(c, current_best_arch))
                metrics.early_archetype_fit_per_pack.append(fit_count)

        # Late metrics (picks 6+, i.e. pick_num >= 5)
        if pick_num >= 5 and current_best_arch:
            fit_count = sum(1 for c in pack if card_fits_archetype(c, current_best_arch))
            metrics.late_archetype_fit_per_pack.append(fit_count)

            # CORRECTED: C/F-tier cards (off-archetype), not B-tier
            off_count = sum(1 for c in pack if card_is_off_archetype(c, current_best_arch))
            metrics.late_off_archetype_per_pack.append(off_count)

            # Track convergence: first time we see 2+ S/A cards for archetype
            if not convergence_found and fit_count >= 2:
                metrics.convergence_pick = pick_num + 1  # 1-indexed
                convergence_found = True

        # Pick a card based on strategy
        if strategy == "archetype_committed":
            chosen, committed_arch = pick_archetype_committed(pack, drafted, committed_arch)
        elif strategy == "power_chaser":
            chosen = pick_power_chaser(pack, drafted)
        elif strategy == "signal_reader":
            chosen, committed_arch = pick_signal_reader(pack, drafted, pick_num, committed_arch)
        else:
            raise ValueError(f"Unknown strategy: {strategy}")

        # Record trace before updating state
        if trace:
            lock_info = list(state.locked_slots)
            target_arch = committed_arch or current_best_arch or ""
            pack_desc = []
            for c in pack:
                sym_str = "/".join(c.symbols) if c.symbols else "Generic"
                arch_str = c.archetype or "Generic"
                tier = c.archetype_fitness.get(target_arch, "?")
                # Show which archetypes this card is S/A for
                sa_archs = [a for a in ARCHETYPE_NAMES
                            if c.archetype_fitness.get(a, "F") in ("S", "A")]
                sa_str = ",".join(sa_archs) if sa_archs else "none"
                chosen_marker = " <<PICKED>>" if c is chosen else ""
                pack_desc.append(
                    f"  [{sym_str}] {arch_str} (pwr={c.power:.1f}, "
                    f"tier={tier}, S/A-for=[{sa_str}]){chosen_marker}")
            new_locks = state.add_card(chosen, threshold_low, threshold_high)
            lock_str = f" | NEW LOCKS: {new_locks}" if new_locks else ""
            sa_count = sum(1 for c in pack if card_fits_archetype(c, target_arch)) if target_arch else 0
            cf_count = sum(1 for c in pack if card_is_off_archetype(c, target_arch)) if target_arch else 0
            unique_archs = get_unique_archetypes_with_sa(pack)
            trace_lines.append(
                f"Pick {pick_num+1}: Locked={lock_info} | "
                f"Counts={dict(state.symbol_counts)}{lock_str}\n"
                f"  Target={target_arch} | S/A={sa_count}/4 | C/F={cf_count}/4 | "
                f"UniqueArchs={len(unique_archs)}\n"
                + "\n".join(pack_desc)
            )
        else:
            state.add_card(chosen, threshold_low, threshold_high)

        drafted.append(chosen)
        metrics.drafted_card_ids.append(chosen.id)

    # Deck concentration: fraction of drafted cards that are S/A tier in best archetype
    if drafted:
        scores = evaluate_archetype_strength(drafted)
        best_arch = max(scores, key=scores.get)
        sa_count = sum(1 for c in drafted if card_fits_archetype(c, best_arch))
        metrics.deck_concentration = sa_count / len(drafted)

    if trace:
        return metrics, trace_lines
    return metrics


def run_simulations(n: int = NUM_SIMULATIONS, threshold_low: int = 3,
                    threshold_high: int = 8, max_locks: int = 4,
                    symbol_dist: dict = None) -> tuple:
    """Run n drafts for each strategy and collect aggregate metrics."""
    pool = generate_card_pool(symbol_dist)

    results = {}
    for strategy in ["archetype_committed", "power_chaser", "signal_reader"]:
        all_metrics = []
        for _ in range(n):
            m = run_single_draft(pool, strategy, threshold_low, threshold_high, max_locks)
            all_metrics.append(m)
        results[strategy] = all_metrics

    return results, pool


def compute_aggregate(results: dict) -> dict:
    """Compute aggregate statistics from simulation results."""
    agg = {}
    for strategy, all_metrics in results.items():
        s = {}

        # Early unique ARCHETYPES with S/A cards per pack (picks 1-5)
        all_early_archs = [v for m in all_metrics for v in m.early_unique_archetypes_per_pack]
        s["early_unique_archs"] = statistics.mean(all_early_archs) if all_early_archs else 0

        # Early archetype fit per pack (picks 1-5) — S/A cards for emerging archetype
        all_early_fit = [v for m in all_metrics for v in m.early_archetype_fit_per_pack]
        s["early_arch_fit"] = statistics.mean(all_early_fit) if all_early_fit else 0

        # Late archetype fit per pack (picks 6+) — S/A cards for committed archetype
        all_late_fit = [v for m in all_metrics for v in m.late_archetype_fit_per_pack]
        s["late_arch_fit"] = statistics.mean(all_late_fit) if all_late_fit else 0

        # Late off-archetype per pack (picks 6+) — C/F-tier cards
        all_late_off = [v for m in all_metrics for v in m.late_off_archetype_per_pack]
        s["late_off_arch"] = statistics.mean(all_late_off) if all_late_off else 0

        # Convergence pick
        conv_picks = [m.convergence_pick for m in all_metrics if m.convergence_pick is not None]
        s["convergence_pick"] = statistics.mean(conv_picks) if conv_picks else float("inf")
        s["convergence_pct"] = len(conv_picks) / len(all_metrics) * 100

        # Deck concentration
        s["deck_concentration"] = statistics.mean([m.deck_concentration for m in all_metrics])

        # Card overlap (run-to-run variety): average pairwise Jaccard overlap
        if len(all_metrics) >= 2:
            overlaps = []
            sample_size = min(200, len(all_metrics))
            sampled = random.sample(all_metrics, sample_size)
            for i in range(0, sample_size - 1, 2):
                s1 = set(sampled[i].drafted_card_ids)
                s2 = set(sampled[i + 1].drafted_card_ids)
                if s1 | s2:
                    overlap = len(s1 & s2) / len(s1 | s2)
                    overlaps.append(overlap)
            s["card_overlap"] = statistics.mean(overlaps) if overlaps else 0
        else:
            s["card_overlap"] = 0

        agg[strategy] = s

    return agg


def print_results(agg: dict, label: str = ""):
    """Print formatted results."""
    if label:
        print(f"\n{'='*70}")
        print(f"  {label}")
        print(f"{'='*70}")

    for strategy, s in agg.items():
        print(f"\n--- {strategy} ---")
        print(f"  Picks 1-5: unique ARCHETYPES w/ S/A card/pack: {s['early_unique_archs']:.2f}  (target >= 3)")
        print(f"  Picks 1-5: S/A cards for archetype/pack:       {s['early_arch_fit']:.2f}  (target <= 2)")
        print(f"  Picks 6+:  S/A cards for archetype/pack:       {s['late_arch_fit']:.2f}  (target >= 2)")
        print(f"  Picks 6+:  C/F-tier cards/pack:                {s['late_off_arch']:.2f}  (target >= 0.5)")
        print(f"  Convergence pick:                              {s['convergence_pick']:.1f}  (target 5-8)")
        print(f"    (converged in {s['convergence_pct']:.0f}% of runs)")
        print(f"  Deck S/A concentration:                        {s['deck_concentration']*100:.1f}%  (target 60-80%)")
        print(f"  Card overlap (Jaccard):                        {s['card_overlap']*100:.1f}%  (target < 40%)")


# ─── Archetype Frequency ────────────────────────────────────────────────────

def measure_archetype_frequency(pool: list, n: int = NUM_SIMULATIONS,
                                threshold_low: int = 3, threshold_high: int = 8,
                                max_locks: int = 4) -> dict:
    """Measure which archetypes committed players end up in."""
    arch_counts = Counter()
    for _ in range(n):
        m = run_single_draft(pool, "archetype_committed", threshold_low, threshold_high, max_locks)
        # Reconstruct drafted cards to determine final archetype
        drafted_cards = [next(c for c in pool if c.id == cid) for cid in m.drafted_card_ids]
        scores = evaluate_archetype_strength(drafted_cards)
        best = max(scores, key=scores.get)
        arch_counts[best] += 1

    total = sum(arch_counts.values())
    return {k: v / total for k, v in arch_counts.items()}


# ─── Draft Traces ────────────────────────────────────────────────────────────

def print_draft_trace(pool: list, strategy: str, label: str,
                      threshold_low: int = 3, threshold_high: int = 8,
                      max_locks: int = 4, max_picks: int = 15):
    """Print a detailed pick-by-pick draft trace (first max_picks picks for brevity)."""
    print(f"\n{'='*70}")
    print(f"  DRAFT TRACE: {label} ({strategy})")
    print(f"  Thresholds: ({threshold_low}, {threshold_high}), Max locks: {max_locks}")
    print(f"{'='*70}")

    metrics, trace_lines = run_single_draft(
        pool, strategy, threshold_low, threshold_high, max_locks, trace=True
    )
    for i, line in enumerate(trace_lines):
        if i >= max_picks:
            print(f"  ... (showing first {max_picks} picks)")
            break
        print(line)

    print(f"\n  Final deck S/A concentration: {metrics.deck_concentration*100:.1f}%")
    if metrics.convergence_pick:
        print(f"  Convergence pick: {metrics.convergence_pick}")
    else:
        print(f"  Did not converge (never saw 2+ S/A archetype cards in a pack after pick 5)")


# ─── Parameter Sensitivity Sweeps ────────────────────────────────────────────

def run_threshold_sweep():
    """Test different threshold pairs."""
    print("\n" + "="*70)
    print("  PARAMETER SENSITIVITY: Threshold Pairs")
    print("="*70)

    threshold_pairs = [(2, 6), (3, 8), (4, 10), (5, 12)]
    for low, high in threshold_pairs:
        results, pool = run_simulations(500, threshold_low=low, threshold_high=high)
        agg = compute_aggregate(results)
        s = agg["archetype_committed"]
        print(f"  ({low},{high}): late_SA={s['late_arch_fit']:.2f} "
              f"conv={s['convergence_pick']:.1f} "
              f"conc={s['deck_concentration']*100:.1f}% "
              f"early_archs={s['early_unique_archs']:.2f} "
              f"off_CF={s['late_off_arch']:.2f}")


def run_single_threshold_test():
    """Test single threshold at 3 (original Round 1 design: only one lock per resonance)."""
    print("\n" + "="*70)
    print("  PARAMETER SENSITIVITY: Single Threshold at 3 (1 lock per resonance)")
    print("="*70)
    results, pool = run_simulations(500, threshold_low=3, threshold_high=999)
    agg = compute_aggregate(results)
    s = agg["archetype_committed"]
    print(f"  Single(3): late_SA={s['late_arch_fit']:.2f} "
          f"conv={s['convergence_pick']:.1f} "
          f"conc={s['deck_concentration']*100:.1f}% "
          f"early_archs={s['early_unique_archs']:.2f} "
          f"off_CF={s['late_off_arch']:.2f}")


def run_lock_cap_sweep():
    """Test max 4 vs max 6 total locks (capped at PACK_SIZE for pack building)."""
    print("\n" + "="*70)
    print("  PARAMETER SENSITIVITY: Lock Cap Variants")
    print("="*70)
    for max_locks in [4, 6]:
        results, pool = run_simulations(500, max_locks=max_locks)
        agg = compute_aggregate(results)
        s = agg["archetype_committed"]
        print(f"  MaxLocks={max_locks}: late_SA={s['late_arch_fit']:.2f} "
              f"conv={s['convergence_pick']:.1f} "
              f"conc={s['deck_concentration']*100:.1f}% "
              f"early_archs={s['early_unique_archs']:.2f} "
              f"off_CF={s['late_off_arch']:.2f}")


def run_symbol_dist_sweep():
    """Test different symbol distributions."""
    print("\n" + "="*70)
    print("  PARAMETER SENSITIVITY: Symbol Distribution")
    print("="*70)

    distributions = [
        ("Mostly 1-sym (60/25/15)", {1: 0.60, 2: 0.25, 3: 0.15}),
        ("Default (25/55/20)",      {1: 0.25, 2: 0.55, 3: 0.20}),
        ("Heavy 2-sym (15/70/15)",  {1: 0.15, 2: 0.70, 3: 0.15}),
        ("Mostly 3-sym (15/35/50)", {1: 0.15, 2: 0.35, 3: 0.50}),
    ]
    for label, dist in distributions:
        results, pool = run_simulations(500, symbol_dist=dist)
        agg = compute_aggregate(results)
        s = agg["archetype_committed"]
        print(f"  {label}: late_SA={s['late_arch_fit']:.2f} "
              f"conv={s['convergence_pick']:.1f} "
              f"conc={s['deck_concentration']*100:.1f}% "
              f"early_archs={s['early_unique_archs']:.2f} "
              f"off_CF={s['late_off_arch']:.2f}")


# ─── Fitness Model Validation ───────────────────────────────────────────────

def validate_fitness_model():
    """Print the fitness matrix for one archetype to verify correctness."""
    print("\n" + "="*70)
    print("  FITNESS MODEL VALIDATION")
    print("="*70)

    for home_name, home_pri, home_sec in ARCHETYPES:
        fitness = compute_fitness(home_name)
        s_count = sum(1 for t in fitness.values() if t == "S")
        a_count = sum(1 for t in fitness.values() if t == "A")
        b_count = sum(1 for t in fitness.values() if t == "B")
        c_count = sum(1 for t in fitness.values() if t == "C")
        f_count = sum(1 for t in fitness.values() if t == "F")
        tiers = " | ".join(f"{n}={fitness[n]}" for n in ARCHETYPE_NAMES)
        print(f"  {home_name:15s} ({home_pri}/{home_sec}): "
              f"S={s_count} A={a_count} B={b_count} C={c_count} F={f_count}")
        print(f"    {tiers}")

    # Also check pool composition
    pool = generate_card_pool()
    print(f"\n  Pool: {len(pool)} cards total")
    for arch in ARCHETYPE_NAMES:
        sa_cards = sum(1 for c in pool if card_fits_archetype(c, arch))
        cf_cards = sum(1 for c in pool if card_is_off_archetype(c, arch))
        b_cards = sum(1 for c in pool if c.archetype_fitness.get(arch, "F") == "B")
        print(f"    {arch:15s}: S/A={sa_cards} B={b_cards} C/F={cf_cards}")

    # Expected S/A in random 4-card pack
    for arch in ARCHETYPE_NAMES[:2]:
        sa_cards = sum(1 for c in pool if card_fits_archetype(c, arch))
        expected = 4 * sa_cards / len(pool)
        print(f"    Expected S/A cards for {arch} in random pack of 4: {expected:.2f}")


# ─── Main ────────────────────────────────────────────────────────────────────

def main():
    random.seed(42)

    print("="*70)
    print("  LANE LOCKING DRAFT SIMULATION (ARCHETYPE-LEVEL METRICS)")
    print("  Algorithm: 'Your pack has 4 slots; when your symbol count in a")
    print("  resonance first reaches 3, one open slot locks to that resonance;")
    print("  when it first reaches 8, a second slot locks.'")
    print("  Symbol distribution: 25% 1-sym, 55% 2-sym, 20% 3-sym (+ 36 generic)")
    print("="*70)

    # ── Fitness model validation ──
    validate_fitness_model()

    # ── Main simulation (1000 drafts) ──
    print("\n" + "="*70)
    print("  MAIN RESULTS (1000 drafts per strategy)")
    print("="*70)
    results, pool = run_simulations(1000)
    agg = compute_aggregate(results)
    print_results(agg)

    # ── Archetype frequency ──
    print("\n" + "="*70)
    print("  ARCHETYPE FREQUENCY (archetype_committed, 1000 drafts)")
    print("="*70)
    freq = measure_archetype_frequency(pool, 1000)
    for arch in ARCHETYPE_NAMES:
        pct = freq.get(arch, 0)
        status = "PASS" if 0.05 <= pct <= 0.20 else "FAIL"
        print(f"  {arch:15s}: {pct*100:5.1f}%  [{status}]")

    # ── Draft traces (show first 15 picks each for readability) ──
    print_draft_trace(pool, "archetype_committed", "Early Committer (Archetype-Committed)")
    print_draft_trace(pool, "power_chaser", "Flexible Player (Power Chaser)")
    print_draft_trace(pool, "signal_reader", "Signal Reader")

    # ── Parameter sensitivity sweeps ──
    run_threshold_sweep()
    run_single_threshold_test()
    run_lock_cap_sweep()
    run_symbol_dist_sweep()

    # ── Target Scorecard Summary ──
    print("\n" + "="*70)
    print("  TARGET SCORECARD (archetype_committed strategy)")
    print("  NOTE: All metrics at ARCHETYPE level (S/A fitness), not resonance level")
    print("="*70)
    s = agg["archetype_committed"]
    targets = [
        ("Picks 1-5: unique archetypes w/ S/A per pack", ">= 3", f"{s['early_unique_archs']:.2f}",
         s["early_unique_archs"] >= 3),
        ("Picks 1-5: S/A cards for archetype/pack", "<= 2", f"{s['early_arch_fit']:.2f}",
         s["early_arch_fit"] <= 2),
        ("Picks 6+: S/A cards for archetype/pack", ">= 2", f"{s['late_arch_fit']:.2f}",
         s["late_arch_fit"] >= 2),
        ("Picks 6+: C/F-tier cards/pack", ">= 0.5", f"{s['late_off_arch']:.2f}",
         s["late_off_arch"] >= 0.5),
        ("Convergence pick", "5-8", f"{s['convergence_pick']:.1f}",
         5 <= s["convergence_pick"] <= 8),
        ("Deck concentration", "60-80%", f"{s['deck_concentration']*100:.1f}%",
         60 <= s["deck_concentration"] * 100 <= 80),
        ("Card overlap", "< 40%", f"{s['card_overlap']*100:.1f}%",
         s["card_overlap"] * 100 < 40),
    ]
    max_freq = max(freq.values()) if freq else 0
    min_freq = min(freq.values()) if freq else 0
    arch_pass = max_freq <= 0.20 and min_freq >= 0.05
    targets.append(("Archetype freq", "5-20% each",
                     f"{min_freq*100:.1f}-{max_freq*100:.1f}%", arch_pass))

    print(f"  {'Metric':<50} {'Target':<12} {'Actual':<12} {'Result'}")
    print(f"  {'-'*50} {'-'*12} {'-'*12} {'-'*6}")
    pass_count = 0
    for name, target, actual, passed in targets:
        result = "PASS" if passed else "FAIL"
        if passed:
            pass_count += 1
        print(f"  {name:<50} {target:<12} {actual:<12} {result}")
    print(f"\n  {pass_count}/{len(targets)} targets passed")


if __name__ == "__main__":
    main()
