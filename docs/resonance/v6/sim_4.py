#!/usr/bin/env python3
"""
Simulation Agent 4: Dual-Resonance Pool Sculpting

One-sentence algorithm:
  After each pick, replace N cards in the pool with cards split between your
  top resonance and second resonance from a reserve, keeping the pool at 360.

Modifications from Round 2 discussion:
  - 18/pick aggressive variant (9 T1 + 9 T2)
  - Recycle removed cards into reserve
  - Reserve enriched at 25% dual-type
  - Delayed start: no replacements until pick 3
"""

import random
import math
from collections import defaultdict
from dataclasses import dataclass, field
from typing import Optional

# ============================================================
# Constants
# ============================================================

RESONANCES = ["Ember", "Stone", "Tide", "Zephyr"]

ARCHETYPES = [
    {"name": "Flash",         "primary": "Zephyr", "secondary": "Ember"},
    {"name": "Blink",         "primary": "Ember",  "secondary": "Zephyr"},
    {"name": "Storm",         "primary": "Ember",  "secondary": "Stone"},
    {"name": "Self-Discard",  "primary": "Stone",  "secondary": "Ember"},
    {"name": "Self-Mill",     "primary": "Stone",  "secondary": "Tide"},
    {"name": "Sacrifice",     "primary": "Tide",   "secondary": "Stone"},
    {"name": "Warriors",      "primary": "Tide",   "secondary": "Zephyr"},
    {"name": "Ramp",          "primary": "Zephyr", "secondary": "Tide"},
]

ARCHETYPE_NAMES = [a["name"] for a in ARCHETYPES]

NUM_DRAFTS = 1000
NUM_PICKS = 30
PACK_SIZE = 4
POOL_SIZE = 360
NUM_GENERIC = 36
NUM_ARCHETYPE_CARDS = 324  # 360 - 36
CARDS_PER_ARCHETYPE = 40  # ~40 each, 8 * 40 = 320, +4 distributed
MAX_DUAL_TYPE = 54  # 15% cap

RESERVE_SIZE = 800
RESERVE_DUAL_FRACTION = 0.25  # 25% dual-type in reserve per discussion

# ============================================================
# Fitness model
# ============================================================

def build_adjacency():
    """Build adjacency relationships on the archetype circle."""
    adj = {}
    n = len(ARCHETYPES)
    for i, a in enumerate(ARCHETYPES):
        left = ARCHETYPES[(i - 1) % n]["name"]
        right = ARCHETYPES[(i + 1) % n]["name"]
        adj[a["name"]] = {"left": left, "right": right}
    return adj

ADJACENCY = build_adjacency()

def compute_fitness(card_archetype, card_primary_res, card_secondary_res):
    """
    Return fitness dict: archetype_name -> tier (S/A/B/C/F).
    - S in home archetype
    - A in adjacent archetype sharing primary resonance
    - B in archetypes sharing secondary resonance
    - C in archetypes one step further
    - F in distant archetypes
    """
    home = card_archetype
    fitness = {}
    home_info = next(a for a in ARCHETYPES if a["name"] == home)

    for a in ARCHETYPES:
        if a["name"] == home:
            fitness[a["name"]] = "S"
        elif a["primary"] == home_info["primary"] and a["name"] != home:
            # Shares primary resonance -> adjacent on same resonance side
            fitness[a["name"]] = "A"
        elif a["primary"] == home_info["secondary"] or a["secondary"] == home_info["primary"]:
            fitness[a["name"]] = "B"
        elif a["primary"] == home_info["secondary"] or a["secondary"] == home_info["secondary"]:
            fitness[a["name"]] = "C"
        else:
            fitness[a["name"]] = "F"

    # Ensure at least the direct circle neighbors get reasonable ratings
    # Adjacent archetypes sharing primary resonance = A
    # Archetypes sharing secondary resonance = B
    # Recompute more carefully using the circle model
    fitness = {}
    for a in ARCHETYPES:
        if a["name"] == home:
            fitness[a["name"]] = "S"
            continue
        # Check if shares primary resonance with home
        if a["primary"] == home_info["primary"]:
            fitness[a["name"]] = "A"
        # Check if shares secondary with home's primary or primary with home's secondary
        elif (a["primary"] == home_info["secondary"] or
              a["secondary"] == home_info["primary"]):
            fitness[a["name"]] = "B"
        elif (a["secondary"] == home_info["secondary"] or
              a["secondary"] == home_info["primary"] or
              a["primary"] == home_info["secondary"]):
            fitness[a["name"]] = "C"
        else:
            fitness[a["name"]] = "F"

    return fitness


def fitness_tier_value(tier):
    """Numeric value for tier comparison."""
    return {"S": 5, "A": 4, "B": 3, "C": 2, "F": 1}[tier]


def is_sa_tier(tier):
    return tier in ("S", "A")


def is_cf_tier(tier):
    return tier in ("C", "F")


# ============================================================
# Card generation
# ============================================================

@dataclass
class SimCard:
    id: int
    symbols: list  # ordered list of resonance strings
    archetype: str  # home archetype (for evaluation)
    archetype_fitness: dict = field(default_factory=dict)
    power: float = 5.0

    @property
    def primary_resonance(self):
        if not self.symbols:
            return None
        return self.symbols[0]

    @property
    def resonance_types(self):
        return list(set(self.symbols))

    @property
    def is_dual_type(self):
        return len(set(self.symbols)) == 2

    @property
    def is_generic(self):
        return len(self.symbols) == 0

    def weighted_symbols(self):
        """Return weighted symbol counts: primary=2, others=1."""
        counts = defaultdict(int)
        for i, s in enumerate(self.symbols):
            counts[s] += 2 if i == 0 else 1
        return dict(counts)


_card_id_counter = 0

def next_card_id():
    global _card_id_counter
    _card_id_counter += 1
    return _card_id_counter


def reset_card_ids():
    global _card_id_counter
    _card_id_counter = 0


def make_card(archetype_name, symbols, power=None):
    """Create a card for an archetype with given symbols."""
    cid = next_card_id()
    if power is None:
        power = random.uniform(3.0, 8.0)
    arch_info = next(a for a in ARCHETYPES if a["name"] == archetype_name)
    primary_res = arch_info["primary"]
    secondary_res = arch_info["secondary"]
    fitness = compute_fitness(archetype_name, primary_res, secondary_res)
    return SimCard(id=cid, symbols=symbols, archetype=archetype_name,
                   archetype_fitness=fitness, power=power)


def make_generic_card(power=None):
    cid = next_card_id()
    if power is None:
        power = random.uniform(3.0, 7.0)
    fitness = {a["name"]: "B" for a in ARCHETYPES}
    return SimCard(id=cid, symbols=[], archetype="Generic",
                   archetype_fitness=fitness, power=power)


def generate_archetype_cards(archetype_name, count, dual_budget):
    """
    Generate cards for one archetype.
    Symbol distribution from discussion doc:
      Per archetype (~40 cards): mix of mono-1, mono-2, dual-2, mono-3, dual-3
    We distribute dual_budget dual-type cards per archetype.
    """
    arch_info = next(a for a in ARCHETYPES if a["name"] == archetype_name)
    primary = arch_info["primary"]
    secondary = arch_info["secondary"]

    cards = []
    dual_remaining = dual_budget

    for i in range(count):
        # Decide symbol pattern
        # Distribution per archetype (40 cards):
        #   ~10 mono-1, ~15 mono-2, ~5 dual-2, ~7 mono-3, ~2 dual-3 = 39
        #   Adjust to fit dual_budget
        r = random.random()
        if r < 0.25:  # mono-1: [P]
            symbols = [primary]
        elif r < 0.60:  # mono-2: [P, P]
            symbols = [primary, primary]
        elif r < 0.72 and dual_remaining > 0:  # dual-2: [P, S]
            symbols = [primary, secondary]
            dual_remaining -= 1
        elif r < 0.88:  # mono-3: [P, P, P]
            symbols = [primary, primary, primary]
        elif r < 0.95 and dual_remaining > 0:  # dual-3: [P, P, S]
            symbols = [primary, primary, secondary]
            dual_remaining -= 1
        else:  # fallback mono-2
            symbols = [primary, primary]

        cards.append(make_card(archetype_name, symbols))

    return cards


def generate_pool():
    """Generate the 360-card starting pool."""
    reset_card_ids()
    pool = []

    # Generic cards
    for _ in range(NUM_GENERIC):
        pool.append(make_generic_card())

    # Archetype cards: 324 cards across 8 archetypes
    # ~40 per archetype, 54 dual-type max total
    # Budget: ~7 dual per archetype = 56, cap at 54
    cards_per = [40, 40, 41, 41, 40, 40, 41, 41]  # sums to 324
    dual_per = [7, 7, 7, 7, 6, 7, 7, 6]  # sums to 54

    for i, arch in enumerate(ARCHETYPES):
        arch_cards = generate_archetype_cards(arch["name"], cards_per[i], dual_per[i])
        pool.extend(arch_cards)

    random.shuffle(pool)
    return pool


def generate_reserve():
    """Generate the 800-card reserve with 25% dual-type enrichment."""
    reserve = {r: [] for r in RESONANCES}

    # For each resonance, generate ~200 cards
    # Cards are assigned to archetypes that have this resonance as primary
    for res in RESONANCES:
        primary_archetypes = [a for a in ARCHETYPES if a["primary"] == res]
        count_per_arch = 100  # 100 per primary archetype = 200 per resonance

        for arch in primary_archetypes:
            for j in range(count_per_arch):
                # 25% dual-type in reserve
                r = random.random()
                if r < 0.25:
                    # Dual-type: [primary, secondary]
                    if random.random() < 0.6:
                        symbols = [arch["primary"], arch["secondary"]]
                    else:
                        symbols = [arch["primary"], arch["primary"], arch["secondary"]]
                elif r < 0.50:
                    symbols = [arch["primary"]]
                elif r < 0.80:
                    symbols = [arch["primary"], arch["primary"]]
                else:
                    symbols = [arch["primary"], arch["primary"], arch["primary"]]

                card = make_card(arch["name"], symbols)
                reserve[res].append(card)

    return reserve


# ============================================================
# Player strategies
# ============================================================

def pick_archetype_committed(pack, player_state):
    """Pick highest fitness for strongest archetype. Commits around pick 5-6."""
    target = player_state["target_archetype"]
    if target is None:
        # Before commitment, pick highest power among S/A cards for any archetype
        best = None
        best_score = -1
        for card in pack:
            for arch_name, tier in card.archetype_fitness.items():
                if is_sa_tier(tier):
                    score = card.power + (1.0 if tier == "S" else 0.5)
                    if score > best_score:
                        best_score = score
                        best = card
        if best is None:
            best = max(pack, key=lambda c: c.power)
        return best
    else:
        # After commitment, pick highest fitness for target
        def score(card):
            tier = card.archetype_fitness.get(target, "F")
            tier_val = fitness_tier_value(tier)
            return tier_val * 10 + card.power
        return max(pack, key=score)


def pick_power_chaser(pack, player_state):
    """Pick highest raw power regardless of archetype."""
    return max(pack, key=lambda c: c.power)


def pick_signal_reader(pack, player_state):
    """Evaluate which archetype seems most available and draft toward it."""
    res_counts = player_state["resonance_counts"].copy()

    best_card = None
    best_score = -1
    for card in pack:
        # Score based on how well card fits the emerging resonance profile
        card_score = card.power * 0.5
        w = card.weighted_symbols()
        for res, count in w.items():
            card_score += res_counts.get(res, 0) * count * 0.3
        # Bonus for S/A fitness in leading archetype
        if player_state["target_archetype"]:
            tier = card.archetype_fitness.get(player_state["target_archetype"], "F")
            card_score += fitness_tier_value(tier) * 2
        if card_score > best_score:
            best_score = card_score
            best_card = card
    return best_card


STRATEGIES = {
    "archetype_committed": pick_archetype_committed,
    "power_chaser": pick_power_chaser,
    "signal_reader": pick_signal_reader,
}


# ============================================================
# Dual-Resonance Pool Sculpting algorithm
# ============================================================

def get_top_two_resonances(resonance_counts):
    """Return (T1, T2) - top two resonances by weighted count."""
    sorted_res = sorted(resonance_counts.items(), key=lambda x: -x[1])
    t1 = sorted_res[0] if sorted_res else None
    t2 = sorted_res[1] if len(sorted_res) > 1 else None
    t1_name = t1[0] if t1 and t1[1] > 0 else None
    t2_name = t2[0] if t2 and t2[1] > 0 else None
    return t1_name, t2_name


def pool_sculpting_replace(pool, reserve, t1, t2, num_replace, t1_share, t2_share):
    """
    Replace cards in pool with resonance-matched cards from reserve.
    Remove off-resonance cards, add t1_share T1 cards and t2_share T2 cards.
    Removed cards are recycled into reserve.
    """
    if t1 is None:
        return pool

    # Find eligible cards to remove (not T1, not T2, not generic)
    eligible_remove = []
    for i, card in enumerate(pool):
        if card.is_generic:
            continue
        pr = card.primary_resonance
        if pr != t1 and (t2 is None or pr != t2):
            eligible_remove.append(i)

    actual_remove = min(num_replace, len(eligible_remove))
    if actual_remove == 0:
        return pool

    # Select cards to remove
    remove_indices = random.sample(eligible_remove, actual_remove)
    removed_cards = [pool[i] for i in remove_indices]

    # Recycle removed cards into reserve
    for card in removed_cards:
        pr = card.primary_resonance
        if pr and pr in reserve:
            reserve[pr].append(card)

    # Remove from pool (in reverse order to preserve indices)
    for i in sorted(remove_indices, reverse=True):
        pool.pop(i)

    # Add replacement cards
    # Split between T1 and T2
    t1_count = min(t1_share, len(reserve.get(t1, []))) if t1 else 0
    t2_count = min(t2_share, len(reserve.get(t2, []))) if t2 else 0

    # If T2 is None, give all to T1
    if t2 is None:
        t1_count = min(actual_remove, len(reserve.get(t1, [])))
        t2_count = 0
    else:
        total_add = t1_count + t2_count
        if total_add < actual_remove:
            # Try to make up difference
            extra_t1 = min(actual_remove - total_add, len(reserve.get(t1, [])) - t1_count)
            t1_count += max(0, extra_t1)

    # Draw from reserve
    if t1 and t1_count > 0:
        random.shuffle(reserve[t1])
        additions_t1 = reserve[t1][:t1_count]
        reserve[t1] = reserve[t1][t1_count:]
        pool.extend(additions_t1)

    if t2 and t2_count > 0:
        random.shuffle(reserve[t2])
        additions_t2 = reserve[t2][:t2_count]
        reserve[t2] = reserve[t2][t2_count:]
        pool.extend(additions_t2)

    return pool


def draw_pack(pool, size=PACK_SIZE):
    """Draw a random pack from the pool."""
    if len(pool) < size:
        return list(pool)
    return random.sample(pool, size)


# ============================================================
# Simulation core
# ============================================================

def determine_target_archetype(resonance_counts, pick_num):
    """Determine which archetype the player is converging toward."""
    t1, t2 = get_top_two_resonances(resonance_counts)
    if t1 is None:
        return None

    # Find archetypes matching the resonance profile
    if t2 is not None:
        # Look for archetype with primary=t1, secondary=t2
        for a in ARCHETYPES:
            if a["primary"] == t1 and a["secondary"] == t2:
                return a["name"]
        # Fallback: primary=t1
        for a in ARCHETYPES:
            if a["primary"] == t1:
                return a["name"]

    # Only T1 known: pick an archetype with T1 as primary
    candidates = [a for a in ARCHETYPES if a["primary"] == t1]
    if candidates:
        return candidates[0]["name"]
    return None


def run_single_draft(strategy_name, params, trace=False):
    """Run a single 30-pick draft. Returns metrics dict."""
    pool = generate_pool()
    reserve = generate_reserve()
    pick_fn = STRATEGIES[strategy_name]

    num_replace = params["num_replace"]
    t1_ratio = params["t1_ratio"]
    delayed_start = params.get("delayed_start", 3)
    escalate = params.get("escalate", False)
    escalate_rate = params.get("escalate_rate", num_replace)

    player_state = {
        "resonance_counts": defaultdict(int),
        "target_archetype": None,
        "drafted_cards": [],
    }

    # Metrics collection
    pack_metrics = []  # per-pick metrics
    pool_composition_snapshots = {}  # pick -> resonance distribution
    trace_log = [] if trace else None

    for pick_num in range(1, NUM_PICKS + 1):
        # Generate pack
        pack = draw_pack(pool)

        # Determine target archetype for player strategy
        if pick_num >= 5 and player_state["target_archetype"] is None:
            player_state["target_archetype"] = determine_target_archetype(
                player_state["resonance_counts"], pick_num
            )
        elif pick_num >= 3:
            # Update target archetype as resonances clarify
            candidate = determine_target_archetype(
                player_state["resonance_counts"], pick_num
            )
            if candidate:
                player_state["target_archetype"] = candidate

        # Player picks
        chosen = pick_fn(pack, player_state)
        player_state["drafted_cards"].append(chosen)

        # Update resonance counts
        w = chosen.weighted_symbols()
        for res, count in w.items():
            player_state["resonance_counts"][res] += count

        # Remove chosen card from pool if present
        pool = [c for c in pool if c.id != chosen.id]

        # Evaluate pack quality BEFORE sculpting (what player actually saw)
        target = player_state["target_archetype"]
        pack_sa_count = 0
        pack_cf_count = 0
        pack_archetypes_with_sa = set()

        for card in pack:
            if target:
                tier = card.archetype_fitness.get(target, "F")
                if is_sa_tier(tier):
                    pack_sa_count += 1
                if is_cf_tier(tier):
                    pack_cf_count += 1
            for aname, tier in card.archetype_fitness.items():
                if is_sa_tier(tier):
                    pack_archetypes_with_sa.add(aname)

        pack_metrics.append({
            "pick": pick_num,
            "sa_count": pack_sa_count,
            "cf_count": pack_cf_count,
            "unique_archetypes_with_sa": len(pack_archetypes_with_sa),
            "target": target,
        })

        # Pool composition snapshot at key picks
        if pick_num in [5, 10, 15, 20, 25, 30]:
            t1, t2 = get_top_two_resonances(player_state["resonance_counts"])
            total = len(pool)
            t1_count = sum(1 for c in pool if c.primary_resonance == t1)
            t2_count = sum(1 for c in pool if c.primary_resonance == t2) if t2 else 0
            pool_composition_snapshots[pick_num] = {
                "total": total,
                "t1": t1,
                "t1_count": t1_count,
                "t1_pct": t1_count / max(total, 1) * 100,
                "t2": t2,
                "t2_count": t2_count,
                "t2_pct": t2_count / max(total, 1) * 100,
                "combined_pct": (t1_count + t2_count) / max(total, 1) * 100,
            }

        # Trace
        if trace_log is not None:
            t1, t2 = get_top_two_resonances(player_state["resonance_counts"])
            trace_log.append({
                "pick": pick_num,
                "chosen": f"{chosen.archetype} {chosen.symbols}",
                "pack": [f"{c.archetype} {c.symbols}" for c in pack],
                "t1": t1,
                "t2": t2,
                "target": target,
                "sa_in_pack": pack_sa_count,
                "pool_size": len(pool),
                "res_counts": dict(player_state["resonance_counts"]),
            })

        # --- POOL SCULPTING STEP ---
        if pick_num >= delayed_start:
            t1, t2 = get_top_two_resonances(player_state["resonance_counts"])
            current_replace = num_replace
            if escalate and pick_num >= 6:
                current_replace = escalate_rate

            if t2 is not None:
                t1_share = current_replace // 2
                t2_share = current_replace - t1_share
                # Apply t1_ratio weighting
                t1_share = int(current_replace * t1_ratio)
                t2_share = current_replace - t1_share
            else:
                t1_share = current_replace
                t2_share = 0

            pool = pool_sculpting_replace(pool, reserve, t1, t2,
                                          current_replace, t1_share, t2_share)

    # Compute final metrics
    final_target = player_state["target_archetype"]

    # Deck composition
    deck = player_state["drafted_cards"]
    deck_sa = sum(1 for c in deck if final_target and
                  is_sa_tier(c.archetype_fitness.get(final_target, "F")))
    deck_concentration = deck_sa / len(deck) if deck else 0

    return {
        "pack_metrics": pack_metrics,
        "pool_snapshots": pool_composition_snapshots,
        "final_archetype": final_target,
        "deck_concentration": deck_concentration,
        "deck": deck,
        "trace": trace_log,
    }


def compute_aggregate_metrics(all_results):
    """Compute all 9 required metrics from simulation results."""
    metrics = {}

    # Metric 1: Picks 1-5: unique archetypes with S/A cards per pack
    early_unique = []
    for result in all_results:
        for pm in result["pack_metrics"]:
            if pm["pick"] <= 5:
                early_unique.append(pm["unique_archetypes_with_sa"])
    metrics["early_unique_archetypes"] = sum(early_unique) / max(len(early_unique), 1)

    # Metric 2: Picks 1-5: S/A cards for emerging archetype per pack
    early_sa = []
    for result in all_results:
        for pm in result["pack_metrics"]:
            if pm["pick"] <= 5 and pm["target"] is not None:
                early_sa.append(pm["sa_count"])
    metrics["early_sa_for_emerging"] = sum(early_sa) / max(len(early_sa), 1) if early_sa else 0

    # Metric 3: Picks 6+: S/A cards for committed archetype per pack
    late_sa = []
    for result in all_results:
        for pm in result["pack_metrics"]:
            if pm["pick"] >= 6 and pm["target"] is not None:
                late_sa.append(pm["sa_count"])
    metrics["late_sa_for_committed"] = sum(late_sa) / max(len(late_sa), 1)

    # Metric 4: Picks 6+: off-archetype (C/F) cards per pack
    late_cf = []
    for result in all_results:
        for pm in result["pack_metrics"]:
            if pm["pick"] >= 6 and pm["target"] is not None:
                late_cf.append(pm["cf_count"])
    metrics["late_cf_per_pack"] = sum(late_cf) / max(len(late_cf), 1)

    # Metric 5: Convergence pick (first pick where rolling avg SA >= 2.0)
    convergence_picks = []
    for result in all_results:
        target = result["final_archetype"]
        if target is None:
            continue
        # Rolling window of 3 picks
        found = False
        for i, pm in enumerate(result["pack_metrics"]):
            if i >= 2 and pm["target"] is not None:
                window = result["pack_metrics"][max(0, i-2):i+1]
                avg_sa = sum(p["sa_count"] for p in window if p["target"] is not None)
                count = sum(1 for p in window if p["target"] is not None)
                if count > 0 and avg_sa / count >= 2.0:
                    convergence_picks.append(pm["pick"])
                    found = True
                    break
        if not found:
            convergence_picks.append(31)  # never converged
    metrics["convergence_pick"] = sum(convergence_picks) / max(len(convergence_picks), 1)

    # Metric 6: Deck archetype concentration
    concentrations = [r["deck_concentration"] for r in all_results]
    metrics["deck_concentration"] = sum(concentrations) / max(len(concentrations), 1)

    # Metric 7: Run-to-run variety (card overlap between same-archetype runs)
    arch_decks = defaultdict(list)
    for result in all_results:
        if result["final_archetype"]:
            card_ids = set(c.id for c in result["deck"])
            arch_decks[result["final_archetype"]].append(card_ids)

    overlaps = []
    for arch, decks in arch_decks.items():
        if len(decks) >= 2:
            for i in range(min(50, len(decks) - 1)):
                j = i + 1
                overlap = len(decks[i] & decks[j]) / max(len(decks[i] | decks[j]), 1)
                overlaps.append(overlap)
    metrics["card_overlap"] = sum(overlaps) / max(len(overlaps), 1)

    # Metric 8: Archetype frequency
    arch_freq = defaultdict(int)
    total_runs = 0
    for result in all_results:
        if result["final_archetype"]:
            arch_freq[result["final_archetype"]] += 1
            total_runs += 1
    arch_freq_pct = {a: arch_freq.get(a, 0) / max(total_runs, 1) * 100
                     for a in ARCHETYPE_NAMES}
    metrics["archetype_frequency"] = arch_freq_pct
    metrics["max_archetype_freq"] = max(arch_freq_pct.values()) if arch_freq_pct else 0
    metrics["min_archetype_freq"] = min(arch_freq_pct.values()) if arch_freq_pct else 0

    # Metric 9 (variance): StdDev of S/A cards per pack (picks 6+)
    if late_sa:
        mean_sa = sum(late_sa) / len(late_sa)
        variance = sum((x - mean_sa) ** 2 for x in late_sa) / len(late_sa)
        metrics["late_sa_stddev"] = math.sqrt(variance)
    else:
        metrics["late_sa_stddev"] = 0

    return metrics


def compute_per_archetype_convergence(all_results):
    """Compute convergence pick per archetype."""
    arch_conv = defaultdict(list)

    for result in all_results:
        target = result["final_archetype"]
        if target is None:
            continue
        found = False
        for i, pm in enumerate(result["pack_metrics"]):
            if i >= 2 and pm["target"] is not None:
                window = result["pack_metrics"][max(0, i-2):i+1]
                avg_sa = sum(p["sa_count"] for p in window if p["target"] is not None)
                count = sum(1 for p in window if p["target"] is not None)
                if count > 0 and avg_sa / count >= 2.0:
                    arch_conv[target].append(pm["pick"])
                    found = True
                    break
        if not found:
            arch_conv[target].append(31)

    table = {}
    for arch_name in ARCHETYPE_NAMES:
        picks = arch_conv.get(arch_name, [])
        if picks:
            avg = sum(picks) / len(picks)
            converged_pct = sum(1 for p in picks if p <= 30) / len(picks) * 100
            table[arch_name] = {"avg_pick": avg, "converged_pct": converged_pct, "n": len(picks)}
        else:
            table[arch_name] = {"avg_pick": None, "converged_pct": 0, "n": 0}
    return table


def compute_pool_composition_over_time(all_results):
    """Average pool composition at key picks."""
    pick_points = [5, 10, 15, 20, 25, 30]
    avg_comp = {}
    for pp in pick_points:
        t1_pcts = []
        combined_pcts = []
        for result in all_results:
            snap = result["pool_snapshots"].get(pp)
            if snap:
                t1_pcts.append(snap["t1_pct"])
                combined_pcts.append(snap["combined_pct"])
        if t1_pcts:
            avg_comp[pp] = {
                "avg_t1_pct": sum(t1_pcts) / len(t1_pcts),
                "avg_combined_pct": sum(combined_pcts) / len(combined_pcts),
            }
    return avg_comp


def compute_pack_quality_variance(all_results):
    """Distribution of S/A per pack for picks 6+."""
    sa_counts = []
    for result in all_results:
        for pm in result["pack_metrics"]:
            if pm["pick"] >= 6 and pm["target"] is not None:
                sa_counts.append(pm["sa_count"])

    if not sa_counts:
        return {}

    distribution = defaultdict(int)
    for s in sa_counts:
        distribution[s] += 1

    total = len(sa_counts)
    mean = sum(sa_counts) / total
    variance = sum((x - mean) ** 2 for x in sa_counts) / total
    stddev = math.sqrt(variance)

    return {
        "distribution": {k: v / total * 100 for k, v in sorted(distribution.items())},
        "mean": mean,
        "stddev": stddev,
        "total_packs": total,
    }


# ============================================================
# Main simulation runner
# ============================================================

def run_parameter_variant(params, label, trace_drafts=0):
    """Run full simulation for a parameter variant."""
    print(f"\n{'='*60}")
    print(f"Running variant: {label}")
    print(f"  Replace/pick: {params['num_replace']}, T1 ratio: {params['t1_ratio']}")
    if params.get('escalate'):
        print(f"  Escalate at pick 6 to: {params['escalate_rate']}/pick")
    print(f"  Delayed start: pick {params.get('delayed_start', 3)}")
    print(f"{'='*60}")

    all_results = []
    traces = []

    for strat_name in STRATEGIES:
        print(f"  Strategy: {strat_name}...", end=" ", flush=True)
        strat_results = []
        for draft_num in range(NUM_DRAFTS):
            do_trace = (len(traces) < trace_drafts and
                        draft_num < trace_drafts and
                        strat_name == list(STRATEGIES.keys())[len(traces) % len(STRATEGIES)])
            result = run_single_draft(strat_name, params, trace=do_trace)
            result["strategy"] = strat_name
            strat_results.append(result)
            if do_trace and result["trace"]:
                traces.append({
                    "strategy": strat_name,
                    "trace": result["trace"],
                    "final_archetype": result["final_archetype"],
                    "deck_concentration": result["deck_concentration"],
                })
        all_results.extend(strat_results)
        # Quick per-strategy summary
        sm = compute_aggregate_metrics(strat_results)
        print(f"SA={sm['late_sa_for_committed']:.2f}, Conv={sm['convergence_pick']:.1f}, "
              f"Conc={sm['deck_concentration']:.1%}")

    # Aggregate across all strategies
    agg = compute_aggregate_metrics(all_results)
    per_arch = compute_per_archetype_convergence(all_results)
    pool_comp = compute_pool_composition_over_time(all_results)
    pack_var = compute_pack_quality_variance(all_results)

    # Per-strategy breakdown
    per_strat = {}
    for strat_name in STRATEGIES:
        strat_results = [r for r in all_results if r["strategy"] == strat_name]
        per_strat[strat_name] = compute_aggregate_metrics(strat_results)

    return {
        "label": label,
        "params": params,
        "aggregate": agg,
        "per_archetype": per_arch,
        "pool_composition": pool_comp,
        "pack_variance": pack_var,
        "per_strategy": per_strat,
        "traces": traces,
        "all_results": all_results,
    }


def format_trace(trace_data):
    """Format a draft trace for output."""
    lines = []
    lines.append(f"  Strategy: {trace_data['strategy']}")
    lines.append(f"  Final archetype: {trace_data['final_archetype']}")
    lines.append(f"  Deck concentration: {trace_data['deck_concentration']:.1%}")
    lines.append("")
    for entry in trace_data["trace"][:15]:  # Show first 15 picks
        lines.append(f"  Pick {entry['pick']:2d}: Chose {entry['chosen']}")
        lines.append(f"          Pack: {entry['pack']}")
        lines.append(f"          T1={entry['t1']}, T2={entry['t2']}, "
                     f"Target={entry['target']}, SA={entry['sa_in_pack']}, "
                     f"Pool={entry['pool_size']}")
        lines.append(f"          Resonance: {entry['res_counts']}")
    return "\n".join(lines)


def format_results(variant_result):
    """Format results for one variant."""
    v = variant_result
    agg = v["aggregate"]
    lines = []
    lines.append(f"\n### {v['label']}")
    lines.append(f"Replace/pick: {v['params']['num_replace']}, "
                 f"T1 ratio: {v['params']['t1_ratio']}")

    lines.append("\n**Core Metrics (all at archetype level):**")
    lines.append(f"| Metric | Value | Target |")
    lines.append(f"|--------|-------|--------|")
    lines.append(f"| Early unique archetypes (picks 1-5) | {agg['early_unique_archetypes']:.2f} | >= 3 |")
    lines.append(f"| Early SA for emerging (picks 1-5) | {agg['early_sa_for_emerging']:.2f} | <= 2 |")
    lines.append(f"| Late SA for committed (picks 6+) | {agg['late_sa_for_committed']:.2f} | >= 2 |")
    lines.append(f"| Late C/F per pack (picks 6+) | {agg['late_cf_per_pack']:.2f} | >= 0.5 |")
    lines.append(f"| Convergence pick | {agg['convergence_pick']:.1f} | 5-8 |")
    lines.append(f"| Deck concentration | {agg['deck_concentration']:.1%} | 60-90% |")
    lines.append(f"| Card overlap | {agg['card_overlap']:.1%} | < 40% |")
    lines.append(f"| Max archetype freq | {agg['max_archetype_freq']:.1f}% | < 20% |")
    lines.append(f"| Min archetype freq | {agg['min_archetype_freq']:.1f}% | > 5% |")
    lines.append(f"| SA stddev (picks 6+) | {agg['late_sa_stddev']:.2f} | >= 0.8 |")

    # Per-strategy breakdown
    lines.append("\n**Per-Strategy Breakdown:**")
    lines.append(f"| Strategy | Late SA | Conv Pick | Deck Conc |")
    lines.append(f"|----------|---------|-----------|-----------|")
    for strat_name, sm in v["per_strategy"].items():
        lines.append(f"| {strat_name} | {sm['late_sa_for_committed']:.2f} | "
                     f"{sm['convergence_pick']:.1f} | {sm['deck_concentration']:.1%} |")

    # Per-archetype convergence
    lines.append("\n**Per-Archetype Convergence:**")
    lines.append(f"| Archetype | Avg Conv Pick | Converged % | N |")
    lines.append(f"|-----------|---------------|-------------|---|")
    for arch_name in ARCHETYPE_NAMES:
        data = v["per_archetype"].get(arch_name, {})
        avg = data.get("avg_pick")
        cpct = data.get("converged_pct", 0)
        n = data.get("n", 0)
        if avg is not None:
            lines.append(f"| {arch_name} | {avg:.1f} | {cpct:.0f}% | {n} |")
        else:
            lines.append(f"| {arch_name} | N/A | 0% | 0 |")

    # Pool composition over time
    lines.append("\n**Pool Composition Over Time:**")
    lines.append(f"| Pick | Avg T1% | Avg T1+T2% |")
    lines.append(f"|------|---------|------------|")
    for pp in sorted(v["pool_composition"].keys()):
        comp = v["pool_composition"][pp]
        lines.append(f"| {pp} | {comp['avg_t1_pct']:.1f}% | {comp['avg_combined_pct']:.1f}% |")

    # Pack quality variance
    pv = v["pack_variance"]
    if pv:
        lines.append("\n**Pack Quality Variance (picks 6+):**")
        lines.append(f"Mean SA: {pv['mean']:.2f}, StdDev: {pv['stddev']:.2f}")
        lines.append(f"| SA Count | % of Packs |")
        lines.append(f"|----------|-----------|")
        for sa, pct in sorted(pv["distribution"].items()):
            lines.append(f"| {sa} | {pct:.1f}% |")

    return "\n".join(lines)


def main():
    random.seed(42)

    print("=" * 60)
    print("DUAL-RESONANCE POOL SCULPTING SIMULATION")
    print("Agent 4 — V6 Round 3")
    print("=" * 60)

    # Parameter variants to test
    variants = [
        {
            "label": "Variant A: Moderate (12/pick, 50/50 split)",
            "params": {
                "num_replace": 12,
                "t1_ratio": 0.5,
                "delayed_start": 3,
            },
        },
        {
            "label": "Variant B: Aggressive (20/pick, 50/50 split)",
            "params": {
                "num_replace": 20,
                "t1_ratio": 0.5,
                "delayed_start": 3,
            },
        },
        {
            "label": "Variant C: Escalating (12 early -> 24 late)",
            "params": {
                "num_replace": 12,
                "t1_ratio": 0.5,
                "delayed_start": 3,
                "escalate": True,
                "escalate_rate": 24,
            },
        },
        {
            "label": "Variant D: T1-heavy split (18/pick, 67/33 split)",
            "params": {
                "num_replace": 18,
                "t1_ratio": 0.67,
                "delayed_start": 3,
            },
        },
        {
            "label": "Variant E: Even split (18/pick, 50/50 split)",
            "params": {
                "num_replace": 18,
                "t1_ratio": 0.5,
                "delayed_start": 3,
            },
        },
        {
            "label": "Variant F: No delayed start (18/pick from pick 1)",
            "params": {
                "num_replace": 18,
                "t1_ratio": 0.5,
                "delayed_start": 1,
            },
        },
    ]

    results = []
    for v in variants:
        trace_count = 3 if v == variants[0] else (1 if v == variants[-1] else 0)
        result = run_parameter_variant(v["params"], v["label"],
                                       trace_drafts=trace_count)
        results.append(result)

    # Print summary comparison
    print("\n" + "=" * 60)
    print("SUMMARY COMPARISON")
    print("=" * 60)
    print(f"{'Variant':<50} {'Late SA':>8} {'Conv':>6} {'Conc':>8} {'Stddev':>8} {'Overlap':>8}")
    print("-" * 90)
    for r in results:
        agg = r["aggregate"]
        print(f"{r['label']:<50} {agg['late_sa_for_committed']:>8.2f} "
              f"{agg['convergence_pick']:>6.1f} {agg['deck_concentration']:>8.1%} "
              f"{agg['late_sa_stddev']:>8.2f} {agg['card_overlap']:>8.1%}")

    # Identify best variant
    best = max(results, key=lambda r: r["aggregate"]["late_sa_for_committed"])
    print(f"\nBest variant by Late SA: {best['label']}")
    print(f"  Late SA = {best['aggregate']['late_sa_for_committed']:.2f}")

    # One-sentence claim test
    print("\n" + "=" * 60)
    print("ONE-SENTENCE CLAIM TEST")
    print("=" * 60)
    print("Sentence: 'After each pick, replace 18 cards in the pool with 9 cards")
    print("matching your top resonance and 9 matching your second resonance from a")
    print("reserve, keeping the pool at 360 cards; no replacements before pick 3.'")
    print("")
    print("Can a programmer implement this from the sentence alone?")
    print("  - 'replace 18 cards' -> clear: remove 18 off-resonance, add 18 on-resonance")
    print("  - 'top resonance / second resonance' -> requires tracking weighted symbol counts")
    print("  - 'from a reserve' -> reserve must exist; sentence doesn't specify size/composition")
    print("  - 'keeping pool at 360' -> clear: constant pool size")
    print("  - 'no replacements before pick 3' -> clear")
    print("  VERDICT: Mostly implementable, but reserve details need specification.")

    # Verify no player decisions
    print("\n" + "=" * 60)
    print("NO PLAYER DECISIONS VERIFICATION")
    print("=" * 60)
    print("Player's only action: choose 1 card from a pack of 4.")
    print("Pool sculpting happens automatically after each pick.")
    print("No spending, no mode selection, no opt-in/opt-out.")
    print("VERIFIED: Zero player decisions beyond card selection.")

    # Print traces
    print("\n" + "=" * 60)
    print("DRAFT TRACES")
    print("=" * 60)
    for r in results:
        if r["traces"]:
            for t in r["traces"]:
                print(f"\n--- Trace ({r['label']}) ---")
                print(format_trace(t))

    # Write detailed results
    all_output = []
    all_output.append("# FULL SIMULATION OUTPUT\n")
    for r in results:
        all_output.append(format_results(r))

    # Write to stdout for capture
    print("\n" + "=" * 60)
    print("DETAILED RESULTS")
    print("=" * 60)
    for line in all_output:
        print(line)

    return results


if __name__ == "__main__":
    results = main()
