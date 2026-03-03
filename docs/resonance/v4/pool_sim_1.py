#!/usr/bin/env python3
"""
Agent 1: Symbol Count Distribution Simulation

Investigates what ratio of 0/1/2/3-symbol cards produces the best draft
experience with Pack Widening v3. Tests 8 distributions from extreme to
balanced.

All metrics are evaluated at the ARCHETYPE level, not resonance level.
"""

import random
import statistics
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional

# ─── Constants ───────────────────────────────────────────────────────────────

NUM_DRAFTS = 2000
NUM_PICKS = 30
TOTAL_CARDS = 360
GENERIC_COUNT = 36  # ~10%
NON_GENERIC_COUNT = TOTAL_CARDS - GENERIC_COUNT

SPEND_COST = 3
BONUS_CARDS = 1
PRIMARY_WEIGHT = 2
SECONDARY_WEIGHT = 1
BASE_PACK_SIZE = 4
COMMITMENT_PICK = 5  # Commit after this many picks


# ─── Resonance & Archetype Definitions ───────────────────────────────────────

class Resonance(Enum):
    EMBER = "Ember"
    STONE = "Stone"
    TIDE = "Tide"
    ZEPHYR = "Zephyr"


# 8 archetypes on a circle, each with (primary_resonance, secondary_resonance)
ARCHETYPES = {
    "Flash":        (Resonance.ZEPHYR, Resonance.EMBER),
    "Blink":        (Resonance.EMBER,  Resonance.ZEPHYR),
    "Storm":        (Resonance.EMBER,  Resonance.STONE),
    "SelfDiscard":  (Resonance.STONE,  Resonance.EMBER),
    "SelfMill":     (Resonance.STONE,  Resonance.TIDE),
    "Sacrifice":    (Resonance.TIDE,   Resonance.STONE),
    "Warriors":     (Resonance.TIDE,   Resonance.ZEPHYR),
    "Ramp":         (Resonance.ZEPHYR, Resonance.TIDE),
}

ARCHETYPE_NAMES = list(ARCHETYPES.keys())
ARCHETYPE_INDEX = {name: i for i, name in enumerate(ARCHETYPE_NAMES)}


def get_adjacent(arch_name: str) -> list[str]:
    """Return the two adjacent archetypes on the circle."""
    idx = ARCHETYPE_INDEX[arch_name]
    n = len(ARCHETYPE_NAMES)
    return [ARCHETYPE_NAMES[(idx - 1) % n], ARCHETYPE_NAMES[(idx + 1) % n]]


def compute_fitness_tier(card_arch: Optional[str], card_symbols: list,
                         target_arch: str) -> str:
    """
    Compute fitness tier of a card for a target archetype.
    S = home archetype
    A = adjacent archetype sharing card's primary resonance
    B = shares secondary resonance, or generic
    C = distant
    """
    if card_arch is None:
        return "B"  # Generic -> B for all

    if card_arch == target_arch:
        return "S"

    adjacent = get_adjacent(target_arch)
    if card_arch in adjacent:
        # Adjacent archetype: A-tier if shares primary resonance with target
        target_primary = ARCHETYPES[target_arch][0]
        target_secondary = ARCHETYPES[target_arch][1]
        card_primary = ARCHETYPES[card_arch][0]

        # An adjacent card is A-tier if its primary resonance matches the
        # target's primary OR secondary resonance (meaning they share a
        # resonance that the card uses as primary)
        if card_symbols and (card_symbols[0] == target_primary or
                             card_symbols[0] == target_secondary):
            return "A"
        return "B"

    # Non-adjacent: check for any shared resonance
    target_resonances = {ARCHETYPES[target_arch][0], ARCHETYPES[target_arch][1]}
    if card_symbols:
        card_resonances = set(card_symbols)
        if card_resonances & target_resonances:
            return "B"

    return "C"


# ─── Card Model ──────────────────────────────────────────────────────────────

@dataclass
class SimCard:
    id: int
    symbols: list  # list[Resonance], ordered. [] = generic
    archetype: Optional[str]  # None for generic
    power: float  # 0-10

    @property
    def primary_resonance(self) -> Optional[Resonance]:
        return self.symbols[0] if self.symbols else None

    def tokens_earned(self) -> dict:
        tokens = {r: 0 for r in Resonance}
        if not self.symbols:
            return tokens
        tokens[self.symbols[0]] += PRIMARY_WEIGHT
        for sym in self.symbols[1:]:
            tokens[sym] += SECONDARY_WEIGHT
        return tokens

    def total_tokens(self) -> int:
        return sum(self.tokens_earned().values())

    def fitness_for(self, target_arch: str) -> str:
        return compute_fitness_tier(self.archetype, self.symbols, target_arch)

    def is_sa_for(self, target_arch: str) -> bool:
        return self.fitness_for(target_arch) in ("S", "A")


# ─── Pool Construction ───────────────────────────────────────────────────────

def build_pool(symbol_dist: dict) -> list[SimCard]:
    """
    Build a 360-card pool with given symbol distribution.
    symbol_dist: {1: frac, 2: frac, 3: frac} for non-generic cards (must sum to ~1.0).
    """
    cards = []
    card_id = 0

    # Generic cards (0 symbols)
    for _ in range(GENERIC_COUNT):
        cards.append(SimCard(id=card_id, symbols=[], archetype=None,
                             power=random.uniform(3, 7)))
        card_id += 1

    # Distribute non-generic cards equally across 8 archetypes
    base_per_arch = NON_GENERIC_COUNT // 8
    remainder = NON_GENERIC_COUNT % 8

    for arch_idx, arch_name in enumerate(ARCHETYPE_NAMES):
        count = base_per_arch + (1 if arch_idx < remainder else 0)
        primary_res, secondary_res = ARCHETYPES[arch_name]

        # Allocate symbol counts
        n1 = round(count * symbol_dist.get(1, 0))
        n3 = round(count * symbol_dist.get(3, 0))
        n2 = count - n1 - n3
        if n2 < 0:
            n1 = max(0, n1 + n2)
            n2 = count - n1 - n3
            if n2 < 0:
                n3 = max(0, n3 + n2)
                n2 = count - n1 - n3

        for _ in range(n1):
            cards.append(SimCard(id=card_id, symbols=[primary_res],
                                 archetype=arch_name, power=random.uniform(3, 8)))
            card_id += 1

        for _ in range(n2):
            cards.append(SimCard(id=card_id, symbols=[primary_res, secondary_res],
                                 archetype=arch_name, power=random.uniform(3, 8)))
            card_id += 1

        for _ in range(n3):
            # 3-symbol: [primary, secondary, secondary]
            cards.append(SimCard(id=card_id,
                                 symbols=[primary_res, secondary_res, secondary_res],
                                 archetype=arch_name, power=random.uniform(3, 8)))
            card_id += 1

    return cards


# ─── Draft Simulation ────────────────────────────────────────────────────────

@dataclass
class DraftResult:
    # Per-pick tracking (0-indexed corresponds to pick 1..30)
    sa_per_pick: list = field(default_factory=list)
    spent_on_pick: list = field(default_factory=list)  # bool per pick
    tokens_earned_pick: list = field(default_factory=list)
    primary_tokens_pick: list = field(default_factory=list)
    token_balance_pick: list = field(default_factory=list)  # primary token balance after each pick

    # Summary
    first_spend_pick: Optional[int] = None
    target_arch: Optional[str] = None
    drafted_cards: list = field(default_factory=list)

    # Early game
    early_unique_archetypes: list = field(default_factory=list)  # per early pick
    early_sa_for_target: list = field(default_factory=list)


def simulate_draft(pool: list[SimCard]) -> DraftResult:
    """Simulate a single 30-pick draft with an archetype-committed player."""
    result = DraftResult()
    tokens = {r: 0 for r in Resonance}
    drafted = []
    target_arch = None
    primary_res = None

    # Index for bonus draws
    primary_index = {r: [c for c in pool if c.primary_resonance == r] for r in Resonance}

    for pick_num in range(1, NUM_PICKS + 1):
        # Determine spending
        spent = False
        if target_arch is not None and primary_res is not None:
            if tokens[primary_res] >= SPEND_COST:
                spent = True
                tokens[primary_res] -= SPEND_COST
                if result.first_spend_pick is None:
                    result.first_spend_pick = pick_num

        # Draw base pack
        pack = random.sample(pool, BASE_PACK_SIZE)

        # Bonus card if spending
        if spent and primary_index[primary_res]:
            pack.append(random.choice(primary_index[primary_res]))

        # Measure pack quality
        if target_arch is not None:
            sa_count = sum(1 for c in pack if c.is_sa_for(target_arch))
        else:
            sa_count = 0

        result.sa_per_pick.append(sa_count)
        result.spent_on_pick.append(spent)

        # Early game diversity (picks 1-5): count unique archetypes with S/A in pack
        if pick_num <= COMMITMENT_PICK:
            unique_archs = set()
            for card in pack:
                for arch in ARCHETYPE_NAMES:
                    if card.is_sa_for(arch):
                        unique_archs.add(arch)
            result.early_unique_archetypes.append(len(unique_archs))

        # Pick a card
        if target_arch is None:
            # Pre-commitment: pick highest power
            chosen = max(pack, key=lambda c: c.power)
        else:
            # Post-commitment: pick best for target
            def score(card):
                tier = card.fitness_for(target_arch)
                tier_score = {"S": 4, "A": 3, "B": 2, "C": 1}[tier]
                return (tier_score, card.power)
            chosen = max(pack, key=score)

        drafted.append(chosen)

        # Earn tokens
        earned = chosen.tokens_earned()
        for r in Resonance:
            tokens[r] += earned[r]

        result.tokens_earned_pick.append(sum(earned.values()))
        if primary_res is not None:
            result.primary_tokens_pick.append(earned[primary_res])
            result.token_balance_pick.append(tokens[primary_res])
        else:
            result.primary_tokens_pick.append(0)
            result.token_balance_pick.append(0)

        # Commit at pick 5
        if pick_num == COMMITMENT_PICK and target_arch is None:
            arch_scores = {}
            for arch in ARCHETYPE_NAMES:
                arch_scores[arch] = sum(1 for c in drafted if c.is_sa_for(arch))
            best_arch = max(arch_scores, key=arch_scores.get)
            target_arch = best_arch
            primary_res = ARCHETYPES[best_arch][0]
            result.target_arch = target_arch

            # Retroactively measure early SA for the committed archetype
            # We need to re-simulate packs for this — instead, measure from
            # drafted cards
            result.early_sa_for_target = [
                1 if c.is_sa_for(target_arch) else 0 for c in drafted[:COMMITMENT_PICK]
            ]

    result.drafted_cards = drafted
    return result


# ─── Evaluation ──────────────────────────────────────────────────────────────

@dataclass
class DistributionMetrics:
    name: str
    symbol_dist: dict

    # Token economy (picks 6-30)
    avg_tokens_per_pick: float = 0.0
    avg_primary_tokens_per_pick: float = 0.0

    # Spend timing
    avg_first_spend_pick: float = 0.0
    pct_never_spend: float = 0.0  # fraction of drafts that never spend

    # Spend frequency (picks 6+)
    spend_freq_6plus: float = 0.0

    # Per-pick SA (for convergence curve)
    sa_by_pick: dict = field(default_factory=dict)  # pick -> mean SA

    # Spend vs non-spend
    sa_spend_packs: float = 0.0
    sa_non_spend_packs: float = 0.0

    # SA trend (picks 6-30 linear slope)
    sa_trend_slope: float = 0.0

    # Overall 6+ stats
    sa_6plus_mean: float = 0.0
    sa_6plus_stddev: float = 0.0

    # Deck quality
    deck_sa_fraction: float = 0.0

    # Early game
    early_unique_archetypes: float = 0.0

    # Token balance curve (average primary tokens after each pick)
    token_balance_curve: dict = field(default_factory=dict)  # pick -> mean balance

    # Consecutive non-spend streaks (picks 6+)
    avg_max_non_spend_streak: float = 0.0

    # Save/spend rhythm quality
    always_spend_pct: float = 0.0  # fraction of drafts where player spends every pick 6+


def evaluate_distribution(name: str, symbol_dist: dict) -> DistributionMetrics:
    """Run NUM_DRAFTS simulations and compute all metrics."""
    m = DistributionMetrics(name=name, symbol_dist=symbol_dist)

    # Accumulators
    sa_by_pick_all = {p: [] for p in range(1, NUM_PICKS + 1)}
    token_bal_by_pick = {p: [] for p in range(1, NUM_PICKS + 1)}
    tokens_6plus = []
    primary_tokens_6plus = []
    first_spends = []
    never_spend_count = 0
    spend_freq_per_draft = []
    spend_sa_all = []
    non_spend_sa_all = []
    sa_6plus_all = []
    deck_sa_fracs = []
    early_unique_all = []
    max_non_spend_streaks = []
    always_spend_count = 0

    for _ in range(NUM_DRAFTS):
        pool = build_pool(symbol_dist)
        draft = simulate_draft(pool)

        # Per-pick SA
        for i, sa in enumerate(draft.sa_per_pick):
            sa_by_pick_all[i + 1].append(sa)

        # Token balance curve
        for i, bal in enumerate(draft.token_balance_pick):
            token_bal_by_pick[i + 1].append(bal)

        # Token economy picks 6-30
        tokens_6plus.extend(draft.tokens_earned_pick[5:])
        primary_tokens_6plus.extend(draft.primary_tokens_pick[5:])

        # First spend
        if draft.first_spend_pick is not None:
            first_spends.append(draft.first_spend_pick)
        else:
            never_spend_count += 1

        # Spend frequency picks 6+
        spend_count = sum(1 for i in range(5, NUM_PICKS) if draft.spent_on_pick[i])
        total_6plus = NUM_PICKS - 5
        spend_freq_per_draft.append(spend_count / total_6plus)

        # Always-spend check
        if spend_count == total_6plus:
            always_spend_count += 1

        # Spend vs non-spend SA (picks 6+)
        for i in range(5, NUM_PICKS):
            sa = draft.sa_per_pick[i]
            if draft.spent_on_pick[i]:
                spend_sa_all.append(sa)
            else:
                non_spend_sa_all.append(sa)
            sa_6plus_all.append(sa)

        # Max non-spend streak (picks 6+)
        max_streak = 0
        current_streak = 0
        for i in range(5, NUM_PICKS):
            if not draft.spent_on_pick[i]:
                current_streak += 1
                max_streak = max(max_streak, current_streak)
            else:
                current_streak = 0
        max_non_spend_streaks.append(max_streak)

        # Deck SA fraction
        if draft.target_arch:
            sa_cards = sum(1 for c in draft.drafted_cards if c.is_sa_for(draft.target_arch))
            deck_sa_fracs.append(sa_cards / len(draft.drafted_cards))

        # Early unique archetypes
        if draft.early_unique_archetypes:
            early_unique_all.extend(draft.early_unique_archetypes)

    # Aggregate
    m.avg_tokens_per_pick = statistics.mean(tokens_6plus) if tokens_6plus else 0
    m.avg_primary_tokens_per_pick = statistics.mean(primary_tokens_6plus) if primary_tokens_6plus else 0

    m.avg_first_spend_pick = statistics.mean(first_spends) if first_spends else float('inf')
    m.pct_never_spend = never_spend_count / NUM_DRAFTS

    m.spend_freq_6plus = statistics.mean(spend_freq_per_draft)
    m.always_spend_pct = always_spend_count / NUM_DRAFTS

    for p in range(1, NUM_PICKS + 1):
        m.sa_by_pick[p] = statistics.mean(sa_by_pick_all[p])
    for p in range(1, NUM_PICKS + 1):
        m.token_balance_curve[p] = statistics.mean(token_bal_by_pick[p])

    m.sa_spend_packs = statistics.mean(spend_sa_all) if spend_sa_all else 0
    m.sa_non_spend_packs = statistics.mean(non_spend_sa_all) if non_spend_sa_all else 0

    m.sa_6plus_mean = statistics.mean(sa_6plus_all) if sa_6plus_all else 0
    m.sa_6plus_stddev = statistics.stdev(sa_6plus_all) if len(sa_6plus_all) > 1 else 0

    # SA trend: linear regression picks 6-30
    x_vals = list(range(6, 31))
    y_vals = [m.sa_by_pick[p] for p in x_vals]
    x_mean = statistics.mean(x_vals)
    y_mean = statistics.mean(y_vals)
    num = sum((x_vals[i] - x_mean) * (y_vals[i] - y_mean) for i in range(len(x_vals)))
    den = sum((x_vals[i] - x_mean) ** 2 for i in range(len(x_vals)))
    m.sa_trend_slope = num / den if den != 0 else 0

    m.deck_sa_fraction = statistics.mean(deck_sa_fracs) if deck_sa_fracs else 0
    m.early_unique_archetypes = statistics.mean(early_unique_all) if early_unique_all else 0
    m.avg_max_non_spend_streak = statistics.mean(max_non_spend_streaks) if max_non_spend_streaks else 0

    return m


# ─── Configurations ──────────────────────────────────────────────────────────

DISTRIBUTIONS = {
    "All 1-sym (100/0/0)":          {1: 1.00, 2: 0.00, 3: 0.00},
    "Heavy 1-sym (70/20/10)":       {1: 0.70, 2: 0.20, 3: 0.10},
    "Moderate 1-sym (50/35/15)":    {1: 0.50, 2: 0.35, 3: 0.15},
    "Balanced (33/34/33)":          {1: 0.33, 2: 0.34, 3: 0.33},
    "V4 Default (20/55/25)":        {1: 0.20, 2: 0.55, 3: 0.25},
    "Heavy 2-sym (10/80/10)":       {1: 0.10, 2: 0.80, 3: 0.10},
    "Heavy 3-sym (10/30/60)":       {1: 0.10, 2: 0.30, 3: 0.60},
    "All 3-sym (0/0/100)":          {1: 0.00, 2: 0.00, 3: 1.00},
}


# ─── Main ────────────────────────────────────────────────────────────────────

def main():
    random.seed(42)

    results = []
    for name, dist in DISTRIBUTIONS.items():
        print(f"Simulating: {name} ...", flush=True)
        r = evaluate_distribution(name, dist)
        results.append(r)

    print("\n" + "=" * 130)
    print("SYMBOL COUNT DISTRIBUTION ANALYSIS — Pack Widening v3")
    print(f"({NUM_DRAFTS} drafts per configuration, {NUM_PICKS} picks per draft)")
    print("=" * 130)

    # ─── Table 1: Token Economy ───────────────────────────────────────────
    print("\n### Table 1: Token Economy (picks 6-30, committed player)")
    print(f"{'Distribution':<30} {'Tok/Pick':>9} {'PrimTok':>8} {'1stSpend':>9} "
          f"{'SpendFrq':>9} {'AlwaysSpd':>10} {'MaxNoSpd':>9}")
    print("-" * 90)
    for r in results:
        first_sp = f"{r.avg_first_spend_pick:.1f}" if r.avg_first_spend_pick < 99 else "never"
        print(f"{r.name:<30} {r.avg_tokens_per_pick:>9.2f} {r.avg_primary_tokens_per_pick:>8.2f} "
              f"{first_sp:>9} {r.spend_freq_6plus:>9.1%} "
              f"{r.always_spend_pct:>10.1%} {r.avg_max_non_spend_streak:>9.1f}")

    # ─── Table 2: Convergence Curve ───────────────────────────────────────
    print("\n### Table 2: S/A Cards Per Pack — Convergence Curve")
    picks_show = [6, 8, 10, 12, 15, 18, 20, 25, 30]
    header = f"{'Distribution':<30}" + "".join(f"{'P'+str(p):>7}" for p in picks_show)
    print(header)
    print("-" * (30 + 7 * len(picks_show)))
    for r in results:
        row = f"{r.name:<30}"
        for p in picks_show:
            row += f"{r.sa_by_pick[p]:>7.2f}"
        print(row)

    # ─── Table 3: Spend vs Non-Spend & Trend ─────────────────────────────
    print("\n### Table 3: Spend vs Non-Spend Pack Quality")
    print(f"{'Distribution':<30} {'SA(Spend)':>10} {'SA(NoSpd)':>10} {'Gap':>7} "
          f"{'SA6+Mean':>9} {'SA6+Std':>8} {'Trend/Pk':>9}")
    print("-" * 87)
    for r in results:
        gap = r.sa_spend_packs - r.sa_non_spend_packs
        print(f"{r.name:<30} {r.sa_spend_packs:>10.2f} {r.sa_non_spend_packs:>10.2f} "
              f"{gap:>7.2f} {r.sa_6plus_mean:>9.2f} {r.sa_6plus_stddev:>8.2f} "
              f"{r.sa_trend_slope:>9.4f}")

    # ─── Table 4: Deck Composition & Early Game ──────────────────────────
    print("\n### Table 4: Deck Quality & Early Diversity")
    print(f"{'Distribution':<30} {'DeckSA%':>8} {'EarlyUniqueArch':>16}")
    print("-" * 56)
    for r in results:
        print(f"{r.name:<30} {r.deck_sa_fraction:>8.1%} {r.early_unique_archetypes:>16.1f}")

    # ─── Table 5: Token Balance Curve ─────────────────────────────────────
    print("\n### Table 5: Average Primary Token Balance After Pick")
    tok_picks = [5, 6, 7, 8, 10, 15, 20, 25, 30]
    header = f"{'Distribution':<30}" + "".join(f"{'P'+str(p):>7}" for p in tok_picks)
    print(header)
    print("-" * (30 + 7 * len(tok_picks)))
    for r in results:
        row = f"{r.name:<30}"
        for p in tok_picks:
            row += f"{r.token_balance_curve[p]:>7.1f}"
        print(row)

    # ─── Summary Assessment ───────────────────────────────────────────────
    print("\n### Summary Assessment")
    print(f"{'Distribution':<30} {'SpendRhythm':>12} {'Convergence':>12} {'Variance':>9} {'Overall':>10}")
    print("-" * 75)
    for r in results:
        # Spend rhythm: best if 30-60% (genuine save/spend decisions)
        if r.spend_freq_6plus < 0.30:
            rhythm = "Too slow"
        elif r.spend_freq_6plus <= 0.55:
            rhythm = "Good rhythm"
        elif r.spend_freq_6plus <= 0.65:
            rhythm = "Fast"
        elif r.spend_freq_6plus <= 0.75:
            rhythm = "Very fast"
        else:
            rhythm = "Always-spend"

        # Convergence
        if r.sa_6plus_mean < 1.5:
            conv = "Weak"
        elif r.sa_6plus_mean < 1.8:
            conv = "Low"
        elif r.sa_6plus_mean < 2.0:
            conv = "Good"
        elif r.sa_6plus_mean < 2.3:
            conv = "Strong"
        else:
            conv = "On rails"

        # Variance
        var_ok = "Good" if r.sa_6plus_stddev >= 0.8 else "Low"

        # Overall
        score = 0
        if 0.35 <= r.spend_freq_6plus <= 0.60:
            score += 3
        elif 0.30 <= r.spend_freq_6plus <= 0.65:
            score += 2
        elif 0.25 <= r.spend_freq_6plus <= 0.70:
            score += 1

        if 1.8 <= r.sa_6plus_mean <= 2.2:
            score += 2
        elif 1.5 <= r.sa_6plus_mean <= 2.5:
            score += 1

        if r.sa_6plus_stddev >= 0.8:
            score += 1

        if r.sa_trend_slope > -0.015:
            score += 1

        if r.always_spend_pct < 0.05:
            score += 1

        labels = {0: "Poor", 1: "Fair", 2: "Fair+", 3: "OK", 4: "Good",
                  5: "Great", 6: "Excellent", 7: "Excellent", 8: "Perfect"}
        overall = labels.get(score, "Good")

        print(f"{r.name:<30} {rhythm:>12} {conv:>12} {var_ok:>9} {overall:>10}")

    # ─── Key Finding: Save/Spend Rhythm ───────────────────────────────────
    print("\n### Key Finding: Save/Spend Rhythm Analysis")
    print("How many picks (out of 25 post-commitment) involve spending vs saving?")
    print(f"{'Distribution':<30} {'Spend Picks':>12} {'Save Picks':>12} {'Ratio':>8}")
    print("-" * 64)
    for r in results:
        spend = r.spend_freq_6plus * 25
        save = 25 - spend
        ratio = f"{spend:.0f}:{save:.0f}"
        print(f"{r.name:<30} {spend:>12.1f} {save:>12.1f} {ratio:>8}")


if __name__ == "__main__":
    main()
