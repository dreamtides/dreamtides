#!/usr/bin/env python3
"""
Agent 2: Rarity x Pack Widening v3 Simulation

Investigates how rarity should interact with Pack Widening v3 and resonance symbols.

Tests 5 rarity models:
  A: Flat rarity (cosmetic only, uniform power)
  B: Standard TCG (180C/100U/60R/20L, power scales with rarity)
  C: Roguelike-skewed (120C/120U/80R/40L, power scales)
  D: Rarity-symbol correlation (rares have more symbols)
  E: Inverse correlation (rares have fewer symbols)

All models use Pack Widening v3 (spend cost 3, bonus 1, primary weight 2).
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
BONUS_CARDS = 1
SPEND_COST = 3
PRIMARY_TOKEN_WEIGHT = 2
SECONDARY_TOKEN_WEIGHT = 1
NUM_PICKS = 30
NUM_SIMULATIONS = 1200

RESONANCES = ["Ember", "Stone", "Tide", "Zephyr"]

ARCHETYPES = [
    ("Flash",        "Zephyr", "Ember"),   # 1
    ("Blink",        "Ember",  "Zephyr"),  # 2
    ("Storm",        "Ember",  "Stone"),   # 3
    ("Self-Discard", "Stone",  "Ember"),   # 4
    ("Self-Mill",    "Stone",  "Tide"),    # 5
    ("Sacrifice",    "Tide",   "Stone"),   # 6
    ("Warriors",     "Tide",   "Zephyr"),  # 7
    ("Ramp",         "Zephyr", "Tide"),    # 8
]

ARCHETYPE_NAMES = [a[0] for a in ARCHETYPES]


# ─── Data Classes ────────────────────────────────────────────────────────────

class Rarity(Enum):
    COMMON = 0
    UNCOMMON = 1
    RARE = 2
    LEGENDARY = 3


RARITY_NAMES = {Rarity.COMMON: "C", Rarity.UNCOMMON: "U",
                Rarity.RARE: "R", Rarity.LEGENDARY: "L"}


@dataclass
class SimCard:
    id: int
    symbols: list
    archetype: Optional[str]
    archetype_fitness: dict
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
    d = abs(i - j)
    return min(d, NUM_ARCHETYPES - d)


def are_adjacent(i: int, j: int) -> bool:
    return circle_distance(i, j) == 1


def compute_fitness(card_archetype: Optional[str], card_symbols: list) -> dict:
    """Compute fitness tiers using the archetype circle model."""
    fitness = {}
    if card_archetype is None:
        # Generic cards are B-tier everywhere
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
            fitness[name] = "A"
        elif home_secondary in (arch_pri, arch_sec) and i != home_idx:
            fitness[name] = "B"
        elif home_primary in (arch_pri, arch_sec):
            fitness[name] = "B"
        else:
            fitness[name] = "F"

    return fitness


# ─── Power Scaling by Rarity ────────────────────────────────────────────────

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


# ─── Symbol Generation ──────────────────────────────────────────────────────

def make_symbols(num_symbols: int, primary: str, secondary: str) -> list:
    """Generate a symbol list for a card in an archetype."""
    if num_symbols == 1:
        return [primary] if random.random() < 0.80 else [secondary]
    elif num_symbols == 2:
        r = random.random()
        if r < 0.55:
            return [primary, secondary]
        elif r < 0.80:
            return [primary, primary]
        elif r < 0.95:
            return [secondary, primary]
        else:
            return [secondary, secondary]
    else:  # 3
        r = random.random()
        if r < 0.45:
            return [primary, primary, secondary]
        elif r < 0.75:
            return [primary, secondary, primary]
        elif r < 0.90:
            return [primary, secondary, secondary]
        else:
            return [secondary, primary, secondary]


def symbol_count_default(card_index: int, n_cards: int) -> int:
    """Default distribution: 25% 1-sym, 55% 2-sym, 20% 3-sym."""
    n1 = round(n_cards * 0.25)
    n2 = round(n_cards * 0.55)
    if card_index < n1:
        return 1
    elif card_index < n1 + n2:
        return 2
    else:
        return 3


# ─── Pool Generation ────────────────────────────────────────────────────────

def distribute_cards_per_archetype() -> list:
    cards_per = [NUM_ARCHETYPE_CARDS // NUM_ARCHETYPES] * NUM_ARCHETYPES
    leftover = NUM_ARCHETYPE_CARDS - sum(cards_per)
    for i in range(leftover):
        cards_per[i] += 1
    return cards_per


def _generate_pool_with_rarity_counts(rarity_counts: dict, power_fn,
                                       symbol_fn) -> list:
    """Generate pool: rarity from fixed distribution, symbols from default."""
    cards = []
    card_id = 0

    rarity_list = []
    for r, count in rarity_counts.items():
        rarity_list.extend([r] * count)
    random.shuffle(rarity_list)
    rarity_idx = 0

    cards_per = distribute_cards_per_archetype()

    for arch_idx, (arch_name, primary, secondary) in enumerate(ARCHETYPES):
        n_cards = cards_per[arch_idx]
        for i in range(n_cards):
            num_sym = symbol_fn(i, n_cards)
            symbols = make_symbols(num_sym, primary, secondary)
            rarity = rarity_list[rarity_idx] if rarity_idx < len(rarity_list) else Rarity.COMMON
            rarity_idx += 1
            fitness = compute_fitness(arch_name, symbols)
            cards.append(SimCard(
                id=card_id, symbols=symbols, archetype=arch_name,
                archetype_fitness=fitness,
                rarity=rarity, power=power_fn(rarity),
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


def _generate_pool_rarity_symbol_correlated(rarity_counts: dict, power_fn,
                                              symbol_count_fn) -> list:
    """Generate pool where symbol count depends on rarity."""
    cards = []
    card_id = 0

    cards_per = distribute_cards_per_archetype()

    for arch_idx, (arch_name, primary, secondary) in enumerate(ARCHETYPES):
        n_cards = cards_per[arch_idx]
        # Proportional rarity assignment
        arch_rarities = []
        for r, total_count in rarity_counts.items():
            archetype_share = total_count * n_cards / NUM_CARDS
            arch_rarities.extend([r] * round(archetype_share))
        while len(arch_rarities) < n_cards:
            arch_rarities.append(Rarity.COMMON)
        arch_rarities = arch_rarities[:n_cards]
        random.shuffle(arch_rarities)

        for i in range(n_cards):
            rarity = arch_rarities[i]
            num_sym = symbol_count_fn(rarity)
            symbols = make_symbols(num_sym, primary, secondary)
            fitness = compute_fitness(arch_name, symbols)
            cards.append(SimCard(
                id=card_id, symbols=symbols, archetype=arch_name,
                archetype_fitness=fitness,
                rarity=rarity, power=power_fn(rarity),
            ))
            card_id += 1

    # Generic cards
    generic_rarities = []
    for r, total_count in rarity_counts.items():
        generic_share = total_count * NUM_GENERIC / NUM_CARDS
        generic_rarities.extend([r] * round(generic_share))
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


# ─── Pool Generators (5 models) ─────────────────────────────────────────────

def generate_pool_model_a() -> list:
    """Model A: Flat rarity. Cosmetic only, uniform power."""
    return _generate_pool_with_rarity_counts(
        {Rarity.COMMON: 180, Rarity.UNCOMMON: 100,
         Rarity.RARE: 60, Rarity.LEGENDARY: 20},
        power_fn=flat_power, symbol_fn=symbol_count_default,
    )


def generate_pool_model_b() -> list:
    """Model B: Standard TCG. 180C/100U/60R/20L, power scales."""
    return _generate_pool_with_rarity_counts(
        {Rarity.COMMON: 180, Rarity.UNCOMMON: 100,
         Rarity.RARE: 60, Rarity.LEGENDARY: 20},
        power_fn=scaled_power, symbol_fn=symbol_count_default,
    )


def generate_pool_model_c() -> list:
    """Model C: Roguelike-skewed. 120C/120U/80R/40L, power scales."""
    return _generate_pool_with_rarity_counts(
        {Rarity.COMMON: 120, Rarity.UNCOMMON: 120,
         Rarity.RARE: 80, Rarity.LEGENDARY: 40},
        power_fn=scaled_power, symbol_fn=symbol_count_default,
    )


def generate_pool_model_d() -> list:
    """Model D: Rarity-symbol correlation. Rares have more symbols."""
    def symbol_fn_d(rarity: Rarity) -> int:
        if rarity == Rarity.COMMON:
            return random.choices([1, 2], weights=[0.75, 0.25])[0]
        elif rarity == Rarity.UNCOMMON:
            return random.choices([1, 2, 3], weights=[0.15, 0.70, 0.15])[0]
        elif rarity == Rarity.RARE:
            return random.choices([2, 3], weights=[0.50, 0.50])[0]
        else:  # Legendary
            return random.choices([2, 3], weights=[0.30, 0.70])[0]

    return _generate_pool_rarity_symbol_correlated(
        {Rarity.COMMON: 180, Rarity.UNCOMMON: 100,
         Rarity.RARE: 60, Rarity.LEGENDARY: 20},
        power_fn=scaled_power, symbol_count_fn=symbol_fn_d,
    )


def generate_pool_model_e() -> list:
    """Model E: Inverse correlation. Rares have fewer symbols."""
    def symbol_fn_e(rarity: Rarity) -> int:
        if rarity == Rarity.COMMON:
            return random.choices([2, 3], weights=[0.65, 0.35])[0]
        elif rarity == Rarity.UNCOMMON:
            return random.choices([1, 2], weights=[0.50, 0.50])[0]
        elif rarity == Rarity.RARE:
            return random.choices([1, 2], weights=[0.80, 0.20])[0]
        else:  # Legendary
            return 1

    return _generate_pool_rarity_symbol_correlated(
        {Rarity.COMMON: 180, Rarity.UNCOMMON: 100,
         Rarity.RARE: 60, Rarity.LEGENDARY: 20},
        power_fn=scaled_power, symbol_count_fn=symbol_fn_e,
    )


# ─── Pack Widening v3 Algorithm ──────────────────────────────────────────────

@dataclass
class PackWideningState:
    tokens: dict = field(default_factory=lambda: {r: 0 for r in RESONANCES})

    def can_spend(self) -> list:
        """Return list of resonances with enough tokens to spend."""
        return [r for r in RESONANCES if self.tokens[r] >= SPEND_COST]

    def spend(self, resonance: str):
        self.tokens[resonance] -= SPEND_COST

    def earn_tokens(self, card: SimCard):
        if not card.symbols:
            return
        for i, sym in enumerate(card.symbols):
            weight = PRIMARY_TOKEN_WEIGHT if i == 0 else SECONDARY_TOKEN_WEIGHT
            self.tokens[sym] += weight

    def total_tokens(self) -> int:
        return sum(self.tokens.values())

    def primary_tokens(self, archetype_name: str) -> int:
        """Tokens in this archetype's primary resonance."""
        idx = get_archetype_index(archetype_name)
        primary_res = ARCHETYPES[idx][1]
        return self.tokens[primary_res]


def build_primary_index(pool: list) -> dict:
    """Pre-build index: resonance -> list of cards with that primary."""
    index = {r: [] for r in RESONANCES}
    for card in pool:
        if card.primary_resonance:
            index[card.primary_resonance].append(card)
    return index


def build_pack_widening(pool: list, primary_index: dict, state: PackWideningState,
                         committed_arch: Optional[str], drafted: list) -> tuple:
    """
    Build a pack using Pack Widening v3.
    Returns (pack, spent_resonance_or_None, is_spend_pack).
    """
    spendable = state.can_spend()
    spent_resonance = None

    # Spending decision: spend on primary resonance if committed, else best available
    if spendable:
        if committed_arch:
            arch_idx = get_archetype_index(committed_arch)
            primary_res = ARCHETYPES[arch_idx][1]
            if primary_res in spendable:
                spent_resonance = primary_res
            else:
                # Spend on whichever spendable resonance has most tokens
                spent_resonance = max(spendable, key=lambda r: state.tokens[r])
        else:
            # Pre-commit: spend on highest token count
            spent_resonance = max(spendable, key=lambda r: state.tokens[r])

    if spent_resonance:
        state.spend(spent_resonance)

    # Draw base pack: 4 cards uniformly at random
    pack = random.sample(pool, PACK_SIZE)

    # If spending, draw bonus card(s) from primary resonance pool
    if spent_resonance:
        bonus_pool = primary_index[spent_resonance]
        if bonus_pool:
            bonus_cards = random.choices(bonus_pool, k=BONUS_CARDS)
            pack.extend(bonus_cards)

    is_spend = spent_resonance is not None
    return pack, spent_resonance, is_spend


# ─── Player Strategies ───────────────────────────────────────────────────────

TIER_VALUES = {"S": 5, "A": 4, "B": 3, "C": 1, "F": 0}


def evaluate_archetype_strength(drafted: list) -> dict:
    scores = {name: 0 for name in ARCHETYPE_NAMES}
    for card in drafted:
        for arch, tier in card.archetype_fitness.items():
            scores[arch] += TIER_VALUES[tier]
    return scores


def card_fits_archetype(card: SimCard, arch_name: str) -> bool:
    return card.archetype_fitness.get(arch_name, "F") in ("S", "A")


def card_is_off_archetype(card: SimCard, arch_name: str) -> bool:
    return card.archetype_fitness.get(arch_name, "F") in ("C", "F")


def pick_archetype_committed(pack: list, drafted: list,
                              committed_arch: Optional[str]) -> tuple:
    """Pick best card for committed archetype. Commits at pick 5."""
    if committed_arch is None and len(drafted) >= 5:
        scores = evaluate_archetype_strength(drafted)
        committed_arch = max(scores, key=scores.get)

    if committed_arch is None:
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
    """Always picks highest raw power."""
    return max(pack, key=lambda c: c.power)


# ─── Evaluation Helpers ─────────────────────────────────────────────────────

def get_unique_archetypes_with_sa(pack: list) -> set:
    archetypes_present = set()
    for card in pack:
        for arch_name in ARCHETYPE_NAMES:
            if card.archetype_fitness.get(arch_name, "F") in ("S", "A"):
                archetypes_present.add(arch_name)
    return archetypes_present


# ─── Simulation Core ────────────────────────────────────────────────────────

@dataclass
class DraftMetrics:
    # Standard metrics
    early_unique_archetypes_per_pack: list = field(default_factory=list)
    early_archetype_fit_per_pack: list = field(default_factory=list)
    late_archetype_fit_per_pack: list = field(default_factory=list)
    late_off_archetype_per_pack: list = field(default_factory=list)
    convergence_pick: Optional[int] = None
    deck_concentration: float = 0.0
    drafted_card_ids: list = field(default_factory=list)

    # StdDev of S/A per pack (late)
    late_sa_per_pack_values: list = field(default_factory=list)

    # Power metrics
    total_power: float = 0.0
    rarity_counts_drafted: dict = field(default_factory=lambda: {r: 0 for r in Rarity})

    # Token economy
    tokens_earned_per_pick: list = field(default_factory=list)
    spend_picks: int = 0
    non_spend_picks: int = 0
    total_picks_late: int = 0  # picks 6+

    # Draft tension (separate for spend and non-spend packs)
    tension_spend_packs: int = 0
    tension_non_spend_packs: int = 0
    total_spend_packs_late: int = 0
    total_non_spend_packs_late: int = 0

    # Rarity-token interaction
    tokens_from_rares: list = field(default_factory=list)  # tokens earned from R/L picks
    tokens_from_commons: list = field(default_factory=list)  # tokens earned from C/U picks

    # Token balance at each pick for curve analysis
    primary_token_curve: list = field(default_factory=list)

    # Archetype committed to
    final_archetype: Optional[str] = None


def tokens_earned_from_card(card: SimCard) -> int:
    """How many total tokens does picking this card earn?"""
    if not card.symbols:
        return 0
    total = 0
    for i in range(len(card.symbols)):
        total += PRIMARY_TOKEN_WEIGHT if i == 0 else SECONDARY_TOKEN_WEIGHT
    return total


def run_single_draft(pool: list, primary_index: dict,
                      strategy: str = "archetype_committed") -> DraftMetrics:
    state = PackWideningState()
    drafted = []
    metrics = DraftMetrics()
    committed_arch = None
    convergence_found = False

    for pick_num in range(NUM_PICKS):
        # Build pack
        pack, spent_res, is_spend = build_pack_widening(
            pool, primary_index, state, committed_arch, drafted)

        # Current best archetype for evaluation
        if drafted:
            scores = evaluate_archetype_strength(drafted)
            current_best_arch = max(scores, key=scores.get)
        else:
            current_best_arch = None

        # Early metrics (picks 0-4)
        if pick_num < 5:
            # Only count base pack cards for fair comparison
            base_pack = pack[:PACK_SIZE]
            unique_archs = get_unique_archetypes_with_sa(base_pack)
            metrics.early_unique_archetypes_per_pack.append(len(unique_archs))
            if current_best_arch:
                fit_count = sum(1 for c in base_pack
                                if card_fits_archetype(c, current_best_arch))
                metrics.early_archetype_fit_per_pack.append(fit_count)

        # Late metrics (picks 5+)
        if pick_num >= 5 and current_best_arch:
            metrics.total_picks_late += 1
            fit_count = sum(1 for c in pack
                            if card_fits_archetype(c, current_best_arch))
            metrics.late_archetype_fit_per_pack.append(fit_count)
            metrics.late_sa_per_pack_values.append(fit_count)
            off_count = sum(1 for c in pack
                            if card_is_off_archetype(c, current_best_arch))
            metrics.late_off_archetype_per_pack.append(off_count)

            if not convergence_found and fit_count >= 2:
                metrics.convergence_pick = pick_num + 1
                convergence_found = True

            # Draft tension: best off-archetype power vs best on-archetype power
            on_arch = [c for c in pack if card_fits_archetype(c, current_best_arch)]
            off_arch = [c for c in pack if not card_fits_archetype(c, current_best_arch)]
            has_tension = False
            if on_arch and off_arch:
                best_on_power = max(c.power for c in on_arch)
                best_off_power = max(c.power for c in off_arch)
                if best_off_power - best_on_power >= 2.0:
                    has_tension = True

            if is_spend:
                metrics.total_spend_packs_late += 1
                if has_tension:
                    metrics.tension_spend_packs += 1
            else:
                metrics.total_non_spend_packs_late += 1
                if has_tension:
                    metrics.tension_non_spend_packs += 1

        # Track spending
        if is_spend:
            metrics.spend_picks += 1
        else:
            metrics.non_spend_picks += 1

        # Pick card
        if strategy == "archetype_committed":
            chosen, committed_arch = pick_archetype_committed(
                pack, drafted, committed_arch)
        elif strategy == "power_chaser":
            chosen = pick_power_chaser(pack, drafted)
        else:
            raise ValueError(f"Unknown strategy: {strategy}")

        # Track tokens earned before adding
        tok_earned = tokens_earned_from_card(chosen)
        metrics.tokens_earned_per_pick.append(tok_earned)

        # Rarity-token interaction tracking
        if chosen.rarity in (Rarity.RARE, Rarity.LEGENDARY):
            metrics.tokens_from_rares.append(tok_earned)
        else:
            metrics.tokens_from_commons.append(tok_earned)

        # Earn tokens
        state.earn_tokens(chosen)

        # Track primary token curve
        if committed_arch:
            metrics.primary_token_curve.append(
                state.primary_tokens(committed_arch))
        else:
            # Before commitment, track max token
            metrics.primary_token_curve.append(max(state.tokens.values()))

        drafted.append(chosen)
        metrics.drafted_card_ids.append(chosen.id)
        metrics.total_power += chosen.power
        metrics.rarity_counts_drafted[chosen.rarity] += 1

    # Deck concentration
    if drafted:
        scores = evaluate_archetype_strength(drafted)
        best_arch = max(scores, key=scores.get)
        sa_count = sum(1 for c in drafted if card_fits_archetype(c, best_arch))
        metrics.deck_concentration = sa_count / len(drafted)
        metrics.final_archetype = best_arch

    return metrics


# ─── Aggregate Results ───────────────────────────────────────────────────────

def compute_aggregate(all_metrics: list) -> dict:
    s = {}

    all_early_archs = [v for m in all_metrics
                       for v in m.early_unique_archetypes_per_pack]
    s["early_unique_archs"] = statistics.mean(all_early_archs) if all_early_archs else 0

    all_early_fit = [v for m in all_metrics
                     for v in m.early_archetype_fit_per_pack]
    s["early_arch_fit"] = statistics.mean(all_early_fit) if all_early_fit else 0

    all_late_fit = [v for m in all_metrics
                    for v in m.late_archetype_fit_per_pack]
    s["late_arch_fit"] = statistics.mean(all_late_fit) if all_late_fit else 0

    all_late_off = [v for m in all_metrics
                    for v in m.late_off_archetype_per_pack]
    s["late_off_arch"] = statistics.mean(all_late_off) if all_late_off else 0

    conv_picks = [m.convergence_pick for m in all_metrics
                  if m.convergence_pick is not None]
    s["convergence_pick"] = statistics.mean(conv_picks) if conv_picks else float("inf")

    s["deck_concentration"] = statistics.mean(
        [m.deck_concentration for m in all_metrics])

    # StdDev of S/A per pack (late)
    all_late_sa = [v for m in all_metrics for v in m.late_sa_per_pack_values]
    s["late_sa_stddev"] = statistics.stdev(all_late_sa) if len(all_late_sa) > 1 else 0

    # Card overlap
    if len(all_metrics) >= 2:
        overlaps = []
        sample_size = min(200, len(all_metrics))
        sampled = random.sample(all_metrics, sample_size)
        for i in range(0, sample_size - 1, 2):
            s1 = set(sampled[i].drafted_card_ids)
            s2 = set(sampled[i + 1].drafted_card_ids)
            if s1 | s2:
                overlaps.append(len(s1 & s2) / len(s1 | s2))
        s["card_overlap"] = statistics.mean(overlaps) if overlaps else 0
    else:
        s["card_overlap"] = 0

    # Power stats
    all_powers = [m.total_power / NUM_PICKS for m in all_metrics]
    s["avg_power"] = statistics.mean(all_powers)
    s["power_stdev"] = statistics.stdev(all_powers) if len(all_powers) > 1 else 0

    # Token economy
    all_tok = [v for m in all_metrics for v in m.tokens_earned_per_pick]
    s["avg_tokens_per_pick"] = statistics.mean(all_tok) if all_tok else 0

    total_spend = sum(m.spend_picks for m in all_metrics)
    total_non_spend = sum(m.non_spend_picks for m in all_metrics)
    s["spend_frequency"] = total_spend / (total_spend + total_non_spend) if (total_spend + total_non_spend) > 0 else 0

    # Spend frequency for late picks only
    total_spend_late = sum(m.total_spend_packs_late for m in all_metrics)
    total_non_spend_late = sum(m.total_non_spend_packs_late for m in all_metrics)
    s["late_spend_frequency"] = total_spend_late / (total_spend_late + total_non_spend_late) if (total_spend_late + total_non_spend_late) > 0 else 0

    # Draft tension: separate for spend vs non-spend
    total_tension_spend = sum(m.tension_spend_packs for m in all_metrics)
    total_tension_non_spend = sum(m.tension_non_spend_packs for m in all_metrics)
    s["tension_rate_spend"] = (total_tension_spend / total_spend_late * 100
                                if total_spend_late > 0 else 0)
    s["tension_rate_non_spend"] = (total_tension_non_spend / total_non_spend_late * 100
                                    if total_non_spend_late > 0 else 0)
    total_tension = total_tension_spend + total_tension_non_spend
    total_late = total_spend_late + total_non_spend_late
    s["tension_rate_overall"] = total_tension / total_late * 100 if total_late > 0 else 0

    # Rarity-token interaction
    all_rare_tok = [v for m in all_metrics for v in m.tokens_from_rares]
    all_common_tok = [v for m in all_metrics for v in m.tokens_from_commons]
    s["avg_tokens_from_rares"] = statistics.mean(all_rare_tok) if all_rare_tok else 0
    s["avg_tokens_from_commons"] = statistics.mean(all_common_tok) if all_common_tok else 0

    # Rarity drafted breakdown
    rarity_totals = {r: 0 for r in Rarity}
    for m in all_metrics:
        for r, cnt in m.rarity_counts_drafted.items():
            rarity_totals[r] += cnt
    total_drafted = sum(rarity_totals.values())
    s["rarity_drafted"] = {
        RARITY_NAMES[r]: cnt / total_drafted * 100 if total_drafted else 0
        for r, cnt in rarity_totals.items()
    }

    return s


def measure_archetype_frequency(pool: list, primary_index: dict,
                                 n: int = 1000) -> dict:
    arch_counts = Counter()
    for _ in range(n):
        m = run_single_draft(pool, primary_index, "archetype_committed")
        arch_counts[m.final_archetype] += 1
    total = sum(arch_counts.values())
    return {k: v / total for k, v in arch_counts.items() if k is not None}


# ─── Pool Statistics ────────────────────────────────────────────────────────

def pool_stats(pool: list, label: str) -> dict:
    stats = {}
    rarity_counts = Counter(c.rarity for c in pool)
    stats["rarity_counts"] = {RARITY_NAMES[r]: rarity_counts.get(r, 0) for r in Rarity}
    stats["avg_power"] = statistics.mean(c.power for c in pool)
    stats["power_by_rarity"] = {}
    for r in Rarity:
        cards_of_r = [c for c in pool if c.rarity == r]
        if cards_of_r:
            stats["power_by_rarity"][RARITY_NAMES[r]] = statistics.mean(
                c.power for c in cards_of_r)

    sym_counts = Counter(len(c.symbols) for c in pool)
    stats["symbol_dist"] = dict(sorted(sym_counts.items()))

    stats["symbols_by_rarity"] = {}
    for r in Rarity:
        cards_of_r = [c for c in pool if c.rarity == r and c.symbols]
        if cards_of_r:
            stats["symbols_by_rarity"][RARITY_NAMES[r]] = statistics.mean(
                len(c.symbols) for c in cards_of_r)

    # Average tokens earned per card by rarity
    stats["tokens_by_rarity"] = {}
    for r in Rarity:
        cards_of_r = [c for c in pool if c.rarity == r]
        if cards_of_r:
            stats["tokens_by_rarity"][RARITY_NAMES[r]] = statistics.mean(
                tokens_earned_from_card(c) for c in cards_of_r)

    print(f"\n  Pool stats for {label}:")
    print(f"    Rarity: {stats['rarity_counts']}")
    print(f"    Avg power: {stats['avg_power']:.2f}")
    pwr_str = ', '.join(f'{k}={v:.2f}' for k, v in stats['power_by_rarity'].items())
    print(f"    Power by rarity: {pwr_str}")
    print(f"    Symbol dist: {stats['symbol_dist']}")
    sym_str = ', '.join(f'{k}={v:.2f}' for k, v in stats['symbols_by_rarity'].items())
    print(f"    Avg symbols by rarity: {sym_str}")
    tok_str = ', '.join(f'{k}={v:.2f}' for k, v in stats['tokens_by_rarity'].items())
    print(f"    Avg tokens/card by rarity: {tok_str}")

    return stats


# ─── Main ────────────────────────────────────────────────────────────────────

def print_model_results(model_name: str, agg: dict, arch_freq: dict,
                         pool_st: dict):
    s = agg

    print(f"\n{'='*70}")
    print(f"  {model_name}")
    print(f"{'='*70}")

    # Standard targets
    print(f"  Early unique archs w/ S/A:     {s['early_unique_archs']:.2f}  (>= 3)")
    print(f"  Early S/A for arch/pack:       {s['early_arch_fit']:.2f}  (<= 2)")
    print(f"  Late S/A for arch/pack:        {s['late_arch_fit']:.2f}  (>= 2)")
    print(f"  Late C/F cards/pack:           {s['late_off_arch']:.2f}  (>= 0.5)")
    print(f"  Convergence pick:              {s['convergence_pick']:.1f}  (5-8)")
    print(f"  Deck concentration:            {s['deck_concentration']*100:.1f}%  (60-90%)")
    print(f"  Card overlap:                  {s['card_overlap']*100:.1f}%  (< 40%)")
    print(f"  Late S/A stddev:               {s['late_sa_stddev']:.2f}  (>= 0.8)")

    if arch_freq:
        max_freq = max(arch_freq.values())
        min_freq = min(arch_freq.values())
        print(f"  Archetype freq range:          {min_freq*100:.1f}-{max_freq*100:.1f}%  (5-20%)")

    # Token economy
    print(f"\n  --- Token Economy ---")
    print(f"  Avg tokens per pick:           {s['avg_tokens_per_pick']:.2f}")
    print(f"  Overall spend frequency:       {s['spend_frequency']*100:.1f}%")
    print(f"  Late spend frequency:          {s['late_spend_frequency']*100:.1f}%")

    # Draft tension
    print(f"\n  --- Draft Tension (rare-vs-common dilemma) ---")
    print(f"  Tension rate (spend packs):    {s['tension_rate_spend']:.1f}%")
    print(f"  Tension rate (non-spend packs):{s['tension_rate_non_spend']:.1f}%")
    print(f"  Tension rate (overall):        {s['tension_rate_overall']:.1f}%")

    # Power & rarity
    print(f"\n  --- Power & Rarity ---")
    print(f"  Avg power per card:            {s['avg_power']:.2f}")
    print(f"  Power stdev across runs:       {s['power_stdev']:.3f}")
    print(f"  Avg tokens from R/L picks:     {s['avg_tokens_from_rares']:.2f}")
    print(f"  Avg tokens from C/U picks:     {s['avg_tokens_from_commons']:.2f}")
    rd = s['rarity_drafted']
    print(f"  Rarity drafted (C/U/R/L):      {rd['C']:.1f}%/{rd['U']:.1f}%/{rd['R']:.1f}%/{rd['L']:.1f}%")


def count_passes(agg: dict, arch_freq: dict) -> int:
    s = agg
    max_freq = max(arch_freq.values()) if arch_freq else 1
    min_freq = min(arch_freq.values()) if arch_freq else 0
    passes = 0
    if s["early_unique_archs"] >= 3: passes += 1
    if s["early_arch_fit"] <= 2: passes += 1
    if s["late_arch_fit"] >= 2: passes += 1
    if s["late_off_arch"] >= 0.5: passes += 1
    if 5 <= s["convergence_pick"] <= 8: passes += 1
    if 60 <= s["deck_concentration"] * 100 <= 90: passes += 1
    if s["card_overlap"] * 100 < 40: passes += 1
    if max_freq <= 0.20 and min_freq >= 0.05: passes += 1
    if s["late_sa_stddev"] >= 0.8: passes += 1
    return passes


def main():
    random.seed(42)

    print("=" * 70)
    print("  RARITY x PACK WIDENING v3 SIMULATION")
    print(f"  Cost={SPEND_COST}, Bonus={BONUS_CARDS}, PrimaryWt={PRIMARY_TOKEN_WEIGHT}")
    print(f"  {NUM_SIMULATIONS} drafts per model, {NUM_PICKS} picks per draft")
    print("=" * 70)

    models = {
        "Model A: Flat Rarity (cosmetic, uniform power)": generate_pool_model_a,
        "Model B: Standard TCG (180C/100U/60R/20L, power scales)": generate_pool_model_b,
        "Model C: Roguelike-Skewed (120C/120U/80R/40L, power scales)": generate_pool_model_c,
        "Model D: Rarity-Symbol Correlation (rares=more symbols)": generate_pool_model_d,
        "Model E: Inverse Correlation (rares=fewer symbols)": generate_pool_model_e,
    }

    all_results = {}

    for model_name, gen_fn in models.items():
        print(f"\n{'='*70}")
        print(f"  Generating: {model_name}")

        pool = gen_fn()
        pstats = pool_stats(pool, model_name)
        primary_index = build_primary_index(pool)

        print(f"  Running {NUM_SIMULATIONS} archetype-committed drafts...")
        committed_metrics = []
        for _ in range(NUM_SIMULATIONS):
            m = run_single_draft(pool, primary_index, "archetype_committed")
            committed_metrics.append(m)
        agg_committed = compute_aggregate(committed_metrics)

        print(f"  Running {NUM_SIMULATIONS} power-chaser drafts...")
        power_metrics = []
        for _ in range(NUM_SIMULATIONS):
            m = run_single_draft(pool, primary_index, "power_chaser")
            power_metrics.append(m)
        agg_power = compute_aggregate(power_metrics)

        print(f"  Measuring archetype frequency...")
        arch_freq = measure_archetype_frequency(pool, primary_index, 1000)

        print_model_results(model_name, agg_committed, arch_freq, pstats)

        passes = count_passes(agg_committed, arch_freq)
        print(f"\n  TARGETS PASSED: {passes}/9")

        all_results[model_name] = {
            "agg_committed": agg_committed,
            "agg_power": agg_power,
            "arch_freq": arch_freq,
            "pool_stats": pstats,
            "passes": passes,
        }

    # ─── Summary Comparison Table ────────────────────────────────────────
    print("\n\n" + "=" * 70)
    print("  SUMMARY COMPARISON TABLE")
    print("=" * 70)

    short_names = ["A:Flat", "B:TCG", "C:Rogue", "D:SymCor", "E:InvCor"]
    model_keys = list(models.keys())

    print(f"\n  {'Metric':<38} {'Target':<10}", end="")
    for sn in short_names:
        print(f" {sn:>10}", end="")
    print()
    print(f"  {'-'*38} {'-'*10}", end="")
    for _ in short_names:
        print(f" {'-'*10}", end="")
    print()

    def get_c(r, key):
        return r["agg_committed"][key]

    metric_rows = [
        ("Early unique archs w/ S/A", ">= 3",
         lambda r: f"{get_c(r, 'early_unique_archs'):.2f}"),
        ("Early S/A for arch", "<= 2",
         lambda r: f"{get_c(r, 'early_arch_fit'):.2f}"),
        ("Late S/A for arch", ">= 2",
         lambda r: f"{get_c(r, 'late_arch_fit'):.2f}"),
        ("Late C/F cards", ">= 0.5",
         lambda r: f"{get_c(r, 'late_off_arch'):.2f}"),
        ("Convergence pick", "5-8",
         lambda r: f"{get_c(r, 'convergence_pick'):.1f}"),
        ("Deck concentration", "60-90%",
         lambda r: f"{get_c(r, 'deck_concentration')*100:.1f}%"),
        ("Card overlap", "< 40%",
         lambda r: f"{get_c(r, 'card_overlap')*100:.1f}%"),
        ("Late S/A stddev", ">= 0.8",
         lambda r: f"{get_c(r, 'late_sa_stddev'):.2f}"),
        ("Archetype freq range", "5-20%",
         lambda r: f"{min(r['arch_freq'].values())*100:.0f}-{max(r['arch_freq'].values())*100:.0f}%"
         if r['arch_freq'] else "N/A"),
        ("Targets passed", "9/9",
         lambda r: f"{r['passes']}/9"),
        ("---", "---", lambda r: "---"),
        ("Avg tokens per pick", "",
         lambda r: f"{get_c(r, 'avg_tokens_per_pick'):.2f}"),
        ("Late spend frequency", "",
         lambda r: f"{get_c(r, 'late_spend_frequency')*100:.1f}%"),
        ("Tension (spend packs)", "",
         lambda r: f"{get_c(r, 'tension_rate_spend'):.1f}%"),
        ("Tension (non-spend packs)", "",
         lambda r: f"{get_c(r, 'tension_rate_non_spend'):.1f}%"),
        ("Tension (overall)", "",
         lambda r: f"{get_c(r, 'tension_rate_overall'):.1f}%"),
        ("Avg tokens from R/L picks", "",
         lambda r: f"{get_c(r, 'avg_tokens_from_rares'):.2f}"),
        ("Avg tokens from C/U picks", "",
         lambda r: f"{get_c(r, 'avg_tokens_from_commons'):.2f}"),
        ("Avg power (committed)", "",
         lambda r: f"{get_c(r, 'avg_power'):.2f}"),
        ("Power stdev across runs", "",
         lambda r: f"{get_c(r, 'power_stdev'):.3f}"),
        ("Avg power (power chaser)", "",
         lambda r: f"{r['agg_power']['avg_power']:.2f}"),
        ("Power gap (chaser - committed)", "",
         lambda r: f"{r['agg_power']['avg_power'] - get_c(r, 'avg_power'):.2f}"),
    ]

    for name, target, fn in metric_rows:
        print(f"  {name:<38} {target:<10}", end="")
        for mk in model_keys:
            val = fn(all_results[mk])
            print(f" {val:>10}", end="")
        print()

    print("\n  Done.")


if __name__ == "__main__":
    main()
