#!/usr/bin/env python3
"""
Pool Evolution / Top-Pair Pool Seeding — V5 Round 3 Simulation (Agent 3)

Algorithm (one-sentence): "After each pick, if you have drafted 2+ cards with
the same ordered resonance pair, 4 cards matching your most-drafted pair are
added to the pool from a reserve."

Includes:
  - Full card pool generation (360 cards, 8 archetypes + generics)
  - Top-Pair Pool Seeding algorithm with reserve injection
  - V3 Lane Locking baseline
  - V4 Pack Widening baseline (auto-spend variant)
  - 3 player strategies (archetype-committed, power-chaser, signal-reader)
  - 1000 drafts x 30 picks x 3 strategies
  - All 8 measurable targets at ARCHETYPE level
  - Variance target (stddev of S/A per pack, picks 6+)
  - Per-archetype convergence table
  - Parameter sensitivity sweeps
  - Escalating Pair Injection variant
  - Symbol distribution sensitivity (15% vs 30% 1-symbol)
  - 3 detailed draft traces
  - Pool size progression tracking
"""

import random
import statistics
import math
from dataclasses import dataclass, field
from enum import Enum
from collections import defaultdict, Counter
from typing import Optional

# ============================================================================
# Core Types
# ============================================================================

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

# The 8 archetypes on a circle (name, primary, secondary)
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

def circle_distance(i, j):
    d = abs(i - j)
    return min(d, NUM_ARCHETYPES - d)

def compute_fitness(card_arch_idx, player_arch_idx):
    """Compute fitness tier of a card for a player targeting a specific archetype."""
    if card_arch_idx == -1:  # generic
        return Tier.B
    if card_arch_idx == player_arch_idx:
        return Tier.S

    dist = circle_distance(card_arch_idx, player_arch_idx)
    card_primary = ARCHETYPES[card_arch_idx][1]
    player_primary = ARCHETYPES[player_arch_idx][1]
    card_secondary = ARCHETYPES[card_arch_idx][2]
    player_secondary = ARCHETYPES[player_arch_idx][2]

    if dist == 1:
        if card_primary == player_primary:
            return Tier.A
        return Tier.B
    elif dist == 2:
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
    symbols: list           # list of Resonance, 0-3 elements
    archetype_idx: int      # index into ARCHETYPES, -1 for generic
    power: float            # raw card strength 0-10
    fitness: dict = field(default_factory=dict)  # arch_idx -> Tier

    @property
    def primary_resonance(self):
        return self.symbols[0] if self.symbols else None

    @property
    def secondary_resonance(self):
        return self.symbols[1] if len(self.symbols) >= 2 else None

    @property
    def ordered_pair(self):
        """The (primary, secondary) pair, or None if < 2 symbols."""
        if len(self.symbols) >= 2:
            return (self.symbols[0], self.symbols[1])
        return None

    def is_sa_for(self, arch_idx):
        return self.fitness.get(arch_idx, Tier.F) in (Tier.S, Tier.A)

    def is_cf_for(self, arch_idx):
        return self.fitness.get(arch_idx, Tier.F) in (Tier.C, Tier.F)

# ============================================================================
# Card Pool Construction
# ============================================================================

def generate_card_pool(one_sym_pct=0.15, two_sym_pct=0.65, three_sym_pct=0.20,
                       seed=None):
    """Generate 360 cards: ~40 per archetype (320) + 36 generic."""
    if seed is not None:
        random.seed(seed)

    cards = []
    card_id = 0
    cards_per_archetype = 40
    num_generic = 36

    for arch_idx in range(NUM_ARCHETYPES):
        name, primary, secondary = ARCHETYPES[arch_idx]
        n = cards_per_archetype

        n1 = round(n * one_sym_pct)
        n3 = round(n * three_sym_pct)
        n2 = n - n1 - n3

        for _ in range(n1):
            # 1-symbol: just primary
            symbols = [primary]
            c = SimCard(id=card_id, symbols=symbols, archetype_idx=arch_idx,
                        power=random.uniform(3, 8))
            cards.append(c)
            card_id += 1

        for _ in range(n2):
            # 2-symbol: (primary, secondary)
            symbols = [primary, secondary]
            c = SimCard(id=card_id, symbols=symbols, archetype_idx=arch_idx,
                        power=random.uniform(3, 8))
            cards.append(c)
            card_id += 1

        for _ in range(n3):
            # 3-symbol: (primary, secondary, primary) or (primary, primary, secondary)
            if random.random() < 0.5:
                symbols = [primary, secondary, primary]
            else:
                symbols = [primary, primary, secondary]
            c = SimCard(id=card_id, symbols=symbols, archetype_idx=arch_idx,
                        power=random.uniform(3, 8))
            cards.append(c)
            card_id += 1

    # Generic cards
    for _ in range(num_generic):
        c = SimCard(id=card_id, symbols=[], archetype_idx=-1,
                    power=random.uniform(4, 9))
        cards.append(c)
        card_id += 1

    # Compute fitness for all cards
    for c in cards:
        for arch_idx in range(NUM_ARCHETYPES):
            c.fitness[arch_idx] = compute_fitness(c.archetype_idx, arch_idx)

    return cards

def generate_reserve_pool(main_pool, reserve_multiplier=3, seed=None):
    """Generate reserve cards matching each ordered pair for injection.

    Creates reserve_multiplier * 40 cards per archetype pair in the reserve.
    """
    if seed is not None:
        random.seed(seed)

    reserve = defaultdict(list)  # pair -> list of SimCards
    card_id = max(c.id for c in main_pool) + 1

    for arch_idx in range(NUM_ARCHETYPES):
        name, primary, secondary = ARCHETYPES[arch_idx]
        pair = (primary, secondary)
        n_reserve = reserve_multiplier * 40  # plenty of reserve cards

        for _ in range(n_reserve):
            # 2-symbol cards matching the pair
            symbols = [primary, secondary]
            c = SimCard(id=card_id, symbols=symbols, archetype_idx=arch_idx,
                        power=random.uniform(3, 8))
            for aidx in range(NUM_ARCHETYPES):
                c.fitness[aidx] = compute_fitness(arch_idx, aidx)
            reserve[pair].append(c)
            card_id += 1

    return reserve

# ============================================================================
# Player Strategies
# ============================================================================

def get_symbol_counts(drafted):
    """Return weighted symbol counts from drafted cards."""
    counts = defaultdict(float)
    for c in drafted:
        for i, sym in enumerate(c.symbols):
            weight = 2.0 if i == 0 else 1.0
            counts[sym] += weight
    return counts

def get_pair_counts(drafted):
    """Return pair counts from drafted cards (2+ symbol cards only)."""
    counts = Counter()
    for c in drafted:
        pair = c.ordered_pair
        if pair is not None:
            counts[pair] += 1
    return counts

def get_top_pair(drafted):
    """Return the most-drafted ordered pair, or None."""
    counts = get_pair_counts(drafted)
    if not counts:
        return None, 0
    top = counts.most_common(1)[0]
    return top[0], top[1]

def determine_archetype(drafted, strategy="committed"):
    """Determine the player's target archetype based on drafted cards."""
    if not drafted:
        return None

    arch_scores = defaultdict(float)
    for c in drafted:
        if c.archetype_idx >= 0:
            arch_scores[c.archetype_idx] += 1.0
        pair = c.ordered_pair
        if pair:
            for aidx, (_, pri, sec) in enumerate(ARCHETYPES):
                if pair == (pri, sec):
                    arch_scores[aidx] += 0.5

    if not arch_scores:
        return random.randint(0, 7)
    return max(arch_scores, key=arch_scores.get)

def pick_card_committed(pack, drafted, target_arch):
    """Archetype-committed: picks best fitness for target archetype."""
    if target_arch is None:
        # Early: pick highest power
        return max(range(len(pack)), key=lambda i: pack[i].power)

    def score(c):
        tier = c.fitness.get(target_arch, Tier.F)
        tier_score = {Tier.S: 5, Tier.A: 4, Tier.B: 2, Tier.C: 1, Tier.F: 0}
        return tier_score[tier] * 10 + c.power

    return max(range(len(pack)), key=lambda i: score(pack[i]))

def pick_card_power(pack, drafted, target_arch):
    """Power-chaser: picks highest raw power."""
    return max(range(len(pack)), key=lambda i: pack[i].power)

def pick_card_signal(pack, drafted, target_arch):
    """Signal-reader: evaluates which archetype is open based on what's seen."""
    if len(drafted) < 5:
        # Early: pick highest power but lean toward open archetypes
        return max(range(len(pack)), key=lambda i: pack[i].power)

    if target_arch is not None:
        return pick_card_committed(pack, drafted, target_arch)
    return max(range(len(pack)), key=lambda i: pack[i].power)


# ============================================================================
# Draft Algorithms
# ============================================================================

def draft_pool_seeding(pool, reserve, pick_fn, num_picks=30,
                       injection_rate=4, activation_threshold=2,
                       escalating=False, remove_per_pick=0,
                       trace=False):
    """
    Top-Pair Pool Seeding algorithm.

    After each pick, if the player has drafted 2+ cards with the same ordered
    resonance pair, `injection_rate` cards matching the most-drafted pair are
    added to the pool from a reserve.

    If escalating=True, inject min(top_pair_count, 5) instead of fixed rate.
    """
    active_pool = list(pool)  # mutable copy
    drafted = []
    target_arch = None
    trace_log = []
    pool_sizes = [len(active_pool)]
    pack_results = []  # list of (pick_num, pack, chosen_card)

    for pick_num in range(1, num_picks + 1):
        # Determine target archetype (committed around pick 5-6)
        if pick_num >= 5 and target_arch is None:
            target_arch = determine_archetype(drafted)
        elif pick_num >= 3 and target_arch is None:
            # Signal-reader commits earlier if clear signal
            pair_counts = get_pair_counts(drafted)
            if pair_counts and pair_counts.most_common(1)[0][1] >= 2:
                target_arch = determine_archetype(drafted)

        # Generate pack: draw 4 random cards from active pool
        if len(active_pool) < 4:
            # Shouldn't happen but safety
            pack_indices = list(range(len(active_pool)))
        else:
            pack_indices = random.sample(range(len(active_pool)), 4)

        pack = [active_pool[i] for i in pack_indices]

        # Player picks
        choice_idx = pick_fn(pack, drafted, target_arch)
        chosen = pack[choice_idx]
        drafted.append(chosen)

        # Remove chosen card from pool
        pool_idx = pack_indices[choice_idx]
        active_pool.pop(pool_idx)

        # --- Pool Seeding Logic ---
        top_pair, top_count = get_top_pair(drafted)
        if top_pair and top_count >= activation_threshold:
            # Determine injection count
            if escalating:
                inject_count = min(top_count, 5)
            else:
                inject_count = injection_rate

            # Add cards from reserve
            available_reserve = reserve.get(top_pair, [])
            if available_reserve:
                to_inject = random.sample(available_reserve,
                                          min(inject_count, len(available_reserve)))
                active_pool.extend(to_inject)

            # Optional removal of off-pair cards
            if remove_per_pick > 0 and len(active_pool) > 100:
                off_pair_indices = [
                    i for i, c in enumerate(active_pool)
                    if c.ordered_pair and c.ordered_pair != top_pair
                ]
                if off_pair_indices:
                    to_remove = random.sample(off_pair_indices,
                                              min(remove_per_pick, len(off_pair_indices)))
                    for idx in sorted(to_remove, reverse=True):
                        active_pool.pop(idx)

        pool_sizes.append(len(active_pool))

        if trace:
            sa_count = sum(1 for c in pack if target_arch is not None and c.is_sa_for(target_arch))
            trace_log.append({
                'pick': pick_num,
                'pack': [(c.id, [s.value for s in c.symbols],
                          ARCHETYPE_NAMES[c.archetype_idx] if c.archetype_idx >= 0 else "Generic",
                          c.fitness.get(target_arch, Tier.F).value if target_arch is not None else "?")
                         for c in pack],
                'chosen': (chosen.id,
                           ARCHETYPE_NAMES[chosen.archetype_idx] if chosen.archetype_idx >= 0 else "Generic"),
                'target_arch': ARCHETYPE_NAMES[target_arch] if target_arch is not None else "None",
                'top_pair': (top_pair[0].value, top_pair[1].value) if top_pair else None,
                'top_pair_count': top_count,
                'pool_size': len(active_pool),
                'sa_in_pack': sa_count,
            })

        pack_results.append((pick_num, pack, chosen, target_arch))

    return drafted, target_arch, pack_results, pool_sizes, trace_log


def draft_lane_locking(pool, pick_fn, num_picks=30, threshold1=3, threshold2=8,
                       trace=False):
    """
    V3 Lane Locking baseline.
    At symbol count threshold1 in a resonance, 1 slot locks to that resonance.
    At threshold2, a second slot locks. Primary symbol = 2 weight.
    """
    active_pool = list(pool)
    drafted = []
    target_arch = None
    locked_slots = []  # list of Resonance
    trace_log = []
    pack_results = []

    for pick_num in range(1, num_picks + 1):
        if pick_num >= 5 and target_arch is None:
            target_arch = determine_archetype(drafted)

        # Build pack
        pack = []
        sym_counts = get_symbol_counts(drafted)

        # Update locks
        locked_slots = []
        for res in Resonance:
            count = sym_counts.get(res, 0)
            if count >= threshold2:
                locked_slots.append(res)
                locked_slots.append(res)
            elif count >= threshold1:
                locked_slots.append(res)

        # Cap at 4 locks
        locked_slots = locked_slots[:4]

        # Fill locked slots
        for res in locked_slots:
            matching = [c for c in active_pool if c.primary_resonance == res]
            if matching:
                pack.append(random.choice(matching))
            else:
                pack.append(random.choice(active_pool))

        # Fill remaining slots randomly
        remaining = 4 - len(pack)
        for _ in range(remaining):
            pack.append(random.choice(active_pool))

        # Player picks
        choice_idx = pick_fn(pack, drafted, target_arch)
        chosen = pack[choice_idx]
        drafted.append(chosen)

        # Remove from pool
        try:
            active_pool.remove(chosen)
        except ValueError:
            pass  # Reserve card not in pool

        pack_results.append((pick_num, pack, chosen, target_arch))

        if trace:
            sa_count = sum(1 for c in pack if target_arch is not None and c.is_sa_for(target_arch))
            trace_log.append({
                'pick': pick_num,
                'locked_slots': len(locked_slots),
                'sa_in_pack': sa_count,
                'target_arch': ARCHETYPE_NAMES[target_arch] if target_arch is not None else "None",
            })

    return drafted, target_arch, pack_results, trace_log


def draft_pack_widening(pool, pick_fn, num_picks=30, cost=3, bonus_count=1,
                        trace=False):
    """
    V4 Pack Widening baseline (auto-spend variant).
    Each symbol drafted adds tokens (primary=2, secondary/tertiary=1).
    When the highest resonance reaches `cost` tokens, auto-spend: add
    `bonus_count` cards of that resonance to the next pack, deduct tokens.
    """
    active_pool = list(pool)
    drafted = []
    target_arch = None
    tokens = defaultdict(float)
    trace_log = []
    pack_results = []

    for pick_num in range(1, num_picks + 1):
        if pick_num >= 5 and target_arch is None:
            target_arch = determine_archetype(drafted)

        # Check if bonus fires
        bonus_cards = []
        if tokens:
            top_res = max(tokens, key=tokens.get)
            if tokens[top_res] >= cost:
                tokens[top_res] -= cost
                matching = [c for c in active_pool if c.primary_resonance == top_res]
                for _ in range(bonus_count):
                    if matching:
                        bonus_cards.append(random.choice(matching))

        # Build pack: 4 random + bonus
        base_pack = [random.choice(active_pool) for _ in range(4)]
        pack = base_pack + bonus_cards

        # Player picks
        choice_idx = pick_fn(pack, drafted, target_arch)
        chosen = pack[choice_idx]
        drafted.append(chosen)

        # Add tokens from drafted card
        for i, sym in enumerate(chosen.symbols):
            weight = 2.0 if i == 0 else 1.0
            tokens[sym] += weight

        # Remove from pool
        try:
            active_pool.remove(chosen)
        except ValueError:
            pass

        pack_results.append((pick_num, pack, chosen, target_arch))

        if trace:
            sa_count = sum(1 for c in pack if target_arch is not None and c.is_sa_for(target_arch))
            trace_log.append({
                'pick': pick_num,
                'bonus_count': len(bonus_cards),
                'sa_in_pack': sa_count,
                'target_arch': ARCHETYPE_NAMES[target_arch] if target_arch is not None else "None",
                'tokens': dict(tokens),
            })

    return drafted, target_arch, pack_results, trace_log


# ============================================================================
# Metrics Computation
# ============================================================================

def compute_metrics(pack_results, drafted, target_arch):
    """Compute all 8 metrics + variance at ARCHETYPE level."""
    metrics = {}

    # Target archetype — use the one determined by the draft
    if target_arch is None:
        target_arch = determine_archetype(drafted)
    if target_arch is None:
        return None

    # Picks 1-5 metrics
    early_packs = [(pn, pack, chosen, ta) for pn, pack, chosen, ta in pack_results if pn <= 5]
    late_packs = [(pn, pack, chosen, ta) for pn, pack, chosen, ta in pack_results if pn >= 6]

    # 1. Picks 1-5: unique archetypes with S/A cards per pack
    early_arch_diversity = []
    for pn, pack, chosen, ta in early_packs:
        archetypes_with_sa = set()
        for c in pack[:4]:  # base pack only
            for aidx in range(NUM_ARCHETYPES):
                if c.is_sa_for(aidx):
                    archetypes_with_sa.add(aidx)
        early_arch_diversity.append(len(archetypes_with_sa))
    metrics['early_arch_diversity'] = statistics.mean(early_arch_diversity) if early_arch_diversity else 0

    # 2. Picks 1-5: S/A cards for player's emerging archetype per pack
    early_sa_for_target = []
    for pn, pack, chosen, ta in early_packs:
        sa = sum(1 for c in pack[:4] if c.is_sa_for(target_arch))
        early_sa_for_target.append(sa)
    metrics['early_sa_target'] = statistics.mean(early_sa_for_target) if early_sa_for_target else 0

    # 3. Picks 6+: S/A cards for committed archetype per pack
    late_sa = []
    for pn, pack, chosen, ta in late_packs:
        sa = sum(1 for c in pack[:4] if c.is_sa_for(target_arch))
        late_sa.append(sa)
    metrics['late_sa_target'] = statistics.mean(late_sa) if late_sa else 0

    # Also count total pack size SA (including bonus if any)
    late_sa_full = []
    for pn, pack, chosen, ta in late_packs:
        sa = sum(1 for c in pack if c.is_sa_for(target_arch))
        late_sa_full.append(sa)
    metrics['late_sa_target_full'] = statistics.mean(late_sa_full) if late_sa_full else 0

    # 4. Picks 6+: off-archetype (C/F) cards per pack
    late_cf = []
    for pn, pack, chosen, ta in late_packs:
        cf = sum(1 for c in pack[:4] if c.is_cf_for(target_arch))
        late_cf.append(cf)
    metrics['late_cf'] = statistics.mean(late_cf) if late_cf else 0

    # 5. Convergence pick (when 2+ S/A becomes regular)
    convergence_pick = 30  # default: never converges
    running_hits = 0
    for pn, pack, chosen, ta in pack_results:
        sa = sum(1 for c in pack[:4] if c.is_sa_for(target_arch))
        if sa >= 2:
            running_hits += 1
        else:
            running_hits = 0
        if running_hits >= 3:  # 3 consecutive packs with 2+ S/A
            convergence_pick = pn - 2
            break
    metrics['convergence_pick'] = convergence_pick

    # 6. Deck archetype concentration
    sa_drafted = sum(1 for c in drafted if c.is_sa_for(target_arch))
    metrics['deck_concentration'] = sa_drafted / len(drafted) if drafted else 0

    # 7. Variance: stddev of S/A per pack (picks 6+)
    if late_sa:
        metrics['sa_stddev'] = statistics.stdev(late_sa) if len(late_sa) > 1 else 0
    else:
        metrics['sa_stddev'] = 0

    # Distribution of S/A per pack
    sa_dist = Counter(late_sa)
    metrics['sa_distribution'] = sa_dist

    return metrics


def compute_card_overlap(deck1, deck2):
    """Fraction of shared cards between two decks."""
    ids1 = set(c.id for c in deck1)
    ids2 = set(c.id for c in deck2)
    if not ids1 or not ids2:
        return 0
    return len(ids1 & ids2) / max(len(ids1), len(ids2))


# ============================================================================
# Simulation Runners
# ============================================================================

def run_simulation(algorithm, pool, reserve, num_runs=1000, num_picks=30,
                   strategy="committed", algo_params=None, trace_runs=None):
    """Run num_runs drafts and aggregate metrics."""
    if algo_params is None:
        algo_params = {}
    if trace_runs is None:
        trace_runs = set()

    all_metrics = []
    all_decks = []
    arch_frequency = Counter()
    all_pool_sizes = []
    traces = {}

    for run in range(num_runs):
        do_trace = run in trace_runs

        if strategy == "committed":
            pick_fn = pick_card_committed
        elif strategy == "power":
            pick_fn = pick_card_power
        elif strategy == "signal":
            pick_fn = pick_card_signal
        else:
            pick_fn = pick_card_committed

        if algorithm == "pool_seeding":
            drafted, target_arch, pack_results, pool_sizes, trace_log = draft_pool_seeding(
                pool, reserve, pick_fn, num_picks,
                injection_rate=algo_params.get('injection_rate', 4),
                activation_threshold=algo_params.get('activation_threshold', 2),
                escalating=algo_params.get('escalating', False),
                remove_per_pick=algo_params.get('remove_per_pick', 0),
                trace=do_trace)
            all_pool_sizes.append(pool_sizes)
        elif algorithm == "lane_locking":
            drafted, target_arch, pack_results, trace_log = draft_lane_locking(
                pool, pick_fn, num_picks,
                threshold1=algo_params.get('threshold1', 3),
                threshold2=algo_params.get('threshold2', 8),
                trace=do_trace)
            all_pool_sizes.append([len(pool)] * (num_picks + 1))
        elif algorithm == "pack_widening":
            drafted, target_arch, pack_results, trace_log = draft_pack_widening(
                pool, pick_fn, num_picks,
                cost=algo_params.get('cost', 3),
                bonus_count=algo_params.get('bonus_count', 1),
                trace=do_trace)
            all_pool_sizes.append([len(pool)] * (num_picks + 1))
        else:
            raise ValueError(f"Unknown algorithm: {algorithm}")

        m = compute_metrics(pack_results, drafted, target_arch)
        if m is not None:
            all_metrics.append(m)

        all_decks.append(drafted)
        if target_arch is not None:
            arch_frequency[target_arch] += 1

        if do_trace:
            traces[run] = trace_log

    return all_metrics, all_decks, arch_frequency, all_pool_sizes, traces


def aggregate_metrics(all_metrics):
    """Aggregate metrics across multiple runs."""
    if not all_metrics:
        return {}

    agg = {}
    keys = ['early_arch_diversity', 'early_sa_target', 'late_sa_target',
            'late_sa_target_full', 'late_cf', 'convergence_pick',
            'deck_concentration', 'sa_stddev']

    for key in keys:
        values = [m[key] for m in all_metrics if key in m]
        if values:
            agg[key] = statistics.mean(values)
        else:
            agg[key] = 0

    # Aggregate SA distribution
    total_dist = Counter()
    for m in all_metrics:
        if 'sa_distribution' in m:
            for k, v in m['sa_distribution'].items():
                total_dist[k] += v
    total = sum(total_dist.values())
    if total > 0:
        agg['sa_distribution'] = {k: v / total for k, v in sorted(total_dist.items())}
    else:
        agg['sa_distribution'] = {}

    return agg


def compute_overlap_metric(all_decks, sample_size=200):
    """Compute average pairwise card overlap for run-to-run variety."""
    if len(all_decks) < 2:
        return 0
    pairs = min(sample_size, len(all_decks) * (len(all_decks) - 1) // 2)
    overlaps = []
    indices = list(range(len(all_decks)))
    for _ in range(pairs):
        i, j = random.sample(indices, 2)
        overlaps.append(compute_card_overlap(all_decks[i], all_decks[j]))
    return statistics.mean(overlaps)


def per_archetype_convergence(algorithm, pool, reserve, num_runs=200,
                              algo_params=None):
    """Run convergence test for each archetype individually."""
    if algo_params is None:
        algo_params = {}

    results = {}
    for arch_idx in range(NUM_ARCHETYPES):
        convergence_picks = []

        for _ in range(num_runs):
            # Force target archetype
            target_arch = arch_idx

            def pick_fn_forced(pack, drafted, _target):
                return pick_card_committed(pack, drafted, target_arch)

            if algorithm == "pool_seeding":
                drafted, _, pack_results, _, _ = draft_pool_seeding(
                    pool, reserve, pick_fn_forced, 30,
                    injection_rate=algo_params.get('injection_rate', 4),
                    activation_threshold=algo_params.get('activation_threshold', 2),
                    escalating=algo_params.get('escalating', False),
                    remove_per_pick=algo_params.get('remove_per_pick', 0))
            elif algorithm == "lane_locking":
                drafted, _, pack_results, _ = draft_lane_locking(
                    pool, pick_fn_forced, 30,
                    threshold1=algo_params.get('threshold1', 3),
                    threshold2=algo_params.get('threshold2', 8))
            elif algorithm == "pack_widening":
                drafted, _, pack_results, _ = draft_pack_widening(
                    pool, pick_fn_forced, 30,
                    cost=algo_params.get('cost', 3),
                    bonus_count=algo_params.get('bonus_count', 1))
            else:
                raise ValueError(f"Unknown algorithm: {algorithm}")

            # Find convergence pick for this archetype
            conv_pick = 30
            running_hits = 0
            for pn, pack, chosen, ta in pack_results:
                sa = sum(1 for c in pack[:4] if c.is_sa_for(arch_idx))
                if sa >= 2:
                    running_hits += 1
                else:
                    running_hits = 0
                if running_hits >= 3:
                    conv_pick = pn - 2
                    break
            convergence_picks.append(conv_pick)

        results[ARCHETYPE_NAMES[arch_idx]] = statistics.mean(convergence_picks)

    return results


# ============================================================================
# Pair Precision Validation
# ============================================================================

def validate_pair_precision(pool):
    """Check what fraction of cards with ordered pairs map to their home archetype."""
    pair_cards = [c for c in pool if c.ordered_pair is not None]
    correct = 0
    for c in pair_cards:
        pair = c.ordered_pair
        # Check if pair matches card's home archetype
        if c.archetype_idx >= 0:
            _, pri, sec = ARCHETYPES[c.archetype_idx]
            if pair == (pri, sec):
                correct += 1
    precision = correct / len(pair_cards) if pair_cards else 0
    print(f"Pair precision: {precision:.2%} ({correct}/{len(pair_cards)} cards with pairs)")

    # Check S-tier precision: for pair-matched cards, what % are S-tier for
    # the archetype identified by that pair?
    s_correct = 0
    for c in pair_cards:
        pair = c.ordered_pair
        # Find which archetype this pair maps to
        for aidx, (_, pri, sec) in enumerate(ARCHETYPES):
            if pair == (pri, sec):
                if c.fitness.get(aidx, Tier.F) == Tier.S:
                    s_correct += 1
                break
    s_precision = s_correct / len(pair_cards) if pair_cards else 0
    print(f"S-tier precision for pair-matched: {s_precision:.2%}")

    # SA precision
    sa_correct = 0
    for c in pair_cards:
        pair = c.ordered_pair
        for aidx, (_, pri, sec) in enumerate(ARCHETYPES):
            if pair == (pri, sec):
                if c.fitness.get(aidx, Tier.F) in (Tier.S, Tier.A):
                    sa_correct += 1
                break
    sa_precision = sa_correct / len(pair_cards) if pair_cards else 0
    print(f"S/A-tier precision for pair-matched: {sa_precision:.2%}")

    return precision, s_precision, sa_precision


# ============================================================================
# Main Simulation
# ============================================================================

def main():
    random.seed(42)

    print("=" * 80)
    print("TOP-PAIR POOL SEEDING — V5 Round 3 Simulation (Agent 3)")
    print("=" * 80)
    print()

    # Generate card pool
    print("--- Card Pool Generation ---")
    pool = generate_card_pool(one_sym_pct=0.15, two_sym_pct=0.65,
                              three_sym_pct=0.20, seed=42)
    reserve = generate_reserve_pool(pool, reserve_multiplier=3, seed=43)

    # Pool statistics
    sym_counts = Counter(len(c.symbols) for c in pool)
    print(f"Total cards: {len(pool)}")
    print(f"Symbol distribution: 0-sym={sym_counts[0]}, 1-sym={sym_counts[1]}, "
          f"2-sym={sym_counts[2]}, 3-sym={sym_counts[3]}")
    print(f"Per-archetype: ~{(len(pool) - sym_counts[0]) // 8}")

    # Count S/A cards baseline
    for arch_idx in range(NUM_ARCHETYPES):
        sa = sum(1 for c in pool if c.is_sa_for(arch_idx))
        print(f"  {ARCHETYPE_NAMES[arch_idx]}: {sa} S/A cards ({sa/len(pool):.1%})")

    print()

    # Validate pair precision
    print("--- Pair Precision Validation ---")
    validate_pair_precision(pool)
    print()

    # ========================================================================
    # Main algorithm: Top-Pair Pool Seeding (injection_rate=4, threshold=2)
    # ========================================================================
    print("=" * 80)
    print("MAIN: Top-Pair Pool Seeding (rate=4, threshold=2)")
    print("=" * 80)

    for strategy in ["committed", "power", "signal"]:
        print(f"\n--- Strategy: {strategy} ---")
        metrics_list, decks, arch_freq, pool_sizes, traces = run_simulation(
            "pool_seeding", pool, reserve, num_runs=1000,
            strategy=strategy,
            algo_params={'injection_rate': 4, 'activation_threshold': 2},
            trace_runs={0, 1, 2} if strategy == "committed" else set())

        agg = aggregate_metrics(metrics_list)
        overlap = compute_overlap_metric(decks)

        print(f"  Picks 1-5 unique archetypes w/ S/A per pack: {agg['early_arch_diversity']:.2f}")
        print(f"  Picks 1-5 S/A for target per pack: {agg['early_sa_target']:.2f}")
        print(f"  Picks 6+ S/A for target per pack (base 4): {agg['late_sa_target']:.2f}")
        print(f"  Picks 6+ S/A for target per pack (full): {agg['late_sa_target_full']:.2f}")
        print(f"  Picks 6+ off-archetype (C/F) per pack: {agg['late_cf']:.2f}")
        print(f"  Convergence pick: {agg['convergence_pick']:.1f}")
        print(f"  Deck concentration: {agg['deck_concentration']:.1%}")
        print(f"  S/A stddev (picks 6+): {agg['sa_stddev']:.2f}")
        print(f"  S/A distribution: {agg.get('sa_distribution', {})}")
        print(f"  Run-to-run overlap: {overlap:.1%}")
        print(f"  Archetype frequency: {dict(arch_freq)}")

        # Pool size progression
        if pool_sizes:
            avg_pool = [statistics.mean(ps[i] for ps in pool_sizes if i < len(ps))
                        for i in range(31)]
            print(f"  Pool size (start): {avg_pool[0]:.0f}")
            print(f"  Pool size (pick 10): {avg_pool[10]:.0f}")
            print(f"  Pool size (pick 20): {avg_pool[20]:.0f}")
            print(f"  Pool size (pick 30): {avg_pool[30]:.0f}")

        if traces:
            for run_id, trace_log in traces.items():
                print(f"\n  --- Draft Trace (run {run_id}) ---")
                for entry in trace_log[:15]:  # first 15 picks
                    print(f"    Pick {entry['pick']}: target={entry['target_arch']}, "
                          f"pair={entry['top_pair']}(x{entry['top_pair_count']}), "
                          f"SA={entry['sa_in_pack']}, pool={entry['pool_size']}")
                    for card_info in entry['pack']:
                        print(f"      Card {card_info[0]}: {card_info[1]} "
                              f"({card_info[2]}) -> {card_info[3]}")
                    print(f"      -> Chose: {entry['chosen']}")
                print()

    # ========================================================================
    # Per-archetype convergence table
    # ========================================================================
    print("\n" + "=" * 80)
    print("PER-ARCHETYPE CONVERGENCE TABLE — Pool Seeding (rate=4)")
    print("=" * 80)
    arch_conv = per_archetype_convergence(
        "pool_seeding", pool, reserve, num_runs=200,
        algo_params={'injection_rate': 4, 'activation_threshold': 2})
    for name, pick in arch_conv.items():
        print(f"  {name:25s}: {pick:.1f}")

    # ========================================================================
    # V3 Lane Locking Baseline
    # ========================================================================
    print("\n" + "=" * 80)
    print("BASELINE: V3 Lane Locking (threshold 3/8)")
    print("=" * 80)

    for strategy in ["committed"]:
        print(f"\n--- Strategy: {strategy} ---")
        metrics_list, decks, arch_freq, _, _ = run_simulation(
            "lane_locking", pool, reserve, num_runs=1000,
            strategy=strategy,
            algo_params={'threshold1': 3, 'threshold2': 8})

        agg = aggregate_metrics(metrics_list)
        overlap = compute_overlap_metric(decks)

        print(f"  Picks 1-5 unique archetypes w/ S/A per pack: {agg['early_arch_diversity']:.2f}")
        print(f"  Picks 1-5 S/A for target per pack: {agg['early_sa_target']:.2f}")
        print(f"  Picks 6+ S/A for target per pack: {agg['late_sa_target']:.2f}")
        print(f"  Picks 6+ off-archetype (C/F) per pack: {agg['late_cf']:.2f}")
        print(f"  Convergence pick: {agg['convergence_pick']:.1f}")
        print(f"  Deck concentration: {agg['deck_concentration']:.1%}")
        print(f"  S/A stddev (picks 6+): {agg['sa_stddev']:.2f}")
        print(f"  S/A distribution: {agg.get('sa_distribution', {})}")
        print(f"  Run-to-run overlap: {overlap:.1%}")

    # Per-archetype convergence for Lane Locking
    print("\nPer-archetype convergence — Lane Locking:")
    ll_conv = per_archetype_convergence(
        "lane_locking", pool, reserve, num_runs=200,
        algo_params={'threshold1': 3, 'threshold2': 8})
    for name, pick in ll_conv.items():
        print(f"  {name:25s}: {pick:.1f}")

    # ========================================================================
    # V4 Pack Widening Baseline (auto-spend)
    # ========================================================================
    print("\n" + "=" * 80)
    print("BASELINE: V4 Pack Widening (cost=3, bonus=1, auto-spend)")
    print("=" * 80)

    for strategy in ["committed"]:
        print(f"\n--- Strategy: {strategy} ---")
        metrics_list, decks, arch_freq, _, _ = run_simulation(
            "pack_widening", pool, reserve, num_runs=1000,
            strategy=strategy,
            algo_params={'cost': 3, 'bonus_count': 1})

        agg = aggregate_metrics(metrics_list)
        overlap = compute_overlap_metric(decks)

        print(f"  Picks 1-5 unique archetypes w/ S/A per pack: {agg['early_arch_diversity']:.2f}")
        print(f"  Picks 1-5 S/A for target per pack: {agg['early_sa_target']:.2f}")
        print(f"  Picks 6+ S/A for target per pack: {agg['late_sa_target']:.2f}")
        print(f"  Picks 6+ off-archetype (C/F) per pack: {agg['late_cf']:.2f}")
        print(f"  Convergence pick: {agg['convergence_pick']:.1f}")
        print(f"  Deck concentration: {agg['deck_concentration']:.1%}")
        print(f"  S/A stddev (picks 6+): {agg['sa_stddev']:.2f}")
        print(f"  S/A distribution: {agg.get('sa_distribution', {})}")
        print(f"  Run-to-run overlap: {overlap:.1%}")

    # Per-archetype convergence for Pack Widening
    print("\nPer-archetype convergence — Pack Widening:")
    pw_conv = per_archetype_convergence(
        "pack_widening", pool, reserve, num_runs=200,
        algo_params={'cost': 3, 'bonus_count': 1})
    for name, pick in pw_conv.items():
        print(f"  {name:25s}: {pick:.1f}")

    # ========================================================================
    # Parameter Sensitivity: Injection Rate 3/4/5
    # ========================================================================
    print("\n" + "=" * 80)
    print("PARAMETER SENSITIVITY: Injection Rate 3/4/5")
    print("=" * 80)

    for rate in [3, 4, 5]:
        print(f"\n--- Injection Rate = {rate} ---")
        metrics_list, decks, _, pool_sizes, _ = run_simulation(
            "pool_seeding", pool, reserve, num_runs=500,
            strategy="committed",
            algo_params={'injection_rate': rate, 'activation_threshold': 2})

        agg = aggregate_metrics(metrics_list)
        overlap = compute_overlap_metric(decks)
        avg_pool_end = statistics.mean(ps[-1] for ps in pool_sizes)

        print(f"  Late S/A: {agg['late_sa_target']:.2f}, "
              f"Convergence: {agg['convergence_pick']:.1f}, "
              f"StdDev: {agg['sa_stddev']:.2f}, "
              f"Deck Conc: {agg['deck_concentration']:.1%}, "
              f"Overlap: {overlap:.1%}, "
              f"Pool@30: {avg_pool_end:.0f}")

    # ========================================================================
    # Parameter Sensitivity: With/Without Removal
    # ========================================================================
    print("\n" + "=" * 80)
    print("PARAMETER SENSITIVITY: With Removal (rate=4, remove=1)")
    print("=" * 80)

    metrics_list, decks, _, pool_sizes, _ = run_simulation(
        "pool_seeding", pool, reserve, num_runs=500,
        strategy="committed",
        algo_params={'injection_rate': 4, 'activation_threshold': 2,
                     'remove_per_pick': 1})

    agg = aggregate_metrics(metrics_list)
    overlap = compute_overlap_metric(decks)
    avg_pool_end = statistics.mean(ps[-1] for ps in pool_sizes)

    print(f"  Late S/A: {agg['late_sa_target']:.2f}, "
          f"Convergence: {agg['convergence_pick']:.1f}, "
          f"StdDev: {agg['sa_stddev']:.2f}, "
          f"Deck Conc: {agg['deck_concentration']:.1%}, "
          f"Overlap: {overlap:.1%}, "
          f"Pool@30: {avg_pool_end:.0f}")

    # With removal = 2
    print("\n--- With Removal (rate=4, remove=2) ---")
    metrics_list, decks, _, pool_sizes, _ = run_simulation(
        "pool_seeding", pool, reserve, num_runs=500,
        strategy="committed",
        algo_params={'injection_rate': 4, 'activation_threshold': 2,
                     'remove_per_pick': 2})

    agg = aggregate_metrics(metrics_list)
    overlap = compute_overlap_metric(decks)
    avg_pool_end = statistics.mean(ps[-1] for ps in pool_sizes)

    print(f"  Late S/A: {agg['late_sa_target']:.2f}, "
          f"Convergence: {agg['convergence_pick']:.1f}, "
          f"StdDev: {agg['sa_stddev']:.2f}, "
          f"Deck Conc: {agg['deck_concentration']:.1%}, "
          f"Overlap: {overlap:.1%}, "
          f"Pool@30: {avg_pool_end:.0f}")

    # ========================================================================
    # Escalating Pair Injection: inject min(pair_count, 5)
    # ========================================================================
    print("\n" + "=" * 80)
    print("ESCALATING PAIR INJECTION: inject min(pair_count, 5)")
    print("=" * 80)

    metrics_list, decks, _, pool_sizes, _ = run_simulation(
        "pool_seeding", pool, reserve, num_runs=500,
        strategy="committed",
        algo_params={'escalating': True, 'activation_threshold': 2})

    agg = aggregate_metrics(metrics_list)
    overlap = compute_overlap_metric(decks)
    avg_pool_end = statistics.mean(ps[-1] for ps in pool_sizes)

    print(f"  Late S/A: {agg['late_sa_target']:.2f}, "
          f"Convergence: {agg['convergence_pick']:.1f}, "
          f"StdDev: {agg['sa_stddev']:.2f}, "
          f"Deck Conc: {agg['deck_concentration']:.1%}, "
          f"Overlap: {overlap:.1%}, "
          f"Pool@30: {avg_pool_end:.0f}")

    # Per-archetype convergence for Escalating
    print("\nPer-archetype convergence — Escalating:")
    esc_conv = per_archetype_convergence(
        "pool_seeding", pool, reserve, num_runs=200,
        algo_params={'escalating': True, 'activation_threshold': 2})
    for name, pick in esc_conv.items():
        print(f"  {name:25s}: {pick:.1f}")

    # ========================================================================
    # Symbol Distribution Sensitivity: 30% 1-symbol
    # ========================================================================
    print("\n" + "=" * 80)
    print("SYMBOL SENSITIVITY: 30% 1-symbol (vs 15% baseline)")
    print("=" * 80)

    pool_30 = generate_card_pool(one_sym_pct=0.30, two_sym_pct=0.50,
                                  three_sym_pct=0.20, seed=44)
    reserve_30 = generate_reserve_pool(pool_30, seed=45)

    sym_counts_30 = Counter(len(c.symbols) for c in pool_30)
    print(f"Pool: 0-sym={sym_counts_30[0]}, 1-sym={sym_counts_30[1]}, "
          f"2-sym={sym_counts_30[2]}, 3-sym={sym_counts_30[3]}")

    print("\n--- Pool Seeding (rate=4) on 30% 1-sym pool ---")
    metrics_list, decks, _, pool_sizes, _ = run_simulation(
        "pool_seeding", pool_30, reserve_30, num_runs=500,
        strategy="committed",
        algo_params={'injection_rate': 4, 'activation_threshold': 2})

    agg = aggregate_metrics(metrics_list)
    overlap = compute_overlap_metric(decks)
    avg_pool_end = statistics.mean(ps[-1] for ps in pool_sizes)

    print(f"  Late S/A: {agg['late_sa_target']:.2f}, "
          f"Convergence: {agg['convergence_pick']:.1f}, "
          f"StdDev: {agg['sa_stddev']:.2f}, "
          f"Deck Conc: {agg['deck_concentration']:.1%}, "
          f"Overlap: {overlap:.1%}, "
          f"Pool@30: {avg_pool_end:.0f}")

    print("\n--- Lane Locking on 30% 1-sym pool ---")
    metrics_list_ll30, _, _, _, _ = run_simulation(
        "lane_locking", pool_30, reserve_30, num_runs=500,
        strategy="committed",
        algo_params={'threshold1': 3, 'threshold2': 8})
    agg_ll30 = aggregate_metrics(metrics_list_ll30)
    print(f"  Late S/A: {agg_ll30['late_sa_target']:.2f}, "
          f"Convergence: {agg_ll30['convergence_pick']:.1f}")

    # ========================================================================
    # Detailed Draft Traces (3 scenarios)
    # ========================================================================
    print("\n" + "=" * 80)
    print("DETAILED DRAFT TRACES")
    print("=" * 80)

    # Trace 1: Early committer (archetype-committed, commits by pick 5)
    print("\n--- Trace 1: Early Committer ---")
    random.seed(100)
    drafted_t1, ta_t1, pr_t1, ps_t1, tl_t1 = draft_pool_seeding(
        pool, reserve, pick_card_committed, 30,
        injection_rate=4, activation_threshold=2, trace=True)
    for entry in tl_t1:
        print(f"  Pick {entry['pick']:2d}: target={entry['target_arch']:15s} "
              f"pair={str(entry['top_pair']):20s}(x{entry['top_pair_count']}) "
              f"SA={entry['sa_in_pack']} pool={entry['pool_size']}")

    # Trace 2: Flexible player (power-chaser, stays flexible)
    print("\n--- Trace 2: Power Chaser (Flexible) ---")
    random.seed(200)
    drafted_t2, ta_t2, pr_t2, ps_t2, tl_t2 = draft_pool_seeding(
        pool, reserve, pick_card_power, 30,
        injection_rate=4, activation_threshold=2, trace=True)
    for entry in tl_t2:
        print(f"  Pick {entry['pick']:2d}: target={entry['target_arch']:15s} "
              f"pair={str(entry['top_pair']):20s}(x{entry['top_pair_count']}) "
              f"SA={entry['sa_in_pack']} pool={entry['pool_size']}")

    # Trace 3: Signal reader
    print("\n--- Trace 3: Signal Reader ---")
    random.seed(300)
    drafted_t3, ta_t3, pr_t3, ps_t3, tl_t3 = draft_pool_seeding(
        pool, reserve, pick_card_signal, 30,
        injection_rate=4, activation_threshold=2, trace=True)
    for entry in tl_t3:
        print(f"  Pick {entry['pick']:2d}: target={entry['target_arch']:15s} "
              f"pair={str(entry['top_pair']):20s}(x{entry['top_pair_count']}) "
              f"SA={entry['sa_in_pack']} pool={entry['pool_size']}")

    # Pool size progression summary
    print("\n--- Pool Size Progression (main algo, committed, 1000 runs avg) ---")
    random.seed(42)
    _, _, _, pool_sizes_main, _ = run_simulation(
        "pool_seeding", pool, reserve, num_runs=1000,
        strategy="committed",
        algo_params={'injection_rate': 4, 'activation_threshold': 2})
    for pick in [0, 5, 10, 15, 20, 25, 30]:
        avg = statistics.mean(ps[pick] for ps in pool_sizes_main if pick < len(ps))
        print(f"  After pick {pick:2d}: {avg:.0f} cards in pool")

    print("\n" + "=" * 80)
    print("SIMULATION COMPLETE")
    print("=" * 80)


if __name__ == "__main__":
    main()
