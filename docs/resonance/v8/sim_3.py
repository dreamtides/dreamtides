#!/usr/bin/env python3
"""
Simulation Agent 3: Escalating Pair Lock
=========================================
Algorithm: Track drafted resonance pairs; slots unlock progressively at
pair-count thresholds (default 4/7/10). From pick 3+, a guaranteed floor
slot draws pair-matched. Locked slots draw pair-matched with 80-90%
probability (uniform jitter). Remaining slots are random.

Key mechanism: The pair counter increments by 1.0 for each dual-res card
picked matching any archetype pair, and by 0.5 for single-res cards
matching a pair's primary resonance. Pairs compete; the leading pair
determines locked-slot content. This creates a natural positive feedback
loop: once a pair leads, locked slots show that pair's cards, the player
picks them, and the counter accelerates.

Agreed conditions from Round 3:
- Fitness models: Optimistic, Graduated Realistic, Pessimistic, Hostile
- Pool compositions: V7 Standard (15% dual-res), 40% Enriched
"""

import random
import math
import statistics
from dataclasses import dataclass, field
from collections import defaultdict
from typing import Optional

# ── Constants ──────────────────────────────────────────────────────────

RESONANCES = ["Ember", "Stone", "Tide", "Zephyr"]

ARCHETYPES = [
    ("Flash", "Zephyr", "Ember"),
    ("Blink", "Ember", "Zephyr"),
    ("Storm", "Ember", "Stone"),
    ("Self-Discard", "Stone", "Ember"),
    ("Self-Mill", "Stone", "Tide"),
    ("Sacrifice", "Tide", "Stone"),
    ("Warriors", "Tide", "Zephyr"),
    ("Ramp", "Zephyr", "Tide"),
]

ARCHETYPE_NAMES = [a[0] for a in ARCHETYPES]
ARCHETYPE_MAP = {a[0]: (a[1], a[2]) for a in ARCHETYPES}

# All valid archetype pairs (ordered: primary, secondary)
ARCHETYPE_PAIRS = [(a[1], a[2]) for a in ARCHETYPES]
PAIR_TO_ARCHETYPE = {(a[1], a[2]): a[0] for a in ARCHETYPES}

# Co-primary pairs (share primary resonance)
CO_PRIMARY = {
    "Flash": "Ramp", "Ramp": "Flash",
    "Blink": "Storm", "Storm": "Blink",
    "Self-Discard": "Self-Mill", "Self-Mill": "Self-Discard",
    "Sacrifice": "Warriors", "Warriors": "Sacrifice",
}

PACK_SIZE = 4
NUM_PICKS = 30
NUM_DRAFTS = 1000
POOL_SIZE = 360

# ── Fitness Models ─────────────────────────────────────────────────────

FITNESS_MODELS = {
    "Optimistic": {
        ("Warriors", "Sacrifice"): 1.0, ("Sacrifice", "Warriors"): 1.0,
        ("Self-Discard", "Self-Mill"): 1.0, ("Self-Mill", "Self-Discard"): 1.0,
        ("Blink", "Storm"): 1.0, ("Storm", "Blink"): 1.0,
        ("Flash", "Ramp"): 1.0, ("Ramp", "Flash"): 1.0,
    },
    "Graduated Realistic": {
        ("Warriors", "Sacrifice"): 0.50, ("Sacrifice", "Warriors"): 0.50,
        ("Self-Discard", "Self-Mill"): 0.40, ("Self-Mill", "Self-Discard"): 0.40,
        ("Blink", "Storm"): 0.30, ("Storm", "Blink"): 0.30,
        ("Flash", "Ramp"): 0.25, ("Ramp", "Flash"): 0.25,
    },
    "Pessimistic": {
        ("Warriors", "Sacrifice"): 0.35, ("Sacrifice", "Warriors"): 0.35,
        ("Self-Discard", "Self-Mill"): 0.25, ("Self-Mill", "Self-Discard"): 0.25,
        ("Blink", "Storm"): 0.15, ("Storm", "Blink"): 0.15,
        ("Flash", "Ramp"): 0.10, ("Ramp", "Flash"): 0.10,
    },
    "Hostile": {
        ("Warriors", "Sacrifice"): 0.08, ("Sacrifice", "Warriors"): 0.08,
        ("Self-Discard", "Self-Mill"): 0.08, ("Self-Mill", "Self-Discard"): 0.08,
        ("Blink", "Storm"): 0.08, ("Storm", "Blink"): 0.08,
        ("Flash", "Ramp"): 0.08, ("Ramp", "Flash"): 0.08,
    },
}


@dataclass
class SimCard:
    card_id: int
    symbols: list  # ordered resonance symbols
    archetype: str  # primary archetype (home archetype)
    power: float  # raw card strength 0-10
    rarity: str  # "common", "uncommon", "rare"

    def has_pair(self, primary: str, secondary: str) -> bool:
        if len(self.symbols) < 2:
            return False
        return self.symbols[0] == primary and self.symbols[1] == secondary

    def has_primary(self, res: str) -> bool:
        return len(self.symbols) > 0 and self.symbols[0] == res


def card_tier(card: SimCard, player_archetype: str, fitness_model: dict) -> str:
    """Determine card tier for a given player archetype."""
    if card.archetype == player_archetype:
        return "S"
    sibling = CO_PRIMARY.get(player_archetype)
    if card.archetype == sibling:
        rate = fitness_model.get((player_archetype, sibling), 0.0)
        if (card.card_id * 7 + hash(player_archetype)) % 1000 < rate * 1000:
            return "A"
        else:
            return "B"
    if card.archetype == "Generic":
        if (card.card_id * 13 + hash(player_archetype)) % 100 < 40:
            return "A"
        return "B"
    player_primary, player_secondary = ARCHETYPE_MAP[player_archetype]
    card_primary = ARCHETYPE_MAP.get(card.archetype, (None, None))[0]
    if card_primary == player_secondary:
        if (card.card_id * 17 + hash(player_archetype)) % 100 < 10:
            return "A"
        return "C"
    return "C"


def is_sa(card: SimCard, player_archetype: str, fitness_model: dict) -> bool:
    return card_tier(card, player_archetype, fitness_model) in ("S", "A")


# ── Pool Construction ──────────────────────────────────────────────────

def build_pool(dual_res_pct: float) -> list:
    """Build a 360-card pool with configurable dual-resonance percentage."""
    cards = []
    card_id = 0

    num_generic = 40
    num_archetype_cards = POOL_SIZE - num_generic  # 320
    cards_per_archetype = num_archetype_cards // 8  # 40

    total_dual_res = int(num_archetype_cards * dual_res_pct)
    dual_per_archetype = total_dual_res // 8
    single_per_archetype = cards_per_archetype - dual_per_archetype

    for arch_name, primary, secondary in ARCHETYPES:
        for i in range(single_per_archetype):
            power = random.gauss(5.0, 1.5)
            power = max(1.0, min(9.5, power))
            rarity = "common" if i < single_per_archetype * 0.6 else (
                "uncommon" if i < single_per_archetype * 0.85 else "rare")
            cards.append(SimCard(
                card_id=card_id, symbols=[primary],
                archetype=arch_name, power=power, rarity=rarity
            ))
            card_id += 1

        for i in range(dual_per_archetype):
            power = random.gauss(5.5, 1.5)
            power = max(1.0, min(9.5, power))
            rarity = "common" if i < dual_per_archetype * 0.5 else (
                "uncommon" if i < dual_per_archetype * 0.75 else "rare")
            cards.append(SimCard(
                card_id=card_id, symbols=[primary, secondary],
                archetype=arch_name, power=power, rarity=rarity
            ))
            card_id += 1

    for i in range(num_generic):
        power = random.gauss(4.5, 1.5)
        power = max(1.0, min(9.0, power))
        cards.append(SimCard(
            card_id=card_id, symbols=[],
            archetype="Generic", power=power, rarity="common"
        ))
        card_id += 1

    return cards


def build_subpools(pool: list) -> dict:
    subpools = {
        "pair": defaultdict(list),
        "primary": defaultdict(list),
        "all": pool[:],
    }
    for card in pool:
        if len(card.symbols) >= 1:
            subpools["primary"][card.symbols[0]].append(card)
        if len(card.symbols) >= 2:
            subpools["pair"][(card.symbols[0], card.symbols[1])].append(card)
    return subpools


# ── Escalating Pair Lock Algorithm ─────────────────────────────────────

@dataclass
class EscalatingPairLockState:
    """Algorithm state tracking pair counters and lock status.

    The pair counter advances for ALL valid archetype pairs simultaneously.
    When a player picks a dual-res card with symbols (X, Y), the counter
    for pair (X, Y) increments by 1.0. When a single-res card with symbol
    X is picked, ALL pairs with X as primary get +0.5 credit.

    This means: picking a Zephyr single-res card gives +0.5 to both
    Flash(Ze/Em) and Ramp(Ze/Ti) pair counters. The pair that accumulates
    the most evidence wins. Once a pair leads, its locked slots show
    pair-matched cards, creating positive feedback.
    """
    pair_counters: dict = field(default_factory=lambda: defaultdict(float))
    leading_pair: Optional[tuple] = None
    locked_slots: int = 0
    picks_made: int = 0
    # Parameters
    thresholds: tuple = (4, 7, 10)
    lock_prob_range: tuple = (0.80, 0.90)
    floor_start_pick: int = 3


def update_pair_counters(state: EscalatingPairLockState, card: SimCard):
    """Update ALL pair counters after drafting a card."""
    state.picks_made += 1

    if len(card.symbols) >= 2:
        pair = (card.symbols[0], card.symbols[1])
        # Only count valid archetype pairs
        if pair in PAIR_TO_ARCHETYPE:
            state.pair_counters[pair] += 1.0
    elif len(card.symbols) == 1:
        # Single-res: give 0.5 credit to ALL pairs with this as primary
        res = card.symbols[0]
        for pair in ARCHETYPE_PAIRS:
            if pair[0] == res:
                state.pair_counters[pair] += 0.5

    # Update leading pair (highest counter)
    if state.pair_counters:
        best_pair = max(state.pair_counters, key=state.pair_counters.get)
        state.leading_pair = best_pair
        count = state.pair_counters[best_pair]

        if count >= state.thresholds[2]:
            state.locked_slots = 3
        elif count >= state.thresholds[1]:
            state.locked_slots = 2
        elif count >= state.thresholds[0]:
            state.locked_slots = 1
        else:
            state.locked_slots = 0


def generate_pack(state: EscalatingPairLockState, subpools: dict, pool: list) -> list:
    """Generate a 4-card pack using the Escalating Pair Lock algorithm."""
    pack = []
    pair = state.leading_pair

    effective_locks = state.locked_slots
    if state.picks_made >= state.floor_start_pick and pair is not None:
        effective_locks = max(effective_locks, 1)

    lock_prob = random.uniform(state.lock_prob_range[0], state.lock_prob_range[1])

    pair_pool = subpools["pair"].get(pair, []) if pair else []

    for slot in range(PACK_SIZE):
        if slot < effective_locks and pair_pool:
            if random.random() < lock_prob:
                card = random.choice(pair_pool)
            else:
                card = random.choice(pool)
        else:
            card = random.choice(pool)
        pack.append(card)

    return pack


# ── Player Strategies ──────────────────────────────────────────────────

def pick_archetype_committed(pack: list, player_arch: str,
                             fitness_model: dict, commitment_pick: int,
                             pick_num: int) -> SimCard:
    """Commits at commitment_pick. Post-commitment: pick best for archetype."""
    if pick_num < commitment_pick:
        return max(pack, key=lambda c: c.power)
    def score(c):
        t = card_tier(c, player_arch, fitness_model)
        tier_score = {"S": 4, "A": 3, "B": 1, "C": 0}.get(t, 0)
        return tier_score * 10 + c.power
    return max(pack, key=score)


def pick_power_chaser(pack: list) -> SimCard:
    return max(pack, key=lambda c: c.power)


def pick_signal_reader(pack: list, resonance_counts: dict,
                       fitness_model: dict, pick_num: int) -> SimCard:
    """Evaluates which archetype seems most available."""
    if pick_num < 4:
        return max(pack, key=lambda c: c.power)

    best_arch = None
    best_score = -1
    for arch_name, primary, secondary in ARCHETYPES:
        score = resonance_counts.get(primary, 0) * 2 + resonance_counts.get(secondary, 0)
        if score > best_score:
            best_score = score
            best_arch = arch_name

    if best_arch is None:
        return max(pack, key=lambda c: c.power)

    def score(c):
        t = card_tier(c, best_arch, fitness_model)
        tier_score = {"S": 4, "A": 3, "B": 1, "C": 0}.get(t, 0)
        return tier_score * 10 + c.power
    return max(pack, key=score)


# ── Draft Simulation ───────────────────────────────────────────────────

@dataclass
class DraftResult:
    picks: list = field(default_factory=list)
    packs: list = field(default_factory=list)
    player_archetype: str = ""
    strategy: str = ""
    sa_per_pack: list = field(default_factory=list)
    convergence_pick: int = 0
    deck_sa_count: int = 0
    deck_total: int = 0
    unique_archetypes_early: list = field(default_factory=list)
    sa_early: list = field(default_factory=list)
    locked_pair: Optional[tuple] = None
    pair_aligned: bool = False  # True if leading pair matches player archetype


def run_draft(pool: list, subpools: dict, fitness_model: dict,
              strategy: str, player_archetype: str,
              thresholds=(4, 7, 10), lock_prob_range=(0.80, 0.90),
              floor_start_pick=3) -> DraftResult:
    """Run a single draft of 30 picks."""
    state = EscalatingPairLockState(
        thresholds=thresholds,
        lock_prob_range=lock_prob_range,
        floor_start_pick=floor_start_pick,
    )
    result = DraftResult(player_archetype=player_archetype, strategy=strategy)

    resonance_counts = defaultdict(int)
    commitment_pick = random.randint(4, 6)
    converged = False

    target_pair = (ARCHETYPE_MAP[player_archetype][0],
                   ARCHETYPE_MAP[player_archetype][1])

    for pick_num in range(NUM_PICKS):
        pack = generate_pack(state, subpools, pool)
        result.packs.append(pack)

        sa_count = sum(1 for c in pack if is_sa(c, player_archetype, fitness_model))

        if strategy == "archetype-committed":
            chosen = pick_archetype_committed(
                pack, player_archetype, fitness_model,
                commitment_pick, pick_num)
        elif strategy == "power-chaser":
            chosen = pick_power_chaser(pack)
        else:
            chosen = pick_signal_reader(pack, resonance_counts, fitness_model, pick_num)

        result.picks.append(chosen)

        for sym in chosen.symbols:
            resonance_counts[sym] += 1

        update_pair_counters(state, chosen)

        # Track metrics
        if pick_num < 5:
            archs_with_sa = set()
            sa_for_emerging = 0
            for c in pack:
                for arch in ARCHETYPE_NAMES:
                    if is_sa(c, arch, fitness_model):
                        archs_with_sa.add(arch)
                if is_sa(c, player_archetype, fitness_model):
                    sa_for_emerging += 1
            result.unique_archetypes_early.append(len(archs_with_sa))
            result.sa_early.append(sa_for_emerging)
        else:
            result.sa_per_pack.append(sa_count)

        if not converged and pick_num >= 4 and len(result.sa_per_pack) >= 2:
            recent = result.sa_per_pack[-3:] if len(result.sa_per_pack) >= 3 else result.sa_per_pack
            if statistics.mean(recent) >= 1.5:
                result.convergence_pick = pick_num + 1
                converged = True

    if not converged:
        result.convergence_pick = NUM_PICKS

    result.locked_pair = state.leading_pair
    result.pair_aligned = (state.leading_pair == target_pair)

    for card in result.picks:
        result.deck_total += 1
        if is_sa(card, player_archetype, fitness_model):
            result.deck_sa_count += 1

    return result


# ── Metrics Computation ────────────────────────────────────────────────

def compute_metrics(results: list) -> dict:
    metrics = {}

    # M1: Picks 1-5, unique archetypes with S/A cards per pack
    m1_vals = [statistics.mean(r.unique_archetypes_early)
               for r in results if r.unique_archetypes_early]
    metrics["M1"] = statistics.mean(m1_vals) if m1_vals else 0

    # M2: Picks 1-5, S/A for emerging archetype per pack
    m2_vals = [statistics.mean(r.sa_early) for r in results if r.sa_early]
    metrics["M2"] = statistics.mean(m2_vals) if m2_vals else 0

    # M3: Picks 6+, S/A for committed archetype per pack (avg)
    m3_vals = [statistics.mean(r.sa_per_pack) for r in results if r.sa_per_pack]
    metrics["M3"] = statistics.mean(m3_vals) if m3_vals else 0

    # M4: Off-archetype cards per pack (approximated)
    metrics["M4"] = max(0, 4.0 - metrics["M3"] - 0.5)

    # M5: Convergence pick
    m5_vals = [r.convergence_pick for r in results if r.convergence_pick > 0]
    metrics["M5"] = statistics.mean(m5_vals) if m5_vals else NUM_PICKS

    # M6: Deck archetype concentration
    m6_vals = [r.deck_sa_count / r.deck_total for r in results if r.deck_total > 0]
    metrics["M6"] = statistics.mean(m6_vals) if m6_vals else 0

    # M7: Run-to-run variety
    by_arch = defaultdict(list)
    for r in results:
        card_ids = set(c.card_id for c in r.picks)
        by_arch[r.player_archetype].append(card_ids)
    overlaps = []
    for arch, runs in by_arch.items():
        for i in range(min(len(runs) - 1, 50)):
            if i + 1 < len(runs):
                intersection = len(runs[i] & runs[i + 1])
                if len(runs[i]) > 0:
                    overlaps.append(intersection / len(runs[i]))
    metrics["M7"] = statistics.mean(overlaps) if overlaps else 0

    # M8: Archetype frequency
    arch_freq = defaultdict(int)
    for r in results:
        arch_freq[r.player_archetype] += 1
    total_runs = len(results)
    if total_runs > 0 and arch_freq:
        metrics["M8_max"] = max(arch_freq.values()) / total_runs
        metrics["M8_min"] = min(arch_freq.values()) / total_runs
    else:
        metrics["M8_max"] = metrics["M8_min"] = 0

    # M9: StdDev of S/A per pack (picks 6+)
    all_sa = []
    for r in results:
        all_sa.extend(r.sa_per_pack)
    metrics["M9"] = statistics.stdev(all_sa) if len(all_sa) > 1 else 0

    # M10: Consecutive bad packs for committed players
    max_consec_vals = []
    for r in results:
        if r.strategy == "archetype-committed":
            max_consec = 0
            current = 0
            for sa in r.sa_per_pack:
                if sa < 1.5:
                    current += 1
                    max_consec = max(max_consec, current)
                else:
                    current = 0
            max_consec_vals.append(max_consec)
    metrics["M10_avg"] = statistics.mean(max_consec_vals) if max_consec_vals else 0
    metrics["M10_worst"] = max(max_consec_vals) if max_consec_vals else 0

    # Pair alignment rate
    aligned = [r for r in results if r.pair_aligned]
    metrics["pair_align_rate"] = len(aligned) / len(results) if results else 0

    return metrics


def compute_per_archetype_m3(results: list) -> dict:
    """Compute M3 per archetype (committed players only)."""
    arch_m3 = {}
    for arch in ARCHETYPE_NAMES:
        arch_results = [r for r in results
                        if r.player_archetype == arch and r.strategy == "archetype-committed"]
        if arch_results:
            m3_vals = [statistics.mean(r.sa_per_pack) for r in arch_results if r.sa_per_pack]
            arch_m3[arch] = statistics.mean(m3_vals) if m3_vals else 0
        else:
            arch_m3[arch] = 0
    return arch_m3


def compute_per_archetype_convergence(results: list) -> dict:
    """Compute convergence pick per archetype (committed players only)."""
    arch_conv = {}
    for arch in ARCHETYPE_NAMES:
        arch_results = [r for r in results
                        if r.player_archetype == arch and r.strategy == "archetype-committed"]
        if arch_results:
            arch_conv[arch] = statistics.mean([r.convergence_pick for r in arch_results])
        else:
            arch_conv[arch] = 0
    return arch_conv


def compute_pack_quality_distribution(results: list) -> dict:
    all_sa = []
    for r in results:
        if r.strategy == "archetype-committed":
            all_sa.extend(r.sa_per_pack)
    if not all_sa:
        return {}
    all_sa.sort()
    n = len(all_sa)
    return {
        "p10": all_sa[int(n * 0.10)],
        "p25": all_sa[int(n * 0.25)],
        "p50": all_sa[int(n * 0.50)],
        "p75": all_sa[int(n * 0.75)],
        "p90": all_sa[int(n * 0.90)],
        "mean": statistics.mean(all_sa),
    }


def compute_consecutive_bad_packs(results: list) -> dict:
    all_max_consec = []
    all_avg_consec = []
    for r in results:
        if r.strategy == "archetype-committed":
            streaks = []
            current = 0
            for sa in r.sa_per_pack:
                if sa < 1.5:
                    current += 1
                else:
                    if current > 0:
                        streaks.append(current)
                    current = 0
            if current > 0:
                streaks.append(current)
            max_consec = max(streaks) if streaks else 0
            avg_consec = statistics.mean(streaks) if streaks else 0
            all_max_consec.append(max_consec)
            all_avg_consec.append(avg_consec)
    return {
        "avg_worst_streak": statistics.mean(all_max_consec) if all_max_consec else 0,
        "global_worst_streak": max(all_max_consec) if all_max_consec else 0,
        "avg_streak_length": statistics.mean(all_avg_consec) if all_avg_consec else 0,
    }


# ── Draft Trace ────────────────────────────────────────────────────────

def format_draft_trace(result: DraftResult, fitness_model: dict,
                       thresholds=(4, 7, 10)) -> str:
    lines = []
    lines.append(f"=== Draft Trace: {result.strategy} ({result.player_archetype}) ===")
    lines.append(f"Target pair: {ARCHETYPE_MAP[result.player_archetype]}")
    lines.append(f"Final locked pair: {result.locked_pair} (aligned={result.pair_aligned})")
    lines.append(f"Convergence pick: {result.convergence_pick}")
    lines.append(f"Deck S/A: {result.deck_sa_count}/{result.deck_total}")

    state = EscalatingPairLockState(thresholds=thresholds,
                                    lock_prob_range=(0.80, 0.90),
                                    floor_start_pick=3)
    for i, (pack, pick) in enumerate(zip(result.packs, result.picks)):
        sa_count = sum(1 for c in pack if is_sa(c, result.player_archetype, fitness_model))
        tiers = [card_tier(c, result.player_archetype, fitness_model) for c in pack]
        pick_tier = card_tier(pick, result.player_archetype, fitness_model)

        target = ARCHETYPE_MAP[result.player_archetype]
        target_ct = state.pair_counters.get(target, 0)
        lead_ct = state.pair_counters.get(state.leading_pair, 0) if state.leading_pair else 0

        if i < 20 or i == 29:
            lines.append(
                f"  Pick {i+1:2d}: [{'/'.join(tiers)}] SA={sa_count} "
                f"locks={state.locked_slots} "
                f"lead={state.leading_pair} lead_ct={lead_ct:.1f} "
                f"target_ct={target_ct:.1f} "
                f"-> {pick.archetype}({pick_tier}) syms={pick.symbols}"
            )
        update_pair_counters(state, pick)

    return "\n".join(lines)


# ── Main Simulation ────────────────────────────────────────────────────

def run_simulation_batch(pool_label: str, dual_res_pct: float,
                         fitness_label: str, fitness_model: dict,
                         thresholds=(4, 7, 10),
                         lock_prob_range=(0.80, 0.90),
                         floor_start_pick=3,
                         seed=42) -> dict:
    random.seed(seed)
    pool = build_pool(dual_res_pct)
    subpools = build_subpools(pool)

    all_results = []
    strategies = ["archetype-committed", "power-chaser", "signal-reader"]

    for draft_num in range(NUM_DRAFTS):
        player_arch = ARCHETYPE_NAMES[draft_num % 8]
        strategy = strategies[draft_num % 3]

        result = run_draft(
            pool, subpools, fitness_model, strategy, player_arch,
            thresholds=thresholds, lock_prob_range=lock_prob_range,
            floor_start_pick=floor_start_pick,
        )
        all_results.append(result)

    # Overall metrics
    metrics = compute_metrics(all_results)

    # Committed-only metrics (the primary evaluation)
    committed = [r for r in all_results if r.strategy == "archetype-committed"]
    committed_metrics = compute_metrics(committed)

    arch_m3 = compute_per_archetype_m3(all_results)
    arch_conv = compute_per_archetype_convergence(all_results)
    pack_dist = compute_pack_quality_distribution(all_results)
    consec_bad = compute_consecutive_bad_packs(all_results)

    strat_metrics = {}
    for strat in strategies:
        strat_results = [r for r in all_results if r.strategy == strat]
        strat_metrics[strat] = compute_metrics(strat_results)

    # Draft traces
    traces = []
    for strat in strategies:
        strat_results = [r for r in all_results if r.strategy == strat]
        if strat_results:
            traces.append(format_draft_trace(strat_results[0], fitness_model,
                                             thresholds))

    return {
        "pool_label": pool_label,
        "fitness_label": fitness_label,
        "metrics": metrics,
        "committed_metrics": committed_metrics,
        "arch_m3": arch_m3,
        "arch_conv": arch_conv,
        "pack_dist": pack_dist,
        "consec_bad": consec_bad,
        "strat_metrics": strat_metrics,
        "traces": traces,
    }


def run_parameter_sweep(dual_res_pct: float, fitness_label: str,
                        fitness_model: dict) -> list:
    sweep_results = []

    # Sweep 1: Threshold variants
    for name, thresholds in [("Conservative (5/9/13)", (5, 9, 13)),
                              ("Balanced (4/7/10)", (4, 7, 10)),
                              ("Aggressive (3/5/8)", (3, 5, 8))]:
        b = run_simulation_batch(
            "40% Enriched", dual_res_pct, fitness_label, fitness_model,
            thresholds=thresholds)
        cm = b["committed_metrics"]
        sweep_results.append({
            "variant": f"Thresholds: {name}",
            "M3": cm["M3"], "M5": cm["M5"], "M9": cm["M9"],
            "M10_avg": cm["M10_avg"],
            "M6": cm["M6"], "pair_align": cm["pair_align_rate"],
        })

    # Sweep 2: Lock probability variants
    for name, prob_range in [("Low (70-80%)", (0.70, 0.80)),
                              ("Default (80-90%)", (0.80, 0.90)),
                              ("High (90-100%)", (0.90, 1.00))]:
        b = run_simulation_batch(
            "40% Enriched", dual_res_pct, fitness_label, fitness_model,
            lock_prob_range=prob_range)
        cm = b["committed_metrics"]
        sweep_results.append({
            "variant": f"Lock prob: {name}",
            "M3": cm["M3"], "M5": cm["M5"], "M9": cm["M9"],
            "M10_avg": cm["M10_avg"],
            "M6": cm["M6"], "pair_align": cm["pair_align_rate"],
        })

    # Sweep 3: Floor start pick
    for name, floor_pick in [("No floor", 999),
                              ("Floor from pick 2", 2),
                              ("Floor from pick 3 (default)", 3),
                              ("Floor from pick 5", 5)]:
        b = run_simulation_batch(
            "40% Enriched", dual_res_pct, fitness_label, fitness_model,
            floor_start_pick=floor_pick)
        cm = b["committed_metrics"]
        sweep_results.append({
            "variant": f"Floor: {name}",
            "M3": cm["M3"], "M5": cm["M5"], "M9": cm["M9"],
            "M10_avg": cm["M10_avg"],
            "M6": cm["M6"], "pair_align": cm["pair_align_rate"],
        })

    return sweep_results


def print_metrics(m: dict, label: str = ""):
    if label:
        print(f"\n  {label}")
    print(f"  M1={m['M1']:.2f} M2={m['M2']:.2f} M3={m['M3']:.2f} "
          f"M4={m['M4']:.2f} M5={m['M5']:.1f} M6={m['M6']:.1%} "
          f"M7={m['M7']:.1%} M9={m['M9']:.2f} "
          f"M10avg={m['M10_avg']:.2f} align={m['pair_align_rate']:.1%}")


def main():
    print("=" * 70)
    print("ESCALATING PAIR LOCK — V8 Simulation Agent 3")
    print("=" * 70)

    pool_configs = [
        ("V7 Standard (15%)", 0.15),
        ("40% Enriched", 0.40),
    ]

    all_batches = {}

    for pool_label, dual_pct in pool_configs:
        for fitness_label, fitness_model in FITNESS_MODELS.items():
            print(f"\nRunning: {pool_label} x {fitness_label}...")
            batch = run_simulation_batch(
                pool_label, dual_pct, fitness_label, fitness_model)
            key = (pool_label, fitness_label)
            all_batches[key] = batch
            print_metrics(batch["committed_metrics"], "Committed:")
            print_metrics(batch["metrics"], "All strategies:")

    # ── Detailed Results ──
    print("\n" + "=" * 70)
    print("DETAILED RESULTS (Committed Players Only)")
    print("=" * 70)

    for (pool_label, fitness_label), batch in all_batches.items():
        m = batch["committed_metrics"]
        print(f"\n{'─' * 60}")
        print(f"Pool: {pool_label} | Fitness: {fitness_label}")
        print(f"{'─' * 60}")

        pass_fail = lambda val, cond: "PASS" if cond else "FAIL"

        print(f"  M1  unique archs early:     {m['M1']:.2f}  {pass_fail(m['M1'], m['M1'] >= 3)}")
        print(f"  M2  SA early per pack:       {m['M2']:.2f}  {pass_fail(m['M2'], m['M2'] <= 2)}")
        print(f"  M3  SA post-commit avg:      {m['M3']:.2f}  {pass_fail(m['M3'], m['M3'] >= 2.0)}")
        print(f"  M4  off-arch per pack:       {m['M4']:.2f}  {pass_fail(m['M4'], m['M4'] >= 0.5)}")
        print(f"  M5  convergence pick:        {m['M5']:.1f}   {pass_fail(m['M5'], 5 <= m['M5'] <= 8)}")
        print(f"  M6  deck concentration:      {m['M6']:.1%}  {pass_fail(m['M6'], 0.6 <= m['M6'] <= 0.9)}")
        print(f"  M7  run overlap:             {m['M7']:.1%}  {pass_fail(m['M7'], m['M7'] < 0.4)}")
        print(f"  M8  arch freq max/min:       {m['M8_max']:.1%}/{m['M8_min']:.1%}")
        print(f"  M9  SA stddev:               {m['M9']:.2f}  {pass_fail(m['M9'], m['M9'] >= 0.8)}")
        print(f"  M10 consec bad avg/worst:    {m['M10_avg']:.2f}/{m['M10_worst']}")
        print(f"  Pair alignment rate:         {m['pair_align_rate']:.1%}")

        # Per-archetype M3
        print(f"\n  Per-archetype M3 (committed):")
        for arch in ARCHETYPE_NAMES:
            m3 = batch["arch_m3"].get(arch, 0)
            conv = batch["arch_conv"].get(arch, 0)
            print(f"    {arch:15s}: M3={m3:.2f}  conv={conv:.1f}")

        # Pack quality distribution
        pd = batch["pack_dist"]
        if pd:
            print(f"\n  Pack quality distribution (committed, picks 6+):")
            print(f"    P10={pd['p10']:.1f}  P25={pd['p25']:.1f}  P50={pd['p50']:.1f}  "
                  f"P75={pd['p75']:.1f}  P90={pd['p90']:.1f}  Mean={pd['mean']:.2f}")

        # Consecutive bad packs
        cb = batch["consec_bad"]
        print(f"\n  Consecutive bad packs (S/A < 1.5, committed):")
        print(f"    Avg worst streak: {cb['avg_worst_streak']:.2f}")
        print(f"    Global worst: {cb['global_worst_streak']}")
        print(f"    Avg streak len: {cb['avg_streak_length']:.2f}")

    # ── Parameter Sensitivity ──
    print("\n" + "=" * 70)
    print("PARAMETER SENSITIVITY (40% Enriched, Graduated Realistic, committed)")
    print("=" * 70)

    sweep = run_parameter_sweep(0.40, "Graduated Realistic",
                                FITNESS_MODELS["Graduated Realistic"])
    print(f"\n{'Variant':45s} {'M3':>6s} {'M5':>6s} {'M9':>6s} {'M10':>6s} {'M6':>6s} {'Align':>6s}")
    print("─" * 82)
    for s in sweep:
        print(f"  {s['variant']:43s} {s['M3']:6.2f} {s['M5']:6.1f} "
              f"{s['M9']:6.2f} {s['M10_avg']:6.2f} {s['M6']:6.1%} {s['pair_align']:6.1%}")

    # ── Fitness Degradation ──
    print("\n" + "=" * 70)
    print("FITNESS DEGRADATION CURVE (committed players)")
    print("=" * 70)

    for pool_label in ["40% Enriched", "V7 Standard (15%)"]:
        print(f"\n  {pool_label}:")
        print(f"  {'Fitness':25s} {'M3':>6s} {'M5':>6s} {'M6':>6s} {'M10':>6s} {'Align':>6s} {'Worst Arch M3':>14s}")
        print(f"  {'─'*75}")
        for fl in ["Optimistic", "Graduated Realistic", "Pessimistic", "Hostile"]:
            key = (pool_label, fl)
            if key not in all_batches:
                continue
            b = all_batches[key]
            cm = b["committed_metrics"]
            worst_arch = min(b["arch_m3"], key=b["arch_m3"].get) if b["arch_m3"] else "?"
            worst_val = b["arch_m3"].get(worst_arch, 0)
            print(f"  {fl:25s} {cm['M3']:6.2f} {cm['M5']:6.1f} "
                  f"{cm['M6']:6.1%} {cm['M10_avg']:6.2f} {cm['pair_align_rate']:6.1%} "
                  f"{worst_arch}={worst_val:.2f}")

    # ── Draft Traces ──
    print("\n" + "=" * 70)
    print("DRAFT TRACES (40% Enriched, Graduated Realistic)")
    print("=" * 70)
    key = ("40% Enriched", "Graduated Realistic")
    for trace in all_batches[key]["traces"]:
        print(f"\n{trace}")

    # ── Pack Quality Histogram ──
    print("\n" + "=" * 70)
    print("PACK QUALITY HISTOGRAM (40% Enriched, Grad. Realistic, committed)")
    print("=" * 70)
    key = ("40% Enriched", "Graduated Realistic")
    committed = [r for r in [] if True]
    # Reconstruct histogram data
    random.seed(42)
    pool = build_pool(0.40)
    subpools = build_subpools(pool)
    fm = FITNESS_MODELS["Graduated Realistic"]
    hist_sa = []
    for draft_num in range(NUM_DRAFTS):
        player_arch = ARCHETYPE_NAMES[draft_num % 8]
        strategy = ["archetype-committed", "power-chaser", "signal-reader"][draft_num % 3]
        if strategy != "archetype-committed":
            continue
        result = run_draft(pool, subpools, fm, strategy, player_arch)
        hist_sa.extend(result.sa_per_pack)

    bins = [0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5]
    counts = [0] * (len(bins) - 1)
    for val in hist_sa:
        placed = False
        for i in range(len(bins) - 1):
            if bins[i] <= val < bins[i + 1]:
                counts[i] += 1
                placed = True
                break
        if not placed:
            counts[-1] += 1

    total = len(hist_sa)
    print(f"\n  Total packs: {total}")
    for i in range(len(counts)):
        pct = counts[i] / total * 100 if total > 0 else 0
        bar = "#" * int(pct / 2)
        print(f"  [{bins[i]:.1f}-{bins[i+1]:.1f}): {counts[i]:5d} ({pct:5.1f}%) {bar}")


if __name__ == "__main__":
    main()
