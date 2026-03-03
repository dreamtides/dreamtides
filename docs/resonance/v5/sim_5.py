#!/usr/bin/env python3
"""
Resonance V5 — Agent 5: Hybrid Resonance-Triggered Pair Bonus
Simulation for conditional pack enhancement draft algorithm.

Algorithm (one-sentence):
"Draw 4 random cards; if any card's primary resonance matches your most-drafted
resonance, add 1 bonus card whose ordered pair matches your most-drafted pair."

Also implements:
- V3 Lane Locking baseline (threshold 3/8, primary=2)
- V4 Pack Widening baseline (cost 3, bonus 1, auto-spend on highest resonance)
- D4 Hybrid variant: 1 guaranteed pair slot + conditional bonus on remaining 3
- Parameter sensitivity sweeps
- Symbol distribution sensitivity
"""

import random
import math
from collections import defaultdict, Counter
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional

# ── Resonance and Archetype definitions ──────────────────────────────────────

class Resonance(Enum):
    EMBER = "Ember"
    STONE = "Stone"
    TIDE = "Tide"
    ZEPHYR = "Zephyr"

RESONANCES = list(Resonance)

# 8 archetypes on a circle
ARCHETYPES = [
    {"name": "Flash/Tempo/Prison",   "primary": Resonance.ZEPHYR, "secondary": Resonance.EMBER},
    {"name": "Blink/Flicker",        "primary": Resonance.EMBER,  "secondary": Resonance.ZEPHYR},
    {"name": "Storm/Spellslinger",   "primary": Resonance.EMBER,  "secondary": Resonance.STONE},
    {"name": "Self-Discard",         "primary": Resonance.STONE,  "secondary": Resonance.EMBER},
    {"name": "Self-Mill/Reanimator", "primary": Resonance.STONE,  "secondary": Resonance.TIDE},
    {"name": "Sacrifice/Abandon",    "primary": Resonance.TIDE,   "secondary": Resonance.STONE},
    {"name": "Warriors/Midrange",    "primary": Resonance.TIDE,   "secondary": Resonance.ZEPHYR},
    {"name": "Ramp/Spirit Animals",  "primary": Resonance.ZEPHYR, "secondary": Resonance.TIDE},
]

ARCHETYPE_NAMES = [a["name"] for a in ARCHETYPES]

def get_adjacent_indices(idx):
    """Return indices of the two adjacent archetypes on the circle."""
    return [(idx - 1) % 8, (idx + 1) % 8]

def get_fitness_tier(card_arch_idx, player_arch_idx):
    """
    Determine fitness tier of a card for a player's archetype.
    S-tier: home archetype
    A-tier: adjacent archetype sharing primary resonance
    B-tier: shares secondary resonance
    C/F: distant
    """
    if card_arch_idx == player_arch_idx:
        return "S"
    card = ARCHETYPES[card_arch_idx]
    player = ARCHETYPES[player_arch_idx]
    adj = get_adjacent_indices(player_arch_idx)
    if card_arch_idx in adj and card["primary"] == player["primary"]:
        return "A"
    if card["primary"] == player["secondary"] or card["secondary"] == player["secondary"]:
        return "B"
    return "F"

# ── Card model ───────────────────────────────────────────────────────────────

@dataclass
class SimCard:
    id: int
    symbols: list  # list of Resonance, ordered, 0-3 elements
    archetype_idx: int  # -1 for generic
    archetype_name: str
    power: float

    @property
    def primary_resonance(self):
        return self.symbols[0] if self.symbols else None

    @property
    def ordered_pair(self):
        if len(self.symbols) >= 2:
            return (self.symbols[0], self.symbols[1])
        return None

    def fitness_for(self, player_arch_idx):
        if self.archetype_idx == -1:
            return "B"  # generics are B-tier for everyone
        return get_fitness_tier(self.archetype_idx, player_arch_idx)

    def is_sa_for(self, player_arch_idx):
        tier = self.fitness_for(player_arch_idx)
        return tier in ("S", "A")


def generate_card_pool(sym_dist=(0.15, 0.60, 0.25), seed=None):
    """
    Generate 360 cards: ~40 per archetype (320) + 36 generic.
    sym_dist: (fraction 1-sym, fraction 2-sym, fraction 3-sym) among non-generic.
    """
    if seed is not None:
        random.seed(seed)
    cards = []
    card_id = 0

    # Generate archetype cards: 40 per archetype
    for arch_idx, arch in enumerate(ARCHETYPES):
        n_cards = 40
        prim = arch["primary"]
        sec = arch["secondary"]

        # Determine how many have 1, 2, 3 symbols
        n1 = round(n_cards * sym_dist[0])
        n3 = round(n_cards * sym_dist[2])
        n2 = n_cards - n1 - n3

        # 1-symbol cards: just the primary
        for _ in range(n1):
            cards.append(SimCard(
                id=card_id, symbols=[prim],
                archetype_idx=arch_idx, archetype_name=arch["name"],
                power=random.uniform(3, 8)
            ))
            card_id += 1

        # 2-symbol cards: primary + secondary (mostly), some primary + primary
        for i in range(n2):
            if random.random() < 0.85:
                syms = [prim, sec]
            else:
                syms = [prim, prim]
            cards.append(SimCard(
                id=card_id, symbols=syms,
                archetype_idx=arch_idx, archetype_name=arch["name"],
                power=random.uniform(3, 8)
            ))
            card_id += 1

        # 3-symbol cards: primary + secondary + (primary or secondary)
        for _ in range(n3):
            third = random.choice([prim, sec])
            cards.append(SimCard(
                id=card_id, symbols=[prim, sec, third],
                archetype_idx=arch_idx, archetype_name=arch["name"],
                power=random.uniform(3, 8)
            ))
            card_id += 1

    # 36 generic cards
    for _ in range(36):
        cards.append(SimCard(
            id=card_id, symbols=[],
            archetype_idx=-1, archetype_name="Generic",
            power=random.uniform(4, 9)
        ))
        card_id += 1

    return cards


# ── Player state tracking ────────────────────────────────────────────────────

class PlayerState:
    def __init__(self):
        self.drafted = []
        self.resonance_counts = Counter()  # weighted: primary=2, sec/tert=1
        self.pair_counts = Counter()       # (Res, Res) -> count
        self.raw_resonance_counts = Counter()  # unweighted: each symbol = 1
        self.pick_num = 0
        self.target_archetype_idx = None   # set when committed

    def draft_card(self, card):
        self.drafted.append(card)
        self.pick_num += 1
        for i, sym in enumerate(card.symbols):
            weight = 2 if i == 0 else 1
            self.resonance_counts[sym] += weight
            self.raw_resonance_counts[sym] += 1
        if card.ordered_pair:
            self.pair_counts[card.ordered_pair] += 1

    @property
    def most_drafted_resonance(self):
        if not self.resonance_counts:
            return None
        return self.resonance_counts.most_common(1)[0][0]

    @property
    def most_drafted_pair(self):
        if not self.pair_counts:
            return None
        return self.pair_counts.most_common(1)[0][0]

    @property
    def top_resonance_weighted(self):
        if not self.resonance_counts:
            return 0
        return self.resonance_counts.most_common(1)[0][1]


# ── Draft algorithms ─────────────────────────────────────────────────────────

def draw_random_pack(pool, n=4):
    """Draw n random cards from pool (with replacement)."""
    return random.choices(pool, k=n)


def algo_baseline(pool, player, pack_size=4):
    """Pure random baseline: draw 4 random cards."""
    return draw_random_pack(pool, pack_size)


def algo_hybrid_trigger(pool, player, bonus_count=1, trigger_mode="resonance"):
    """
    Hybrid Resonance-Triggered Pair Bonus.
    Draw 4 random cards; if any card's primary resonance matches player's
    most-drafted resonance, add bonus_count card(s) whose ordered pair matches
    player's most-drafted pair.

    trigger_mode:
      "resonance" — trigger if ANY of 4 has primary matching most-drafted resonance
      "pair" — trigger if ANY of 4 has ordered pair matching most-drafted pair
    """
    base_pack = draw_random_pack(pool, 4)

    if player.pick_num < 1:
        return base_pack

    top_res = player.most_drafted_resonance
    top_pair = player.most_drafted_pair

    triggered = False
    if trigger_mode == "resonance":
        if top_res is not None:
            for card in base_pack:
                if card.primary_resonance == top_res:
                    triggered = True
                    break
    elif trigger_mode == "pair":
        if top_pair is not None:
            for card in base_pack:
                if card.ordered_pair == top_pair:
                    triggered = True
                    break

    if triggered and top_pair is not None:
        # Add bonus card(s) whose ordered pair matches most-drafted pair
        pair_matched = [c for c in pool if c.ordered_pair == top_pair]
        if pair_matched:
            bonus = random.choices(pair_matched, k=bonus_count)
            return base_pack + bonus

    return base_pack


def algo_d4_hybrid(pool, player, threshold=3, bonus_count=1):
    """
    D4 Hybrid: 1 guaranteed pair slot + conditional resonance-triggered pair
    bonus on remaining 3 random cards.
    """
    top_pair = player.most_drafted_pair
    top_pair_count = player.pair_counts[top_pair] if top_pair else 0
    top_res = player.most_drafted_resonance

    pack = []

    # Guaranteed pair slot if threshold met
    if top_pair and top_pair_count >= threshold:
        pair_matched = [c for c in pool if c.ordered_pair == top_pair]
        if pair_matched:
            pack.append(random.choice(pair_matched))

    # Fill remaining with random
    remaining = 4 - len(pack)
    random_cards = draw_random_pack(pool, remaining)
    pack.extend(random_cards)

    # Conditional trigger on the random portion
    if top_res is not None and top_pair is not None and player.pick_num >= 1:
        triggered = False
        for card in random_cards:
            if card.primary_resonance == top_res:
                triggered = True
                break
        if triggered:
            pair_matched = [c for c in pool if c.ordered_pair == top_pair]
            if pair_matched:
                bonus = random.choices(pair_matched, k=bonus_count)
                pack.extend(bonus)

    return pack


def algo_lane_locking(pool, player, threshold1=3, threshold2=8):
    """
    V3 Lane Locking baseline.
    When weighted resonance count reaches threshold, one pack slot is permanently
    locked to that resonance. Second lock at threshold2.
    """
    pack = []
    top_res = player.most_drafted_resonance
    top_count = player.top_resonance_weighted

    locked_slots = 0
    if top_res and top_count >= threshold2:
        locked_slots = 2
    elif top_res and top_count >= threshold1:
        locked_slots = 1

    # Fill locked slots with resonance-matched cards
    if locked_slots > 0 and top_res:
        res_cards = [c for c in pool if c.primary_resonance == top_res]
        if res_cards:
            pack.extend(random.choices(res_cards, k=min(locked_slots, len(res_cards))))

    # Fill remaining with random
    remaining = 4 - len(pack)
    pack.extend(draw_random_pack(pool, remaining))

    return pack


def algo_pack_widening(pool, player, cost=3, bonus=1):
    """
    V4 Pack Widening baseline (auto-spend variant).
    Tokens accumulate from drafted symbols (primary=2, secondary/tertiary=1).
    When highest resonance reaches 'cost', auto-spend: add 'bonus' cards of that
    resonance, deduct cost.
    """
    pack = draw_random_pack(pool, 4)

    # Check if we can auto-spend
    top_res = player.most_drafted_resonance
    if top_res and player.resonance_counts[top_res] >= cost:
        res_cards = [c for c in pool if c.primary_resonance == top_res]
        if res_cards:
            pack.extend(random.choices(res_cards, k=bonus))
            # Deduct cost (we need to modify player state; handled after draft)
            player._pending_deduction = (top_res, cost)

    return pack


# ── Player strategies ────────────────────────────────────────────────────────

def pick_archetype_committed(pack, player):
    """Pick best card for target archetype. Commits around pick 5-6."""
    if player.target_archetype_idx is None:
        return pick_power_chaser(pack, player)

    # Pick card with best fitness for target archetype
    best_card = None
    best_score = -1
    for card in pack:
        tier = card.fitness_for(player.target_archetype_idx)
        score = {"S": 100, "A": 80, "B": 50, "F": 10}[tier] + card.power
        if score > best_score:
            best_score = score
            best_card = card
    return best_card


def pick_power_chaser(pack, player):
    """Pick highest power card regardless of archetype."""
    return max(pack, key=lambda c: c.power)


def pick_signal_reader(pack, player):
    """
    Evaluate which archetype has the most S/A cards in the pack and draft
    toward the open archetype. Commits later (around pick 8).
    """
    if player.pick_num < 8:
        # Before commitment, pick best overall value considering archetype diversity
        # Weight toward archetypes that appear more in packs
        arch_counts = Counter()
        for card in pack:
            if card.archetype_idx >= 0:
                arch_counts[card.archetype_idx] += 1

        best_card = None
        best_score = -1
        for card in pack:
            if card.archetype_idx >= 0:
                score = card.power + arch_counts[card.archetype_idx] * 3
            else:
                score = card.power
            if score > best_score:
                best_score = score
                best_card = card
        return best_card
    else:
        return pick_archetype_committed(pack, player)


# ── Simulation runner ────────────────────────────────────────────────────────

def determine_target_archetype(player, commit_pick=5):
    """
    Determine what archetype the player should commit to based on drafted cards.
    Uses pair counts primarily, falls back to resonance counts.
    """
    if player.pick_num < commit_pick:
        return None

    # Find archetype with most S/A cards drafted
    arch_sa = Counter()
    for card in player.drafted:
        if card.archetype_idx >= 0:
            arch_sa[card.archetype_idx] += 1

    if arch_sa:
        return arch_sa.most_common(1)[0][0]
    return random.randint(0, 7)


def run_single_draft(pool, algo_fn, strategy_fn, commit_pick=5, trace=False):
    """Run a single 30-pick draft. Returns detailed stats."""
    player = PlayerState()
    stats = {
        "sa_per_pack": [],
        "off_arch_per_pack": [],
        "unique_archs_per_pack": [],
        "pack_sizes": [],
        "fire_rate_hits": 0,
        "fire_rate_total": 0,
        "picks": [],
    }

    for pick in range(30):
        # Determine commitment
        if player.target_archetype_idx is None and player.pick_num >= commit_pick:
            player.target_archetype_idx = determine_target_archetype(player, commit_pick)

        # Generate pack
        pack = algo_fn(pool, player)
        stats["pack_sizes"].append(len(pack))

        # Track fire rate (pack size > 4 means bonus triggered)
        if player.pick_num >= commit_pick:
            stats["fire_rate_total"] += 1
            if len(pack) > 4:
                stats["fire_rate_hits"] += 1

        # Evaluate pack quality
        target = player.target_archetype_idx
        if target is not None:
            sa_count = sum(1 for c in pack if c.is_sa_for(target))
            off_count = sum(1 for c in pack if c.fitness_for(target) in ("F",))
            stats["sa_per_pack"].append((pick, sa_count, len(pack)))
            stats["off_arch_per_pack"].append((pick, off_count))

        # Count unique archetypes with S/A cards
        unique_archs = set()
        for card in pack:
            for ai in range(8):
                if card.is_sa_for(ai):
                    unique_archs.add(ai)
        stats["unique_archs_per_pack"].append(len(unique_archs))

        # Strategy picks a card
        chosen = strategy_fn(pack, player)

        if trace:
            stats["picks"].append({
                "pick": pick + 1,
                "pack_size": len(pack),
                "pack": [(c.archetype_name, [s.value for s in c.symbols],
                          c.fitness_for(target) if target is not None else "?")
                         for c in pack],
                "chosen": (chosen.archetype_name, [s.value for s in chosen.symbols]),
                "target_arch": ARCHETYPE_NAMES[target] if target is not None else "uncommitted",
                "top_res": player.most_drafted_resonance.value if player.most_drafted_resonance else "none",
                "top_pair": (player.most_drafted_pair[0].value, player.most_drafted_pair[1].value) if player.most_drafted_pair else "none",
                "sa_in_pack": sum(1 for c in pack if target is not None and c.is_sa_for(target)),
            })

        # Handle pack widening deduction
        if hasattr(player, '_pending_deduction'):
            res, cost = player._pending_deduction
            player.resonance_counts[res] -= cost
            del player._pending_deduction

        player.draft_card(chosen)

    # Final deck analysis
    target = player.target_archetype_idx
    if target is not None:
        deck_sa = sum(1 for c in player.drafted if c.is_sa_for(target))
        stats["deck_concentration"] = deck_sa / len(player.drafted)
    else:
        stats["deck_concentration"] = 0

    stats["target_archetype_idx"] = target
    stats["drafted_cards"] = [c.id for c in player.drafted]

    return stats


def run_simulations(pool, algo_fn, strategy_fn, n_runs=1000, commit_pick=5):
    """Run n_runs drafts and aggregate metrics."""
    all_stats = []
    for _ in range(n_runs):
        stats = run_single_draft(pool, algo_fn, strategy_fn, commit_pick)
        all_stats.append(stats)
    return all_stats


# ── Metrics computation ──────────────────────────────────────────────────────

def compute_metrics(all_stats, label=""):
    """Compute all 8 measurable targets + variance from simulation stats."""
    # Picks 1-5 metrics
    early_unique_archs = []
    early_sa_for_arch = []

    # Picks 6+ metrics
    late_sa = []
    late_off = []
    late_sa_per_pack_list = []

    # Convergence
    convergence_picks = []

    # Deck concentration
    deck_concs = []

    # Run-to-run variety
    all_drafted_ids = []

    # Archetype frequency
    arch_freq = Counter()

    for stats in all_stats:
        target = stats["target_archetype_idx"]
        if target is not None:
            arch_freq[target] += 1

        deck_concs.append(stats["deck_concentration"])
        all_drafted_ids.append(set(stats["drafted_cards"]))

        # Separate early/late picks
        for pick_idx, sa_count, pack_size in stats["sa_per_pack"]:
            if pick_idx < 5:
                early_sa_for_arch.append(sa_count)
            else:
                late_sa.append(sa_count)
                late_sa_per_pack_list.append(sa_count)

        for pick_idx, off_count in stats["off_arch_per_pack"]:
            if pick_idx >= 5:
                late_off.append(off_count)

        for u in stats["unique_archs_per_pack"][:5]:
            early_unique_archs.append(u)

        # Convergence: find first pick where running avg of last 3 packs >= 2.0
        sa_seq = [(p, sa) for p, sa, _ in stats["sa_per_pack"]]
        conv_pick = 30  # default: never converged
        for i in range(2, len(sa_seq)):
            recent = [sa_seq[j][1] for j in range(max(0, i-2), i+1)]
            if sum(recent) / len(recent) >= 2.0:
                conv_pick = sa_seq[i][0] + 1
                break
        convergence_picks.append(conv_pick)

    # Compute overlap for run-to-run variety
    overlaps = []
    sample_size = min(200, len(all_drafted_ids))
    for i in range(sample_size):
        j = (i + 1) % len(all_drafted_ids)
        if all_drafted_ids[i] and all_drafted_ids[j]:
            overlap = len(all_drafted_ids[i] & all_drafted_ids[j]) / max(
                len(all_drafted_ids[i] | all_drafted_ids[j]), 1)
            overlaps.append(overlap)

    # Fire rate
    total_fires = sum(s["fire_rate_hits"] for s in all_stats)
    total_opps = sum(s["fire_rate_total"] for s in all_stats)
    fire_rate = total_fires / max(total_opps, 1)

    metrics = {
        "label": label,
        "early_unique_archs": sum(early_unique_archs) / max(len(early_unique_archs), 1),
        "early_sa_for_arch": sum(early_sa_for_arch) / max(len(early_sa_for_arch), 1),
        "late_sa": sum(late_sa) / max(len(late_sa), 1),
        "late_off": sum(late_off) / max(len(late_off), 1),
        "convergence_pick": sum(convergence_picks) / max(len(convergence_picks), 1),
        "deck_concentration": sum(deck_concs) / max(len(deck_concs), 1),
        "run_variety_overlap": sum(overlaps) / max(len(overlaps), 1),
        "arch_freq_max": max(arch_freq.values()) / max(sum(arch_freq.values()), 1) if arch_freq else 0,
        "arch_freq_min": min(arch_freq.values()) / max(sum(arch_freq.values()), 1) if arch_freq else 0,
        "late_sa_stddev": (sum((x - sum(late_sa)/max(len(late_sa),1))**2 for x in late_sa) / max(len(late_sa),1))**0.5 if late_sa else 0,
        "fire_rate": fire_rate,
    }

    return metrics


def compute_per_archetype_convergence(pool, algo_fn, n_runs=125, commit_pick=5):
    """
    For each of the 8 archetypes, run simulations where committed player
    targets that specific archetype. Report avg convergence pick.
    """
    results = {}
    for arch_idx in range(8):
        convergence_picks = []
        for _ in range(n_runs):
            player = PlayerState()
            player.target_archetype_idx = arch_idx  # Force commitment from start

            for pick in range(30):
                pack = algo_fn(pool, player)

                # Find SA count
                sa_count = sum(1 for c in pack if c.is_sa_for(arch_idx))

                # Pick best card for target
                chosen = pick_archetype_committed(pack, player)

                # Handle pack widening deduction
                if hasattr(player, '_pending_deduction'):
                    res, cost = player._pending_deduction
                    player.resonance_counts[res] -= cost
                    del player._pending_deduction

                player.draft_card(chosen)

            # Determine convergence pick (where rolling 3-pack avg >= 2.0)
            # Re-simulate to track pack SA
            player2 = PlayerState()
            player2.target_archetype_idx = arch_idx
            sa_sequence = []
            for pick in range(30):
                pack = algo_fn(pool, player2)
                sa_count = sum(1 for c in pack if c.is_sa_for(arch_idx))
                sa_sequence.append(sa_count)
                chosen = pick_archetype_committed(pack, player2)
                if hasattr(player2, '_pending_deduction'):
                    res, cost = player2._pending_deduction
                    player2.resonance_counts[res] -= cost
                    del player2._pending_deduction
                player2.draft_card(chosen)

            conv_pick = 30
            for i in range(2, len(sa_sequence)):
                recent = sa_sequence[max(0, i-2):i+1]
                if sum(recent)/len(recent) >= 2.0:
                    conv_pick = i + 1
                    break
            convergence_picks.append(conv_pick)

        results[ARCHETYPE_NAMES[arch_idx]] = sum(convergence_picks) / len(convergence_picks)
    return results


def print_metrics(metrics, targets=None):
    """Print metrics table with pass/fail."""
    print(f"\n{'='*70}")
    print(f"  {metrics['label']}")
    print(f"{'='*70}")

    checks = [
        ("Picks 1-5: unique archs w/ S/A per pack", "early_unique_archs", ">= 3", lambda v: v >= 3),
        ("Picks 1-5: S/A for emerging arch per pack", "early_sa_for_arch", "<= 2", lambda v: v <= 2),
        ("Picks 6+: S/A for committed arch per pack", "late_sa", ">= 2.0", lambda v: v >= 2.0),
        ("Picks 6+: off-archetype (C/F) per pack", "late_off", ">= 0.5", lambda v: v >= 0.5),
        ("Convergence pick", "convergence_pick", "5-8", lambda v: 5 <= v <= 8),
        ("Deck archetype concentration", "deck_concentration", "60-90%", lambda v: 0.6 <= v <= 0.9),
        ("Run-to-run overlap", "run_variety_overlap", "< 40%", lambda v: v < 0.4),
        ("Archetype freq max", "arch_freq_max", "<= 20%", lambda v: v <= 0.2),
        ("Archetype freq min", "arch_freq_min", ">= 5%", lambda v: v >= 0.05),
        ("StdDev S/A per pack (6+)", "late_sa_stddev", ">= 0.8", lambda v: v >= 0.8),
        ("Fire rate (bonus trigger)", "fire_rate", "info", lambda v: True),
    ]

    print(f"{'Metric':<48} {'Target':<12} {'Actual':<12} {'Pass/Fail':<10}")
    print("-" * 82)
    for name, key, target_str, check_fn in checks:
        val = metrics[key]
        if isinstance(val, float):
            val_str = f"{val:.3f}"
        else:
            val_str = str(val)
        passed = "PASS" if check_fn(val) else "FAIL"
        if target_str == "info":
            passed = "—"
        print(f"{name:<48} {target_str:<12} {val_str:<12} {passed:<10}")


def print_convergence_table(results, label=""):
    """Print per-archetype convergence table."""
    print(f"\nPer-Archetype Convergence — {label}")
    print(f"{'Archetype':<28} {'Avg Convergence Pick':<20}")
    print("-" * 48)
    for arch_name in ARCHETYPE_NAMES:
        val = results.get(arch_name, float('nan'))
        print(f"{arch_name:<28} {val:.1f}")


# ── Pair precision measurement ───────────────────────────────────────────────

def measure_pair_precision(pool):
    """
    Measure actual pair precision: for cards with 2+ symbols, what fraction
    whose ordered pair matches an archetype is actually S-tier for that archetype?
    """
    pair_to_arch = {}
    for arch_idx, arch in enumerate(ARCHETYPES):
        pair = (arch["primary"], arch["secondary"])
        pair_to_arch[pair] = arch_idx

    total_with_pair = 0
    s_tier_match = 0
    a_tier_match = 0

    for card in pool:
        if card.ordered_pair:
            total_with_pair += 1
            if card.ordered_pair in pair_to_arch:
                target_arch = pair_to_arch[card.ordered_pair]
                tier = card.fitness_for(target_arch)
                if tier == "S":
                    s_tier_match += 1
                elif tier == "A":
                    a_tier_match += 1

    print(f"\n=== Pair Precision Analysis ===")
    print(f"Cards with 2+ symbols: {total_with_pair}")
    print(f"Cards whose pair matches an archetype: {s_tier_match + a_tier_match}")
    print(f"  S-tier (home arch): {s_tier_match} ({100*s_tier_match/max(total_with_pair,1):.1f}%)")
    print(f"  A-tier (adjacent): {a_tier_match} ({100*a_tier_match/max(total_with_pair,1):.1f}%)")
    print(f"  Pair precision (S-tier): {100*s_tier_match/max(s_tier_match+a_tier_match,1):.1f}%")


# ── Draft traces ─────────────────────────────────────────────────────────────

def run_trace(pool, algo_fn, strategy_fn, commit_pick, label="", forced_arch=None):
    """Run a single traced draft and print pick-by-pick details."""
    player = PlayerState()
    if forced_arch is not None:
        player.target_archetype_idx = forced_arch

    print(f"\n{'='*70}")
    print(f"  DRAFT TRACE: {label}")
    print(f"{'='*70}")

    for pick in range(30):
        if player.target_archetype_idx is None and player.pick_num >= commit_pick:
            player.target_archetype_idx = determine_target_archetype(player, commit_pick)

        pack = algo_fn(pool, player)
        target = player.target_archetype_idx
        target_name = ARCHETYPE_NAMES[target] if target is not None else "uncommitted"

        chosen = strategy_fn(pack, player)

        sa_count = sum(1 for c in pack if target is not None and c.is_sa_for(target))

        if pick < 12 or pick >= 27 or (len(pack) > 4):
            print(f"\nPick {pick+1} | Target: {target_name} | "
                  f"Top res: {player.most_drafted_resonance.value if player.most_drafted_resonance else 'none'} | "
                  f"Top pair: {(player.most_drafted_pair[0].value + '/' + player.most_drafted_pair[1].value) if player.most_drafted_pair else 'none'} | "
                  f"Pack({len(pack)}): S/A={sa_count}")
            for i, c in enumerate(pack):
                tier = c.fitness_for(target) if target is not None else "?"
                marker = " <-- PICKED" if c is chosen else ""
                bonus_marker = " [BONUS]" if i >= 4 else ""
                print(f"  [{'/'.join(s.value for s in c.symbols) or 'generic'}] "
                      f"{c.archetype_name} (tier={tier}, pwr={c.power:.1f})"
                      f"{bonus_marker}{marker}")

        # Handle pack widening deduction
        if hasattr(player, '_pending_deduction'):
            res, cost = player._pending_deduction
            player.resonance_counts[res] -= cost
            del player._pending_deduction

        player.draft_card(chosen)

    target = player.target_archetype_idx
    if target is not None:
        deck_sa = sum(1 for c in player.drafted if c.is_sa_for(target))
        print(f"\nFinal deck: {deck_sa}/{len(player.drafted)} S/A for "
              f"{ARCHETYPE_NAMES[target]} ({100*deck_sa/len(player.drafted):.1f}%)")


# ── Main simulation ──────────────────────────────────────────────────────────

def main():
    random.seed(42)
    pool = generate_card_pool(sym_dist=(0.15, 0.60, 0.25), seed=42)

    print(f"Pool size: {len(pool)}")
    print(f"  Generic: {sum(1 for c in pool if c.archetype_idx == -1)}")
    print(f"  1-symbol: {sum(1 for c in pool if len(c.symbols) == 1)}")
    print(f"  2-symbol: {sum(1 for c in pool if len(c.symbols) == 2)}")
    print(f"  3-symbol: {sum(1 for c in pool if len(c.symbols) == 3)}")

    # ── Pair precision ───────────────────────────────────────────────────
    measure_pair_precision(pool)

    # ── Baseline: Pure Random ────────────────────────────────────────────
    print("\n\n" + "="*70)
    print("  SECTION 1: BASELINE COMPARISONS")
    print("="*70)

    stats_baseline = run_simulations(pool, algo_baseline, pick_archetype_committed, n_runs=1000)
    m_baseline = compute_metrics(stats_baseline, "Pure Random Baseline")
    print_metrics(m_baseline)

    # ── V3 Lane Locking ──────────────────────────────────────────────────
    stats_ll = run_simulations(pool, lambda p, pl: algo_lane_locking(p, pl, 3, 8),
                               pick_archetype_committed, n_runs=1000)
    m_ll = compute_metrics(stats_ll, "V3 Lane Locking (threshold 3/8)")
    print_metrics(m_ll)

    # ── V4 Pack Widening (auto-spend) ────────────────────────────────────
    stats_pw = run_simulations(pool, lambda p, pl: algo_pack_widening(p, pl, 3, 1),
                               pick_archetype_committed, n_runs=1000)
    m_pw = compute_metrics(stats_pw, "V4 Pack Widening (cost 3, bonus 1, auto)")
    print_metrics(m_pw)

    # ── Main Algorithm: Hybrid Resonance-Triggered Pair Bonus ────────────
    print("\n\n" + "="*70)
    print("  SECTION 2: HYBRID RESONANCE-TRIGGERED PAIR BONUS")
    print("="*70)

    # Primary configuration: resonance trigger, +1 bonus
    stats_main = run_simulations(pool,
        lambda p, pl: algo_hybrid_trigger(p, pl, bonus_count=1, trigger_mode="resonance"),
        pick_archetype_committed, n_runs=1000)
    m_main = compute_metrics(stats_main, "Hybrid Trigger: resonance trigger, +1 bonus")
    print_metrics(m_main)

    # ── Parameter sensitivity: +2 bonus ──────────────────────────────────
    print("\n\n" + "="*70)
    print("  SECTION 3: PARAMETER SENSITIVITY")
    print("="*70)

    stats_b2 = run_simulations(pool,
        lambda p, pl: algo_hybrid_trigger(p, pl, bonus_count=2, trigger_mode="resonance"),
        pick_archetype_committed, n_runs=1000)
    m_b2 = compute_metrics(stats_b2, "Hybrid Trigger: resonance trigger, +2 bonus")
    print_metrics(m_b2)

    # ── Parameter sensitivity: pair trigger instead of resonance trigger ──
    stats_pt = run_simulations(pool,
        lambda p, pl: algo_hybrid_trigger(p, pl, bonus_count=1, trigger_mode="pair"),
        pick_archetype_committed, n_runs=1000)
    m_pt = compute_metrics(stats_pt, "Hybrid Trigger: pair trigger, +1 bonus")
    print_metrics(m_pt)

    # pair trigger +2
    stats_pt2 = run_simulations(pool,
        lambda p, pl: algo_hybrid_trigger(p, pl, bonus_count=2, trigger_mode="pair"),
        pick_archetype_committed, n_runs=1000)
    m_pt2 = compute_metrics(stats_pt2, "Hybrid Trigger: pair trigger, +2 bonus")
    print_metrics(m_pt2)

    # ── D4 Hybrid: guaranteed pair slot + conditional bonus ──────────────
    print("\n\n" + "="*70)
    print("  SECTION 4: D4 HYBRID (1 GUARANTEED PAIR SLOT + CONDITIONAL BONUS)")
    print("="*70)

    stats_d4h = run_simulations(pool,
        lambda p, pl: algo_d4_hybrid(p, pl, threshold=3, bonus_count=1),
        pick_archetype_committed, n_runs=1000)
    m_d4h = compute_metrics(stats_d4h, "D4 Hybrid: 1 pair slot (thresh 3) + cond bonus")
    print_metrics(m_d4h)

    # D4 hybrid with threshold 2
    stats_d4h2 = run_simulations(pool,
        lambda p, pl: algo_d4_hybrid(p, pl, threshold=2, bonus_count=1),
        pick_archetype_committed, n_runs=1000)
    m_d4h2 = compute_metrics(stats_d4h2, "D4 Hybrid: 1 pair slot (thresh 2) + cond bonus")
    print_metrics(m_d4h2)

    # ── Symbol distribution sensitivity ──────────────────────────────────
    print("\n\n" + "="*70)
    print("  SECTION 5: SYMBOL DISTRIBUTION SENSITIVITY")
    print("="*70)

    # High 1-symbol (30%)
    pool_high1 = generate_card_pool(sym_dist=(0.30, 0.50, 0.20), seed=42)
    stats_h1 = run_simulations(pool_high1,
        lambda p, pl: algo_hybrid_trigger(p, pl, bonus_count=1, trigger_mode="resonance"),
        pick_archetype_committed, n_runs=1000)
    m_h1 = compute_metrics(stats_h1, "Hybrid Trigger w/ 30% 1-sym, 50% 2-sym, 20% 3-sym")
    print_metrics(m_h1)

    # Standard distribution for comparison
    print(f"\n  (Standard 15/60/25 reference: late S/A = {m_main['late_sa']:.3f})")

    # ── Player strategy comparison ───────────────────────────────────────
    print("\n\n" + "="*70)
    print("  SECTION 6: PLAYER STRATEGY COMPARISON")
    print("="*70)

    algo = lambda p, pl: algo_hybrid_trigger(p, pl, bonus_count=1, trigger_mode="resonance")

    stats_power = run_simulations(pool, algo, pick_power_chaser, n_runs=1000, commit_pick=30)
    m_power = compute_metrics(stats_power, "Power Chaser (hybrid trigger)")
    print_metrics(m_power)

    stats_signal = run_simulations(pool, algo, pick_signal_reader, n_runs=1000, commit_pick=8)
    m_signal = compute_metrics(stats_signal, "Signal Reader (hybrid trigger, commit 8)")
    print_metrics(m_signal)

    # ── Per-archetype convergence ────────────────────────────────────────
    print("\n\n" + "="*70)
    print("  SECTION 7: PER-ARCHETYPE CONVERGENCE")
    print("="*70)

    algo_main = lambda p, pl: algo_hybrid_trigger(p, pl, bonus_count=1, trigger_mode="resonance")
    conv_main = compute_per_archetype_convergence(pool, algo_main, n_runs=125)
    print_convergence_table(conv_main, "Hybrid Trigger (resonance +1)")

    algo_ll_fn = lambda p, pl: algo_lane_locking(p, pl, 3, 8)
    conv_ll = compute_per_archetype_convergence(pool, algo_ll_fn, n_runs=125)
    print_convergence_table(conv_ll, "V3 Lane Locking")

    algo_pw_fn = lambda p, pl: algo_pack_widening(p, pl, 3, 1)
    conv_pw = compute_per_archetype_convergence(pool, algo_pw_fn, n_runs=125)
    print_convergence_table(conv_pw, "V4 Pack Widening (auto)")

    algo_d4h_fn = lambda p, pl: algo_d4_hybrid(p, pl, threshold=3, bonus_count=1)
    conv_d4h = compute_per_archetype_convergence(pool, algo_d4h_fn, n_runs=125)
    print_convergence_table(conv_d4h, "D4 Hybrid")

    # ── Draft traces ─────────────────────────────────────────────────────
    print("\n\n" + "="*70)
    print("  SECTION 8: DRAFT TRACES")
    print("="*70)

    # Trace 1: Early committer (pick 3)
    run_trace(pool, algo_main, pick_archetype_committed, commit_pick=3,
              label="Early Committer (Warriors, commit pick 3)", forced_arch=6)

    # Trace 2: Flexible player (pick 10)
    run_trace(pool, algo_main, pick_archetype_committed, commit_pick=10,
              label="Flexible Player (uncommitted until pick 10)")

    # Trace 3: Signal reader (pick 8)
    run_trace(pool, algo_main, pick_signal_reader, commit_pick=8,
              label="Signal Reader (commit pick 8)")

    # ── Summary comparison table ─────────────────────────────────────────
    print("\n\n" + "="*70)
    print("  SECTION 9: SUMMARY COMPARISON")
    print("="*70)

    all_m = [m_baseline, m_ll, m_pw, m_main, m_b2, m_d4h, m_d4h2]
    print(f"\n{'Algorithm':<52} {'Late S/A':<10} {'StdDev':<8} {'Conv':<8} {'Deck%':<8} {'Fire%':<8}")
    print("-" * 94)
    for m in all_m:
        print(f"{m['label']:<52} {m['late_sa']:.2f}      {m['late_sa_stddev']:.2f}    "
              f"{m['convergence_pick']:.1f}    {m['deck_concentration']:.2f}    "
              f"{m['fire_rate']:.2f}")


if __name__ == "__main__":
    main()
