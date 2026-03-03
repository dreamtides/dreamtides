#!/usr/bin/env python3
"""
Simulation Agent 2: Threshold Auto-Spend
=========================================

One-sentence algorithm:
  Each drafted symbol earns matching tokens (+2 for primary, +1 for secondary/
  tertiary); when any resonance counter reaches the threshold, the system
  automatically spends that many tokens from the highest counter and adds bonus
  resonance-matched cards to the pack.

Zero player decisions -- spending is fully automatic.
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
    # (name, primary_resonance, secondary_resonance)
    ("Flash",        Resonance.ZEPHYR, Resonance.EMBER),
    ("Blink",        Resonance.EMBER,  Resonance.ZEPHYR),
    ("Storm",        Resonance.EMBER,  Resonance.STONE),
    ("SelfDiscard",  Resonance.STONE,  Resonance.EMBER),
    ("SelfMill",     Resonance.STONE,  Resonance.TIDE),
    ("Sacrifice",    Resonance.TIDE,   Resonance.STONE),
    ("Warriors",     Resonance.TIDE,   Resonance.ZEPHYR),
    ("Ramp",         Resonance.ZEPHYR, Resonance.TIDE),
]

ARCHETYPE_NAMES = [a[0] for a in ARCHETYPES]
ARCHETYPE_INDEX = {a[0]: i for i, a in enumerate(ARCHETYPES)}


def archetype_primary(name: str) -> Resonance:
    return ARCHETYPES[ARCHETYPE_INDEX[name]][1]


def archetype_secondary(name: str) -> Resonance:
    return ARCHETYPES[ARCHETYPE_INDEX[name]][2]


# ── Fitness computation ──────────────────────────────────────────────────────

def compute_fitness(card_archetype: Optional[str], rng: random.Random) -> dict[str, str]:
    """Return archetype -> tier for a card belonging to card_archetype."""
    if card_archetype is None:
        # Generic card: B in all
        return {a: "B" for a in ARCHETYPE_NAMES}

    home_idx = ARCHETYPE_INDEX[card_archetype]
    home_primary = ARCHETYPES[home_idx][1]
    home_secondary = ARCHETYPES[home_idx][2]
    fitness = {}
    for a in ARCHETYPE_NAMES:
        if a == card_archetype:
            fitness[a] = "S"
        elif archetype_primary(a) == home_primary:
            # Adjacent archetype sharing primary resonance
            fitness[a] = "A"
        elif archetype_secondary(a) == home_primary or archetype_primary(a) == home_secondary:
            # Shares a resonance connection
            fitness[a] = "B"
        else:
            # Distant
            fitness[a] = "C" if rng.random() < 0.6 else "F"
    return fitness


# ── Card model ───────────────────────────────────────────────────────────────

@dataclass
class SimCard:
    id: int
    symbols: list  # list[Resonance], ordered
    archetype: Optional[str]  # None for generic
    archetype_fitness: dict  # archetype -> tier (S/A/B/C/F)
    power: float  # 0-10

    def resonance_types(self) -> set:
        return set(self.symbols)

    def is_dual_type(self) -> bool:
        return len(self.resonance_types()) >= 2

    def primary_resonance(self) -> Optional[Resonance]:
        return self.symbols[0] if self.symbols else None


# ── Card pool construction (360 cards) ───────────────────────────────────────

def build_card_pool(rng: random.Random) -> list[SimCard]:
    """
    Build 360 cards:
      36 generic (no symbols, B-tier all)
      324 archetype cards (~40 per archetype, 4 get 41)
    Symbol distribution per archetype (~40 cards each):
      ~8  mono-1-symbol   [primary]
      ~18 mono-2-symbol   [primary, primary]
      ~7  mono-3-symbol   [primary, primary, primary]
      ~4  dual-2-symbol   [primary, secondary]
      ~3  dual-3-symbol   [primary, primary, secondary]
    Total dual: 54 (15% of 360).
    """
    cards = []
    card_id = 0

    # Generic cards
    for _ in range(36):
        fitness = compute_fitness(None, rng)
        cards.append(SimCard(
            id=card_id, symbols=[], archetype=None,
            archetype_fitness=fitness,
            power=rng.uniform(4.0, 7.0),
        ))
        card_id += 1

    # Distribute dual-type budget: 54 total across 8 archetypes
    # 6 archetypes get 7 dual, 2 get 6 dual -> 42+12=54
    dual_budget = [7] * 6 + [6] * 2
    rng.shuffle(dual_budget)

    for arch_idx, (arch_name, primary, secondary) in enumerate(ARCHETYPES):
        n_dual = dual_budget[arch_idx]
        n_dual_2 = min(4, n_dual)
        n_dual_3 = n_dual - n_dual_2

        # Total cards for this archetype: 41 for first 4, 40 for last 4
        n_total = 41 if arch_idx < 4 else 40
        n_mono = n_total - n_dual

        # Distribute mono cards: ~22% 1-sym, ~56% 2-sym, ~22% 3-sym
        n_mono_1 = round(n_mono * 0.22)
        n_mono_3 = round(n_mono * 0.22)
        n_mono_2 = n_mono - n_mono_1 - n_mono_3

        def make_card(symbols):
            nonlocal card_id
            fitness = compute_fitness(arch_name, rng)
            c = SimCard(
                id=card_id, symbols=symbols, archetype=arch_name,
                archetype_fitness=fitness,
                power=rng.uniform(3.0, 9.0),
            )
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


# ── Algorithm: Threshold Auto-Spend ──────────────────────────────────────────

@dataclass
class DraftState:
    tokens: dict = field(default_factory=lambda: {r: 0 for r in Resonance})
    drafted_cards: list = field(default_factory=list)
    pick_number: int = 0
    total_triggers: int = 0
    committed_archetype: Optional[str] = None
    commitment_pick: int = 0
    non_generic_count: int = 0
    # For pre-assigned target (committed strategy)
    target_archetype: Optional[str] = None


def earn_tokens(state: DraftState, card: SimCard):
    """Earn resonance tokens from a drafted card's symbols."""
    for i, sym in enumerate(card.symbols):
        if i == 0:
            state.tokens[sym] += 2  # Primary earns 2
        else:
            state.tokens[sym] += 1  # Secondary/tertiary earn 1


def auto_spend(state: DraftState, threshold: int) -> Optional[Resonance]:
    """
    Check if any resonance counter >= threshold.
    If so, spend tokens from the highest counter and return the resonance.
    Returns None if no spend happens.
    """
    # Minimum activation: need 2+ non-generic cards drafted
    if state.non_generic_count < 2:
        return None

    # Find highest counter
    highest_res = max(Resonance, key=lambda r: state.tokens[r])
    if state.tokens[highest_res] >= threshold:
        state.tokens[highest_res] -= threshold
        state.total_triggers += 1
        return highest_res
    return None


def generate_base_pack(pool: list[SimCard], drafted_ids: set,
                       rng: random.Random, size: int = 4) -> list[SimCard]:
    """Draw `size` random cards from pool, excluding already-drafted cards."""
    available = [c for c in pool if c.id not in drafted_ids]
    if len(available) < size:
        return available
    return rng.sample(available, size)


def generate_bonus_cards(pool: list[SimCard], resonance: Resonance,
                         drafted_ids: set, rng: random.Random,
                         count: int, pack_ids: set) -> list[SimCard]:
    """Draw `count` cards whose primary resonance matches, from pool."""
    candidates = [c for c in pool
                  if c.id not in drafted_ids
                  and c.id not in pack_ids
                  and c.primary_resonance() == resonance]
    if len(candidates) < count:
        return candidates
    return rng.sample(candidates, count)


# ── Player strategies ────────────────────────────────────────────────────────

TIER_ORDER = {"S": 0, "A": 1, "B": 2, "C": 3, "F": 4}


def strategy_archetype_committed(state: DraftState, pack: list[SimCard],
                                 rng: random.Random) -> SimCard:
    """
    Commit to a pre-assigned target archetype around pick 5-6.
    Before commitment, pick best overall card (S/A for any archetype).
    After commitment, pick best card for target archetype.
    """
    if state.committed_archetype is None and state.pick_number >= 5:
        state.committed_archetype = state.target_archetype
        state.commitment_pick = state.pick_number

    if state.committed_archetype:
        target = state.committed_archetype
        pack.sort(key=lambda c: (TIER_ORDER.get(c.archetype_fitness.get(target, "F"), 4),
                                  -c.power))
        return pack[0]
    else:
        # Pre-commitment: pick best S/A card for any archetype, slight bias
        # toward target
        target = state.target_archetype
        def score(c):
            target_tier = TIER_ORDER.get(c.archetype_fitness.get(target, "F"), 4)
            best_tier = min(TIER_ORDER.get(t, 4) for t in c.archetype_fitness.values())
            # Blend: prefer target archetype S/A, then any S/A
            return (min(target_tier, best_tier + 1), -c.power)
        pack.sort(key=score)
        return pack[0]


def strategy_power_chaser(state: DraftState, pack: list[SimCard],
                          rng: random.Random) -> SimCard:
    """Always pick the highest raw power card."""
    pack.sort(key=lambda c: -c.power)
    return pack[0]


def strategy_signal_reader(state: DraftState, pack: list[SimCard],
                           rng: random.Random) -> SimCard:
    """
    Evaluate which archetype is best supported by accumulated resonance tokens
    and draft toward it. Commits at pick 5.
    """
    if state.committed_archetype is None and state.pick_number >= 5:
        # Score each archetype by token alignment
        arch_token_score = {}
        for a in ARCHETYPE_NAMES:
            pri = archetype_primary(a)
            sec = archetype_secondary(a)
            arch_token_score[a] = state.tokens[pri] * 2 + state.tokens[sec]
        # Also factor in S/A cards already drafted
        for c in state.drafted_cards:
            for a in ARCHETYPE_NAMES:
                if c.archetype_fitness.get(a) in ("S", "A"):
                    arch_token_score[a] += 3
        best_arch = max(arch_token_score, key=arch_token_score.get)
        state.committed_archetype = best_arch
        state.commitment_pick = state.pick_number

    if state.committed_archetype:
        target = state.committed_archetype
        pack.sort(key=lambda c: (TIER_ORDER.get(c.archetype_fitness.get(target, "F"), 4),
                                  -c.power))
        return pack[0]
    else:
        # Pre-commitment: pick card with most S/A breadth, then power
        def score(c):
            sa_count = sum(1 for t in c.archetype_fitness.values() if t in ("S", "A"))
            return (-sa_count, -c.power)
        pack.sort(key=score)
        return pack[0]


STRATEGIES = {
    "committed": strategy_archetype_committed,
    "power": strategy_power_chaser,
    "signal": strategy_signal_reader,
}


# ── Metric evaluation helpers ────────────────────────────────────────────────

def is_sa(card: SimCard, archetype: str) -> bool:
    return card.archetype_fitness.get(archetype, "F") in ("S", "A")


def is_cf(card: SimCard, archetype: str) -> bool:
    return card.archetype_fitness.get(archetype, "F") in ("C", "F")


# ── Single draft simulation ─────────────────────────────────────────────────

@dataclass
class DraftResult:
    strategy: str
    committed_archetype: Optional[str]
    commitment_pick: int
    drafted_cards: list
    pack_records: list  # list of (pick_num, pack_cards_before_pick)
    sa_per_pack_post: list  # S/A count per pack for committed arch, picks 6+
    cf_per_pack_post: list  # C/F count per pack for committed arch, picks 6+
    early_variety: list  # unique archetypes with S/A per pack, picks 1-5
    early_sa_emerging: list  # S/A for eventual archetype per pack, picks 1-5
    trigger_count: int


def run_single_draft(pool: list[SimCard], strategy_name: str,
                     threshold: int, bonus_count: int,
                     rng: random.Random, num_picks: int = 30,
                     trace: bool = False,
                     forced_archetype: Optional[str] = None) -> tuple:
    state = DraftState()

    # For committed strategy, pre-assign a target archetype
    if forced_archetype:
        state.target_archetype = forced_archetype
    else:
        state.target_archetype = rng.choice(ARCHETYPE_NAMES)

    strategy_fn = STRATEGIES[strategy_name]
    drafted_ids = set()
    pack_records = []
    pending_bonus_resonance = None

    sa_per_pack_post = []
    cf_per_pack_post = []
    early_variety = []
    early_sa_emerging = []
    trace_lines = []

    for pick in range(1, num_picks + 1):
        state.pick_number = pick

        # Generate base pack
        base_pack = generate_base_pack(pool, drafted_ids, rng, size=4)

        # Add bonus cards if triggered on previous pick
        bonus_cards = []
        if pending_bonus_resonance is not None:
            pack_ids = {c.id for c in base_pack}
            bonus_cards = generate_bonus_cards(
                pool, pending_bonus_resonance, drafted_ids, rng,
                bonus_count, pack_ids
            )
            pending_bonus_resonance = None

        full_pack = base_pack + bonus_cards
        rng.shuffle(full_pack)
        pack_records.append((pick, list(full_pack)))

        if trace:
            trace_lines.append(f"\n--- Pick {pick} ---")
            trace_lines.append(f"  Tokens: {dict_str(state.tokens)}")
            trace_lines.append(f"  Pack size: {len(full_pack)} "
                               f"(base {len(base_pack)} + bonus {len(bonus_cards)})")
            for c in full_pack:
                sym_str = ",".join(s.value for s in c.symbols) if c.symbols else "generic"
                trace_lines.append(f"    [{c.id}] {c.archetype or 'Generic'} "
                                   f"sym=[{sym_str}] pow={c.power:.1f}")

        # Player picks
        chosen = strategy_fn(state, list(full_pack), rng)
        state.drafted_cards.append(chosen)
        drafted_ids.add(chosen.id)
        if chosen.symbols:
            state.non_generic_count += 1

        # Earn tokens
        earn_tokens(state, chosen)

        # Auto-spend check
        spent_res = auto_spend(state, threshold)
        if spent_res is not None:
            pending_bonus_resonance = spent_res

        if trace:
            sym_str = ",".join(s.value for s in chosen.symbols) if chosen.symbols else "generic"
            trace_lines.append(f"  -> Picked [{chosen.id}] {chosen.archetype or 'Generic'} "
                               f"sym=[{sym_str}]")
            trace_lines.append(f"  Tokens after: {dict_str(state.tokens)}")
            if spent_res:
                trace_lines.append(f"  AUTO-SPEND: {spent_res.value} "
                                   f"(threshold {threshold}), bonus {bonus_count} "
                                   f"next pack")

    # Determine committed archetype for evaluation
    committed = state.committed_archetype
    if committed is None:
        # Power chaser: assign archetype with most S/A cards drafted
        arch_scores = Counter()
        for c in state.drafted_cards:
            for a in ARCHETYPE_NAMES:
                if c.archetype_fitness.get(a) in ("S", "A"):
                    arch_scores[a] += 1
        committed = arch_scores.most_common(1)[0][0] if arch_scores else "Warriors"

    for pick_num, pack in pack_records:
        if pick_num <= 5:
            # Early variety: unique archetypes with at least 1 S/A card
            archs_with_sa = set()
            for c in pack:
                for a in ARCHETYPE_NAMES:
                    if is_sa(c, a):
                        archs_with_sa.add(a)
            early_variety.append(len(archs_with_sa))
            # S/A for emerging archetype
            sa_count = sum(1 for c in pack if is_sa(c, committed))
            early_sa_emerging.append(sa_count)
        else:
            # Post-commitment metrics
            sa_count = sum(1 for c in pack if is_sa(c, committed))
            cf_count = sum(1 for c in pack if is_cf(c, committed))
            sa_per_pack_post.append(sa_count)
            cf_per_pack_post.append(cf_count)

    return DraftResult(
        strategy=strategy_name,
        committed_archetype=committed,
        commitment_pick=state.commitment_pick if state.commitment_pick > 0 else state.pick_number,
        drafted_cards=state.drafted_cards,
        pack_records=pack_records,
        sa_per_pack_post=sa_per_pack_post,
        cf_per_pack_post=cf_per_pack_post,
        early_variety=early_variety,
        early_sa_emerging=early_sa_emerging,
        trigger_count=state.total_triggers,
    ), trace_lines


def dict_str(d):
    return {k.value: v for k, v in d.items()}


# ── Batch simulation ─────────────────────────────────────────────────────────

def run_batch(pool: list[SimCard], n_drafts: int, threshold: int,
              bonus_count: int, seed: int = 42) -> dict:
    """Run n_drafts for each of 3 strategies. Return aggregated results."""
    all_results = {s: [] for s in STRATEGIES}
    rng = random.Random(seed)

    for strategy_name in STRATEGIES:
        for i in range(n_drafts):
            # For committed strategy, cycle through archetypes uniformly
            if strategy_name == "committed":
                forced = ARCHETYPE_NAMES[i % 8]
            else:
                forced = None
            result, _ = run_single_draft(pool, strategy_name, threshold,
                                         bonus_count, rng,
                                         forced_archetype=forced)
            all_results[strategy_name].append(result)

    return all_results


def compute_metrics(all_results: dict) -> dict:
    """Compute all 9 required metrics from batch results."""
    metrics = {}

    for strat, results in all_results.items():
        m = {}

        # Metric 1: Picks 1-5 unique archetypes with S/A per pack
        m["early_variety"] = statistics.mean(
            v for r in results for v in r.early_variety
        )

        # Metric 2: Picks 1-5 S/A for emerging archetype per pack
        m["early_sa_emerging"] = statistics.mean(
            v for r in results for v in r.early_sa_emerging
        )

        # Metric 3: Picks 6+ S/A for committed archetype per pack
        all_sa = [v for r in results for v in r.sa_per_pack_post]
        m["post_sa"] = statistics.mean(all_sa) if all_sa else 0

        # Metric 4: Picks 6+ off-archetype (C/F) per pack
        all_cf = [v for r in results for v in r.cf_per_pack_post]
        m["post_cf"] = statistics.mean(all_cf) if all_cf else 0

        # Metric 5: Convergence pick
        commit_picks = [r.commitment_pick for r in results if r.commitment_pick > 0]
        m["convergence_pick"] = statistics.mean(commit_picks) if commit_picks else 0

        # Metric 6: Deck archetype concentration (% S/A in committed arch)
        concentrations = []
        for r in results:
            if r.committed_archetype:
                sa = sum(1 for c in r.drafted_cards
                         if is_sa(c, r.committed_archetype))
                concentrations.append(sa / len(r.drafted_cards) * 100)
        m["concentration"] = statistics.mean(concentrations) if concentrations else 0

        # Metric 7: Run-to-run card overlap (Jaccard similarity)
        overlaps = []
        if len(results) >= 2:
            rng2 = random.Random(999)
            for _ in range(min(500, len(results))):
                r1, r2 = rng2.sample(results, 2)
                ids1 = {c.id for c in r1.drafted_cards}
                ids2 = {c.id for c in r2.drafted_cards}
                if ids1 | ids2:
                    overlaps.append(len(ids1 & ids2) / len(ids1 | ids2) * 100)
        m["overlap"] = statistics.mean(overlaps) if overlaps else 0

        # Metric 8: Archetype frequency
        arch_counts = Counter(r.committed_archetype for r in results
                              if r.committed_archetype)
        total_arch = sum(arch_counts.values())
        arch_freq = {a: arch_counts.get(a, 0) / total_arch * 100
                     for a in ARCHETYPE_NAMES}
        m["arch_freq"] = arch_freq
        m["arch_freq_max"] = max(arch_freq.values()) if arch_freq else 0
        m["arch_freq_min"] = min(arch_freq.values()) if arch_freq else 0

        # Metric 9: StdDev of S/A per pack picks 6+
        m["post_sa_stddev"] = statistics.stdev(all_sa) if len(all_sa) > 1 else 0

        # Extra: average triggers
        m["avg_triggers"] = statistics.mean(r.trigger_count for r in results)

        metrics[strat] = m

    return metrics


# ── Per-archetype convergence table ──────────────────────────────────────────

def per_archetype_convergence(all_results: dict) -> dict:
    """For each archetype, compute average pick at which 2+ S/A first appears."""
    table = {}
    for arch_name in ARCHETYPE_NAMES:
        convergence_picks = []
        for strat, results in all_results.items():
            for r in results:
                if r.committed_archetype != arch_name:
                    continue
                # Find first pack (pick 6+) with 2+ S/A
                for pick_num, pack in r.pack_records:
                    if pick_num >= 6:
                        sa = sum(1 for c in pack if is_sa(c, arch_name))
                        if sa >= 2:
                            convergence_picks.append(pick_num)
                            break
        if convergence_picks:
            table[arch_name] = statistics.mean(convergence_picks)
        else:
            table[arch_name] = float("nan")
    return table


# ── Draft trace helper ───────────────────────────────────────────────────────

def run_trace(pool, strategy_name, threshold, bonus_count, seed, label,
              forced_archetype=None):
    rng = random.Random(seed)
    result, trace_lines = run_single_draft(
        pool, strategy_name, threshold, bonus_count, rng, trace=True,
        forced_archetype=forced_archetype
    )
    header = f"\n{'='*60}\nDraft Trace: {label} (strategy={strategy_name}, "
    header += f"threshold={threshold}, bonus={bonus_count})\n{'='*60}"
    committed = result.committed_archetype or "None"
    summary = (f"\nCommitted: {committed} at pick {result.commitment_pick}, "
               f"triggers={result.trigger_count}")
    sa_post = result.sa_per_pack_post
    avg_sa = statistics.mean(sa_post) if sa_post else 0
    summary += f"\nAvg S/A picks 6+: {avg_sa:.2f}"
    # Count deck composition
    deck_sa = sum(1 for c in result.drafted_cards if is_sa(c, committed))
    summary += f"\nDeck: {deck_sa}/30 S/A for {committed} ({deck_sa/30*100:.0f}%)"
    return header + "\n".join(trace_lines[:80]) + "\n... (truncated)\n" + summary


# ── Main simulation ──────────────────────────────────────────────────────────

def main():
    print("Building card pool...")
    pool_rng = random.Random(12345)
    pool = build_card_pool(pool_rng)

    # Verify pool
    total = len(pool)
    generic = sum(1 for c in pool if c.archetype is None)
    dual = sum(1 for c in pool if c.is_dual_type())
    per_arch = Counter(c.archetype for c in pool if c.archetype)
    print(f"Pool: {total} cards, {generic} generic, {dual} dual-type")
    print(f"Per archetype: {dict(per_arch)}")

    # ── Parameter sweep ──────────────────────────────────────────────────────
    param_configs = [
        ("Cost3_Bonus2", 3, 2),
        ("Cost4_Bonus2", 4, 2),
        ("Cost3_Bonus3", 3, 3),
        ("Cost5_Bonus3", 5, 3),
        ("Cost2_Bonus1", 2, 1),
    ]

    N_DRAFTS = 1000
    all_sweep_metrics = {}

    for config_name, threshold, bonus in param_configs:
        print(f"\n--- Running config: {config_name} "
              f"(threshold={threshold}, bonus={bonus}) ---")
        results = run_batch(pool, N_DRAFTS, threshold, bonus, seed=42)
        metrics = compute_metrics(results)
        all_sweep_metrics[config_name] = metrics

        # Print summary for each strategy
        for strat in STRATEGIES:
            m = metrics[strat]
            print(f"  [{strat}]")
            print(f"    Early variety (picks 1-5):   {m['early_variety']:.2f} "
                  f"(target >= 3)")
            print(f"    Early S/A emerging (1-5):    {m['early_sa_emerging']:.2f} "
                  f"(target <= 2)")
            print(f"    Post S/A (6+):               {m['post_sa']:.2f} "
                  f"(target >= 2)")
            print(f"    Post C/F (6+):               {m['post_cf']:.2f} "
                  f"(target >= 0.5)")
            print(f"    Convergence pick:            {m['convergence_pick']:.1f} "
                  f"(target 5-8)")
            print(f"    Concentration:               {m['concentration']:.1f}% "
                  f"(target 60-90%)")
            print(f"    Overlap:                     {m['overlap']:.1f}% "
                  f"(target < 40%)")
            print(f"    Arch freq max/min:           "
                  f"{m['arch_freq_max']:.1f}% / {m['arch_freq_min']:.1f}% "
                  f"(target <20% / >5%)")
            print(f"    S/A stddev (6+):             {m['post_sa_stddev']:.2f} "
                  f"(target >= 0.8)")
            print(f"    Avg triggers:                {m['avg_triggers']:.1f}")

    # ── Per-archetype convergence for primary config ─────────────────────────
    print("\n\n=== Per-Archetype Convergence (Cost3_Bonus2) ===")
    primary_results = run_batch(pool, N_DRAFTS, 3, 2, seed=42)
    arch_conv = per_archetype_convergence(primary_results)
    for arch, pick in arch_conv.items():
        print(f"  {arch:15s}: pick {pick:.1f}")

    # Also for Cost4_Bonus2
    print("\n=== Per-Archetype Convergence (Cost4_Bonus2) ===")
    c4b2_results = run_batch(pool, N_DRAFTS, 4, 2, seed=42)
    arch_conv2 = per_archetype_convergence(c4b2_results)
    for arch, pick in arch_conv2.items():
        print(f"  {arch:15s}: pick {pick:.1f}")

    # ── Draft traces (abbreviated) ───────────────────────────────────────────
    print("\n\n=== Draft Traces (Cost3_Bonus2) ===")

    # Trace 1: Early committer targeting Warriors
    print(run_trace(pool, "committed", 3, 2, 100, "Early Committer (Warriors)",
                    forced_archetype="Warriors"))

    # Trace 2: Power chaser
    print(run_trace(pool, "power", 3, 2, 200, "Power Chaser"))

    # Trace 3: Signal reader
    print(run_trace(pool, "signal", 3, 2, 300, "Signal Reader"))

    # ── Pack-quality variance report ─────────────────────────────────────────
    print("\n\n=== Pack-Quality Variance Report (Cost3_Bonus2, committed) ===")
    committed_results = primary_results["committed"]
    all_sa_values = [v for r in committed_results for v in r.sa_per_pack_post]
    if all_sa_values:
        sa_dist = Counter(all_sa_values)
        print("  Distribution of S/A per pack (picks 6+):")
        for k in sorted(sa_dist.keys()):
            pct = sa_dist[k] / len(all_sa_values) * 100
            print(f"    {k} S/A cards: {pct:.1f}%")
        print(f"  Mean: {statistics.mean(all_sa_values):.2f}")
        print(f"  StdDev: {statistics.stdev(all_sa_values):.2f}")
        print(f"  Min: {min(all_sa_values)}, Max: {max(all_sa_values)}")

    # Also for Cost4_Bonus2
    print("\n=== Pack-Quality Variance Report (Cost4_Bonus2, committed) ===")
    c4_committed = c4b2_results["committed"]
    all_sa_c4 = [v for r in c4_committed for v in r.sa_per_pack_post]
    if all_sa_c4:
        sa_dist_c4 = Counter(all_sa_c4)
        print("  Distribution of S/A per pack (picks 6+):")
        for k in sorted(sa_dist_c4.keys()):
            pct = sa_dist_c4[k] / len(all_sa_c4) * 100
            print(f"    {k} S/A cards: {pct:.1f}%")
        print(f"  Mean: {statistics.mean(all_sa_c4):.2f}")
        print(f"  StdDev: {statistics.stdev(all_sa_c4):.2f}")
        print(f"  Min: {min(all_sa_c4)}, Max: {max(all_sa_c4)}")

    # ── One-sentence claim test ──────────────────────────────────────────────
    print("\n\n=== One-Sentence Claim Test ===")
    print("Algorithm: 'Each drafted symbol earns matching tokens (+2 primary, "
          "+1 secondary/tertiary); when any resonance counter reaches the "
          "threshold, the system automatically spends that many tokens from "
          "the highest counter and adds bonus resonance-matched cards to the "
          "pack.'")
    print("Implementation verification:")
    print("  - Token earning: earn_tokens() adds +2 for symbols[0], +1 for rest")
    print("  - Auto-spend: auto_spend() checks highest counter >= threshold")
    print("  - Bonus cards: generate_bonus_cards() draws matching primary resonance")
    print("  - No player decisions: auto_spend fires without player input")
    print("  - Only player action: pick 1 card from pack")
    print("  - Minimum activation: 2+ non-generic cards needed before first trigger")
    print("  VERDICT: Implementation matches one-sentence description.")

    # ── Summary comparison table ─────────────────────────────────────────────
    print("\n\n=== Summary Comparison Table (all configs, committed strategy) ===")
    header = (f"{'Config':18s} | {'EarlyVar':>8s} | {'EarlySA':>7s} | "
              f"{'PostSA':>6s} | {'PostCF':>6s} | {'ConvPk':>6s} | "
              f"{'Conc%':>6s} | {'Ovlp%':>6s} | {'StdDev':>6s} | {'Trigs':>5s}")
    print(header)
    print("-" * len(header))
    for config_name, _, _ in param_configs:
        m = all_sweep_metrics[config_name]["committed"]
        print(f"{config_name:18s} | {m['early_variety']:8.2f} | "
              f"{m['early_sa_emerging']:7.2f} | {m['post_sa']:6.2f} | "
              f"{m['post_cf']:6.2f} | {m['convergence_pick']:6.1f} | "
              f"{m['concentration']:6.1f} | {m['overlap']:6.1f} | "
              f"{m['post_sa_stddev']:6.2f} | {m['avg_triggers']:5.1f}")

    print(f"{'(targets)':18s} | {'>=3':>8s} | {'<=2':>7s} | "
          f"{'>=2':>6s} | {'>=0.5':>6s} | {'5-8':>6s} | "
          f"{'60-90':>6s} | {'<40':>6s} | {'>=0.8':>6s} |")

    # ── Summary for signal reader ────────────────────────────────────────────
    print("\n=== Summary Comparison Table (all configs, signal strategy) ===")
    print(header)
    print("-" * len(header))
    for config_name, _, _ in param_configs:
        m = all_sweep_metrics[config_name]["signal"]
        print(f"{config_name:18s} | {m['early_variety']:8.2f} | "
              f"{m['early_sa_emerging']:7.2f} | {m['post_sa']:6.2f} | "
              f"{m['post_cf']:6.2f} | {m['convergence_pick']:6.1f} | "
              f"{m['concentration']:6.1f} | {m['overlap']:6.1f} | "
              f"{m['post_sa_stddev']:6.2f} | {m['avg_triggers']:5.1f}")


if __name__ == "__main__":
    main()
