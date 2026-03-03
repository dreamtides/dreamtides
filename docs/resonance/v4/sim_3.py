#!/usr/bin/env python3
"""
Resonance V4 Simulation — Agent 3: Pack Widening v2 (Economic & Resource Domain)

Algorithm: "Each symbol you draft earns 1 matching token (primary earns 2);
before seeing a pack, you may spend 2 tokens of one resonance to add 2 extra
cards with that primary resonance to the pack."

Also implements V3 Lane Locking baseline (threshold 3/8, primary=2) for comparison.
"""

import random
import statistics
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional

# ---------------------------------------------------------------------------
# Data model
# ---------------------------------------------------------------------------

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

# Archetypes on the circle (index 0-7)
ARCHETYPES = [
    "Flash/Tempo/Prison",    # 0: Zephyr primary, Ember secondary
    "Blink/Flicker",         # 1: Ember primary, Zephyr secondary
    "Storm/Spellslinger",    # 2: Ember primary, Stone secondary
    "Self-Discard",          # 3: Stone primary, Ember secondary
    "Self-Mill/Reanimator",  # 4: Stone primary, Tide secondary
    "Sacrifice/Abandon",     # 5: Tide primary, Stone secondary
    "Warriors/Midrange",     # 6: Tide primary, Zephyr secondary
    "Ramp/Spirit Animals",   # 7: Zephyr primary, Tide secondary
]

ARCHETYPE_RESONANCES = {
    "Flash/Tempo/Prison":    (Resonance.ZEPHYR, Resonance.EMBER),
    "Blink/Flicker":         (Resonance.EMBER,  Resonance.ZEPHYR),
    "Storm/Spellslinger":    (Resonance.EMBER,  Resonance.STONE),
    "Self-Discard":          (Resonance.STONE,  Resonance.EMBER),
    "Self-Mill/Reanimator":  (Resonance.STONE,  Resonance.TIDE),
    "Sacrifice/Abandon":     (Resonance.TIDE,   Resonance.STONE),
    "Warriors/Midrange":     (Resonance.TIDE,   Resonance.ZEPHYR),
    "Ramp/Spirit Animals":   (Resonance.ZEPHYR, Resonance.TIDE),
}

# Adjacency: archetypes at positions i and (i+1)%8 are adjacent
def adjacent_archetypes(arch: str) -> list[str]:
    idx = ARCHETYPES.index(arch)
    return [ARCHETYPES[(idx - 1) % 8], ARCHETYPES[(idx + 1) % 8]]

def archetype_distance(a: str, b: str) -> int:
    """Circular distance on the archetype ring."""
    ia, ib = ARCHETYPES.index(a), ARCHETYPES.index(b)
    d = abs(ia - ib)
    return min(d, 8 - d)

@dataclass
class SimCard:
    id: int
    symbols: list  # list[Resonance], ordered, 0-3 elements
    archetype: str  # home archetype (or "Generic")
    archetype_fitness: dict = field(default_factory=dict)  # archetype -> Tier
    power: float = 0.0

    @property
    def primary_resonance(self) -> Optional[Resonance]:
        return self.symbols[0] if self.symbols else None

def fitness_tier(card: SimCard, target_arch: str) -> Tier:
    """Return the fitness tier of a card for a target archetype."""
    if card.archetype == "Generic":
        return Tier.B
    return card.archetype_fitness.get(target_arch, Tier.F)

def is_sa(card: SimCard, target_arch: str) -> bool:
    t = fitness_tier(card, target_arch)
    return t in (Tier.S, Tier.A)

# ---------------------------------------------------------------------------
# Card pool construction
# ---------------------------------------------------------------------------

def build_card_pool(sym_dist: tuple[float, float, float] = (0.20, 0.55, 0.25),
                    seed: int = 42) -> list[SimCard]:
    """Build 360 cards: 40 per archetype (320) + 36 generic."""
    rng = random.Random(seed)
    cards = []
    card_id = 0

    for arch in ARCHETYPES:
        primary_res, secondary_res = ARCHETYPE_RESONANCES[arch]
        n_cards = 40
        # Distribute symbol counts
        n1 = round(n_cards * sym_dist[0])
        n3 = round(n_cards * sym_dist[2])
        n2 = n_cards - n1 - n3

        for _ in range(n1):
            # 1-symbol: always primary
            syms = [primary_res]
            cards.append(_make_card(card_id, syms, arch, rng))
            card_id += 1

        for _ in range(n2):
            # 2-symbol: primary + secondary
            syms = [primary_res, secondary_res]
            cards.append(_make_card(card_id, syms, arch, rng))
            card_id += 1

        for _ in range(n3):
            # 3-symbol: primary + primary + secondary (double primary)
            syms = [primary_res, primary_res, secondary_res]
            cards.append(_make_card(card_id, syms, arch, rng))
            card_id += 1

    # 36 generic cards
    for _ in range(36):
        c = SimCard(id=card_id, symbols=[], archetype="Generic", power=rng.uniform(4, 8))
        # Generic = B-tier in all archetypes
        c.archetype_fitness = {a: Tier.B for a in ARCHETYPES}
        cards.append(c)
        card_id += 1

    # Assign archetype fitness for non-generic cards
    for c in cards:
        if c.archetype == "Generic":
            continue
        for target in ARCHETYPES:
            if target == c.archetype:
                c.archetype_fitness[target] = Tier.S
            elif archetype_distance(c.archetype, target) == 1:
                # Adjacent: A-tier if they share primary resonance, else B
                home_primary = ARCHETYPE_RESONANCES[c.archetype][0]
                tgt_primary = ARCHETYPE_RESONANCES[target][0]
                tgt_secondary = ARCHETYPE_RESONANCES[target][1]
                if home_primary == tgt_primary or home_primary == tgt_secondary:
                    c.archetype_fitness[target] = Tier.A
                else:
                    c.archetype_fitness[target] = Tier.B
            elif archetype_distance(c.archetype, target) == 2:
                c.archetype_fitness[target] = Tier.B
            else:
                c.archetype_fitness[target] = rng.choice([Tier.C, Tier.F])

    return cards

def _make_card(card_id: int, syms: list, arch: str, rng: random.Random) -> SimCard:
    return SimCard(id=card_id, symbols=syms, archetype=arch,
                   power=rng.uniform(3, 10))

# ---------------------------------------------------------------------------
# Resonance token tracking
# ---------------------------------------------------------------------------

@dataclass
class TokenBank:
    tokens: dict = field(default_factory=lambda: {r: 0 for r in Resonance})

    def earn(self, symbols: list):
        """Primary symbol earns 2, secondary/tertiary earn 1 each."""
        for i, sym in enumerate(symbols):
            self.tokens[sym] += 2 if i == 0 else 1

    def can_spend(self, resonance: Resonance, cost: int) -> bool:
        return self.tokens[resonance] >= cost

    def spend(self, resonance: Resonance, cost: int):
        self.tokens[resonance] -= cost

    def best_resonance(self) -> Resonance:
        return max(self.tokens, key=lambda r: self.tokens[r])

    def total(self) -> int:
        return sum(self.tokens.values())

# ---------------------------------------------------------------------------
# Pack Widening v2 draft algorithm
# ---------------------------------------------------------------------------

def generate_pack_widening(pool: list[SimCard], bank: TokenBank,
                           spend_cost: int = 2, bonus_count: int = 2,
                           spend_resonance: Optional[Resonance] = None,
                           rng: random.Random = None) -> list[SimCard]:
    """
    Generate a pack using Pack Widening v2.

    Base: 4 random cards from pool.
    If player spends `spend_cost` tokens of `spend_resonance`:
      add `bonus_count` extra cards with that primary resonance.
    """
    if rng is None:
        rng = random.Random()

    # Base pack: 4 random cards
    base = rng.sample(pool, min(4, len(pool)))

    # Check if spending
    if spend_resonance and bank.can_spend(spend_resonance, spend_cost):
        bank.spend(spend_resonance, spend_cost)
        # Bonus cards: random cards with spend_resonance as PRIMARY
        eligible = [c for c in pool if c.primary_resonance == spend_resonance
                    and c not in base]
        bonus = rng.sample(eligible, min(bonus_count, len(eligible)))
        return base + bonus
    else:
        return base

# ---------------------------------------------------------------------------
# V3 Lane Locking baseline
# ---------------------------------------------------------------------------

@dataclass
class LaneLockState:
    symbol_counts: dict = field(default_factory=lambda: {r: 0 for r in Resonance})
    locked_slots: list = field(default_factory=list)  # list of Resonance

    def add_symbols(self, symbols: list):
        for i, sym in enumerate(symbols):
            self.symbol_counts[sym] += 2 if i == 0 else 1
        # Check for new locks
        self._check_locks()

    def _check_locks(self):
        threshold_1 = 3
        threshold_2 = 8
        for res in Resonance:
            count_of_res = self.locked_slots.count(res)
            if self.symbol_counts[res] >= threshold_2 and count_of_res < 2:
                while self.locked_slots.count(res) < 2:
                    self.locked_slots.append(res)
            elif self.symbol_counts[res] >= threshold_1 and count_of_res < 1:
                self.locked_slots.append(res)

def generate_lane_locking(pool: list[SimCard], state: LaneLockState,
                          rng: random.Random = None) -> list[SimCard]:
    """
    V3 Lane Locking: threshold 3 locks 1 slot, threshold 8 locks 2 slots.
    Locked slots are filled with random cards of that resonance (primary match).
    Remaining slots are random from pool.
    """
    if rng is None:
        rng = random.Random()

    pack = []
    used = set()

    # Fill locked slots
    for res in state.locked_slots[:4]:  # max 4 locked slots
        eligible = [c for c in pool if c.primary_resonance == res and c.id not in used]
        if eligible:
            card = rng.choice(eligible)
            pack.append(card)
            used.add(card.id)

    # Fill remaining slots randomly
    remaining_slots = 4 - len(pack)
    if remaining_slots > 0:
        eligible = [c for c in pool if c.id not in used]
        fill = rng.sample(eligible, min(remaining_slots, len(eligible)))
        pack.extend(fill)

    return pack

# ---------------------------------------------------------------------------
# Player strategies
# ---------------------------------------------------------------------------

def pick_archetype_committed(pack: list[SimCard], target_arch: str,
                             rng: random.Random) -> SimCard:
    """Pick card with highest fitness for target archetype."""
    tier_order = {Tier.S: 0, Tier.A: 1, Tier.B: 2, Tier.C: 3, Tier.F: 4}
    best = min(pack, key=lambda c: (tier_order[fitness_tier(c, target_arch)],
                                     -c.power))
    return best

def pick_power_chaser(pack: list[SimCard], rng: random.Random) -> SimCard:
    """Pick highest raw power card."""
    return max(pack, key=lambda c: c.power)

def pick_signal_reader(pack: list[SimCard], seen_counts: dict,
                       pick_num: int, rng: random.Random) -> tuple[SimCard, str]:
    """
    Evaluate which archetype is most available based on S/A cards seen.
    Commit after pick 5 to the archetype with most S/A sightings.
    """
    if pick_num <= 5:
        # Count S/A cards per archetype in this pack
        for c in pack:
            for arch in ARCHETYPES:
                if is_sa(c, arch):
                    seen_counts[arch] = seen_counts.get(arch, 0) + 1
        # Pick the card that is S/A for the most-seen archetype so far
        best_arch = max(ARCHETYPES, key=lambda a: seen_counts.get(a, 0))
        best = max(pack, key=lambda c: (1 if is_sa(c, best_arch) else 0, c.power))
        return best, best_arch
    else:
        # Committed: pick best for target
        best_arch = max(ARCHETYPES, key=lambda a: seen_counts.get(a, 0))
        best = pick_archetype_committed(pack, best_arch, rng)
        return best, best_arch

# ---------------------------------------------------------------------------
# Draft simulation
# ---------------------------------------------------------------------------

@dataclass
class DraftMetrics:
    # Per-pack metrics stored as lists
    sa_per_pack_early: list = field(default_factory=list)     # picks 1-5
    unique_archetypes_early: list = field(default_factory=list)
    sa_per_pack_late: list = field(default_factory=list)      # picks 6+
    off_arch_per_pack_late: list = field(default_factory=list)
    convergence_pick: int = 30
    deck_sa_count: int = 0
    deck_total: int = 0
    picked_card_ids: list = field(default_factory=list)

def run_single_draft(pool: list[SimCard], target_arch: str,
                     strategy: str, algorithm: str,
                     spend_cost: int = 2, bonus_count: int = 2,
                     rng: random.Random = None,
                     trace: bool = False) -> DraftMetrics:
    if rng is None:
        rng = random.Random()

    metrics = DraftMetrics()
    bank = TokenBank()
    lane_state = LaneLockState()
    seen_counts = {}  # for signal reader
    committed_arch = target_arch if strategy == "committed" else None
    trace_lines = []

    for pick_num in range(1, 31):
        # Determine spend resonance for Pack Widening
        spend_resonance = None
        if algorithm == "pack_widening":
            if committed_arch and pick_num >= 3:
                primary_res = ARCHETYPE_RESONANCES[committed_arch][0]
                if bank.can_spend(primary_res, spend_cost):
                    spend_resonance = primary_res
            elif strategy == "signal_reader" and pick_num > 5 and committed_arch:
                primary_res = ARCHETYPE_RESONANCES[committed_arch][0]
                if bank.can_spend(primary_res, spend_cost):
                    spend_resonance = primary_res
            elif strategy == "power":
                # Power chaser: spend on strongest resonance if tokens available
                best_res = bank.best_resonance()
                if bank.can_spend(best_res, spend_cost) and bank.tokens[best_res] >= 4:
                    spend_resonance = best_res

        # Generate pack
        if algorithm == "pack_widening":
            pack = generate_pack_widening(pool, bank, spend_cost, bonus_count,
                                          spend_resonance, rng)
        elif algorithm == "lane_locking":
            pack = generate_lane_locking(pool, lane_state, rng)
        else:
            pack = rng.sample(pool, min(4, len(pool)))

        if not pack:
            break

        # Count S/A for target archetype
        eval_arch = committed_arch or target_arch
        sa_count = sum(1 for c in pack if is_sa(c, eval_arch))
        off_count = sum(1 for c in pack if fitness_tier(c, eval_arch) in (Tier.C, Tier.F))

        # Unique archetypes with S/A cards
        unique_sa_archs = set()
        for c in pack:
            for a in ARCHETYPES:
                if is_sa(c, a):
                    unique_sa_archs.add(a)

        if pick_num <= 5:
            metrics.sa_per_pack_early.append(sa_count)
            metrics.unique_archetypes_early.append(len(unique_sa_archs))
        else:
            metrics.sa_per_pack_late.append(sa_count)
            metrics.off_arch_per_pack_late.append(off_count)

        # Check convergence: first pick where 2+ S/A becomes regular
        # (defined as 2+ S/A in 3 of last 4 packs)
        if pick_num >= 6 and metrics.convergence_pick == 30:
            recent = metrics.sa_per_pack_late[-4:] if len(metrics.sa_per_pack_late) >= 4 else metrics.sa_per_pack_late
            if len(recent) >= 4 and sum(1 for x in recent if x >= 2) >= 3:
                metrics.convergence_pick = pick_num - 3  # where the window started

        # Pick a card
        if strategy == "committed":
            picked = pick_archetype_committed(pack, target_arch, rng)
        elif strategy == "power":
            picked = pick_power_chaser(pack, rng)
        elif strategy == "signal_reader":
            picked, committed_arch = pick_signal_reader(pack, seen_counts, pick_num, rng)
        else:
            picked = rng.choice(pack)

        metrics.picked_card_ids.append(picked.id)
        if is_sa(picked, eval_arch):
            metrics.deck_sa_count += 1
        metrics.deck_total += 1

        # Post-pick: update state
        if algorithm == "pack_widening":
            bank.earn(picked.symbols)
        elif algorithm == "lane_locking":
            lane_state.add_symbols(picked.symbols)

        # Update committed arch for signal reader
        if strategy == "signal_reader" and pick_num == 5:
            committed_arch = max(ARCHETYPES, key=lambda a: seen_counts.get(a, 0))

        if trace:
            spent = "YES" if spend_resonance else "no"
            trace_lines.append(
                f"  Pick {pick_num:2d}: pack_size={len(pack)}, "
                f"S/A={sa_count}, off={off_count}, spent={spent}, "
                f"picked={picked.archetype}({picked.symbols}), "
                f"tokens={dict((r.value, bank.tokens[r]) for r in Resonance) if algorithm == 'pack_widening' else 'N/A'}"
            )

    if trace:
        return metrics, trace_lines
    return metrics

# ---------------------------------------------------------------------------
# Simulation runner
# ---------------------------------------------------------------------------

def run_simulations(pool: list[SimCard], algorithm: str,
                    n_drafts: int = 1000,
                    spend_cost: int = 2, bonus_count: int = 2,
                    seed: int = 0) -> dict:
    """Run n_drafts for all 3 strategies, return aggregated metrics."""
    results = {}
    for strategy in ["committed", "power", "signal_reader"]:
        all_metrics = []
        for i in range(n_drafts):
            rng = random.Random(seed + i * 1000 + hash(strategy) % 10000)
            target_arch = ARCHETYPES[i % 8]  # cycle through archetypes
            m = run_single_draft(pool, target_arch, strategy, algorithm,
                                 spend_cost, bonus_count, rng)
            all_metrics.append((m, target_arch))
        results[strategy] = all_metrics
    return results

def compute_aggregate_metrics(results: dict) -> dict:
    """Compute all 8 measurable targets + variance from simulation results."""
    agg = {}
    for strategy, data in results.items():
        metrics_list = [m for m, _ in data]

        # Picks 1-5: unique archetypes with S/A cards
        early_unique = [x for m in metrics_list for x in m.unique_archetypes_early]
        avg_early_unique = statistics.mean(early_unique) if early_unique else 0

        # Picks 1-5: S/A for emerging archetype
        early_sa = [x for m in metrics_list for x in m.sa_per_pack_early]
        avg_early_sa = statistics.mean(early_sa) if early_sa else 0

        # Picks 6+: S/A for committed archetype
        late_sa = [x for m in metrics_list for x in m.sa_per_pack_late]
        avg_late_sa = statistics.mean(late_sa) if late_sa else 0
        std_late_sa = statistics.stdev(late_sa) if len(late_sa) > 1 else 0

        # Picks 6+: off-archetype cards
        late_off = [x for m in metrics_list for x in m.off_arch_per_pack_late]
        avg_late_off = statistics.mean(late_off) if late_off else 0

        # Convergence pick
        conv_picks = [m.convergence_pick for m in metrics_list]
        avg_conv = statistics.mean(conv_picks)

        # Deck concentration
        concentrations = [m.deck_sa_count / m.deck_total if m.deck_total > 0 else 0
                         for m in metrics_list]
        avg_concentration = statistics.mean(concentrations)

        # Run-to-run variety (card overlap between consecutive drafts with same target)
        overlaps = []
        by_arch = {}
        for m, target in data:
            by_arch.setdefault(target, []).append(set(m.picked_card_ids))
        for arch, decks in by_arch.items():
            for j in range(len(decks) - 1):
                if decks[j] and decks[j+1]:
                    overlap = len(decks[j] & decks[j+1]) / max(len(decks[j]), 1)
                    overlaps.append(overlap)
        avg_overlap = statistics.mean(overlaps) if overlaps else 0

        # Archetype frequency
        arch_counts = {}
        for m, target in data:
            # Determine the dominant archetype in the deck
            arch_cards = {}
            for cid in m.picked_card_ids:
                card = next((c for c in pool_global if c.id == cid), None)
                if card and card.archetype != "Generic":
                    arch_cards[card.archetype] = arch_cards.get(card.archetype, 0) + 1
            if arch_cards:
                dominant = max(arch_cards, key=arch_cards.get)
                arch_counts[dominant] = arch_counts.get(dominant, 0) + 1

        total_decks = sum(arch_counts.values()) if arch_counts else 1
        arch_freq = {a: arch_counts.get(a, 0) / total_decks for a in ARCHETYPES}

        # S/A distribution for committed player picks 6+
        sa_distribution = {i: 0 for i in range(7)}  # 0 to 6
        for m in metrics_list:
            for sa in m.sa_per_pack_late:
                sa_distribution[min(sa, 6)] = sa_distribution.get(min(sa, 6), 0) + 1
        total_late_packs = sum(sa_distribution.values()) if sum(sa_distribution.values()) > 0 else 1
        sa_dist_pct = {k: v / total_late_packs for k, v in sa_distribution.items()}

        agg[strategy] = {
            "avg_early_unique_archetypes": avg_early_unique,
            "avg_early_sa": avg_early_sa,
            "avg_late_sa": avg_late_sa,
            "std_late_sa": std_late_sa,
            "avg_late_off": avg_late_off,
            "avg_convergence_pick": avg_conv,
            "avg_concentration": avg_concentration,
            "avg_overlap": avg_overlap,
            "arch_freq": arch_freq,
            "sa_distribution": sa_dist_pct,
        }
    return agg

def per_archetype_convergence(pool: list[SimCard], algorithm: str,
                              n_per_arch: int = 125,
                              spend_cost: int = 2, bonus_count: int = 2,
                              seed: int = 0) -> dict:
    """For each archetype, run n_per_arch committed-player drafts and
    report average convergence pick."""
    conv_table = {}
    for arch in ARCHETYPES:
        conv_picks = []
        for i in range(n_per_arch):
            rng = random.Random(seed + i * 100 + ARCHETYPES.index(arch) * 10000)
            m = run_single_draft(pool, arch, "committed", algorithm,
                                 spend_cost, bonus_count, rng)
            conv_picks.append(m.convergence_pick)
        conv_table[arch] = statistics.mean(conv_picks)
    return conv_table

# ---------------------------------------------------------------------------
# Draft traces
# ---------------------------------------------------------------------------

def run_trace(pool: list[SimCard], target_arch: str, strategy: str,
              algorithm: str, spend_cost: int = 2, bonus_count: int = 2,
              seed: int = 99) -> list[str]:
    rng = random.Random(seed)
    result = run_single_draft(pool, target_arch, strategy, algorithm,
                              spend_cost, bonus_count, rng, trace=True)
    metrics, lines = result
    return lines

# ---------------------------------------------------------------------------
# Parameter sensitivity sweep
# ---------------------------------------------------------------------------

def sensitivity_sweep(pool: list[SimCard], param_name: str,
                      values: list, n_drafts: int = 500,
                      base_spend_cost: int = 2,
                      base_bonus_count: int = 2,
                      seed: int = 0) -> dict:
    """Sweep one parameter, return key metrics for committed strategy."""
    sweep_results = {}
    for val in values:
        sc = base_spend_cost
        bc = base_bonus_count
        if param_name == "spend_cost":
            sc = val
        elif param_name == "bonus_count":
            bc = val

        all_metrics = []
        for i in range(n_drafts):
            rng = random.Random(seed + i * 1000)
            target = ARCHETYPES[i % 8]
            m = run_single_draft(pool, target, "committed", "pack_widening",
                                 sc, bc, rng)
            all_metrics.append(m)

        late_sa = [x for m in all_metrics for x in m.sa_per_pack_late]
        avg_sa = statistics.mean(late_sa) if late_sa else 0
        std_sa = statistics.stdev(late_sa) if len(late_sa) > 1 else 0
        conv = statistics.mean([m.convergence_pick for m in all_metrics])

        sweep_results[val] = {
            "avg_late_sa": avg_sa,
            "std_late_sa": std_sa,
            "avg_convergence": conv,
        }
    return sweep_results

def symbol_distribution_sweep(n_drafts: int = 500, seed: int = 0) -> dict:
    """Sweep symbol distributions."""
    distributions = {
        "mostly_1sym": (0.60, 0.30, 0.10),
        "mostly_2sym": (0.20, 0.55, 0.25),
        "mostly_3sym": (0.10, 0.30, 0.60),
        "balanced": (0.33, 0.34, 0.33),
    }
    results = {}
    for name, dist in distributions.items():
        p = build_card_pool(sym_dist=dist, seed=seed)
        all_metrics = []
        for i in range(n_drafts):
            rng = random.Random(seed + i * 1000)
            target = ARCHETYPES[i % 8]
            m = run_single_draft(p, target, "committed", "pack_widening",
                                 2, 2, rng)
            all_metrics.append(m)

        late_sa = [x for m in all_metrics for x in m.sa_per_pack_late]
        avg_sa = statistics.mean(late_sa) if late_sa else 0
        std_sa = statistics.stdev(late_sa) if len(late_sa) > 1 else 0
        conv = statistics.mean([m.convergence_pick for m in all_metrics])

        results[name] = {
            "distribution": dist,
            "avg_late_sa": avg_sa,
            "std_late_sa": std_sa,
            "avg_convergence": conv,
        }
    return results

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

pool_global = []  # module-level for access in compute_aggregate_metrics

def main():
    global pool_global
    print("=" * 70)
    print("Resonance V4 Simulation — Agent 3: Pack Widening v2")
    print("=" * 70)

    # Build card pool
    pool = build_card_pool(sym_dist=(0.20, 0.55, 0.25), seed=42)
    pool_global = pool
    print(f"\nCard pool: {len(pool)} cards")
    print(f"  Generic: {sum(1 for c in pool if c.archetype == 'Generic')}")
    for arch in ARCHETYPES:
        count = sum(1 for c in pool if c.archetype == arch)
        print(f"  {arch}: {count}")

    # Verify S/A rates
    for arch in ARCHETYPES[:2]:
        sa_cards = sum(1 for c in pool if is_sa(c, arch))
        print(f"  S/A cards for {arch}: {sa_cards} ({sa_cards/len(pool)*100:.1f}%)")

    # ----- Pack Widening v2 -----
    print("\n" + "=" * 70)
    print("PACK WIDENING V2 (spend_cost=2, bonus_count=2)")
    print("=" * 70)

    pw_results = run_simulations(pool, "pack_widening", n_drafts=1000,
                                  spend_cost=2, bonus_count=2, seed=100)
    pw_agg = compute_aggregate_metrics(pw_results)

    for strategy in ["committed", "power", "signal_reader"]:
        a = pw_agg[strategy]
        print(f"\n  Strategy: {strategy}")
        print(f"    Picks 1-5 avg unique archetypes w/ S/A: {a['avg_early_unique_archetypes']:.2f}")
        print(f"    Picks 1-5 avg S/A for target:           {a['avg_early_sa']:.2f}")
        print(f"    Picks 6+ avg S/A for target:            {a['avg_late_sa']:.2f}")
        print(f"    Picks 6+ stddev S/A:                    {a['std_late_sa']:.2f}")
        print(f"    Picks 6+ avg off-archetype:             {a['avg_late_off']:.2f}")
        print(f"    Avg convergence pick:                   {a['avg_convergence_pick']:.1f}")
        print(f"    Deck concentration (S/A %):             {a['avg_concentration']*100:.1f}%")
        print(f"    Run-to-run overlap:                     {a['avg_overlap']*100:.1f}%")
        print(f"    S/A distribution (picks 6+):")
        for k in range(5):
            print(f"      {k} S/A cards: {a['sa_distribution'].get(k, 0)*100:.1f}%")

    # ----- Lane Locking baseline -----
    print("\n" + "=" * 70)
    print("V3 LANE LOCKING BASELINE (threshold 3/8, primary=2)")
    print("=" * 70)

    ll_results = run_simulations(pool, "lane_locking", n_drafts=1000, seed=100)
    ll_agg = compute_aggregate_metrics(ll_results)

    for strategy in ["committed", "power", "signal_reader"]:
        a = ll_agg[strategy]
        print(f"\n  Strategy: {strategy}")
        print(f"    Picks 1-5 avg unique archetypes w/ S/A: {a['avg_early_unique_archetypes']:.2f}")
        print(f"    Picks 1-5 avg S/A for target:           {a['avg_early_sa']:.2f}")
        print(f"    Picks 6+ avg S/A for target:            {a['avg_late_sa']:.2f}")
        print(f"    Picks 6+ stddev S/A:                    {a['std_late_sa']:.2f}")
        print(f"    Picks 6+ avg off-archetype:             {a['avg_late_off']:.2f}")
        print(f"    Avg convergence pick:                   {a['avg_convergence_pick']:.1f}")
        print(f"    Deck concentration (S/A %):             {a['avg_concentration']*100:.1f}%")
        print(f"    Run-to-run overlap:                     {a['avg_overlap']*100:.1f}%")
        print(f"    S/A distribution (picks 6+):")
        for k in range(5):
            print(f"      {k} S/A cards: {a['sa_distribution'].get(k, 0)*100:.1f}%")

    # ----- Per-archetype convergence -----
    print("\n" + "=" * 70)
    print("PER-ARCHETYPE CONVERGENCE TABLE")
    print("=" * 70)

    pw_conv = per_archetype_convergence(pool, "pack_widening", n_per_arch=125, seed=200)
    ll_conv = per_archetype_convergence(pool, "lane_locking", n_per_arch=125, seed=200)

    print(f"\n  {'Archetype':<25} {'Pack Widening':>15} {'Lane Locking':>15}")
    print(f"  {'-'*25} {'-'*15} {'-'*15}")
    for arch in ARCHETYPES:
        print(f"  {arch:<25} {pw_conv[arch]:>15.1f} {ll_conv[arch]:>15.1f}")

    # ----- Archetype frequency -----
    print("\n" + "=" * 70)
    print("ARCHETYPE FREQUENCY (committed strategy)")
    print("=" * 70)
    a = pw_agg["committed"]
    for arch in ARCHETYPES:
        freq = a["arch_freq"].get(arch, 0)
        print(f"  {arch:<25}: {freq*100:.1f}%")

    # ----- Parameter sensitivity -----
    print("\n" + "=" * 70)
    print("PARAMETER SENSITIVITY SWEEPS")
    print("=" * 70)

    print("\n  Spend cost sweep (bonus_count=2):")
    sc_sweep = sensitivity_sweep(pool, "spend_cost", [1, 2, 3, 4], n_drafts=500, seed=300)
    for val, r in sc_sweep.items():
        print(f"    cost={val}: avg_S/A={r['avg_late_sa']:.2f}, "
              f"std={r['std_late_sa']:.2f}, conv={r['avg_convergence']:.1f}")

    print("\n  Bonus count sweep (spend_cost=2):")
    bc_sweep = sensitivity_sweep(pool, "bonus_count", [1, 2, 3], n_drafts=500, seed=300)
    for val, r in bc_sweep.items():
        print(f"    bonus={val}: avg_S/A={r['avg_late_sa']:.2f}, "
              f"std={r['std_late_sa']:.2f}, conv={r['avg_convergence']:.1f}")

    print("\n  Symbol distribution sweep:")
    sd_sweep = symbol_distribution_sweep(n_drafts=500, seed=400)
    for name, r in sd_sweep.items():
        print(f"    {name} {r['distribution']}: avg_S/A={r['avg_late_sa']:.2f}, "
              f"std={r['std_late_sa']:.2f}, conv={r['avg_convergence']:.1f}")

    # ----- Earn rate sweep (primary weight) -----
    print("\n  Earn rate sweep (primary weight 1, 2, 3):")
    # For this we modify the TokenBank.earn behavior; do it manually
    for primary_weight in [1, 2, 3]:
        all_metrics = []
        for i in range(500):
            rng_d = random.Random(300 + i * 1000)
            target = ARCHETYPES[i % 8]
            bank = TokenBank()
            # Custom draft loop with modified earn
            metrics = DraftMetrics()
            for pick_num in range(1, 31):
                spend_resonance = None
                if pick_num >= 3:
                    primary_res = ARCHETYPE_RESONANCES[target][0]
                    if bank.can_spend(primary_res, 2):
                        spend_resonance = primary_res
                pack = generate_pack_widening(pool, bank, 2, 2, spend_resonance, rng_d)
                if not pack:
                    break
                sa_count = sum(1 for c in pack if is_sa(c, target))
                off_count = sum(1 for c in pack if fitness_tier(c, target) in (Tier.C, Tier.F))
                if pick_num <= 5:
                    metrics.sa_per_pack_early.append(sa_count)
                else:
                    metrics.sa_per_pack_late.append(sa_count)
                    metrics.off_arch_per_pack_late.append(off_count)
                # Convergence check
                if pick_num >= 6 and metrics.convergence_pick == 30:
                    recent = metrics.sa_per_pack_late[-4:]
                    if len(recent) >= 4 and sum(1 for x in recent if x >= 2) >= 3:
                        metrics.convergence_pick = pick_num - 3
                picked = pick_archetype_committed(pack, target, rng_d)
                # Custom earn with modified primary weight
                for idx, sym in enumerate(picked.symbols):
                    bank.tokens[sym] += primary_weight if idx == 0 else 1
                if is_sa(picked, target):
                    metrics.deck_sa_count += 1
                metrics.deck_total += 1
            all_metrics.append(metrics)
        late_sa = [x for m in all_metrics for x in m.sa_per_pack_late]
        avg_sa = statistics.mean(late_sa) if late_sa else 0
        std_sa = statistics.stdev(late_sa) if len(late_sa) > 1 else 0
        conv = statistics.mean([m.convergence_pick for m in all_metrics])
        print(f"    primary_weight={primary_weight}: avg_S/A={avg_sa:.2f}, "
              f"std={std_sa:.2f}, conv={conv:.1f}")

    # ----- Draft traces -----
    print("\n" + "=" * 70)
    print("DRAFT TRACES")
    print("=" * 70)

    print("\n  Trace 1: Early committer (Warriors, committed from pick 3)")
    lines = run_trace(pool, "Warriors/Midrange", "committed", "pack_widening", seed=777)
    for l in lines:
        print(l)

    print("\n  Trace 2: Flexible player (power chaser, no commitment)")
    lines = run_trace(pool, "Warriors/Midrange", "power", "pack_widening", seed=888)
    for l in lines:
        print(l)

    print("\n  Trace 3: Signal reader")
    lines = run_trace(pool, "Storm/Spellslinger", "signal_reader", "pack_widening", seed=999)
    for l in lines:
        print(l)

    # ----- One-sentence verification -----
    print("\n" + "=" * 70)
    print("ONE-SENTENCE CLAIM VERIFICATION")
    print("=" * 70)
    print("""
  Algorithm: "Each symbol you draft earns 1 matching token (primary earns 2);
  before seeing a pack, you may spend 2 tokens of one resonance to add 2 extra
  cards with that primary resonance to the pack."

  Implementation check:
  - Token earning: earn() adds 2 for primary, 1 for secondary/tertiary. MATCH.
  - Spend decision: before pack generation, player chooses resonance. MATCH.
  - Spend cost: 2 tokens. MATCH.
  - Bonus cards: 2 extra cards with spend resonance as PRIMARY. MATCH.
  - The one-sentence description fully specifies the algorithm.
    """)

    print("\nSimulation complete.")

if __name__ == "__main__":
    main()
