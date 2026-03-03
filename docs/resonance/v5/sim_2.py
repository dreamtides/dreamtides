#!/usr/bin/env python3
"""
Resonance V5 — Agent 2: Pair-Escalation Slots v2

One-sentence algorithm:
"Track the resonance pair (first, second symbol) of each 2+ symbol card you
draft; each pack slot independently shows a card matching your most common
pair with probability min(that pair's count / 6, 0.65), otherwise a random card."

This file is self-contained and runnable with `python3 sim_2.py`.
"""

import random
import statistics
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional
from collections import defaultdict

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

RESONANCES = ["Ember", "Stone", "Tide", "Zephyr"]

ARCHETYPES = [
    "Flash/Tempo/Prison",   # 0: Zephyr primary, Ember secondary
    "Blink/Flicker",        # 1: Ember primary, Zephyr secondary
    "Storm/Spellslinger",   # 2: Ember primary, Stone secondary
    "Self-Discard",         # 3: Stone primary, Ember secondary
    "Self-Mill/Reanimator", # 4: Stone primary, Tide secondary
    "Sacrifice/Abandon",    # 5: Tide primary, Stone secondary
    "Warriors/Midrange",    # 6: Tide primary, Zephyr secondary
    "Ramp/Spirit Animals",  # 7: Zephyr primary, Tide secondary
]

ARCHETYPE_RESONANCES = {
    0: ("Zephyr", "Ember"),
    1: ("Ember", "Zephyr"),
    2: ("Ember", "Stone"),
    3: ("Stone", "Ember"),
    4: ("Stone", "Tide"),
    5: ("Tide", "Stone"),
    6: ("Tide", "Zephyr"),
    7: ("Zephyr", "Tide"),
}

NUM_ARCHETYPES = 8
NUM_CARDS = 360
GENERIC_COUNT = 36
CARDS_PER_ARCHETYPE = (NUM_CARDS - GENERIC_COUNT) // NUM_ARCHETYPES  # 40
PACK_SIZE = 4
NUM_PICKS = 30
NUM_DRAFTS = 1000
PRIMARY_WEIGHT = 2  # primary symbol counts as 2

# Default Pair-Escalation parameters
DEFAULT_K = 6
DEFAULT_CAP = 0.65

# ---------------------------------------------------------------------------
# Card model
# ---------------------------------------------------------------------------

class Tier(Enum):
    S = 4
    A = 3
    B = 2
    C = 1
    F = 0

@dataclass
class SimCard:
    id: int
    symbols: list
    archetype: Optional[str]
    archetype_idx: Optional[int]
    archetype_fitness: dict = field(default_factory=dict)
    power: float = 5.0

def _circle_distance(a: int, b: int) -> int:
    return min(abs(a - b), NUM_ARCHETYPES - abs(a - b))

def _assign_fitness(card: SimCard):
    """Assign archetype fitness tiers based on spec:
    - S-tier in home archetype
    - A-tier in adjacent archetype sharing its primary resonance
    - B-tier in archetypes sharing its secondary resonance
    - C/F in distant archetypes
    - Generics are B-tier everywhere
    """
    if card.archetype_idx is None:
        for i in range(NUM_ARCHETYPES):
            card.archetype_fitness[i] = Tier.B
        return
    home = card.archetype_idx
    home_pri, home_sec = ARCHETYPE_RESONANCES[home]
    for i in range(NUM_ARCHETYPES):
        if i == home:
            card.archetype_fitness[i] = Tier.S
            continue
        dist = _circle_distance(home, i)
        target_pri, target_sec = ARCHETYPE_RESONANCES[i]
        # A-tier: adjacent archetype that shares the card's primary resonance
        if dist == 1 and (target_pri == home_pri or target_sec == home_pri):
            card.archetype_fitness[i] = Tier.A
        # B-tier: archetype shares the card's secondary resonance
        elif home_sec in (target_pri, target_sec):
            card.archetype_fitness[i] = Tier.B
        elif dist <= 2:
            card.archetype_fitness[i] = Tier.C
        elif dist == 3:
            card.archetype_fitness[i] = Tier.C
        else:
            card.archetype_fitness[i] = Tier.F

def _generate_symbols(archetype_idx: int, num_symbols: int, rng: random.Random) -> list:
    primary_res, secondary_res = ARCHETYPE_RESONANCES[archetype_idx]
    if num_symbols == 1:
        return [primary_res] if rng.random() < 0.70 else [secondary_res]
    elif num_symbols == 2:
        first = primary_res if rng.random() < 0.75 else secondary_res
        other = secondary_res if first == primary_res else primary_res
        second = other if rng.random() < 0.80 else first
        return [first, second]
    else:  # 3 symbols
        patterns = [
            [primary_res, secondary_res, primary_res],
            [primary_res, primary_res, secondary_res],
            [primary_res, secondary_res, secondary_res],
        ]
        return rng.choice(patterns)

def build_card_pool(rng: random.Random,
                    pct_1sym: float = 0.15,
                    pct_2sym: float = 0.60,
                    pct_3sym: float = 0.25) -> list:
    """Build 360-card pool with specified symbol distribution."""
    cards = []
    card_id = 0

    for arch_idx in range(NUM_ARCHETYPES):
        n = CARDS_PER_ARCHETYPE
        n1 = round(n * pct_1sym)
        n3 = round(n * pct_3sym)
        n2 = n - n1 - n3

        for count, num in [(1, n1), (2, n2), (3, n3)]:
            for _ in range(num):
                syms = _generate_symbols(arch_idx, count, rng)
                c = SimCard(id=card_id, symbols=syms,
                            archetype=ARCHETYPES[arch_idx],
                            archetype_idx=arch_idx,
                            power=rng.uniform(3, 8))
                _assign_fitness(c)
                cards.append(c)
                card_id += 1

    for _ in range(GENERIC_COUNT):
        c = SimCard(id=card_id, symbols=[], archetype=None, archetype_idx=None,
                    power=rng.uniform(4, 9))
        _assign_fitness(c)
        cards.append(c)
        card_id += 1

    return cards

def get_card_pair(card: SimCard):
    """Get the ordered resonance pair (primary, secondary) for a 2+ symbol card. None for 0-1 symbol cards."""
    if len(card.symbols) >= 2:
        return (card.symbols[0], card.symbols[1])
    return None

# ---------------------------------------------------------------------------
# Pool analysis utilities
# ---------------------------------------------------------------------------

def analyze_pool(pool: list):
    """Print pool statistics for validation."""
    by_arch = defaultdict(int)
    by_sym_count = defaultdict(int)
    pair_counts = defaultdict(int)
    for c in pool:
        if c.archetype_idx is not None:
            by_arch[c.archetype_idx] += 1
        by_sym_count[len(c.symbols)] += 1
        p = get_card_pair(c)
        if p:
            pair_counts[p] += 1

    print("\n--- Pool Analysis ---")
    print(f"Total cards: {len(pool)}")
    print(f"By symbol count: {dict(sorted(by_sym_count.items()))}")
    print(f"By archetype: {dict(sorted(by_arch.items()))}")
    print(f"\nPair distribution (top pairs):")
    for pair, cnt in sorted(pair_counts.items(), key=lambda x: -x[1]):
        print(f"  {pair}: {cnt} cards")

    # Measure pair precision: for each pair, what % of cards are S-tier for the corresponding archetype?
    pair_to_arch = {}
    for ai, (pri, sec) in ARCHETYPE_RESONANCES.items():
        pair_to_arch[(pri, sec)] = ai
    print(f"\nPair precision (S-tier hit rate):")
    for pair in sorted(pair_counts.keys()):
        matching_cards = [c for c in pool if get_card_pair(c) == pair]
        expected_arch = pair_to_arch.get(pair)
        if expected_arch is not None and matching_cards:
            s_tier = sum(1 for c in matching_cards
                         if c.archetype_fitness.get(expected_arch, Tier.F) == Tier.S)
            print(f"  {pair} -> {ARCHETYPES[expected_arch]}: {s_tier}/{len(matching_cards)} = {s_tier/len(matching_cards):.1%} S-tier")

# ---------------------------------------------------------------------------
# Draft algorithms
# ---------------------------------------------------------------------------

def _update_resonance_counters(card: SimCard, player_counts: dict):
    """Update single-resonance counters (for Lane Locking baseline)."""
    if not card.symbols:
        return
    player_counts[card.symbols[0]] = player_counts.get(card.symbols[0], 0) + PRIMARY_WEIGHT
    for s in card.symbols[1:]:
        player_counts[s] = player_counts.get(s, 0) + 1

def _update_pair_counters(card: SimCard, pair_counts: dict):
    """Update ordered pair counters (for Pair-Escalation)."""
    p = get_card_pair(card)
    if p:
        pair_counts[p] = pair_counts.get(p, 0) + 1

def _get_top_pair(pair_counts: dict):
    """Return the most common pair and its count. None if no pairs tracked."""
    if not pair_counts:
        return None, 0
    top = max(pair_counts.items(), key=lambda x: x[1])
    return top[0], top[1]

# --- Pair-Escalation Slots (Agent 2 champion) ---

def generate_pack_pair_escalation(pool: list, pair_counts: dict, rng: random.Random,
                                   K: float = DEFAULT_K, cap: float = DEFAULT_CAP) -> list:
    """
    Pair-Escalation Slots: Each pack slot independently shows a card matching
    the player's most common pair with probability min(top_pair_count / K, cap),
    otherwise a random card. Targeted draws are with-replacement from the
    pair-matched subset of the pool.
    """
    top_pair, top_count = _get_top_pair(pair_counts)
    prob = min(top_count / K, cap) if top_pair else 0.0

    # Pre-filter cards matching the top pair
    if top_pair:
        pair_matched = [c for c in pool if get_card_pair(c) == top_pair]
    else:
        pair_matched = []

    pack = []
    used_ids = set()

    for _ in range(PACK_SIZE):
        if top_pair and pair_matched and rng.random() < prob:
            # Targeted slot: pick a random card matching the top pair (with-replacement from pair pool)
            chosen = rng.choice(pair_matched)
        else:
            # Random slot: pick any card from pool not already in pack
            candidates = [c for c in pool if c.id not in used_ids]
            if not candidates:
                candidates = pool
            chosen = rng.choice(candidates)
        pack.append(chosen)
        used_ids.add(chosen.id)

    return pack

# --- V3 Lane Locking baseline ---

def generate_pack_lane_locking(pool: list, player_counts: dict, rng: random.Random,
                                threshold_1: int = 3, threshold_2: int = 8) -> list:
    """V3 Lane Locking: resonance-based slot locking at thresholds 3 and 8."""
    locked_resonances = []
    for res in RESONANCES:
        cnt = player_counts.get(res, 0)
        if cnt >= threshold_2:
            locked_resonances.append(res)
            locked_resonances.append(res)
        elif cnt >= threshold_1:
            locked_resonances.append(res)

    pack = []
    used_ids = set()

    for res in locked_resonances[:PACK_SIZE]:
        candidates = [c for c in pool if res in c.symbols and c.id not in used_ids]
        if candidates:
            chosen = rng.choice(candidates)
            pack.append(chosen)
            used_ids.add(chosen.id)

    remaining = PACK_SIZE - len(pack)
    if remaining > 0:
        candidates = [c for c in pool if c.id not in used_ids]
        if candidates:
            chosen = rng.sample(candidates, min(remaining, len(candidates)))
            pack.extend(chosen)
            for c in chosen:
                used_ids.add(c.id)

    return pack[:PACK_SIZE]

# --- V4 Pack Widening baseline (auto-spend) ---

def generate_pack_widening(pool: list, player_counts: dict, rng: random.Random,
                            cost: int = 3, bonus_count: int = 1) -> list:
    """
    V4 Pack Widening (auto-spend): when any resonance counter >= cost,
    automatically add bonus_count cards of that resonance to the pack and
    deduct cost tokens. Auto-spends on the highest resonance.
    """
    # Determine bonus cards
    bonus_cards = []
    # Copy counters so we can deduct
    temp_counts = dict(player_counts)

    # Find the resonance with highest count
    if temp_counts:
        best_res = max(RESONANCES, key=lambda r: temp_counts.get(r, 0))
        while temp_counts.get(best_res, 0) >= cost:
            # Add bonus card of that resonance
            res_pool = [c for c in pool if best_res in c.symbols]
            if res_pool:
                for _ in range(bonus_count):
                    bonus_cards.append(rng.choice(res_pool))
            temp_counts[best_res] -= cost
            # Update the actual player_counts
            player_counts[best_res] = temp_counts[best_res]

    # Base pack: 4 random cards
    pack = rng.sample(pool, min(PACK_SIZE, len(pool)))
    # Add bonus cards
    pack.extend(bonus_cards)
    return pack

# --- Random baseline ---

def generate_pack_random(pool: list, rng: random.Random) -> list:
    return rng.sample(pool, min(PACK_SIZE, len(pool)))

# ---------------------------------------------------------------------------
# Player strategies
# ---------------------------------------------------------------------------

def _best_archetype_from_pairs(pair_counts: dict) -> Optional[int]:
    """Identify the archetype from pair counts."""
    if not pair_counts:
        return None
    pair_to_arch = {}
    for ai, (pri, sec) in ARCHETYPE_RESONANCES.items():
        pair_to_arch[(pri, sec)] = ai
    # Score each archetype by summing pair counts that map to it
    arch_scores = [0] * NUM_ARCHETYPES
    for (p, s), cnt in pair_counts.items():
        arch = pair_to_arch.get((p, s))
        if arch is not None:
            arch_scores[arch] += cnt
        # Also give partial credit for sharing primary
        for ai, (pri, sec) in ARCHETYPE_RESONANCES.items():
            if pri == p and ai != arch:
                arch_scores[ai] += cnt * 0.3
    if max(arch_scores) == 0:
        return None
    return arch_scores.index(max(arch_scores))

def _best_archetype_from_resonances(player_counts: dict) -> Optional[int]:
    """Identify the archetype from resonance counts."""
    if not player_counts or sum(player_counts.values()) == 0:
        return None
    scores = []
    for arch_idx in range(NUM_ARCHETYPES):
        pri, sec = ARCHETYPE_RESONANCES[arch_idx]
        score = player_counts.get(pri, 0) * 2 + player_counts.get(sec, 0)
        scores.append(score)
    return scores.index(max(scores))

def pick_committed(pack: list, pair_counts: dict, res_counts: dict,
                   target_arch: int, pick_num: int) -> tuple:
    """Archetype-committed player. Commits around pick 5-6.
    If target_arch is already set (forced), always uses it."""
    if target_arch is not None:
        # Already committed (forced or naturally determined)
        best = max(pack, key=lambda c: (c.archetype_fitness.get(target_arch, Tier.F).value,
                                         c.power))
        return best, target_arch
    if pick_num < 5:
        # Early: pick highest power, use emerging arch for tracking
        target = _best_archetype_from_pairs(pair_counts)
        if target is None:
            target = _best_archetype_from_resonances(res_counts)
        if target is not None:
            sa_cards = [c for c in pack
                        if c.archetype_fitness.get(target, Tier.F).value >= Tier.A.value]
            if sa_cards:
                return max(sa_cards, key=lambda c: c.power), None  # Don't lock in yet
        return max(pack, key=lambda c: c.power), None
    else:
        # Commit now
        target_arch = _best_archetype_from_pairs(pair_counts)
        if target_arch is None:
            target_arch = _best_archetype_from_resonances(res_counts)
        if target_arch is None:
            target_arch = 0
        best = max(pack, key=lambda c: (c.archetype_fitness.get(target_arch, Tier.F).value,
                                         c.power))
        return best, target_arch

def pick_power_chaser(pack: list, pair_counts: dict, res_counts: dict,
                      target_arch: int, pick_num: int) -> tuple:
    chosen = max(pack, key=lambda c: c.power)
    target = _best_archetype_from_pairs(pair_counts)
    if target is None:
        target = _best_archetype_from_resonances(res_counts)
    return chosen, target

def pick_signal_reader(pack: list, pair_counts: dict, res_counts: dict,
                       target_arch: int, pick_num: int,
                       seen_res_counts: dict = None) -> tuple:
    if pick_num < 5:
        # Track what's in the pack
        pack_res = defaultdict(int)
        for c in pack:
            for s in c.symbols:
                pack_res[s] += 1
        if pack_res:
            best_res = max(pack_res, key=pack_res.get)
            res_cards = [c for c in pack if best_res in c.symbols]
            return max(res_cards, key=lambda c: c.power), None
        return max(pack, key=lambda c: c.power), None
    else:
        if target_arch is None:
            target_arch = _best_archetype_from_pairs(pair_counts)
            if target_arch is None:
                target_arch = _best_archetype_from_resonances(res_counts)
            if target_arch is None:
                target_arch = 0
        best = max(pack, key=lambda c: (c.archetype_fitness.get(target_arch, Tier.F).value,
                                         c.power))
        return best, target_arch

# ---------------------------------------------------------------------------
# Simulation engine
# ---------------------------------------------------------------------------

@dataclass
class DraftResult:
    picks: list = field(default_factory=list)
    packs_seen: list = field(default_factory=list)
    target_archetype: int = 0
    strategy: str = ""
    pair_counts_history: list = field(default_factory=list)

def run_single_draft(pool: list, strategy: str, rng: random.Random,
                     algorithm: str = "pair_escalation",
                     forced_arch: int = None,
                     K: float = DEFAULT_K, cap: float = DEFAULT_CAP) -> DraftResult:
    """Run a single 30-pick draft."""
    pair_counts = {}
    res_counts = defaultdict(int)
    result = DraftResult(strategy=strategy)
    target_arch = forced_arch
    seen_res_counts = defaultdict(int)

    for pick_num in range(NUM_PICKS):
        # Generate pack based on algorithm
        if algorithm == "pair_escalation":
            pack = generate_pack_pair_escalation(pool, pair_counts, rng, K=K, cap=cap)
        elif algorithm == "lane_locking":
            pack = generate_pack_lane_locking(pool, res_counts, rng)
        elif algorithm == "pack_widening":
            pack = generate_pack_widening(pool, res_counts, rng)
        elif algorithm == "random":
            pack = generate_pack_random(pool, rng)
        else:
            pack = generate_pack_random(pool, rng)

        if not pack:
            break

        result.packs_seen.append(list(pack))
        result.pair_counts_history.append(dict(pair_counts))

        # Player picks a card
        if strategy == "committed":
            chosen, target_arch = pick_committed(pack, pair_counts, dict(res_counts),
                                                  target_arch, pick_num)
        elif strategy == "power":
            chosen, target_arch = pick_power_chaser(pack, pair_counts, dict(res_counts),
                                                     target_arch, pick_num)
        elif strategy == "signal":
            for c in pack:
                for s in c.symbols:
                    seen_res_counts[s] += 1
            if pick_num == 4 and target_arch is None:
                arch_scores = []
                for ai in range(NUM_ARCHETYPES):
                    pri, sec = ARCHETYPE_RESONANCES[ai]
                    score = seen_res_counts.get(pri, 0) * 2 + seen_res_counts.get(sec, 0)
                    arch_scores.append(score)
                target_arch = arch_scores.index(max(arch_scores))
            chosen, target_arch = pick_signal_reader(pack, pair_counts, dict(res_counts),
                                                      target_arch, pick_num,
                                                      seen_res_counts)
        else:
            chosen = pack[0]
            target_arch = 0

        result.picks.append(chosen)
        _update_pair_counters(chosen, pair_counts)
        _update_resonance_counters(chosen, res_counts)

    if target_arch is None:
        target_arch = _best_archetype_from_pairs(pair_counts)
        if target_arch is None:
            target_arch = _best_archetype_from_resonances(res_counts)
        if target_arch is None:
            target_arch = 0
    result.target_archetype = target_arch
    return result

# ---------------------------------------------------------------------------
# Metrics computation
# ---------------------------------------------------------------------------

def compute_metrics(results: list) -> dict:
    """Compute all 8 measurable targets + variance from a list of DraftResults."""
    early_unique_archetypes = []
    early_sa_for_emerging = []
    late_sa_for_committed = []
    late_off_archetype = []
    convergence_picks = []
    deck_concentrations = []
    all_final_decks = []
    archetype_frequencies = defaultdict(int)

    for dr in results:
        tgt = dr.target_archetype
        archetype_frequencies[tgt] += 1

        sa_streak = 0
        convergence_pick = NUM_PICKS

        for pick_num, pack in enumerate(dr.packs_seen):
            sa_count = sum(1 for c in pack
                          if c.archetype_fitness.get(tgt, Tier.F).value >= Tier.A.value)
            cf_count = sum(1 for c in pack
                          if c.archetype_fitness.get(tgt, Tier.F).value <= Tier.C.value)

            if pick_num < 5:
                unique_archs = set()
                for c in pack:
                    for ai in range(NUM_ARCHETYPES):
                        if c.archetype_fitness.get(ai, Tier.F).value >= Tier.A.value:
                            unique_archs.add(ai)
                early_unique_archetypes.append(len(unique_archs))
                early_sa_for_emerging.append(sa_count)
            else:
                late_sa_for_committed.append(sa_count)
                late_off_archetype.append(cf_count)

            if pick_num >= 5:
                if sa_count >= 2:
                    sa_streak += 1
                else:
                    sa_streak = 0
                if sa_streak >= 3 and convergence_pick == NUM_PICKS:
                    convergence_pick = pick_num - 2

        convergence_picks.append(convergence_pick)

        sa_in_deck = sum(1 for c in dr.picks
                        if c.archetype_fitness.get(tgt, Tier.F).value >= Tier.A.value)
        deck_concentrations.append(sa_in_deck / len(dr.picks) if dr.picks else 0)
        all_final_decks.append(set(c.id for c in dr.picks))

    # Run-to-run variety
    overlaps = []
    by_arch = defaultdict(list)
    for i, dr in enumerate(results):
        by_arch[dr.target_archetype].append(i)
    for arch, indices in by_arch.items():
        for i in range(min(50, len(indices))):
            for j in range(i + 1, min(50, len(indices))):
                s1 = all_final_decks[indices[i]]
                s2 = all_final_decks[indices[j]]
                if s1 and s2:
                    overlap = len(s1 & s2) / max(len(s1 | s2), 1)
                    overlaps.append(overlap)

    total_runs = len(results)
    arch_freq = {a: archetype_frequencies.get(a, 0) / total_runs for a in range(NUM_ARCHETYPES)}

    late_sa_stddev = statistics.stdev(late_sa_for_committed) if len(late_sa_for_committed) > 1 else 0

    return {
        "early_unique_archetypes": statistics.mean(early_unique_archetypes) if early_unique_archetypes else 0,
        "early_sa_emerging": statistics.mean(early_sa_for_emerging) if early_sa_for_emerging else 0,
        "late_sa_committed": statistics.mean(late_sa_for_committed) if late_sa_for_committed else 0,
        "late_off_archetype": statistics.mean(late_off_archetype) if late_off_archetype else 0,
        "convergence_pick": statistics.mean(convergence_picks) if convergence_picks else 30,
        "deck_concentration": statistics.mean(deck_concentrations) if deck_concentrations else 0,
        "run_to_run_overlap": statistics.mean(overlaps) if overlaps else 0,
        "arch_freq_max": max(arch_freq.values()) if arch_freq else 0,
        "arch_freq_min": min(arch_freq.values()) if arch_freq else 0,
        "arch_freq": arch_freq,
        "late_sa_stddev": late_sa_stddev,
        "late_sa_distribution": _distribution(late_sa_for_committed),
    }

def _distribution(values: list) -> dict:
    counts = defaultdict(int)
    for v in values:
        counts[v] += 1
    total = len(values) if values else 1
    return {k: counts[k] / total for k in sorted(counts.keys())}

def compute_per_archetype_convergence(pool: list, rng: random.Random,
                                       algorithm: str = "pair_escalation",
                                       n_drafts: int = 200,
                                       K: float = DEFAULT_K,
                                       cap: float = DEFAULT_CAP) -> dict:
    """For each archetype, run drafts and find average convergence pick."""
    convergence_table = {}
    for arch_idx in range(NUM_ARCHETYPES):
        conv_picks = []
        for _ in range(n_drafts):
            dr = run_single_draft(pool, "committed", rng, algorithm,
                                  forced_arch=arch_idx, K=K, cap=cap)
            sa_streak = 0
            conv = NUM_PICKS
            for pick_num in range(5, len(dr.packs_seen)):
                pack = dr.packs_seen[pick_num]
                sa = sum(1 for c in pack
                        if c.archetype_fitness.get(arch_idx, Tier.F).value >= Tier.A.value)
                if sa >= 2:
                    sa_streak += 1
                else:
                    sa_streak = 0
                if sa_streak >= 3 and conv == NUM_PICKS:
                    conv = pick_num - 2
            conv_picks.append(conv)
        convergence_table[arch_idx] = statistics.mean(conv_picks)
    return convergence_table

# ---------------------------------------------------------------------------
# Printing utilities
# ---------------------------------------------------------------------------

def _pass_fail(actual, target_min=None, target_max=None):
    if target_min is not None and actual < target_min:
        return "FAIL"
    if target_max is not None and actual > target_max:
        return "FAIL"
    return "PASS"

def print_scorecard(m: dict, label: str = ""):
    print(f"\n--- Scorecard: {label} ---")
    rows = [
        ("Picks 1-5 unique archetypes/pack", ">=3", m['early_unique_archetypes'], 3, None),
        ("Picks 1-5 S/A for emerging/pack", "<=2", m['early_sa_emerging'], None, 2),
        ("Picks 6+ S/A for committed/pack", ">=2", m['late_sa_committed'], 2, None),
        ("Picks 6+ off-archetype (C/F)/pack", ">=0.5", m['late_off_archetype'], 0.5, None),
        ("Convergence pick", "5-8", m['convergence_pick'], None, None),
        ("Deck concentration", "60-90%", m['deck_concentration'], None, None),
        ("Run-to-run overlap", "<40%", m['run_to_run_overlap'], None, None),
        ("Late S/A stddev", ">=0.8", m['late_sa_stddev'], 0.8, None),
    ]
    print(f"  {'Metric':45s} | {'Target':12s} | {'Actual':10s} | {'P/F':6s}")
    print(f"  {'-'*80}")
    for name, target, val, tmin, tmax in rows:
        if "concentration" in name or "overlap" in name:
            val_str = f"{val:.1%}"
        elif "pick" in name.lower() and "S/A" not in name:
            val_str = f"{val:.1f}"
        else:
            val_str = f"{val:.2f}"

        if tmin is not None:
            pf = _pass_fail(val, target_min=tmin)
        elif tmax is not None:
            pf = _pass_fail(val, target_max=tmax)
        elif "pick" in name.lower():
            pf = "PASS" if 5 <= val <= 8 else "FAIL"
        elif "concentration" in name:
            pf = "PASS" if 0.60 <= val <= 0.90 else "FAIL"
        elif "overlap" in name:
            pf = "PASS" if val < 0.40 else "FAIL"
        else:
            pf = "--"
        print(f"  {name:45s} | {target:12s} | {val_str:10s} | {pf:6s}")

    # Distribution
    if m.get('late_sa_distribution'):
        print(f"\n  S/A distribution (picks 6+):")
        for k, v in sorted(m['late_sa_distribution'].items()):
            bar = "#" * int(v * 50)
            print(f"    {k} S/A cards: {v:6.1%}  {bar}")

    # Archetype frequencies
    if m.get('arch_freq'):
        print(f"\n  Archetype frequencies:")
        for ai in range(NUM_ARCHETYPES):
            freq = m['arch_freq'].get(ai, 0)
            print(f"    {ARCHETYPES[ai]:30s}: {freq:.1%}")

def print_comparison(metrics_dict: dict):
    """Print side-by-side comparison of multiple algorithms."""
    algos = list(metrics_dict.keys())
    print(f"\n{'='*100}")
    print(f" SIDE-BY-SIDE COMPARISON")
    print(f"{'='*100}")

    header = f"  {'Metric':40s} | {'Target':8s}"
    for a in algos:
        header += f" | {a:12s}"
    print(header)
    print(f"  {'-'*len(header)}")

    metric_defs = [
        ("Early unique archetypes", ">=3", "early_unique_archetypes", 3, None),
        ("Early S/A emerging", "<=2", "early_sa_emerging", None, 2),
        ("Late S/A committed", ">=2", "late_sa_committed", 2, None),
        ("Late off-archetype", ">=0.5", "late_off_archetype", 0.5, None),
        ("Convergence pick", "5-8", "convergence_pick", None, None),
        ("Deck concentration", "60-90%", "deck_concentration", None, None),
        ("Run-to-run overlap", "<40%", "run_to_run_overlap", None, None),
        ("Late S/A stddev", ">=0.8", "late_sa_stddev", 0.8, None),
    ]

    for name, target, key, tmin, tmax in metric_defs:
        row = f"  {name:40s} | {target:8s}"
        for a in algos:
            m = metrics_dict[a]
            val = m[key]
            if "concentration" in key or "overlap" in key:
                val_str = f"{val:.1%}"
            elif "pick" in key:
                val_str = f"{val:.1f}"
            else:
                val_str = f"{val:.2f}"

            if tmin is not None:
                pf = "P" if val >= tmin else "F"
            elif tmax is not None:
                pf = "P" if val <= tmax else "F"
            elif "pick" in key:
                pf = "P" if 5 <= val <= 8 else "F"
            elif "concentration" in key:
                pf = "P" if 0.60 <= val <= 0.90 else "F"
            elif "overlap" in key:
                pf = "P" if val < 0.40 else "F"
            else:
                pf = "-"
            row += f" | {val_str:>8s} ({pf})"
        print(row)

def format_trace(dr: DraftResult, label: str) -> str:
    """Format a single draft as a readable trace."""
    lines = [f"\n=== Draft Trace: {label} (Strategy: {dr.strategy}) ==="]
    lines.append(f"Target archetype: {ARCHETYPES[dr.target_archetype]} (idx {dr.target_archetype})")
    lines.append(f"{'Pick':>4} | {'Chosen Card':45s} | {'Pack S/A':>7} | {'Pair State':30s} | {'Pack Summary'}")
    lines.append("-" * 160)

    for i, (pick, pack) in enumerate(zip(dr.picks, dr.packs_seen)):
        tgt = dr.target_archetype
        sa_count = sum(1 for c in pack
                      if c.archetype_fitness.get(tgt, Tier.F).value >= Tier.A.value)
        arch_label = pick.archetype if pick.archetype else "Generic"
        syms = "/".join(pick.symbols) if pick.symbols else "none"
        fitness = pick.archetype_fitness.get(tgt, Tier.F).name
        chosen_str = f"{arch_label} [{syms}] (fit={fitness})"

        # Pair state at this pick
        if i < len(dr.pair_counts_history):
            pc = dr.pair_counts_history[i]
            top_pair, top_count = _get_top_pair(pc)
            if top_pair:
                prob = min(top_count / DEFAULT_K, DEFAULT_CAP)
                pair_str = f"{top_pair}:{top_count} P={prob:.2f}"
            else:
                pair_str = "no pairs yet"
        else:
            pair_str = "?"

        pack_archs = []
        for c in pack:
            a = c.archetype[:10] if c.archetype else "Generic"
            f = c.archetype_fitness.get(tgt, Tier.F).name
            pack_archs.append(f"{a}({f})")
        pack_str = ", ".join(pack_archs)

        lines.append(f"{i+1:>4} | {chosen_str:45s} | {sa_count:>7} | {pair_str:30s} | {pack_str}")

    tgt = dr.target_archetype
    sa = sum(1 for c in dr.picks if c.archetype_fitness.get(tgt, Tier.F).value >= Tier.A.value)
    lines.append(f"\nFinal deck: {sa}/{len(dr.picks)} S/A cards for {ARCHETYPES[tgt]} "
                 f"({100*sa/len(dr.picks):.1f}%)")
    return "\n".join(lines)

# ---------------------------------------------------------------------------
# Full simulation runner
# ---------------------------------------------------------------------------

def run_simulation(pool: list, algorithm: str, rng: random.Random,
                   n_drafts: int = NUM_DRAFTS, K: float = DEFAULT_K,
                   cap: float = DEFAULT_CAP, label: str = "",
                   quiet: bool = False) -> tuple:
    """Run full simulation with all 3 strategies."""
    all_results = []
    strategies = ["committed", "power", "signal"]

    for strategy in strategies:
        for _ in range(n_drafts):
            dr = run_single_draft(pool, strategy, rng, algorithm,
                                  K=K, cap=cap)
            all_results.append(dr)

    committed = [r for r in all_results if r.strategy == "committed"]
    power = [r for r in all_results if r.strategy == "power"]
    signal = [r for r in all_results if r.strategy == "signal"]

    metrics = compute_metrics(committed)
    power_m = compute_metrics(power)
    signal_m = compute_metrics(signal)

    if not quiet:
        algo_name = algorithm.replace("_", " ").title()
        print(f"\n{'='*70}")
        print(f" {algo_name} Results {label}")
        print(f"{'='*70}")
        if algorithm == "pair_escalation":
            print(f" Params: K={K}, cap={cap}")
        print_scorecard(metrics, f"{algo_name} — Committed")
        print(f"\n  --- Power Chaser ---")
        print(f"  Late S/A: {power_m['late_sa_committed']:.2f}, "
              f"Deck conc: {power_m['deck_concentration']:.1%}")
        print(f"\n  --- Signal Reader ---")
        print(f"  Late S/A: {signal_m['late_sa_committed']:.2f}, "
              f"Deck conc: {signal_m['deck_concentration']:.1%}")

    return metrics, committed, all_results

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    print("=" * 80)
    print(" Resonance V5 — Agent 2: Pair-Escalation Slots v2")
    print("=" * 80)

    # Build pool
    rng_pool = random.Random(42)
    pool = build_card_pool(rng_pool, pct_1sym=0.15, pct_2sym=0.60, pct_3sym=0.25)
    analyze_pool(pool)

    # =================================================================
    # 1. PRIMARY ALGORITHM: Pair-Escalation Slots (K=6, cap=0.65)
    # =================================================================
    print("\n\n" + "=" * 80)
    print(" 1. PRIMARY: Pair-Escalation Slots v2 (K=6, cap=0.65)")
    print("=" * 80)
    pe_metrics, pe_committed, pe_all = run_simulation(
        pool, "pair_escalation", random.Random(100), K=6, cap=0.65)

    # =================================================================
    # 2. V3 LANE LOCKING BASELINE
    # =================================================================
    print("\n\n" + "=" * 80)
    print(" 2. BASELINE: V3 Lane Locking (threshold 3/8, primary=2)")
    print("=" * 80)
    ll_metrics, ll_committed, ll_all = run_simulation(
        pool, "lane_locking", random.Random(100))

    # =================================================================
    # 3. V4 PACK WIDENING BASELINE (auto-spend)
    # =================================================================
    print("\n\n" + "=" * 80)
    print(" 3. BASELINE: V4 Pack Widening (auto-spend, cost=3, bonus=1)")
    print("=" * 80)
    pw_metrics, pw_committed, pw_all = run_simulation(
        pool, "pack_widening", random.Random(100))

    # =================================================================
    # 4. RANDOM BASELINE
    # =================================================================
    print("\n\n" + "=" * 80)
    print(" 4. BASELINE: Random (no algorithm)")
    print("=" * 80)
    rand_metrics, rand_committed, rand_all = run_simulation(
        pool, "random", random.Random(100))

    # =================================================================
    # 5. SIDE-BY-SIDE COMPARISON
    # =================================================================
    print_comparison({
        "PairEsc(6,.65)": pe_metrics,
        "LaneLocking": ll_metrics,
        "PackWidening": pw_metrics,
        "Random": rand_metrics,
    })

    # =================================================================
    # 6. PER-ARCHETYPE CONVERGENCE TABLE
    # =================================================================
    print("\n\n" + "=" * 80)
    print(" 6. PER-ARCHETYPE CONVERGENCE TABLE")
    print("=" * 80)

    rng_conv = random.Random(200)
    pe_conv = compute_per_archetype_convergence(pool, rng_conv,
                                                 "pair_escalation", n_drafts=200, K=6, cap=0.65)
    ll_conv = compute_per_archetype_convergence(pool, random.Random(200),
                                                 "lane_locking", n_drafts=200)
    pw_conv = compute_per_archetype_convergence(pool, random.Random(200),
                                                 "pack_widening", n_drafts=200)

    print(f"\n  {'Archetype':30s} | {'PairEsc':>10s} | {'LaneLock':>10s} | {'PackWiden':>10s}")
    print(f"  {'-'*70}")
    for ai in range(NUM_ARCHETYPES):
        print(f"  {ARCHETYPES[ai]:30s} | {pe_conv[ai]:10.1f} | {ll_conv[ai]:10.1f} | {pw_conv[ai]:10.1f}")

    # =================================================================
    # 7. PARAMETER SENSITIVITY SWEEPS
    # =================================================================
    print("\n\n" + "=" * 80)
    print(" 7. PARAMETER SENSITIVITY SWEEPS")
    print("=" * 80)

    # 7a. K and cap sweep
    print("\n--- K/Cap Sweep ---")
    print(f"  {'K':>4s} {'Cap':>5s} | {'Late S/A':>8s} {'Off-arch':>8s} {'StdDev':>8s} {'DeckConc':>8s} {'ConvPick':>8s}")
    print(f"  {'-'*60}")
    for K_val, cap_val in [(5, 0.65), (6, 0.65), (7, 0.70), (4, 0.60), (6, 0.50), (6, 0.80), (8, 0.70)]:
        m, _, _ = run_simulation(pool, "pair_escalation", random.Random(100),
                                  K=K_val, cap=cap_val, quiet=True)
        print(f"  {K_val:>4d} {cap_val:>5.2f} | {m['late_sa_committed']:>8.2f} {m['late_off_archetype']:>8.2f} "
              f"{m['late_sa_stddev']:>8.2f} {m['deck_concentration']:>8.1%} {m['convergence_pick']:>8.1f}")

    # 7b. Symbol distribution sensitivity
    print("\n--- Symbol Distribution Sensitivity ---")
    dists = [
        (0.15, 0.60, 0.25, "15/60/25 (default)"),
        (0.30, 0.50, 0.20, "30/50/20 (more 1-sym)"),
        (0.10, 0.65, 0.25, "10/65/25 (fewer 1-sym)"),
        (0.05, 0.70, 0.25, "05/70/25 (min 1-sym)"),
        (0.15, 0.45, 0.40, "15/45/40 (more 3-sym)"),
    ]
    print(f"  {'Distribution':25s} | {'Late S/A':>8s} {'Off-arch':>8s} {'StdDev':>8s} {'DeckConc':>8s} {'ConvPick':>8s}")
    print(f"  {'-'*70}")
    for p1, p2, p3, desc in dists:
        test_pool = build_card_pool(random.Random(42), pct_1sym=p1, pct_2sym=p2, pct_3sym=p3)
        m, _, _ = run_simulation(test_pool, "pair_escalation", random.Random(100),
                                  K=6, cap=0.65, quiet=True)
        print(f"  {desc:25s} | {m['late_sa_committed']:>8.2f} {m['late_off_archetype']:>8.2f} "
              f"{m['late_sa_stddev']:>8.2f} {m['deck_concentration']:>8.1%} {m['convergence_pick']:>8.1f}")

    # =================================================================
    # 8. DRAFT TRACES
    # =================================================================
    print("\n\n" + "=" * 80)
    print(" 8. DRAFT TRACES")
    print("=" * 80)

    # Trace 1: Early committer (forced Warriors at idx 6)
    dr1 = run_single_draft(pool, "committed", random.Random(456),
                           "pair_escalation", forced_arch=6, K=6, cap=0.65)
    print(format_trace(dr1, "Early Committer (Warriors, forced arch=6)"))

    # Trace 2: Flexible player (no forced arch, stays flexible 8+ picks)
    dr2 = run_single_draft(pool, "committed", random.Random(789),
                           "pair_escalation", forced_arch=None, K=6, cap=0.65)
    print(format_trace(dr2, "Flexible Player (no forced arch)"))

    # Trace 3: Signal reader
    dr3 = run_single_draft(pool, "signal", random.Random(654),
                           "pair_escalation", K=6, cap=0.65)
    print(format_trace(dr3, "Signal Reader"))

    # =================================================================
    # 9. SUMMARY
    # =================================================================
    print("\n\n" + "=" * 80)
    print(" 9. SUMMARY")
    print("=" * 80)
    print("""
Algorithm: Pair-Escalation Slots v2
One-sentence: "Track the resonance pair (first, second symbol) of each 2+ symbol
card you draft; each pack slot independently shows a card matching your most
common pair with probability min(that pair's count / 6, 0.65), otherwise a
random card."

No player decisions beyond picking 1 card from pack.
Uses only visible card properties (ordered symbol pairs).
Fully automatic and passive.
""")

    print("\nSimulation complete.")
