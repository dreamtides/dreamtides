#!/usr/bin/env python3
"""
Resonance Swap (Modified) — Pool Manipulation Draft Simulation

Algorithm (one sentence): "When you draft a card, 3 random cards matching its
primary resonance are added to the draft pool from a reserve, and 3 random
cards of other resonances are moved from the pool to the reserve."

Asymmetric starting pool: each run randomly boosts one resonance (+20 cards)
and suppresses another (-20 cards) to create a detectable early signal.

CORRECTED: All evaluation metrics operate at the ARCHETYPE level (S/A-tier
fitness), not at the resonance level. A card "fits" an archetype only if it
has S or A tier fitness for that specific archetype.
"""

import random
import math
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional

# ──────────────────────────────────────────────────────────────────────────────
# Core types
# ──────────────────────────────────────────────────────────────────────────────

class Resonance(Enum):
    EMBER = "Ember"
    STONE = "Stone"
    TIDE = "Tide"
    ZEPHYR = "Zephyr"

class Tier(Enum):
    S = 5
    A = 4
    B = 3
    C = 2
    F = 1

class Rarity(Enum):
    COMMON = "common"
    UNCOMMON = "uncommon"
    RARE = "rare"
    LEGENDARY = "legendary"

# 8 archetypes on a circle
ARCHETYPES = [
    ("Flash",        Resonance.ZEPHYR, Resonance.EMBER),   # 1
    ("Blink",        Resonance.EMBER,  Resonance.ZEPHYR),  # 2
    ("Storm",        Resonance.EMBER,  Resonance.STONE),   # 3
    ("Self-Discard", Resonance.STONE,  Resonance.EMBER),   # 4
    ("Self-Mill",    Resonance.STONE,  Resonance.TIDE),    # 5
    ("Sacrifice",    Resonance.TIDE,   Resonance.STONE),   # 6
    ("Warriors",     Resonance.TIDE,   Resonance.ZEPHYR),  # 7
    ("Ramp",         Resonance.ZEPHYR, Resonance.TIDE),    # 8
]

ARCHETYPE_NAMES = [a[0] for a in ARCHETYPES]

def archetype_index(name: str) -> int:
    return ARCHETYPE_NAMES.index(name)

def circle_distance(i: int, j: int) -> int:
    """Minimum distance on the 8-archetype circle."""
    return min(abs(i - j), 8 - abs(i - j))

@dataclass
class SimCard:
    id: int
    symbols: list  # list of Resonance, ordered. [] = generic
    archetype: str  # home archetype name
    archetype_fitness: dict = field(default_factory=dict)  # arch_name -> Tier
    rarity: Rarity = Rarity.COMMON
    power: float = 5.0

    @property
    def primary_resonance(self) -> Optional[Resonance]:
        return self.symbols[0] if self.symbols else None

# ──────────────────────────────────────────────────────────────────────────────
# Card pool generation
# ──────────────────────────────────────────────────────────────────────────────

def assign_fitness(card: SimCard):
    """Assign archetype fitness tiers based on home archetype position.

    - S-tier: home archetype
    - A-tier: adjacent archetype on the circle (distance 1, shares primary)
    - B-tier: any non-adjacent archetype that shares the home's secondary resonance
    - C-tier: distance 2-3 on circle with no shared secondary
    - F-tier: distance 4 (opposite) on circle with no shared secondary
    - Generic cards: B-tier in all archetypes
    """
    if card.archetype == "Generic":
        for name in ARCHETYPE_NAMES:
            card.archetype_fitness[name] = Tier.B
        return

    home_idx = archetype_index(card.archetype)
    home_secondary = ARCHETYPES[home_idx][2]

    for j, (name, pri, sec) in enumerate(ARCHETYPES):
        dist = circle_distance(home_idx, j)
        if dist == 0:
            card.archetype_fitness[name] = Tier.S
        elif dist == 1:
            # Adjacent on circle — always shares primary resonance
            card.archetype_fitness[name] = Tier.A
        else:
            # Check if the target archetype uses home's secondary resonance
            target_resonances = {pri, sec}
            if home_secondary in target_resonances:
                card.archetype_fitness[name] = Tier.B
            elif dist >= 4:
                card.archetype_fitness[name] = Tier.F
            else:
                card.archetype_fitness[name] = Tier.C

def generate_card_pool(
    symbol_dist: tuple = (0.25, 0.50, 0.15),
    seed: int = 42,
) -> list[SimCard]:
    """Generate 360 cards: 8 archetypes x ~40 + 36 generic.

    symbol_dist = (frac_1sym, frac_2sym, frac_3sym) among non-generic cards.
    """
    rng = random.Random(seed)
    cards = []
    card_id = 0

    frac_1, frac_2, frac_3 = symbol_dist
    non_generic_total = 324  # 360 - 36
    per_archetype = non_generic_total // 8  # 40 (with 4 remainder, distributed)

    remainder = non_generic_total - per_archetype * 8
    arch_counts = [per_archetype] * 8
    for i in range(remainder):
        arch_counts[i] += 1

    rarities = ([Rarity.COMMON] * 20 + [Rarity.UNCOMMON] * 12
                + [Rarity.RARE] * 6 + [Rarity.LEGENDARY] * 2)

    for arch_idx, (arch_name, pri, sec) in enumerate(ARCHETYPES):
        n = arch_counts[arch_idx]
        n1 = round(n * frac_1)
        n3 = round(n * frac_3)
        n2 = n - n1 - n3

        for i in range(n):
            if i < n1:
                symbols = [pri]
            elif i < n1 + n2:
                symbols = [pri, sec]
            else:
                # 3-symbol: mostly [P, P, S] with some [P, S, S]
                if rng.random() < 0.6:
                    symbols = [pri, pri, sec]
                else:
                    symbols = [pri, sec, sec]

            r = rarities[i % len(rarities)]
            power = rng.uniform(3.0, 9.0)
            card = SimCard(
                id=card_id, symbols=symbols, archetype=arch_name,
                rarity=r, power=power,
            )
            assign_fitness(card)
            cards.append(card)
            card_id += 1

    # 36 generic cards
    for i in range(36):
        r = [Rarity.COMMON, Rarity.UNCOMMON, Rarity.RARE, Rarity.LEGENDARY][i % 4]
        power = rng.uniform(4.0, 8.0)
        card = SimCard(
            id=card_id, symbols=[], archetype="Generic",
            rarity=r, power=power,
        )
        assign_fitness(card)
        cards.append(card)
        card_id += 1

    return cards

def generate_reserve_pool(
    main_pool: list[SimCard],
    reserve_size: int = 200,
    seed: int = 99,
) -> list[SimCard]:
    """Generate reserve cards mirroring the main pool's archetype distribution."""
    rng = random.Random(seed)
    cards = []
    base_id = max(c.id for c in main_pool) + 1

    per_resonance = reserve_size // 4
    remainder = reserve_size - per_resonance * 4

    # Group archetypes by primary resonance
    res_archetypes = defaultdict(list)
    for name, pri, sec in ARCHETYPES:
        res_archetypes[pri].append((name, pri, sec))

    card_id = base_id
    for res in Resonance:
        n = per_resonance + (1 if remainder > 0 else 0)
        if remainder > 0:
            remainder -= 1
        archs = res_archetypes[res]
        for i in range(n):
            arch_name, pri, sec = archs[i % len(archs)]
            sym_type = rng.choices([1, 2, 3], weights=[25, 50, 15])[0]
            if sym_type == 1:
                symbols = [pri]
            elif sym_type == 2:
                symbols = [pri, sec]
            else:
                symbols = [pri, pri, sec] if rng.random() < 0.6 else [pri, sec, sec]

            power = rng.uniform(3.0, 9.0)
            r = rng.choice([Rarity.COMMON, Rarity.UNCOMMON, Rarity.RARE])
            card = SimCard(
                id=card_id, symbols=symbols, archetype=arch_name,
                rarity=r, power=power,
            )
            assign_fitness(card)
            cards.append(card)
            card_id += 1

    return cards

# ──────────────────────────────────────────────────────────────────────────────
# Draft engine: Resonance Swap (Modified)
# ──────────────────────────────────────────────────────────────────────────────

class DraftState:
    def __init__(
        self,
        pool: list[SimCard],
        reserve: list[SimCard],
        swap_count: int = 3,
        asymmetric: bool = True,
        rng: random.Random = None,
    ):
        self.pool = list(pool)
        # Reserve indexed by primary resonance
        self.reserve = defaultdict(list)
        for c in reserve:
            pr = c.primary_resonance
            if pr is not None:
                self.reserve[pr].append(c)
        self.swap_count = swap_count
        self.rng = rng or random.Random()
        self.drafted: list[SimCard] = []
        self.symbol_counts: dict[Resonance, int] = {r: 0 for r in Resonance}

        if asymmetric:
            self._apply_asymmetric_start()

    def _apply_asymmetric_start(self):
        """Move 20 cards of one resonance from reserve to pool,
        remove 20 of another resonance from pool to reserve."""
        resonances = list(Resonance)
        self.rng.shuffle(resonances)
        boost_res = resonances[0]
        suppress_res = resonances[1]
        self.boosted_resonance = boost_res
        self.suppressed_resonance = suppress_res

        # Add 20 from reserve of boost_res to pool
        available = self.reserve[boost_res]
        to_add = min(20, len(available))
        added = available[:to_add]
        self.reserve[boost_res] = available[to_add:]
        self.pool.extend(added)

        # Remove 20 of suppress_res from pool to reserve
        suppress_cards = [c for c in self.pool
                          if c.primary_resonance == suppress_res]
        self.rng.shuffle(suppress_cards)
        to_remove = suppress_cards[:min(20, len(suppress_cards))]
        remove_ids = {c.id for c in to_remove}
        self.pool = [c for c in self.pool if c.id not in remove_ids]
        self.reserve[suppress_res].extend(to_remove)

    def make_pack(self, size: int = 4) -> list[SimCard]:
        """Draw 'size' random cards from the pool."""
        if len(self.pool) < size:
            return list(self.pool)
        return self.rng.sample(self.pool, size)

    def pick_card(self, card: SimCard):
        """Player picks a card. Execute the resonance swap."""
        # Remove card from pool
        self.pool = [c for c in self.pool if c.id != card.id]
        self.drafted.append(card)

        # Count symbols
        for i, sym in enumerate(card.symbols):
            weight = 2 if i == 0 else 1
            self.symbol_counts[sym] += weight

        # Execute swap based on picked card's primary resonance
        picked_primary = card.primary_resonance
        if picked_primary is None:
            return  # Generic card — no swap

        # Add swap_count cards of the same primary resonance from reserve
        available = self.reserve[picked_primary]
        to_add = min(self.swap_count, len(available))
        if to_add > 0:
            self.rng.shuffle(available)
            added = available[:to_add]
            self.reserve[picked_primary] = available[to_add:]
            self.pool.extend(added)

        # Remove swap_count cards of OTHER resonances from pool
        other_cards = [c for c in self.pool
                       if c.primary_resonance != picked_primary
                       and c.primary_resonance is not None]
        if other_cards:
            self.rng.shuffle(other_cards)
            to_remove = other_cards[:min(self.swap_count, len(other_cards))]
            remove_ids = {c.id for c in to_remove}
            self.pool = [c for c in self.pool if c.id not in remove_ids]
            for c in to_remove:
                self.reserve[c.primary_resonance].append(c)

# ──────────────────────────────────────────────────────────────────────────────
# Player strategies
# ──────────────────────────────────────────────────────────────────────────────

def card_fitness_score(card: SimCard, archetype: str) -> float:
    """Numeric score for how well a card fits an archetype."""
    tier = card.archetype_fitness.get(archetype, Tier.F)
    return tier.value

def is_sa_tier(card: SimCard, archetype: str) -> bool:
    """True if card has S or A tier fitness for the given archetype."""
    tier = card.archetype_fitness.get(archetype, Tier.F)
    return tier in (Tier.S, Tier.A)

def is_cf_tier(card: SimCard, archetype: str) -> bool:
    """True if card has C or F tier fitness for the given archetype."""
    tier = card.archetype_fitness.get(archetype, Tier.F)
    return tier in (Tier.C, Tier.F)

def player_best_archetype(drafted: list[SimCard]) -> str:
    """Determine which archetype the drafted cards best serve."""
    scores = {}
    for arch in ARCHETYPE_NAMES:
        scores[arch] = sum(card_fitness_score(c, arch) for c in drafted)
    return max(scores, key=scores.get)

def archetype_committed_pick(pack: list[SimCard], drafted: list[SimCard],
                              pick_num: int, rng: random.Random) -> SimCard:
    """Commits to best archetype around pick 5-6, picks highest fitness."""
    if pick_num < 5:
        # Early: pick best power with slight archetype lean
        if drafted:
            best_arch = player_best_archetype(drafted)
            return max(pack, key=lambda c: (
                c.power * 0.6 + card_fitness_score(c, best_arch) * 0.4))
        return max(pack, key=lambda c: c.power)
    else:
        best_arch = player_best_archetype(drafted)
        return max(pack, key=lambda c: (
            card_fitness_score(c, best_arch) * 2.0 + c.power * 0.3))

def power_chaser_pick(pack: list[SimCard], drafted: list[SimCard],
                       pick_num: int, rng: random.Random) -> SimCard:
    """Always picks highest raw power."""
    return max(pack, key=lambda c: c.power + rng.uniform(-0.5, 0.5))

def signal_reader_pick(pack: list[SimCard], drafted: list[SimCard],
                        pick_num: int, rng: random.Random) -> SimCard:
    """Evaluates which resonance appears most in packs and drafts toward it."""
    if pick_num < 5:
        res_count = Counter()
        for c in pack:
            for sym in c.symbols:
                res_count[sym] += 1
        if drafted:
            for c in drafted:
                for i, sym in enumerate(c.symbols):
                    res_count[sym] += 2 if i == 0 else 1

        if res_count:
            open_res = res_count.most_common(1)[0][0]
            target_archs = [name for name, pri, sec in ARCHETYPES
                            if pri == open_res]
            if target_archs:
                target = target_archs[0]
                return max(pack, key=lambda c: (
                    card_fitness_score(c, target) * 1.5 + c.power * 0.5))

        return max(pack, key=lambda c: c.power)
    else:
        best_arch = player_best_archetype(drafted)
        return max(pack, key=lambda c: (
            card_fitness_score(c, best_arch) * 2.0 + c.power * 0.3))

STRATEGIES = {
    "archetype_committed": archetype_committed_pick,
    "power_chaser": power_chaser_pick,
    "signal_reader": signal_reader_pick,
}

# ──────────────────────────────────────────────────────────────────────────────
# ARCHETYPE-LEVEL Metrics (corrected from resonance-level)
# ──────────────────────────────────────────────────────────────────────────────

def count_unique_archetypes_with_sa(pack: list[SimCard]) -> int:
    """Count how many distinct archetypes have at least one S/A card in pack.

    This is the corrected metric: measures archetype diversity, not resonance
    diversity. A pack of 4 Tide cards might all be Warriors S-tier, giving
    only 1-2 unique archetypes despite having 1 resonance type represented.
    """
    archetypes_present = set()
    for card in pack:
        for arch_name in ARCHETYPE_NAMES:
            if is_sa_tier(card, arch_name):
                archetypes_present.add(arch_name)
    return len(archetypes_present)

def count_sa_cards_for_archetype(pack: list[SimCard], archetype: str) -> int:
    """Count cards in pack with S or A tier fitness for the given archetype.

    This is the corrected metric: a card must be specifically good for the
    player's TARGET ARCHETYPE (not just match a resonance). A Tide card is
    only S/A for Warriors and Sacrifice (and their adjacents), not for all
    Tide archetypes equally.
    """
    return sum(1 for c in pack if is_sa_tier(c, archetype))

def count_cf_cards_for_archetype(pack: list[SimCard], archetype: str) -> int:
    """Count cards in pack with C or F tier fitness for the given archetype.

    Off-archetype cards = cards that are poor fits for the player's committed
    archetype. These are splashable picks from distant archetypes.
    """
    return sum(1 for c in pack if is_cf_tier(c, archetype))

def deck_concentration(drafted: list[SimCard], archetype: str) -> float:
    """Fraction of drafted cards that are S or A tier in the archetype."""
    if not drafted:
        return 0.0
    sa_count = sum(1 for c in drafted if is_sa_tier(c, archetype))
    return sa_count / len(drafted)

def card_overlap(drafted_a: list[SimCard], drafted_b: list[SimCard]) -> float:
    """Fraction of cards shared between two draft runs."""
    ids_a = {c.id for c in drafted_a}
    ids_b = {c.id for c in drafted_b}
    if not ids_a or not ids_b:
        return 0.0
    intersection = ids_a & ids_b
    return len(intersection) / max(len(ids_a), len(ids_b))

# ──────────────────────────────────────────────────────────────────────────────
# Run a single draft
# ──────────────────────────────────────────────────────────────────────────────

def run_draft(
    pool: list[SimCard],
    reserve: list[SimCard],
    strategy_fn,
    swap_count: int = 3,
    asymmetric: bool = True,
    seed: int = 0,
    trace: bool = False,
) -> dict:
    """Run a 30-pick draft and collect archetype-level metrics."""
    rng = random.Random(seed)
    state = DraftState(pool, reserve, swap_count=swap_count,
                       asymmetric=asymmetric, rng=rng)

    metrics = {
        "early_unique_archetypes": [],   # picks 1-5: unique archs with S/A per pack
        "early_sa_fitting": [],          # picks 1-5: S/A cards for emerging arch
        "late_sa_fitting": [],           # picks 6+: S/A cards for committed arch
        "late_cf_cards": [],             # picks 6+: C/F-tier cards per pack
        "convergence_pick": None,        # first pick where 2+ S/A consistently
        "all_packs": [],
        "all_picks": [],
        "pool_sizes": [],
    }

    consecutive_2plus = 0
    convergence_found = False

    for pick_num in range(30):
        pack = state.make_pack(4)
        if not pack:
            break

        picked = strategy_fn(pack, state.drafted, pick_num, rng)
        state.pick_card(picked)

        best_arch = (player_best_archetype(state.drafted)
                     if state.drafted else ARCHETYPE_NAMES[0])

        sa_fitting = count_sa_cards_for_archetype(pack, best_arch)
        unique_archs = count_unique_archetypes_with_sa(pack)
        cf_cards = count_cf_cards_for_archetype(pack, best_arch)

        if pick_num < 5:
            metrics["early_unique_archetypes"].append(unique_archs)
            metrics["early_sa_fitting"].append(sa_fitting)
        else:
            metrics["late_sa_fitting"].append(sa_fitting)
            metrics["late_cf_cards"].append(cf_cards)

        # Convergence detection: 3 consecutive packs with 2+ S/A cards
        if sa_fitting >= 2:
            consecutive_2plus += 1
        else:
            consecutive_2plus = 0
        if consecutive_2plus >= 3 and not convergence_found:
            metrics["convergence_pick"] = pick_num - 2
            convergence_found = True

        if trace:
            metrics["all_packs"].append(pack)
            metrics["all_picks"].append(picked)
            metrics["pool_sizes"].append(len(state.pool))

    # Deck concentration
    best_arch = player_best_archetype(state.drafted)
    metrics["deck_concentration"] = deck_concentration(state.drafted, best_arch)
    metrics["drafted"] = state.drafted
    metrics["best_arch"] = best_arch
    metrics["boosted"] = getattr(state, "boosted_resonance", None)
    metrics["suppressed"] = getattr(state, "suppressed_resonance", None)

    return metrics

# ──────────────────────────────────────────────────────────────────────────────
# Signal detection test
# ──────────────────────────────────────────────────────────────────────────────

def test_signal_detection(pool, reserve, n_trials=500, seed=1000):
    """Test whether a signal-reader can identify the boosted resonance
    from the first 5 packs."""
    correct = 0
    for trial in range(n_trials):
        rng = random.Random(seed + trial)
        state = DraftState(pool, reserve, swap_count=3, asymmetric=True, rng=rng)
        boosted = state.boosted_resonance

        res_counts = Counter()
        for _ in range(5):
            pack = state.make_pack(4)
            if not pack:
                break
            for c in pack:
                if c.primary_resonance:
                    res_counts[c.primary_resonance] += 1
            picked = rng.choice(pack)
            state.pick_card(picked)

        if res_counts:
            guessed = res_counts.most_common(1)[0][0]
            if guessed == boosted:
                correct += 1

    return correct / n_trials

# ──────────────────────────────────────────────────────────────────────────────
# Main simulation
# ──────────────────────────────────────────────────────────────────────────────

def run_simulation(
    n_drafts: int = 1000,
    swap_count: int = 3,
    asymmetric: bool = True,
    reserve_size: int = 200,
    symbol_dist: tuple = (0.25, 0.50, 0.15),
    label: str = "",
    verbose: bool = True,
):
    """Run full simulation across all strategies with archetype-level metrics."""
    pool = generate_card_pool(symbol_dist=symbol_dist, seed=42)
    reserve = generate_reserve_pool(pool, reserve_size=reserve_size, seed=99)

    results = {}

    for strat_name, strat_fn in STRATEGIES.items():
        all_metrics = []
        for i in range(n_drafts):
            m = run_draft(pool, reserve, strat_fn, swap_count=swap_count,
                          asymmetric=asymmetric,
                          seed=i * 7 + hash(strat_name) % 1000)
            all_metrics.append(m)

        # Aggregate archetype-level metrics
        early_unique_archs = [v for m in all_metrics
                              for v in m["early_unique_archetypes"]]
        early_sa = [v for m in all_metrics for v in m["early_sa_fitting"]]
        late_sa = [v for m in all_metrics for v in m["late_sa_fitting"]]
        late_cf = [v for m in all_metrics for v in m["late_cf_cards"]]
        convergence_picks = [m["convergence_pick"] for m in all_metrics
                             if m["convergence_pick"] is not None]
        deck_concs = [m["deck_concentration"] for m in all_metrics]

        # Overlap: compare pairs of runs
        overlaps = []
        for j in range(min(100, n_drafts - 1)):
            overlaps.append(card_overlap(all_metrics[j]["drafted"],
                                         all_metrics[j + 1]["drafted"]))

        # Archetype frequency
        arch_freq = Counter()
        for m in all_metrics:
            arch_freq[m["best_arch"]] += 1

        avg = lambda lst: sum(lst) / len(lst) if lst else 0

        results[strat_name] = {
            "early_unique_archetypes": avg(early_unique_archs),
            "early_sa_fitting": avg(early_sa),
            "late_sa_fitting": avg(late_sa),
            "late_cf_cards": avg(late_cf),
            "convergence_pick": (avg(convergence_picks)
                                 if convergence_picks else None),
            "convergence_rate": len(convergence_picks) / n_drafts,
            "deck_concentration": avg(deck_concs),
            "overlap": avg(overlaps),
            "arch_freq": arch_freq,
            "n_drafts": n_drafts,
        }

    if verbose:
        print(f"\n{'='*70}")
        print(f"SIMULATION: {label or 'Resonance Swap (Modified)'}")
        print(f"  swap_count={swap_count}, asymmetric={asymmetric}, "
              f"reserve_size={reserve_size}, symbol_dist={symbol_dist}")
        print(f"  n_drafts={n_drafts}")
        print(f"{'='*70}")

        for strat_name, r in results.items():
            print(f"\n--- Strategy: {strat_name} ---")
            print(f"  Picks 1-5 unique archs with S/A/pack: "
                  f"{r['early_unique_archetypes']:.2f} (target >= 3)")
            print(f"  Picks 1-5 S/A cards for arch/pack:    "
                  f"{r['early_sa_fitting']:.2f} (target <= 2)")
            print(f"  Picks 6+ S/A cards for arch/pack:     "
                  f"{r['late_sa_fitting']:.2f} (target >= 2)")
            print(f"  Picks 6+ C/F-tier cards/pack:         "
                  f"{r['late_cf_cards']:.2f} (target >= 0.5)")
            if r["convergence_pick"] is not None:
                print(f"  Convergence pick:                     "
                      f"{r['convergence_pick']:.1f} (target 5-8)")
            else:
                print(f"  Convergence pick:                     "
                      f"N/A (never converged)")
            print(f"  Convergence rate:                     "
                  f"{r['convergence_rate']*100:.1f}%")
            print(f"  Deck concentration (S/A):             "
                  f"{r['deck_concentration']*100:.1f}% (target 60-80%)")
            print(f"  Run-to-run overlap:                   "
                  f"{r['overlap']*100:.1f}% (target < 40%)")

            total = sum(r["arch_freq"].values())
            print(f"  Archetype distribution:")
            for arch in ARCHETYPE_NAMES:
                count = r["arch_freq"].get(arch, 0)
                pct = count / total * 100 if total > 0 else 0
                marker = ""
                if pct > 20:
                    marker = " *** >20%"
                elif 0 < pct < 5:
                    marker = " *** <5%"
                elif pct == 0:
                    marker = " (0%)"
                print(f"    {arch:15s}: {pct:5.1f}%{marker}")
            gen_count = r["arch_freq"].get("Generic", 0)
            if gen_count > 0:
                print(f"    {'Generic':15s}: "
                      f"{gen_count/total*100:5.1f}%")

    return results

def run_draft_trace(pool, reserve, strategy_fn, strat_name, seed=42,
                    swap_count=3, asymmetric=True):
    """Run and print a detailed draft trace with archetype-level metrics."""
    rng = random.Random(seed)
    state = DraftState(pool, reserve, swap_count=swap_count,
                       asymmetric=asymmetric, rng=rng)

    print(f"\n{'='*70}")
    print(f"DRAFT TRACE: {strat_name}")
    print(f"  Boosted: "
          f"{state.boosted_resonance.value if hasattr(state, 'boosted_resonance') else 'N/A'}")
    print(f"  Suppressed: "
          f"{state.suppressed_resonance.value if hasattr(state, 'suppressed_resonance') else 'N/A'}")
    print(f"  Starting pool size: {len(state.pool)}")
    print(f"{'='*70}")

    for pick_num in range(30):
        pack = state.make_pack(4)
        if not pack:
            break

        picked = strategy_fn(pack, state.drafted, pick_num, rng)

        best_arch = (player_best_archetype(state.drafted)
                     if len(state.drafted) > 0 else "?")
        sa_fitting = (count_sa_cards_for_archetype(pack, best_arch)
                      if best_arch != "?" else 0)
        cf_cards = (count_cf_cards_for_archetype(pack, best_arch)
                    if best_arch != "?" else 0)
        unique_archs = count_unique_archetypes_with_sa(pack)

        sym_str = lambda c: "/".join(s.value for s in c.symbols) if c.symbols else "Generic"
        arch_short = lambda c: c.archetype[:10]

        print(f"\nPick {pick_num+1}:")
        print(f"  Pool size: {len(state.pool)} | Best archetype: {best_arch}")
        print(f"  Symbol counts: "
              f"{', '.join(f'{r.value}={state.symbol_counts[r]}' for r in Resonance)}")
        print(f"  Pack ({unique_archs} unique archs with S/A, "
              f"{sa_fitting} S/A for {best_arch}, {cf_cards} C/F):")
        for c in pack:
            picked_mark = " <-- PICKED" if c.id == picked.id else ""
            tier_for_best = (card_fitness_score(c, best_arch)
                             if best_arch != "?" else 0)
            tier_name = (c.archetype_fitness.get(best_arch, Tier.F).name
                         if best_arch != "?" else "?")
            print(f"    [{sym_str(c):20s}] arch={arch_short(c):12s} "
                  f"pwr={c.power:.1f} tier={tier_name}{picked_mark}")

        state.pick_card(picked)

        pool_res = Counter()
        for c in state.pool:
            pr = c.primary_resonance
            if pr:
                pool_res[pr] += 1
        total_with_res = sum(pool_res.values())
        if total_with_res > 0:
            dist_str = ", ".join(
                f"{r.value}={pool_res[r]}({pool_res[r]/total_with_res*100:.0f}%)"
                for r in Resonance)
            generic_count = len(state.pool) - total_with_res
            print(f"  Pool after swap: {dist_str} + {generic_count} generic")

    # Final deck summary
    best_arch = player_best_archetype(state.drafted)
    conc = deck_concentration(state.drafted, best_arch)
    print(f"\n--- Final Deck Summary ---")
    print(f"  Best archetype: {best_arch}")
    print(f"  S/A concentration: {conc*100:.1f}%")

    # Show archetype tier breakdown
    tier_counts = Counter()
    for c in state.drafted:
        tier = c.archetype_fitness.get(best_arch, Tier.F)
        tier_counts[tier.name] += 1
    print(f"  Tier breakdown: "
          f"{', '.join(f'{t}={tier_counts.get(t,0)}' for t in ['S','A','B','C','F'])}")

    res_in_deck = Counter()
    for c in state.drafted:
        if c.primary_resonance:
            res_in_deck[c.primary_resonance] += 1
    print(f"  Deck resonance mix: "
          f"{', '.join(f'{r.value}={res_in_deck.get(r,0)}' for r in Resonance)}")


def main():
    print("=" * 70)
    print("RESONANCE SWAP (MODIFIED) — ARCHETYPE-LEVEL EVALUATION")
    print("=" * 70)
    print()
    print('Algorithm: "When you draft a card, 3 random cards matching its')
    print("primary resonance are added to the draft pool from a reserve,")
    print("and 3 random cards of other resonances are moved from the pool")
    print('to the reserve."')
    print()
    print("Asymmetric start: +20 of one resonance, -20 of another.")
    print()
    print("CRITICAL CORRECTION: All metrics now measured at the ARCHETYPE")
    print("level (S/A tier fitness), not at the resonance level.")
    print()

    # ──────────────────────────────────────────────────────────────────────
    # Main simulation
    # ──────────────────────────────────────────────────────────────────────
    print("\n" + "=" * 70)
    print("MAIN SIMULATION (1000 drafts, swap=3, asymmetric, reserve=200)")
    print("=" * 70)
    main_results = run_simulation(
        n_drafts=1000, swap_count=3, asymmetric=True,
        reserve_size=200, symbol_dist=(0.25, 0.50, 0.15),
        label="Main Configuration",
    )

    # ──────────────────────────────────────────────────────────────────────
    # Signal detection
    # ──────────────────────────────────────────────────────────────────────
    print("\n" + "=" * 70)
    print("SIGNAL DETECTION TEST")
    print("=" * 70)
    pool = generate_card_pool(seed=42)
    reserve = generate_reserve_pool(pool, reserve_size=200, seed=99)
    detection_rate = test_signal_detection(pool, reserve, n_trials=500,
                                           seed=2000)
    print(f"  Boosted resonance correctly identified from first 5 packs: "
          f"{detection_rate*100:.1f}%")
    print(f"  Random baseline: 25.0%")
    print(f"  Lift over random: "
          f"{(detection_rate - 0.25)*100:.1f} percentage points")

    # ──────────────────────────────────────────────────────────────────────
    # Parameter sensitivity sweeps
    # ──────────────────────────────────────────────────────────────────────
    print("\n" + "=" * 70)
    print("PARAMETER SENSITIVITY SWEEPS")
    print("=" * 70)

    # Swap count sweep
    print("\n--- Swap Count Sweep (2, 3, 4) ---")
    for sc in [2, 3, 4]:
        run_simulation(
            n_drafts=500, swap_count=sc, asymmetric=True,
            reserve_size=200, symbol_dist=(0.25, 0.50, 0.15),
            label=f"Swap Count = {sc}",
        )

    # Symmetric vs asymmetric
    print("\n--- Symmetric vs Asymmetric Starting Pool ---")
    run_simulation(
        n_drafts=500, swap_count=3, asymmetric=False,
        reserve_size=200, symbol_dist=(0.25, 0.50, 0.15),
        label="Symmetric (no asymmetric start)",
    )
    run_simulation(
        n_drafts=500, swap_count=3, asymmetric=True,
        reserve_size=200, symbol_dist=(0.25, 0.50, 0.15),
        label="Asymmetric (+20/-20)",
    )

    # Reserve size sweep
    print("\n--- Reserve Size Sweep (150, 200, 300) ---")
    for rs in [150, 200, 300]:
        run_simulation(
            n_drafts=500, swap_count=3, asymmetric=True,
            reserve_size=rs, symbol_dist=(0.25, 0.50, 0.15),
            label=f"Reserve Size = {rs}",
        )

    # ──────────────────────────────────────────────────────────────────────
    # Draft traces
    # ──────────────────────────────────────────────────────────────────────
    print("\n" + "=" * 70)
    print("DETAILED DRAFT TRACES")
    print("=" * 70)

    pool = generate_card_pool(seed=42)
    reserve = generate_reserve_pool(pool, reserve_size=200, seed=99)

    # Trace 1: Early committer (archetype-committed)
    run_draft_trace(pool, reserve, archetype_committed_pick,
                    "Early Committer (archetype-committed)", seed=777)

    # Trace 2: Flexible player (power-chaser who stays flexible)
    run_draft_trace(pool, reserve, power_chaser_pick,
                    "Flexible Player (power-chaser)", seed=888)

    # Trace 3: Signal reader
    run_draft_trace(pool, reserve, signal_reader_pick,
                    "Signal Reader", seed=999)

    # ──────────────────────────────────────────────────────────────────────
    # Summary scorecard
    # ──────────────────────────────────────────────────────────────────────
    print("\n" + "=" * 70)
    print("TARGET SCORECARD (archetype_committed strategy, main config)")
    print("All metrics at ARCHETYPE level (S/A tier fitness)")
    print("=" * 70)
    r = main_results["archetype_committed"]
    targets = [
        ("Picks 1-5: unique archs w/ S/A per pack",
         ">= 3", r["early_unique_archetypes"],
         r["early_unique_archetypes"] >= 3.0),
        ("Picks 1-5: S/A cards for arch per pack",
         "<= 2", r["early_sa_fitting"],
         r["early_sa_fitting"] <= 2.0),
        ("Picks 6+: S/A cards for arch per pack",
         ">= 2", r["late_sa_fitting"],
         r["late_sa_fitting"] >= 2.0),
        ("Picks 6+: C/F-tier cards per pack",
         ">= 0.5", r["late_cf_cards"],
         r["late_cf_cards"] >= 0.5),
        ("Convergence pick", "5-8", r["convergence_pick"],
         r["convergence_pick"] is not None and
         5 <= r["convergence_pick"] <= 8),
        ("Deck concentration (S/A)",
         "60-80%", f"{r['deck_concentration']*100:.1f}%",
         0.60 <= r["deck_concentration"] <= 0.80),
        ("Run-to-run overlap",
         "< 40%", f"{r['overlap']*100:.1f}%",
         r["overlap"] < 0.40),
    ]

    # Archetype frequency
    total = sum(r["arch_freq"].values())
    arch_pcts = {arch: r["arch_freq"].get(arch, 0) / total * 100
                 for arch in ARCHETYPE_NAMES}
    max_arch_pct = max(arch_pcts.values())
    min_arch_pct = min(arch_pcts.values()) if arch_pcts else 0
    arch_pass = max_arch_pct <= 20 and min_arch_pct >= 5
    targets.append(("Archetype frequency", "5-20% each",
                     f"{min_arch_pct:.1f}%-{max_arch_pct:.1f}%", arch_pass))

    print(f"\n{'Metric':<45s} {'Target':<12s} {'Actual':<12s} {'Result':<8s}")
    print("-" * 80)
    for name, target, actual, passed in targets:
        actual_str = (f"{actual:.2f}" if isinstance(actual, float)
                      else str(actual))
        result = "PASS" if passed else "FAIL"
        print(f"{name:<45s} {target:<12s} {actual_str:<12s} {result:<8s}")

    passes = sum(1 for _, _, _, p in targets if p)
    print(f"\nTotal: {passes}/{len(targets)} targets passed")


if __name__ == "__main__":
    main()
