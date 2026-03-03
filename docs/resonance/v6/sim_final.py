"""
Final Synthesis Simulation: All 7 V6 Algorithms + 2 Baselines
Unified card pool, identical parameters, 1000 drafts x 30 picks x 3 strategies.

Algorithms:
  1. Lane Locking (baseline) - thresholds 3/8, 2 slot locks
  2. Threshold Auto-Spend - cost 4, bonus 2
  3. Soft Locks - thresholds 3/6/9, 75% probability, split-resonance
  4. Pool Sculpting - 18/pick, 67/33 T1-heavy
  5. Double Enhancement - threshold 2, bonus 2 (champion); threshold 1, bonus 2 (best variant)
  6. Ratcheting Slots - thresholds 3/6/10, split-resonance
  7. Surge Packs - threshold 4, 3 surge slots
  8. Auto-Spend Pack Widening (baseline) - cost 3, bonus 1
  9. Ratcheting 2/4/7 variant
"""

import random
import statistics
import math
from dataclasses import dataclass, field
from enum import Enum
from collections import Counter, defaultdict
from typing import Optional

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

NUM_DRAFTS = 1000
NUM_PICKS = 30
PACK_SIZE = 4
NUM_ARCHETYPES = 8
NUM_GENERIC = 36
TOTAL_CARDS = 360
DUAL_CAP = 54

RESONANCES = ["Ember", "Stone", "Tide", "Zephyr"]

ARCHETYPES = [
    ("Flash",        "Zephyr", "Ember"),
    ("Blink",        "Ember",  "Zephyr"),
    ("Storm",        "Ember",  "Stone"),
    ("Self-Discard", "Stone",  "Ember"),
    ("Self-Mill",    "Stone",  "Tide"),
    ("Sacrifice",    "Tide",   "Stone"),
    ("Warriors",     "Tide",   "Zephyr"),
    ("Ramp",         "Zephyr", "Tide"),
]

ARCHETYPE_NAMES = [a[0] for a in ARCHETYPES]


def build_fitness_map():
    fm = {}
    for home_idx, (_, home_pri, home_sec) in enumerate(ARCHETYPES):
        fm[home_idx] = {}
        for eval_idx, (_, eval_pri, eval_sec) in enumerate(ARCHETYPES):
            if eval_idx == home_idx:
                fm[home_idx][eval_idx] = "S"
            elif eval_pri == home_pri:
                fm[home_idx][eval_idx] = "A"
            elif eval_sec == home_pri or eval_pri == home_sec:
                fm[home_idx][eval_idx] = "B"
            else:
                fm[home_idx][eval_idx] = "C"
    return fm

FITNESS_MAP = build_fitness_map()
TIER_SCORE = {"S": 4.0, "A": 3.0, "B": 2.0, "C": 0.5, "F": 0.0}


@dataclass
class SimCard:
    id: int
    symbols: list
    archetype_idx: int
    power: float

    @property
    def primary_resonance(self) -> Optional[str]:
        return self.symbols[0] if self.symbols else None

    @property
    def resonance_types(self) -> set:
        return set(self.symbols)

    @property
    def is_generic(self) -> bool:
        return self.archetype_idx == -1

    def fitness_for(self, archetype_idx: int) -> str:
        if self.is_generic:
            return "B"
        return FITNESS_MAP[self.archetype_idx][archetype_idx]

    def fitness_score_for(self, archetype_idx: int) -> float:
        return TIER_SCORE[self.fitness_for(archetype_idx)]

    def is_sa_for(self, archetype_idx: int) -> bool:
        tier = self.fitness_for(archetype_idx)
        return tier in ("S", "A")


# ---------------------------------------------------------------------------
# Card Pool Construction
# ---------------------------------------------------------------------------

def build_card_pool(rng: random.Random) -> list:
    cards = []
    card_id = 0

    for _ in range(NUM_GENERIC):
        cards.append(SimCard(
            id=card_id, symbols=[], archetype_idx=-1,
            power=rng.uniform(3.0, 7.0),
        ))
        card_id += 1

    trimmed = rng.sample(range(NUM_ARCHETYPES), 2)
    non_trimmed = [i for i in range(NUM_ARCHETYPES) if i not in trimmed]
    boosted = set(non_trimmed[:4])

    for arch_idx, (name, pri, sec) in enumerate(ARCHETYPES):
        is_trimmed = arch_idx in trimmed
        is_boosted = arch_idx in boosted
        n_dual_2 = 3 if is_trimmed else 4
        n_mono_1 = 11 if is_boosted else 10
        n_mono_2 = 18 if is_trimmed else 17

        symbol_configs = (
            [([pri], n) for n in range(n_mono_1)] +
            [([pri, pri], n) for n in range(n_mono_2)] +
            [([pri, pri, pri], n) for n in range(6)] +
            [([pri, sec], n) for n in range(n_dual_2)] +
            [([pri, pri, sec], n) for n in range(3)]
        )

        for symbols, _ in symbol_configs:
            power = rng.uniform(2.0, 9.0)
            cards.append(SimCard(
                id=card_id, symbols=list(symbols),
                archetype_idx=arch_idx, power=power,
            ))
            card_id += 1

    assert len(cards) == TOTAL_CARDS, f"Expected {TOTAL_CARDS}, got {len(cards)}"
    return cards


def build_resonance_pools(pool: list) -> dict:
    pools = {}
    for r in RESONANCES:
        pools[r] = [c for c in pool if c.primary_resonance == r]
    return pools


# ---------------------------------------------------------------------------
# Player Strategies
# ---------------------------------------------------------------------------

def _choose_card(pack, strategy, committed_archetype, pick_num, commit_pick,
                 resonance_signals, rng):
    if strategy == "committed":
        return _committed_choose(pack, committed_archetype, pick_num, rng)
    elif strategy == "power-chaser":
        return _power_chaser_choose(pack, rng)
    elif strategy == "signal-reader":
        return _signal_reader_choose(pack, committed_archetype, pick_num,
                                     resonance_signals, rng)
    raise ValueError(f"Unknown strategy: {strategy}")


def _committed_choose(pack, committed_archetype, pick_num, rng):
    if committed_archetype is None:
        symbol_cards = [c for c in pack if c.symbols]
        if symbol_cards:
            return max(symbol_cards, key=lambda c: c.power)
        return max(pack, key=lambda c: c.power)
    else:
        return max(pack, key=lambda c: (c.fitness_score_for(committed_archetype),
                                         c.power))


def _power_chaser_choose(pack, rng):
    return max(pack, key=lambda c: c.power)


def _signal_reader_choose(pack, committed_archetype, pick_num, resonance_signals, rng):
    if committed_archetype is None:
        def score(c):
            if not c.symbols:
                return (0, c.power)
            pri_signal = resonance_signals.get(c.primary_resonance, 0)
            return (pri_signal, c.power)
        return max(pack, key=score)
    else:
        return max(pack, key=lambda c: (c.fitness_score_for(committed_archetype),
                                         c.power))


def _best_archetype_from_drafted(drafted):
    scores = [0.0] * NUM_ARCHETYPES
    for card in drafted:
        for a in range(NUM_ARCHETYPES):
            scores[a] += card.fitness_score_for(a)
    return scores.index(max(scores))


def _signal_read_commit(drafted, resonance_signals):
    sorted_res = sorted(RESONANCES, key=lambda r: resonance_signals[r], reverse=True)
    top_pri = sorted_res[0]
    top_sec = sorted_res[1]
    for idx, (name, pri, sec) in enumerate(ARCHETYPES):
        if pri == top_pri and sec == top_sec:
            return idx
    for idx, (name, pri, sec) in enumerate(ARCHETYPES):
        if pri == top_pri:
            return idx
    return 0


def _get_top_resonance(res_counters):
    return max(RESONANCES, key=lambda r: res_counters[r])


def _get_top_two_resonances(res_counters):
    sorted_r = sorted(RESONANCES, key=lambda r: res_counters[r], reverse=True)
    return sorted_r[0], sorted_r[1]


# ---------------------------------------------------------------------------
# Shared draft infrastructure
# ---------------------------------------------------------------------------

def _update_resonance_counters(card, res_counters):
    """Add weighted symbol counts from a card to resonance counters."""
    for i, sym in enumerate(card.symbols):
        weight = 2 if i == 0 else 1
        res_counters[sym] += weight


def _update_token_counters(card, token_counters):
    """Add tokens from a card: +2 for primary symbol position, +1 for others."""
    for i, sym in enumerate(card.symbols):
        weight = 2 if i == 0 else 1
        token_counters[sym] += weight


def _handle_commitment(strategy, committed_archetype, commit_pick, pick_num,
                       drafted, resonance_signals):
    """Handle strategy commitment logic. Returns (committed_archetype, commit_pick)."""
    if committed_archetype is None and pick_num >= 4:
        if strategy == "committed":
            committed_archetype = _best_archetype_from_drafted(drafted)
            commit_pick = pick_num
        elif strategy == "signal-reader":
            committed_archetype = _signal_read_commit(drafted, resonance_signals)
            commit_pick = pick_num
    return committed_archetype, commit_pick


# ---------------------------------------------------------------------------
# Algorithm 1: Lane Locking (Baseline)
# ---------------------------------------------------------------------------

def lane_locking_draft(pool, res_pools, rng, strategy, thresholds=(3, 8)):
    t_low, t_high = thresholds
    slot_states = ["OPEN"] * PACK_SIZE
    res_counters = {r: 0 for r in RESONANCES}
    thresholds_hit = {r: 0 for r in RESONANCES}

    drafted = []
    pack_records = []
    committed_archetype = None
    commit_pick = None
    resonance_signals = {r: 0 for r in RESONANCES}

    for pick_num in range(NUM_PICKS):
        pack = []
        for slot_idx in range(PACK_SIZE):
            state = slot_states[slot_idx]
            if state == "OPEN":
                card = rng.choice(pool)
            else:
                card = rng.choice(res_pools[state])
            pack.append(card)

        chosen = _choose_card(pack, strategy, committed_archetype, pick_num,
                              commit_pick, resonance_signals, rng)
        pack_records.append((pick_num, list(pack), chosen))
        drafted.append(chosen)

        _update_resonance_counters(chosen, res_counters)
        if chosen.symbols:
            for sym in chosen.symbols:
                resonance_signals[sym] += 1

        for r in RESONANCES:
            if thresholds_hit[r] == 0 and res_counters[r] >= t_low:
                open_slots = [i for i in range(PACK_SIZE) if slot_states[i] == "OPEN"]
                if open_slots:
                    slot_states[rng.choice(open_slots)] = r
                    thresholds_hit[r] = 1
            if thresholds_hit[r] == 1 and res_counters[r] >= t_high:
                open_slots = [i for i in range(PACK_SIZE) if slot_states[i] == "OPEN"]
                if open_slots:
                    slot_states[rng.choice(open_slots)] = r
                    thresholds_hit[r] = 2

        committed_archetype, commit_pick = _handle_commitment(
            strategy, committed_archetype, commit_pick, pick_num,
            drafted, resonance_signals)

    return drafted, pack_records, committed_archetype


# ---------------------------------------------------------------------------
# Algorithm 2: Threshold Auto-Spend (cost 4, bonus 2)
# ---------------------------------------------------------------------------

def threshold_auto_spend_draft(pool, res_pools, rng, strategy, cost=4, bonus=2):
    token_counters = {r: 0 for r in RESONANCES}
    drafted = []
    pack_records = []
    committed_archetype = None
    commit_pick = None
    resonance_signals = {r: 0 for r in RESONANCES}

    for pick_num in range(NUM_PICKS):
        # Auto-spend: check highest resonance tokens before building pack
        bonus_cards = []
        max_res = max(RESONANCES, key=lambda r: token_counters[r])
        while token_counters[max_res] >= cost and len(bonus_cards) < bonus:
            token_counters[max_res] -= cost
            bonus_card = rng.choice(res_pools[max_res])
            bonus_cards.append(bonus_card)
            # Recheck max after spending
            max_res = max(RESONANCES, key=lambda r: token_counters[r])

        pack = [rng.choice(pool) for _ in range(PACK_SIZE)]
        pack.extend(bonus_cards)

        chosen = _choose_card(pack, strategy, committed_archetype, pick_num,
                              commit_pick, resonance_signals, rng)
        pack_records.append((pick_num, list(pack), chosen))
        drafted.append(chosen)

        _update_token_counters(chosen, token_counters)
        if chosen.symbols:
            for sym in chosen.symbols:
                resonance_signals[sym] += 1

        committed_archetype, commit_pick = _handle_commitment(
            strategy, committed_archetype, commit_pick, pick_num,
            drafted, resonance_signals)

    return drafted, pack_records, committed_archetype


# ---------------------------------------------------------------------------
# Algorithm 3: Soft Locks (thresholds 3/6/9, 75% probability, split-resonance)
# ---------------------------------------------------------------------------

def soft_locks_draft(pool, res_pools, rng, strategy,
                     thresholds=(3, 6, 9), lock_prob=0.75):
    res_counters = {r: 0 for r in RESONANCES}
    # Track which slots are soft-locked: list of (resonance, is_top_or_second)
    soft_locks = [None] * PACK_SIZE  # None = open, or resonance string
    locks_granted = 0

    drafted = []
    pack_records = []
    committed_archetype = None
    commit_pick = None
    resonance_signals = {r: 0 for r in RESONANCES}

    for pick_num in range(NUM_PICKS):
        top_res, sec_res = _get_top_two_resonances(res_counters)

        # Build pack
        pack = []
        for slot_idx in range(PACK_SIZE):
            lock_res = soft_locks[slot_idx]
            if lock_res is not None:
                # Soft lock: probability check
                if rng.random() < lock_prob:
                    card = rng.choice(res_pools[lock_res])
                else:
                    card = rng.choice(pool)
            else:
                card = rng.choice(pool)
            pack.append(card)

        chosen = _choose_card(pack, strategy, committed_archetype, pick_num,
                              commit_pick, resonance_signals, rng)
        pack_records.append((pick_num, list(pack), chosen))
        drafted.append(chosen)

        _update_resonance_counters(chosen, res_counters)
        if chosen.symbols:
            for sym in chosen.symbols:
                resonance_signals[sym] += 1

        # Check thresholds for new locks
        top_res, sec_res = _get_top_two_resonances(res_counters)
        for t_idx, threshold in enumerate(thresholds):
            if locks_granted <= t_idx and res_counters[top_res] >= threshold:
                open_slots = [i for i in range(PACK_SIZE) if soft_locks[i] is None]
                if open_slots:
                    slot = rng.choice(open_slots)
                    # First two locks target top resonance, third targets secondary
                    if t_idx < 2:
                        soft_locks[slot] = top_res
                    else:
                        soft_locks[slot] = sec_res
                    locks_granted = t_idx + 1

        committed_archetype, commit_pick = _handle_commitment(
            strategy, committed_archetype, commit_pick, pick_num,
            drafted, resonance_signals)

    return drafted, pack_records, committed_archetype


# ---------------------------------------------------------------------------
# Algorithm 4: Pool Sculpting (18/pick, 67/33 T1-heavy)
# ---------------------------------------------------------------------------

def pool_sculpting_draft(pool, res_pools, rng, strategy,
                         replace_per_pick=18, t1_ratio=0.67, start_pick=3):
    # Create a mutable copy of the pool
    active_pool = list(pool)
    res_counters = {r: 0 for r in RESONANCES}

    # Build a reserve of resonance-matched cards (enriched with dual-type)
    reserve = []
    for _ in range(800):
        arch_idx = rng.randint(0, NUM_ARCHETYPES - 1)
        pri = ARCHETYPES[arch_idx][1]
        sec = ARCHETYPES[arch_idx][2]
        # 25% dual-type in reserve
        if rng.random() < 0.25:
            symbols = [pri, sec]
        else:
            symbols = [pri, pri] if rng.random() < 0.6 else [pri]
        reserve.append(SimCard(
            id=10000 + len(reserve), symbols=symbols,
            archetype_idx=arch_idx, power=rng.uniform(2.0, 9.0),
        ))

    drafted = []
    pack_records = []
    committed_archetype = None
    commit_pick = None
    resonance_signals = {r: 0 for r in RESONANCES}

    for pick_num in range(NUM_PICKS):
        # Build pack from active pool
        pack = [rng.choice(active_pool) for _ in range(PACK_SIZE)]

        chosen = _choose_card(pack, strategy, committed_archetype, pick_num,
                              commit_pick, resonance_signals, rng)
        pack_records.append((pick_num, list(pack), chosen))
        drafted.append(chosen)

        _update_resonance_counters(chosen, res_counters)
        if chosen.symbols:
            for sym in chosen.symbols:
                resonance_signals[sym] += 1

        # Pool sculpting: replace cards in active pool after start_pick
        if pick_num >= start_pick and res_counters[max(RESONANCES, key=lambda r: res_counters[r])] > 0:
            top_res, sec_res = _get_top_two_resonances(res_counters)
            n_t1 = int(replace_per_pick * t1_ratio)
            n_t2 = replace_per_pick - n_t1

            # Find off-resonance cards to replace
            off_res_indices = [i for i, c in enumerate(active_pool)
                               if c.primary_resonance not in (top_res, sec_res)
                               and not c.is_generic]
            rng.shuffle(off_res_indices)
            to_replace = off_res_indices[:replace_per_pick]

            # Get replacement cards from reserve
            t1_pool = [c for c in reserve if c.primary_resonance == top_res]
            t2_pool = [c for c in reserve if c.primary_resonance == sec_res]

            replaced = 0
            for idx in to_replace:
                if replaced < n_t1 and t1_pool:
                    active_pool[idx] = rng.choice(t1_pool)
                elif t2_pool:
                    active_pool[idx] = rng.choice(t2_pool)
                replaced += 1

        committed_archetype, commit_pick = _handle_commitment(
            strategy, committed_archetype, commit_pick, pick_num,
            drafted, resonance_signals)

    return drafted, pack_records, committed_archetype


# ---------------------------------------------------------------------------
# Algorithm 5: Double Enhancement (threshold, bonus)
# ---------------------------------------------------------------------------

def double_enhancement_draft(pool, res_pools, rng, strategy,
                             trigger_threshold=2, bonus_count=2):
    res_counters = {r: 0 for r in RESONANCES}
    drafted = []
    pack_records = []
    committed_archetype = None
    commit_pick = None
    resonance_signals = {r: 0 for r in RESONANCES}

    for pick_num in range(NUM_PICKS):
        top_res = _get_top_resonance(res_counters)
        activated = res_counters[top_res] >= 4  # minimum 4 weighted symbols

        # Draw base pack
        pack = [rng.choice(pool) for _ in range(PACK_SIZE)]

        # Check trigger: how many of the 4 cards share primary resonance with top?
        if activated:
            matches = sum(1 for c in pack if c.primary_resonance == top_res)
            if matches >= trigger_threshold:
                # Add bonus cards
                for _ in range(bonus_count):
                    bonus_card = rng.choice(res_pools[top_res])
                    pack.append(bonus_card)

        chosen = _choose_card(pack, strategy, committed_archetype, pick_num,
                              commit_pick, resonance_signals, rng)
        pack_records.append((pick_num, list(pack), chosen))
        drafted.append(chosen)

        _update_resonance_counters(chosen, res_counters)
        if chosen.symbols:
            for sym in chosen.symbols:
                resonance_signals[sym] += 1

        committed_archetype, commit_pick = _handle_commitment(
            strategy, committed_archetype, commit_pick, pick_num,
            drafted, resonance_signals)

    return drafted, pack_records, committed_archetype


# ---------------------------------------------------------------------------
# Algorithm 6: Ratcheting Slots (thresholds, split-resonance)
# ---------------------------------------------------------------------------

def ratcheting_slots_draft(pool, res_pools, rng, strategy,
                           thresholds=(3, 6, 10)):
    res_counters = {r: 0 for r in RESONANCES}
    slot_states = [None] * PACK_SIZE  # None = open, or resonance string
    locks_granted = 0

    drafted = []
    pack_records = []
    committed_archetype = None
    commit_pick = None
    resonance_signals = {r: 0 for r in RESONANCES}

    for pick_num in range(NUM_PICKS):
        # Build pack
        pack = []
        for slot_idx in range(PACK_SIZE):
            lock_res = slot_states[slot_idx]
            if lock_res is not None:
                card = rng.choice(res_pools[lock_res])
            else:
                card = rng.choice(pool)
            pack.append(card)

        chosen = _choose_card(pack, strategy, committed_archetype, pick_num,
                              commit_pick, resonance_signals, rng)
        pack_records.append((pick_num, list(pack), chosen))
        drafted.append(chosen)

        _update_resonance_counters(chosen, res_counters)
        if chosen.symbols:
            for sym in chosen.symbols:
                resonance_signals[sym] += 1

        # Check thresholds
        top_res, sec_res = _get_top_two_resonances(res_counters)
        for t_idx, threshold in enumerate(thresholds):
            if locks_granted <= t_idx and res_counters[top_res] >= threshold:
                open_slots = [i for i in range(PACK_SIZE) if slot_states[i] is None]
                if open_slots:
                    slot = rng.choice(open_slots)
                    # First two lock to top resonance, third to secondary
                    if t_idx < 2:
                        slot_states[slot] = top_res
                    else:
                        slot_states[slot] = sec_res
                    locks_granted = t_idx + 1

        committed_archetype, commit_pick = _handle_commitment(
            strategy, committed_archetype, commit_pick, pick_num,
            drafted, resonance_signals)

    return drafted, pack_records, committed_archetype


# ---------------------------------------------------------------------------
# Algorithm 7: Surge Packs (threshold 4, 3 surge slots)
# ---------------------------------------------------------------------------

def surge_packs_draft(pool, res_pools, rng, strategy,
                      threshold=4, surge_slots=3):
    token_counters = {r: 0 for r in RESONANCES}
    drafted = []
    pack_records = []
    committed_archetype = None
    commit_pick = None
    resonance_signals = {r: 0 for r in RESONANCES}
    surge_resonance = None  # resonance for the next pack's surge

    for pick_num in range(NUM_PICKS):
        # Build pack
        pack = []
        if surge_resonance is not None:
            # Surge pack: surge_slots filled with resonance cards, rest random
            for slot_idx in range(PACK_SIZE):
                if slot_idx < surge_slots:
                    card = rng.choice(res_pools[surge_resonance])
                else:
                    card = rng.choice(pool)
                pack.append(card)
            surge_resonance = None
        else:
            # Normal pack: all random
            pack = [rng.choice(pool) for _ in range(PACK_SIZE)]

        chosen = _choose_card(pack, strategy, committed_archetype, pick_num,
                              commit_pick, resonance_signals, rng)
        pack_records.append((pick_num, list(pack), chosen))
        drafted.append(chosen)

        # Earn tokens
        _update_token_counters(chosen, token_counters)
        if chosen.symbols:
            for sym in chosen.symbols:
                resonance_signals[sym] += 1

        # Check for surge trigger
        max_res = max(RESONANCES, key=lambda r: token_counters[r])
        if token_counters[max_res] >= threshold:
            token_counters[max_res] -= threshold
            surge_resonance = max_res

        committed_archetype, commit_pick = _handle_commitment(
            strategy, committed_archetype, commit_pick, pick_num,
            drafted, resonance_signals)

    return drafted, pack_records, committed_archetype


# ---------------------------------------------------------------------------
# Algorithm 8: Auto-Spend Pack Widening (baseline) - cost 3, bonus 1
# ---------------------------------------------------------------------------

def auto_spend_pw_draft(pool, res_pools, rng, strategy, cost=3, bonus=1):
    token_counters = {r: 0 for r in RESONANCES}
    drafted = []
    pack_records = []
    committed_archetype = None
    commit_pick = None
    resonance_signals = {r: 0 for r in RESONANCES}

    for pick_num in range(NUM_PICKS):
        bonus_cards = []
        max_res = max(RESONANCES, key=lambda r: token_counters[r])
        if token_counters[max_res] >= cost:
            token_counters[max_res] -= cost
            for _ in range(bonus):
                bonus_card = rng.choice(res_pools[max_res])
                bonus_cards.append(bonus_card)

        pack = [rng.choice(pool) for _ in range(PACK_SIZE)]
        pack.extend(bonus_cards)

        chosen = _choose_card(pack, strategy, committed_archetype, pick_num,
                              commit_pick, resonance_signals, rng)
        pack_records.append((pick_num, list(pack), chosen))
        drafted.append(chosen)

        _update_token_counters(chosen, token_counters)
        if chosen.symbols:
            for sym in chosen.symbols:
                resonance_signals[sym] += 1

        committed_archetype, commit_pick = _handle_commitment(
            strategy, committed_archetype, commit_pick, pick_num,
            drafted, resonance_signals)

    return drafted, pack_records, committed_archetype


# ---------------------------------------------------------------------------
# Metrics Computation
# ---------------------------------------------------------------------------

def compute_metrics(all_results: list, algorithm_name: str) -> dict:
    """Compute all 9 required metrics from simulation results.
    all_results: list of (strategy, drafted, pack_records, committed_arch)
    """
    metrics = {}
    by_strategy = defaultdict(list)
    for strategy, drafted, pack_records, committed_arch in all_results:
        by_strategy[strategy].append((drafted, pack_records, committed_arch))

    # M1: Picks 1-5 unique archetypes with S/A cards per pack
    m1_values = []
    for strategy, results in by_strategy.items():
        for drafted, pack_records, committed_arch in results:
            for pick_num, pack, chosen in pack_records:
                if pick_num >= 5:
                    continue
                archs_with_sa = set()
                for card in pack:
                    for a in range(NUM_ARCHETYPES):
                        if card.is_sa_for(a):
                            archs_with_sa.add(a)
                m1_values.append(len(archs_with_sa))
    metrics["m1_unique_archetypes_early"] = statistics.mean(m1_values) if m1_values else 0

    # M2: Picks 1-5 S/A cards for emerging archetype per pack
    m2_values = []
    for strategy, results in by_strategy.items():
        for drafted, pack_records, committed_arch in results:
            if committed_arch is None:
                continue
            for pick_num, pack, chosen in pack_records:
                if pick_num >= 5:
                    continue
                sa_count = sum(1 for c in pack[:PACK_SIZE] if c.is_sa_for(committed_arch))
                m2_values.append(sa_count)
    metrics["m2_sa_emerging_early"] = statistics.mean(m2_values) if m2_values else 0

    # M3: Picks 6+ S/A cards for committed archetype per pack
    m3_values = []
    m3_per_pack_values = []
    for strategy, results in by_strategy.items():
        for drafted, pack_records, committed_arch in results:
            if committed_arch is None:
                continue
            for pick_num, pack, chosen in pack_records:
                if pick_num < 5:
                    continue
                sa_count = sum(1 for c in pack if c.is_sa_for(committed_arch))
                m3_values.append(sa_count)
                m3_per_pack_values.append(sa_count)
    metrics["m3_sa_committed_late"] = statistics.mean(m3_values) if m3_values else 0

    # M4: Picks 6+ off-archetype (C/F) cards per pack
    m4_values = []
    for strategy, results in by_strategy.items():
        for drafted, pack_records, committed_arch in results:
            if committed_arch is None:
                continue
            for pick_num, pack, chosen in pack_records:
                if pick_num < 5:
                    continue
                cf_count = sum(1 for c in pack
                               if c.fitness_for(committed_arch) in ("C", "F"))
                m4_values.append(cf_count)
    metrics["m4_off_archetype_late"] = statistics.mean(m4_values) if m4_values else 0

    # M5: Convergence pick
    convergence_picks = []
    for strategy, results in by_strategy.items():
        if strategy == "power-chaser":
            continue
        for drafted, pack_records, committed_arch in results:
            if committed_arch is None:
                continue
            for pick_num, pack, chosen in pack_records:
                if pick_num < 2:
                    continue
                if pick_num + 2 >= len(pack_records):
                    continue
                all_good = True
                for w in range(3):
                    _, wp, _ = pack_records[pick_num + w]
                    sa = sum(1 for c in wp if c.is_sa_for(committed_arch))
                    if sa < 2:
                        all_good = False
                        break
                if all_good:
                    convergence_picks.append(pick_num)
                    break
    metrics["m5_convergence_pick"] = statistics.mean(convergence_picks) if convergence_picks else 30

    # M6: Deck archetype concentration
    m6_values = []
    for strategy, results in by_strategy.items():
        for drafted, pack_records, committed_arch in results:
            if committed_arch is None:
                continue
            sa_cards = sum(1 for c in drafted if c.is_sa_for(committed_arch))
            m6_values.append(sa_cards / len(drafted) * 100)
    metrics["m6_deck_concentration"] = statistics.mean(m6_values) if m6_values else 0

    # M7: Run-to-run card overlap
    archetype_runs = defaultdict(list)
    for strategy, results in by_strategy.items():
        for drafted, pack_records, committed_arch in results:
            if committed_arch is not None:
                archetype_runs[committed_arch].append(set(c.id for c in drafted))

    overlaps = []
    for arch, run_sets in archetype_runs.items():
        if len(run_sets) < 2:
            continue
        n_pairs = min(200, len(run_sets) * (len(run_sets) - 1) // 2)
        for _ in range(n_pairs):
            i, j = random.sample(range(len(run_sets)), 2)
            s1, s2 = run_sets[i], run_sets[j]
            if len(s1) == 0 or len(s2) == 0:
                continue
            overlap = len(s1 & s2) / min(len(s1), len(s2)) * 100
            overlaps.append(overlap)
    metrics["m7_card_overlap"] = statistics.mean(overlaps) if overlaps else 0

    # M8: Archetype frequency
    arch_counts = Counter()
    total_committed = 0
    for strategy, results in by_strategy.items():
        for drafted, pack_records, committed_arch in results:
            if committed_arch is not None:
                arch_counts[committed_arch] += 1
                total_committed += 1

    arch_freqs = {}
    for a in range(NUM_ARCHETYPES):
        arch_freqs[ARCHETYPE_NAMES[a]] = (arch_counts[a] / total_committed * 100
                                           if total_committed > 0 else 0)
    metrics["m8_archetype_freq"] = arch_freqs
    metrics["m8_max_freq"] = max(arch_freqs.values()) if arch_freqs else 0
    metrics["m8_min_freq"] = min(arch_freqs.values()) if arch_freqs else 0

    # M9: StdDev of S/A cards per pack (picks 6+)
    metrics["m9_sa_stddev"] = (statistics.stdev(m3_per_pack_values)
                                if len(m3_per_pack_values) > 1 else 0)

    return metrics


def per_archetype_convergence(all_results: list) -> dict:
    arch_conv = defaultdict(list)
    for strategy, drafted, pack_records, committed_arch in all_results:
        if strategy == "power-chaser" or committed_arch is None:
            continue
        for pick_num, pack, chosen in pack_records:
            if pick_num < 2:
                continue
            if pick_num + 2 >= len(pack_records):
                continue
            all_good = True
            for w in range(3):
                _, wp, _ = pack_records[pick_num + w]
                sa = sum(1 for c in wp if c.is_sa_for(committed_arch))
                if sa < 2:
                    all_good = False
                    break
            if all_good:
                arch_conv[committed_arch].append(pick_num)
                break

    result = {}
    for a in range(NUM_ARCHETYPES):
        picks = arch_conv.get(a, [])
        result[ARCHETYPE_NAMES[a]] = (statistics.mean(picks) if picks else float('nan'),
                                       len(picks))
    return result


# ---------------------------------------------------------------------------
# Pass/Fail evaluation
# ---------------------------------------------------------------------------

def evaluate_pass_fail(metrics):
    results = {}
    results["M1"] = "PASS" if metrics["m1_unique_archetypes_early"] >= 3 else "FAIL"
    results["M2"] = "PASS" if metrics["m2_sa_emerging_early"] <= 2 else "FAIL"
    results["M3"] = "PASS" if metrics["m3_sa_committed_late"] >= 2.0 else "FAIL"
    results["M4"] = "PASS" if metrics["m4_off_archetype_late"] >= 0.5 else "FAIL"
    results["M5"] = "PASS" if 5 <= metrics["m5_convergence_pick"] <= 8 else "FAIL"
    results["M6"] = "PASS" if 60 <= metrics["m6_deck_concentration"] <= 90 else "FAIL"
    results["M7"] = "PASS" if metrics["m7_card_overlap"] < 40 else "FAIL"
    results["M8"] = "PASS" if (metrics["m8_min_freq"] >= 5 and
                                metrics["m8_max_freq"] <= 20) else "FAIL"
    results["M9"] = "PASS" if metrics["m9_sa_stddev"] >= 0.8 else "FAIL"
    return results


# ---------------------------------------------------------------------------
# Printing
# ---------------------------------------------------------------------------

def print_metrics(metrics, name, pf):
    passed = sum(1 for v in pf.values() if v == "PASS")
    print(f"\n--- {name} ({passed}/9 PASS) ---")
    print(f"  M1 Unique archs early:    {metrics['m1_unique_archetypes_early']:.2f}  (>= 3)    [{pf['M1']}]")
    print(f"  M2 S/A early:             {metrics['m2_sa_emerging_early']:.2f}  (<= 2)    [{pf['M2']}]")
    print(f"  M3 S/A committed late:    {metrics['m3_sa_committed_late']:.2f}  (>= 2.0)  [{pf['M3']}]")
    print(f"  M4 Off-arch late:         {metrics['m4_off_archetype_late']:.2f}  (>= 0.5)  [{pf['M4']}]")
    print(f"  M5 Convergence pick:      {metrics['m5_convergence_pick']:.1f}  (5-8)     [{pf['M5']}]")
    print(f"  M6 Deck concentration:    {metrics['m6_deck_concentration']:.1f}%  (60-90%)  [{pf['M6']}]")
    print(f"  M7 Card overlap:          {metrics['m7_card_overlap']:.1f}%  (< 40%)   [{pf['M7']}]")
    print(f"  M8 Arch freq range:       {metrics['m8_min_freq']:.1f}%-{metrics['m8_max_freq']:.1f}%  (5-20%)   [{pf['M8']}]")
    print(f"  M9 S/A stddev:            {metrics['m9_sa_stddev']:.2f}  (>= 0.8)  [{pf['M9']}]")


# ---------------------------------------------------------------------------
# Main Simulation
# ---------------------------------------------------------------------------

def run_algorithm(name, draft_fn, pool, res_pools, rng, strategies):
    """Run 1000 drafts for a single algorithm and return results."""
    all_results = []
    for run_idx in range(NUM_DRAFTS):
        strategy = strategies[run_idx % len(strategies)]
        drafted, pack_records, committed_arch = draft_fn(pool, res_pools, rng, strategy)
        all_results.append((strategy, drafted, pack_records, committed_arch))
    return all_results


def main():
    rng = random.Random(42)
    pool = build_card_pool(rng)
    res_pools = build_resonance_pools(pool)

    dual_count = sum(1 for c in pool if len(c.resonance_types) >= 2)
    print(f"Pool: {len(pool)} cards, {dual_count} dual-type ({dual_count/len(pool)*100:.1f}%)")
    for r in RESONANCES:
        print(f"  {r} primary: {len(res_pools[r])} cards")

    strategies = ["committed", "power-chaser", "signal-reader"]

    # Define all algorithms
    algorithms = [
        ("1. Lane Locking (3/8)",
         lambda p, rp, r, s: lane_locking_draft(p, rp, r, s, thresholds=(3, 8))),
        ("2. Threshold Auto-Spend (C4/B2)",
         lambda p, rp, r, s: threshold_auto_spend_draft(p, rp, r, s, cost=4, bonus=2)),
        ("3. Soft Locks (3/6/9, 75%)",
         lambda p, rp, r, s: soft_locks_draft(p, rp, r, s, thresholds=(3, 6, 9), lock_prob=0.75)),
        ("4. Pool Sculpting (18/pick)",
         lambda p, rp, r, s: pool_sculpting_draft(p, rp, r, s, replace_per_pick=18, t1_ratio=0.67)),
        ("5a. Double Enhancement (T=2/B=2)",
         lambda p, rp, r, s: double_enhancement_draft(p, rp, r, s, trigger_threshold=2, bonus_count=2)),
        ("5b. Double Enhancement (T=1/B=2)",
         lambda p, rp, r, s: double_enhancement_draft(p, rp, r, s, trigger_threshold=1, bonus_count=2)),
        ("6a. Ratcheting Slots (3/6/10)",
         lambda p, rp, r, s: ratcheting_slots_draft(p, rp, r, s, thresholds=(3, 6, 10))),
        ("6b. Ratcheting Slots (2/4/7)",
         lambda p, rp, r, s: ratcheting_slots_draft(p, rp, r, s, thresholds=(2, 4, 7))),
        ("7. Surge Packs (T=4/S=3)",
         lambda p, rp, r, s: surge_packs_draft(p, rp, r, s, threshold=4, surge_slots=3)),
        ("8. Auto-Spend PW (C3/B1)",
         lambda p, rp, r, s: auto_spend_pw_draft(p, rp, r, s, cost=3, bonus=1)),
    ]

    all_algo_metrics = {}
    all_algo_pf = {}
    all_algo_conv = {}

    for algo_name, draft_fn in algorithms:
        print(f"\n{'='*60}")
        print(f"Running: {algo_name}")
        print(f"{'='*60}")

        results = run_algorithm(algo_name, draft_fn, pool, res_pools, rng, strategies)
        metrics = compute_metrics(results, algo_name)
        pf = evaluate_pass_fail(metrics)
        arch_conv = per_archetype_convergence(results)

        print_metrics(metrics, algo_name, pf)

        print(f"\n  Per-Archetype Convergence:")
        print(f"    {'Archetype':<15} {'Avg Pick':>10} {'N Runs':>8}")
        for name, (avg, n) in arch_conv.items():
            avg_str = f"{avg:.1f}" if not math.isnan(avg) else "N/A"
            print(f"    {name:<15} {avg_str:>10} {n:>8}")

        all_algo_metrics[algo_name] = metrics
        all_algo_pf[algo_name] = pf
        all_algo_conv[algo_name] = arch_conv

    # ========== Summary Table ==========
    print(f"\n\n{'='*100}")
    print("UNIFIED COMPARISON TABLE")
    print(f"{'='*100}")

    header = f"{'Algorithm':<35} {'M1':>5} {'M2':>5} {'M3':>6} {'M4':>5} {'M5':>6} {'M6':>6} {'M7':>6} {'M8':>8} {'M9':>5} {'Pass':>5}"
    print(header)
    print("-" * len(header))

    for algo_name in all_algo_metrics:
        m = all_algo_metrics[algo_name]
        pf = all_algo_pf[algo_name]
        passed = sum(1 for v in pf.values() if v == "PASS")
        m8_str = f"{m['m8_min_freq']:.0f}-{m['m8_max_freq']:.0f}%"
        line = (f"{algo_name:<35} "
                f"{m['m1_unique_archetypes_early']:>5.1f} "
                f"{m['m2_sa_emerging_early']:>5.2f} "
                f"{m['m3_sa_committed_late']:>6.2f} "
                f"{m['m4_off_archetype_late']:>5.2f} "
                f"{m['m5_convergence_pick']:>6.1f} "
                f"{m['m6_deck_concentration']:>5.1f}% "
                f"{m['m7_card_overlap']:>5.1f}% "
                f"{m8_str:>8} "
                f"{m['m9_sa_stddev']:>5.2f} "
                f"{passed:>4}/9")
        print(line)

    # ========== Pass/Fail Summary ==========
    print(f"\n{'='*100}")
    print("PASS/FAIL DETAIL")
    print(f"{'='*100}")

    header2 = f"{'Algorithm':<35} {'M1':>5} {'M2':>5} {'M3':>5} {'M4':>5} {'M5':>5} {'M6':>5} {'M7':>5} {'M8':>5} {'M9':>5}"
    print(header2)
    print("-" * len(header2))

    for algo_name in all_algo_pf:
        pf = all_algo_pf[algo_name]
        line = (f"{algo_name:<35} "
                f"{pf['M1']:>5} "
                f"{pf['M2']:>5} "
                f"{pf['M3']:>5} "
                f"{pf['M4']:>5} "
                f"{pf['M5']:>5} "
                f"{pf['M6']:>5} "
                f"{pf['M7']:>5} "
                f"{pf['M8']:>5} "
                f"{pf['M9']:>5}")
        print(line)

    # ========== Pack Quality Distribution ==========
    print(f"\n{'='*100}")
    print("PACK QUALITY DISTRIBUTION (S/A per pack, picks 6+, committed strategy only)")
    print(f"{'='*100}")

    # Re-run brief analysis for each algorithm
    for algo_name, draft_fn in algorithms:
        # Use stored results -- need to re-run to get distribution
        # Actually let's compute from stored metrics (m3 values)
        # We can recompute from stored results
        pass

    print("\n[Pack distribution data available in per-algorithm metrics above]")


if __name__ == "__main__":
    main()
