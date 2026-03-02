#!/usr/bin/env python3
"""
Simulation for Agent 2: Balanced Pack with Majority Bonus
(Corrected: all evaluation metrics at ARCHETYPE level, not resonance level)

Algorithm: "Each pack has one card per resonance type, but if you have a clear
majority resonance (strictly more weighted symbols than any other, counting
primary=2), it replaces one random non-majority slot, giving you 2 of your
majority resonance."

CRITICAL DISTINCTION:
- A resonance (e.g., Tide) is shared by multiple archetypes
  (Warriors=Tide/Zephyr, Sacrifice=Tide/Stone)
- An archetype (e.g., Warriors) is specific -- a card is only a good Warriors
  card if it has S or A tier fitness for Warriors
- Measuring "2+ cards matching your resonance" is MUCH easier than
  "2+ cards from your archetype"
"""

import random
from dataclasses import dataclass
from enum import Enum
from collections import defaultdict, Counter

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

NUM_CARDS = 360
NUM_GENERIC = 36
NUM_ARCHETYPE_CARDS = NUM_CARDS - NUM_GENERIC  # 324
NUM_ARCHETYPES = 8
PACK_SIZE = 4
NUM_PICKS = 30
NUM_SIMULATIONS = 1000

RESONANCES = ["Ember", "Stone", "Tide", "Zephyr"]

# 8 archetypes on a circle: (name, primary_resonance, secondary_resonance)
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


def get_adjacent(idx):
    """Return indices of adjacent archetypes on the circle."""
    return [(idx - 1) % NUM_ARCHETYPES, (idx + 1) % NUM_ARCHETYPES]


# ---------------------------------------------------------------------------
# Data Classes
# ---------------------------------------------------------------------------

class Rarity(Enum):
    COMMON = "common"
    UNCOMMON = "uncommon"
    RARE = "rare"
    LEGENDARY = "legendary"


@dataclass
class SimCard:
    id: int
    symbols: list          # ordered list of resonance strings, 0-3 elements
    archetype: str         # home archetype name (or "Generic")
    archetype_idx: int     # index into ARCHETYPES (-1 for generic)
    archetype_fitness: dict  # archetype_name -> tier (S/A/B/C/F)
    rarity: Rarity
    power: float           # raw card strength 0-10


# ---------------------------------------------------------------------------
# Fitness Helpers
# ---------------------------------------------------------------------------

TIER_SCORES = {"S": 5, "A": 4, "B": 3, "C": 2, "F": 1}


def card_tier(card, arch_name):
    """Return tier string for a card in an archetype."""
    return card.archetype_fitness.get(arch_name, "F")


def card_fitness_score(card, arch_name):
    """Return numeric fitness score for a card in an archetype."""
    return TIER_SCORES.get(card_tier(card, arch_name), 1)


def is_sa_tier(card, arch_name):
    """Card is S or A tier for the given archetype."""
    return card_tier(card, arch_name) in ("S", "A")


def is_cf_tier(card, arch_name):
    """Card is C or F tier for the given archetype."""
    return card_tier(card, arch_name) in ("C", "F")


# ---------------------------------------------------------------------------
# Card Pool Construction
# ---------------------------------------------------------------------------

def compute_fitness(arch_idx):
    """
    Compute fitness tiers for a card belonging to archetype at arch_idx.

    - S-tier: home archetype
    - A-tier: adjacent archetype sharing primary resonance
    - B-tier: archetypes sharing secondary resonance (non-adjacent included)
    - C/F-tier: distant archetypes
    - Generic: B-tier in all
    """
    _, primary, secondary = ARCHETYPES[arch_idx]
    adj = get_adjacent(arch_idx)
    fitness = {}

    for other_idx, (other_name, other_pri, other_sec) in enumerate(ARCHETYPES):
        if other_idx == arch_idx:
            fitness[other_name] = "S"
        elif other_idx in adj:
            # Adjacent archetype: A-tier if shares primary, B-tier otherwise
            if other_pri == primary or other_sec == primary:
                fitness[other_name] = "A"
            else:
                fitness[other_name] = "B"
        else:
            # Non-adjacent: B if shares secondary resonance, C otherwise
            shares_secondary = (other_pri == secondary or other_sec == secondary)
            shares_primary = (other_pri == primary or other_sec == primary)
            if shares_secondary or shares_primary:
                fitness[other_name] = "B"
            else:
                fitness[other_name] = "C"

    return fitness


def build_card_pool(symbol_dist=(0.30, 0.55, 0.15)):
    """
    Build 360 cards:
    - 36 generic (0 symbols, B-tier in all archetypes)
    - 324 archetype cards distributed across 8 archetypes (~40 each)

    symbol_dist: (pct_1sym, pct_2sym, pct_3sym) among non-generic cards
    """
    cards = []
    card_id = 0

    # Generic cards
    generic_fitness = {name: "B" for name, _, _ in ARCHETYPES}
    for _ in range(NUM_GENERIC):
        cards.append(SimCard(
            id=card_id,
            symbols=[],
            archetype="Generic",
            archetype_idx=-1,
            archetype_fitness=dict(generic_fitness),
            rarity=random.choice([Rarity.COMMON, Rarity.UNCOMMON, Rarity.RARE]),
            power=round(random.uniform(4.0, 7.0), 1),
        ))
        card_id += 1

    # Distribute archetype cards
    base_per_arch = NUM_ARCHETYPE_CARDS // NUM_ARCHETYPES  # 40
    remainder = NUM_ARCHETYPE_CARDS % NUM_ARCHETYPES        # 4
    arch_counts = [base_per_arch] * NUM_ARCHETYPES
    for i in range(remainder):
        arch_counts[i] += 1

    pct_1, pct_2, pct_3 = symbol_dist

    for arch_idx, (arch_name, primary, secondary) in enumerate(ARCHETYPES):
        count = arch_counts[arch_idx]
        n1 = round(count * pct_1)
        n3 = round(count * pct_3)
        n2 = count - n1 - n3

        fitness = compute_fitness(arch_idx)

        for i in range(count):
            if i < n1:
                num_symbols = 1
            elif i < n1 + n2:
                num_symbols = 2
            else:
                num_symbols = 3

            # Build symbol list
            if num_symbols == 1:
                symbols = [primary] if random.random() < 0.75 else [secondary]
            elif num_symbols == 2:
                r = random.random()
                if r < 0.60:
                    symbols = [primary, secondary]
                elif r < 0.80:
                    symbols = [primary, primary]
                elif r < 0.95:
                    symbols = [secondary, primary]
                else:
                    symbols = [secondary, secondary]
            else:
                r = random.random()
                if r < 0.50:
                    symbols = [primary, primary, secondary]
                elif r < 0.80:
                    symbols = [primary, secondary, primary]
                else:
                    symbols = [primary, secondary, secondary]

            r_val = random.random()
            if r_val < 0.50:
                rarity = Rarity.COMMON
            elif r_val < 0.80:
                rarity = Rarity.UNCOMMON
            elif r_val < 0.95:
                rarity = Rarity.RARE
            else:
                rarity = Rarity.LEGENDARY

            cards.append(SimCard(
                id=card_id,
                symbols=symbols,
                archetype=arch_name,
                archetype_idx=arch_idx,
                archetype_fitness=dict(fitness),
                rarity=rarity,
                power=round(random.uniform(3.0, 9.0), 1),
            ))
            card_id += 1

    random.shuffle(cards)
    return cards


# ---------------------------------------------------------------------------
# Symbol Counting (used by the DRAFT ALGORITHM -- visible to player)
# ---------------------------------------------------------------------------

def count_weighted_symbols(drafted_cards):
    """
    Count weighted symbols across all drafted cards.
    Primary (first) = 2, secondary (second) = 1, tertiary (third) = 1.
    Returns dict: resonance -> weighted count.
    """
    counts = defaultdict(int)
    for card in drafted_cards:
        for i, sym in enumerate(card.symbols):
            counts[sym] += 2 if i == 0 else 1
    return counts


def get_majority_resonance(symbol_counts, threshold=0):
    """
    Return the majority resonance if one has strictly more weighted symbols
    than any other, by at least 'threshold'.
    Returns resonance string or None.
    """
    if not symbol_counts:
        return None
    sorted_res = sorted(symbol_counts.items(), key=lambda x: -x[1])
    if not sorted_res:
        return None
    top_res, top_count = sorted_res[0]
    second_count = sorted_res[1][1] if len(sorted_res) > 1 else 0
    if top_count > second_count + threshold:
        return top_res
    return None


# ---------------------------------------------------------------------------
# Pack Generation: Balanced Pack with Majority Bonus
# (ALGORITHM: uses only visible card properties -- resonance symbols)
# ---------------------------------------------------------------------------

def generate_pack(card_pool, drafted_cards, majority_threshold=0,
                  variant=None, used_ids=None):
    """
    Generate a pack of 4 cards.

    Base: one card per resonance type (Ember, Stone, Tide, Zephyr).
    If player has a clear majority resonance, it replaces one random
    non-majority slot, giving 2 of the majority resonance.

    variant=None: standard algorithm
    variant="A": dual override -- if both top-2 resonances have >=5,
                 pack is 2/2/0/0
    variant="B": threshold gate -- override only when top resonance >= 5
    """
    if used_ids is None:
        used_ids = set()

    symbol_counts = count_weighted_symbols(drafted_cards)

    # Determine pack resonance slots
    if variant == "A":
        sorted_res = sorted(symbol_counts.items(), key=lambda x: -x[1])
        if (len(sorted_res) >= 2
                and sorted_res[0][1] >= 5
                and sorted_res[1][1] >= 5):
            top1, top2 = sorted_res[0][0], sorted_res[1][0]
            slots = [top1, top1, top2, top2]
        else:
            majority = get_majority_resonance(symbol_counts, threshold=majority_threshold)
            if majority:
                other_res = [r for r in RESONANCES if r != majority]
                replaced = random.choice(other_res)
                slots = [majority, majority] + [r for r in other_res if r != replaced]
            else:
                slots = list(RESONANCES)
    elif variant == "B":
        sorted_res = sorted(symbol_counts.items(), key=lambda x: -x[1])
        if sorted_res and sorted_res[0][1] >= 5:
            majority = get_majority_resonance(symbol_counts, threshold=majority_threshold)
        else:
            majority = None
        if majority:
            other_res = [r for r in RESONANCES if r != majority]
            replaced = random.choice(other_res)
            slots = [majority, majority] + [r for r in other_res if r != replaced]
        else:
            slots = list(RESONANCES)
    else:
        majority = get_majority_resonance(symbol_counts, threshold=majority_threshold)
        if majority:
            other_res = [r for r in RESONANCES if r != majority]
            replaced = random.choice(other_res)
            slots = [majority, majority] + [r for r in other_res if r != replaced]
        else:
            slots = list(RESONANCES)

    # For each slot, pick a random card matching that resonance
    pack = []
    used_in_pack = set()

    for slot_resonance in slots:
        candidates = [
            c for c in card_pool
            if c.id not in used_ids
            and c.id not in used_in_pack
            and len(c.symbols) > 0
            and c.symbols[0] == slot_resonance
        ]
        if not candidates:
            candidates = [
                c for c in card_pool
                if c.id not in used_ids
                and c.id not in used_in_pack
                and slot_resonance in c.symbols
            ]
        if not candidates:
            candidates = [
                c for c in card_pool
                if c.id not in used_ids
                and c.id not in used_in_pack
            ]
        if candidates:
            chosen = random.choice(candidates)
            pack.append(chosen)
            used_in_pack.add(chosen.id)

    return pack


# ---------------------------------------------------------------------------
# Archetype-Level Pack Evaluation (EVALUATION ONLY -- invisible to algorithm)
# ---------------------------------------------------------------------------

def count_sa_in_pack(pack, arch_name):
    """Count cards in pack that are S or A tier for the given archetype."""
    return sum(1 for c in pack if is_sa_tier(c, arch_name))


def count_cf_in_pack(pack, arch_name):
    """Count cards in pack that are C or F tier for the given archetype."""
    return sum(1 for c in pack if is_cf_tier(c, arch_name))


def unique_archetypes_with_sa(pack):
    """
    Count how many distinct archetypes have at least one S/A-tier card in pack.
    """
    archs_with_sa = set()
    for card in pack:
        for arch_name in ARCHETYPE_NAMES:
            if is_sa_tier(card, arch_name):
                archs_with_sa.add(arch_name)
    return len(archs_with_sa)


# ---------------------------------------------------------------------------
# Player Strategies (target ARCHETYPES, evaluate by fitness tier)
# ---------------------------------------------------------------------------

def strategy_archetype_committed(pack, drafted, pick_num, target_arch=None):
    """
    Picks by power through picks 1-5.
    At pick 6, commits to the archetype with highest accumulated S/A count.
    Then always picks the card with highest fitness for target archetype.
    """
    if target_arch is None and pick_num <= 5:
        best = max(pack, key=lambda c: c.power)
        return best, None

    if target_arch is None and pick_num == 6:
        # Commit: find archetype with most S/A-tier drafted cards
        arch_sa = defaultdict(int)
        for card in drafted:
            for aname in ARCHETYPE_NAMES:
                if is_sa_tier(card, aname):
                    arch_sa[aname] += 1
        if arch_sa:
            target_arch = max(arch_sa, key=arch_sa.get)
        else:
            target_arch = ARCHETYPES[0][0]

    if target_arch:
        best = max(pack, key=lambda c: card_fitness_score(c, target_arch))
        return best, target_arch

    best = max(pack, key=lambda c: c.power)
    return best, target_arch


def strategy_power_chaser(pack, drafted, pick_num, target_arch=None):
    """Picks highest raw power regardless of archetype."""
    best = max(pack, key=lambda c: c.power)
    return best, target_arch


def strategy_signal_reader(pack, drafted, pick_num, target_arch=None):
    """
    Reads which archetypes are over-represented in packs.
    Through pick 5, tracks which archetypes appear most as S/A-tier cards.
    At pick 6, commits to the archetype most seen.
    """
    if pick_num <= 5:
        # Track archetypes represented in this pack (S/A tier)
        arch_seen = defaultdict(int)
        for card in pack:
            for aname in ARCHETYPE_NAMES:
                if is_sa_tier(card, aname):
                    arch_seen[aname] += 1

        # Pick the best card from the most-represented archetype in this pack
        if arch_seen:
            open_arch = max(arch_seen, key=arch_seen.get)
            matching = [c for c in pack if is_sa_tier(c, open_arch)]
            if matching:
                best = max(matching, key=lambda c: card_fitness_score(c, open_arch))
                return best, target_arch

        best = max(pack, key=lambda c: c.power)
        return best, target_arch

    if target_arch is None and pick_num == 6:
        # Commit to archetype with most S/A cards drafted so far
        arch_sa = defaultdict(int)
        for card in drafted:
            for aname in ARCHETYPE_NAMES:
                if is_sa_tier(card, aname):
                    arch_sa[aname] += 1
        if arch_sa:
            target_arch = max(arch_sa, key=arch_sa.get)
        else:
            target_arch = ARCHETYPES[0][0]

    if target_arch:
        best = max(pack, key=lambda c: card_fitness_score(c, target_arch))
        return best, target_arch

    best = max(pack, key=lambda c: c.power)
    return best, target_arch


# ---------------------------------------------------------------------------
# Draft Simulation
# ---------------------------------------------------------------------------

def run_single_draft(card_pool, strategy_fn, majority_threshold=0, variant=None):
    """Run a single draft of 30 picks. Returns drafted cards and per-pick metadata."""
    drafted = []
    pick_data = []
    target_arch = None
    used_ids = set()

    for pick_num in range(1, NUM_PICKS + 1):
        pack = generate_pack(card_pool, drafted, majority_threshold, variant, used_ids)

        chosen, new_target = strategy_fn(pack, drafted, pick_num, target_arch)
        if new_target is not None:
            target_arch = new_target

        sym_counts = count_weighted_symbols(drafted)
        majority = get_majority_resonance(sym_counts, threshold=majority_threshold)

        # ARCHETYPE-LEVEL metrics
        sa_count = count_sa_in_pack(pack, target_arch) if target_arch else 0
        cf_count = count_cf_in_pack(pack, target_arch) if target_arch else 0
        unique_archs = unique_archetypes_with_sa(pack)

        pick_data.append({
            "pick_num": pick_num,
            "pack": pack,
            "chosen": chosen,
            "target_arch": target_arch,
            "majority": majority,
            "sym_counts": dict(sym_counts),
            "sa_for_archetype": sa_count,
            "cf_for_archetype": cf_count,
            "unique_archetypes_sa": unique_archs,
        })

        drafted.append(chosen)
        used_ids.add(chosen.id)

    return drafted, pick_data


# ---------------------------------------------------------------------------
# Metrics Calculation (ALL at ARCHETYPE level)
# ---------------------------------------------------------------------------

def calculate_metrics(all_pick_data, all_drafted):
    """Calculate all 8 measurable target metrics across many drafts."""

    # Metric 1: Picks 1-5, unique archetypes with S/A cards per pack (>= 3)
    early_unique_archs = []
    for draft_picks in all_pick_data:
        for pd in draft_picks:
            if pd["pick_num"] <= 5:
                early_unique_archs.append(pd["unique_archetypes_sa"])
    m1 = sum(early_unique_archs) / len(early_unique_archs) if early_unique_archs else 0

    # Metric 2: Picks 1-5, S/A-tier cards for emerging archetype per pack (<= 2)
    early_sa = []
    for draft_picks in all_pick_data:
        for pd in draft_picks:
            if pd["pick_num"] <= 5 and pd["target_arch"]:
                early_sa.append(pd["sa_for_archetype"])
    m2 = sum(early_sa) / len(early_sa) if early_sa else -1  # -1 = no target yet

    # Metric 3: Picks 6+, S/A-tier cards for committed archetype per pack (>= 2)
    late_sa = []
    for draft_picks in all_pick_data:
        for pd in draft_picks:
            if pd["pick_num"] >= 6 and pd["target_arch"]:
                late_sa.append(pd["sa_for_archetype"])
    m3 = sum(late_sa) / len(late_sa) if late_sa else 0

    # Metric 4: Picks 6+, C/F-tier cards per pack (>= 0.5)
    late_cf = []
    for draft_picks in all_pick_data:
        for pd in draft_picks:
            if pd["pick_num"] >= 6 and pd["target_arch"]:
                late_cf.append(pd["cf_for_archetype"])
    m4 = sum(late_cf) / len(late_cf) if late_cf else 0

    # Metric 5: Convergence pick -- first pick where S/A-tier count >= 2
    convergence_picks = []
    for draft_picks in all_pick_data:
        found = False
        for pd in draft_picks:
            if pd["target_arch"] and pd["sa_for_archetype"] >= 2:
                convergence_picks.append(pd["pick_num"])
                found = True
                break
        if not found:
            convergence_picks.append(31)
    m5 = sum(convergence_picks) / len(convergence_picks) if convergence_picks else 31

    # Metric 6: Deck archetype concentration (60-80% S/A-tier)
    concentrations = []
    for draft_cards, draft_picks in zip(all_drafted, all_pick_data):
        target = None
        for pd in draft_picks:
            if pd["target_arch"]:
                target = pd["target_arch"]
                break
        if target and draft_cards:
            sa_count = sum(1 for c in draft_cards if is_sa_tier(c, target))
            concentrations.append(sa_count / len(draft_cards))
    m6 = sum(concentrations) / len(concentrations) if concentrations else 0

    # Metric 7: Run-to-run variety (< 40% card overlap)
    overlaps = []
    for i in range(min(100, len(all_drafted) - 1)):
        ids_a = set(c.id for c in all_drafted[i])
        ids_b = set(c.id for c in all_drafted[i + 1])
        overlap = len(ids_a & ids_b) / max(len(ids_a | ids_b), 1)
        overlaps.append(overlap)
    m7 = sum(overlaps) / len(overlaps) if overlaps else 0

    # Metric 8: Archetype frequency across runs (no archetype > 20% or < 5%)
    arch_freq = Counter()
    for draft_picks in all_pick_data:
        target = None
        for pd in draft_picks:
            if pd["target_arch"]:
                target = pd["target_arch"]
                break
        if target:
            arch_freq[target] += 1
    total_drafts = sum(arch_freq.values())
    arch_pcts = {}
    for name in ARCHETYPE_NAMES:
        arch_pcts[name] = (arch_freq.get(name, 0) / total_drafts * 100
                           if total_drafts > 0 else 0)
    m8_max = max(arch_pcts.values()) if arch_pcts else 0
    m8_min = min(arch_pcts.values()) if arch_pcts else 0

    return {
        "m1_early_unique_archs": m1,
        "m2_early_sa": m2,
        "m3_late_sa": m3,
        "m4_late_cf": m4,
        "m5_convergence_pick": m5,
        "m6_concentration": m6,
        "m7_overlap": m7,
        "m8_max_arch_pct": m8_max,
        "m8_min_arch_pct": m8_min,
        "arch_pcts": arch_pcts,
    }


def print_metrics(metrics, label):
    """Print metrics table with pass/fail."""
    print(f"\n{'='*70}")
    print(f"  {label}")
    print(f"{'='*70}")
    print(f"{'Metric':<55} {'Target':<12} {'Actual':<10} {'Pass?':<6}")
    print(f"{'-'*55} {'-'*12} {'-'*10} {'-'*6}")

    m = metrics

    # Handle m2: if -1, no target yet (committed player has no target picks 1-5)
    m2_actual = m['m2_early_sa']
    m2_display = f"{m2_actual:.2f}" if m2_actual >= 0 else "N/A*"
    m2_pass = m2_actual <= 2.0 or m2_actual < 0

    rows = [
        ("Picks 1-5: unique archetypes w/ S/A per pack",
         ">= 3", f"{m['m1_early_unique_archs']:.2f}",
         m['m1_early_unique_archs'] >= 3.0),
        ("Picks 1-5: S/A cards for emerging arch per pack",
         "<= 2", m2_display, m2_pass),
        ("Picks 6+: S/A cards for committed arch per pack",
         ">= 2", f"{m['m3_late_sa']:.2f}",
         m['m3_late_sa'] >= 2.0),
        ("Picks 6+: C/F-tier cards per pack",
         ">= 0.5", f"{m['m4_late_cf']:.2f}",
         m['m4_late_cf'] >= 0.5),
        ("Convergence pick (first 2+ S/A for arch)",
         "5-8", f"{m['m5_convergence_pick']:.1f}",
         5 <= m['m5_convergence_pick'] <= 8),
        ("Deck archetype concentration (S/A tier %)",
         "60-80%", f"{m['m6_concentration']*100:.1f}%",
         0.60 <= m['m6_concentration'] <= 0.80),
        ("Run-to-run card overlap",
         "< 40%", f"{m['m7_overlap']*100:.1f}%",
         m['m7_overlap'] < 0.40),
        ("Max archetype frequency",
         "<= 20%", f"{m['m8_max_arch_pct']:.1f}%",
         m['m8_max_arch_pct'] <= 20.0),
        ("Min archetype frequency",
         ">= 5%", f"{m['m8_min_arch_pct']:.1f}%",
         m['m8_min_arch_pct'] >= 5.0),
    ]

    pass_count = 0
    for name, target, actual, passed in rows:
        status = "PASS" if passed else "FAIL"
        if passed:
            pass_count += 1
        print(f"{name:<55} {target:<12} {actual:<10} {status:<6}")

    print(f"\nPassed: {pass_count}/{len(rows)}")

    if m.get("arch_pcts"):
        print(f"\nArchetype distribution:")
        for name, pct in sorted(m["arch_pcts"].items(), key=lambda x: -x[1]):
            print(f"  {name:<15} {pct:.1f}%")


# ---------------------------------------------------------------------------
# Draft Trace (pick-by-pick)
# ---------------------------------------------------------------------------

def run_traced_draft(card_pool, strategy_fn, strategy_name,
                     majority_threshold=0, variant=None):
    """Run a single draft with detailed pick-by-pick trace output."""
    drafted = []
    target_arch = None
    used_ids = set()

    print(f"\n{'='*70}")
    print(f"  DRAFT TRACE: {strategy_name}")
    print(f"{'='*70}")

    for pick_num in range(1, NUM_PICKS + 1):
        pack = generate_pack(card_pool, drafted, majority_threshold, variant, used_ids)

        chosen, new_target = strategy_fn(pack, drafted, pick_num, target_arch)
        if new_target is not None:
            target_arch = new_target

        sym_counts = count_weighted_symbols(drafted)
        majority = get_majority_resonance(sym_counts, threshold=majority_threshold)

        # Archetype-level evaluation
        sa_count = count_sa_in_pack(pack, target_arch) if target_arch else 0
        cf_count = count_cf_in_pack(pack, target_arch) if target_arch else 0
        unique_archs = unique_archetypes_with_sa(pack)

        if pick_num <= 10 or pick_num in [15, 20, 25, 30]:
            pack_desc = []
            for c in pack:
                sym_str = "/".join(c.symbols) if c.symbols else "Generic"
                tier_str = ""
                if target_arch:
                    t = card_tier(c, target_arch)
                    tier_str = f" [{t}-tier for {target_arch}]"
                chosen_mark = " <-- PICKED" if c.id == chosen.id else ""
                pack_desc.append(
                    f"    [{sym_str}] {c.archetype} (pow={c.power})"
                    f"{tier_str}{chosen_mark}"
                )

            sym_str = ", ".join(f"{r}={sym_counts.get(r,0)}" for r in RESONANCES)
            maj_str = f", majority={majority}" if majority else ", no majority"

            print(f"\nPick {pick_num}: symbols=[{sym_str}]{maj_str}")
            if target_arch:
                print(f"  Committed to: {target_arch}")
                print(f"  S/A for arch: {sa_count}, C/F: {cf_count}, "
                      f"Unique archs w/ S/A: {unique_archs}")
            else:
                print(f"  Unique archetypes w/ S/A card: {unique_archs}")
            print(f"  Pack:")
            for line in pack_desc:
                print(line)

        drafted.append(chosen)
        used_ids.add(chosen.id)

    # Final deck summary
    print(f"\n--- Final Deck Summary ---")
    print(f"  Target archetype: {target_arch}")
    sym_counts = count_weighted_symbols(drafted)
    print(f"  Final symbols: {dict(sym_counts)}")

    if target_arch:
        sa_count = sum(1 for c in drafted if is_sa_tier(c, target_arch))
        cf_count = sum(1 for c in drafted if is_cf_tier(c, target_arch))
        print(f"  S/A tier cards: {sa_count}/{len(drafted)} "
              f"({sa_count/len(drafted)*100:.0f}%)")
        print(f"  C/F tier cards: {cf_count}/{len(drafted)} "
              f"({cf_count/len(drafted)*100:.0f}%)")

    arch_counts = Counter(c.archetype for c in drafted)
    print(f"  Cards by home archetype: {dict(arch_counts.most_common())}")


# ---------------------------------------------------------------------------
# Batch Simulation
# ---------------------------------------------------------------------------

def run_batch_simulation(card_pool, strategy_fn, strategy_name,
                         n_sims=NUM_SIMULATIONS,
                         majority_threshold=0, variant=None):
    """Run n_sims drafts and compute aggregate metrics."""
    all_drafted = []
    all_pick_data = []

    for _ in range(n_sims):
        drafted, pick_data = run_single_draft(
            card_pool, strategy_fn, majority_threshold, variant
        )
        all_drafted.append(drafted)
        all_pick_data.append(pick_data)

    return calculate_metrics(all_pick_data, all_drafted)


# ---------------------------------------------------------------------------
# Parameter Sensitivity Sweeps
# ---------------------------------------------------------------------------

def run_sensitivity_sweeps(card_pool):
    """Run parameter sensitivity sweeps."""
    print(f"\n{'#'*70}")
    print(f"  PARAMETER SENSITIVITY SWEEPS")
    print(f"{'#'*70}")

    # Sweep 1: Majority threshold (0, 3, 5)
    print(f"\n--- Sweep 1: Majority Threshold ---")
    print(f"Using archetype-committed strategy, 500 sims each")
    for threshold in [0, 3, 5]:
        metrics = run_batch_simulation(
            card_pool, strategy_archetype_committed,
            f"threshold={threshold}", n_sims=500,
            majority_threshold=threshold
        )
        print(f"\n  Threshold={threshold}:")
        print(f"    Picks 1-5 unique archs w/ S/A: "
              f"{metrics['m1_early_unique_archs']:.2f}")
        print(f"    Picks 6+ S/A for arch: {metrics['m3_late_sa']:.2f}")
        print(f"    Picks 6+ C/F: {metrics['m4_late_cf']:.2f}")
        print(f"    Convergence pick: {metrics['m5_convergence_pick']:.1f}")
        print(f"    Concentration: {metrics['m6_concentration']*100:.1f}%")

    # Sweep 2: Variant A (dual override) and Variant B (threshold gate)
    print(f"\n--- Sweep 2: Algorithm Variants ---")
    print(f"Using archetype-committed strategy, 500 sims each")

    for vname, vcode in [("Standard", None),
                         ("Variant A (dual)", "A"),
                         ("Variant B (threshold gate)", "B")]:
        metrics = run_batch_simulation(
            card_pool, strategy_archetype_committed,
            f"Variant: {vname}", n_sims=500,
            majority_threshold=0, variant=vcode
        )
        print(f"\n  {vname}:")
        print(f"    Picks 1-5 unique archs w/ S/A: "
              f"{metrics['m1_early_unique_archs']:.2f}")
        print(f"    Picks 6+ S/A for arch: {metrics['m3_late_sa']:.2f}")
        print(f"    Picks 6+ C/F: {metrics['m4_late_cf']:.2f}")
        print(f"    Convergence pick: {metrics['m5_convergence_pick']:.1f}")
        print(f"    Concentration: {metrics['m6_concentration']*100:.1f}%")

    # Sweep 3: Symbol distribution
    print(f"\n--- Sweep 3: Symbol Distribution ---")
    print(f"Using archetype-committed strategy, 500 sims each")

    distributions = [
        ("Mostly 1-sym (60/30/10)", (0.60, 0.30, 0.10)),
        ("Baseline (30/55/15)",     (0.30, 0.55, 0.15)),
        ("Mostly 2-sym (15/70/15)", (0.15, 0.70, 0.15)),
        ("Mostly 3-sym (10/30/60)", (0.10, 0.30, 0.60)),
    ]

    for dist_name, dist in distributions:
        pool = build_card_pool(symbol_dist=dist)
        metrics = run_batch_simulation(
            pool, strategy_archetype_committed,
            f"Dist: {dist_name}", n_sims=500
        )
        print(f"\n  {dist_name}:")
        print(f"    Picks 1-5 unique archs w/ S/A: "
              f"{metrics['m1_early_unique_archs']:.2f}")
        print(f"    Picks 6+ S/A for arch: {metrics['m3_late_sa']:.2f}")
        print(f"    Picks 6+ C/F: {metrics['m4_late_cf']:.2f}")
        print(f"    Convergence pick: {metrics['m5_convergence_pick']:.1f}")
        print(f"    Concentration: {metrics['m6_concentration']*100:.1f}%")
        print(f"    Run overlap: {metrics['m7_overlap']*100:.1f}%")


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    random.seed(42)

    print("=" * 70)
    print("  BALANCED PACK WITH MAJORITY BONUS -- Draft Simulation")
    print("  Agent 2: Structural/Guaranteed Domain")
    print("  (Corrected: archetype-level metrics)")
    print("=" * 70)
    print()
    print('Algorithm: "Each pack has one card per resonance type, but if you')
    print("have a clear majority resonance (strictly more weighted symbols than")
    print("any other, counting primary=2), it replaces one random non-majority")
    print('slot, giving you 2 of your majority resonance."')
    print()
    print(f"Card pool: {NUM_CARDS} cards ({NUM_GENERIC} generic + "
          f"{NUM_ARCHETYPE_CARDS} archetype)")
    print(f"Symbol distribution (non-generic): 30% 1-sym, 55% 2-sym, 15% 3-sym")
    print(f"Pack size: {PACK_SIZE}, Picks per draft: {NUM_PICKS}")
    print(f"Simulations: {NUM_SIMULATIONS} per strategy")

    # Build card pool
    card_pool = build_card_pool()

    # Verify card pool
    sym_counts_pool = Counter()
    for c in card_pool:
        sym_counts_pool[len(c.symbols)] += 1
    print(f"\nCard pool symbol distribution: {dict(sym_counts_pool)}")

    # Verify fitness distribution
    print(f"\nFitness verification (sample: first archetype card):")
    sample = [c for c in card_pool if c.archetype != "Generic"][0]
    print(f"  {sample.archetype} card {sample.id}, symbols={sample.symbols}")
    for aname in ARCHETYPE_NAMES:
        print(f"    {aname}: {sample.archetype_fitness[aname]}")

    # Run batch simulations for all 3 strategies
    print(f"\n{'#'*70}")
    print(f"  MAIN SIMULATION RESULTS ({NUM_SIMULATIONS} drafts per strategy)")
    print(f"{'#'*70}")

    strategies = [
        ("Archetype-Committed", strategy_archetype_committed),
        ("Power-Chaser", strategy_power_chaser),
        ("Signal-Reader", strategy_signal_reader),
    ]

    for name, fn in strategies:
        metrics = run_batch_simulation(card_pool, fn, name)
        print_metrics(metrics, f"Strategy: {name}")

    # Run parameter sensitivity sweeps
    run_sensitivity_sweeps(card_pool)

    # Run 3 detailed draft traces
    print(f"\n{'#'*70}")
    print(f"  DETAILED DRAFT TRACES")
    print(f"{'#'*70}")

    random.seed(123)
    trace_pool = build_card_pool()

    # Trace 1: Early committer (archetype-committed)
    random.seed(200)
    run_traced_draft(trace_pool, strategy_archetype_committed,
                     "Early Committer (Archetype-Committed)")

    # Trace 2: Flexible player (power-chaser, stays flexible 8+ picks)
    random.seed(300)
    run_traced_draft(trace_pool, strategy_power_chaser,
                     "Flexible Player (Power-Chaser)")

    # Trace 3: Signal-reader
    random.seed(400)
    run_traced_draft(trace_pool, strategy_signal_reader, "Signal Reader")

    print(f"\n{'='*70}")
    print(f"  SIMULATION COMPLETE")
    print(f"{'='*70}")


if __name__ == "__main__":
    main()
