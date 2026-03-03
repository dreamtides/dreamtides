"""
Resonance V4 — Agent 5 Simulation: Deck Echo Filter

Algorithm: To make each pack, draw 12 random cards, then keep each independently
with probability (2 + its weighted symbol overlap with your drafted deck) / 6,
and fill any remaining pack slots randomly from the rejects.

Weighted overlap: primary symbol matches count 1.5x, secondary/tertiary 1.0x.
"""

import random
import math
from collections import defaultdict
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

NUM_CARDS = 360
NUM_GENERIC = 36
NUM_ARCHETYPE_CARDS = NUM_CARDS - NUM_GENERIC  # 324
NUM_ARCHETYPES = 8
CARDS_PER_ARCHETYPE = NUM_ARCHETYPE_CARDS // NUM_ARCHETYPES  # 40 (remainder distributed)
PACK_SIZE = 4
NUM_PICKS = 30
NUM_DRAFTS = 1000
CANDIDATE_POOL_SIZE = 12

RESONANCES = ["Ember", "Stone", "Tide", "Zephyr"]

# Archetypes arranged on a circle, each with (primary, secondary) resonance
ARCHETYPES = [
    ("Flash/Tempo/Prison",   "Zephyr", "Ember"),
    ("Blink/Flicker",        "Ember",  "Zephyr"),
    ("Storm/Spellslinger",   "Ember",  "Stone"),
    ("Self-Discard",         "Stone",  "Ember"),
    ("Self-Mill/Reanimator", "Stone",  "Tide"),
    ("Sacrifice/Abandon",    "Tide",   "Stone"),
    ("Warriors/Midrange",    "Tide",   "Zephyr"),
    ("Ramp/Spirit Animals",  "Zephyr", "Tide"),
]

ARCHETYPE_NAMES = [a[0] for a in ARCHETYPES]


class Tier(Enum):
    S = 4
    A = 3
    B = 2
    C = 1
    F = 0


@dataclass
class SimCard:
    id: int
    symbols: list  # ordered list of resonance strings; [] = generic
    archetype: Optional[str]  # home archetype name, None for generic
    archetype_fitness: dict = field(default_factory=dict)  # archetype -> Tier
    power: float = 0.0


# ---------------------------------------------------------------------------
# Card Pool Construction
# ---------------------------------------------------------------------------

def circle_distance(i: int, j: int) -> int:
    """Minimum distance on the 8-archetype circle."""
    d = abs(i - j)
    return min(d, NUM_ARCHETYPES - d)


def assign_fitness(card: SimCard, home_idx: int):
    """Assign archetype fitness tiers based on circle distance."""
    for idx, name in enumerate(ARCHETYPE_NAMES):
        dist = circle_distance(home_idx, idx)
        if dist == 0:
            card.archetype_fitness[name] = Tier.S
        elif dist == 1:
            # Adjacent archetype — check if shares primary resonance
            home_primary = ARCHETYPES[home_idx][1]
            adj_primary = ARCHETYPES[idx][1]
            adj_secondary = ARCHETYPES[idx][2]
            if home_primary == adj_primary or home_primary == adj_secondary:
                card.archetype_fitness[name] = Tier.A
            else:
                card.archetype_fitness[name] = Tier.A  # adjacent always A
        elif dist == 2:
            card.archetype_fitness[name] = Tier.B
        elif dist == 3:
            card.archetype_fitness[name] = Tier.C
        else:
            card.archetype_fitness[name] = Tier.F


def generate_symbols(primary_res: str, secondary_res: str, num_symbols: int) -> list:
    """Generate an ordered symbol list for a card belonging to an archetype."""
    if num_symbols == 1:
        # 70% primary, 30% secondary
        return [primary_res] if random.random() < 0.7 else [secondary_res]
    elif num_symbols == 2:
        # primary is always first; second symbol is primary or secondary
        second = primary_res if random.random() < 0.3 else secondary_res
        return [primary_res, second]
    elif num_symbols == 3:
        # primary first, then mix of primary/secondary for remaining
        s2 = primary_res if random.random() < 0.4 else secondary_res
        s3 = secondary_res if random.random() < 0.7 else primary_res
        return [primary_res, s2, s3]
    return []


def build_card_pool(sym_dist: tuple = (0.30, 0.50, 0.20), seed: int = None) -> list:
    """
    Build 360 cards with the given symbol distribution.
    sym_dist: (frac_1sym, frac_2sym, frac_3sym) for non-generic cards.
    """
    if seed is not None:
        random.seed(seed)

    cards = []
    card_id = 0

    # Generic cards (36)
    for _ in range(NUM_GENERIC):
        c = SimCard(id=card_id, symbols=[], archetype=None, power=random.uniform(4, 8))
        for name in ARCHETYPE_NAMES:
            c.archetype_fitness[name] = Tier.B
        cards.append(c)
        card_id += 1

    # Archetype cards
    # Distribute 324 cards evenly: 40 per archetype with 4 leftover
    per_arch = [40] * 8
    leftover = NUM_ARCHETYPE_CARDS - 40 * 8
    for i in range(leftover):
        per_arch[i] += 1

    frac_1, frac_2, frac_3 = sym_dist
    for arch_idx, (arch_name, pri, sec) in enumerate(ARCHETYPES):
        n = per_arch[arch_idx]
        # Determine how many of each symbol count
        n1 = round(n * frac_1)
        n3 = round(n * frac_3)
        n2 = n - n1 - n3

        for _ in range(n1):
            syms = generate_symbols(pri, sec, 1)
            c = SimCard(id=card_id, symbols=syms, archetype=arch_name,
                        power=random.uniform(3, 9))
            assign_fitness(c, arch_idx)
            cards.append(c)
            card_id += 1

        for _ in range(n2):
            syms = generate_symbols(pri, sec, 2)
            c = SimCard(id=card_id, symbols=syms, archetype=arch_name,
                        power=random.uniform(3, 9))
            assign_fitness(c, arch_idx)
            cards.append(c)
            card_id += 1

        for _ in range(n3):
            syms = generate_symbols(pri, sec, 3)
            c = SimCard(id=card_id, symbols=syms, archetype=arch_name,
                        power=random.uniform(3, 9))
            assign_fitness(c, arch_idx)
            cards.append(c)
            card_id += 1

    random.shuffle(cards)
    return cards


# ---------------------------------------------------------------------------
# Resonance Symbol Counting (for player state)
# ---------------------------------------------------------------------------

def player_resonance_profile(drafted: list) -> dict:
    """
    Compute weighted resonance symbol counts for a player's drafted deck.
    Primary symbol counts as 2, secondary/tertiary as 1.
    Returns dict: resonance -> weighted count.
    """
    counts = defaultdict(float)
    for card in drafted:
        for i, sym in enumerate(card.symbols):
            if i == 0:
                counts[sym] += 2.0
            else:
                counts[sym] += 1.0
    return dict(counts)


def player_resonance_set_weighted(drafted: list) -> dict:
    """
    Return a dict mapping each resonance to its total weight in the player's
    drafted deck, distinguishing primary (2) from secondary/tertiary (1).
    Used by the echo scoring to weight matches.
    """
    return player_resonance_profile(drafted)


# ---------------------------------------------------------------------------
# Echo Score Computation
# ---------------------------------------------------------------------------

def echo_score(candidate: SimCard, drafted: list, primary_weight: float = 1.5) -> float:
    """
    Compute the echo score of a candidate card against the player's drafted deck.
    For each symbol on the candidate:
      - Check if that resonance appears in the drafted deck.
      - If it matches a resonance that is a PRIMARY symbol on any drafted card,
        weight it at primary_weight.
      - If it only matches secondary/tertiary, weight at 1.0.
    """
    if not drafted or not candidate.symbols:
        return 0.0

    # Collect which resonances appear as primary vs secondary in drafted deck
    primary_resonances = set()
    secondary_resonances = set()
    for card in drafted:
        for i, sym in enumerate(card.symbols):
            if i == 0:
                primary_resonances.add(sym)
            else:
                secondary_resonances.add(sym)

    score = 0.0
    for sym in candidate.symbols:
        if sym in primary_resonances:
            score += primary_weight
        elif sym in secondary_resonances:
            score += 1.0
    return score


# ---------------------------------------------------------------------------
# Deck Echo Filter — Pack Generation
# ---------------------------------------------------------------------------

def deck_echo_filter_pack(pool: list, drafted: list, candidate_n: int = 12,
                          base_num: float = 2.0, denom: float = 6.0,
                          primary_weight: float = 1.5) -> list:
    """
    Generate a pack using the Deck Echo Filter algorithm.
    1. Draw candidate_n cards uniformly from pool.
    2. For each candidate, compute echo_score.
    3. Each candidate survives independently with P = min((base_num + echo) / denom, 5/6).
    4. Take up to PACK_SIZE from survivors (random if > PACK_SIZE).
    5. Fill remaining from rejects (random).
    """
    if len(pool) <= PACK_SIZE:
        return list(pool)

    candidates = random.sample(pool, min(candidate_n, len(pool)))

    survivors = []
    rejects = []

    for card in candidates:
        es = echo_score(card, drafted, primary_weight)
        prob = min((base_num + es) / denom, 5.0 / 6.0)
        if random.random() < prob:
            survivors.append(card)
        else:
            rejects.append(card)

    # Build pack from survivors first
    pack = []
    if len(survivors) >= PACK_SIZE:
        pack = random.sample(survivors, PACK_SIZE)
    else:
        pack = list(survivors)
        # Fill remaining from rejects
        remaining = PACK_SIZE - len(pack)
        if remaining > 0 and rejects:
            fill = random.sample(rejects, min(remaining, len(rejects)))
            pack.extend(fill)

    # Edge case: still not enough (very unlikely)
    while len(pack) < PACK_SIZE and len(pool) > len(pack):
        extra = random.choice(pool)
        if extra not in pack:
            pack.append(extra)

    return pack[:PACK_SIZE]


# ---------------------------------------------------------------------------
# Lane Locking Baseline (V3)
# ---------------------------------------------------------------------------

def lane_locking_pack(pool: list, drafted: list, threshold_lock1: int = 3,
                      threshold_lock2: int = 8, locked_slots_per_lock: int = 1) -> list:
    """
    V3 Lane Locking baseline.
    Count player's resonance symbols (primary=2, sec/tert=1).
    For each resonance exceeding threshold_lock1, lock 1 pack slot to that resonance.
    For each exceeding threshold_lock2, lock a second slot.
    Locked slots show a random card with that resonance as primary.
    Remaining slots are random from pool.
    Max 4 locked slots total (though typically 2).
    """
    if len(pool) <= PACK_SIZE:
        return list(pool)

    profile = player_resonance_profile(drafted)

    locked = []  # list of resonances to fill
    for res in RESONANCES:
        count = profile.get(res, 0)
        if count >= threshold_lock2:
            locked.append(res)
            locked.append(res)
        elif count >= threshold_lock1:
            locked.append(res)

    locked = locked[:PACK_SIZE]  # cap at 4

    pack = []
    used_ids = set()

    # Fill locked slots
    for res in locked:
        candidates = [c for c in pool if c.symbols and c.symbols[0] == res and c.id not in used_ids]
        if candidates:
            chosen = random.choice(candidates)
            pack.append(chosen)
            used_ids.add(chosen.id)

    # Fill remaining slots randomly
    remaining_pool = [c for c in pool if c.id not in used_ids]
    remaining_slots = PACK_SIZE - len(pack)
    if remaining_slots > 0 and remaining_pool:
        fills = random.sample(remaining_pool, min(remaining_slots, len(remaining_pool)))
        pack.extend(fills)

    return pack[:PACK_SIZE]


# ---------------------------------------------------------------------------
# Player Strategies
# ---------------------------------------------------------------------------

def pick_archetype_committed(pack: list, drafted: list, target_arch: str) -> SimCard:
    """Pick the card with highest fitness for target archetype, then by power."""
    def score(c):
        fit = c.archetype_fitness.get(target_arch, Tier.F)
        return (fit.value, c.power)
    return max(pack, key=score)


def pick_power_chaser(pack: list, drafted: list, target_arch: str) -> SimCard:
    """Pick the highest power card regardless of archetype."""
    return max(pack, key=lambda c: c.power)


def determine_strongest_archetype(drafted: list) -> str:
    """Determine which archetype the drafted cards best support."""
    scores = defaultdict(float)
    for card in drafted:
        for arch, tier in card.archetype_fitness.items():
            scores[arch] += tier.value
    if not scores:
        return random.choice(ARCHETYPE_NAMES)
    return max(scores, key=scores.get)


def pick_signal_reader(pack: list, drafted: list, target_arch: str) -> SimCard:
    """
    For first 5 picks, pick highest power. Then identify strongest archetype
    from drafted cards and pick accordingly.
    """
    if len(drafted) < 5:
        return pick_power_chaser(pack, drafted, target_arch)
    inferred_arch = determine_strongest_archetype(drafted)
    return pick_archetype_committed(pack, drafted, inferred_arch)


# ---------------------------------------------------------------------------
# Metrics
# ---------------------------------------------------------------------------

def card_is_sa_for(card: SimCard, archetype: str) -> bool:
    """Check if a card is S or A tier for the given archetype."""
    tier = card.archetype_fitness.get(archetype, Tier.F)
    return tier.value >= Tier.A.value


def count_sa_in_pack(pack: list, archetype: str) -> int:
    return sum(1 for c in pack if card_is_sa_for(c, archetype))


def count_unique_archetypes_sa(pack: list) -> int:
    """Count how many distinct archetypes have at least one S/A card in the pack."""
    archs = set()
    for c in pack:
        for arch, tier in c.archetype_fitness.items():
            if tier.value >= Tier.A.value:
                archs.add(arch)
    return len(archs)


def count_off_archetype(pack: list, archetype: str) -> int:
    """Count cards that are C or F tier for the given archetype."""
    return sum(1 for c in pack if c.archetype_fitness.get(archetype, Tier.F).value <= Tier.C.value)


# ---------------------------------------------------------------------------
# Draft Simulation
# ---------------------------------------------------------------------------

def run_single_draft(pool: list, strategy: str, target_arch: str,
                     pack_fn, pack_fn_kwargs: dict = None) -> dict:
    """
    Run a single 30-pick draft. Returns metrics dict.
    """
    if pack_fn_kwargs is None:
        pack_fn_kwargs = {}

    drafted = []
    pick_fn = {
        "committed": pick_archetype_committed,
        "power": pick_power_chaser,
        "signal": pick_signal_reader,
    }[strategy]

    # For signal reader, target_arch emerges; for others it's fixed
    effective_target = target_arch

    metrics = {
        "early_unique_archetypes": [],     # picks 1-5
        "early_sa_for_target": [],         # picks 1-5
        "late_sa_for_target": [],          # picks 6+
        "late_off_archetype": [],          # picks 6+
        "all_packs": [],                   # all packs for trace
        "all_picks": [],                   # all picks for trace
        "deck_cards": [],                  # final drafted cards
    }

    for pick_num in range(1, NUM_PICKS + 1):
        pack = pack_fn(pool, drafted, **pack_fn_kwargs)

        # For signal reader, update target after pick 5
        if strategy == "signal" and pick_num == 6:
            effective_target = determine_strongest_archetype(drafted)

        sa_count = count_sa_in_pack(pack, effective_target)
        unique_archs = count_unique_archetypes_sa(pack)
        off_arch = count_off_archetype(pack, effective_target)

        if pick_num <= 5:
            metrics["early_unique_archetypes"].append(unique_archs)
            metrics["early_sa_for_target"].append(sa_count)
        else:
            metrics["late_sa_for_target"].append(sa_count)
            metrics["late_off_archetype"].append(off_arch)

        metrics["all_packs"].append((pick_num, pack, sa_count, effective_target))

        chosen = pick_fn(pack, drafted, effective_target)
        drafted.append(chosen)
        metrics["all_picks"].append((pick_num, chosen))

    metrics["deck_cards"] = drafted
    metrics["final_target"] = effective_target
    return metrics


def compute_deck_concentration(drafted: list, target_arch: str) -> float:
    """Fraction of drafted cards that are S/A for target archetype."""
    if not drafted:
        return 0.0
    return sum(1 for c in drafted if card_is_sa_for(c, target_arch)) / len(drafted)


def compute_convergence_pick(sa_per_pack_by_pick: dict) -> int:
    """
    Find the earliest pick from which the player 'regularly' sees 2+ S/A cards.
    'Regularly' = rolling average of 3 consecutive picks is >= 2.0.
    """
    picks_sorted = sorted(sa_per_pack_by_pick.keys())
    for i in range(len(picks_sorted) - 2):
        p1, p2, p3 = picks_sorted[i], picks_sorted[i+1], picks_sorted[i+2]
        avg = (sa_per_pack_by_pick[p1] + sa_per_pack_by_pick[p2] + sa_per_pack_by_pick[p3]) / 3
        if avg >= 2.0:
            return p1
    return 30  # never converged


# ---------------------------------------------------------------------------
# Main Simulation
# ---------------------------------------------------------------------------

def run_simulation(pack_fn, pack_fn_kwargs: dict, label: str,
                   num_drafts: int = NUM_DRAFTS, pool: list = None,
                   seed: int = 42):
    """
    Run full simulation across all strategies and all archetypes.
    Returns a results dict.
    """
    if pool is None:
        pool = build_card_pool(seed=seed)

    results = {
        "label": label,
        "early_unique_archs": [],
        "early_sa": [],
        "late_sa": [],
        "late_off": [],
        "deck_conc": [],
        "convergence_picks": defaultdict(list),  # per archetype
        "late_sa_per_pack_dist": [],  # for variance
        "card_overlap_runs": [],
        "archetype_frequency": defaultdict(int),
        "traces": [],
    }

    random.seed(seed)

    for draft_i in range(num_drafts):
        # Rotate through archetypes evenly for committed player
        arch_idx = draft_i % NUM_ARCHETYPES
        target_arch = ARCHETYPE_NAMES[arch_idx]

        # Choose strategy: 60% committed, 20% power, 20% signal
        r = random.random()
        if r < 0.6:
            strategy = "committed"
        elif r < 0.8:
            strategy = "power"
        else:
            strategy = "signal"

        m = run_single_draft(pool, strategy, target_arch, pack_fn, pack_fn_kwargs)

        final_target = m.get("final_target", target_arch)

        # Early metrics
        if m["early_unique_archetypes"]:
            results["early_unique_archs"].extend(m["early_unique_archetypes"])
        if m["early_sa_for_target"]:
            results["early_sa"].extend(m["early_sa_for_target"])

        # Late metrics
        if m["late_sa_for_target"]:
            results["late_sa"].extend(m["late_sa_for_target"])
            results["late_sa_per_pack_dist"].extend(m["late_sa_for_target"])
        if m["late_off_archetype"]:
            results["late_off"].extend(m["late_off_archetype"])

        # Deck concentration (committed only)
        if strategy == "committed":
            conc = compute_deck_concentration(m["deck_cards"], final_target)
            results["deck_conc"].append(conc)

            # Per-archetype convergence
            sa_by_pick = {}
            for pick_num, pack, sa, tgt in m["all_packs"]:
                sa_by_pick[pick_num] = sa
            conv_pick = compute_convergence_pick(sa_by_pick)
            results["convergence_picks"][final_target].append(conv_pick)

            results["archetype_frequency"][final_target] += 1

        # Store trace for first 3 committed drafts
        if strategy == "committed" and len(results["traces"]) < 3:
            results["traces"].append(m)

    return results


def compute_card_overlap(pool, pack_fn, pack_fn_kwargs, target_arch, seed=42, n_runs=50):
    """Run n_runs drafts with same target archetype, compute average pairwise overlap."""
    decks = []
    for i in range(n_runs):
        random.seed(seed + i * 1000)
        m = run_single_draft(pool, "committed", target_arch, pack_fn, pack_fn_kwargs)
        deck_ids = set(c.id for c in m["deck_cards"])
        decks.append(deck_ids)

    overlaps = []
    for i in range(len(decks)):
        for j in range(i+1, len(decks)):
            inter = len(decks[i] & decks[j])
            union = len(decks[i] | decks[j])
            if union > 0:
                overlaps.append(inter / union)
    return sum(overlaps) / len(overlaps) if overlaps else 0.0


def print_results(results: dict):
    label = results["label"]
    print(f"\n{'='*60}")
    print(f"  {label}")
    print(f"{'='*60}")

    avg_early_unique = sum(results["early_unique_archs"]) / max(len(results["early_unique_archs"]), 1)
    avg_early_sa = sum(results["early_sa"]) / max(len(results["early_sa"]), 1)
    avg_late_sa = sum(results["late_sa"]) / max(len(results["late_sa"]), 1)
    avg_late_off = sum(results["late_off"]) / max(len(results["late_off"]), 1)
    avg_deck_conc = sum(results["deck_conc"]) / max(len(results["deck_conc"]), 1)

    # Variance
    late_sa_vals = results["late_sa_per_pack_dist"]
    if late_sa_vals:
        mean_sa = sum(late_sa_vals) / len(late_sa_vals)
        var_sa = sum((x - mean_sa)**2 for x in late_sa_vals) / len(late_sa_vals)
        stddev_sa = math.sqrt(var_sa)
    else:
        mean_sa = stddev_sa = 0.0

    # Distribution of S/A per pack
    dist = defaultdict(int)
    for v in late_sa_vals:
        dist[v] += 1

    print(f"\nPicks 1-5: Unique archetypes w/ S/A per pack: {avg_early_unique:.2f} (target >= 3)")
    print(f"Picks 1-5: S/A for target per pack:           {avg_early_sa:.2f} (target <= 2)")
    print(f"Picks 6+:  S/A for target per pack:           {avg_late_sa:.2f} (target >= 2)")
    print(f"Picks 6+:  Off-archetype (C/F) per pack:      {avg_late_off:.2f} (target >= 0.5)")
    print(f"Deck concentration (committed):               {avg_deck_conc:.2%} (target 60-90%)")
    print(f"S/A stddev (picks 6+):                        {stddev_sa:.3f} (target >= 0.8)")

    print(f"\nS/A per pack distribution (picks 6+):")
    total = max(len(late_sa_vals), 1)
    for k in sorted(dist.keys()):
        print(f"  {k} cards: {dist[k]} ({dist[k]/total:.1%})")

    # Per-archetype convergence
    print(f"\nPer-archetype convergence (avg pick to regularly see 2+ S/A):")
    for arch in ARCHETYPE_NAMES:
        vals = results["convergence_picks"].get(arch, [])
        if vals:
            avg_conv = sum(vals) / len(vals)
            print(f"  {arch:30s}: pick {avg_conv:.1f}")
        else:
            print(f"  {arch:30s}: no data")

    # Archetype frequency
    total_committed = sum(results["archetype_frequency"].values())
    if total_committed > 0:
        print(f"\nArchetype frequency among committed players:")
        for arch in ARCHETYPE_NAMES:
            freq = results["archetype_frequency"].get(arch, 0)
            pct = freq / total_committed
            print(f"  {arch:30s}: {pct:.1%}")

    return {
        "avg_early_unique": avg_early_unique,
        "avg_early_sa": avg_early_sa,
        "avg_late_sa": avg_late_sa,
        "avg_late_off": avg_late_off,
        "avg_deck_conc": avg_deck_conc,
        "stddev_sa": stddev_sa,
        "convergence_picks": results["convergence_picks"],
    }


# ---------------------------------------------------------------------------
# Draft Traces
# ---------------------------------------------------------------------------

def print_trace(trace: dict, label: str):
    print(f"\n--- Draft Trace: {label} ---")
    target = trace.get("final_target", "unknown")
    print(f"Target archetype: {target}")
    for pick_num, pack, sa_count, tgt in trace["all_packs"][:15]:  # First 15 picks
        pick_card = trace["all_picks"][pick_num - 1][1]
        pack_desc = []
        for c in pack:
            tier = c.archetype_fitness.get(tgt, Tier.F)
            syms = "/".join(c.symbols) if c.symbols else "generic"
            marker = "*" if tier.value >= Tier.A.value else " "
            picked = "<-PICKED" if c.id == pick_card.id else ""
            pack_desc.append(f"  {marker}[{syms}] {c.archetype or 'generic'} "
                             f"(pow={c.power:.1f}, {tier.name}-tier){picked}")
        print(f"Pick {pick_num}: S/A={sa_count} for {tgt}")
        for d in pack_desc:
            print(d)


# ---------------------------------------------------------------------------
# Parameter Sensitivity Sweeps
# ---------------------------------------------------------------------------

def run_sensitivity_sweep(pool):
    print("\n" + "=" * 60)
    print("  PARAMETER SENSITIVITY SWEEPS")
    print("=" * 60)

    # Sweep 1: Candidate pool size
    print("\n--- Sweep 1: Candidate Pool Size ---")
    for cand_size in [8, 10, 12, 14, 16]:
        random.seed(42)
        r = run_simulation(
            deck_echo_filter_pack,
            {"candidate_n": cand_size, "base_num": 2.0, "denom": 6.0, "primary_weight": 1.5},
            f"Deck Echo (candidates={cand_size})",
            num_drafts=500, pool=pool, seed=42
        )
        s = print_results(r)
        print()

    # Sweep 2: Acceptance formula (survival formula)
    print("\n--- Sweep 2: Acceptance Formula ---")
    formulas = [
        (2.0, 6.0, "(2+echo)/6"),
        (1.0, 4.0, "(1+echo)/4"),
        (2.0, 5.0, "(2+echo)/5"),
        (1.5, 5.0, "(1.5+echo)/5"),
        (3.0, 7.0, "(3+echo)/7"),
    ]
    for base, denom, name in formulas:
        random.seed(42)
        r = run_simulation(
            deck_echo_filter_pack,
            {"candidate_n": 12, "base_num": base, "denom": denom, "primary_weight": 1.5},
            f"Deck Echo ({name})",
            num_drafts=500, pool=pool, seed=42
        )
        s = print_results(r)
        print()

    # Sweep 3: Primary weight
    print("\n--- Sweep 3: Primary Symbol Weight ---")
    for pw in [1.0, 1.25, 1.5, 2.0]:
        random.seed(42)
        r = run_simulation(
            deck_echo_filter_pack,
            {"candidate_n": 12, "base_num": 2.0, "denom": 6.0, "primary_weight": pw},
            f"Deck Echo (primary_weight={pw})",
            num_drafts=500, pool=pool, seed=42
        )
        s = print_results(r)
        print()


def run_symbol_dist_sweep():
    print("\n" + "=" * 60)
    print("  SYMBOL DISTRIBUTION SWEEP")
    print("=" * 60)

    distributions = [
        ((0.60, 0.30, 0.10), "Heavy 1-sym (60/30/10)"),
        ((0.30, 0.50, 0.20), "Balanced (30/50/20) [default]"),
        ((0.10, 0.60, 0.30), "Heavy 2-sym (10/60/30)"),
        ((0.10, 0.30, 0.60), "Heavy 3-sym (10/30/60)"),
        ((0.20, 0.50, 0.30), "More 3-sym (20/50/30)"),
    ]

    for dist, name in distributions:
        pool = build_card_pool(sym_dist=dist, seed=42)
        random.seed(42)
        r = run_simulation(
            deck_echo_filter_pack,
            {"candidate_n": 12, "base_num": 2.0, "denom": 6.0, "primary_weight": 1.5},
            f"Deck Echo — {name}",
            num_drafts=500, pool=pool, seed=42
        )
        print_results(r)
        print()


# ---------------------------------------------------------------------------
# Progressive Denominator Variant
# ---------------------------------------------------------------------------

def deck_echo_filter_progressive_pack(pool, drafted, candidate_n=12,
                                       primary_weight=1.5):
    """Variant with progressive denominator: 6 for picks 1-10, 5 for 11-20, 4 for 21-30."""
    pick_num = len(drafted) + 1
    if pick_num <= 10:
        denom = 6.0
    elif pick_num <= 20:
        denom = 5.0
    else:
        denom = 4.0
    return deck_echo_filter_pack(pool, drafted, candidate_n=candidate_n,
                                  base_num=2.0, denom=denom,
                                  primary_weight=primary_weight)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    print("Building card pool (30/50/20 distribution)...")
    pool = build_card_pool(sym_dist=(0.30, 0.50, 0.20), seed=42)

    # Verify pool
    generic = sum(1 for c in pool if not c.symbols)
    one_sym = sum(1 for c in pool if len(c.symbols) == 1)
    two_sym = sum(1 for c in pool if len(c.symbols) == 2)
    three_sym = sum(1 for c in pool if len(c.symbols) == 3)
    print(f"Pool: {len(pool)} cards — {generic} generic, {one_sym} 1-sym, "
          f"{two_sym} 2-sym, {three_sym} 3-sym")

    # -----------------------------------------------------------------------
    # Main Algorithm: Deck Echo Filter (refined)
    # -----------------------------------------------------------------------
    print("\n" + "#" * 60)
    print("  MAIN ALGORITHM: DECK ECHO FILTER (REFINED)")
    print("#" * 60)

    echo_results = run_simulation(
        deck_echo_filter_pack,
        {"candidate_n": 12, "base_num": 2.0, "denom": 6.0, "primary_weight": 1.5},
        "Deck Echo Filter (12 candidates, (2+echo)/6, pw=1.5)",
        num_drafts=NUM_DRAFTS, pool=pool, seed=42
    )
    echo_stats = print_results(echo_results)

    # Card overlap
    print("\nComputing run-to-run card overlap...")
    overlap = compute_card_overlap(pool, deck_echo_filter_pack,
                                   {"candidate_n": 12, "base_num": 2.0, "denom": 6.0,
                                    "primary_weight": 1.5},
                                   "Warriors/Midrange", seed=42)
    print(f"Card overlap (same archetype, Jaccard): {overlap:.2%}")

    # Draft traces
    if echo_results["traces"]:
        print_trace(echo_results["traces"][0], "Early Committer (Committed)")

    # Run a specific signal-reader trace
    random.seed(999)
    signal_trace = run_single_draft(pool, "signal", "Warriors/Midrange",
                                    deck_echo_filter_pack,
                                    {"candidate_n": 12, "base_num": 2.0,
                                     "denom": 6.0, "primary_weight": 1.5})
    print_trace(signal_trace, "Signal Reader")

    # Run a flexible player trace
    random.seed(777)
    flex_trace = run_single_draft(pool, "power", "Blink/Flicker",
                                  deck_echo_filter_pack,
                                  {"candidate_n": 12, "base_num": 2.0,
                                   "denom": 6.0, "primary_weight": 1.5})
    print_trace(flex_trace, "Flexible Player (Power Chaser)")

    # -----------------------------------------------------------------------
    # Progressive Denominator Variant
    # -----------------------------------------------------------------------
    print("\n" + "#" * 60)
    print("  VARIANT: PROGRESSIVE DENOMINATOR")
    print("#" * 60)

    prog_results = run_simulation(
        deck_echo_filter_progressive_pack,
        {"candidate_n": 12, "primary_weight": 1.5},
        "Deck Echo Filter — Progressive Denominator (6/5/4)",
        num_drafts=NUM_DRAFTS, pool=pool, seed=42
    )
    prog_stats = print_results(prog_results)

    # -----------------------------------------------------------------------
    # V3 Lane Locking Baseline
    # -----------------------------------------------------------------------
    print("\n" + "#" * 60)
    print("  V3 BASELINE: LANE LOCKING (3/8, primary=2)")
    print("#" * 60)

    lane_results = run_simulation(
        lane_locking_pack,
        {"threshold_lock1": 3, "threshold_lock2": 8, "locked_slots_per_lock": 1},
        "V3 Lane Locking (threshold 3/8)",
        num_drafts=NUM_DRAFTS, pool=pool, seed=42
    )
    lane_stats = print_results(lane_results)

    lane_overlap = compute_card_overlap(pool, lane_locking_pack,
                                        {"threshold_lock1": 3, "threshold_lock2": 8,
                                         "locked_slots_per_lock": 1},
                                        "Warriors/Midrange", seed=42)
    print(f"Card overlap (Lane Locking, Jaccard): {lane_overlap:.2%}")

    # -----------------------------------------------------------------------
    # Parameter Sensitivity Sweeps
    # -----------------------------------------------------------------------
    run_sensitivity_sweep(pool)
    run_symbol_dist_sweep()

    # -----------------------------------------------------------------------
    # Summary Comparison Table
    # -----------------------------------------------------------------------
    print("\n" + "=" * 60)
    print("  SUMMARY COMPARISON: DECK ECHO vs LANE LOCKING")
    print("=" * 60)
    print(f"{'Metric':<45} {'Deck Echo':>12} {'Lane Lock':>12} {'Target':>12}")
    print("-" * 81)
    print(f"{'Picks 1-5: unique archs w/ S/A':<45} {echo_stats['avg_early_unique']:>12.2f} {lane_stats['avg_early_unique']:>12.2f} {'>= 3':>12}")
    print(f"{'Picks 1-5: S/A for target':<45} {echo_stats['avg_early_sa']:>12.2f} {lane_stats['avg_early_sa']:>12.2f} {'<= 2':>12}")
    print(f"{'Picks 6+: S/A for target':<45} {echo_stats['avg_late_sa']:>12.2f} {lane_stats['avg_late_sa']:>12.2f} {'>= 2':>12}")
    print(f"{'Picks 6+: off-archetype (C/F)':<45} {echo_stats['avg_late_off']:>12.2f} {lane_stats['avg_late_off']:>12.2f} {'>= 0.5':>12}")
    print(f"{'Deck concentration':<45} {echo_stats['avg_deck_conc']:>11.1%} {lane_stats['avg_deck_conc']:>11.1%} {'60-90%':>12}")
    print(f"{'S/A stddev (picks 6+)':<45} {echo_stats['stddev_sa']:>12.3f} {lane_stats['stddev_sa']:>12.3f} {'>= 0.8':>12}")
    print(f"{'Card overlap (Jaccard)':<45} {overlap:>11.1%} {lane_overlap:>11.1%} {'< 40%':>12}")


if __name__ == "__main__":
    main()
