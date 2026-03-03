#!/usr/bin/env python3
"""
Resonance V4 — Agent 2: Square-Root Affinity Sampling Simulation

One-sentence algorithm:
"Each card in the pool is drawn with weight equal to 1.5 plus the square root
of its total resonance symbol overlap with your drafted deck (capped at 4.5),
so cards matching your deck appear more often but with diminishing returns."

This file is self-contained and runnable with `python3 sim_2.py`.
"""

import random
import math
import statistics
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional
from collections import defaultdict

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

RESONANCES = ["Ember", "Stone", "Tide", "Zephyr"]

# Eight archetypes arranged on the circle (index 0-7).
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

# Primary and secondary resonance for each archetype (by index).
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

# Algorithm parameters (defaults — swept later)
BASE_WEIGHT = 1.5
SQRT_CAP = 4.5
EXPONENT = 0.5        # sqrt
PRIMARY_MULT = 2      # primary symbol counter increment

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
    symbols: list  # list of resonance strings, ordered; [] = generic
    archetype: Optional[str]  # home archetype name, None for generic
    archetype_idx: Optional[int]
    archetype_fitness: dict = field(default_factory=dict)  # archetype_idx -> Tier
    power: float = 5.0

def _circle_distance(a: int, b: int) -> int:
    """Minimum distance on the 8-archetype circle."""
    return min(abs(a - b), NUM_ARCHETYPES - abs(a - b))

def _assign_fitness(card: SimCard):
    """Assign archetype fitness tiers based on circle distance."""
    if card.archetype_idx is None:
        # Generic: B-tier everywhere
        for i in range(NUM_ARCHETYPES):
            card.archetype_fitness[i] = Tier.B
        return
    home = card.archetype_idx
    for i in range(NUM_ARCHETYPES):
        dist = _circle_distance(home, i)
        if dist == 0:
            card.archetype_fitness[i] = Tier.S
        elif dist == 1:
            card.archetype_fitness[i] = Tier.A
        elif dist == 2:
            card.archetype_fitness[i] = Tier.B
        elif dist == 3:
            card.archetype_fitness[i] = Tier.C
        else:
            card.archetype_fitness[i] = Tier.F

def _generate_symbols(archetype_idx: int, num_symbols: int, rng: random.Random) -> list:
    """Generate an ordered symbol list for a card of the given archetype."""
    primary_res, secondary_res = ARCHETYPE_RESONANCES[archetype_idx]
    if num_symbols == 1:
        # 70% primary, 30% secondary
        return [primary_res] if rng.random() < 0.7 else [secondary_res]
    elif num_symbols == 2:
        # First symbol: 75% primary, 25% secondary
        first = primary_res if rng.random() < 0.75 else secondary_res
        # Second symbol: the other resonance 80%, same 20%
        other = secondary_res if first == primary_res else primary_res
        second = other if rng.random() < 0.8 else first
        return [first, second]
    else:  # 3 symbols
        # Strong archetype marker: primary-secondary-primary or primary-primary-secondary
        patterns = [
            [primary_res, secondary_res, primary_res],
            [primary_res, primary_res, secondary_res],
            [primary_res, secondary_res, secondary_res],
        ]
        return rng.choice(patterns)

def build_card_pool(rng: random.Random,
                    pct_1sym: float = 0.25,
                    pct_2sym: float = 0.55,
                    pct_3sym: float = 0.20,
                    removed_resonance: Optional[str] = None,
                    removal_fraction: float = 0.0) -> list:
    """
    Build 360-card pool.
    pct_1sym/pct_2sym/pct_3sym: distribution among non-generic cards.
    removed_resonance / removal_fraction: for signal-reading pool asymmetry.
    """
    cards = []
    card_id = 0

    for arch_idx in range(NUM_ARCHETYPES):
        n = CARDS_PER_ARCHETYPE  # 40
        n1 = round(n * pct_1sym)
        n3 = round(n * pct_3sym)
        n2 = n - n1 - n3

        for count, num in [(1, n1), (2, n2), (3, n3)]:
            for _ in range(num):
                syms = _generate_symbols(arch_idx, count, rng)
                c = SimCard(id=card_id, symbols=syms, archetype=ARCHETYPES[arch_idx],
                            archetype_idx=arch_idx, power=rng.uniform(3, 8))
                _assign_fitness(c)
                cards.append(c)
                card_id += 1

    # Generic cards
    for _ in range(GENERIC_COUNT):
        c = SimCard(id=card_id, symbols=[], archetype=None, archetype_idx=None,
                    power=rng.uniform(4, 9))
        _assign_fitness(c)
        cards.append(c)
        card_id += 1

    # Pool asymmetry: remove some cards of a specific resonance
    if removed_resonance and removal_fraction > 0:
        to_remove = [c for c in cards if removed_resonance in c.symbols]
        n_remove = int(len(to_remove) * removal_fraction)
        removed = set(c.id for c in rng.sample(to_remove, min(n_remove, len(to_remove))))
        cards = [c for c in cards if c.id not in removed]

    return cards

# ---------------------------------------------------------------------------
# Draft algorithms
# ---------------------------------------------------------------------------

def _compute_affinity(card: SimCard, player_counts: dict,
                      base: float, cap: float, exponent: float) -> float:
    """Compute sampling weight for a card given player resonance counts."""
    if not card.symbols:
        return base  # generic cards always at base
    raw = sum(player_counts.get(s, 0) for s in card.symbols)
    bonus = min(raw ** exponent, cap)
    return base + bonus

def _update_counters(card: SimCard, player_counts: dict, primary_mult: int = PRIMARY_MULT):
    """Update resonance counters after drafting a card."""
    if not card.symbols:
        return
    player_counts[card.symbols[0]] = player_counts.get(card.symbols[0], 0) + primary_mult
    for s in card.symbols[1:]:
        player_counts[s] = player_counts.get(s, 0) + 1

def generate_pack_sqrt(pool: list, player_counts: dict, rng: random.Random,
                       base: float = BASE_WEIGHT, cap: float = SQRT_CAP,
                       exponent: float = EXPONENT) -> list:
    """Generate a pack of PACK_SIZE cards using weighted sampling."""
    weights = [_compute_affinity(c, player_counts, base, cap, exponent) for c in pool]
    total = sum(weights)
    if total == 0:
        return rng.sample(pool, min(PACK_SIZE, len(pool)))

    pack = []
    available = list(range(len(pool)))
    avail_weights = list(weights)

    for _ in range(min(PACK_SIZE, len(pool))):
        total_w = sum(avail_weights)
        r = rng.random() * total_w
        cumulative = 0.0
        chosen_local = 0
        for j, w in enumerate(avail_weights):
            cumulative += w
            if cumulative >= r:
                chosen_local = j
                break
        pack.append(pool[available[chosen_local]])
        available.pop(chosen_local)
        avail_weights.pop(chosen_local)

    return pack

# Lane Locking baseline (V3) --------------------------------------------------

def generate_pack_lane_locking(pool: list, player_counts: dict, rng: random.Random,
                               threshold_1: int = 3, threshold_2: int = 8,
                               primary_mult: int = 2) -> list:
    """
    V3 Lane Locking: if a resonance counter >= threshold_1, one pack slot is
    locked to that resonance. If >= threshold_2, two slots are locked.
    Remaining slots are random.
    """
    # Determine locked slots
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

    # Fill locked slots
    for res in locked_resonances[:PACK_SIZE]:
        candidates = [c for c in pool if res in c.symbols and c.id not in used_ids]
        if candidates:
            chosen = rng.choice(candidates)
            pack.append(chosen)
            used_ids.add(chosen.id)

    # Fill remaining slots randomly
    remaining = PACK_SIZE - len(pack)
    if remaining > 0:
        candidates = [c for c in pool if c.id not in used_ids]
        if candidates:
            chosen = rng.sample(candidates, min(remaining, len(candidates)))
            pack.extend(chosen)

    return pack[:PACK_SIZE]

# ---------------------------------------------------------------------------
# Player strategies
# ---------------------------------------------------------------------------

def _best_archetype(player_counts: dict) -> int:
    """Identify the archetype whose resonances the player has invested most in."""
    scores = []
    for arch_idx in range(NUM_ARCHETYPES):
        pri, sec = ARCHETYPE_RESONANCES[arch_idx]
        score = player_counts.get(pri, 0) * 2 + player_counts.get(sec, 0)
        scores.append(score)
    return scores.index(max(scores))

def pick_committed(pack: list, player_counts: dict, target_arch: int,
                   pick_num: int) -> tuple:
    """
    Archetype-committed player. Commits around pick 5-6.
    Before commitment: picks highest-power card that has any S/A fitness overlap.
    After commitment: picks card with best fitness for target archetype.
    """
    if pick_num < 5:
        # Early: pick highest power among cards with decent fitness for emerging arch
        target = _best_archetype(player_counts) if sum(player_counts.values()) > 0 else None
        if target is not None:
            sa_cards = [c for c in pack
                        if c.archetype_fitness.get(target, Tier.F).value >= Tier.A.value]
            if sa_cards:
                return max(sa_cards, key=lambda c: c.power), target
        return max(pack, key=lambda c: c.power), target
    else:
        # Committed: best fitness for target
        if target_arch is None:
            target_arch = _best_archetype(player_counts)
        best = max(pack, key=lambda c: (c.archetype_fitness.get(target_arch, Tier.F).value,
                                         c.power))
        return best, target_arch

def pick_power_chaser(pack: list, player_counts: dict, target_arch: int,
                      pick_num: int) -> tuple:
    """Always picks the highest raw power card."""
    chosen = max(pack, key=lambda c: c.power)
    target = _best_archetype(player_counts)
    return chosen, target

def pick_signal_reader(pack: list, player_counts: dict, target_arch: int,
                       pick_num: int, pool_snapshot: list = None) -> tuple:
    """
    Signal reader: examines packs for over-represented resonances and pivots.
    First 5 picks: evaluates which resonance appears most in packs seen.
    Then commits to the archetype whose resonances appear most available.
    """
    if pick_num < 5:
        # Track resonances seen in pack
        res_counts = defaultdict(int)
        for c in pack:
            for s in c.symbols:
                res_counts[s] += 1
        # Pick the card from the most-represented resonance
        if res_counts:
            best_res = max(res_counts, key=res_counts.get)
            res_cards = [c for c in pack if best_res in c.symbols]
            return max(res_cards, key=lambda c: c.power), None
        return max(pack, key=lambda c: c.power), None
    else:
        if target_arch is None:
            target_arch = _best_archetype(player_counts)
        best = max(pack, key=lambda c: (c.archetype_fitness.get(target_arch, Tier.F).value,
                                         c.power))
        return best, target_arch

# ---------------------------------------------------------------------------
# Simulation engine
# ---------------------------------------------------------------------------

@dataclass
class DraftResult:
    picks: list = field(default_factory=list)       # list of SimCard
    packs_seen: list = field(default_factory=list)   # list of list of SimCard
    target_archetype: int = 0
    strategy: str = ""

def run_single_draft(pool_template: list, strategy: str, rng: random.Random,
                     pack_fn=generate_pack_sqrt, forced_arch: int = None,
                     base: float = BASE_WEIGHT, cap: float = SQRT_CAP,
                     exponent: float = EXPONENT) -> DraftResult:
    """Run a single 30-pick draft."""
    pool = list(pool_template)
    player_counts = defaultdict(int)
    result = DraftResult(strategy=strategy)
    target_arch = forced_arch

    # For signal reader: track resonance appearances
    seen_res_counts = defaultdict(int)

    for pick_num in range(NUM_PICKS):
        if pack_fn == generate_pack_sqrt:
            pack = generate_pack_sqrt(pool, player_counts, rng, base, cap, exponent)
        else:
            pack = generate_pack_lane_locking(pool, player_counts, rng)

        if not pack:
            break

        result.packs_seen.append(list(pack))

        if strategy == "committed":
            chosen, target_arch = pick_committed(pack, dict(player_counts),
                                                  target_arch, pick_num)
        elif strategy == "power":
            chosen, target_arch = pick_power_chaser(pack, dict(player_counts),
                                                     target_arch, pick_num)
        elif strategy == "signal":
            # Signal reader accumulates resonance sighting data
            for c in pack:
                for s in c.symbols:
                    seen_res_counts[s] += 1
            if pick_num == 4 and target_arch is None:
                # At pick 5, commit to archetype with most-seen resonances
                arch_scores = []
                for ai in range(NUM_ARCHETYPES):
                    pri, sec = ARCHETYPE_RESONANCES[ai]
                    score = seen_res_counts.get(pri, 0) * 2 + seen_res_counts.get(sec, 0)
                    arch_scores.append(score)
                target_arch = arch_scores.index(max(arch_scores))
            chosen, target_arch = pick_signal_reader(pack, dict(player_counts),
                                                      target_arch, pick_num)
        else:
            chosen = pack[0]
            target_arch = 0

        result.picks.append(chosen)
        _update_counters(chosen, player_counts)

    if target_arch is None:
        target_arch = _best_archetype(dict(player_counts))
    result.target_archetype = target_arch
    return result

# ---------------------------------------------------------------------------
# Metrics computation
# ---------------------------------------------------------------------------

def compute_metrics(results: list) -> dict:
    """Compute all 8 measurable targets + variance from a list of DraftResults."""
    # Metric accumulators
    early_unique_archetypes = []     # picks 1-5: unique archetypes with S/A per pack
    early_sa_for_emerging = []       # picks 1-5: S/A for emerging archetype per pack
    late_sa_for_committed = []       # picks 6+: S/A for committed archetype per pack
    late_off_archetype = []          # picks 6+: C/F cards per pack
    convergence_picks = []           # pick at which 2+ S/A becomes regular
    deck_concentrations = []         # % of final deck that is S/A for target
    all_final_decks = []             # for run-to-run variety
    archetype_frequencies = defaultdict(int)

    for dr in results:
        tgt = dr.target_archetype
        archetype_frequencies[tgt] += 1

        # Per-pick analysis
        sa_streak = 0
        convergence_pick = NUM_PICKS  # default: never converged

        for pick_num, pack in enumerate(dr.packs_seen):
            sa_count = sum(1 for c in pack
                          if c.archetype_fitness.get(tgt, Tier.F).value >= Tier.A.value)
            cf_count = sum(1 for c in pack
                          if c.archetype_fitness.get(tgt, Tier.F).value <= Tier.C.value)

            if pick_num < 5:
                # Early metrics
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

            # Convergence tracking
            if pick_num >= 5:
                if sa_count >= 2:
                    sa_streak += 1
                else:
                    sa_streak = 0
                if sa_streak >= 3 and convergence_pick == NUM_PICKS:
                    convergence_pick = pick_num - 2  # started 3 picks ago

        convergence_picks.append(convergence_pick)

        # Deck concentration
        sa_in_deck = sum(1 for c in dr.picks
                        if c.archetype_fitness.get(tgt, Tier.F).value >= Tier.A.value)
        deck_concentrations.append(sa_in_deck / len(dr.picks) if dr.picks else 0)

        all_final_decks.append(set(c.id for c in dr.picks))

    # Run-to-run variety: average pairwise overlap among same-archetype drafts
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

    # Archetype frequency
    total_runs = len(results)
    arch_freq = {a: archetype_frequencies.get(a, 0) / total_runs for a in range(NUM_ARCHETYPES)}

    # Variance of late S/A
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
        "late_sa_stddev": late_sa_stddev,
        "late_sa_distribution": _distribution(late_sa_for_committed),
    }

def _distribution(values: list) -> dict:
    """Count how often each integer value appears."""
    counts = defaultdict(int)
    for v in values:
        counts[v] += 1
    total = len(values) if values else 1
    return {k: counts[k] / total for k in sorted(counts.keys())}

def compute_per_archetype_convergence(pool_template: list, rng: random.Random,
                                       pack_fn=generate_pack_sqrt,
                                       n_drafts: int = 200,
                                       base: float = BASE_WEIGHT,
                                       cap: float = SQRT_CAP,
                                       exponent: float = EXPONENT) -> dict:
    """For each archetype, run drafts targeting that archetype and find convergence pick."""
    convergence_table = {}
    for arch_idx in range(NUM_ARCHETYPES):
        conv_picks = []
        for _ in range(n_drafts):
            dr = run_single_draft(pool_template, "committed", rng, pack_fn,
                                  forced_arch=arch_idx, base=base, cap=cap,
                                  exponent=exponent)
            # Find convergence pick: first pick >= 5 where 3 consecutive packs have 2+ S/A
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
# Draft traces
# ---------------------------------------------------------------------------

def format_trace(dr: DraftResult, label: str) -> str:
    """Format a single draft as a readable trace."""
    lines = [f"\n=== Draft Trace: {label} (Strategy: {dr.strategy}) ==="]
    lines.append(f"Target archetype: {ARCHETYPES[dr.target_archetype]} (idx {dr.target_archetype})")
    lines.append(f"{'Pick':>4} | {'Chosen Card':40s} | {'Pack S/A':>7} | {'Pack Summary'}")
    lines.append("-" * 100)

    for i, (pick, pack) in enumerate(zip(dr.picks, dr.packs_seen)):
        tgt = dr.target_archetype
        sa_count = sum(1 for c in pack
                      if c.archetype_fitness.get(tgt, Tier.F).value >= Tier.A.value)
        arch_label = pick.archetype if pick.archetype else "Generic"
        syms = "/".join(pick.symbols) if pick.symbols else "none"
        fitness = pick.archetype_fitness.get(tgt, Tier.F).name
        chosen_str = f"{arch_label} [{syms}] (fitness={fitness})"

        pack_archs = []
        for c in pack:
            a = c.archetype if c.archetype else "Generic"
            f = c.archetype_fitness.get(tgt, Tier.F).name
            pack_archs.append(f"{a[:12]}({f})")
        pack_str = ", ".join(pack_archs)

        lines.append(f"{i+1:>4} | {chosen_str:40s} | {sa_count:>7} | {pack_str}")

    # Final deck stats
    tgt = dr.target_archetype
    sa = sum(1 for c in dr.picks if c.archetype_fitness.get(tgt, Tier.F).value >= Tier.A.value)
    lines.append(f"\nFinal deck: {sa}/{len(dr.picks)} S/A cards for {ARCHETYPES[tgt]} "
                 f"({100*sa/len(dr.picks):.1f}%)")
    return "\n".join(lines)

# ---------------------------------------------------------------------------
# Main simulation
# ---------------------------------------------------------------------------

def run_full_simulation(base=BASE_WEIGHT, cap=SQRT_CAP, exponent=EXPONENT,
                        pct_1sym=0.25, pct_2sym=0.55, pct_3sym=0.20,
                        use_lane_locking=False, label="", n_drafts=NUM_DRAFTS,
                        removed_resonance=None, removal_fraction=0.0,
                        quiet=False):
    """Run a full simulation suite and return metrics."""
    rng = random.Random(42)
    pool = build_card_pool(rng, pct_1sym, pct_2sym, pct_3sym,
                           removed_resonance, removal_fraction)

    pack_fn = generate_pack_lane_locking if use_lane_locking else generate_pack_sqrt

    all_results = []
    strategies = ["committed", "power", "signal"]

    for strategy in strategies:
        for _ in range(n_drafts):
            dr = run_single_draft(pool, strategy, rng, pack_fn,
                                  base=base, cap=cap, exponent=exponent)
            all_results.append(dr)

    # Compute metrics (committed strategy only for most metrics)
    committed_results = [r for r in all_results if r.strategy == "committed"]
    power_results = [r for r in all_results if r.strategy == "power"]
    signal_results = [r for r in all_results if r.strategy == "signal"]

    metrics = compute_metrics(committed_results)

    # Also compute for other strategies
    power_metrics = compute_metrics(power_results)
    signal_metrics = compute_metrics(signal_results)

    if not quiet:
        algo_name = "Lane Locking" if use_lane_locking else "Sqrt Affinity"
        print(f"\n{'='*70}")
        print(f" {algo_name} Results {label}")
        print(f"{'='*70}")
        print(f" Params: base={base}, cap={cap}, exp={exponent}")
        print(f" Symbol dist: 1-sym={pct_1sym:.0%}, 2-sym={pct_2sym:.0%}, 3-sym={pct_3sym:.0%}")
        print(f" Pool size: {len(pool)}")
        print(f"{'='*70}")
        print(f"\n--- Committed Player Metrics ---")
        _print_scorecard(metrics)
        print(f"\n--- Power Chaser Metrics ---")
        print(f"  Late S/A: {power_metrics['late_sa_committed']:.2f}  "
              f"Deck concentration: {power_metrics['deck_concentration']:.1%}")
        print(f"\n--- Signal Reader Metrics ---")
        print(f"  Late S/A: {signal_metrics['late_sa_committed']:.2f}  "
              f"Deck concentration: {signal_metrics['deck_concentration']:.1%}")

    return metrics, committed_results, pool

def _print_scorecard(m: dict):
    print(f"  Picks 1-5 unique archetypes/pack:   {m['early_unique_archetypes']:.2f}  (target: >= 3)")
    print(f"  Picks 1-5 S/A for emerging/pack:    {m['early_sa_emerging']:.2f}  (target: <= 2)")
    print(f"  Picks 6+ S/A for committed/pack:    {m['late_sa_committed']:.2f}  (target: >= 2)")
    print(f"  Picks 6+ off-archetype (C/F)/pack:  {m['late_off_archetype']:.2f}  (target: >= 0.5)")
    print(f"  Convergence pick:                   {m['convergence_pick']:.1f}  (target: 5-8)")
    print(f"  Deck concentration:                 {m['deck_concentration']:.1%}  (target: 60-90%)")
    print(f"  Run-to-run overlap:                 {m['run_to_run_overlap']:.1%}  (target: < 40%)")
    print(f"  Archetype freq max:                 {m['arch_freq_max']:.1%}  (target: < 20%)")
    print(f"  Archetype freq min:                 {m['arch_freq_min']:.1%}  (target: > 5%)")
    print(f"  Late S/A stddev:                    {m['late_sa_stddev']:.2f}  (target: >= 0.8)")
    if m['late_sa_distribution']:
        print(f"  S/A distribution (picks 6+):")
        for k, v in sorted(m['late_sa_distribution'].items()):
            print(f"    {k} S/A cards: {v:.1%}")

def _pass_fail(actual, target_min=None, target_max=None):
    if target_min is not None and actual < target_min:
        return "FAIL"
    if target_max is not None and actual > target_max:
        return "FAIL"
    return "PASS"

def print_comparison_table(sqrt_m: dict, ll_m: dict):
    """Print side-by-side comparison of Sqrt Affinity vs Lane Locking."""
    print(f"\n{'='*80}")
    print(f" COMPARISON: Sqrt Affinity vs Lane Locking")
    print(f"{'='*80}")
    header = f"{'Metric':45s} | {'Target':12s} | {'SqrtAff':8s} | {'LaneLk':8s} | {'SA P/F':6s} | {'LL P/F':6s}"
    print(header)
    print("-" * 100)

    rows = [
        ("Picks 1-5 unique archetypes/pack", ">=3",
         sqrt_m['early_unique_archetypes'], ll_m['early_unique_archetypes'], 3, None),
        ("Picks 1-5 S/A for emerging/pack", "<=2",
         sqrt_m['early_sa_emerging'], ll_m['early_sa_emerging'], None, 2),
        ("Picks 6+ S/A for committed/pack", ">=2",
         sqrt_m['late_sa_committed'], ll_m['late_sa_committed'], 2, None),
        ("Picks 6+ off-archetype (C/F)/pack", ">=0.5",
         sqrt_m['late_off_archetype'], ll_m['late_off_archetype'], 0.5, None),
        ("Convergence pick", "5-8",
         sqrt_m['convergence_pick'], ll_m['convergence_pick'], None, None),
        ("Deck concentration", "60-90%",
         sqrt_m['deck_concentration'], ll_m['deck_concentration'], None, None),
        ("Run-to-run overlap", "<40%",
         sqrt_m['run_to_run_overlap'], ll_m['run_to_run_overlap'], None, None),
        ("Late S/A stddev", ">=0.8",
         sqrt_m['late_sa_stddev'], ll_m['late_sa_stddev'], 0.8, None),
    ]

    for name, target, sa_val, ll_val, tmin, tmax in rows:
        if "concentration" in name or "overlap" in name:
            sa_str = f"{sa_val:.1%}"
            ll_str = f"{ll_val:.1%}"
        elif "pick" in name.lower() and "S/A" not in name:
            sa_str = f"{sa_val:.1f}"
            ll_str = f"{ll_val:.1f}"
        else:
            sa_str = f"{sa_val:.2f}"
            ll_str = f"{ll_val:.2f}"

        if tmin is not None:
            sa_pf = _pass_fail(sa_val, target_min=tmin)
            ll_pf = _pass_fail(ll_val, target_min=tmin)
        elif tmax is not None:
            sa_pf = _pass_fail(sa_val, target_max=tmax)
            ll_pf = _pass_fail(ll_val, target_max=tmax)
        else:
            sa_pf = "—"
            ll_pf = "—"

        print(f"{name:45s} | {target:12s} | {sa_str:8s} | {ll_str:8s} | {sa_pf:6s} | {ll_pf:6s}")

if __name__ == "__main__":
    print("=" * 70)
    print(" Resonance V4 — Agent 2: Square-Root Affinity Sampling")
    print("=" * 70)

    # ===== 1. Main simulation (default params) =====
    print("\n\n### 1. DEFAULT PARAMETERS ###")
    sqrt_metrics, sqrt_results, pool = run_full_simulation()

    # ===== 2. Lane Locking baseline =====
    print("\n\n### 2. LANE LOCKING BASELINE ###")
    ll_metrics, ll_results, _ = run_full_simulation(use_lane_locking=True, label="(V3 Baseline)")

    # ===== 3. Comparison =====
    print_comparison_table(sqrt_metrics, ll_metrics)

    # ===== 4. Per-archetype convergence =====
    print("\n\n### 4. PER-ARCHETYPE CONVERGENCE TABLE ###")
    rng_conv = random.Random(42)
    pool_conv = build_card_pool(rng_conv)

    print("\n--- Sqrt Affinity ---")
    sqrt_conv = compute_per_archetype_convergence(pool_conv, random.Random(99),
                                                   generate_pack_sqrt, n_drafts=200)
    print(f"{'Archetype':30s} | {'Avg Convergence Pick':>20s}")
    print("-" * 55)
    for ai in range(NUM_ARCHETYPES):
        print(f"{ARCHETYPES[ai]:30s} | {sqrt_conv[ai]:20.1f}")

    print("\n--- Lane Locking ---")
    ll_conv = compute_per_archetype_convergence(pool_conv, random.Random(99),
                                                 generate_pack_lane_locking, n_drafts=200)
    print(f"{'Archetype':30s} | {'Avg Convergence Pick':>20s}")
    print("-" * 55)
    for ai in range(NUM_ARCHETYPES):
        print(f"{ARCHETYPES[ai]:30s} | {ll_conv[ai]:20.1f}")

    # ===== 5. Parameter sensitivity sweeps =====
    print("\n\n### 5. PARAMETER SENSITIVITY SWEEPS ###")

    # 5a. Base weight sweep
    print("\n--- Base Weight Sweep ---")
    for bw in [1.0, 1.25, 1.5, 1.75, 2.0]:
        m, _, _ = run_full_simulation(base=bw, label=f"base={bw}", quiet=True)
        print(f"  base={bw:.2f}: late_sa={m['late_sa_committed']:.2f}, "
              f"off_arch={m['late_off_archetype']:.2f}, "
              f"stddev={m['late_sa_stddev']:.2f}, "
              f"conc={m['deck_concentration']:.1%}")

    # 5b. Cap sweep
    print("\n--- Cap Sweep ---")
    for cp in [3.0, 4.0, 4.5, 5.5, 7.0, 999]:
        m, _, _ = run_full_simulation(cap=cp, label=f"cap={cp}", quiet=True)
        print(f"  cap={cp:5.1f}: late_sa={m['late_sa_committed']:.2f}, "
              f"off_arch={m['late_off_archetype']:.2f}, "
              f"stddev={m['late_sa_stddev']:.2f}, "
              f"conc={m['deck_concentration']:.1%}")

    # 5c. Exponent sweep
    print("\n--- Exponent Sweep ---")
    for ex in [0.33, 0.4, 0.5, 0.67, 0.8, 1.0]:
        m, _, _ = run_full_simulation(exponent=ex, label=f"exp={ex}", quiet=True)
        print(f"  exp={ex:.2f}: late_sa={m['late_sa_committed']:.2f}, "
              f"off_arch={m['late_off_archetype']:.2f}, "
              f"stddev={m['late_sa_stddev']:.2f}, "
              f"conc={m['deck_concentration']:.1%}")

    # 5d. Symbol distribution sweep
    print("\n--- Symbol Distribution Sweep ---")
    dists = [
        (0.40, 0.40, 0.20, "1-sym heavy"),
        (0.25, 0.55, 0.20, "Default (2-sym)"),
        (0.20, 0.40, 0.40, "3-sym heavy"),
        (0.10, 0.60, 0.30, "Extreme 2-sym"),
        (0.50, 0.35, 0.15, "Extreme 1-sym"),
    ]
    for p1, p2, p3, desc in dists:
        m, _, _ = run_full_simulation(pct_1sym=p1, pct_2sym=p2, pct_3sym=p3,
                                       label=desc, quiet=True)
        print(f"  {desc:20s}: late_sa={m['late_sa_committed']:.2f}, "
              f"off_arch={m['late_off_archetype']:.2f}, "
              f"stddev={m['late_sa_stddev']:.2f}, "
              f"conc={m['deck_concentration']:.1%}")

    # ===== 6. Draft traces =====
    print("\n\n### 6. DRAFT TRACES ###")
    rng_trace = random.Random(123)
    trace_pool = build_card_pool(rng_trace)

    # Trace 1: Early committer (forced Warriors)
    dr1 = run_single_draft(trace_pool, "committed", random.Random(456),
                           generate_pack_sqrt, forced_arch=6)
    print(format_trace(dr1, "Early Committer (Warriors)"))

    # Trace 2: Flexible player
    dr2 = run_single_draft(trace_pool, "committed", random.Random(789),
                           generate_pack_sqrt, forced_arch=None)
    print(format_trace(dr2, "Flexible Player"))

    # Trace 3: Signal reader (with pool asymmetry)
    trace_pool_asym = build_card_pool(random.Random(321),
                                       removed_resonance="Ember",
                                       removal_fraction=0.2)
    dr3 = run_single_draft(trace_pool_asym, "signal", random.Random(654),
                           generate_pack_sqrt)
    print(format_trace(dr3, "Signal Reader (Ember depleted)"))

    # ===== 7. Pack quality variance report =====
    print("\n\n### 7. PACK QUALITY VARIANCE (Committed, picks 6+) ###")
    dist = sqrt_metrics['late_sa_distribution']
    print(f"  StdDev: {sqrt_metrics['late_sa_stddev']:.3f}")
    print(f"  Distribution:")
    for k in sorted(dist.keys()):
        bar = "#" * int(dist[k] * 50)
        print(f"    {k} S/A cards: {dist[k]:6.1%}  {bar}")

    # ===== 8. One-sentence claim verification =====
    print("\n\n### 8. ONE-SENTENCE CLAIM VERIFICATION ###")
    print("Claim: 'Each card in the pool is drawn with weight equal to 1.5 plus")
    print("the square root of its total resonance symbol overlap with your drafted")
    print("deck (capped at 4.5), so cards matching your deck appear more often but")
    print("with diminishing returns.'")
    print()
    print("Verification: The implementation in generate_pack_sqrt() computes:")
    print("  weight = base + min(sum(player_count[r] for r in card.symbols)^exponent, cap)")
    print(f"  With base={BASE_WEIGHT}, exponent={EXPONENT}, cap={SQRT_CAP}")
    print("  This exactly matches the one-sentence description.")
    print("  The algorithm uses ONLY visible card properties (symbols).")
    print("  Archetype fitness is used ONLY for evaluation metrics, never for pack generation.")

    print("\n\nSimulation complete.")
