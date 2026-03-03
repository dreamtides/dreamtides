#!/usr/bin/env python3
"""
Agent 4: Symbol Pattern Composition
Investigates what specific symbol patterns cards should have and how different
patterns affect the token economy and draft decisions under Pack Widening v3.
"""

import random
import math
from dataclasses import dataclass, field
from enum import Enum
from collections import Counter, defaultdict
from typing import Optional

# ---------------------------------------------------------------------------
# Core types
# ---------------------------------------------------------------------------

class Resonance(Enum):
    EMBER = 0
    STONE = 1
    TIDE = 2
    ZEPHYR = 3

# Archetypes arranged on a circle, each with (primary, secondary) resonance
ARCHETYPES = {
    "Flash":        (Resonance.ZEPHYR, Resonance.EMBER),
    "Blink":        (Resonance.EMBER,  Resonance.ZEPHYR),
    "Storm":        (Resonance.EMBER,  Resonance.STONE),
    "SelfDiscard":  (Resonance.STONE,  Resonance.EMBER),
    "SelfMill":     (Resonance.STONE,  Resonance.TIDE),
    "Sacrifice":    (Resonance.TIDE,   Resonance.STONE),
    "Warriors":     (Resonance.TIDE,   Resonance.ZEPHYR),
    "Ramp":         (Resonance.ZEPHYR, Resonance.TIDE),
}

ARCHETYPE_NAMES = list(ARCHETYPES.keys())

# Circle order for adjacency
CIRCLE_ORDER = ["Flash", "Blink", "Storm", "SelfDiscard",
                "SelfMill", "Sacrifice", "Warriors", "Ramp"]

def adjacent_archetypes(arch: str) -> list[str]:
    idx = CIRCLE_ORDER.index(arch)
    prev_idx = (idx - 1) % 8
    next_idx = (idx + 1) % 8
    return [CIRCLE_ORDER[prev_idx], CIRCLE_ORDER[next_idx]]


@dataclass
class SimCard:
    id: int
    symbols: list[Resonance]
    archetype: Optional[str]   # None for generic
    power: float

    def primary_resonance(self) -> Optional[Resonance]:
        return self.symbols[0] if self.symbols else None

    def fitness_for(self, target_arch: str) -> str:
        """Return fitness tier: S, A, B, C, or F."""
        if self.archetype is None:
            return "B"  # generic
        if self.archetype == target_arch:
            return "S"
        if target_arch in adjacent_archetypes(self.archetype):
            # Adjacent AND shares primary resonance
            t_pri = ARCHETYPES[target_arch][0]
            c_pri = ARCHETYPES[self.archetype][0]
            c_sec = ARCHETYPES[self.archetype][1]
            if c_pri == t_pri:
                return "A"
            # Check if card's primary = target's secondary or vice versa
            t_sec = ARCHETYPES[target_arch][1]
            if c_pri == t_sec or c_sec == t_pri:
                return "A"
            return "B"
        # Check shared resonance for B-tier
        t_pri, t_sec = ARCHETYPES[target_arch]
        c_pri = ARCHETYPES[self.archetype][0]
        c_sec = ARCHETYPES[self.archetype][1]
        if c_pri == t_sec or c_sec == t_pri or c_sec == t_sec:
            return "B"
        return "C"

    def is_sa_for(self, target_arch: str) -> bool:
        tier = self.fitness_for(target_arch)
        return tier in ("S", "A")

    def token_profile(self) -> dict[Resonance, int]:
        """Return tokens earned from drafting this card."""
        tokens = defaultdict(int)
        for i, sym in enumerate(self.symbols):
            tokens[sym] += 2 if i == 0 else 1
        return dict(tokens)


# ---------------------------------------------------------------------------
# Symbol pattern types
# ---------------------------------------------------------------------------

# For an archetype with primary P and secondary S, the possible patterns are:
# 0-sym: [] (generic)
# 1-sym: [P], [S]
# 2-sym: [P,S], [P,P], [S,P], [P,O], [S,S], [S,O]
# 3-sym: [P,P,S], [P,S,S], [P,S,P], [P,P,P], [P,S,O], [P,P,O], [P,O,O],
#         [S,P,P], [S,P,S], [S,S,P], etc.
# We'll use symbolic tags: P=primary, S=secondary, O=other (random off-resonance)

def resolve_pattern(pattern: list[str], primary: Resonance, secondary: Resonance) -> list[Resonance]:
    """Resolve a symbolic pattern like ['P','S','O'] into actual resonances."""
    others = [r for r in Resonance if r != primary and r != secondary]
    result = []
    for sym in pattern:
        if sym == 'P':
            result.append(primary)
        elif sym == 'S':
            result.append(secondary)
        elif sym == 'O':
            result.append(random.choice(others))
        else:
            raise ValueError(f"Unknown symbol type: {sym}")
    return result


# ---------------------------------------------------------------------------
# Pool construction
# ---------------------------------------------------------------------------

def build_pool(
    total_cards: int,
    generic_fraction: float,
    symbol_count_dist: dict[int, float],  # {1: 0.2, 2: 0.55, 3: 0.25}
    pattern_weights: dict[str, float],    # e.g. {"P": 0.3, "PS": 0.4, "PP": 0.1, ...}
    config_name: str = "",
) -> list[SimCard]:
    """Build a card pool with the given pattern composition."""

    n_generic = int(total_cards * generic_fraction)
    n_resonance = total_cards - n_generic

    cards_per_arch = n_resonance // 8
    leftover = n_resonance - cards_per_arch * 8

    # Normalize symbol count distribution
    total_w = sum(symbol_count_dist.values())
    sc_dist = {k: v / total_w for k, v in symbol_count_dist.items()}

    # Filter pattern_weights by symbol count and normalize within each count
    patterns_by_count: dict[int, list[tuple[list[str], float]]] = defaultdict(list)
    for pat_str, weight in pattern_weights.items():
        pat_list = list(pat_str) if pat_str else []
        n = len(pat_list)
        patterns_by_count[n].append((pat_list, weight))

    # Normalize weights within each count group
    for n in patterns_by_count:
        total = sum(w for _, w in patterns_by_count[n])
        if total > 0:
            patterns_by_count[n] = [(p, w / total) for p, w in patterns_by_count[n]]

    cards: list[SimCard] = []
    card_id = 0

    for arch_idx, arch_name in enumerate(ARCHETYPE_NAMES):
        pri, sec = ARCHETYPES[arch_name]
        n_arch = cards_per_arch + (1 if arch_idx < leftover else 0)

        # Distribute cards across symbol counts
        count_alloc = {}
        remaining = n_arch
        counts = sorted(sc_dist.keys())
        for i, c in enumerate(counts):
            if i == len(counts) - 1:
                count_alloc[c] = remaining
            else:
                count_alloc[c] = round(n_arch * sc_dist[c])
                remaining -= count_alloc[c]

        for sym_count, n_cards in count_alloc.items():
            if sym_count not in patterns_by_count or not patterns_by_count[sym_count]:
                # Default: all primary
                default_pat = ['P'] * sym_count
                for _ in range(n_cards):
                    symbols = resolve_pattern(default_pat, pri, sec)
                    cards.append(SimCard(
                        id=card_id,
                        symbols=symbols,
                        archetype=arch_name,
                        power=random.uniform(3, 8),
                    ))
                    card_id += 1
            else:
                pats, weights = zip(*patterns_by_count[sym_count])
                for _ in range(n_cards):
                    pat = random.choices(pats, weights=weights, k=1)[0]
                    symbols = resolve_pattern(pat, pri, sec)
                    cards.append(SimCard(
                        id=card_id,
                        symbols=symbols,
                        archetype=arch_name,
                        power=random.uniform(3, 8),
                    ))
                    card_id += 1

    # Generic cards
    for _ in range(n_generic):
        cards.append(SimCard(
            id=card_id,
            symbols=[],
            archetype=None,
            power=random.uniform(4, 9),
        ))
        card_id += 1

    return cards


# ---------------------------------------------------------------------------
# Draft simulation (Pack Widening v3)
# ---------------------------------------------------------------------------

SPEND_COST = 3
BONUS_CARDS = 1
PRIMARY_WEIGHT = 2
SECONDARY_WEIGHT = 1
BASE_PACK = 4
NUM_PICKS = 30

def run_draft(pool: list[SimCard], strategy: str = "committed") -> dict:
    """Run a single 30-pick draft and return detailed metrics."""
    tokens = {r: 0 for r in Resonance}
    drafted: list[SimCard] = []
    pick_data: list[dict] = []
    early_packs: list[list[SimCard]] = []  # store early packs for retroactive eval

    # Pre-index cards by primary resonance
    by_primary: dict[Resonance, list[SimCard]] = defaultdict(list)
    for c in pool:
        pr = c.primary_resonance()
        if pr is not None:
            by_primary[pr].append(c)

    target_arch: Optional[str] = None
    commitment_pick: Optional[int] = None

    for pick_num in range(1, NUM_PICKS + 1):
        # Decide spending
        spent_resonance: Optional[Resonance] = None
        if target_arch is not None:
            t_pri = ARCHETYPES[target_arch][0]
            # Spend on primary resonance if possible
            if tokens[t_pri] >= SPEND_COST:
                spent_resonance = t_pri
                tokens[t_pri] -= SPEND_COST

        # Draw base pack
        pack = random.sample(pool, BASE_PACK)

        # Add bonus card if spending
        if spent_resonance is not None:
            bonus_pool = by_primary[spent_resonance]
            if bonus_pool:
                bonus = random.choice(bonus_pool)
                pack.append(bonus)

        # Save early packs for retroactive evaluation
        if pick_num <= 5:
            early_packs.append(list(pack))

        # Pick a card
        if strategy == "committed" and target_arch is not None:
            # Pick best S/A card, else best power
            sa_cards = [c for c in pack if c.is_sa_for(target_arch)]
            if sa_cards:
                chosen = max(sa_cards, key=lambda c: (
                    0 if c.fitness_for(target_arch) == "S" else 1,
                    -c.power
                ))
            else:
                chosen = max(pack, key=lambda c: c.power)
        elif strategy == "power":
            chosen = max(pack, key=lambda c: c.power)
        else:
            # Early: pick highest power among S/A of any archetype
            chosen = max(pack, key=lambda c: c.power)

        # Record token profile of chosen card
        chosen_profile = chosen.token_profile()

        # Count S/A cards per archetype in pack
        sa_by_arch = defaultdict(int)
        for c in pack:
            for arch in ARCHETYPE_NAMES:
                if c.is_sa_for(arch):
                    sa_by_arch[arch] += 1

        # Unique archetypes with S/A cards in pack
        archs_with_sa = sum(1 for a in ARCHETYPE_NAMES if sa_by_arch[a] > 0)

        # Token profiles of S/A cards in pack (for genuine choice measurement)
        sa_profiles = []
        if target_arch:
            for c in pack:
                if c.is_sa_for(target_arch):
                    sa_profiles.append(tuple(sorted(c.token_profile().items(), key=lambda x: x[0].value)))

        unique_profiles = len(set(sa_profiles))

        # Off-archetype cards
        off_arch = 0
        if target_arch:
            for c in pack:
                tier = c.fitness_for(target_arch)
                if tier in ("C", "F"):
                    off_arch += 1

        # S/A for target
        sa_for_target = 0
        if target_arch:
            sa_for_target = sum(1 for c in pack if c.is_sa_for(target_arch))

        pick_data.append({
            "pick": pick_num,
            "spent": spent_resonance is not None,
            "spent_resonance": spent_resonance,
            "chosen_profile": chosen_profile,
            "archs_with_sa": archs_with_sa,
            "sa_for_target": sa_for_target,
            "off_arch": off_arch,
            "unique_sa_profiles": unique_profiles,
            "pack_size": len(pack),
            "chosen_arch": chosen.archetype,
            "chosen_fitness": chosen.fitness_for(target_arch) if target_arch else None,
        })

        drafted.append(chosen)

        # Earn tokens
        for i, sym in enumerate(chosen.symbols):
            tokens[sym] += PRIMARY_WEIGHT if i == 0 else SECONDARY_WEIGHT

        # Commitment logic: commit around pick 5-6 based on drafted cards
        if target_arch is None and pick_num >= 5:
            arch_scores = defaultdict(float)
            for c in drafted:
                for arch in ARCHETYPE_NAMES:
                    tier = c.fitness_for(arch)
                    if tier == "S":
                        arch_scores[arch] += 2.0
                    elif tier == "A":
                        arch_scores[arch] += 1.5
                    elif tier == "B":
                        arch_scores[arch] += 0.5
            if arch_scores:
                target_arch = max(arch_scores, key=arch_scores.get)
                commitment_pick = pick_num

    # Retroactively evaluate early packs now that we know target_arch
    early_sa_retro = []
    if target_arch:
        for pack in early_packs:
            sa_count_early = sum(1 for c in pack if c.is_sa_for(target_arch))
            early_sa_retro.append(sa_count_early)

    # Compute final deck stats
    if target_arch:
        sa_count = sum(1 for c in drafted if c.is_sa_for(target_arch))
        sa_fraction = sa_count / len(drafted)
    else:
        sa_fraction = 0.0
        sa_count = 0

    return {
        "target_arch": target_arch,
        "commitment_pick": commitment_pick,
        "drafted": drafted,
        "pick_data": pick_data,
        "tokens": tokens,
        "sa_fraction": sa_fraction,
        "sa_count": sa_count,
        "early_sa_retro": early_sa_retro,
    }


# ---------------------------------------------------------------------------
# Pattern composition configurations
# ---------------------------------------------------------------------------

# Symbol count dist: 20% 1-sym, 55% 2-sym, 25% 3-sym (the V4 default)
DEFAULT_SC_DIST = {1: 0.20, 2: 0.55, 3: 0.25}

CONFIGS = {
    # Config A: Uniform [P,S] -- all 2-sym cards use [P,S] pattern
    "A_Uniform_PS": {
        "desc": "All cards use [P,S] pattern (1-sym: [P]; 3-sym: [P,P,S])",
        "sc_dist": DEFAULT_SC_DIST,
        "patterns": {
            "P": 1.0,
            "PS": 1.0,
            "PPS": 1.0,
        },
    },
    # Config B: Primary-heavy -- all patterns maximize primary resonance
    "B_Primary_Heavy": {
        "desc": "Maximize primary tokens: [P], [P,P], [P,P,P]",
        "sc_dist": DEFAULT_SC_DIST,
        "patterns": {
            "P": 1.0,
            "PP": 1.0,
            "PPP": 1.0,
        },
    },
    # Config C: Balanced variety -- mix of primary-heavy and spread patterns
    "C_Balanced_Variety": {
        "desc": "Mix: 1-sym [P]; 2-sym 40%[P,S] 30%[P,P] 30%[P,O]; 3-sym 40%[P,P,S] 30%[P,S,O] 30%[P,S,S]",
        "sc_dist": DEFAULT_SC_DIST,
        "patterns": {
            "P": 1.0,
            "PS": 0.40, "PP": 0.30, "PO": 0.30,
            "PPS": 0.40, "PSO": 0.30, "PSS": 0.30,
        },
    },
    # Config D: Maximum variety -- every distinct pattern represented
    "D_Max_Variety": {
        "desc": "Every pattern type equally weighted",
        "sc_dist": DEFAULT_SC_DIST,
        "patterns": {
            "P": 0.6, "S": 0.4,
            "PS": 0.20, "PP": 0.20, "SP": 0.15, "PO": 0.20, "SS": 0.10, "SO": 0.15,
            "PPS": 0.15, "PSS": 0.15, "PSP": 0.10, "PPP": 0.05, "PSO": 0.15,
            "PPO": 0.10, "POO": 0.10, "SPP": 0.05, "SPS": 0.05, "SSP": 0.05,
            "SPO": 0.05,
        },
    },
    # Config E: Bridge-optimized -- many [P,O] and [P,S,O] patterns for cross-resonance tokens
    "E_Bridge_Optimized": {
        "desc": "Heavy bridge patterns: [P,O] and [P,S,O] emphasized for multi-resonance tokens",
        "sc_dist": DEFAULT_SC_DIST,
        "patterns": {
            "P": 0.7, "S": 0.3,
            "PS": 0.20, "PO": 0.50, "PP": 0.10, "SP": 0.10, "SO": 0.10,
            "PSO": 0.40, "POO": 0.30, "PPS": 0.15, "PSS": 0.15,
        },
    },
    # Config F: Secondary-spread -- lots of S-resonance tokens for B-tier connections
    "F_Secondary_Spread": {
        "desc": "More secondary: [S,P], [S,S], [P,S,S]; enables wider B-tier network",
        "sc_dist": DEFAULT_SC_DIST,
        "patterns": {
            "P": 0.5, "S": 0.5,
            "SP": 0.40, "SS": 0.20, "PS": 0.20, "PP": 0.10, "SO": 0.10,
            "PSS": 0.30, "SPP": 0.20, "SPS": 0.20, "PPS": 0.15, "SSP": 0.15,
        },
    },
    # Config G: Concentrated + bridge hybrid
    "G_Concentrated_Bridge": {
        "desc": "Mix of concentrated [P,P] and bridge [P,O]: fast primary tokens + bridge option",
        "sc_dist": DEFAULT_SC_DIST,
        "patterns": {
            "P": 1.0,
            "PP": 0.45, "PO": 0.35, "PS": 0.20,
            "PPP": 0.20, "PPO": 0.30, "PPS": 0.25, "PSO": 0.25,
        },
    },
}


# ---------------------------------------------------------------------------
# Running experiments
# ---------------------------------------------------------------------------

def run_experiment(config_name: str, config: dict, n_drafts: int = 1000) -> dict:
    """Run n_drafts for a given configuration and aggregate metrics."""
    pool = build_pool(
        total_cards=360,
        generic_fraction=0.10,
        symbol_count_dist=config["sc_dist"],
        pattern_weights=config["patterns"],
        config_name=config_name,
    )

    all_results = []
    for _ in range(n_drafts):
        result = run_draft(pool, strategy="committed")
        all_results.append(result)

    # Aggregate metrics
    metrics = aggregate_metrics(all_results, config_name)
    return metrics


def aggregate_metrics(results: list[dict], config_name: str) -> dict:
    """Compute all required metrics from draft results."""

    n = len(results)

    # -- Early picks (1-5) --
    early_archs_with_sa = []
    early_sa_for_emerging = []

    # -- Late picks (6+) --
    late_sa_for_target = []
    late_off_arch = []
    late_sa_values_per_draft = []  # for stddev

    # -- Token profiles --
    genuine_choice_rates = []
    token_scatter_rates = []
    spend_rates = []
    first_spend_picks = []

    # -- Commitment and deck quality --
    commitment_picks = []
    sa_fractions = []
    arch_freq = Counter()

    # -- Bridge viability --
    bridge_token_counts = []  # how many tokens earned in off-primary resonances

    for result in results:
        target = result["target_arch"]
        if target is None:
            continue

        arch_freq[target] += 1
        if result["commitment_pick"]:
            commitment_picks.append(result["commitment_pick"])
        sa_fractions.append(result["sa_fraction"])

        t_pri = ARCHETYPES[target][0]
        t_sec = ARCHETYPES[target][1]

        draft_late_sa = []
        draft_has_first_spend = False

        for pd in result["pick_data"]:
            pick = pd["pick"]

            if pick <= 5:
                early_archs_with_sa.append(pd["archs_with_sa"])
            else:
                late_sa_for_target.append(pd["sa_for_target"])
                late_off_arch.append(pd["off_arch"])
                draft_late_sa.append(pd["sa_for_target"])

                if pd["spent"]:
                    spend_rates.append(1)
                else:
                    spend_rates.append(0)

                # Genuine choice: 2+ S/A cards with different token profiles
                if pd["unique_sa_profiles"] >= 2:
                    genuine_choice_rates.append(1)
                else:
                    genuine_choice_rates.append(0)

            # Token scatter: tokens earned in non-target resonances
            profile = pd["chosen_profile"]
            if profile:
                total_tokens = sum(profile.values())
                off_tokens = sum(v for r, v in profile.items() if r != t_pri and r != t_sec)
                if total_tokens > 0:
                    token_scatter_rates.append(off_tokens / total_tokens)

            # First spend
            if pd["spent"] and not draft_has_first_spend:
                first_spend_picks.append(pick)
                draft_has_first_spend = True

        if draft_late_sa:
            late_sa_values_per_draft.append(draft_late_sa)

        # Retroactive early SA: evaluate early packs against final target_arch
        for sa_val in result.get("early_sa_retro", []):
            early_sa_for_emerging.append(sa_val)

        # Bridge tokens
        final_tokens = result["tokens"]
        off_pri_tokens = sum(v for r, v in final_tokens.items() if r != t_pri)
        bridge_token_counts.append(off_pri_tokens)

    # Compute aggregates
    def mean(xs):
        return sum(xs) / len(xs) if xs else 0

    def stddev(xs):
        if len(xs) < 2:
            return 0
        m = mean(xs)
        return math.sqrt(sum((x - m) ** 2 for x in xs) / (len(xs) - 1))

    # StdDev of S/A per pack (picks 6+) -- compute per-draft stddev, then average
    per_draft_stddevs = []
    for draft_sa in late_sa_values_per_draft:
        if len(draft_sa) >= 2:
            per_draft_stddevs.append(stddev(draft_sa))

    # Overall stddev of late S/A values (pooled across all drafts)
    pooled_sa_stddev = stddev(late_sa_for_target) if len(late_sa_for_target) >= 2 else 0

    # Archetype frequency
    total_committed = sum(arch_freq.values())
    arch_freq_pct = {a: arch_freq[a] / total_committed * 100 if total_committed > 0 else 0
                     for a in ARCHETYPE_NAMES}

    # Card overlap (run-to-run variety)
    # Compare pairs of drafts that committed to the same archetype
    overlap_rates = []
    drafts_by_arch = defaultdict(list)
    for r in results:
        if r["target_arch"]:
            drafts_by_arch[r["target_arch"]].append(set(c.id for c in r["drafted"]))

    for arch, draft_sets in drafts_by_arch.items():
        # Sample pairs
        for i in range(min(50, len(draft_sets))):
            for j in range(i + 1, min(i + 3, len(draft_sets))):
                overlap = len(draft_sets[i] & draft_sets[j])
                overlap_rate = overlap / 30.0
                overlap_rates.append(overlap_rate)

    # Accidental commitment: how often does picking S/A cards push tokens
    # toward a resonance the player doesn't intend to spend on?
    # Approximate: fraction of total tokens in non-primary resonance
    accidental_commitment_rates = []
    for result in results:
        target = result["target_arch"]
        if not target:
            continue
        t_pri = ARCHETYPES[target][0]
        final = result["tokens"]
        total = sum(final.values())
        if total > 0:
            # Fraction of tokens NOT in primary or secondary
            t_sec = ARCHETYPES[target][1]
            wasted = sum(v for r, v in final.items() if r != t_pri and r != t_sec)
            accidental_commitment_rates.append(wasted / total)

    return {
        "config": config_name,
        "early_archs_sa": mean(early_archs_with_sa),
        "early_sa_emerging": mean(early_sa_for_emerging),
        "late_sa_target": mean(late_sa_for_target),
        "late_off_arch": mean(late_off_arch),
        "convergence_pick": mean(commitment_picks),
        "sa_fraction": mean(sa_fractions),
        "sa_stddev_pooled": pooled_sa_stddev,
        "sa_stddev_avg_per_draft": mean(per_draft_stddevs),
        "genuine_choice_rate": mean(genuine_choice_rates),
        "token_scatter_rate": mean(token_scatter_rates),
        "spend_rate_late": mean(spend_rates),
        "first_spend_pick": mean(first_spend_picks),
        "card_overlap": mean(overlap_rates),
        "arch_freq_pct": arch_freq_pct,
        "bridge_tokens_avg": mean(bridge_token_counts),
        "accidental_commitment_rate": mean(accidental_commitment_rates),
    }


# ---------------------------------------------------------------------------
# Token profile analysis (per-pattern)
# ---------------------------------------------------------------------------

def analyze_token_profiles():
    """Enumerate distinct patterns and show their token profiles."""
    print("=" * 70)
    print("TOKEN PROFILES PER PATTERN (for archetype with P=Tide, S=Zephyr)")
    print("=" * 70)

    patterns = {
        "1-sym [P]":    ["P"],
        "1-sym [S]":    ["S"],
        "2-sym [P,S]":  ["P", "S"],
        "2-sym [P,P]":  ["P", "P"],
        "2-sym [S,P]":  ["S", "P"],
        "2-sym [P,O]":  ["P", "O"],
        "3-sym [P,P,S]": ["P", "P", "S"],
        "3-sym [P,S,S]": ["P", "S", "S"],
        "3-sym [P,S,O]": ["P", "S", "O"],
        "3-sym [P,P,O]": ["P", "P", "O"],
        "3-sym [P,O,O]": ["P", "O", "O"],
    }

    # Use Warriors as example: P=Tide, S=Zephyr
    pri = Resonance.TIDE
    sec = Resonance.ZEPHYR

    print(f"\n{'Pattern':<20} {'Pri tokens':>10} {'Sec tokens':>10} {'Other tokens':>12} {'Total':>6}")
    print("-" * 60)

    for name, pat in patterns.items():
        # Compute token profile
        tok = defaultdict(int)
        for i, sym in enumerate(pat):
            if sym == 'P':
                tok[pri] += PRIMARY_WEIGHT if i == 0 else SECONDARY_WEIGHT
            elif sym == 'S':
                tok[sec] += PRIMARY_WEIGHT if i == 0 else SECONDARY_WEIGHT
            elif sym == 'O':
                tok["other"] += PRIMARY_WEIGHT if i == 0 else SECONDARY_WEIGHT

        p_tok = tok.get(pri, 0)
        s_tok = tok.get(sec, 0)
        o_tok = tok.get("other", 0)
        total = p_tok + s_tok + o_tok
        print(f"{name:<20} {p_tok:>10} {s_tok:>10} {o_tok:>12} {total:>6}")

    print()


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    random.seed(42)

    analyze_token_profiles()

    print("=" * 70)
    print("RUNNING DRAFT SIMULATIONS (1000 drafts per config)")
    print("=" * 70)

    all_metrics = {}
    for config_name, config in CONFIGS.items():
        print(f"\n  Running {config_name}: {config['desc']}")
        metrics = run_experiment(config_name, config, n_drafts=1000)
        all_metrics[config_name] = metrics

    # Print results
    print("\n" + "=" * 70)
    print("RESULTS SUMMARY")
    print("=" * 70)

    header = (f"{'Config':<25} {'EarlySA':>8} {'EarlyArch':>10} {'LateSA':>7} "
              f"{'OffArch':>8} {'SAStdDev':>8} {'GChoice':>8} {'Scatter':>8} "
              f"{'SpendR':>7} {'1stSpnd':>7} {'SAFrac':>7} {'Overlap':>8} "
              f"{'Bridge':>7} {'AccComm':>8}")
    print(header)
    print("-" * len(header))

    for name, m in all_metrics.items():
        print(f"{name:<25} "
              f"{m['early_sa_emerging']:>8.2f} "
              f"{m['early_archs_sa']:>10.2f} "
              f"{m['late_sa_target']:>7.2f} "
              f"{m['late_off_arch']:>8.2f} "
              f"{m['sa_stddev_pooled']:>8.2f} "
              f"{m['genuine_choice_rate']:>8.3f} "
              f"{m['token_scatter_rate']:>8.3f} "
              f"{m['spend_rate_late']:>7.3f} "
              f"{m['first_spend_pick']:>7.1f} "
              f"{m['sa_fraction']:>7.3f} "
              f"{m['card_overlap']:>8.3f} "
              f"{m['bridge_tokens_avg']:>7.1f} "
              f"{m['accidental_commitment_rate']:>8.3f}")

    # Print targets comparison
    print("\n" + "=" * 70)
    print("TARGET COMPARISON")
    print("=" * 70)

    targets = {
        "Early archs w/ SA (>=3)": ("early_archs_sa", 3.0, ">="),
        "Early SA emerging (<=2)": ("early_sa_emerging", 2.0, "<="),
        "Late SA target (>=2)": ("late_sa_target", 2.0, ">="),
        "Late off-arch (>=0.5)": ("late_off_arch", 0.5, ">="),
        "Convergence pick (5-8)": ("convergence_pick", (5, 8), "range"),
        "SA fraction (60-90%)": ("sa_fraction", (0.60, 0.90), "range"),
        "SA StdDev late (>=0.8)": ("sa_stddev_pooled", 0.8, ">="),
        "Card overlap (<40%)": ("card_overlap", 0.40, "<"),
    }

    print(f"\n{'Metric':<30}", end="")
    for name in all_metrics:
        short = name[:12]
        print(f" {short:>14}", end="")
    print()
    print("-" * (30 + 15 * len(all_metrics)))

    for tname, (key, target, op) in targets.items():
        print(f"{tname:<30}", end="")
        for name, m in all_metrics.items():
            val = m[key]
            if op == ">=":
                hit = val >= target
            elif op == "<=":
                hit = val <= target
            elif op == "<":
                hit = val < target
            elif op == "range":
                hit = target[0] <= val <= target[1]
            mark = "PASS" if hit else "FAIL"
            if isinstance(val, float):
                print(f" {val:>8.2f} {mark:>4}", end="")
            print(end="")
        print()

    # Re-print with formatted values
    print(f"\n{'Metric':<30} {'Target':<12}", end="")
    for name in all_metrics:
        print(f" {name[:14]:>14}", end="")
    print()
    print("-" * (42 + 15 * len(all_metrics)))

    for tname, (key, target, op) in targets.items():
        if op == "range":
            tstr = f"{target[0]}-{target[1]}"
        elif op == ">=":
            tstr = f">={target}"
        elif op == "<=":
            tstr = f"<={target}"
        elif op == "<":
            tstr = f"<{target}"
        else:
            tstr = str(target)
        print(f"{tname:<30} {tstr:<12}", end="")
        for name, m in all_metrics.items():
            val = m[key]
            if op == ">=":
                hit = val >= target
            elif op == "<=":
                hit = val <= target
            elif op == "<":
                hit = val < target
            elif op == "range":
                hit = target[0] <= val <= target[1]
            mark = " ok" if hit else "  X"
            print(f" {val:>10.3f}{mark}", end="")
        print()

    # Archetype frequency
    print("\n" + "=" * 70)
    print("ARCHETYPE FREQUENCY (target: 5%-20% each)")
    print("=" * 70)
    for name, m in all_metrics.items():
        print(f"\n  {name}:")
        freqs = m["arch_freq_pct"]
        for arch in ARCHETYPE_NAMES:
            pct = freqs[arch]
            mark = "ok" if 5 <= pct <= 20 else "X"
            print(f"    {arch:<15} {pct:5.1f}% {mark}")

    # Detailed per-config token economy analysis
    print("\n" + "=" * 70)
    print("TOKEN ECONOMY COMPARISON")
    print("=" * 70)
    print(f"\n{'Config':<25} {'Genuine':>8} {'Scatter':>8} {'SpendRate':>9} "
          f"{'1stSpend':>8} {'BridgeTok':>9} {'AccComm':>8}")
    print("-" * 76)
    for name, m in all_metrics.items():
        print(f"{name:<25} "
              f"{m['genuine_choice_rate']:>8.1%} "
              f"{m['token_scatter_rate']:>8.1%} "
              f"{m['spend_rate_late']:>9.1%} "
              f"{m['first_spend_pick']:>8.1f} "
              f"{m['bridge_tokens_avg']:>9.1f} "
              f"{m['accidental_commitment_rate']:>8.1%}")

    print("\n" + "=" * 70)
    print("KEY FINDINGS")
    print("=" * 70)

    # Sort configs by genuine choice rate
    by_choice = sorted(all_metrics.items(), key=lambda x: x[1]["genuine_choice_rate"], reverse=True)
    print(f"\nBest genuine choice rate: {by_choice[0][0]} ({by_choice[0][1]['genuine_choice_rate']:.1%})")
    print(f"Worst genuine choice rate: {by_choice[-1][0]} ({by_choice[-1][1]['genuine_choice_rate']:.1%})")

    by_scatter = sorted(all_metrics.items(), key=lambda x: x[1]["token_scatter_rate"])
    print(f"\nLowest token scatter: {by_scatter[0][0]} ({by_scatter[0][1]['token_scatter_rate']:.1%})")
    print(f"Highest token scatter: {by_scatter[-1][0]} ({by_scatter[-1][1]['token_scatter_rate']:.1%})")

    by_bridge = sorted(all_metrics.items(), key=lambda x: x[1]["bridge_tokens_avg"], reverse=True)
    print(f"\nBest bridge viability: {by_bridge[0][0]} (avg {by_bridge[0][1]['bridge_tokens_avg']:.1f} off-primary tokens)")

    by_sa = sorted(all_metrics.items(), key=lambda x: x[1]["late_sa_target"], reverse=True)
    print(f"\nBest late S/A: {by_sa[0][0]} ({by_sa[0][1]['late_sa_target']:.2f})")

    # Count how many targets each config hits
    print("\n\nTargets hit per config:")
    for name, m in all_metrics.items():
        hits = 0
        for tname, (key, target, op) in targets.items():
            val = m[key]
            if op == ">=":
                hit = val >= target
            elif op == "<=":
                hit = val <= target
            elif op == "<":
                hit = val < target
            elif op == "range":
                hit = target[0] <= val <= target[1]
            if hit:
                hits += 1
        print(f"  {name:<25} {hits}/{len(targets)}")

    print("\nDone.")


if __name__ == "__main__":
    main()
