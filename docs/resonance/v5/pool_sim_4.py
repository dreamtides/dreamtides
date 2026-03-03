#!/usr/bin/env python3
"""
Agent 4: Symbol Pattern Composition for Pair-Escalation Slots

Investigates what specific symbol patterns cards should have and how different
patterns affect the pair-matching economy under the Pair-Escalation Slots
algorithm.

Algorithm: "Track the ordered symbol pair (first, second) of each 2+ symbol
card you draft; each pack slot independently shows a card matching your most
common pair with probability equal to that pair's count divided by 6 (capped
at 50%), otherwise a random card."

Parameters: K=6, C=0.50, 4 slots, 30 picks.

Key insight: Only patterns where position[0]=Primary AND position[1]=Secondary
produce the HOME archetype's ordered pair. Patterns like [P,P], [P,O], [S,P]
produce DIFFERENT pairs that feed other archetypes or degenerate (same-resonance)
pairs.

Tests 5 pattern compositions:
  1. All [P,S] -- maximum pair alignment
  2. Concentrated+Bridge -- mix of [P,S], [P,P], [P,O]
  3. Home-pair dominant (80% [P,S])
  4. Mixed pairs (50% [P,S], 25% [S,P], 25% [P,O])
  5. Degenerate-heavy (50% [P,S], 30% [P,P], 20% [P,O])

Measures per-configuration:
  - Home pair contribution rate
  - Pair scatter (distinct pairs accumulated)
  - Pair-matched pool S-tier composition
  - Cross-archetype pair feeding
  - Genuine choice rate
  - Degenerate pair impact
  - All 9 standard measurable targets
"""

import random
import math
import statistics
from dataclasses import dataclass, field
from enum import Enum
from collections import Counter, defaultdict
from typing import Optional

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

RESONANCES = ["Ember", "Stone", "Tide", "Zephyr"]

ARCHETYPES = [
    {"id": 0, "name": "Flash/Tempo/Prison",   "primary": "Zephyr", "secondary": "Ember"},
    {"id": 1, "name": "Blink/Flicker",        "primary": "Ember",  "secondary": "Zephyr"},
    {"id": 2, "name": "Storm/Spellslinger",   "primary": "Ember",  "secondary": "Stone"},
    {"id": 3, "name": "Self-Discard",          "primary": "Stone",  "secondary": "Ember"},
    {"id": 4, "name": "Self-Mill/Reanimator",  "primary": "Stone",  "secondary": "Tide"},
    {"id": 5, "name": "Sacrifice/Abandon",     "primary": "Tide",   "secondary": "Stone"},
    {"id": 6, "name": "Warriors/Midrange",     "primary": "Tide",   "secondary": "Zephyr"},
    {"id": 7, "name": "Ramp/Spirit Animals",   "primary": "Zephyr", "secondary": "Tide"},
]

NUM_ARCHETYPES = 8
NUM_CARDS = 360
GENERIC_COUNT = 36
CARDS_PER_ARCHETYPE = (NUM_CARDS - GENERIC_COUNT) // NUM_ARCHETYPES  # 40
PACK_SIZE = 4
NUM_PICKS = 30
NUM_DRAFTS = 1200

# Pair-Escalation Slots parameters
PE_K = 6
PE_CAP = 0.50

# Build pair-to-archetype mapping
PAIR_TO_ARCH = {}
for _a in ARCHETYPES:
    PAIR_TO_ARCH[(_a["primary"], _a["secondary"])] = _a["id"]


# ---------------------------------------------------------------------------
# Card model
# ---------------------------------------------------------------------------

@dataclass
class SimCard:
    id: int
    symbols: list          # list of resonance strings, ordered
    archetype: Optional[int]  # archetype id, None for generic
    power: float
    archetype_fitness: dict = field(default_factory=dict)  # archetype_id -> tier str
    pattern_tag: str = ""  # symbolic pattern tag like "PS", "PP", "PO", "SP"

    @property
    def ordered_pair(self):
        if len(self.symbols) >= 2:
            return (self.symbols[0], self.symbols[1])
        return None


def circle_distance(a_id, b_id):
    d = abs(a_id - b_id)
    return min(d, NUM_ARCHETYPES - d)


def assign_fitness(card):
    fitness = {}
    if card.archetype is None:
        for a in ARCHETYPES:
            fitness[a["id"]] = "B"
    else:
        home = ARCHETYPES[card.archetype]
        for a in ARCHETYPES:
            if a["id"] == card.archetype:
                fitness[a["id"]] = "S"
            elif circle_distance(a["id"], card.archetype) == 1:
                if a["primary"] == home["primary"] or a["secondary"] == home["primary"]:
                    fitness[a["id"]] = "A"
                else:
                    fitness[a["id"]] = "B"
            elif circle_distance(a["id"], card.archetype) == 2:
                fitness[a["id"]] = "C"
            else:
                fitness[a["id"]] = "F"
    card.archetype_fitness = fitness


# ---------------------------------------------------------------------------
# Symbol pattern resolution
# ---------------------------------------------------------------------------

def resolve_pattern(pattern_str, primary, secondary, rng):
    """Resolve a symbolic pattern like 'PSO' into actual resonance list.
    P=primary, S=secondary, O=random other resonance."""
    others = [r for r in RESONANCES if r != primary and r != secondary]
    result = []
    for ch in pattern_str:
        if ch == 'P':
            result.append(primary)
        elif ch == 'S':
            result.append(secondary)
        elif ch == 'O':
            result.append(rng.choice(others))
        else:
            raise ValueError(f"Unknown symbol type: {ch}")
    return result


# ---------------------------------------------------------------------------
# Pool construction with configurable pattern composition
# ---------------------------------------------------------------------------

def build_pool_with_patterns(rng, pattern_config, pct_1sym=0.15, pct_2sym=0.60,
                              pct_3sym=0.25):
    """
    Build a card pool where each archetype's cards follow the given pattern
    composition.

    pattern_config is a dict with keys:
      "1sym": dict of pattern_str -> weight (e.g. {"P": 0.7, "S": 0.3})
      "2sym": dict of pattern_str -> weight (e.g. {"PS": 0.8, "PP": 0.1, "PO": 0.1})
      "3sym": dict of pattern_str -> weight (e.g. {"PSP": 0.5, "PPS": 0.3, "PSO": 0.2})
    """
    cards = []
    card_id = 0

    for arch in ARCHETYPES:
        pri = arch["primary"]
        sec = arch["secondary"]
        n = CARDS_PER_ARCHETYPE
        n1 = round(n * pct_1sym)
        n3 = round(n * pct_3sym)
        n2 = n - n1 - n3

        for sym_count, n_cards, key in [(1, n1, "1sym"), (2, n2, "2sym"), (3, n3, "3sym")]:
            pats_weights = pattern_config.get(key, {})
            if not pats_weights:
                # Default: all primary
                pats_weights = {"P" * sym_count: 1.0}

            pat_strs = list(pats_weights.keys())
            weights = list(pats_weights.values())

            for _ in range(n_cards):
                pat_str = rng.choices(pat_strs, weights=weights, k=1)[0]
                symbols = resolve_pattern(pat_str, pri, sec, rng)
                c = SimCard(
                    id=card_id,
                    symbols=symbols,
                    archetype=arch["id"],
                    power=rng.uniform(3, 8),
                    pattern_tag=pat_str,
                )
                assign_fitness(c)
                cards.append(c)
                card_id += 1

    # Generic cards (no symbols)
    for _ in range(GENERIC_COUNT):
        c = SimCard(id=card_id, symbols=[], archetype=None,
                    power=rng.uniform(4, 9), pattern_tag="")
        assign_fitness(c)
        cards.append(c)
        card_id += 1

    return cards


# ---------------------------------------------------------------------------
# Pattern composition configurations
# ---------------------------------------------------------------------------

CONFIGS = {
    "1_All_PS": {
        "desc": "All 2+ sym cards produce home pair [P,S]. Maximum pair alignment.",
        "1sym": {"P": 0.70, "S": 0.30},
        "2sym": {"PS": 1.0},
        "3sym": {"PSP": 0.50, "PSS": 0.25, "PSO": 0.25},
    },
    "2_Concentrated_Bridge": {
        "desc": "V4-style mix: [P,S]+[P,P]+[P,O]. Some degenerate/cross pairs.",
        "1sym": {"P": 0.70, "S": 0.30},
        "2sym": {"PS": 0.50, "PP": 0.25, "PO": 0.25},
        "3sym": {"PSP": 0.30, "PPS": 0.30, "PPO": 0.20, "PSO": 0.20},
    },
    "3_Home_Dominant_80": {
        "desc": "80% home pair [P,S], 10% [P,P], 10% [S,P]. Mostly aligned.",
        "1sym": {"P": 0.70, "S": 0.30},
        "2sym": {"PS": 0.80, "PP": 0.10, "SP": 0.10},
        "3sym": {"PSP": 0.60, "PPS": 0.15, "SPP": 0.10, "PSO": 0.15},
    },
    "4_Mixed_Pairs": {
        "desc": "50% [P,S], 25% [S,P] (adj pair), 25% [P,O] (cross). High scatter.",
        "1sym": {"P": 0.50, "S": 0.50},
        "2sym": {"PS": 0.50, "SP": 0.25, "PO": 0.25},
        "3sym": {"PSP": 0.30, "SPP": 0.20, "SPS": 0.15, "POO": 0.15, "PSO": 0.20},
    },
    "5_Degenerate_Heavy": {
        "desc": "50% [P,S], 30% [P,P] (degenerate), 20% [P,O] (cross). Pair waste.",
        "1sym": {"P": 0.80, "S": 0.20},
        "2sym": {"PS": 0.50, "PP": 0.30, "PO": 0.20},
        "3sym": {"PSP": 0.25, "PPS": 0.25, "PPP": 0.20, "PPO": 0.15, "PSO": 0.15},
    },
}


# ---------------------------------------------------------------------------
# Pair-Escalation Slots: pack generation
# ---------------------------------------------------------------------------

def gen_pack_pair_escalation(pool, pair_counts, rng, K=PE_K, cap=PE_CAP):
    """
    Each slot independently pair-matched with probability min(top_pair/K, cap).
    """
    top_pair, top_count = get_top_pair(pair_counts)
    prob = min(top_count / K, cap) if top_pair else 0.0
    pair_matched = [c for c in pool if c.ordered_pair == top_pair] if top_pair else []

    pack = []
    used_ids = set()
    for _ in range(PACK_SIZE):
        if top_pair and pair_matched and rng.random() < prob:
            chosen = rng.choice(pair_matched)
        else:
            candidates = [c for c in pool if c.id not in used_ids]
            if not candidates:
                candidates = pool
            chosen = rng.choice(candidates)
        pack.append(chosen)
        used_ids.add(chosen.id)
    return pack


def get_top_pair(pair_counts):
    if not pair_counts:
        return None, 0
    top = max(pair_counts.items(), key=lambda x: x[1])
    return top[0], top[1]


def update_pair_counters(card, pair_counts):
    p = card.ordered_pair
    if p:
        pair_counts[p] = pair_counts.get(p, 0) + 1


# ---------------------------------------------------------------------------
# Player strategy
# ---------------------------------------------------------------------------

TIER_ORDER = {"S": 5, "A": 4, "B": 3, "C": 2, "F": 1}


def pick_card_committed(pack, target_arch, pick_num, pair_counts):
    """Committed player: pick best fitness for target arch."""
    if target_arch is not None:
        best = max(pack, key=lambda c: (
            TIER_ORDER.get(c.archetype_fitness.get(target_arch, "F"), 0),
            c.power))
        return best

    # Before commitment, pick highest power
    return max(pack, key=lambda c: c.power)


def determine_target_from_pairs(pair_counts):
    """Determine best archetype from pair counts."""
    if not pair_counts:
        return None
    arch_scores = [0.0] * NUM_ARCHETYPES
    for (p, s), cnt in pair_counts.items():
        arch = PAIR_TO_ARCH.get((p, s))
        if arch is not None:
            arch_scores[arch] += cnt
        # Partial credit: primary match
        for ai, a in enumerate(ARCHETYPES):
            if a["primary"] == p and ai != arch:
                arch_scores[ai] += cnt * 0.3
    if max(arch_scores) == 0:
        return None
    return arch_scores.index(max(arch_scores))


# ---------------------------------------------------------------------------
# Draft result
# ---------------------------------------------------------------------------

@dataclass
class DraftResult:
    picks: list = field(default_factory=list)
    packs_seen: list = field(default_factory=list)
    target_archetype: int = 0
    pair_counts: dict = field(default_factory=dict)
    pair_history: list = field(default_factory=list)  # list of (pick_num, pair_added)


# ---------------------------------------------------------------------------
# Single draft
# ---------------------------------------------------------------------------

def run_single_draft(pool, rng, forced_arch=None):
    """Run one 30-pick draft under Pair-Escalation Slots."""
    pair_counts = {}
    result = DraftResult()
    target_arch = forced_arch

    for pick_num in range(NUM_PICKS):
        pack = gen_pack_pair_escalation(pool, pair_counts, rng)

        if not pack:
            break

        result.packs_seen.append(list(pack))

        # Pick card
        chosen = pick_card_committed(pack, target_arch, pick_num, pair_counts)
        result.picks.append(chosen)

        # Update pair tracking
        old_top_pair, old_top_count = get_top_pair(pair_counts)
        update_pair_counters(chosen, pair_counts)
        new_pair = chosen.ordered_pair
        result.pair_history.append((pick_num, new_pair))

        # Commitment: at pick 5, commit to best archetype from pair profile
        if target_arch is None and pick_num >= 4:
            target_arch = determine_target_from_pairs(pair_counts)
            if target_arch is None:
                target_arch = rng.randint(0, NUM_ARCHETYPES - 1)

    if target_arch is None:
        target_arch = determine_target_from_pairs(pair_counts) or 0
    result.target_archetype = target_arch
    result.pair_counts = dict(pair_counts)
    return result


# ---------------------------------------------------------------------------
# Pool analysis (static, before drafting)
# ---------------------------------------------------------------------------

def analyze_pool_patterns(pool):
    """Analyze the pattern composition of a pool and its pair-matching properties."""
    stats = {}

    # Count patterns per archetype
    for arch in ARCHETYPES:
        aid = arch["id"]
        arch_cards = [c for c in pool if c.archetype == aid]
        home_pair = (arch["primary"], arch["secondary"])

        total_2plus = sum(1 for c in arch_cards if len(c.symbols) >= 2)
        home_pair_cards = sum(1 for c in arch_cards if c.ordered_pair == home_pair)
        degenerate_cards = sum(1 for c in arch_cards
                               if c.ordered_pair and c.ordered_pair[0] == c.ordered_pair[1])

        # Cards whose pair matches an adjacent archetype
        adj_pair_cards = 0
        for c in arch_cards:
            if c.ordered_pair and c.ordered_pair != home_pair:
                if c.ordered_pair in PAIR_TO_ARCH:
                    adj_pair_cards += 1

        stats[aid] = {
            "total_2plus": total_2plus,
            "home_pair_count": home_pair_cards,
            "home_pair_rate": home_pair_cards / total_2plus if total_2plus > 0 else 0,
            "degenerate_count": degenerate_cards,
            "degenerate_rate": degenerate_cards / total_2plus if total_2plus > 0 else 0,
            "adj_pair_count": adj_pair_cards,
            "adj_pair_rate": adj_pair_cards / total_2plus if total_2plus > 0 else 0,
        }

    # Overall pair-matched pool composition: for each archetype pair,
    # what % of matching cards are S-tier for that archetype?
    pair_precision = {}
    for pair_key, arch_id in PAIR_TO_ARCH.items():
        matched = [c for c in pool if c.ordered_pair == pair_key]
        if matched:
            s_count = sum(1 for c in matched
                          if c.archetype_fitness.get(arch_id, "F") == "S")
            a_count = sum(1 for c in matched
                          if c.archetype_fitness.get(arch_id, "F") == "A")
            b_count = sum(1 for c in matched
                          if c.archetype_fitness.get(arch_id, "F") == "B")
            cf_count = sum(1 for c in matched
                           if c.archetype_fitness.get(arch_id, "F") in ("C", "F"))
            total = len(matched)
            pair_precision[pair_key] = {
                "total": total,
                "S_pct": s_count / total,
                "A_pct": a_count / total,
                "B_pct": b_count / total,
                "CF_pct": cf_count / total,
                "SA_pct": (s_count + a_count) / total,
            }

    return stats, pair_precision


# ---------------------------------------------------------------------------
# Draft metrics computation
# ---------------------------------------------------------------------------

def compute_draft_metrics(results):
    """Compute all measurable targets + pattern-specific metrics."""
    early_unique = []
    early_sa = []
    late_sa = []
    late_off = []
    conv_picks = []
    deck_concs = []
    all_decks = []
    arch_freq = defaultdict(int)

    # Pattern-specific metrics
    home_pair_contribution_rates = []
    pair_scatter_counts = []
    cross_arch_feed_rates = []
    degenerate_pair_fractions = []
    genuine_choice_rates = []

    for dr in results:
        tgt = dr.target_archetype
        arch_freq[tgt] += 1
        home_pair = (ARCHETYPES[tgt]["primary"], ARCHETYPES[tgt]["secondary"])

        # Standard metrics
        sa_streak = 0
        conv = NUM_PICKS

        for pn, pack in enumerate(dr.packs_seen):
            sa_count = sum(1 for c in pack
                           if c.archetype_fitness.get(tgt, "F") in ("S", "A"))
            cf_count = sum(1 for c in pack
                           if c.archetype_fitness.get(tgt, "F") in ("C", "F"))

            if pn < 5:
                unique_archs = set()
                for c in pack:
                    for ai in range(NUM_ARCHETYPES):
                        if c.archetype_fitness.get(ai, "F") in ("S", "A"):
                            unique_archs.add(ai)
                early_unique.append(len(unique_archs))
                early_sa.append(sa_count)
            else:
                late_sa.append(sa_count)
                late_off.append(cf_count)

                # Genuine choice: 2+ S/A cards that feed DIFFERENT pairs
                sa_cards = [c for c in pack
                            if c.archetype_fitness.get(tgt, "F") in ("S", "A")]
                if len(sa_cards) >= 2:
                    pairs_in_pack = set(c.ordered_pair for c in sa_cards if c.ordered_pair)
                    if len(pairs_in_pack) >= 2:
                        genuine_choice_rates.append(1)
                    else:
                        genuine_choice_rates.append(0)
                else:
                    genuine_choice_rates.append(0)

            if pn >= 5:
                if sa_count >= 2:
                    sa_streak += 1
                else:
                    sa_streak = 0
                if sa_streak >= 3 and conv == NUM_PICKS:
                    conv = pn - 2

        conv_picks.append(conv)
        sa_deck = sum(1 for c in dr.picks
                      if c.archetype_fitness.get(tgt, "F") in ("S", "A"))
        deck_concs.append(sa_deck / len(dr.picks) if dr.picks else 0)
        all_decks.append(set(c.id for c in dr.picks))

        # Pattern-specific: home pair contribution rate
        pair_cards = [c for c in dr.picks if c.ordered_pair is not None]
        if pair_cards:
            home_count = sum(1 for c in pair_cards if c.ordered_pair == home_pair)
            home_pair_contribution_rates.append(home_count / len(pair_cards))
        else:
            home_pair_contribution_rates.append(0)

        # Pair scatter: how many distinct pairs accumulated
        pair_scatter_counts.append(len(dr.pair_counts))

        # Cross-archetype pair feeding: how many picks produced a pair
        # that matches a DIFFERENT archetype (not home, not degenerate)
        cross_count = 0
        degen_count = 0
        total_pair_picks = 0
        for c in dr.picks:
            p = c.ordered_pair
            if p:
                total_pair_picks += 1
                if p[0] == p[1]:
                    degen_count += 1
                elif p != home_pair and p in PAIR_TO_ARCH:
                    cross_count += 1

        if total_pair_picks > 0:
            cross_arch_feed_rates.append(cross_count / total_pair_picks)
            degenerate_pair_fractions.append(degen_count / total_pair_picks)
        else:
            cross_arch_feed_rates.append(0)
            degenerate_pair_fractions.append(0)

    # Compute overlap
    overlaps = []
    by_arch = defaultdict(list)
    for i, dr in enumerate(results):
        by_arch[dr.target_archetype].append(i)
    for arch, indices in by_arch.items():
        for i in range(min(50, len(indices))):
            for j in range(i + 1, min(50, len(indices))):
                s1 = all_decks[indices[i]]
                s2 = all_decks[indices[j]]
                if s1 and s2:
                    overlaps.append(len(s1 & s2) / max(len(s1 | s2), 1))

    total = len(results)
    af = {a: arch_freq.get(a, 0) / total for a in range(NUM_ARCHETYPES)}
    stddev_val = statistics.stdev(late_sa) if len(late_sa) > 1 else 0

    return {
        "early_unique": statistics.mean(early_unique) if early_unique else 0,
        "early_sa": statistics.mean(early_sa) if early_sa else 0,
        "late_sa": statistics.mean(late_sa) if late_sa else 0,
        "late_off": statistics.mean(late_off) if late_off else 0,
        "conv_pick": statistics.mean(conv_picks) if conv_picks else 30,
        "deck_conc": statistics.mean(deck_concs) if deck_concs else 0,
        "overlap": statistics.mean(overlaps) if overlaps else 0,
        "arch_freq_max": max(af.values()) if af else 0,
        "arch_freq_min": min(af.values()) if af else 0,
        "stddev": stddev_val,
        # Pattern-specific
        "home_pair_rate": statistics.mean(home_pair_contribution_rates),
        "pair_scatter": statistics.mean(pair_scatter_counts),
        "cross_arch_feed": statistics.mean(cross_arch_feed_rates),
        "degenerate_frac": statistics.mean(degenerate_pair_fractions),
        "genuine_choice": statistics.mean(genuine_choice_rates),
    }


# ---------------------------------------------------------------------------
# Pair-matched pool composition analysis
# ---------------------------------------------------------------------------

def analyze_pair_pool_composition(pool, target_arch_id):
    """For a given target archetype, analyze what the pair-matched pool looks like."""
    arch = ARCHETYPES[target_arch_id]
    home_pair = (arch["primary"], arch["secondary"])
    matched = [c for c in pool if c.ordered_pair == home_pair]

    if not matched:
        return {"total": 0, "S_pct": 0, "A_pct": 0, "B_pct": 0, "CF_pct": 0}

    total = len(matched)
    tiers = Counter()
    for c in matched:
        tier = c.archetype_fitness.get(target_arch_id, "F")
        tiers[tier] += 1

    return {
        "total": total,
        "S_pct": tiers.get("S", 0) / total,
        "A_pct": tiers.get("A", 0) / total,
        "B_pct": tiers.get("B", 0) / total,
        "CF_pct": (tiers.get("C", 0) + tiers.get("F", 0)) / total,
    }


# ---------------------------------------------------------------------------
# Main simulation
# ---------------------------------------------------------------------------

def run_config(config_name, config, rng_base_seed=1000):
    """Run full simulation for one pattern configuration."""
    pool_rng = random.Random(42)
    pool = build_pool_with_patterns(
        pool_rng, config,
        pct_1sym=0.15, pct_2sym=0.60, pct_3sym=0.25,
    )

    # Pool analysis
    pool_stats, pair_precision = analyze_pool_patterns(pool)

    # Average home pair rate across archetypes
    avg_home_rate = statistics.mean(
        pool_stats[aid]["home_pair_rate"] for aid in range(NUM_ARCHETYPES))
    avg_degen_rate = statistics.mean(
        pool_stats[aid]["degenerate_rate"] for aid in range(NUM_ARCHETYPES))

    # Average pair precision
    if pair_precision:
        avg_sa_precision = statistics.mean(
            pp["SA_pct"] for pp in pair_precision.values())
        avg_s_precision = statistics.mean(
            pp["S_pct"] for pp in pair_precision.values())
    else:
        avg_sa_precision = 0
        avg_s_precision = 0

    # Pair-matched pool composition (averaged over archetypes)
    pool_comp = []
    for aid in range(NUM_ARCHETYPES):
        comp = analyze_pair_pool_composition(pool, aid)
        pool_comp.append(comp)
    avg_pool_size = statistics.mean(c["total"] for c in pool_comp)
    avg_pool_s = statistics.mean(c["S_pct"] for c in pool_comp)
    avg_pool_a = statistics.mean(c["A_pct"] for c in pool_comp)
    avg_pool_b = statistics.mean(c["B_pct"] for c in pool_comp)

    # Run drafts
    all_results = []
    for i in range(NUM_DRAFTS):
        rng = random.Random(rng_base_seed + i)
        forced = rng.randint(0, NUM_ARCHETYPES - 1)
        dr = run_single_draft(pool, rng, forced_arch=forced)
        all_results.append(dr)

    metrics = compute_draft_metrics(all_results)

    # Add pool-level stats
    metrics["pool_home_pair_rate"] = avg_home_rate
    metrics["pool_degen_rate"] = avg_degen_rate
    metrics["pool_sa_precision"] = avg_sa_precision
    metrics["pool_s_precision"] = avg_s_precision
    metrics["pool_matched_size"] = avg_pool_size
    metrics["pool_matched_s_pct"] = avg_pool_s
    metrics["pool_matched_a_pct"] = avg_pool_a
    metrics["pool_matched_b_pct"] = avg_pool_b

    return metrics, pool_stats, pair_precision


def print_pass_fail(val, target_fn):
    return "PASS" if target_fn(val) else "FAIL"


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    print("=" * 100)
    print("  Agent 4: Symbol Pattern Composition for Pair-Escalation Slots")
    print("  Algorithm: K=6, C=0.50, 4 slots, 30 picks")
    print(f"  Drafts per config: {NUM_DRAFTS}")
    print("=" * 100)

    # =====================================================================
    # 1. ENUMERATE PAIR PROFILES PER PATTERN
    # =====================================================================
    print("\n" + "=" * 80)
    print("  SECTION 1: Ordered Pair Produced per Pattern (Warriors example: P=Tide, S=Zephyr)")
    print("=" * 80)

    example_patterns = {
        "1-sym [P]":    ("P",   None),
        "1-sym [S]":    ("S",   None),
        "2-sym [P,S]":  ("PS",  ("Tide", "Zephyr")),
        "2-sym [P,P]":  ("PP",  ("Tide", "Tide")),
        "2-sym [S,P]":  ("SP",  ("Zephyr", "Tide")),
        "2-sym [P,O]":  ("PO",  ("Tide", "Other")),
        "2-sym [S,S]":  ("SS",  ("Zephyr", "Zephyr")),
        "2-sym [S,O]":  ("SO",  ("Zephyr", "Other")),
        "3-sym [P,S,P]": ("PSP", ("Tide", "Zephyr")),
        "3-sym [P,S,S]": ("PSS", ("Tide", "Zephyr")),
        "3-sym [P,S,O]": ("PSO", ("Tide", "Zephyr")),
        "3-sym [P,P,S]": ("PPS", ("Tide", "Tide")),
        "3-sym [P,P,P]": ("PPP", ("Tide", "Tide")),
        "3-sym [S,P,S]": ("SPS", ("Zephyr", "Tide")),
    }

    print(f"\n  {'Pattern':<20} {'Ordered Pair':<25} {'Feeds Archetype':<25} {'Type'}")
    print("  " + "-" * 85)
    for name, (tag, pair) in example_patterns.items():
        if pair is None:
            print(f"  {name:<20} {'(none -- 1 symbol)':<25} {'(no pair)':<25} No contribution")
        elif pair[0] == pair[1]:
            print(f"  {name:<20} {str(pair):<25} {'(DEGENERATE)':<25} Wasted pair token")
        elif pair == ("Tide", "Zephyr"):
            mapped = PAIR_TO_ARCH.get(pair, "?")
            arch_name = ARCHETYPES[mapped]["name"] if isinstance(mapped, int) else "?"
            print(f"  {name:<20} {str(pair):<25} {arch_name:<25} HOME pair")
        elif pair == ("Zephyr", "Tide"):
            mapped = PAIR_TO_ARCH.get(pair, "?")
            arch_name = ARCHETYPES[mapped]["name"] if isinstance(mapped, int) else "?"
            print(f"  {name:<20} {str(pair):<25} {arch_name:<25} ADJACENT pair")
        else:
            pair_str = f"({pair[0]}, {pair[1]})" if pair[1] != "Other" else f"({pair[0]}, Other)"
            print(f"  {name:<20} {pair_str:<25} {'(varies)':<25} CROSS pair")

    # =====================================================================
    # 2. RUN ALL 5 CONFIGURATIONS
    # =====================================================================
    print("\n\n" + "=" * 100)
    print("  SECTION 2: Simulation Results ({} drafts each)".format(NUM_DRAFTS))
    print("=" * 100)

    all_metrics = {}
    all_pool_stats = {}
    all_pair_precision = {}

    for config_name, config in CONFIGS.items():
        print(f"\n  Running {config_name}: {config['desc']}", flush=True)
        metrics, pool_stats, pair_precision = run_config(config_name, config)
        all_metrics[config_name] = metrics
        all_pool_stats[config_name] = pool_stats
        all_pair_precision[config_name] = pair_precision

        # Quick summary
        print(f"    Late S/A: {metrics['late_sa']:.2f}, "
              f"HomePairRate: {metrics['home_pair_rate']:.1%}, "
              f"PairScatter: {metrics['pair_scatter']:.1f}, "
              f"CrossFeed: {metrics['cross_arch_feed']:.1%}, "
              f"DegenFrac: {metrics['degenerate_frac']:.1%}")

    # =====================================================================
    # 3. STANDARD TARGETS TABLE
    # =====================================================================
    print("\n\n" + "=" * 120)
    print("  SECTION 3: Standard Measurable Targets")
    print("=" * 120)

    targets = [
        ("Early unique archs (>=3)",       "early_unique", lambda v: v >= 3),
        ("Early S/A emerging (<=2)",        "early_sa",     lambda v: v <= 2),
        ("Late S/A committed (>=2)",        "late_sa",      lambda v: v >= 2),
        ("Late off-arch C/F (>=0.5)",       "late_off",     lambda v: v >= 0.5),
        ("Convergence pick (5-8)",          "conv_pick",    lambda v: 5 <= v <= 8),
        ("Deck concentration (60-90%)",     "deck_conc",    lambda v: 0.60 <= v <= 0.90),
        ("Run-to-run overlap (<40%)",       "overlap",      lambda v: v < 0.40),
        ("Arch freq max (<20%)",            "arch_freq_max", lambda v: v <= 0.20),
        ("Arch freq min (>5%)",             "arch_freq_min", lambda v: v >= 0.05),
        ("StdDev S/A late (>=0.8)",         "stddev",       lambda v: v >= 0.8),
    ]

    header = f"  {'Metric':<35}"
    for cn in all_metrics:
        header += f" | {cn:>22}"
    print(header)
    print("  " + "-" * (35 + 25 * len(all_metrics)))

    for name, key, check in targets:
        row = f"  {name:<35}"
        for cn, m in all_metrics.items():
            v = m[key]
            if "conc" in key or "overlap" in key or "freq" in key:
                vs = f"{v:.1%}"
            elif "pick" in key:
                vs = f"{v:.1f}"
            else:
                vs = f"{v:.2f}"
            pf = "P" if check(v) else "F"
            row += f" | {vs:>18}({pf})"
        print(row)

    # Pass count
    print()
    row = f"  {'Targets Passed':<35}"
    for cn, m in all_metrics.items():
        passes = sum(1 for _, key, check in targets if check(m[key]))
        row += f" | {passes:>19}/10"
    print(row)

    # =====================================================================
    # 4. PATTERN-SPECIFIC METRICS TABLE
    # =====================================================================
    print("\n\n" + "=" * 120)
    print("  SECTION 4: Pattern-Specific Metrics")
    print("=" * 120)

    pattern_metrics = [
        ("Home pair contribution rate",  "home_pair_rate",    "{:.1%}"),
        ("Pair scatter (distinct pairs)", "pair_scatter",      "{:.1f}"),
        ("Cross-arch pair feeding",      "cross_arch_feed",   "{:.1%}"),
        ("Degenerate pair fraction",     "degenerate_frac",   "{:.1%}"),
        ("Genuine choice rate (late)",   "genuine_choice",    "{:.1%}"),
        ("Pool: home pair rate",         "pool_home_pair_rate", "{:.1%}"),
        ("Pool: degenerate rate",        "pool_degen_rate",   "{:.1%}"),
        ("Pool: pair SA precision",      "pool_sa_precision", "{:.1%}"),
        ("Pool: matched pool size",      "pool_matched_size", "{:.0f}"),
        ("Pool: matched S-tier %",       "pool_matched_s_pct", "{:.1%}"),
        ("Pool: matched A-tier %",       "pool_matched_a_pct", "{:.1%}"),
        ("Pool: matched B-tier %",       "pool_matched_b_pct", "{:.1%}"),
    ]

    header = f"  {'Metric':<35}"
    for cn in all_metrics:
        header += f" | {cn:>22}"
    print(header)
    print("  " + "-" * (35 + 25 * len(all_metrics)))

    for name, key, fmt in pattern_metrics:
        row = f"  {name:<35}"
        for cn, m in all_metrics.items():
            v = m[key]
            vs = fmt.format(v)
            row += f" | {vs:>22}"
        print(row)

    # =====================================================================
    # 5. PAIR ECONOMY ANALYSIS
    # =====================================================================
    print("\n\n" + "=" * 120)
    print("  SECTION 5: Pair Economy Deep Dive")
    print("=" * 120)

    print("\n  How pair accumulation works per pattern composition:")
    print(f"  {'Config':<25} {'Avg pairs/draft':>16} {'Home pairs':>12} {'Degen pairs':>13} "
          f"{'Cross pairs':>13} {'Effective P':>13}")
    print("  " + "-" * 95)

    for cn, m in all_metrics.items():
        # Estimate: 85% of picks have 2+ symbols (60% 2-sym + 25% 3-sym)
        avg_pair_picks = NUM_PICKS * 0.85
        home = m["home_pair_rate"] * avg_pair_picks
        degen = m["degenerate_frac"] * avg_pair_picks
        cross = m["cross_arch_feed"] * avg_pair_picks
        # Effective probability P = min(home_count / K, cap)
        eff_p = min(home / PE_K, PE_CAP)
        print(f"  {cn:<25} {avg_pair_picks:>16.1f} {home:>12.1f} {degen:>13.1f} "
              f"{cross:>13.1f} {eff_p:>13.1%}")

    # =====================================================================
    # 6. DEGENERATE PAIR IMPACT ANALYSIS
    # =====================================================================
    print("\n\n" + "=" * 100)
    print("  SECTION 6: Degenerate Pair Impact")
    print("=" * 100)

    print("\n  Degenerate pairs (P,P) like (Tide,Tide) don't map to any archetype.")
    print("  They waste pair-count accumulation and dilute the player's top pair.\n")

    print(f"  {'Config':<25} {'Degen%':>8} {'Late S/A':>10} {'Conv':>8} {'DeckConc':>10} {'HomePair%':>10}")
    print("  " + "-" * 73)
    for cn, m in all_metrics.items():
        print(f"  {cn:<25} {m['degenerate_frac']:>8.1%} {m['late_sa']:>10.2f} "
              f"{m['conv_pick']:>8.1f} {m['deck_conc']:>10.1%} {m['home_pair_rate']:>10.1%}")

    print("\n  Observation: Higher degenerate fraction -> lower home pair rate -> "
          "slower pair escalation -> later convergence.")

    # =====================================================================
    # 7. CROSS-ARCHETYPE FEEDING ANALYSIS
    # =====================================================================
    print("\n\n" + "=" * 100)
    print("  SECTION 7: Cross-Archetype Pair Feeding")
    print("=" * 100)

    print("\n  [S,P] patterns produce the ADJACENT archetype's pair, not the home pair.")
    print("  [P,O] patterns produce cross pairs that may or may not map to an archetype.\n")

    print(f"  {'Config':<25} {'CrossFeed%':>12} {'PairScatter':>13} {'Late S/A':>10} {'Genuine%':>10}")
    print("  " + "-" * 73)
    for cn, m in all_metrics.items():
        print(f"  {cn:<25} {m['cross_arch_feed']:>12.1%} {m['pair_scatter']:>13.1f} "
              f"{m['late_sa']:>10.2f} {m['genuine_choice']:>10.1%}")

    print("\n  Higher cross-feed -> more pair scatter -> diluted escalation "
          "BUT potentially more genuine choices.")

    # =====================================================================
    # 8. PAIR-MATCHED POOL COMPOSITION
    # =====================================================================
    print("\n\n" + "=" * 100)
    print("  SECTION 8: Pair-Matched Pool Composition (What pair-targeted slots show)")
    print("=" * 100)

    print("\n  When a slot is pair-matched, it draws from cards whose ordered pair matches.")
    print("  Ideal: high S/A% in the matched pool.\n")

    print(f"  {'Config':<25} {'Pool Size':>10} {'S-tier%':>9} {'A-tier%':>9} {'B-tier%':>9} {'SA Prec':>9}")
    print("  " + "-" * 73)
    for cn, m in all_metrics.items():
        print(f"  {cn:<25} {m['pool_matched_size']:>10.0f} {m['pool_matched_s_pct']:>9.1%} "
              f"{m['pool_matched_a_pct']:>9.1%} {m['pool_matched_b_pct']:>9.1%} "
              f"{m['pool_sa_precision']:>9.1%}")

    # =====================================================================
    # 9. SUMMARY AND RECOMMENDATIONS
    # =====================================================================
    print("\n\n" + "=" * 100)
    print("  SECTION 9: Summary Ranking")
    print("=" * 100)

    # Rank by number of targets passed
    rankings = []
    for cn, m in all_metrics.items():
        passes = sum(1 for _, key, check in targets if check(m[key]))
        rankings.append((cn, passes, m["late_sa"], m["home_pair_rate"],
                         m["deck_conc"], m["genuine_choice"]))

    rankings.sort(key=lambda x: (x[1], x[2]), reverse=True)

    print(f"\n  {'Rank':>4} {'Config':<25} {'Pass':>6} {'LateSA':>8} {'HomePR':>8} "
          f"{'DeckConc':>9} {'Genuine':>9}")
    print("  " + "-" * 70)
    for rank, (cn, passes, lsa, hpr, dc, gc) in enumerate(rankings, 1):
        print(f"  {rank:>4} {cn:<25} {passes:>4}/10 {lsa:>8.2f} {hpr:>8.1%} "
              f"{dc:>9.1%} {gc:>9.1%}")

    print("\n\nSimulation complete.")
