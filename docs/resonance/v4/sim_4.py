#!/usr/bin/env python3
"""
Resonance V4 Simulation — Agent 4: Multiple Phantoms, Ecosystem Competition

Algorithm (one-sentence):
  "Two phantom drafters, each assigned a random resonance (sometimes the same
   one), each remove the best-matching card from the pool each round; you draft
   from what remains."

Also implements V3 Lane Locking (threshold 3/8, primary=2) for comparison.
"""

import random
import statistics
import copy
from dataclasses import dataclass, field
from enum import Enum
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

# Archetype definitions: (name, primary_res, secondary_res, circle_position)
ARCHETYPES = [
    ("Flash/Tempo/Prison",   Resonance.ZEPHYR, Resonance.EMBER,  1),
    ("Blink/Flicker",        Resonance.EMBER,  Resonance.ZEPHYR, 2),
    ("Storm/Spellslinger",   Resonance.EMBER,  Resonance.STONE,  3),
    ("Self-Discard",         Resonance.STONE,  Resonance.EMBER,  4),
    ("Self-Mill/Reanimator", Resonance.STONE,  Resonance.TIDE,   5),
    ("Sacrifice/Abandon",    Resonance.TIDE,   Resonance.STONE,  6),
    ("Warriors/Midrange",    Resonance.TIDE,   Resonance.ZEPHYR, 7),
    ("Ramp/Spirit Animals",  Resonance.ZEPHYR, Resonance.TIDE,   8),
]

ARCHETYPE_NAMES = [a[0] for a in ARCHETYPES]


def archetypes_adjacent(i: int, j: int) -> bool:
    """Check if two archetype indices are adjacent on the circle."""
    diff = abs(i - j)
    return diff == 1 or diff == 7  # 7 = wrap around 8-element circle


@dataclass
class SimCard:
    id: int
    symbols: list  # list of Resonance, ordered, 0-3 elements
    archetype: str  # primary archetype name
    archetype_idx: int  # index into ARCHETYPES
    archetype_fitness: dict  # archetype_name -> Tier
    power: float  # 0-10


# ---------------------------------------------------------------------------
# Card pool construction
# ---------------------------------------------------------------------------

def build_card_pool(
    sym_dist: tuple = (0.35, 0.45, 0.20),
    seed: Optional[int] = None,
) -> list[SimCard]:
    """Build 360-card pool.

    sym_dist: (frac_1sym, frac_2sym, frac_3sym) for non-generic cards.
    """
    rng = random.Random(seed)
    cards: list[SimCard] = []
    card_id = 0

    # 36 generic cards
    for _ in range(36):
        fitness = {name: Tier.B for name in ARCHETYPE_NAMES}
        cards.append(SimCard(
            id=card_id, symbols=[], archetype="Generic",
            archetype_idx=-1, archetype_fitness=fitness,
            power=round(rng.uniform(3, 8), 2),
        ))
        card_id += 1

    # 324 archetype cards (40-41 per archetype)
    per_arch = [40] * 8
    # distribute remaining 4 cards (324 - 320 = 4)
    for i in range(4):
        per_arch[i] += 1

    frac_1, frac_2, frac_3 = sym_dist
    for arch_idx, (name, prim, sec, pos) in enumerate(ARCHETYPES):
        n = per_arch[arch_idx]
        # Determine how many 1/2/3-symbol cards
        n1 = round(n * frac_1)
        n3 = round(n * frac_3)
        n2 = n - n1 - n3
        for sym_count, count in [(1, n1), (2, n2), (3, n3)]:
            for _ in range(count):
                symbols = _make_symbols(prim, sec, sym_count, rng)
                fitness = _compute_fitness(arch_idx)
                cards.append(SimCard(
                    id=card_id, symbols=symbols, archetype=name,
                    archetype_idx=arch_idx, archetype_fitness=fitness,
                    power=round(rng.uniform(2, 9), 2),
                ))
                card_id += 1

    rng.shuffle(cards)
    return cards


def _make_symbols(prim: Resonance, sec: Resonance, count: int,
                  rng: random.Random) -> list:
    """Generate symbol list for a card of the given archetype."""
    if count == 1:
        # 75% primary, 25% secondary
        return [prim if rng.random() < 0.75 else sec]
    elif count == 2:
        # first symbol: 80% primary, 20% secondary
        first = prim if rng.random() < 0.80 else sec
        second = sec if first == prim else prim
        return [first, second]
    else:  # 3
        # [prim, sec, prim] or [prim, prim, sec]
        if rng.random() < 0.5:
            return [prim, sec, prim]
        else:
            return [prim, prim, sec]


def _compute_fitness(home_idx: int) -> dict:
    """Compute archetype fitness tiers for a card from archetype home_idx."""
    fitness = {}
    home_name, home_prim, home_sec, _ = ARCHETYPES[home_idx]
    for j, (aname, aprim, asec, _) in enumerate(ARCHETYPES):
        if j == home_idx:
            fitness[aname] = Tier.S
        elif archetypes_adjacent(home_idx, j):
            # Adjacent archetype sharing primary resonance → A
            # Adjacent sharing only secondary → B
            if aprim == home_prim or asec == home_prim:
                fitness[aname] = Tier.A
            else:
                fitness[aname] = Tier.B
        elif aprim == home_sec or asec == home_sec:
            fitness[aname] = Tier.B
        else:
            # Check for any resonance overlap
            arch_res = {aprim, asec}
            home_res = {home_prim, home_sec}
            if arch_res & home_res:
                fitness[aname] = Tier.C
            else:
                fitness[aname] = Tier.F
    return fitness


def card_symbol_score(card: SimCard, res: Resonance) -> float:
    """Weighted symbol score for a card w.r.t. a resonance. Primary=2, rest=1."""
    score = 0.0
    for i, s in enumerate(card.symbols):
        if s == res:
            score += 2.0 if i == 0 else 1.0
    return score


def card_is_sa(card: SimCard, archetype_name: str) -> bool:
    """Is this card S or A tier for the given archetype?"""
    t = card.archetype_fitness.get(archetype_name, Tier.F)
    return t in (Tier.S, Tier.A)


def card_is_cf(card: SimCard, archetype_name: str) -> bool:
    """Is this card C or F tier for the given archetype?"""
    t = card.archetype_fitness.get(archetype_name, Tier.F)
    return t in (Tier.C, Tier.F)


# ---------------------------------------------------------------------------
# Player symbol counting (for Lane Locking baseline)
# ---------------------------------------------------------------------------

def player_resonance_counts(drafted: list[SimCard]) -> dict:
    """Return weighted resonance counts for drafted cards."""
    counts = {r: 0.0 for r in Resonance}
    for c in drafted:
        for i, s in enumerate(c.symbols):
            counts[s] += 2.0 if i == 0 else 1.0
    return counts


# ---------------------------------------------------------------------------
# Draft algorithms
# ---------------------------------------------------------------------------

def draw_pack_random(pool: list[SimCard], n: int, rng: random.Random) -> list[SimCard]:
    """Draw n random cards from the pool (without removal)."""
    if len(pool) <= n:
        return list(pool)
    return rng.sample(pool, n)


# --- Phantom Drafter Algorithm ---

def phantom_remove(pool: list[SimCard], phantom_resonances: list[Resonance],
                   cards_per_phantom: int, rng: random.Random) -> list[SimCard]:
    """Each phantom removes cards_per_phantom best-matching cards from pool.
    Returns the new pool (cards are removed in place conceptually)."""
    removed = set()
    for res in phantom_resonances:
        # Sort pool by symbol score for this resonance, descending
        candidates = [c for c in pool if c.id not in removed]
        candidates.sort(key=lambda c: (-card_symbol_score(c, res), rng.random()))
        for i in range(min(cards_per_phantom, len(candidates))):
            if card_symbol_score(candidates[i], res) > 0:
                removed.add(candidates[i].id)
    return [c for c in pool if c.id not in removed]


def run_phantom_draft(
    pool: list[SimCard],
    num_phantoms: int,
    cards_per_phantom: int,
    strategy: str,  # "committed", "power", "signal"
    rng: random.Random,
    target_archetype_idx: Optional[int] = None,
) -> dict:
    """Run a single 30-pick phantom draft. Returns metrics dict."""
    pool = list(pool)  # copy
    drafted: list[SimCard] = []

    # Assign phantom resonances
    all_res = list(Resonance)
    phantom_resonances = rng.sample(all_res, min(num_phantoms, 4))
    while len(phantom_resonances) < num_phantoms:
        phantom_resonances.append(rng.choice(all_res))

    # Determine target archetype based on strategy
    if strategy == "committed":
        if target_archetype_idx is None:
            target_archetype_idx = rng.randint(0, 7)
        target_name = ARCHETYPE_NAMES[target_archetype_idx]
        commit_pick = 5
    elif strategy == "power":
        target_archetype_idx = rng.randint(0, 7)
        target_name = ARCHETYPE_NAMES[target_archetype_idx]
        commit_pick = 30  # never really commits for archetype purposes
    elif strategy == "signal":
        target_archetype_idx = None
        target_name = None
        commit_pick = 6
    else:
        raise ValueError(f"Unknown strategy: {strategy}")

    pick_records = []

    for pick_num in range(1, 31):
        # Phantoms remove cards
        pool = phantom_remove(pool, phantom_resonances, cards_per_phantom, rng)

        # Generate pack
        pack = draw_pack_random(pool, 4, rng)
        if not pack:
            break

        # Signal reader: at commit_pick, choose the most available archetype
        if strategy == "signal" and pick_num == commit_pick:
            # Count which resonances are most available in pool
            res_counts = {r: 0 for r in Resonance}
            for c in pool:
                for s in c.symbols:
                    res_counts[s] += 1
            # Find the most available resonance
            open_res = max(res_counts, key=res_counts.get)
            # Pick the archetype with this as primary
            candidates = [i for i, (_, prim, _, _) in enumerate(ARCHETYPES)
                          if prim == open_res]
            target_archetype_idx = rng.choice(candidates)
            target_name = ARCHETYPE_NAMES[target_archetype_idx]

        # Choose a card based on strategy
        if strategy == "committed" and pick_num >= commit_pick:
            # Pick best S/A card for target, else highest power
            sa_cards = [c for c in pack if card_is_sa(c, target_name)]
            if sa_cards:
                chosen = max(sa_cards, key=lambda c: c.power)
            else:
                chosen = max(pack, key=lambda c: c.power)
        elif strategy == "power":
            chosen = max(pack, key=lambda c: c.power)
        elif strategy == "signal":
            if target_name and pick_num >= commit_pick:
                sa_cards = [c for c in pack if card_is_sa(c, target_name)]
                if sa_cards:
                    chosen = max(sa_cards, key=lambda c: c.power)
                else:
                    chosen = max(pack, key=lambda c: c.power)
            else:
                chosen = max(pack, key=lambda c: c.power)
        else:
            # Early picks for committed: pick best power
            chosen = max(pack, key=lambda c: c.power)

        # Record
        sa_count = 0
        cf_count = 0
        if target_name:
            sa_count = sum(1 for c in pack if card_is_sa(c, target_name))
            cf_count = sum(1 for c in pack if card_is_cf(c, target_name))

        unique_archs = len(set(
            a for c in pack for a in ARCHETYPE_NAMES
            if card_is_sa(c, a)
        ))

        pick_records.append({
            "pick": pick_num,
            "pack_size": len(pack),
            "sa_in_pack": sa_count,
            "cf_in_pack": cf_count,
            "unique_archetypes_sa": unique_archs,
            "chosen": chosen,
            "pack": pack,
            "target": target_name,
        })

        # Remove chosen from pool
        pool = [c for c in pool if c.id != chosen.id]
        drafted.append(chosen)

    return _compute_metrics(pick_records, drafted, target_name)


# --- Lane Locking Algorithm (V3 baseline) ---

def run_lane_locking_draft(
    pool: list[SimCard],
    strategy: str,
    rng: random.Random,
    target_archetype_idx: Optional[int] = None,
    lock_threshold_1: int = 3,
    lock_threshold_2: int = 8,
    primary_slots: int = 2,
) -> dict:
    """V3 Lane Locking: at threshold symbol counts, lock pack slots to resonance."""
    pool = list(pool)
    drafted: list[SimCard] = []

    if strategy == "committed":
        if target_archetype_idx is None:
            target_archetype_idx = rng.randint(0, 7)
        target_name = ARCHETYPE_NAMES[target_archetype_idx]
        commit_pick = 5
    elif strategy == "power":
        target_archetype_idx = rng.randint(0, 7)
        target_name = ARCHETYPE_NAMES[target_archetype_idx]
        commit_pick = 30
    elif strategy == "signal":
        # Signal reader commits around pick 6 to whatever looks open
        target_archetype_idx = None
        target_name = None
        commit_pick = 6
    else:
        raise ValueError(f"Unknown strategy: {strategy}")

    pick_records = []

    for pick_num in range(1, 31):
        counts = player_resonance_counts(drafted)

        # Signal reader logic
        if strategy == "signal" and pick_num == commit_pick:
            # Choose most-accumulated resonance
            best_res = max(counts, key=counts.get)
            candidates = [i for i, (_, prim, _, _) in enumerate(ARCHETYPES)
                          if prim == best_res]
            target_archetype_idx = rng.choice(candidates) if candidates else rng.randint(0, 7)
            target_name = ARCHETYPE_NAMES[target_archetype_idx]

        # Determine locked slots
        locked_resonances = []
        sorted_res = sorted(counts, key=counts.get, reverse=True)
        top_res = sorted_res[0] if sorted_res else None
        second_res = sorted_res[1] if len(sorted_res) > 1 else None

        top_count = counts.get(top_res, 0) if top_res else 0
        second_count = counts.get(second_res, 0) if second_res else 0

        n_locked = 0
        if top_count >= lock_threshold_2:
            n_locked = primary_slots
        elif top_count >= lock_threshold_1:
            n_locked = primary_slots

        # Build pack
        pack = []
        # Locked slots: random card with matching resonance from pool
        for _ in range(n_locked):
            matching = [c for c in pool if c.id not in {p.id for p in pack}
                        and any(s == top_res for s in c.symbols)]
            if matching:
                pack.append(rng.choice(matching))

        # Fill remaining slots randomly
        remaining = 4 - len(pack)
        available = [c for c in pool if c.id not in {p.id for p in pack}]
        if available and remaining > 0:
            pack.extend(rng.sample(available, min(remaining, len(available))))

        if not pack:
            break

        # Choose card
        if target_name and pick_num >= commit_pick:
            sa_cards = [c for c in pack if card_is_sa(c, target_name)]
            if sa_cards:
                chosen = max(sa_cards, key=lambda c: c.power)
            else:
                chosen = max(pack, key=lambda c: c.power)
        else:
            chosen = max(pack, key=lambda c: c.power)

        sa_count = sum(1 for c in pack if card_is_sa(c, target_name)) if target_name else 0
        cf_count = sum(1 for c in pack if card_is_cf(c, target_name)) if target_name else 0
        unique_archs = len(set(
            a for c in pack for a in ARCHETYPE_NAMES
            if card_is_sa(c, a)
        ))

        pick_records.append({
            "pick": pick_num,
            "pack_size": len(pack),
            "sa_in_pack": sa_count,
            "cf_in_pack": cf_count,
            "unique_archetypes_sa": unique_archs,
            "chosen": chosen,
            "pack": pack,
            "target": target_name,
        })

        pool = [c for c in pool if c.id != chosen.id]
        drafted.append(chosen)

    return _compute_metrics(pick_records, drafted, target_name)


# ---------------------------------------------------------------------------
# Metric computation
# ---------------------------------------------------------------------------

def _compute_metrics(pick_records: list[dict], drafted: list[SimCard],
                     target_name: Optional[str]) -> dict:
    """Compute all 8+ metrics from pick records."""
    early = [r for r in pick_records if r["pick"] <= 5]
    late = [r for r in pick_records if r["pick"] >= 6]

    # Metric 1: Picks 1-5 unique archetypes with S/A per pack
    early_unique = (statistics.mean([r["unique_archetypes_sa"] for r in early])
                    if early else 0)

    # Metric 2: Picks 1-5 S/A for emerging archetype per pack
    early_sa = (statistics.mean([r["sa_in_pack"] for r in early])
                if early else 0)

    # Metric 3: Picks 6+ S/A for committed archetype per pack
    late_sa_values = [r["sa_in_pack"] for r in late]
    late_sa = statistics.mean(late_sa_values) if late_sa_values else 0

    # Metric 4: Picks 6+ off-archetype (C/F) per pack
    late_cf = (statistics.mean([r["cf_in_pack"] for r in late])
               if late else 0)

    # Metric 5: Convergence pick (first pick where rolling 3-pick avg >= 2.0 S/A)
    convergence_pick = 30
    for i in range(2, len(pick_records)):
        window = pick_records[max(0, i - 2):i + 1]
        avg = statistics.mean([r["sa_in_pack"] for r in window])
        if avg >= 2.0:
            convergence_pick = pick_records[i]["pick"]
            break

    # Metric 6: Deck archetype concentration
    if target_name:
        sa_drafted = sum(1 for c in drafted if card_is_sa(c, target_name))
        concentration = sa_drafted / len(drafted) if drafted else 0
    else:
        concentration = 0

    # Variance: stddev of S/A per pack for picks 6+
    late_sa_std = statistics.stdev(late_sa_values) if len(late_sa_values) > 1 else 0

    # Distribution of S/A counts in late packs
    sa_distribution = {0: 0, 1: 0, 2: 0, 3: 0, 4: 0}
    for v in late_sa_values:
        sa_distribution[min(v, 4)] = sa_distribution.get(min(v, 4), 0) + 1

    return {
        "early_unique_archs": early_unique,
        "early_sa": early_sa,
        "late_sa": late_sa,
        "late_cf": late_cf,
        "convergence_pick": convergence_pick,
        "concentration": concentration,
        "late_sa_std": late_sa_std,
        "late_sa_values": late_sa_values,
        "sa_distribution": sa_distribution,
        "drafted": drafted,
        "pick_records": pick_records,
        "target_name": target_name,
    }


# ---------------------------------------------------------------------------
# Simulation runner
# ---------------------------------------------------------------------------

def run_simulation(
    n_drafts: int = 1000,
    algorithm: str = "phantom",
    strategy: str = "committed",
    num_phantoms: int = 2,
    cards_per_phantom: int = 1,
    sym_dist: tuple = (0.35, 0.45, 0.20),
    target_archetype_idx: Optional[int] = None,
    seed: int = 42,
) -> dict:
    """Run n_drafts simulations and aggregate metrics."""
    rng = random.Random(seed)

    all_metrics = []
    all_card_overlaps = []

    for i in range(n_drafts):
        pool_seed = rng.randint(0, 10**9)
        draft_seed = rng.randint(0, 10**9)
        pool = build_card_pool(sym_dist=sym_dist, seed=pool_seed)
        draft_rng = random.Random(draft_seed)

        if algorithm == "phantom":
            metrics = run_phantom_draft(
                pool, num_phantoms, cards_per_phantom,
                strategy, draft_rng, target_archetype_idx,
            )
        elif algorithm == "lane_locking":
            metrics = run_lane_locking_draft(
                pool, strategy, draft_rng, target_archetype_idx,
            )
        else:
            raise ValueError(f"Unknown algorithm: {algorithm}")

        all_metrics.append(metrics)

    # Aggregate
    agg = {}
    for key in ["early_unique_archs", "early_sa", "late_sa", "late_cf",
                "convergence_pick", "concentration", "late_sa_std"]:
        values = [m[key] for m in all_metrics]
        agg[key] = statistics.mean(values)
        agg[f"{key}_std"] = statistics.stdev(values) if len(values) > 1 else 0

    # Aggregate SA distribution
    total_dist = {0: 0, 1: 0, 2: 0, 3: 0, 4: 0}
    for m in all_metrics:
        for k, v in m["sa_distribution"].items():
            total_dist[k] = total_dist.get(k, 0) + v
    total_packs = sum(total_dist.values())
    agg["sa_distribution"] = {k: v / total_packs if total_packs > 0 else 0
                              for k, v in total_dist.items()}

    # Run-to-run variety: card overlap between pairs of runs with same seed
    # Sample 100 pairs
    if len(all_metrics) >= 2:
        overlaps = []
        for _ in range(min(100, len(all_metrics) // 2)):
            i, j = rng.sample(range(len(all_metrics)), 2)
            cards_i = {c.id for c in all_metrics[i]["drafted"]}
            cards_j = {c.id for c in all_metrics[j]["drafted"]}
            overlap = len(cards_i & cards_j) / max(len(cards_i | cards_j), 1)
            overlaps.append(overlap)
        agg["card_overlap"] = statistics.mean(overlaps)
    else:
        agg["card_overlap"] = 0

    # Archetype frequency: how often each archetype is the target
    arch_freq = {name: 0 for name in ARCHETYPE_NAMES}
    for m in all_metrics:
        if m["target_name"]:
            arch_freq[m["target_name"]] += 1
    total = sum(arch_freq.values())
    agg["arch_frequency"] = {k: v / total if total > 0 else 0
                             for k, v in arch_freq.items()}

    agg["all_metrics"] = all_metrics
    return agg


def run_per_archetype_convergence(
    algorithm: str = "phantom",
    n_drafts: int = 200,
    num_phantoms: int = 2,
    cards_per_phantom: int = 1,
    sym_dist: tuple = (0.35, 0.45, 0.20),
    seed: int = 42,
) -> dict:
    """For each archetype, measure average convergence pick."""
    results = {}
    for arch_idx, (name, _, _, _) in enumerate(ARCHETYPES):
        agg = run_simulation(
            n_drafts=n_drafts, algorithm=algorithm, strategy="committed",
            num_phantoms=num_phantoms, cards_per_phantom=cards_per_phantom,
            sym_dist=sym_dist, target_archetype_idx=arch_idx,
            seed=seed + arch_idx,
        )
        results[name] = agg["convergence_pick"]
    return results


# ---------------------------------------------------------------------------
# Draft trace
# ---------------------------------------------------------------------------

def run_draft_trace(
    algorithm: str = "phantom",
    strategy: str = "committed",
    num_phantoms: int = 2,
    cards_per_phantom: int = 1,
    sym_dist: tuple = (0.35, 0.45, 0.20),
    target_archetype_idx: Optional[int] = None,
    seed: int = 99,
) -> dict:
    """Run a single draft and return detailed pick-by-pick trace."""
    pool = build_card_pool(sym_dist=sym_dist, seed=seed)
    rng = random.Random(seed + 1)

    if algorithm == "phantom":
        return run_phantom_draft(
            pool, num_phantoms, cards_per_phantom,
            strategy, rng, target_archetype_idx,
        )
    else:
        return run_lane_locking_draft(
            pool, strategy, rng, target_archetype_idx,
        )


def format_trace(metrics: dict, label: str, max_picks: int = 12) -> str:
    """Format a draft trace for display."""
    lines = [f"\n=== {label} ==="]
    lines.append(f"Target archetype: {metrics['target_name']}")
    for r in metrics["pick_records"][:max_picks]:
        pack_desc = []
        for c in r["pack"]:
            syms = "/".join(s.value for s in c.symbols) if c.symbols else "Generic"
            tier = c.archetype_fitness.get(metrics["target_name"], Tier.F).value if metrics["target_name"] else "?"
            pack_desc.append(f"[{c.archetype}({syms})={tier}]")
        chosen_syms = "/".join(s.value for s in r["chosen"].symbols) if r["chosen"].symbols else "Generic"
        lines.append(
            f"  Pick {r['pick']:2d}: SA={r['sa_in_pack']} CF={r['cf_in_pack']} | "
            f"Pack: {' '.join(pack_desc)} | "
            f"Chose: {r['chosen'].archetype}({chosen_syms})"
        )
    lines.append(
        f"  ... Late SA avg: {statistics.mean([r['sa_in_pack'] for r in metrics['pick_records'] if r['pick'] >= 6]):.2f}"
    )
    lines.append(f"  Convergence pick: {metrics['convergence_pick']}")
    lines.append(f"  Deck concentration: {metrics['concentration']:.1%}")
    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    print("=" * 70)
    print("RESONANCE V4 — AGENT 4: MULTIPLE PHANTOMS, ECOSYSTEM COMPETITION")
    print("=" * 70)

    SYM_DIST = (0.35, 0.45, 0.20)
    N_DRAFTS = 1000

    # ---- Main results: Phantom algorithm ----
    print("\n--- PHANTOM ALGORITHM (2 phantoms, 1 card/round each) ---")
    for strat in ["committed", "power", "signal"]:
        agg = run_simulation(
            n_drafts=N_DRAFTS, algorithm="phantom", strategy=strat,
            num_phantoms=2, cards_per_phantom=1, sym_dist=SYM_DIST,
        )
        print(f"\n  Strategy: {strat}")
        print(f"    Picks 1-5 unique archetypes w/ S/A: {agg['early_unique_archs']:.2f}")
        print(f"    Picks 1-5 S/A for emerging arch:    {agg['early_sa']:.2f}")
        print(f"    Picks 6+ S/A for committed arch:    {agg['late_sa']:.2f}")
        print(f"    Picks 6+ off-arch (C/F):            {agg['late_cf']:.2f}")
        print(f"    Convergence pick:                   {agg['convergence_pick']:.1f}")
        print(f"    Deck concentration:                 {agg['concentration']:.1%}")
        print(f"    Late SA stddev:                     {agg['late_sa_std']:.2f}")
        print(f"    Card overlap (run-to-run):          {agg['card_overlap']:.1%}")
        print(f"    SA distribution (picks 6+): {agg['sa_distribution']}")

    # ---- V3 Lane Locking baseline ----
    print("\n--- LANE LOCKING BASELINE (threshold 3/8, primary=2) ---")
    for strat in ["committed", "power", "signal"]:
        agg = run_simulation(
            n_drafts=N_DRAFTS, algorithm="lane_locking", strategy=strat,
            sym_dist=SYM_DIST,
        )
        print(f"\n  Strategy: {strat}")
        print(f"    Picks 1-5 unique archetypes w/ S/A: {agg['early_unique_archs']:.2f}")
        print(f"    Picks 1-5 S/A for emerging arch:    {agg['early_sa']:.2f}")
        print(f"    Picks 6+ S/A for committed arch:    {agg['late_sa']:.2f}")
        print(f"    Picks 6+ off-arch (C/F):            {agg['late_cf']:.2f}")
        print(f"    Convergence pick:                   {agg['convergence_pick']:.1f}")
        print(f"    Deck concentration:                 {agg['concentration']:.1%}")
        print(f"    Late SA stddev:                     {agg['late_sa_std']:.2f}")
        print(f"    Card overlap (run-to-run):          {agg['card_overlap']:.1%}")
        print(f"    SA distribution (picks 6+): {agg['sa_distribution']}")

    # ---- Per-archetype convergence ----
    print("\n--- PER-ARCHETYPE CONVERGENCE (Phantom) ---")
    conv_phantom = run_per_archetype_convergence(
        algorithm="phantom", n_drafts=200, num_phantoms=2, cards_per_phantom=1,
        sym_dist=SYM_DIST,
    )
    for name, pick in conv_phantom.items():
        print(f"  {name:25s}: pick {pick:.1f}")

    print("\n--- PER-ARCHETYPE CONVERGENCE (Lane Locking) ---")
    conv_ll = run_per_archetype_convergence(
        algorithm="lane_locking", n_drafts=200, sym_dist=SYM_DIST,
    )
    for name, pick in conv_ll.items():
        print(f"  {name:25s}: pick {pick:.1f}")

    # ---- Parameter sensitivity: phantom count ----
    print("\n--- PARAMETER SENSITIVITY: Phantom Count ---")
    for n_ph in [1, 2, 3, 4]:
        agg = run_simulation(
            n_drafts=500, algorithm="phantom", strategy="committed",
            num_phantoms=n_ph, cards_per_phantom=1, sym_dist=SYM_DIST,
            seed=100,
        )
        print(f"  {n_ph} phantoms: late_sa={agg['late_sa']:.2f}, "
              f"late_cf={agg['late_cf']:.2f}, "
              f"convergence={agg['convergence_pick']:.1f}, "
              f"stddev={agg['late_sa_std']:.2f}, "
              f"concentration={agg['concentration']:.1%}")

    # ---- Parameter sensitivity: cards per phantom ----
    print("\n--- PARAMETER SENSITIVITY: Cards Per Phantom (2 phantoms) ---")
    for cpp in [1, 2, 3]:
        agg = run_simulation(
            n_drafts=500, algorithm="phantom", strategy="committed",
            num_phantoms=2, cards_per_phantom=cpp, sym_dist=SYM_DIST,
            seed=200,
        )
        print(f"  {cpp} cards/phantom: late_sa={agg['late_sa']:.2f}, "
              f"late_cf={agg['late_cf']:.2f}, "
              f"convergence={agg['convergence_pick']:.1f}, "
              f"stddev={agg['late_sa_std']:.2f}, "
              f"concentration={agg['concentration']:.1%}")

    # ---- Parameter sensitivity: resonance overlap ----
    print("\n--- PARAMETER SENSITIVITY: Resonance Overlap ---")
    # Force 2 phantoms on same resonance vs distinct
    for label, force_overlap in [("distinct", False), ("overlap", True)]:
        rng = random.Random(300)
        metrics_list = []
        for _ in range(500):
            pool_seed = rng.randint(0, 10**9)
            draft_seed = rng.randint(0, 10**9)
            pool = build_card_pool(sym_dist=SYM_DIST, seed=pool_seed)
            draft_rng = random.Random(draft_seed)
            # Manually set phantom resonances
            all_res = list(Resonance)
            if force_overlap:
                r = draft_rng.choice(all_res)
                phantom_res = [r, r]
            else:
                phantom_res = draft_rng.sample(all_res, 2)
            # Run draft with custom phantom resonances
            pool_copy = list(pool)
            drafted = []
            target_idx = rng.randint(0, 7)
            target_name = ARCHETYPE_NAMES[target_idx]
            pick_records = []
            for pick_num in range(1, 31):
                pool_copy = phantom_remove(pool_copy, phantom_res, 1, draft_rng)
                pack = draw_pack_random(pool_copy, 4, draft_rng)
                if not pack:
                    break
                if pick_num >= 5:
                    sa_cards = [c for c in pack if card_is_sa(c, target_name)]
                    chosen = max(sa_cards, key=lambda c: c.power) if sa_cards else max(pack, key=lambda c: c.power)
                else:
                    chosen = max(pack, key=lambda c: c.power)
                sa_count = sum(1 for c in pack if card_is_sa(c, target_name))
                cf_count = sum(1 for c in pack if card_is_cf(c, target_name))
                pick_records.append({
                    "pick": pick_num, "pack_size": len(pack),
                    "sa_in_pack": sa_count, "cf_in_pack": cf_count,
                    "unique_archetypes_sa": 0, "chosen": chosen,
                    "pack": pack, "target": target_name,
                })
                pool_copy = [c for c in pool_copy if c.id != chosen.id]
                drafted.append(chosen)
            m = _compute_metrics(pick_records, drafted, target_name)
            metrics_list.append(m)
        avg_late_sa = statistics.mean([m["late_sa"] for m in metrics_list])
        avg_std = statistics.mean([m["late_sa_std"] for m in metrics_list])
        avg_conv = statistics.mean([m["convergence_pick"] for m in metrics_list])
        print(f"  {label}: late_sa={avg_late_sa:.2f}, stddev={avg_std:.2f}, "
              f"convergence={avg_conv:.1f}")

    # ---- Symbol distribution sweep ----
    print("\n--- SYMBOL DISTRIBUTION SWEEP ---")
    for dist_label, dist in [
        ("heavy-1sym (60/30/10)", (0.60, 0.30, 0.10)),
        ("baseline (35/45/20)",   (0.35, 0.45, 0.20)),
        ("heavy-2sym (20/60/20)", (0.20, 0.60, 0.20)),
        ("heavy-3sym (15/35/50)", (0.15, 0.35, 0.50)),
    ]:
        agg = run_simulation(
            n_drafts=500, algorithm="phantom", strategy="committed",
            num_phantoms=2, cards_per_phantom=1, sym_dist=dist,
            seed=400,
        )
        print(f"  {dist_label}: late_sa={agg['late_sa']:.2f}, "
              f"late_cf={agg['late_cf']:.2f}, "
              f"convergence={agg['convergence_pick']:.1f}, "
              f"stddev={agg['late_sa_std']:.2f}")

    # ---- Draft traces ----
    print("\n--- DRAFT TRACES ---")

    # Trace 1: Early committer (Warriors)
    t1 = run_draft_trace(
        algorithm="phantom", strategy="committed",
        num_phantoms=2, cards_per_phantom=1,
        target_archetype_idx=6, seed=1001,  # Warriors
    )
    print(format_trace(t1, "Early Committer (Warriors, commits pick 5)"))

    # Trace 2: Flexible player (power chaser)
    t2 = run_draft_trace(
        algorithm="phantom", strategy="power",
        num_phantoms=2, cards_per_phantom=1,
        target_archetype_idx=2, seed=1002,  # Storm nominal target
    )
    print(format_trace(t2, "Flexible Player (power chaser, never commits)"))

    # Trace 3: Signal reader
    t3 = run_draft_trace(
        algorithm="phantom", strategy="signal",
        num_phantoms=2, cards_per_phantom=1,
        seed=1003,
    )
    print(format_trace(t3, "Signal Reader (commits pick 6 to open archetype)"))

    print("\n" + "=" * 70)
    print("SIMULATION COMPLETE")
    print("=" * 70)


if __name__ == "__main__":
    main()
