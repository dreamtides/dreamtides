#!/usr/bin/env python3
"""
Dual-Threshold Pair Guarantee — V5 Domain 4 Simulation

Algorithm (one-sentence): "Track the ordered symbol pair of each 2+ symbol card
you draft; at 3 matching picks one pack slot is pair-matched, at 7 matching picks
a second slot is pair-matched, and remaining slots are random."

Includes:
  - Card pool generation (360 cards, 8 archetypes + generics)
  - 3 player strategies (archetype-committed, power-chaser, signal-reader)
  - 1000 drafts of 30 picks
  - All 8 measurable targets at ARCHETYPE level
  - Variance target (stddev of S/A per pack, picks 6+)
  - Per-archetype convergence table
  - V3 Lane Locking baseline
  - V4 Pack Widening baseline (auto-spend)
  - Parameter sensitivity sweeps
  - Symbol distribution sensitivity
  - 3 detailed draft traces
"""

import random
import math
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional

# ─── Resonance types ───────────────────────────────────────────────────────────
class Resonance(Enum):
    EMBER = "Ember"
    STONE = "Stone"
    TIDE = "Tide"
    ZEPHYR = "Zephyr"

# ─── Archetype definitions (circle order) ──────────────────────────────────────
ARCHETYPES = [
    {"id": 0, "name": "Flash/Tempo/Prison",   "primary": Resonance.ZEPHYR, "secondary": Resonance.EMBER},
    {"id": 1, "name": "Blink/Flicker",        "primary": Resonance.EMBER,  "secondary": Resonance.ZEPHYR},
    {"id": 2, "name": "Storm/Spellslinger",   "primary": Resonance.EMBER,  "secondary": Resonance.STONE},
    {"id": 3, "name": "Self-Discard",          "primary": Resonance.STONE,  "secondary": Resonance.EMBER},
    {"id": 4, "name": "Self-Mill/Reanimator",  "primary": Resonance.STONE,  "secondary": Resonance.TIDE},
    {"id": 5, "name": "Sacrifice/Abandon",     "primary": Resonance.TIDE,   "secondary": Resonance.STONE},
    {"id": 6, "name": "Warriors/Midrange",     "primary": Resonance.TIDE,   "secondary": Resonance.ZEPHYR},
    {"id": 7, "name": "Ramp/Spirit Animals",   "primary": Resonance.ZEPHYR, "secondary": Resonance.TIDE},
]

def circle_distance(a_id, b_id):
    """Minimum distance on the 8-archetype circle."""
    d = abs(a_id - b_id)
    return min(d, 8 - d)

# ─── Card model ────────────────────────────────────────────────────────────────
@dataclass
class SimCard:
    id: int
    symbols: list  # list of Resonance, ordered, 0-3 elements
    archetype: Optional[int]  # archetype id, None for generics
    power: float
    archetype_fitness: dict = field(default_factory=dict)  # archetype_id -> tier

    @property
    def ordered_pair(self):
        """Return (primary, secondary) if 2+ symbols, else None."""
        if len(self.symbols) >= 2:
            return (self.symbols[0], self.symbols[1])
        return None

def assign_fitness(card, archetypes):
    """Assign S/A/B/C/F fitness for evaluation."""
    fitness = {}
    if card.archetype is None:
        # Generic: B-tier in all archetypes
        for a in archetypes:
            fitness[a["id"]] = "B"
    else:
        home = archetypes[card.archetype]
        for a in archetypes:
            if a["id"] == card.archetype:
                fitness[a["id"]] = "S"
            elif circle_distance(a["id"], card.archetype) == 1:
                # Adjacent: A-tier if shares primary, B-tier if shares secondary
                if a["primary"] == home["primary"] or a["secondary"] == home["primary"]:
                    fitness[a["id"]] = "A"
                else:
                    fitness[a["id"]] = "B"
            elif circle_distance(a["id"], card.archetype) == 2:
                fitness[a["id"]] = "C"
            else:
                fitness[a["id"]] = "F"
    card.archetype_fitness = fitness

def generate_card_pool(sym_dist=(0.15, 0.65, 0.20), seed=None):
    """
    Generate 360 cards: 40 per archetype (320) + 36 generic.
    sym_dist = (frac_1sym, frac_2sym, frac_3sym) among non-generic cards.
    """
    if seed is not None:
        random.seed(seed)

    cards = []
    card_id = 0
    frac_1, frac_2, frac_3 = sym_dist

    for arch in ARCHETYPES:
        n_cards = 40
        n1 = round(n_cards * frac_1)
        n3 = round(n_cards * frac_3)
        n2 = n_cards - n1 - n3

        for _ in range(n1):
            # 1-symbol card: primary only
            symbols = [arch["primary"]]
            c = SimCard(id=card_id, symbols=symbols, archetype=arch["id"],
                        power=random.uniform(3, 8))
            assign_fitness(c, ARCHETYPES)
            cards.append(c)
            card_id += 1

        for _ in range(n2):
            # 2-symbol card: [primary, secondary]
            symbols = [arch["primary"], arch["secondary"]]
            c = SimCard(id=card_id, symbols=symbols, archetype=arch["id"],
                        power=random.uniform(3, 8))
            assign_fitness(c, ARCHETYPES)
            cards.append(c)
            card_id += 1

        for _ in range(n3):
            # 3-symbol card: [primary, primary, secondary] or [primary, secondary, primary]
            if random.random() < 0.5:
                symbols = [arch["primary"], arch["primary"], arch["secondary"]]
            else:
                symbols = [arch["primary"], arch["secondary"], arch["primary"]]
            c = SimCard(id=card_id, symbols=symbols, archetype=arch["id"],
                        power=random.uniform(3, 8))
            assign_fitness(c, ARCHETYPES)
            cards.append(c)
            card_id += 1

    # 36 generic cards
    for _ in range(36):
        c = SimCard(id=card_id, symbols=[], archetype=None,
                    power=random.uniform(4, 9))
        assign_fitness(c, ARCHETYPES)
        cards.append(c)
        card_id += 1

    return cards


# ─── Symbol counting ──────────────────────────────────────────────────────────
def count_symbols(drafted_cards):
    """Count weighted symbols: primary=2, secondary/tertiary=1."""
    counts = Counter()
    for card in drafted_cards:
        for i, sym in enumerate(card.symbols):
            counts[sym] += 2 if i == 0 else 1
    return counts

def count_pairs(drafted_cards):
    """Count ordered pairs from 2+ symbol cards."""
    counts = Counter()
    for card in drafted_cards:
        pair = card.ordered_pair
        if pair is not None:
            counts[pair] += 1
    return counts

def leading_pair(pair_counts):
    """Return the pair with highest count, or None."""
    if not pair_counts:
        return None
    return pair_counts.most_common(1)[0]

def leading_resonance(sym_counts):
    """Return the resonance with highest weighted count."""
    if not sym_counts:
        return None
    return sym_counts.most_common(1)[0]


# ─── Draft algorithms ─────────────────────────────────────────────────────────

def draw_random_card(pool, rng):
    """Draw a random card from pool (with replacement)."""
    return rng.choice(pool)

def draw_pair_matched_card(pool, pair, rng):
    """Draw a random card from pool whose ordered pair matches."""
    candidates = [c for c in pool if c.ordered_pair == pair]
    if candidates:
        return rng.choice(candidates)
    # Fallback: match primary resonance
    primary = pair[0]
    candidates = [c for c in pool if len(c.symbols) > 0 and c.symbols[0] == primary]
    if candidates:
        return rng.choice(candidates)
    return rng.choice(pool)

def draw_resonance_matched_card(pool, resonance, rng):
    """Draw a random card whose primary resonance matches."""
    candidates = [c for c in pool if len(c.symbols) > 0 and c.symbols[0] == resonance]
    if candidates:
        return rng.choice(candidates)
    return rng.choice(pool)


def generate_pack_dual_threshold(pool, drafted_cards, rng, threshold1=3, threshold2=7):
    """
    Dual-Threshold Pair Guarantee.
    Track ordered pairs of drafted 2+ symbol cards.
    At threshold1 matching picks -> 1 slot pair-matched.
    At threshold2 matching picks -> 2 slots pair-matched.
    Remaining slots are random.
    """
    pair_counts = count_pairs(drafted_cards)
    top = leading_pair(pair_counts)

    guaranteed_slots = 0
    top_pair = None
    if top is not None:
        top_pair, top_count = top
        if top_count >= threshold2:
            guaranteed_slots = 2
        elif top_count >= threshold1:
            guaranteed_slots = 1

    pack = []
    for i in range(4):
        if i < guaranteed_slots and top_pair is not None:
            pack.append(draw_pair_matched_card(pool, top_pair, rng))
        else:
            pack.append(draw_random_card(pool, rng))
    return pack


def generate_pack_lane_locking(pool, drafted_cards, rng, threshold1=3, threshold2=8):
    """
    V3 Lane Locking baseline.
    Count weighted symbols (primary=2). At threshold1 of a resonance -> lock 1 slot.
    At threshold2 -> lock 2 slots. Locked slots show matching primary resonance cards.
    """
    sym_counts = count_symbols(drafted_cards)
    top = leading_resonance(sym_counts)

    locked_slots = 0
    top_res = None
    if top is not None:
        top_res, top_count = top
        if top_count >= threshold2:
            locked_slots = 2
        elif top_count >= threshold1:
            locked_slots = 1

    pack = []
    for i in range(4):
        if i < locked_slots and top_res is not None:
            pack.append(draw_resonance_matched_card(pool, top_res, rng))
        else:
            pack.append(draw_random_card(pool, rng))
    return pack


def generate_pack_widening(pool, drafted_cards, rng, cost=3, bonus_count=1):
    """
    V4 Pack Widening baseline (auto-spend on highest resonance).
    Count weighted symbols. When top resonance >= cost, auto-spend:
    deduct cost tokens, add bonus_count cards of that resonance to pack.
    Returns pack (4 + bonus cards) and tokens_spent info.
    """
    sym_counts = count_symbols(drafted_cards)
    top = leading_resonance(sym_counts)

    pack = [draw_random_card(pool, rng) for _ in range(4)]

    # Check if we can auto-spend
    # Note: We use a separate token tracker for pack widening since it spends tokens
    # We simulate auto-spend by checking the accumulated count
    # For simplicity, we track a running token balance
    # This is handled in the draft loop instead
    return pack


# ─── Player strategies ─────────────────────────────────────────────────────────

def pick_archetype_committed(pack, target_archetype, drafted_cards, pick_num):
    """Pick card with highest fitness for target archetype."""
    tier_order = {"S": 5, "A": 4, "B": 3, "C": 2, "F": 1}

    # Before commitment (picks 1-5), pick best power among S/A cards
    # After commitment, always pick highest tier for target
    best_idx = 0
    best_score = -1
    for i, card in enumerate(pack):
        tier = card.archetype_fitness.get(target_archetype, "F")
        score = tier_order[tier] * 100 + card.power
        if score > best_score:
            best_score = score
            best_idx = i
    return best_idx


def pick_power_chaser(pack, target_archetype, drafted_cards, pick_num):
    """Pick highest raw power card."""
    best_idx = 0
    best_power = -1
    for i, card in enumerate(pack):
        if card.power > best_power:
            best_power = card.power
            best_idx = i
    return best_idx


def pick_signal_reader(pack, target_archetype, drafted_cards, pick_num):
    """
    Picks 1-5: evaluate which archetype appears most in packs seen so far,
    lean toward the open archetype. Picks 6+: commit to strongest archetype.
    """
    if pick_num <= 5:
        # Pick highest power among the best-fitting cards for any archetype
        # Slight preference for cards matching the most-seen resonance
        sym_counts = count_symbols(drafted_cards)
        top_res_info = leading_resonance(sym_counts)
        top_res = top_res_info[0] if top_res_info else None

        best_idx = 0
        best_score = -1
        for i, card in enumerate(pack):
            bonus = 0
            if top_res and len(card.symbols) > 0 and card.symbols[0] == top_res:
                bonus = 2
            score = card.power + bonus
            if score > best_score:
                best_score = score
                best_idx = i
        return best_idx
    else:
        # Commit: find strongest archetype from drafted cards
        return pick_archetype_committed(pack, target_archetype, drafted_cards, pick_num)


def determine_target_archetype(drafted_cards):
    """Determine which archetype the drafted cards best fit."""
    tier_scores = {"S": 5, "A": 4, "B": 3, "C": 2, "F": 1}
    arch_scores = Counter()
    for card in drafted_cards:
        for aid, tier in card.archetype_fitness.items():
            arch_scores[aid] += tier_scores[tier]
    if not arch_scores:
        return 0
    return arch_scores.most_common(1)[0][0]


# ─── Draft simulation ──────────────────────────────────────────────────────────

def run_draft(pool, algorithm, strategy, target_archetype=None, rng=None,
              algo_params=None, trace=False):
    """
    Run a single 30-pick draft.
    Returns: drafted cards, per-pick data, trace info.
    """
    if rng is None:
        rng = random.Random()
    if algo_params is None:
        algo_params = {}

    drafted = []
    pick_data = []
    # For Pack Widening, track token balance separately
    pw_tokens = Counter()  # resonance -> token count

    for pick_num in range(1, 31):
        # Generate pack
        if algorithm == "dual_threshold":
            pack = generate_pack_dual_threshold(
                pool, drafted, rng,
                threshold1=algo_params.get("threshold1", 3),
                threshold2=algo_params.get("threshold2", 7)
            )
        elif algorithm == "lane_locking":
            pack = generate_pack_lane_locking(
                pool, drafted, rng,
                threshold1=algo_params.get("threshold1", 3),
                threshold2=algo_params.get("threshold2", 8)
            )
        elif algorithm == "pack_widening":
            # Auto-spend: check if top resonance has enough tokens
            cost = algo_params.get("cost", 3)
            bonus = algo_params.get("bonus", 1)
            pack = [draw_random_card(pool, rng) for _ in range(4)]
            # Auto-spend on highest resonance
            if pw_tokens:
                top_res = pw_tokens.most_common(1)[0]
                if top_res[1] >= cost:
                    pw_tokens[top_res[0]] -= cost
                    for _ in range(bonus):
                        pack.append(draw_resonance_matched_card(pool, top_res[0], rng))
        elif algorithm == "random":
            pack = [draw_random_card(pool, rng) for _ in range(4)]
        else:
            raise ValueError(f"Unknown algorithm: {algorithm}")

        # Determine effective target archetype
        if target_archetype is not None:
            eff_target = target_archetype
        else:
            eff_target = determine_target_archetype(drafted) if drafted else 0

        # Player picks
        if strategy == "committed":
            idx = pick_archetype_committed(pack, eff_target, drafted, pick_num)
        elif strategy == "power":
            idx = pick_power_chaser(pack, eff_target, drafted, pick_num)
        elif strategy == "signal":
            idx = pick_signal_reader(pack, eff_target, drafted, pick_num)
        else:
            raise ValueError(f"Unknown strategy: {strategy}")

        picked = pack[idx]
        drafted.append(picked)

        # Update Pack Widening tokens
        if algorithm == "pack_widening":
            for i, sym in enumerate(picked.symbols):
                pw_tokens[sym] += 2 if i == 0 else 1

        # Record pick data
        sa_count = sum(1 for c in pack[:4] if c.archetype_fitness.get(eff_target, "F") in ("S", "A"))
        off_count = sum(1 for c in pack[:4] if c.archetype_fitness.get(eff_target, "F") in ("C", "F"))

        # Unique archetypes with S/A cards
        archs_with_sa = set()
        for c in pack[:4]:
            for aid, tier in c.archetype_fitness.items():
                if tier in ("S", "A"):
                    archs_with_sa.add(aid)

        pick_info = {
            "pick_num": pick_num,
            "pack_size": len(pack),
            "sa_count": sa_count,
            "off_count": off_count,
            "unique_archs_sa": len(archs_with_sa),
            "picked_card": picked,
            "pack": pack[:],
            "target_archetype": eff_target,
        }
        pick_data.append(pick_info)

        if trace:
            pair_counts = count_pairs(drafted)
            top = leading_pair(pair_counts)
            top_str = f"{top[0][0].value}/{top[0][1].value}={top[1]}" if top else "none"
            pack_desc = []
            for j, c in enumerate(pack):
                syms = "/".join(s.value for s in c.symbols) if c.symbols else "Generic"
                tier = c.archetype_fitness.get(eff_target, "?")
                marker = " <-- PICKED" if j == idx else ""
                arch_name = ARCHETYPES[c.archetype]["name"] if c.archetype is not None else "Generic"
                pack_desc.append(f"    [{syms}] {arch_name} ({tier}-tier, pow={c.power:.1f}){marker}")
            print(f"  Pick {pick_num}: top_pair={top_str}, guaranteed_slots={'2' if top and top[1]>=algo_params.get('threshold2',7) else '1' if top and top[1]>=algo_params.get('threshold1',3) else '0'}")
            print(f"    Target: {ARCHETYPES[eff_target]['name']}, S/A in pack: {sa_count}")
            for desc in pack_desc:
                print(desc)

    return drafted, pick_data


# ─── Metrics calculation ───────────────────────────────────────────────────────

def compute_metrics(all_pick_data):
    """Compute all 8 measurable targets + variance from list of draft pick_data lists."""
    # Metric 1: Picks 1-5 unique archetypes with S/A per pack
    early_unique = []
    # Metric 2: Picks 1-5 S/A for emerging archetype
    early_sa = []
    # Metric 3: Picks 6+ S/A for committed archetype
    late_sa = []
    # Metric 4: Picks 6+ off-archetype (C/F) per pack
    late_off = []
    # Metric 5: Convergence pick
    convergence_picks = []
    # Metric 6: Deck concentration
    deck_concentrations = []
    # Metric 7: Run-to-run variety (need card ids)
    drafted_card_sets = []
    # Metric 8: Archetype frequency
    archetype_counts = Counter()

    for draft_picks in all_pick_data:
        for pd in draft_picks:
            if pd["pick_num"] <= 5:
                early_unique.append(pd["unique_archs_sa"])
                early_sa.append(pd["sa_count"])
            else:
                late_sa.append(pd["sa_count"])
                late_off.append(pd["off_count"])

        # Convergence: first pick where trailing avg of 3 packs has 2+ S/A
        conv_pick = 30
        for i in range(2, len(draft_picks)):
            if draft_picks[i]["pick_num"] >= 4:
                window = [draft_picks[j]["sa_count"] for j in range(max(0, i-2), i+1)]
                if all(w >= 2 for w in window):
                    conv_pick = draft_picks[i]["pick_num"]
                    break
        convergence_picks.append(conv_pick)

        # Deck concentration
        target = draft_picks[-1]["target_archetype"]
        drafted_in_run = [pd["picked_card"] for pd in draft_picks]
        sa_in_deck = sum(1 for c in drafted_in_run
                         if c.archetype_fitness.get(target, "F") in ("S", "A"))
        deck_concentrations.append(sa_in_deck / 30.0)

        # Card ids for variety
        drafted_card_sets.append(set(c.id for c in drafted_in_run))

        # Archetype frequency
        archetype_counts[target] += 1

    # Compute averages
    metrics = {}
    metrics["early_unique_archs"] = sum(early_unique) / len(early_unique) if early_unique else 0
    metrics["early_sa"] = sum(early_sa) / len(early_sa) if early_sa else 0
    metrics["late_sa"] = sum(late_sa) / len(late_sa) if late_sa else 0
    metrics["late_off"] = sum(late_off) / len(late_off) if late_off else 0
    metrics["convergence_pick"] = sum(convergence_picks) / len(convergence_picks) if convergence_picks else 30
    metrics["deck_concentration"] = sum(deck_concentrations) / len(deck_concentrations) if deck_concentrations else 0

    # Variance of S/A per pack (picks 6+)
    if late_sa:
        mean_sa = sum(late_sa) / len(late_sa)
        variance = sum((x - mean_sa) ** 2 for x in late_sa) / len(late_sa)
        metrics["sa_stddev"] = math.sqrt(variance)
    else:
        metrics["sa_stddev"] = 0

    # Distribution of S/A per pack
    sa_dist = Counter(late_sa)
    total_late = len(late_sa)
    metrics["sa_distribution"] = {k: sa_dist[k] / total_late for k in sorted(sa_dist.keys())} if total_late else {}

    # Run-to-run variety (average pairwise overlap)
    if len(drafted_card_sets) >= 2:
        overlaps = []
        # Sample pairs to avoid O(n^2)
        n_samples = min(500, len(drafted_card_sets) * (len(drafted_card_sets) - 1) // 2)
        for _ in range(n_samples):
            i, j = random.sample(range(len(drafted_card_sets)), 2)
            overlap = len(drafted_card_sets[i] & drafted_card_sets[j]) / 30.0
            overlaps.append(overlap)
        metrics["run_overlap"] = sum(overlaps) / len(overlaps)
    else:
        metrics["run_overlap"] = 0

    # Archetype frequency
    total_runs = sum(archetype_counts.values())
    metrics["archetype_freq"] = {aid: archetype_counts[aid] / total_runs
                                  for aid in range(8)}
    max_freq = max(metrics["archetype_freq"].values()) if metrics["archetype_freq"] else 0
    min_freq = min(metrics["archetype_freq"].values()) if metrics["archetype_freq"] else 0
    metrics["max_archetype_freq"] = max_freq
    metrics["min_archetype_freq"] = min_freq

    return metrics


def compute_per_archetype_convergence(pool, algorithm, algo_params, n_drafts=200):
    """
    For each archetype, run n_drafts with committed player targeting that archetype.
    Return average convergence pick for each.
    """
    results = {}
    for arch in ARCHETYPES:
        convergence_picks = []
        for _ in range(n_drafts):
            rng = random.Random()
            _, pick_data = run_draft(pool, algorithm, "committed",
                                      target_archetype=arch["id"],
                                      rng=rng, algo_params=algo_params)
            # Convergence: first pick where trailing 3-pick window all have 2+ S/A
            conv_pick = 30
            for i in range(2, len(pick_data)):
                if pick_data[i]["pick_num"] >= 3:
                    window = [pick_data[j]["sa_count"] for j in range(max(0, i-2), i+1)]
                    if all(w >= 2 for w in window):
                        conv_pick = pick_data[i]["pick_num"]
                        break
            convergence_picks.append(conv_pick)
        results[arch["id"]] = sum(convergence_picks) / len(convergence_picks)
    return results


def measure_pair_precision(pool):
    """Measure what % of pair-matched cards are S/A for the matching archetype."""
    # For each possible ordered pair, find the archetype it maps to
    pair_to_arch = {}
    for arch in ARCHETYPES:
        pair = (arch["primary"], arch["secondary"])
        pair_to_arch[pair] = arch["id"]

    total = 0
    sa_count = 0
    for pair, arch_id in pair_to_arch.items():
        candidates = [c for c in pool if c.ordered_pair == pair]
        for c in candidates:
            total += 1
            if c.archetype_fitness.get(arch_id, "F") in ("S", "A"):
                sa_count += 1

    return sa_count / total if total > 0 else 0


# ─── Main simulation ──────────────────────────────────────────────────────────

def run_full_simulation(algorithm, algo_params, pool, n_drafts=1000,
                        strategies=("committed", "power", "signal"),
                        label=""):
    """Run full simulation with all strategies, return metrics."""
    all_pick_data = []

    for strategy in strategies:
        n_per_strategy = n_drafts // len(strategies)
        for draft_i in range(n_per_strategy):
            rng = random.Random()
            # For committed strategy, pick a random target archetype
            target = random.randint(0, 7) if strategy == "committed" else None
            _, pick_data = run_draft(pool, algorithm, strategy,
                                      target_archetype=target,
                                      rng=rng, algo_params=algo_params)
            all_pick_data.append(pick_data)

    metrics = compute_metrics(all_pick_data)
    return metrics


def print_metrics(metrics, label=""):
    """Print metrics in scorecard format."""
    print(f"\n{'='*60}")
    print(f"  {label}")
    print(f"{'='*60}")
    print(f"  {'Metric':<50} {'Target':>10} {'Actual':>10} {'Pass':>6}")
    print(f"  {'-'*76}")
    print(f"  {'Picks 1-5: unique archs with S/A per pack':<50} {'>=3':>10} {metrics['early_unique_archs']:>10.2f} {'PASS' if metrics['early_unique_archs'] >= 3 else 'FAIL':>6}")
    print(f"  {'Picks 1-5: S/A for emerging arch per pack':<50} {'<=2':>10} {metrics['early_sa']:>10.2f} {'PASS' if metrics['early_sa'] <= 2 else 'FAIL':>6}")
    print(f"  {'Picks 6+: S/A for committed arch per pack':<50} {'>=2':>10} {metrics['late_sa']:>10.2f} {'PASS' if metrics['late_sa'] >= 2 else 'FAIL':>6}")
    print(f"  {'Picks 6+: off-arch (C/F) per pack':<50} {'>=0.5':>10} {metrics['late_off']:>10.2f} {'PASS' if metrics['late_off'] >= 0.5 else 'FAIL':>6}")
    print(f"  {'Convergence pick':<50} {'5-8':>10} {metrics['convergence_pick']:>10.1f} {'PASS' if 5 <= metrics['convergence_pick'] <= 8 else 'FAIL':>6}")
    print(f"  {'Deck concentration':<50} {'60-90%':>10} {metrics['deck_concentration']*100:>9.1f}% {'PASS' if 0.60 <= metrics['deck_concentration'] <= 0.90 else 'FAIL':>6}")
    print(f"  {'Run-to-run card overlap':<50} {'<40%':>10} {metrics['run_overlap']*100:>9.1f}% {'PASS' if metrics['run_overlap'] < 0.40 else 'FAIL':>6}")
    max_f = metrics['max_archetype_freq'] * 100
    min_f = metrics['min_archetype_freq'] * 100
    print(f"  {'Archetype freq (max/min)':<50} {'<20/>5%':>10} {max_f:>4.1f}/{min_f:>4.1f}% {'PASS' if max_f <= 20 and min_f >= 5 else 'FAIL':>6}")
    print(f"  {'StdDev S/A per pack (picks 6+)':<50} {'>=0.8':>10} {metrics['sa_stddev']:>10.2f} {'PASS' if metrics['sa_stddev'] >= 0.8 else 'FAIL':>6}")
    print(f"\n  S/A distribution (picks 6+):")
    for k in sorted(metrics.get("sa_distribution", {}).keys()):
        print(f"    {k} S/A cards: {metrics['sa_distribution'][k]*100:5.1f}%")


def print_convergence_table(conv_results, label=""):
    """Print per-archetype convergence table."""
    print(f"\n  Per-Archetype Convergence — {label}")
    print(f"  {'Archetype':<25} {'Avg Conv Pick':>15}")
    print(f"  {'-'*40}")
    for arch in ARCHETYPES:
        pick = conv_results[arch["id"]]
        print(f"  {arch['name']:<25} {pick:>15.1f}")
    avg = sum(conv_results.values()) / len(conv_results)
    print(f"  {'AVERAGE':<25} {avg:>15.1f}")


def run_draft_trace(pool, algorithm, algo_params, strategy, target_archetype,
                    label="", rng=None):
    """Run a single draft with trace output."""
    if rng is None:
        rng = random.Random(42)
    print(f"\n{'='*60}")
    print(f"  DRAFT TRACE: {label}")
    print(f"  Strategy: {strategy}, Target: {ARCHETYPES[target_archetype]['name']}")
    print(f"  Algorithm: {algorithm}, Params: {algo_params}")
    print(f"{'='*60}")

    drafted, pick_data = run_draft(pool, algorithm, strategy,
                                     target_archetype=target_archetype,
                                     rng=rng, algo_params=algo_params,
                                     trace=True)

    # Summary
    sa_total = sum(1 for c in drafted
                   if c.archetype_fitness.get(target_archetype, "F") in ("S", "A"))
    print(f"\n  Final deck: {sa_total}/30 S/A cards ({sa_total/30*100:.0f}% concentration)")
    pair_counts = count_pairs(drafted)
    top = leading_pair(pair_counts)
    if top:
        print(f"  Final leading pair: {top[0][0].value}/{top[0][1].value} = {top[1]}")


# ─── MAIN ─────────────────────────────────────────────────────────────────────

if __name__ == "__main__":
    random.seed(12345)

    # Generate card pool
    pool = generate_card_pool(sym_dist=(0.15, 0.65, 0.20), seed=42)
    print(f"Card pool: {len(pool)} cards")
    print(f"  Per archetype: ~{sum(1 for c in pool if c.archetype is not None)//8}")
    print(f"  Generics: {sum(1 for c in pool if c.archetype is None)}")
    print(f"  1-symbol: {sum(1 for c in pool if len(c.symbols)==1)}")
    print(f"  2-symbol: {sum(1 for c in pool if len(c.symbols)==2)}")
    print(f"  3-symbol: {sum(1 for c in pool if len(c.symbols)==3)}")

    # Measure pair precision
    pp = measure_pair_precision(pool)
    print(f"\n  Pair precision (% of pair-matched cards that are S/A for target arch): {pp*100:.1f}%")

    # ════════════════════════════════════════════════════════════════════════════
    # 1. DUAL-THRESHOLD PAIR GUARANTEE (primary algorithm)
    # ════════════════════════════════════════════════════════════════════════════
    print("\n" + "█" * 60)
    print("  PRIMARY: Dual-Threshold Pair Guarantee (3/7)")
    print("█" * 60)

    dt_params = {"threshold1": 3, "threshold2": 7}
    dt_metrics = run_full_simulation("dual_threshold", dt_params, pool,
                                      n_drafts=999, label="Dual-Threshold (3/7)")
    print_metrics(dt_metrics, "Dual-Threshold Pair Guarantee (3/7)")

    dt_conv = compute_per_archetype_convergence(pool, "dual_threshold", dt_params,
                                                  n_drafts=200)
    print_convergence_table(dt_conv, "Dual-Threshold (3/7)")

    # ════════════════════════════════════════════════════════════════════════════
    # 2. V3 LANE LOCKING BASELINE
    # ════════════════════════════════════════════════════════════════════════════
    print("\n" + "█" * 60)
    print("  BASELINE: V3 Lane Locking (3/8)")
    print("█" * 60)

    ll_params = {"threshold1": 3, "threshold2": 8}
    ll_metrics = run_full_simulation("lane_locking", ll_params, pool,
                                      n_drafts=999, label="Lane Locking (3/8)")
    print_metrics(ll_metrics, "V3 Lane Locking (3/8)")

    ll_conv = compute_per_archetype_convergence(pool, "lane_locking", ll_params,
                                                  n_drafts=200)
    print_convergence_table(ll_conv, "Lane Locking (3/8)")

    # ════════════════════════════════════════════════════════════════════════════
    # 3. V4 PACK WIDENING BASELINE (auto-spend)
    # ════════════════════════════════════════════════════════════════════════════
    print("\n" + "█" * 60)
    print("  BASELINE: V4 Pack Widening Auto-Spend (cost=3, bonus=1)")
    print("█" * 60)

    pw_params = {"cost": 3, "bonus": 1}
    pw_metrics = run_full_simulation("pack_widening", pw_params, pool,
                                      n_drafts=999, label="Pack Widening Auto-Spend")
    print_metrics(pw_metrics, "V4 Pack Widening Auto-Spend (cost=3, bonus=1)")

    pw_conv = compute_per_archetype_convergence(pool, "pack_widening", pw_params,
                                                  n_drafts=200)
    print_convergence_table(pw_conv, "Pack Widening Auto-Spend")

    # ════════════════════════════════════════════════════════════════════════════
    # 4. PARAMETER SENSITIVITY: threshold pairs
    # ════════════════════════════════════════════════════════════════════════════
    print("\n" + "█" * 60)
    print("  PARAMETER SENSITIVITY: Threshold Pairs")
    print("█" * 60)

    for t1, t2 in [(2, 5), (3, 7), (4, 9)]:
        params = {"threshold1": t1, "threshold2": t2}
        m = run_full_simulation("dual_threshold", params, pool,
                                n_drafts=999, label=f"DT ({t1}/{t2})")
        print_metrics(m, f"Dual-Threshold ({t1}/{t2})")

    # ════════════════════════════════════════════════════════════════════════════
    # 5. SYMBOL DISTRIBUTION SENSITIVITY
    # ════════════════════════════════════════════════════════════════════════════
    print("\n" + "█" * 60)
    print("  SYMBOL DISTRIBUTION SENSITIVITY")
    print("█" * 60)

    # 15% 1-symbol (default)
    pool_15 = generate_card_pool(sym_dist=(0.15, 0.65, 0.20), seed=42)
    pp_15 = measure_pair_precision(pool_15)
    m_15 = run_full_simulation("dual_threshold", dt_params, pool_15,
                                n_drafts=999, label="15% 1-sym")
    print(f"\n  15% 1-symbol: pair_precision={pp_15*100:.1f}%, late_sa={m_15['late_sa']:.2f}")

    # 30% 1-symbol (robustness test)
    pool_30 = generate_card_pool(sym_dist=(0.30, 0.50, 0.20), seed=42)
    pp_30 = measure_pair_precision(pool_30)
    m_30 = run_full_simulation("dual_threshold", dt_params, pool_30,
                                n_drafts=999, label="30% 1-sym")
    print(f"  30% 1-symbol: pair_precision={pp_30*100:.1f}%, late_sa={m_30['late_sa']:.2f}")

    print(f"\n  Summary:")
    print(f"    {'Distribution':<20} {'Pair Prec':>10} {'Late S/A':>10} {'Conv Pick':>10} {'StdDev':>10}")
    print(f"    {'15% 1-sym':<20} {pp_15*100:>9.1f}% {m_15['late_sa']:>10.2f} {m_15['convergence_pick']:>10.1f} {m_15['sa_stddev']:>10.2f}")
    print(f"    {'30% 1-sym':<20} {pp_30*100:>9.1f}% {m_30['late_sa']:>10.2f} {m_30['convergence_pick']:>10.1f} {m_30['sa_stddev']:>10.2f}")

    # ════════════════════════════════════════════════════════════════════════════
    # 6. DRAFT TRACES
    # ════════════════════════════════════════════════════════════════════════════
    print("\n" + "█" * 60)
    print("  DRAFT TRACES")
    print("█" * 60)

    # Trace 1: Early committer (committed player, Warriors)
    run_draft_trace(pool, "dual_threshold", dt_params, "committed",
                    target_archetype=6,
                    label="Trace 1: Early Committer (Warriors)",
                    rng=random.Random(100))

    # Trace 2: Flexible player (signal reader, auto-detected archetype)
    run_draft_trace(pool, "dual_threshold", dt_params, "signal",
                    target_archetype=2,
                    label="Trace 2: Signal Reader (Storm)",
                    rng=random.Random(200))

    # Trace 3: Power chaser
    run_draft_trace(pool, "dual_threshold", dt_params, "power",
                    target_archetype=4,
                    label="Trace 3: Power Chaser",
                    rng=random.Random(300))

    # ════════════════════════════════════════════════════════════════════════════
    # 7. SIDE-BY-SIDE COMPARISON TABLE
    # ════════════════════════════════════════════════════════════════════════════
    print("\n" + "█" * 60)
    print("  SIDE-BY-SIDE COMPARISON")
    print("█" * 60)

    all_results = {
        "DT (3/7)": dt_metrics,
        "Lane Lock": ll_metrics,
        "Pack Wide": pw_metrics,
    }

    print(f"\n  {'Metric':<45} ", end="")
    for name in all_results:
        print(f"{name:>12}", end="")
    print(f"  {'Target':>10}")
    print(f"  {'-'*90}")

    rows = [
        ("Picks 1-5: unique archs S/A", "early_unique_archs", ">=3"),
        ("Picks 1-5: S/A per pack", "early_sa", "<=2"),
        ("Picks 6+: S/A per pack", "late_sa", ">=2"),
        ("Picks 6+: off-arch per pack", "late_off", ">=0.5"),
        ("Convergence pick", "convergence_pick", "5-8"),
        ("Deck concentration", "deck_concentration", "60-90%"),
        ("Run-to-run overlap", "run_overlap", "<40%"),
        ("StdDev S/A (picks 6+)", "sa_stddev", ">=0.8"),
    ]

    for label, key, target in rows:
        print(f"  {label:<45} ", end="")
        for name, m in all_results.items():
            val = m[key]
            if key == "deck_concentration" or key == "run_overlap":
                print(f"{val*100:>11.1f}%", end="")
            elif key == "convergence_pick":
                print(f"{val:>12.1f}", end="")
            else:
                print(f"{val:>12.2f}", end="")
        print(f"  {target:>10}")

    print("\n\nSimulation complete.")
