#!/usr/bin/env python3
"""
V7 Agent 2 Simulation: Surge + Floor (and Floor + Pair hybrid)
===============================================================

Two variants:

Variant A (Pure Floor):
  "Each drafted symbol adds tokens (+2 primary, +1 others); when any counter
  reaches T, spend T and fill 3 of 4 slots with cards of that resonance; on
  non-surge packs (from pick 3 onward), 1 slot always shows a card of the
  player's top resonance, remaining 3 random."

Variant B (Floor + Pair):
  Surge packs use 2+1+1 composition (2 R1 + 1 R2 + 1 random).
  Floor packs use 1 R1 + 3 random (same as Variant A).

Three fitness models (A=Optimistic, B=Moderate, C=Pessimistic).
1000 drafts x 30 picks x 3 strategies.
9 metrics at archetype level.
Parameter sensitivity: threshold T in {3,4,5}, floor activation pick in {2,3,4}.
"""

import random
import math
import statistics
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional


# ── Resonance types ──────────────────────────────────────────────────────────

class Resonance(Enum):
    ZEPHYR = "Zephyr"
    EMBER = "Ember"
    STONE = "Stone"
    TIDE = "Tide"


# ── Archetypes on the circle ─────────────────────────────────────────────────

ARCHETYPES = [
    ("Flash",       Resonance.ZEPHYR, Resonance.EMBER),
    ("Blink",       Resonance.EMBER,  Resonance.ZEPHYR),
    ("Storm",       Resonance.EMBER,  Resonance.STONE),
    ("SelfDiscard", Resonance.STONE,  Resonance.EMBER),
    ("SelfMill",    Resonance.STONE,  Resonance.TIDE),
    ("Sacrifice",   Resonance.TIDE,   Resonance.STONE),
    ("Warriors",    Resonance.TIDE,   Resonance.ZEPHYR),
    ("Ramp",        Resonance.ZEPHYR, Resonance.TIDE),
]

ARCHETYPE_NAMES = [a[0] for a in ARCHETYPES]
ARCHETYPE_INDEX = {a[0]: i for i, a in enumerate(ARCHETYPES)}


def archetype_primary(name: str) -> Resonance:
    return ARCHETYPES[ARCHETYPE_INDEX[name]][1]


def archetype_secondary(name: str) -> Resonance:
    return ARCHETYPES[ARCHETYPE_INDEX[name]][2]


# ── Fitness models ───────────────────────────────────────────────────────────

FITNESS_MODELS = {
    "A": {"sibling_A": 1.00, "sibling_B": 0.00, "sibling_C": 0.00},
    "B": {"sibling_A": 0.50, "sibling_B": 0.30, "sibling_C": 0.20},
    "C": {"sibling_A": 0.25, "sibling_B": 0.40, "sibling_C": 0.35},
}


def compute_fitness(card_archetype: Optional[str], rng: random.Random,
                    fitness_model: str = "A") -> dict:
    """Return archetype -> tier for a card belonging to card_archetype."""
    if card_archetype is None:
        return {a: "B" for a in ARCHETYPE_NAMES}

    model = FITNESS_MODELS[fitness_model]
    home_idx = ARCHETYPE_INDEX[card_archetype]
    home_primary = ARCHETYPES[home_idx][1]
    home_secondary = ARCHETYPES[home_idx][2]
    fitness = {}

    for a in ARCHETYPE_NAMES:
        if a == card_archetype:
            fitness[a] = "S"
        elif archetype_primary(a) == home_primary:
            # Sibling archetype sharing primary resonance
            roll = rng.random()
            if roll < model["sibling_A"]:
                fitness[a] = "A"
            elif roll < model["sibling_A"] + model["sibling_B"]:
                fitness[a] = "B"
            else:
                fitness[a] = "C"
        elif (archetype_secondary(a) == home_primary or
              archetype_primary(a) == home_secondary):
            fitness[a] = "B"
        else:
            fitness[a] = "C" if rng.random() < 0.6 else "F"
    return fitness


# ── Card model ───────────────────────────────────────────────────────────────

@dataclass
class SimCard:
    id: int
    symbols: list
    archetype: Optional[str]
    archetype_fitness: dict
    power: float

    def primary_resonance(self) -> Optional[Resonance]:
        return self.symbols[0] if self.symbols else None

    def is_dual_type(self) -> bool:
        return len(set(self.symbols)) >= 2


# ── Card pool construction ───────────────────────────────────────────────────

def build_card_pool(rng: random.Random, fitness_model: str = "A") -> list:
    """Build 360 cards: 36 generic + ~40 per archetype. Max 54 dual-resonance."""
    cards = []
    card_id = 0

    for _ in range(36):
        fitness = compute_fitness(None, rng, fitness_model)
        cards.append(SimCard(
            id=card_id, symbols=[], archetype=None,
            archetype_fitness=fitness, power=rng.uniform(4.0, 7.0),
        ))
        card_id += 1

    # 54 dual-type cards across 8 archetypes: 6 get 7 dual, 2 get 6 dual
    dual_budget = [7] * 6 + [6] * 2
    rng.shuffle(dual_budget)

    for arch_idx, (arch_name, primary, secondary) in enumerate(ARCHETYPES):
        n_dual = dual_budget[arch_idx]
        n_dual_2 = min(4, n_dual)
        n_dual_3 = n_dual - n_dual_2
        n_total = 41 if arch_idx < 4 else 40
        n_mono = n_total - n_dual
        n_mono_1 = round(n_mono * 0.22)
        n_mono_3 = round(n_mono * 0.22)
        n_mono_2 = n_mono - n_mono_1 - n_mono_3

        def make_card(symbols, arch=arch_name):
            nonlocal card_id
            fitness = compute_fitness(arch, rng, fitness_model)
            c = SimCard(id=card_id, symbols=symbols, archetype=arch,
                        archetype_fitness=fitness, power=rng.uniform(3.0, 9.0))
            card_id += 1
            return c

        for _ in range(n_mono_1):
            cards.append(make_card([primary]))
        for _ in range(n_mono_2):
            cards.append(make_card([primary, primary]))
        for _ in range(n_mono_3):
            cards.append(make_card([primary, primary, primary]))
        for _ in range(n_dual_2):
            cards.append(make_card([primary, secondary]))
        for _ in range(n_dual_3):
            cards.append(make_card([primary, primary, secondary]))

    return cards


# ── Index for resonance-based lookups ────────────────────────────────────────

def build_resonance_index(pool: list) -> dict:
    """Map each resonance to list of card ids with that primary resonance."""
    idx = {r: [] for r in Resonance}
    for c in pool:
        pr = c.primary_resonance()
        if pr is not None:
            idx[pr].append(c.id)
    return idx


# ── Pack generation helpers ──────────────────────────────────────────────────

def draw_random_cards(pool: list, drafted_ids: set, exclude_ids: set,
                      rng: random.Random, count: int) -> list:
    available = [c for c in pool if c.id not in drafted_ids and c.id not in exclude_ids]
    if len(available) < count:
        return available
    return rng.sample(available, count)


def draw_resonance_cards(pool: list, resonance: Resonance, drafted_ids: set,
                         exclude_ids: set, rng: random.Random,
                         count: int) -> list:
    candidates = [c for c in pool
                  if c.id not in drafted_ids
                  and c.id not in exclude_ids
                  and c.primary_resonance() == resonance]
    if len(candidates) < count:
        return candidates
    return rng.sample(candidates, count)


# ── Draft state ──────────────────────────────────────────────────────────────

@dataclass
class DraftState:
    tokens: dict = field(default_factory=lambda: {r: 0 for r in Resonance})
    drafted_cards: list = field(default_factory=list)
    pick_number: int = 0
    total_surges: int = 0
    committed_archetype: Optional[str] = None
    commitment_pick: int = 0
    target_archetype: Optional[str] = None

    def top_resonance(self) -> Optional[Resonance]:
        if not any(v > 0 for v in self.tokens.values()):
            return None
        return max(Resonance, key=lambda r: self.tokens[r])

    def second_resonance(self) -> Optional[Resonance]:
        top = self.top_resonance()
        if top is None:
            return None
        candidates = [r for r in Resonance if r != top and self.tokens[r] > 0]
        if not candidates:
            return None
        return max(candidates, key=lambda r: self.tokens[r])


# ── Token earning ────────────────────────────────────────────────────────────

def earn_tokens(state: DraftState, card: SimCard):
    for i, sym in enumerate(card.symbols):
        state.tokens[sym] += 2 if i == 0 else 1


# ── Surge check ──────────────────────────────────────────────────────────────

def check_surge(state: DraftState, threshold: int) -> Optional[Resonance]:
    highest = max(Resonance, key=lambda r: state.tokens[r])
    if state.tokens[highest] >= threshold:
        state.tokens[highest] -= threshold
        state.total_surges += 1
        return highest
    return None


# ── Pack generation: Surge + Floor variants ──────────────────────────────────

def generate_pack_pure_floor(pool: list, state: DraftState, drafted_ids: set,
                             surge_resonance: Optional[Resonance],
                             floor_active: bool,
                             rng: random.Random) -> tuple:
    """
    Variant A: Pure Floor.
    Surge pack: 3 resonance-matched + 1 random.
    Floor pack: 1 top-resonance + 3 random.
    Normal pack: 4 random.
    Returns (pack, pack_type).
    """
    used_ids = set()

    if surge_resonance is not None:
        # Surge pack: 3 of surge resonance + 1 random
        res_cards = draw_resonance_cards(pool, surge_resonance, drafted_ids,
                                         used_ids, rng, 3)
        used_ids.update(c.id for c in res_cards)
        rand_cards = draw_random_cards(pool, drafted_ids, used_ids, rng,
                                       4 - len(res_cards))
        pack = res_cards + rand_cards
        rng.shuffle(pack)
        return pack, "surge"

    if floor_active:
        top_res = state.top_resonance()
        if top_res is not None:
            floor_cards = draw_resonance_cards(pool, top_res, drafted_ids,
                                               used_ids, rng, 1)
            used_ids.update(c.id for c in floor_cards)
            rand_cards = draw_random_cards(pool, drafted_ids, used_ids, rng,
                                           4 - len(floor_cards))
            pack = floor_cards + rand_cards
            rng.shuffle(pack)
            return pack, "floor"

    # Normal pack: 4 random
    pack = draw_random_cards(pool, drafted_ids, used_ids, rng, 4)
    return pack, "normal"


def generate_pack_floor_pair(pool: list, state: DraftState, drafted_ids: set,
                             surge_resonance: Optional[Resonance],
                             floor_active: bool,
                             rng: random.Random) -> tuple:
    """
    Variant B: Floor + Pair.
    Surge pack: 2 R1 + 1 R2 + 1 random.
    Floor pack: 1 R1 + 3 random.
    Normal pack: 4 random.
    """
    used_ids = set()

    if surge_resonance is not None:
        r2 = state.second_resonance()
        # 2 cards of surge resonance
        r1_cards = draw_resonance_cards(pool, surge_resonance, drafted_ids,
                                        used_ids, rng, 2)
        used_ids.update(c.id for c in r1_cards)
        # 1 card of second resonance (if available)
        r2_cards = []
        if r2 is not None:
            r2_cards = draw_resonance_cards(pool, r2, drafted_ids,
                                            used_ids, rng, 1)
            used_ids.update(c.id for c in r2_cards)
        # Fill remaining with random
        filled = len(r1_cards) + len(r2_cards)
        rand_cards = draw_random_cards(pool, drafted_ids, used_ids, rng,
                                       4 - filled)
        pack = r1_cards + r2_cards + rand_cards
        rng.shuffle(pack)
        return pack, "surge_pair"

    if floor_active:
        top_res = state.top_resonance()
        if top_res is not None:
            floor_cards = draw_resonance_cards(pool, top_res, drafted_ids,
                                               used_ids, rng, 1)
            used_ids.update(c.id for c in floor_cards)
            rand_cards = draw_random_cards(pool, drafted_ids, used_ids, rng,
                                           4 - len(floor_cards))
            pack = floor_cards + rand_cards
            rng.shuffle(pack)
            return pack, "floor"

    pack = draw_random_cards(pool, drafted_ids, used_ids, rng, 4)
    return pack, "normal"


# ── Player strategies ────────────────────────────────────────────────────────

TIER_ORDER = {"S": 0, "A": 1, "B": 2, "C": 3, "F": 4}


def strategy_committed(state: DraftState, pack: list, rng: random.Random) -> SimCard:
    if state.committed_archetype is None and state.pick_number >= 5:
        state.committed_archetype = state.target_archetype
        state.commitment_pick = state.pick_number

    if state.committed_archetype:
        target = state.committed_archetype
        pack.sort(key=lambda c: (TIER_ORDER.get(c.archetype_fitness.get(target, "F"), 4),
                                  -c.power))
        return pack[0]
    else:
        target = state.target_archetype
        def score(c):
            target_tier = TIER_ORDER.get(c.archetype_fitness.get(target, "F"), 4)
            best_tier = min(TIER_ORDER.get(t, 4) for t in c.archetype_fitness.values())
            return (min(target_tier, best_tier + 1), -c.power)
        pack.sort(key=score)
        return pack[0]


def strategy_power(state: DraftState, pack: list, rng: random.Random) -> SimCard:
    pack.sort(key=lambda c: -c.power)
    return pack[0]


def strategy_signal(state: DraftState, pack: list, rng: random.Random) -> SimCard:
    if state.committed_archetype is None and state.pick_number >= 5:
        arch_score = {}
        for a in ARCHETYPE_NAMES:
            pri = archetype_primary(a)
            sec = archetype_secondary(a)
            arch_score[a] = state.tokens[pri] * 2 + state.tokens[sec]
        for c in state.drafted_cards:
            for a in ARCHETYPE_NAMES:
                if c.archetype_fitness.get(a) in ("S", "A"):
                    arch_score[a] += 3
        state.committed_archetype = max(arch_score, key=arch_score.get)
        state.commitment_pick = state.pick_number

    if state.committed_archetype:
        target = state.committed_archetype
        pack.sort(key=lambda c: (TIER_ORDER.get(c.archetype_fitness.get(target, "F"), 4),
                                  -c.power))
        return pack[0]
    else:
        def score(c):
            sa_count = sum(1 for t in c.archetype_fitness.values() if t in ("S", "A"))
            return (-sa_count, -c.power)
        pack.sort(key=score)
        return pack[0]


STRATEGIES = {
    "committed": strategy_committed,
    "power":     strategy_power,
    "signal":    strategy_signal,
}


# ── Metric helpers ───────────────────────────────────────────────────────────

def is_sa(card: SimCard, archetype: str) -> bool:
    return card.archetype_fitness.get(archetype, "F") in ("S", "A")


def is_cf(card: SimCard, archetype: str) -> bool:
    return card.archetype_fitness.get(archetype, "F") in ("C", "F")


# ── Draft result ─────────────────────────────────────────────────────────────

@dataclass
class DraftResult:
    strategy: str
    committed_archetype: Optional[str]
    commitment_pick: int
    drafted_cards: list
    pack_records: list
    sa_per_pack_post: list
    cf_per_pack_post: list
    early_variety: list
    early_sa_emerging: list
    surge_count: int
    pack_types: list


# ── Single draft simulation ─────────────────────────────────────────────────

def run_single_draft(pool: list, strategy_name: str, variant: str,
                     threshold: int, floor_start_pick: int,
                     rng: random.Random, num_picks: int = 30,
                     trace: bool = False,
                     forced_archetype: Optional[str] = None) -> tuple:
    state = DraftState()
    if forced_archetype:
        state.target_archetype = forced_archetype
    else:
        state.target_archetype = rng.choice(ARCHETYPE_NAMES)

    strategy_fn = STRATEGIES[strategy_name]
    drafted_ids = set()
    pack_records = []
    pack_types = []
    pending_surge_resonance = None

    sa_per_pack_post = []
    cf_per_pack_post = []
    early_variety = []
    early_sa_emerging = []
    trace_lines = []

    gen_fn = generate_pack_floor_pair if variant == "floor_pair" else generate_pack_pure_floor

    for pick in range(1, num_picks + 1):
        state.pick_number = pick

        floor_active = (pick >= floor_start_pick and
                        any(v > 0 for v in state.tokens.values()))

        pack, pack_type = gen_fn(pool, state, drafted_ids,
                                 pending_surge_resonance, floor_active, rng)
        pending_surge_resonance = None
        pack_records.append((pick, list(pack)))
        pack_types.append(pack_type)

        if not pack:
            break

        if trace:
            trace_lines.append(f"\n--- Pick {pick} [{pack_type}] ---")
            trace_lines.append(f"  Tokens: {_tok_str(state.tokens)}")
            for c in pack:
                sym_str = ",".join(s.value for s in c.symbols) if c.symbols else "generic"
                tier_for_target = c.archetype_fitness.get(state.target_archetype, "?")
                trace_lines.append(
                    f"    [{c.id:3d}] {c.archetype or 'Generic':12s} "
                    f"sym=[{sym_str:20s}] pow={c.power:.1f} "
                    f"tier({state.target_archetype})={tier_for_target}")

        chosen = strategy_fn(state, list(pack), rng)
        state.drafted_cards.append(chosen)
        drafted_ids.add(chosen.id)

        earn_tokens(state, chosen)

        surge_res = check_surge(state, threshold)
        if surge_res is not None:
            pending_surge_resonance = surge_res

        if trace:
            sym_str = ",".join(s.value for s in chosen.symbols) if chosen.symbols else "generic"
            trace_lines.append(
                f"  -> Picked [{chosen.id}] {chosen.archetype or 'Generic'} "
                f"sym=[{sym_str}]")
            trace_lines.append(f"  Tokens after: {_tok_str(state.tokens)}")
            if surge_res:
                trace_lines.append(f"  SURGE QUEUED: {surge_res.value}")

    committed = state.committed_archetype
    if committed is None:
        arch_scores = Counter()
        for c in state.drafted_cards:
            for a in ARCHETYPE_NAMES:
                if c.archetype_fitness.get(a) in ("S", "A"):
                    arch_scores[a] += 1
        committed = arch_scores.most_common(1)[0][0] if arch_scores else "Warriors"

    for pick_num, pack in pack_records:
        if pick_num <= 5:
            archs_with_sa = set()
            for c in pack:
                for a in ARCHETYPE_NAMES:
                    if is_sa(c, a):
                        archs_with_sa.add(a)
            early_variety.append(len(archs_with_sa))
            sa_count = sum(1 for c in pack if is_sa(c, committed))
            early_sa_emerging.append(sa_count)
        else:
            sa_count = sum(1 for c in pack if is_sa(c, committed))
            cf_count = sum(1 for c in pack if is_cf(c, committed))
            sa_per_pack_post.append(sa_count)
            cf_per_pack_post.append(cf_count)

    result = DraftResult(
        strategy=strategy_name,
        committed_archetype=committed,
        commitment_pick=(state.commitment_pick if state.commitment_pick > 0
                         else state.pick_number),
        drafted_cards=state.drafted_cards,
        pack_records=pack_records,
        sa_per_pack_post=sa_per_pack_post,
        cf_per_pack_post=cf_per_pack_post,
        early_variety=early_variety,
        early_sa_emerging=early_sa_emerging,
        surge_count=state.total_surges,
        pack_types=pack_types,
    )
    return result, trace_lines


def _tok_str(tokens):
    return {r.value: v for r, v in tokens.items()}


# ── Batch simulation ─────────────────────────────────────────────────────────

def run_batch(pool: list, n_drafts: int, variant: str, threshold: int,
              floor_start_pick: int, seed: int = 42) -> dict:
    all_results = {s: [] for s in STRATEGIES}
    rng = random.Random(seed)

    for strat_name in STRATEGIES:
        for i in range(n_drafts):
            forced = ARCHETYPE_NAMES[i % 8] if strat_name == "committed" else None
            result, _ = run_single_draft(pool, strat_name, variant, threshold,
                                         floor_start_pick, rng,
                                         forced_archetype=forced)
            all_results[strat_name].append(result)
    return all_results


# ── Metrics computation ──────────────────────────────────────────────────────

def compute_metrics(all_results: dict) -> dict:
    metrics = {}
    for strat, results in all_results.items():
        m = {}
        m["M1_early_variety"] = statistics.mean(
            v for r in results for v in r.early_variety)
        m["M2_early_sa"] = statistics.mean(
            v for r in results for v in r.early_sa_emerging)

        all_sa = [v for r in results for v in r.sa_per_pack_post]
        m["M3_post_sa"] = statistics.mean(all_sa) if all_sa else 0

        all_cf = [v for r in results for v in r.cf_per_pack_post]
        m["M4_post_cf"] = statistics.mean(all_cf) if all_cf else 0

        commit_picks = [r.commitment_pick for r in results if r.commitment_pick > 0]
        m["M5_convergence"] = statistics.mean(commit_picks) if commit_picks else 0

        concentrations = []
        for r in results:
            if r.committed_archetype:
                sa = sum(1 for c in r.drafted_cards
                         if is_sa(c, r.committed_archetype))
                concentrations.append(sa / len(r.drafted_cards) * 100)
        m["M6_concentration"] = statistics.mean(concentrations) if concentrations else 0

        overlaps = []
        if len(results) >= 2:
            rng2 = random.Random(999)
            for _ in range(min(500, len(results))):
                r1, r2 = rng2.sample(results, 2)
                ids1 = {c.id for c in r1.drafted_cards}
                ids2 = {c.id for c in r2.drafted_cards}
                if ids1 | ids2:
                    overlaps.append(len(ids1 & ids2) / len(ids1 | ids2) * 100)
        m["M7_overlap"] = statistics.mean(overlaps) if overlaps else 0

        arch_counts = Counter(r.committed_archetype for r in results
                              if r.committed_archetype)
        total_arch = sum(arch_counts.values())
        arch_freq = {a: arch_counts.get(a, 0) / total_arch * 100
                     for a in ARCHETYPE_NAMES}
        m["M8_freq"] = arch_freq
        m["M8_freq_max"] = max(arch_freq.values()) if arch_freq else 0
        m["M8_freq_min"] = min(arch_freq.values()) if arch_freq else 0

        m["M9_stddev"] = statistics.stdev(all_sa) if len(all_sa) > 1 else 0

        m["avg_surges"] = statistics.mean(r.surge_count for r in results)

        # Pack type distribution
        type_counts = Counter()
        for r in results:
            for pt in r.pack_types:
                type_counts[pt] += 1
        total_packs = sum(type_counts.values())
        m["pack_type_dist"] = {k: v / total_packs * 100 for k, v in type_counts.items()}

        metrics[strat] = m
    return metrics


# ── Per-archetype convergence ────────────────────────────────────────────────

def per_archetype_convergence(all_results: dict) -> dict:
    table = {}
    for arch_name in ARCHETYPE_NAMES:
        conv_picks = []
        for strat, results in all_results.items():
            for r in results:
                if r.committed_archetype != arch_name:
                    continue
                for pick_num, pack in r.pack_records:
                    if pick_num >= 6:
                        sa = sum(1 for c in pack if is_sa(c, arch_name))
                        if sa >= 2:
                            conv_picks.append(pick_num)
                            break
        table[arch_name] = statistics.mean(conv_picks) if conv_picks else float("nan")
    return table


# ── Draft trace helper ───────────────────────────────────────────────────────

def format_trace(pool, strategy_name, variant, threshold, floor_start,
                 seed, label, forced_archetype=None):
    rng = random.Random(seed)
    result, trace_lines = run_single_draft(
        pool, strategy_name, variant, threshold, floor_start, rng,
        trace=True, forced_archetype=forced_archetype)

    committed = result.committed_archetype or "None"
    sa_post = result.sa_per_pack_post
    avg_sa = statistics.mean(sa_post) if sa_post else 0
    deck_sa = sum(1 for c in result.drafted_cards if is_sa(c, committed))
    surge_count = result.surge_count

    # Count pack types
    type_counts = Counter(result.pack_types)

    header = f"\n{'='*70}\n{label}\n{'='*70}"
    # Show first 15 picks in detail
    detail = "\n".join(trace_lines[:120])
    summary = (
        f"\n--- Summary ---\n"
        f"  Committed: {committed} at pick {result.commitment_pick}\n"
        f"  Surges: {surge_count}\n"
        f"  Pack types: {dict(type_counts)}\n"
        f"  Avg S/A picks 6+: {avg_sa:.2f}\n"
        f"  Deck: {deck_sa}/30 S/A for {committed} ({deck_sa/30*100:.0f}%)\n"
    )
    return header + detail + "\n...(truncated)\n" + summary


# ── Metric pass/fail check ───────────────────────────────────────────────────

def check_pass(m: dict) -> dict:
    return {
        "M1": m["M1_early_variety"] >= 3.0,
        "M2": m["M2_early_sa"] <= 2.0,
        "M3": m["M3_post_sa"] >= 2.0,
        "M4": m["M4_post_cf"] >= 0.5,
        "M5": 5.0 <= m["M5_convergence"] <= 8.0,
        "M6": 60.0 <= m["M6_concentration"] <= 90.0,
        "M7": m["M7_overlap"] < 40.0,
        "M8": m["M8_freq_max"] <= 20.0 and m["M8_freq_min"] >= 5.0,
        "M9": m["M9_stddev"] >= 0.8,
    }


# ── Main simulation ─────────────────────────────────────────────────────────

def main():
    N_DRAFTS = 1000
    THRESHOLD = 4
    FLOOR_START = 3

    # ══════════════════════════════════════════════════════════════════════════
    # Run both variants under all 3 fitness models
    # ══════════════════════════════════════════════════════════════════════════
    print("=" * 80)
    print("V7 AGENT 2: SURGE + FLOOR SIMULATION")
    print("=" * 80)

    all_metrics = {}  # (variant, fitness) -> metrics dict

    for fitness_model in ["A", "B", "C"]:
        print(f"\n\n{'#'*70}")
        print(f"# FITNESS MODEL {fitness_model}")
        print(f"{'#'*70}")

        pool_rng = random.Random(12345)
        pool = build_card_pool(pool_rng, fitness_model)

        total = len(pool)
        generic = sum(1 for c in pool if c.archetype is None)
        dual = sum(1 for c in pool if c.is_dual_type())
        print(f"Pool: {total} cards, {generic} generic, {dual} dual-type")

        for variant in ["pure_floor", "floor_pair"]:
            label = f"Fitness={fitness_model}, Variant={'Pure Floor' if variant == 'pure_floor' else 'Floor+Pair'}"
            print(f"\n--- {label} (T={THRESHOLD}, floor_start={FLOOR_START}) ---")

            results = run_batch(pool, N_DRAFTS, variant, THRESHOLD,
                                FLOOR_START, seed=42)
            metrics = compute_metrics(results)
            all_metrics[(variant, fitness_model)] = metrics

            for strat in STRATEGIES:
                m = metrics[strat]
                passes = check_pass(m)
                n_pass = sum(passes.values())
                print(f"\n  [{strat}] ({n_pass}/9 pass)")
                print(f"    M1 EarlyVar:    {m['M1_early_variety']:.2f}  (>=3)   {'PASS' if passes['M1'] else 'FAIL'}")
                print(f"    M2 EarlySA:     {m['M2_early_sa']:.2f}  (<=2)   {'PASS' if passes['M2'] else 'FAIL'}")
                print(f"    M3 PostSA:      {m['M3_post_sa']:.2f}  (>=2)   {'PASS' if passes['M3'] else 'FAIL'}")
                print(f"    M4 PostCF:      {m['M4_post_cf']:.2f}  (>=0.5) {'PASS' if passes['M4'] else 'FAIL'}")
                print(f"    M5 Convergence: {m['M5_convergence']:.1f}   (5-8)   {'PASS' if passes['M5'] else 'FAIL'}")
                print(f"    M6 Conc%:       {m['M6_concentration']:.1f}  (60-90) {'PASS' if passes['M6'] else 'FAIL'}")
                print(f"    M7 Overlap%:    {m['M7_overlap']:.1f}  (<40)   {'PASS' if passes['M7'] else 'FAIL'}")
                print(f"    M8 FreqMax/Min: {m['M8_freq_max']:.1f}/{m['M8_freq_min']:.1f} (<20/>5) {'PASS' if passes['M8'] else 'FAIL'}")
                print(f"    M9 StdDev:      {m['M9_stddev']:.2f}  (>=0.8) {'PASS' if passes['M9'] else 'FAIL'}")
                print(f"    Avg surges: {m['avg_surges']:.1f}")
                if m.get("pack_type_dist"):
                    print(f"    Pack types: {m['pack_type_dist']}")

    # ══════════════════════════════════════════════════════════════════════════
    # Parameter sensitivity: Threshold sweep
    # ══════════════════════════════════════════════════════════════════════════
    print(f"\n\n{'#'*70}")
    print("# PARAMETER SENSITIVITY: THRESHOLD (T=3,4,5)")
    print(f"{'#'*70}")

    for fitness_model in ["A", "B", "C"]:
        pool_rng = random.Random(12345)
        pool = build_card_pool(pool_rng, fitness_model)

        print(f"\n--- Fitness {fitness_model}, Pure Floor, committed strategy ---")
        print(f"{'T':>3s} | {'M3_SA':>6s} | {'M5_Conv':>7s} | {'M6_Conc':>7s} | {'M9_Std':>6s} | {'Surges':>6s} | {'Pass':>4s}")
        print("-" * 55)

        for T in [3, 4, 5]:
            results = run_batch(pool, N_DRAFTS, "pure_floor", T,
                                FLOOR_START, seed=42)
            m = compute_metrics(results)["committed"]
            passes = check_pass(m)
            n_pass = sum(passes.values())
            print(f"{T:3d} | {m['M3_post_sa']:6.2f} | {m['M5_convergence']:7.1f} | "
                  f"{m['M6_concentration']:7.1f} | {m['M9_stddev']:6.2f} | "
                  f"{m['avg_surges']:6.1f} | {n_pass:4d}/9")

    # ══════════════════════════════════════════════════════════════════════════
    # Parameter sensitivity: Floor activation pick sweep
    # ══════════════════════════════════════════════════════════════════════════
    print(f"\n\n{'#'*70}")
    print("# PARAMETER SENSITIVITY: FLOOR ACTIVATION PICK (2,3,4)")
    print(f"{'#'*70}")

    for fitness_model in ["A", "B", "C"]:
        pool_rng = random.Random(12345)
        pool = build_card_pool(pool_rng, fitness_model)

        print(f"\n--- Fitness {fitness_model}, Pure Floor, T=4, committed ---")
        print(f"{'FloorPk':>7s} | {'M1_Var':>6s} | {'M2_ESA':>6s} | {'M3_SA':>6s} | {'M5_Conv':>7s} | {'M9_Std':>6s} | {'Pass':>4s}")
        print("-" * 60)

        for fp in [2, 3, 4]:
            results = run_batch(pool, N_DRAFTS, "pure_floor", THRESHOLD,
                                fp, seed=42)
            m = compute_metrics(results)["committed"]
            passes = check_pass(m)
            n_pass = sum(passes.values())
            print(f"{fp:7d} | {m['M1_early_variety']:6.2f} | {m['M2_early_sa']:6.2f} | "
                  f"{m['M3_post_sa']:6.2f} | {m['M5_convergence']:7.1f} | "
                  f"{m['M9_stddev']:6.2f} | {n_pass:4d}/9")

    # ══════════════════════════════════════════════════════════════════════════
    # Per-archetype convergence table
    # ══════════════════════════════════════════════════════════════════════════
    print(f"\n\n{'#'*70}")
    print("# PER-ARCHETYPE CONVERGENCE TABLE")
    print(f"{'#'*70}")

    for fitness_model in ["A", "B", "C"]:
        pool_rng = random.Random(12345)
        pool = build_card_pool(pool_rng, fitness_model)

        for variant in ["pure_floor", "floor_pair"]:
            results = run_batch(pool, N_DRAFTS, variant, THRESHOLD,
                                FLOOR_START, seed=42)
            arch_conv = per_archetype_convergence(results)
            vname = "Pure Floor" if variant == "pure_floor" else "Floor+Pair"
            print(f"\n--- Fitness {fitness_model}, {vname} ---")
            for arch, pick in arch_conv.items():
                print(f"  {arch:15s}: pick {pick:.1f}")

    # ══════════════════════════════════════════════════════════════════════════
    # Fitness degradation curve
    # ══════════════════════════════════════════════════════════════════════════
    print(f"\n\n{'#'*70}")
    print("# FITNESS DEGRADATION CURVE (committed strategy)")
    print(f"{'#'*70}")

    print(f"\n{'Variant':15s} | {'Fit':>3s} | {'M3_SA':>6s} | {'M5_Conv':>7s} | {'M6_Conc':>7s} | {'M9_Std':>6s} | {'Pass':>4s}")
    print("-" * 65)
    for variant in ["pure_floor", "floor_pair"]:
        vname = "PureFloor" if variant == "pure_floor" else "Floor+Pair"
        for fm in ["A", "B", "C"]:
            m = all_metrics[(variant, fm)]["committed"]
            passes = check_pass(m)
            n_pass = sum(passes.values())
            print(f"{vname:15s} | {fm:>3s} | {m['M3_post_sa']:6.2f} | "
                  f"{m['M5_convergence']:7.1f} | {m['M6_concentration']:7.1f} | "
                  f"{m['M9_stddev']:6.2f} | {n_pass:4d}/9")

    # ══════════════════════════════════════════════════════════════════════════
    # Draft traces
    # ══════════════════════════════════════════════════════════════════════════
    print(f"\n\n{'#'*70}")
    print("# DRAFT TRACES")
    print(f"{'#'*70}")

    # Use Moderate fitness (B) for traces -- most informative
    pool_rng = random.Random(12345)
    pool_B = build_card_pool(pool_rng, "B")

    # Trace 1: Committed Warriors player, Pure Floor
    print(format_trace(pool_B, "committed", "pure_floor", 4, 3, 100,
                       "Trace 1: Committed Warriors, Pure Floor, Fitness B",
                       forced_archetype="Warriors"))

    # Trace 2: Signal reader, Floor+Pair
    print(format_trace(pool_B, "signal", "floor_pair", 4, 3, 200,
                       "Trace 2: Signal Reader, Floor+Pair, Fitness B"))

    # Trace 3: Power chaser, Pure Floor
    print(format_trace(pool_B, "power", "pure_floor", 4, 3, 300,
                       "Trace 3: Power Chaser, Pure Floor, Fitness B"))

    # ══════════════════════════════════════════════════════════════════════════
    # Comprehensive summary table
    # ══════════════════════════════════════════════════════════════════════════
    print(f"\n\n{'#'*70}")
    print("# COMPREHENSIVE SUMMARY TABLE")
    print(f"{'#'*70}")

    print("\nAll metrics for committed strategy across variants and fitness models:")
    print(f"{'Variant':12s} {'Fit':>3s} | {'M1':>5s} {'M2':>5s} {'M3':>5s} {'M4':>5s} "
          f"{'M5':>5s} {'M6':>6s} {'M7':>5s} {'M8mx':>5s} {'M8mn':>5s} {'M9':>5s} | {'P':>3s}")
    print("-" * 90)
    for variant in ["pure_floor", "floor_pair"]:
        vname = "PurFloor" if variant == "pure_floor" else "Flr+Pair"
        for fm in ["A", "B", "C"]:
            m = all_metrics[(variant, fm)]["committed"]
            passes = check_pass(m)
            n_pass = sum(passes.values())
            print(f"{vname:12s} {fm:>3s} | {m['M1_early_variety']:5.2f} "
                  f"{m['M2_early_sa']:5.2f} {m['M3_post_sa']:5.2f} "
                  f"{m['M4_post_cf']:5.2f} {m['M5_convergence']:5.1f} "
                  f"{m['M6_concentration']:6.1f} {m['M7_overlap']:5.1f} "
                  f"{m['M8_freq_max']:5.1f} {m['M8_freq_min']:5.1f} "
                  f"{m['M9_stddev']:5.2f} | {n_pass:3d}/9")

    print(f"\n{'Targets':12s} {'':>3s} | {'>=3':>5s} {'<=2':>5s} {'>=2':>5s} "
          f"{'>.5':>5s} {'5-8':>5s} {'60-90':>6s} {'<40':>5s} "
          f"{'<20':>5s} {'>5':>5s} {'>=.8':>5s} |")

    # Also show signal and power strategies
    for strat in ["signal", "power"]:
        print(f"\n--- {strat} strategy ---")
        print(f"{'Variant':12s} {'Fit':>3s} | {'M1':>5s} {'M2':>5s} {'M3':>5s} {'M4':>5s} "
              f"{'M5':>5s} {'M6':>6s} {'M7':>5s} {'M9':>5s} | {'P':>3s}")
        print("-" * 75)
        for variant in ["pure_floor", "floor_pair"]:
            vname = "PurFloor" if variant == "pure_floor" else "Flr+Pair"
            for fm in ["A", "B", "C"]:
                m = all_metrics[(variant, fm)][strat]
                passes = check_pass(m)
                n_pass = sum(passes.values())
                print(f"{vname:12s} {fm:>3s} | {m['M1_early_variety']:5.2f} "
                      f"{m['M2_early_sa']:5.2f} {m['M3_post_sa']:5.2f} "
                      f"{m['M4_post_cf']:5.2f} {m['M5_convergence']:5.1f} "
                      f"{m['M6_concentration']:6.1f} {m['M7_overlap']:5.1f} "
                      f"{m['M9_stddev']:5.2f} | {n_pass:3d}/9")


if __name__ == "__main__":
    main()
