#!/usr/bin/env python3
"""
Pool Simulation 5: Pack Widening v3 Algorithm Parameter Tuning

Investigates how Pack Widening v3's parameters should be tuned relative to the
pool design, and what is the ideal progression curve.

Parameter matrix:
- Spend cost: 2, 3, 4, 5
- Bonus card count: 1, 2
- Primary token weight: 1, 2, 3

Crossed with 3 symbol distributions:
- Heavy 1-symbol (50/35/15)
- Default (20/55/25)
- Heavy 3-symbol (10/30/60)

Measures per configuration:
- First spend pick timing
- Spend frequency at picks 6+
- Always-spend degeneracy detection
- S/A per pack convergence curve
- Full measurable targets table
- Three-act draft arc quality
- Decision quality / meaningful choice frequency
"""

import random
import statistics
from dataclasses import dataclass, field
from enum import Enum
from collections import defaultdict
from typing import Optional


# ---------------------------------------------------------------------------
# Core types
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

ARCHETYPES = [
    ("Flash",        Resonance.ZEPHYR, Resonance.EMBER),   # 0
    ("Blink",        Resonance.EMBER,  Resonance.ZEPHYR),  # 1
    ("Storm",        Resonance.EMBER,  Resonance.STONE),   # 2
    ("Self-Discard", Resonance.STONE,  Resonance.EMBER),   # 3
    ("Self-Mill",    Resonance.STONE,  Resonance.TIDE),    # 4
    ("Sacrifice",    Resonance.TIDE,   Resonance.STONE),   # 5
    ("Warriors",     Resonance.TIDE,   Resonance.ZEPHYR),  # 6
    ("Ramp",         Resonance.ZEPHYR, Resonance.TIDE),    # 7
]

ARCHETYPE_NAMES = [a[0] for a in ARCHETYPES]
NUM_ARCHETYPES = 8
NUM_PICKS = 30
BASE_PACK_SIZE = 4
TOTAL_CARDS = 360
NUM_GENERIC = 36


def circle_distance(i: int, j: int) -> int:
    d = abs(i - j)
    return min(d, NUM_ARCHETYPES - d)


def compute_fitness(card_arch_idx: int, player_arch_idx: int) -> Tier:
    """Compute archetype-level fitness tier for a card relative to a player's archetype."""
    if card_arch_idx < 0:
        return Tier.B  # generic
    if card_arch_idx == player_arch_idx:
        return Tier.S
    dist = circle_distance(card_arch_idx, player_arch_idx)
    card_primary = ARCHETYPES[card_arch_idx][1]
    player_primary = ARCHETYPES[player_arch_idx][1]
    if dist == 1:
        if card_primary == player_primary:
            return Tier.A
        return Tier.B
    elif dist == 2:
        card_secondary = ARCHETYPES[card_arch_idx][2]
        player_secondary = ARCHETYPES[player_arch_idx][2]
        card_res = {card_primary, card_secondary}
        player_res = {player_primary, player_secondary}
        if card_res & player_res:
            return Tier.B
        return Tier.C
    elif dist == 3:
        return Tier.C
    else:
        return Tier.F


@dataclass
class SimCard:
    id: int
    symbols: list  # list of Resonance, ordered
    archetype_idx: int  # -1 for generic
    fitness: dict = field(default_factory=dict)  # arch_idx -> Tier

    @property
    def primary_resonance(self) -> Optional[Resonance]:
        return self.symbols[0] if self.symbols else None


def is_sa(card: SimCard, arch_idx: int) -> bool:
    return card.fitness.get(arch_idx, Tier.F) in (Tier.S, Tier.A)


def is_cf(card: SimCard, arch_idx: int) -> bool:
    return card.fitness.get(arch_idx, Tier.F) in (Tier.C, Tier.F)


# ---------------------------------------------------------------------------
# Pool generation
# ---------------------------------------------------------------------------

def generate_pool(pct_1sym: float, pct_2sym: float, pct_3sym: float) -> list:
    cards = []
    card_id = 0

    # Generic cards
    for _ in range(NUM_GENERIC):
        card = SimCard(id=card_id, symbols=[], archetype_idx=-1)
        for j in range(NUM_ARCHETYPES):
            card.fitness[j] = Tier.B
        cards.append(card)
        card_id += 1

    non_generic = TOTAL_CARDS - NUM_GENERIC
    per_archetype = non_generic // NUM_ARCHETYPES

    for arch_idx in range(NUM_ARCHETYPES):
        _, primary, secondary = ARCHETYPES[arch_idx]
        n1 = round(per_archetype * pct_1sym / 100)
        n3 = round(per_archetype * pct_3sym / 100)
        n2 = per_archetype - n1 - n3

        # 1-symbol cards: [Primary]
        for _ in range(n1):
            card = SimCard(id=card_id, symbols=[primary], archetype_idx=arch_idx)
            cards.append(card)
            card_id += 1

        # 2-symbol cards: mix of [P,S] and [P,P]
        for i in range(n2):
            if i % 3 == 0:
                syms = [primary, primary]
            else:
                syms = [primary, secondary]
            card = SimCard(id=card_id, symbols=syms, archetype_idx=arch_idx)
            cards.append(card)
            card_id += 1

        # 3-symbol cards: various patterns
        for i in range(n3):
            if i % 3 == 0:
                syms = [primary, primary, secondary]
            elif i % 3 == 1:
                syms = [primary, secondary, secondary]
            else:
                syms = [primary, secondary, primary]
            card = SimCard(id=card_id, symbols=syms, archetype_idx=arch_idx)
            cards.append(card)
            card_id += 1

    # Compute fitness for all non-generic cards
    for card in cards:
        if card.archetype_idx >= 0:
            for j in range(NUM_ARCHETYPES):
                card.fitness[j] = compute_fitness(card.archetype_idx, j)

    return cards


def build_primary_index(pool: list) -> dict:
    """Build index of cards by primary resonance for bonus card draws."""
    by_primary = defaultdict(list)
    for card in pool:
        if card.primary_resonance is not None:
            by_primary[card.primary_resonance].append(card)
    return dict(by_primary)


# ---------------------------------------------------------------------------
# Pack Widening v3 Algorithm
# ---------------------------------------------------------------------------

@dataclass
class DraftState:
    tokens: dict = field(default_factory=lambda: {r: 0 for r in Resonance})
    picks: list = field(default_factory=list)
    target_archetype: int = -1

    # Algorithm parameters
    spend_cost: int = 3
    bonus_cards: int = 1
    primary_weight: int = 2

    # Tracking
    spend_picks: list = field(default_factory=list)  # pick numbers where spending occurred
    spend_resonances: list = field(default_factory=list)  # which resonance was spent
    token_history: list = field(default_factory=list)  # token snapshot after each pick


def earn_tokens(state: DraftState, card: SimCard):
    """Add tokens based on the picked card's symbols."""
    for i, sym in enumerate(card.symbols):
        if i == 0:
            state.tokens[sym] += state.primary_weight
        else:
            state.tokens[sym] += 1


def can_spend(state: DraftState) -> list:
    """Return list of resonances the player can spend on."""
    return [r for r in Resonance if state.tokens[r] >= state.spend_cost]


def generate_pack(state: DraftState, pool: list, primary_index: dict,
                  pick_num: int, strategy: str = "committed") -> tuple:
    """Generate a pack. Returns (pack, did_spend, spent_resonance)."""
    spendable = can_spend(state)
    did_spend = False
    spent_resonance = None

    if spendable and strategy == "committed":
        # Committed player spends on their archetype's primary resonance if possible
        target_primary = ARCHETYPES[state.target_archetype][1]
        if target_primary in spendable:
            spent_resonance = target_primary
        else:
            # Spend on whatever is available (secondary resonance or first available)
            target_secondary = ARCHETYPES[state.target_archetype][2]
            if target_secondary in spendable:
                spent_resonance = target_secondary
            else:
                spent_resonance = spendable[0]
        state.tokens[spent_resonance] -= state.spend_cost
        did_spend = True
    elif spendable and strategy == "always_spend":
        # Always spend on best available resonance (primary first)
        target_primary = ARCHETYPES[state.target_archetype][1]
        if target_primary in spendable:
            spent_resonance = target_primary
        else:
            spent_resonance = spendable[0]
        state.tokens[spent_resonance] -= state.spend_cost
        did_spend = True
    elif spendable and strategy == "save_then_spend":
        # Only spend if tokens >= cost + 2 (saving buffer)
        threshold = state.spend_cost + 2
        high_spendable = [r for r in Resonance if state.tokens[r] >= threshold]
        if high_spendable:
            target_primary = ARCHETYPES[state.target_archetype][1]
            if target_primary in high_spendable:
                spent_resonance = target_primary
            else:
                spent_resonance = high_spendable[0]
            state.tokens[spent_resonance] -= state.spend_cost
            did_spend = True

    # Draw base pack
    pack = [random.choice(pool) for _ in range(BASE_PACK_SIZE)]

    # Draw bonus cards if spending
    if did_spend and spent_resonance in primary_index:
        candidates = primary_index[spent_resonance]
        for _ in range(state.bonus_cards):
            pack.append(random.choice(candidates))

    return pack, did_spend, spent_resonance


def pick_card_committed(pack: list, state: DraftState, pick_num: int) -> SimCard:
    """Committed player picks best card for their archetype."""
    arch = state.target_archetype
    if pick_num <= 2 and arch == -1:
        return random.choice(pack)
    tier_priority = [Tier.S, Tier.A, Tier.B, Tier.C, Tier.F]
    for tier in tier_priority:
        candidates = [c for c in pack if c.fitness.get(arch, Tier.F) == tier]
        if candidates:
            return random.choice(candidates)
    return random.choice(pack)


# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

@dataclass
class Config:
    spend_cost: int
    bonus_cards: int
    primary_weight: int
    pct_1sym: float
    pct_2sym: float
    pct_3sym: float

    @property
    def dist_label(self) -> str:
        return f"{int(self.pct_1sym)}/{int(self.pct_2sym)}/{int(self.pct_3sym)}"

    @property
    def name(self) -> str:
        return f"C{self.spend_cost} B{self.bonus_cards} W{self.primary_weight} D{self.dist_label}"


# ---------------------------------------------------------------------------
# Metrics
# ---------------------------------------------------------------------------

@dataclass
class Metrics:
    config: Config
    # Spend timing
    first_spend_avg: float = 0.0
    pct_never_spend: float = 0.0
    spend_frequency_late: float = 0.0  # fraction of picks 6+ that involve spending

    # Always-spend degeneracy
    always_spend_sa: float = 0.0  # avg S/A per deck with always-spend
    committed_sa: float = 0.0    # avg S/A per deck with committed strategy
    save_spend_sa: float = 0.0   # avg S/A per deck with save-then-spend
    always_spend_dominant: bool = False

    # Convergence curve
    sa_curve: dict = field(default_factory=dict)  # pick -> avg S/A per pack
    sa_stddev_late: float = 0.0  # stddev of S/A per pack at picks 6+

    # Measurable targets
    early_unique_archs: float = 0.0  # picks 1-5: unique archetypes with S/A per pack
    early_sa_emerging: float = 0.0   # picks 1-5: S/A for emerging archetype per pack
    late_sa: float = 0.0             # picks 6+: avg S/A for committed archetype
    late_cf: float = 0.0             # picks 6+: avg off-archetype C/F per pack
    convergence_pick: int = 31       # first pick where S/A avg >= 2.0
    deck_sa_pct: float = 0.0        # deck archetype concentration
    card_overlap: float = 0.0       # run-to-run variety
    arch_freq_max: float = 0.0
    arch_freq_min: float = 0.0

    # Decision quality
    decision_quality: dict = field(default_factory=dict)  # pick -> % with 2+ S/A
    meaningful_choice_late: float = 0.0  # picks 6+: % of packs with 2+ S/A

    # Three-act arc
    act1_description: str = ""  # exploration phase
    act2_description: str = ""  # commitment phase
    act3_description: str = ""  # refinement phase
    arc_rating: str = ""


# ---------------------------------------------------------------------------
# Simulation
# ---------------------------------------------------------------------------

def run_single_draft(pool: list, primary_index: dict, config: Config,
                     target_arch: int, strategy: str = "committed"):
    """Run one complete 30-pick draft. Returns detailed per-pick data."""
    state = DraftState()
    state.target_archetype = target_arch
    state.spend_cost = config.spend_cost
    state.bonus_cards = config.bonus_cards
    state.primary_weight = config.primary_weight

    pick_data = []

    for pick_num in range(1, NUM_PICKS + 1):
        pack, did_spend, spent_res = generate_pack(
            state, pool, primary_index, pick_num, strategy=strategy
        )

        sa_count = sum(1 for c in pack if is_sa(c, target_arch))
        cf_count = sum(1 for c in pack if is_cf(c, target_arch))

        # Unique archetypes with S/A in this pack
        unique_sa_archs = set()
        for card in pack:
            for arch_idx in range(NUM_ARCHETYPES):
                if is_sa(card, arch_idx):
                    unique_sa_archs.add(arch_idx)

        card = pick_card_committed(pack, state, pick_num)
        earn_tokens(state, card)

        if did_spend:
            state.spend_picks.append(pick_num)
            state.spend_resonances.append(spent_res)

        state.picks.append(card)
        state.token_history.append(dict(state.tokens))

        pick_data.append({
            "pick_num": pick_num,
            "pack_size": len(pack),
            "sa_count": sa_count,
            "cf_count": cf_count,
            "unique_sa_archs": len(unique_sa_archs),
            "did_spend": did_spend,
            "card": card,
        })

    # Deck S/A concentration
    deck_sa = sum(1 for c in state.picks if is_sa(c, target_arch))
    deck_sa_pct = deck_sa / NUM_PICKS

    return {
        "pick_data": pick_data,
        "state": state,
        "deck_sa_pct": deck_sa_pct,
        "first_spend": state.spend_picks[0] if state.spend_picks else None,
        "total_spends": len(state.spend_picks),
        "drafted_ids": [c.id for c in state.picks],
    }


def run_config(config: Config, num_drafts: int = 1000) -> Metrics:
    pool = generate_pool(config.pct_1sym, config.pct_2sym, config.pct_3sym)
    primary_index = build_primary_index(pool)
    metrics = Metrics(config=config)

    # --- Committed strategy (main analysis) ---
    first_spends = []
    pick_sa = defaultdict(list)
    pick_cf = defaultdict(list)
    pick_unique_archs = defaultdict(list)
    pick_dq = defaultdict(list)
    deck_sa_pcts = []
    late_spend_counts = []
    late_pick_counts = []
    all_drafted_ids = []
    arch_chosen = defaultdict(int)

    for _ in range(num_drafts):
        target_arch = random.randint(0, NUM_ARCHETYPES - 1)
        arch_chosen[target_arch] += 1
        result = run_single_draft(pool, primary_index, config, target_arch, "committed")

        if result["first_spend"] is not None:
            first_spends.append(result["first_spend"])

        deck_sa_pcts.append(result["deck_sa_pct"])
        all_drafted_ids.append(set(result["drafted_ids"]))

        late_spends = sum(1 for p in result["state"].spend_picks if p >= 6)
        late_picks = NUM_PICKS - 5  # picks 6 through 30
        late_spend_counts.append(late_spends)
        late_pick_counts.append(late_picks)

        for pd in result["pick_data"]:
            pn = pd["pick_num"]
            pick_sa[pn].append(pd["sa_count"])
            pick_cf[pn].append(pd["cf_count"])
            pick_unique_archs[pn].append(pd["unique_sa_archs"])
            pick_dq[pn].append(1 if pd["sa_count"] >= 2 else 0)

    # Aggregate committed metrics
    metrics.first_spend_avg = statistics.mean(first_spends) if first_spends else 31
    metrics.pct_never_spend = (1 - len(first_spends) / num_drafts) * 100
    metrics.spend_frequency_late = (
        sum(late_spend_counts) / sum(late_pick_counts) if sum(late_pick_counts) > 0 else 0
    )

    # Convergence curve
    for pn in range(1, NUM_PICKS + 1):
        metrics.sa_curve[pn] = statistics.mean(pick_sa[pn])
        metrics.decision_quality[pn] = statistics.mean(pick_dq[pn]) * 100

    # S/A stddev for picks 6+
    late_sa_all = []
    for pn in range(6, NUM_PICKS + 1):
        late_sa_all.extend(pick_sa[pn])
    metrics.sa_stddev_late = statistics.stdev(late_sa_all) if len(late_sa_all) > 1 else 0

    # Early metrics (picks 1-5)
    early_unique = []
    early_sa_vals = []
    for pn in range(1, 6):
        early_unique.extend(pick_unique_archs[pn])
        early_sa_vals.extend(pick_sa[pn])
    metrics.early_unique_archs = statistics.mean(early_unique)
    metrics.early_sa_emerging = statistics.mean(early_sa_vals)

    # Late metrics (picks 6+)
    late_sa_vals = []
    late_cf_vals = []
    late_dq_vals = []
    for pn in range(6, NUM_PICKS + 1):
        late_sa_vals.extend(pick_sa[pn])
        late_cf_vals.extend(pick_cf[pn])
        late_dq_vals.extend(pick_dq[pn])
    metrics.late_sa = statistics.mean(late_sa_vals)
    metrics.late_cf = statistics.mean(late_cf_vals)
    metrics.meaningful_choice_late = statistics.mean(late_dq_vals) * 100

    # Convergence pick
    for pn in range(1, NUM_PICKS + 1):
        if metrics.sa_curve.get(pn, 0) >= 2.0:
            metrics.convergence_pick = pn
            break

    # Deck concentration
    metrics.deck_sa_pct = statistics.mean(deck_sa_pcts) * 100

    # Run-to-run variety (card overlap between random pairs)
    overlaps = []
    sample_size = min(500, len(all_drafted_ids))
    ids_sample = random.sample(all_drafted_ids, sample_size)
    for i in range(0, sample_size - 1, 2):
        s1 = ids_sample[i]
        s2 = ids_sample[i + 1]
        if len(s1 | s2) > 0:
            overlaps.append(len(s1 & s2) / len(s1 | s2))
    metrics.card_overlap = statistics.mean(overlaps) * 100 if overlaps else 0

    # Archetype frequency
    total_chosen = sum(arch_chosen.values())
    arch_freqs = [arch_chosen[i] / total_chosen * 100 for i in range(NUM_ARCHETYPES)]
    metrics.arch_freq_max = max(arch_freqs)
    metrics.arch_freq_min = min(arch_freqs)

    # --- Always-spend strategy comparison ---
    always_sa_pcts = []
    for _ in range(min(500, num_drafts)):
        target_arch = random.randint(0, NUM_ARCHETYPES - 1)
        result = run_single_draft(pool, primary_index, config, target_arch, "always_spend")
        always_sa_pcts.append(result["deck_sa_pct"])
    metrics.always_spend_sa = statistics.mean(always_sa_pcts) * 100

    # --- Save-then-spend strategy comparison ---
    save_sa_pcts = []
    for _ in range(min(500, num_drafts)):
        target_arch = random.randint(0, NUM_ARCHETYPES - 1)
        result = run_single_draft(pool, primary_index, config, target_arch, "save_then_spend")
        save_sa_pcts.append(result["deck_sa_pct"])
    metrics.save_spend_sa = statistics.mean(save_sa_pcts) * 100

    metrics.committed_sa = metrics.deck_sa_pct
    metrics.always_spend_dominant = (
        metrics.always_spend_sa >= metrics.committed_sa
        and metrics.always_spend_sa >= metrics.save_spend_sa
    )

    # --- Three-act arc analysis ---
    # Act 1: Exploration (picks 1-5)
    act1_sa = statistics.mean([metrics.sa_curve.get(p, 0) for p in range(1, 6)])
    act1_spend_rate = metrics.spend_frequency_late  # approximate
    if metrics.first_spend_avg > 5:
        metrics.act1_description = f"Pure exploration (no spending, SA avg {act1_sa:.1f})"
    elif metrics.first_spend_avg > 3:
        metrics.act1_description = f"Mostly exploration (1st spend ~pick {metrics.first_spend_avg:.0f}, SA avg {act1_sa:.1f})"
    else:
        metrics.act1_description = f"Premature spending (1st spend ~pick {metrics.first_spend_avg:.0f}, SA avg {act1_sa:.1f})"

    # Act 2: Commitment (picks 6-15)
    act2_sa = statistics.mean([metrics.sa_curve.get(p, 0) for p in range(6, 16)])
    act2_dq = statistics.mean([metrics.decision_quality.get(p, 0) for p in range(6, 16)])
    if act2_sa >= 2.0 and act2_dq >= 40:
        metrics.act2_description = f"Strong commitment (SA avg {act2_sa:.1f}, DQ {act2_dq:.0f}%)"
    elif act2_sa >= 1.5:
        metrics.act2_description = f"Moderate commitment (SA avg {act2_sa:.1f}, DQ {act2_dq:.0f}%)"
    else:
        metrics.act2_description = f"Weak commitment (SA avg {act2_sa:.1f}, DQ {act2_dq:.0f}%)"

    # Act 3: Refinement (picks 16-30)
    act3_sa = statistics.mean([metrics.sa_curve.get(p, 0) for p in range(16, 31)])
    act3_dq = statistics.mean([metrics.decision_quality.get(p, 0) for p in range(16, 31)])
    if act3_sa >= 2.5:
        metrics.act3_description = f"Deep refinement (SA avg {act3_sa:.1f}, DQ {act3_dq:.0f}%)"
    elif act3_sa >= 2.0:
        metrics.act3_description = f"Steady refinement (SA avg {act3_sa:.1f}, DQ {act3_dq:.0f}%)"
    else:
        metrics.act3_description = f"Stalled refinement (SA avg {act3_sa:.1f}, DQ {act3_dq:.0f}%)"

    # Arc rating
    arc_score = 0
    # Good exploration: first spend after pick 3
    if metrics.first_spend_avg >= 3.0:
        arc_score += 2
    elif metrics.first_spend_avg >= 2.0:
        arc_score += 1

    # Good commitment: convergence by pick 5-8
    if 5 <= metrics.convergence_pick <= 8:
        arc_score += 3
    elif 4 <= metrics.convergence_pick <= 10:
        arc_score += 2
    elif metrics.convergence_pick <= 12:
        arc_score += 1

    # Good spend rhythm: not always-spend-dominant
    if not metrics.always_spend_dominant:
        arc_score += 2
    if 0.3 <= metrics.spend_frequency_late <= 0.7:
        arc_score += 2  # balanced spend frequency
    elif 0.2 <= metrics.spend_frequency_late <= 0.8:
        arc_score += 1

    # Good late quality
    if metrics.late_sa >= 2.0:
        arc_score += 1
    if metrics.sa_stddev_late >= 0.8:
        arc_score += 1

    if arc_score >= 9:
        metrics.arc_rating = "EXCELLENT"
    elif arc_score >= 7:
        metrics.arc_rating = "GOOD"
    elif arc_score >= 5:
        metrics.arc_rating = "FAIR"
    else:
        metrics.arc_rating = "POOR"

    return metrics


# ---------------------------------------------------------------------------
# Build all configurations
# ---------------------------------------------------------------------------

def build_configs() -> list:
    configs = []
    spend_costs = [2, 3, 4, 5]
    bonus_cards_options = [1, 2]
    primary_weights = [1, 2, 3]
    distributions = [
        (50, 35, 15),   # Heavy 1-symbol
        (20, 55, 25),   # Default
        (10, 30, 60),   # Heavy 3-symbol
    ]

    for sc in spend_costs:
        for bc in bonus_cards_options:
            for pw in primary_weights:
                for d1, d2, d3 in distributions:
                    configs.append(Config(
                        spend_cost=sc,
                        bonus_cards=bc,
                        primary_weight=pw,
                        pct_1sym=d1,
                        pct_2sym=d2,
                        pct_3sym=d3,
                    ))

    return configs


# ---------------------------------------------------------------------------
# Output formatting
# ---------------------------------------------------------------------------

def print_section(title: str, width: int = 120):
    print(f"\n{'=' * width}")
    print(title)
    print(f"{'=' * width}")


def print_main_table(results: list, label: str = ""):
    if label:
        print(f"\n--- {label} ---")
    header = (f"{'Config':<28} {'1stSp':>5} {'Nv%':>4} {'SpFr':>5} "
              f"{'SA@5':>5} {'SA@10':>5} {'SA@15':>5} {'SA@20':>5} {'SA@30':>5} "
              f"{'Late':>5} {'SDev':>5} {'Conv':>4} "
              f"{'DkSA':>5} {'DQ%':>4} {'CF':>4} "
              f"{'AlwS':>5} {'SavS':>5} {'Deg':>3} {'Arc':>9}")
    print(header)
    print("-" * len(header))
    for m in results:
        deg = "Y" if m.always_spend_dominant else "N"
        print(f"{m.config.name:<28} "
              f"{m.first_spend_avg:>5.1f} "
              f"{m.pct_never_spend:>3.0f}% "
              f"{m.spend_frequency_late:>5.2f} "
              f"{m.sa_curve.get(5, 0):>5.2f} "
              f"{m.sa_curve.get(10, 0):>5.2f} "
              f"{m.sa_curve.get(15, 0):>5.2f} "
              f"{m.sa_curve.get(20, 0):>5.2f} "
              f"{m.sa_curve.get(30, 0):>5.2f} "
              f"{m.late_sa:>5.2f} "
              f"{m.sa_stddev_late:>5.2f} "
              f"{m.convergence_pick:>4} "
              f"{m.deck_sa_pct:>4.0f}% "
              f"{m.meaningful_choice_late:>3.0f}% "
              f"{m.late_cf:>4.1f} "
              f"{m.always_spend_sa:>4.0f}% "
              f"{m.save_spend_sa:>4.0f}% "
              f"{deg:>3} "
              f"{m.arc_rating:>9}")


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    random.seed(42)

    configs = build_configs()
    print(f"Total configurations: {len(configs)}")

    all_results = []
    for i, config in enumerate(configs):
        if (i + 1) % 10 == 0:
            print(f"  Running config {i+1}/{len(configs)}: {config.name}")
        m = run_config(config, num_drafts=1000)
        all_results.append(m)

    # ==================================================================
    # SECTION 1: Full results table
    # ==================================================================
    print_section("SECTION 1: ALL CONFIGURATIONS")
    print_main_table(all_results)

    # ==================================================================
    # SECTION 2: SPEND COST ANALYSIS (averaged)
    # ==================================================================
    print_section("SECTION 2: SPEND COST ANALYSIS (averaged over bonus/weight/dist)")

    cost_groups = defaultdict(list)
    for m in all_results:
        cost_groups[m.config.spend_cost].append(m)

    print(f"{'Cost':>4} {'1stSp':>6} {'SpFr':>6} {'LateSA':>7} {'Conv':>5} "
          f"{'DkSA%':>6} {'DQ%':>5} {'SDev':>6} {'Deg%':>5} {'CF':>5}")
    print("-" * 65)
    for cost in sorted(cost_groups.keys()):
        group = cost_groups[cost]
        print(f"   {cost} "
              f"{statistics.mean([m.first_spend_avg for m in group]):>6.1f} "
              f"{statistics.mean([m.spend_frequency_late for m in group]):>6.2f} "
              f"{statistics.mean([m.late_sa for m in group]):>7.2f} "
              f"{statistics.mean([m.convergence_pick for m in group]):>5.1f} "
              f"{statistics.mean([m.deck_sa_pct for m in group]):>5.0f}% "
              f"{statistics.mean([m.meaningful_choice_late for m in group]):>4.0f}% "
              f"{statistics.mean([m.sa_stddev_late for m in group]):>6.2f} "
              f"{sum(1 for m in group if m.always_spend_dominant)/len(group)*100:>4.0f}% "
              f"{statistics.mean([m.late_cf for m in group]):>5.2f}")

    # ==================================================================
    # SECTION 3: BONUS CARD COUNT ANALYSIS
    # ==================================================================
    print_section("SECTION 3: BONUS CARD COUNT ANALYSIS (averaged)")

    bonus_groups = defaultdict(list)
    for m in all_results:
        bonus_groups[m.config.bonus_cards].append(m)

    print(f"{'Bonus':>5} {'1stSp':>6} {'SpFr':>6} {'LateSA':>7} {'Conv':>5} "
          f"{'DkSA%':>6} {'DQ%':>5} {'SDev':>6} {'Deg%':>5}")
    print("-" * 60)
    for bc in sorted(bonus_groups.keys()):
        group = bonus_groups[bc]
        print(f"    {bc} "
              f"{statistics.mean([m.first_spend_avg for m in group]):>6.1f} "
              f"{statistics.mean([m.spend_frequency_late for m in group]):>6.2f} "
              f"{statistics.mean([m.late_sa for m in group]):>7.2f} "
              f"{statistics.mean([m.convergence_pick for m in group]):>5.1f} "
              f"{statistics.mean([m.deck_sa_pct for m in group]):>5.0f}% "
              f"{statistics.mean([m.meaningful_choice_late for m in group]):>4.0f}% "
              f"{statistics.mean([m.sa_stddev_late for m in group]):>6.2f} "
              f"{sum(1 for m in group if m.always_spend_dominant)/len(group)*100:>4.0f}%")

    # ==================================================================
    # SECTION 4: PRIMARY WEIGHT ANALYSIS
    # ==================================================================
    print_section("SECTION 4: PRIMARY WEIGHT ANALYSIS (averaged)")

    weight_groups = defaultdict(list)
    for m in all_results:
        weight_groups[m.config.primary_weight].append(m)

    print(f"{'Weight':>6} {'1stSp':>6} {'SpFr':>6} {'LateSA':>7} {'Conv':>5} "
          f"{'DkSA%':>6} {'DQ%':>5} {'SDev':>6} {'Deg%':>5}")
    print("-" * 60)
    for w in sorted(weight_groups.keys()):
        group = weight_groups[w]
        print(f"    W{w} "
              f"{statistics.mean([m.first_spend_avg for m in group]):>6.1f} "
              f"{statistics.mean([m.spend_frequency_late for m in group]):>6.2f} "
              f"{statistics.mean([m.late_sa for m in group]):>7.2f} "
              f"{statistics.mean([m.convergence_pick for m in group]):>5.1f} "
              f"{statistics.mean([m.deck_sa_pct for m in group]):>5.0f}% "
              f"{statistics.mean([m.meaningful_choice_late for m in group]):>4.0f}% "
              f"{statistics.mean([m.sa_stddev_late for m in group]):>6.2f} "
              f"{sum(1 for m in group if m.always_spend_dominant)/len(group)*100:>4.0f}%")

    # ==================================================================
    # SECTION 5: DISTRIBUTION ANALYSIS
    # ==================================================================
    print_section("SECTION 5: DISTRIBUTION ANALYSIS (averaged)")

    dist_groups = defaultdict(list)
    for m in all_results:
        dist_groups[m.config.dist_label].append(m)

    print(f"{'Distribution':<14} {'1stSp':>6} {'SpFr':>6} {'LateSA':>7} {'Conv':>5} "
          f"{'DkSA%':>6} {'DQ%':>5} {'SDev':>6} {'Deg%':>5}")
    print("-" * 65)
    for d in sorted(dist_groups.keys()):
        group = dist_groups[d]
        print(f"D{d:<13} "
              f"{statistics.mean([m.first_spend_avg for m in group]):>6.1f} "
              f"{statistics.mean([m.spend_frequency_late for m in group]):>6.2f} "
              f"{statistics.mean([m.late_sa for m in group]):>7.2f} "
              f"{statistics.mean([m.convergence_pick for m in group]):>5.1f} "
              f"{statistics.mean([m.deck_sa_pct for m in group]):>5.0f}% "
              f"{statistics.mean([m.meaningful_choice_late for m in group]):>4.0f}% "
              f"{statistics.mean([m.sa_stddev_late for m in group]):>6.2f} "
              f"{sum(1 for m in group if m.always_spend_dominant)/len(group)*100:>4.0f}%")

    # ==================================================================
    # SECTION 6: CRITICAL INTERACTION — SPEND COST x DISTRIBUTION
    # ==================================================================
    print_section("SECTION 6: SPEND COST x DISTRIBUTION INTERACTION (key finding)")
    print("Shows how spend cost and symbol distribution interact.")
    print("Columns: spend freq | late S/A | degeneracy% | arc rating\n")

    dists = ["50/35/15", "20/55/25", "10/30/60"]
    header = f"{'Cost':>4}"
    for d in dists:
        header += f"  |  D{d:<12} SpFr  SA  Dg%  Arc"
    print(header)
    print("-" * 130)

    for cost in [2, 3, 4, 5]:
        row = f"   {cost}"
        for d in dists:
            group = [m for m in all_results
                     if m.config.spend_cost == cost and m.config.dist_label == d]
            if group:
                sf = statistics.mean([m.spend_frequency_late for m in group])
                sa = statistics.mean([m.late_sa for m in group])
                dg = sum(1 for m in group if m.always_spend_dominant) / len(group) * 100
                arcs = [m.arc_rating for m in group]
                best_arc = max(set(arcs), key=arcs.count)
                row += f"  |  D{d:<12} {sf:.2f} {sa:.2f} {dg:3.0f}% {best_arc:>9}"
            else:
                row += f"  |  D{d:<12}  ---   ---  ---       ---"
        print(row)

    # ==================================================================
    # SECTION 7: SPEND COST x PRIMARY WEIGHT INTERACTION
    # ==================================================================
    print_section("SECTION 7: SPEND COST x PRIMARY WEIGHT INTERACTION")
    print("Shows the effective 'picks per spend' rate.\n")

    print(f"{'Cost':>4} {'W1 SpFr':>8} {'W1 1stSp':>9} | {'W2 SpFr':>8} {'W2 1stSp':>9} | {'W3 SpFr':>8} {'W3 1stSp':>9}")
    print("-" * 75)
    for cost in [2, 3, 4, 5]:
        parts = []
        for w in [1, 2, 3]:
            group = [m for m in all_results
                     if m.config.spend_cost == cost and m.config.primary_weight == w]
            if group:
                sf = statistics.mean([m.spend_frequency_late for m in group])
                fs = statistics.mean([m.first_spend_avg for m in group])
                parts.append(f"{sf:>8.2f} {fs:>9.1f}")
            else:
                parts.append(f"{'---':>8} {'---':>9}")
        print(f"   {cost} {parts[0]} | {parts[1]} | {parts[2]}")

    # ==================================================================
    # SECTION 8: TOP 15 CONFIGURATIONS
    # ==================================================================
    print_section("SECTION 8: TOP 15 CONFIGURATIONS (by arc rating then late S/A)")

    arc_order = {"EXCELLENT": 4, "GOOD": 3, "FAIR": 2, "POOR": 1}
    sorted_results = sorted(
        all_results,
        key=lambda m: (arc_order.get(m.arc_rating, 0), m.late_sa),
        reverse=True
    )
    print_main_table(sorted_results[:15])

    # ==================================================================
    # SECTION 9: CONVERGENCE CURVES FOR TOP 5
    # ==================================================================
    print_section("SECTION 9: CONVERGENCE CURVES (S/A per pack) — TOP 5")

    picks = [1, 3, 5, 7, 10, 12, 15, 18, 20, 25, 30]
    header = f"{'Config':<28} " + " ".join(f"{'P'+str(p):>5}" for p in picks)
    print(header)
    print("-" * (28 + 6 * len(picks)))
    for m in sorted_results[:5]:
        vals = " ".join(f"{m.sa_curve.get(p, 0):>5.2f}" for p in picks)
        print(f"{m.config.name:<28} {vals}")

    # ==================================================================
    # SECTION 10: THREE-ACT ARC ANALYSIS FOR TOP 10
    # ==================================================================
    print_section("SECTION 10: THREE-ACT DRAFT ARC — TOP 10 CONFIGS")

    for m in sorted_results[:10]:
        print(f"\n{m.config.name} [{m.arc_rating}]")
        print(f"  Act 1 (Exploration, picks 1-5): {m.act1_description}")
        print(f"  Act 2 (Commitment, picks 6-15): {m.act2_description}")
        print(f"  Act 3 (Refinement, picks 16-30): {m.act3_description}")
        print(f"  Spend freq: {m.spend_frequency_late:.2f} | "
              f"1st spend: pick {m.first_spend_avg:.1f} | "
              f"Always-spend dominant: {'YES' if m.always_spend_dominant else 'no'}")

    # ==================================================================
    # SECTION 11: MEASURABLE TARGETS TABLE FOR TOP 5
    # ==================================================================
    print_section("SECTION 11: MEASURABLE TARGETS — TOP 5 CONFIGS")

    for m in sorted_results[:5]:
        print(f"\n--- {m.config.name} [{m.arc_rating}] ---")
        t = []
        # Picks 1-5: unique archetypes >= 3
        v = m.early_unique_archs
        t.append(("Picks 1-5: unique archs w/ S/A per pack (>= 3)", v, v >= 3))
        # Picks 1-5: S/A for emerging arch <= 2
        v = m.early_sa_emerging
        t.append(("Picks 1-5: S/A for emerging arch per pack (<= 2)", v, v <= 2))
        # Picks 6+: S/A >= 2 avg
        v = m.late_sa
        t.append(("Picks 6+: S/A for committed arch per pack (>= 2)", v, v >= 2))
        # Picks 6+: off-archetype >= 0.5
        v = m.late_cf
        t.append(("Picks 6+: off-arch C/F per pack (>= 0.5)", v, v >= 0.5))
        # Convergence pick 5-8
        v = m.convergence_pick
        t.append(("Convergence pick (5-8)", v, 5 <= v <= 8))
        # Deck S/A 60-90%
        v = m.deck_sa_pct
        t.append(("Deck archetype concentration (60-90%)", v, 60 <= v <= 90))
        # Card overlap < 40%
        v = m.card_overlap
        t.append(("Run-to-run variety (< 40% overlap)", v, v < 40))
        # Arch freq balance
        t.append(("Arch freq max (< 20%)", m.arch_freq_max, m.arch_freq_max < 20))
        t.append(("Arch freq min (> 5%)", m.arch_freq_min, m.arch_freq_min > 5))
        # Variance
        v = m.sa_stddev_late
        t.append(("S/A stddev picks 6+ (>= 0.8)", v, v >= 0.8))

        passed = sum(1 for _, _, ok in t if ok)
        print(f"  Passed: {passed}/{len(t)}")
        for label, val, ok in t:
            status = "PASS" if ok else "FAIL"
            if isinstance(val, float):
                print(f"  [{status}] {label}: {val:.2f}")
            else:
                print(f"  [{status}] {label}: {val}")

    # ==================================================================
    # SECTION 12: ALWAYS-SPEND DEGENERACY DEEP DIVE
    # ==================================================================
    print_section("SECTION 12: ALWAYS-SPEND DEGENERACY ANALYSIS")
    print("Compares deck S/A% across three strategies.\n")

    print(f"{'Config':<28} {'Committed':>9} {'AlwSpend':>9} {'SaveSpnd':>9} {'Dominant':>9}")
    print("-" * 68)
    for m in sorted_results[:20]:
        dom = "ALWAYS" if m.always_spend_dominant else "mixed"
        print(f"{m.config.name:<28} "
              f"{m.committed_sa:>8.1f}% "
              f"{m.always_spend_sa:>8.1f}% "
              f"{m.save_spend_sa:>8.1f}% "
              f"{dom:>9}")

    # ==================================================================
    # SECTION 13: RECOMMENDED PARAMETER SET
    # ==================================================================
    print_section("SECTION 13: RECOMMENDED CONFIGURATIONS")
    print("Filters: arc GOOD+, not always-spend-dominant, late SA >= 1.8, convergence <= 10\n")

    recommended = [m for m in all_results
                   if m.arc_rating in ("EXCELLENT", "GOOD")
                   and not m.always_spend_dominant
                   and m.late_sa >= 1.8
                   and m.convergence_pick <= 10]
    recommended.sort(key=lambda m: (arc_order.get(m.arc_rating, 0), m.late_sa), reverse=True)

    if recommended:
        print_main_table(recommended[:10])
    else:
        print("No configurations pass all filters. Relaxing constraints...")
        recommended = [m for m in all_results
                       if m.arc_rating in ("EXCELLENT", "GOOD", "FAIR")
                       and m.late_sa >= 1.5
                       and m.convergence_pick <= 12]
        recommended.sort(key=lambda m: (arc_order.get(m.arc_rating, 0), m.late_sa), reverse=True)
        print_main_table(recommended[:10])


if __name__ == "__main__":
    main()
