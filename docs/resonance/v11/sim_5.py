#!/usr/bin/env python3
"""
Simulation Agent 5: Asymmetric Replacement (Design 6, Timing-Adjusted)
V11 Round 4 — SIM-5

Algorithm: Continuous visible market. AI picks individually replaced from 240-card
reserve; player picks permanently deplete the pool. Single scheduled refill at
pick 16 (+20 cards biased toward archetypes < 10 cards). Informational snapshots
at picks 10, 20. Rolling 5-pick trend windows. This is the "Advanced" candidate.

Parameters:
  - Starting pool: 120 cards (8 archetypes x 15)
  - Replacement reserve: 240 cards (8 archetypes x 30)
  - 30 sequential picks, each pick cycle:
      1) 5 AIs each draft 1 card (highest fitness for their archetype)
      2) Each AI pick replaced by 1 card from reserve (same archetype)
      3) Player sees 5-card pack drawn from pool, picks 1. No replacement.
  - Scheduled refill at pick 16: +20 cards from reserve, biased toward
    archetypes currently below 10 cards in pool
  - AI saturation at 16 on-archetype cards (then pick highest-power generic)
  - 5 AIs on 5 archetypes, 3 open lanes
  - Player strategies: committed, signal_reader, power_chaser
  - 1000 drafts x 3 strategies, Graduated Realistic fitness
"""

import random
import math
from dataclasses import dataclass, field
from collections import defaultdict
from typing import Optional

# ============================================================
# Constants
# ============================================================
NUM_DRAFTS = 1000
NUM_PICKS = 30
PACK_SIZE = 5
STARTING_POOL = 120
CARDS_PER_ARCHETYPE_POOL = 15
RESERVE_PER_ARCHETYPE = 30
RESERVE_TOTAL = 240
NUM_AIS = 5
OPEN_LANES = 3
AI_SATURATION = 16
REFILL_PICK = 16
REFILL_SIZE = 20

ARCHETYPES = [
    ("Flash",        "Zephyr", "Ember"),
    ("Blink",        "Ember",  "Zephyr"),
    ("Storm",        "Ember",  "Stone"),
    ("Self-Discard", "Stone",  "Ember"),
    ("Self-Mill",    "Stone",  "Tide"),
    ("Sacrifice",    "Tide",   "Stone"),
    ("Warriors",     "Tide",   "Zephyr"),
    ("Ramp",         "Zephyr", "Tide"),
]

ARCHETYPE_NAMES = [a[0] for a in ARCHETYPES]
ARCH_BY_NAME = {a[0]: (a[1], a[2]) for a in ARCHETYPES}
ARCH_IDX = {a[0]: i for i, a in enumerate(ARCHETYPES)}

# Sibling pairs (share primary resonance)
SIBLING_PAIRS = {}
for i, (a1, r1a, _) in enumerate(ARCHETYPES):
    for j, (a2, r1b, _) in enumerate(ARCHETYPES):
        if i != j and r1a == r1b:
            SIBLING_PAIRS[a1] = a2

# Graduated Realistic fitness: sibling A-tier rates
SIBLING_RATES = {
    "Warriors": 0.50, "Sacrifice": 0.50,
    "Self-Discard": 0.40, "Self-Mill": 0.40,
    "Blink": 0.30, "Storm": 0.30,
    "Flash": 0.25, "Ramp": 0.25,
}

# Pessimistic fitness: sibling A-tier rates reduced
PESSIMISTIC_RATES = {
    "Warriors": 0.35, "Sacrifice": 0.35,
    "Self-Discard": 0.25, "Self-Mill": 0.25,
    "Blink": 0.15, "Storm": 0.15,
    "Flash": 0.10, "Ramp": 0.10,
}


def get_sibling(arch):
    return SIBLING_PAIRS.get(arch, None)


def get_sibling_rate(arch, rates):
    return rates.get(arch, 0.25)


# ============================================================
# Card Model
# ============================================================
@dataclass
class SimCard:
    id: int
    archetype: str
    primary_res: str
    secondary_res: str
    visible_symbols: list
    power: float
    pair_affinity: float
    cross_affinity: float
    tier_cache: dict = field(default_factory=dict)

    def __hash__(self):
        return self.id

    def __eq__(self, other):
        return isinstance(other, SimCard) and self.id == other.id


_next_card_id = 0


def make_card(archetype, rng):
    """Create a single card for the given archetype."""
    global _next_card_id
    r1, r2 = ARCH_BY_NAME[archetype]

    # Visible symbol distribution: ~11% generic, ~79% single, ~10% dual
    roll = rng.random()
    if roll < 0.11:
        visible = []  # generic-looking (no resonance symbols)
    elif roll < 0.90:
        visible = [r1]  # single symbol (primary)
    else:
        visible = [r1, r2]  # dual symbol

    card = SimCard(
        id=_next_card_id,
        archetype=archetype,
        primary_res=r1,
        secondary_res=r2,
        visible_symbols=visible,
        power=round(rng.uniform(2.0, 9.5), 1),
        pair_affinity=round(rng.random(), 3),
        cross_affinity=round(rng.random(), 3),
    )
    _next_card_id += 1
    return card


def compute_tier(card, target_arch, rates):
    """Compute S/A/C/F tier for a card relative to target archetype."""
    cache_key = (target_arch, id(rates) if isinstance(rates, dict) else 0)
    if cache_key in card.tier_cache:
        return card.tier_cache[cache_key]

    if card.archetype == target_arch:
        # Home cards: top ~30% S, next ~30% A, rest C
        if card.pair_affinity >= 0.70:
            tier = "S"
        elif card.pair_affinity >= 0.35:
            tier = "A"
        else:
            tier = "C"
    elif card.archetype == get_sibling(target_arch):
        rate = get_sibling_rate(target_arch, rates)
        if card.cross_affinity >= (1.0 - rate):
            tier = "A"
        else:
            tier = "C"
    else:
        tier = "F"

    card.tier_cache[cache_key] = tier
    return tier


def is_sa(card, target_arch, rates):
    return compute_tier(card, target_arch, rates) in ("S", "A")


def fitness_score(card, target_arch, rates):
    """Fitness score for AI/player pick logic. Higher = better fit."""
    tier = compute_tier(card, target_arch, rates)
    tier_bonus = {"S": 10.0, "A": 7.0, "C": 2.0, "F": 0.0}[tier]
    return tier_bonus + card.power * 0.1


# ============================================================
# Pool and Reserve Generation
# ============================================================
def generate_pool_and_reserve(rng):
    """Generate starting pool (120 cards) and reserve (240 cards)."""
    global _next_card_id
    _next_card_id = 0

    pool = []
    reserve = defaultdict(list)  # archetype -> list of cards

    for arch_name, _, _ in ARCHETYPES:
        # Starting pool: 15 per archetype
        for _ in range(CARDS_PER_ARCHETYPE_POOL):
            pool.append(make_card(arch_name, rng))

        # Reserve: 30 per archetype
        for _ in range(RESERVE_PER_ARCHETYPE):
            reserve[arch_name].append(make_card(arch_name, rng))

    return pool, reserve


# ============================================================
# AI Drafter
# ============================================================
class AIDrafter:
    def __init__(self, archetype):
        self.archetype = archetype
        self.on_arch_drafted = 0

    def pick(self, pool, rates):
        """Pick 1 card from pool. Highest fitness for assigned archetype.
        After saturation (16 on-archetype), pick highest-power generic."""
        if self.on_arch_drafted >= AI_SATURATION:
            # Saturated: pick highest power from any card
            if not pool:
                return None
            return max(pool, key=lambda c: c.power)

        # Pick highest fitness for assigned archetype
        if not pool:
            return None
        best = max(pool, key=lambda c: fitness_score(c, self.archetype, rates))
        if best.archetype == self.archetype or best.archetype == get_sibling(self.archetype):
            self.on_arch_drafted += 1
        return best


# ============================================================
# Pack Construction
# ============================================================
def build_pack(pool, rng):
    """Build a 5-card pack from the pool, weighted by archetype distribution.

    Each pick, the player sees 5 cards drawn from the current pool. Weighting
    ensures archetype representation roughly mirrors pool composition.
    """
    if len(pool) <= PACK_SIZE:
        return list(pool)

    # Weight cards by archetype scarcity: rarer archetypes slightly upweighted
    # to prevent pack homogeneity from large-archetype dominance
    arch_counts = defaultdict(int)
    for c in pool:
        arch_counts[c.archetype] += 1

    total = len(pool)
    weights = []
    for c in pool:
        # Inverse weighting: rarer archetypes get slight upweight
        count = arch_counts[c.archetype]
        w = 1.0 / max(1, count) * total / 8.0
        # Clamp to avoid extreme weights
        w = max(0.5, min(3.0, w))
        weights.append(w)

    # Weighted sample without replacement
    selected = []
    pool_copy = list(pool)
    w_copy = list(weights)

    for _ in range(PACK_SIZE):
        if not pool_copy:
            break
        total_w = sum(w_copy)
        if total_w <= 0:
            break
        r = rng.random() * total_w
        cumulative = 0
        chosen_idx = 0
        for idx, w in enumerate(w_copy):
            cumulative += w
            if cumulative >= r:
                chosen_idx = idx
                break
        selected.append(pool_copy[chosen_idx])
        pool_copy.pop(chosen_idx)
        w_copy.pop(chosen_idx)

    return selected


# ============================================================
# Scheduled Refill at Pick 16
# ============================================================
def do_scheduled_refill(pool, reserve, rng):
    """Add 20 cards from reserve, biased toward archetypes < 10 cards in pool."""
    # Count cards per archetype in pool
    arch_counts = defaultdict(int)
    for c in pool:
        arch_counts[c.archetype] += 1

    # Compute deficit: how many cards below 10 each archetype is
    deficits = {}
    for arch_name in ARCHETYPE_NAMES:
        count = arch_counts.get(arch_name, 0)
        if count < 10:
            deficits[arch_name] = 10 - count
        else:
            deficits[arch_name] = 0

    total_deficit = sum(deficits.values())

    if total_deficit == 0:
        # No archetype below 10; distribute evenly among all with available reserve
        available = [a for a in ARCHETYPE_NAMES if reserve[a]]
        if not available:
            return pool
        per_arch = max(1, REFILL_SIZE // len(available))
        added = 0
        for arch in available:
            for _ in range(per_arch):
                if added >= REFILL_SIZE:
                    break
                if reserve[arch]:
                    pool.append(reserve[arch].pop(0))
                    added += 1
        return pool

    # Distribute proportionally to deficit
    added = 0
    for arch_name in ARCHETYPE_NAMES:
        if deficits[arch_name] <= 0:
            continue
        # Proportional share
        share = int(round(REFILL_SIZE * deficits[arch_name] / total_deficit))
        share = max(1, share)  # at least 1 if any deficit
        for _ in range(share):
            if added >= REFILL_SIZE:
                break
            if reserve[arch_name]:
                pool.append(reserve[arch_name].pop(0))
                added += 1

    # If we haven't filled all 20 due to rounding, add more from largest deficits
    deficit_order = sorted(deficits.keys(), key=lambda a: deficits[a], reverse=True)
    for arch in deficit_order:
        while added < REFILL_SIZE and reserve[arch]:
            pool.append(reserve[arch].pop(0))
            added += 1
        if added >= REFILL_SIZE:
            break

    return pool


# ============================================================
# Player Strategies
# ============================================================
def player_committed(pack, pick_num, state, rng, rates):
    """Committed: random archetype at pick 1, draft for it throughout."""
    if not pack:
        return None

    if "arch" not in state:
        # Commit to a random open-lane archetype at pick 1
        state["arch"] = rng.choice(state["open_lanes"])

    arch = state["arch"]
    sa_cards = [c for c in pack if is_sa(c, arch, rates)]
    if sa_cards:
        return max(sa_cards, key=lambda c: fitness_score(c, arch, rates))
    # Fallback: best fitness
    return max(pack, key=lambda c: fitness_score(c, arch, rates))


def player_signal_reader(pack, pick_num, state, rng, rates):
    """Signal-reader: explores picks 1-4, commits at pick 5 to the open-lane
    archetype with the most S/A remaining in the pool."""
    if not pack:
        return None

    if pick_num <= 4:
        # Take highest power during exploration
        return max(pack, key=lambda c: c.power)

    if "arch" not in state:
        # At pick 5, commit to the open lane with most S/A in pool
        pool = state["current_pool"]
        open_lanes = state["open_lanes"]
        best_arch = None
        best_sa_count = -1
        for arch in open_lanes:
            sa_count = sum(1 for c in pool if is_sa(c, arch, rates))
            if sa_count > best_sa_count:
                best_sa_count = sa_count
                best_arch = arch
        state["arch"] = best_arch if best_arch else rng.choice(open_lanes)

    arch = state["arch"]
    sa_cards = [c for c in pack if is_sa(c, arch, rates)]
    if sa_cards:
        return max(sa_cards, key=lambda c: fitness_score(c, arch, rates))
    return max(pack, key=lambda c: fitness_score(c, arch, rates))


def player_power_chaser(pack, pick_num, state, rng, rates):
    """Power-chaser: always pick highest power regardless of archetype."""
    if not pack:
        return None
    return max(pack, key=lambda c: c.power)


STRATEGIES = {
    "committed": player_committed,
    "signal_reader": player_signal_reader,
    "power_chaser": player_power_chaser,
}


# ============================================================
# Single Draft
# ============================================================
@dataclass
class DraftResult:
    player_picks: list = field(default_factory=list)
    packs_shown: list = field(default_factory=list)
    committed_archetype: Optional[str] = None
    sa_per_pack: list = field(default_factory=list)
    pool_sizes: list = field(default_factory=list)
    pool_snapshots: dict = field(default_factory=dict)
    open_lanes: list = field(default_factory=list)
    ai_archetypes: list = field(default_factory=list)
    picks_completed: int = 0
    archetype_pool_counts: dict = field(default_factory=dict)
    sa_density_trajectory: list = field(default_factory=list)
    trend_windows: list = field(default_factory=list)


def run_single_draft(rng, strategy_name, rates=None):
    """Run one 30-pick draft with asymmetric replacement."""
    if rates is None:
        rates = SIBLING_RATES

    pool_cards, reserve = generate_pool_and_reserve(rng)
    pool = list(pool_cards)

    # Assign 5 AIs to 5 of 8 archetypes
    arch_indices = list(range(8))
    rng.shuffle(arch_indices)
    ai_arch_indices = arch_indices[:NUM_AIS]
    open_arch_indices = arch_indices[NUM_AIS:]

    ai_archetypes = [ARCHETYPE_NAMES[i] for i in ai_arch_indices]
    open_lanes = [ARCHETYPE_NAMES[i] for i in open_arch_indices]

    ais = [AIDrafter(arch) for arch in ai_archetypes]

    strategy_fn = STRATEGIES[strategy_name]
    state = {
        "open_lanes": open_lanes,
        "current_pool": pool,
    }

    result = DraftResult()
    result.open_lanes = open_lanes
    result.ai_archetypes = ai_archetypes

    # Track per-pick archetype counts and S/A density
    recent_sa_counts = []  # for rolling trend windows

    for pick_num in range(1, NUM_PICKS + 1):
        # ── Snapshot pool composition at key picks ──
        if pick_num in (1, 10, 16, 20, 30) or (pick_num == 16):
            snap = {}
            for arch_name in ARCHETYPE_NAMES:
                arch_cards = [c for c in pool if c.archetype == arch_name]
                sa_count = sum(1 for c in arch_cards if is_sa(c, arch_name, rates))
                snap[arch_name] = {"total": len(arch_cards), "sa": sa_count}
            result.pool_snapshots[f"pick_{pick_num}_pre"] = snap

        # ── Scheduled refill at pick 16 ──
        if pick_num == REFILL_PICK:
            pool = do_scheduled_refill(pool, reserve, rng)
            state["current_pool"] = pool
            # Post-refill snapshot
            snap = {}
            for arch_name in ARCHETYPE_NAMES:
                arch_cards = [c for c in pool if c.archetype == arch_name]
                sa_count = sum(1 for c in arch_cards if is_sa(c, arch_name, rates))
                snap[arch_name] = {"total": len(arch_cards), "sa": sa_count}
            result.pool_snapshots["pick_16_post"] = snap

        # ── Step 1: 5 AIs each draft 1 card ──
        for ai in ais:
            if not pool:
                break
            chosen = ai.pick(pool, rates)
            if chosen and chosen in pool:
                pool.remove(chosen)

                # ── Step 2: Replace AI pick from reserve ──
                if reserve[ai.archetype]:
                    replacement = reserve[ai.archetype].pop(0)
                    pool.append(replacement)

        state["current_pool"] = pool

        # ── Step 3: Build player pack and pick ──
        if len(pool) < PACK_SIZE:
            break

        pack = build_pack(pool, rng)
        result.packs_shown.append(pack)

        chosen = strategy_fn(pack, pick_num, state, rng, rates)
        if chosen:
            result.player_picks.append(chosen)
            if chosen in pool:
                pool.remove(chosen)
            # NO replacement for player picks

        state["current_pool"] = pool
        result.pool_sizes.append(len(pool))
        result.picks_completed = pick_num

        # ── Track archetype counts per pick ──
        arch_counts = defaultdict(int)
        for c in pool:
            arch_counts[c.archetype] += 1
        result.archetype_pool_counts[pick_num] = dict(arch_counts)

        # ── S/A density for committed archetype ──
        committed = state.get("arch", None)
        if committed:
            sa_in_pool = sum(1 for c in pool if is_sa(c, committed, rates))
            total_in_pool = len(pool)
            density = sa_in_pool / max(1, total_in_pool)
            result.sa_density_trajectory.append({
                "pick": pick_num,
                "sa_count": sa_in_pool,
                "pool_size": total_in_pool,
                "density": density,
            })

        # ── Rolling 5-pick trend window ──
        if committed:
            pack_sa = sum(1 for c in pack if is_sa(c, committed, rates))
        else:
            pack_sa = 0
        recent_sa_counts.append(pack_sa)
        if len(recent_sa_counts) >= 5:
            window = recent_sa_counts[-5:]
            result.trend_windows.append({
                "pick": pick_num,
                "window_avg": sum(window) / len(window),
            })

    # Final snapshot
    if pool:
        snap = {}
        for arch_name in ARCHETYPE_NAMES:
            arch_cards = [c for c in pool if c.archetype == arch_name]
            sa_count = sum(1 for c in arch_cards if is_sa(c, arch_name, rates))
            snap[arch_name] = {"total": len(arch_cards), "sa": sa_count}
        result.pool_snapshots["pick_30_post"] = snap

    result.committed_archetype = state.get("arch", None)
    if result.committed_archetype is None and strategy_name == "power_chaser":
        # Power chasers don't commit; use most-drafted archetype
        arch_counts = defaultdict(int)
        for c in result.player_picks:
            arch_counts[c.archetype] += 1
        if arch_counts:
            result.committed_archetype = max(arch_counts, key=arch_counts.get)

    # Compute S/A per pack for committed archetype
    if result.committed_archetype:
        for pack in result.packs_shown:
            sa = sum(1 for c in pack if is_sa(c, result.committed_archetype, rates))
            result.sa_per_pack.append(sa)

    return result


# ============================================================
# Metrics Computation
# ============================================================
def compute_metrics(results, rates=None):
    """Compute all M1-M12 metrics from draft results."""
    if rates is None:
        rates = SIBLING_RATES

    m1_vals, m2_vals, m3_vals, m4_vals = [], [], [], []
    m5_vals, m6_vals, m9_vals, m10_vals, m11_vals = [], [], [], [], []
    per_arch_m3 = defaultdict(list)
    all_p6_sa = []
    consec_bad_list = []
    pool_snapshots_agg = defaultdict(lambda: defaultdict(list))

    for result in results:
        arch = result.committed_archetype
        if not arch:
            continue

        npacks = len(result.packs_shown)

        # M1: Picks 1-5, unique archetypes with S/A cards per pack
        for i in range(min(5, npacks)):
            pack = result.packs_shown[i]
            archs_with_sa = set()
            for c in pack:
                for a_name in ARCHETYPE_NAMES:
                    if is_sa(c, a_name, rates):
                        archs_with_sa.add(a_name)
            m1_vals.append(len(archs_with_sa))

        # M2: Picks 1-5, S/A for committed archetype per pack
        for i in range(min(5, npacks)):
            sa = sum(1 for c in result.packs_shown[i] if is_sa(c, arch, rates))
            m2_vals.append(sa)

        # M3: Picks 6+, S/A for committed archetype per pack
        p6_sa = []
        for i in range(5, npacks):
            sa = sum(1 for c in result.packs_shown[i] if is_sa(c, arch, rates))
            p6_sa.append(sa)
            m3_vals.append(sa)
            all_p6_sa.append(sa)
        if p6_sa:
            per_arch_m3[arch].append(sum(p6_sa) / len(p6_sa))

        # M4: Picks 6+, off-archetype (C/F) cards per pack
        for i in range(5, npacks):
            off = sum(1 for c in result.packs_shown[i]
                      if compute_tier(c, arch, rates) in ("C", "F"))
            m4_vals.append(off)

        # M5: Convergence pick (first 3-pick rolling avg >= 2.0 S/A)
        convergence = 30
        rolling = []
        for i in range(npacks):
            sa = sum(1 for c in result.packs_shown[i] if is_sa(c, arch, rates))
            rolling.append(sa)
            if len(rolling) >= 3 and sum(rolling[-3:]) / 3 >= 2.0:
                convergence = i + 1
                break
        m5_vals.append(convergence)

        # M6: Deck archetype concentration
        if result.player_picks:
            sa_deck = sum(1 for c in result.player_picks if is_sa(c, arch, rates))
            m6_vals.append(sa_deck / len(result.player_picks))

        # M9: StdDev of S/A per pack (picks 6+)
        if len(p6_sa) >= 2:
            mean = sum(p6_sa) / len(p6_sa)
            var = sum((x - mean) ** 2 for x in p6_sa) / len(p6_sa)
            m9_vals.append(math.sqrt(var))

        # M10: Max consecutive packs below 1.5 S/A (picks 6+)
        max_bad = 0
        cur_bad = 0
        for sa in p6_sa:
            if sa < 1.5:
                cur_bad += 1
                max_bad = max(max_bad, cur_bad)
            else:
                cur_bad = 0
        m10_vals.append(max_bad)
        consec_bad_list.append(max_bad)

        # M11': Picks 20+, S/A per pack (relaxed from V9's picks 15+)
        p20_sa = []
        for i in range(19, npacks):
            sa = sum(1 for c in result.packs_shown[i] if is_sa(c, arch, rates))
            p20_sa.append(sa)
        if p20_sa:
            m11_vals.append(sum(p20_sa) / len(p20_sa))

        # Aggregate pool snapshots
        for snap_key, snap_data in result.pool_snapshots.items():
            for arch_name, counts in snap_data.items():
                pool_snapshots_agg[snap_key][arch_name].append(counts)

    def avg(lst):
        return sum(lst) / len(lst) if lst else 0.0

    # M7: Run-to-run card overlap
    m7_overlaps = []
    for i in range(1, len(results)):
        a_ids = set(c.id for c in results[i - 1].player_picks)
        b_ids = set(c.id for c in results[i].player_picks)
        if a_ids and b_ids:
            m7_overlaps.append(len(a_ids & b_ids) / max(len(a_ids), len(b_ids)))

    # M8: Archetype frequency
    arch_freq = defaultdict(int)
    total = 0
    for r in results:
        if r.committed_archetype:
            arch_freq[r.committed_archetype] += 1
            total += 1
    arch_freqs = {}
    for a in ARCHETYPE_NAMES:
        arch_freqs[a] = arch_freq.get(a, 0) / max(1, total) * 100

    # Pack quality percentiles (picks 6+)
    pq = {}
    if all_p6_sa:
        sorted_sa = sorted(all_p6_sa)
        n = len(sorted_sa)
        for pct in [10, 25, 50, 75, 90]:
            idx = min(int(n * pct / 100), n - 1)
            pq[f"p{pct}"] = sorted_sa[idx]

    # Consecutive bad pack distribution
    bad_dist = defaultdict(int)
    for v in consec_bad_list:
        bad_dist[v] += 1

    metrics = {
        "M1": avg(m1_vals),
        "M2": avg(m2_vals),
        "M3": avg(m3_vals),
        "M4": avg(m4_vals),
        "M5": avg(m5_vals),
        "M6": avg(m6_vals) * 100,
        "M7": avg(m7_overlaps) * 100,
        "M9": avg(m9_vals),
        "M10": avg(m10_vals),
        "M10_p95": sorted(m10_vals)[int(len(m10_vals) * 0.95)] if m10_vals else 0,
        "M10_max": max(m10_vals) if m10_vals else 0,
        "M11": avg(m11_vals),
        "M8_max": max(arch_freqs.values()) if arch_freqs else 0,
        "M8_min": min(arch_freqs.values()) if arch_freqs else 0,
    }

    return metrics, per_arch_m3, pq, pool_snapshots_agg, bad_dist, arch_freqs


# ============================================================
# M12 Computation
# ============================================================
def compute_m12(signal_m3, committed_m3):
    """M12 = Signal-reader M3 minus Committed M3."""
    return signal_m3 - committed_m3


# ============================================================
# Draft Trace Formatter
# ============================================================
def format_trace(result, rates):
    """Format a human-readable draft trace."""
    arch = result.committed_archetype or "unknown"
    lines = [
        f"Strategy target: {arch}",
        f"AI lanes: {result.ai_archetypes}",
        f"Open lanes: {result.open_lanes}",
        f"",
    ]

    for i, (pack, pick_card) in enumerate(zip(result.packs_shown, result.player_picks)):
        pick_num = i + 1
        pool_sz = result.pool_sizes[i] if i < len(result.pool_sizes) else "?"
        sa = result.sa_per_pack[i] if i < len(result.sa_per_pack) else "?"

        chosen_tier = compute_tier(pick_card, arch, rates)
        chosen_arch = pick_card.archetype
        sym = "/".join(pick_card.visible_symbols) if pick_card.visible_symbols else "none"

        prefix = ""
        if pick_num == 10:
            prefix = "[SNAPSHOT] "
        elif pick_num == 16:
            prefix = "[REFILL+SNAPSHOT] "
        elif pick_num == 20:
            prefix = "[SNAPSHOT] "

        lines.append(
            f"  {prefix}Pick {pick_num:2d} | Pool:{pool_sz:>3} | "
            f"Pack S/A={sa} | Chose: {chosen_arch}({chosen_tier}) "
            f"sym={sym} pow={pick_card.power:.1f}"
        )

    if result.player_picks:
        sa_total = sum(1 for c in result.player_picks if is_sa(c, arch, rates))
        lines.append(f"\n  Final deck: {sa_total}/{len(result.player_picks)} S/A "
                     f"= {sa_total / len(result.player_picks) * 100:.0f}%")

    return "\n".join(lines)


# ============================================================
# Main Simulation
# ============================================================
def main():
    random.seed(42)

    print("=" * 78)
    print("SIM-5: ASYMMETRIC REPLACEMENT (Design 6, Timing-Adjusted)")
    print("V11 Round 4 — Continuous Market, AI Replacement, Player Depletion")
    print("Pool: 120 starting + 240 reserve | Pack: 5 | Refill at pick 16")
    print("1000 drafts x 3 strategies, Graduated Realistic fitness")
    print("=" * 78)

    # ==========================================================
    # PRIMARY: Graduated Realistic, all strategies
    # ==========================================================
    all_strategy_data = {}

    for strategy in ["committed", "signal_reader", "power_chaser"]:
        print(f"\n{'=' * 60}")
        print(f"  Strategy: {strategy}")
        print(f"{'=' * 60}")

        rng = random.Random(42)
        results = []
        for _ in range(NUM_DRAFTS):
            r = run_single_draft(rng, strategy, SIBLING_RATES)
            results.append(r)

        metrics, pa_m3, pq, pool_snaps, bad_dist, arch_freqs = compute_metrics(
            results, SIBLING_RATES)

        all_strategy_data[strategy] = {
            "metrics": metrics,
            "pa_m3": pa_m3,
            "pq": pq,
            "pool_snaps": pool_snaps,
            "bad_dist": bad_dist,
            "arch_freqs": arch_freqs,
            "results": results,
        }

        m = metrics
        print(f"    M1  = {m['M1']:.2f}   (>= 3)")
        print(f"    M2  = {m['M2']:.2f}   (<= 2)")
        print(f"    M3  = {m['M3']:.2f}   (>= 2.0)")
        print(f"    M4  = {m['M4']:.2f}   (>= 0.5)")
        print(f"    M5  = {m['M5']:.1f}   (5-8)")
        print(f"    M6  = {m['M6']:.1f}%  (60-90%)")
        print(f"    M7  = {m['M7']:.1f}%  (< 40%)")
        print(f"    M8  = {m['M8_min']:.1f}-{m['M8_max']:.1f}%  (5-20%)")
        print(f"    M9  = {m['M9']:.2f}   (>= 0.8)")
        print(f"    M10 = {m['M10']:.2f} avg / p95={m['M10_p95']} / max={m['M10_max']}  (<= 2)")
        print(f"    M11'= {m['M11']:.2f}   (>= 2.5)")

        if pq:
            print(f"    PackQ: p10={pq.get('p10','-')} p25={pq.get('p25','-')} "
                  f"p50={pq.get('p50','-')} p75={pq.get('p75','-')} p90={pq.get('p90','-')}")

        print(f"\n    Per-Archetype M3:")
        for a in ARCHETYPE_NAMES:
            vals = pa_m3.get(a, [])
            m3_a = sum(vals) / len(vals) if vals else 0.0
            print(f"      {a:15s} {m3_a:.2f}")

    # ==========================================================
    # M12: Signal-reader advantage
    # ==========================================================
    m3_signal = all_strategy_data["signal_reader"]["metrics"]["M3"]
    m3_committed = all_strategy_data["committed"]["metrics"]["M3"]
    m12 = compute_m12(m3_signal, m3_committed)

    print(f"\n{'=' * 60}")
    print(f"  M12 (Signal-reader advantage)")
    print(f"{'=' * 60}")
    print(f"    Signal-reader M3: {m3_signal:.2f}")
    print(f"    Committed M3:     {m3_committed:.2f}")
    print(f"    M12 = {m12:.2f}   (>= 0.3)")

    # ==========================================================
    # PESSIMISTIC: Reduced sibling rates
    # ==========================================================
    print(f"\n{'=' * 60}")
    print(f"  PESSIMISTIC FITNESS (reduced sibling rates)")
    print(f"{'=' * 60}")

    for strategy in ["committed", "signal_reader"]:
        rng = random.Random(42)
        results = []
        for _ in range(NUM_DRAFTS):
            r = run_single_draft(rng, strategy, PESSIMISTIC_RATES)
            results.append(r)
        m_pess, _, _, _, _, _ = compute_metrics(results, PESSIMISTIC_RATES)
        print(f"    {strategy}: M3={m_pess['M3']:.2f}  M10={m_pess['M10']:.2f}  M11'={m_pess['M11']:.2f}")

    # ==========================================================
    # Pool Composition Snapshots
    # ==========================================================
    print(f"\n{'=' * 60}")
    print(f"  POOL COMPOSITION SNAPSHOTS (committed strategy, averaged)")
    print(f"{'=' * 60}")

    committed_data = all_strategy_data["committed"]
    pool_snaps = committed_data["pool_snaps"]

    for snap_key in ["pick_1_pre", "pick_10_pre", "pick_16_pre", "pick_16_post",
                     "pick_20_pre", "pick_30_post"]:
        if snap_key not in pool_snaps:
            continue
        print(f"\n    {snap_key}:")
        print(f"    {'Archetype':15s} {'Avg Total':>10} {'Avg S/A':>10}")
        snap_data = pool_snaps[snap_key]
        for arch_name in ARCHETYPE_NAMES:
            if arch_name in snap_data:
                entries = snap_data[arch_name]
                avg_total = sum(e["total"] for e in entries) / len(entries)
                avg_sa = sum(e["sa"] for e in entries) / len(entries)
                print(f"    {arch_name:15s} {avg_total:10.1f} {avg_sa:10.1f}")

    # ==========================================================
    # Pack Quality Distribution (committed)
    # ==========================================================
    print(f"\n{'=' * 60}")
    print(f"  PACK QUALITY DISTRIBUTION (picks 6+, committed)")
    print(f"{'=' * 60}")
    pq = committed_data["pq"]
    if pq:
        print(f"    p10={pq.get('p10','-')}  p25={pq.get('p25','-')}  "
              f"p50={pq.get('p50','-')}  p75={pq.get('p75','-')}  p90={pq.get('p90','-')}")

    # ==========================================================
    # Consecutive Bad Pack Analysis
    # ==========================================================
    print(f"\n{'=' * 60}")
    print(f"  CONSECUTIVE BAD PACK ANALYSIS (committed)")
    print(f"{'=' * 60}")
    bad_dist = committed_data["bad_dist"]
    total_drafts = sum(bad_dist.values())
    print(f"    {'Max Streak':>12} {'Count':>8} {'Pct':>8}")
    for streak in sorted(bad_dist.keys()):
        count = bad_dist[streak]
        pct = count / total_drafts * 100
        print(f"    {streak:>12} {count:>8} {pct:>7.1f}%")

    # ==========================================================
    # S/A Density Trajectory (committed, sample drafts)
    # ==========================================================
    print(f"\n{'=' * 60}")
    print(f"  S/A DENSITY TRAJECTORY (committed, first 5 drafts averaged)")
    print(f"{'=' * 60}")

    # Aggregate trajectory across all committed drafts
    committed_results = committed_data["results"]
    traj_by_pick = defaultdict(list)
    for r in committed_results:
        for entry in r.sa_density_trajectory:
            traj_by_pick[entry["pick"]].append(entry)

    print(f"    {'Pick':>4} {'Avg SA in Pool':>15} {'Avg Pool Size':>15} {'Avg Density':>12}")
    for pick_num in sorted(traj_by_pick.keys()):
        entries = traj_by_pick[pick_num]
        avg_sa = sum(e["sa_count"] for e in entries) / len(entries)
        avg_pool = sum(e["pool_size"] for e in entries) / len(entries)
        avg_dens = sum(e["density"] for e in entries) / len(entries)
        if pick_num <= 30:
            print(f"    {pick_num:>4} {avg_sa:>15.1f} {avg_pool:>15.1f} {avg_dens:>12.4f}")

    # ==========================================================
    # Draft Traces
    # ==========================================================
    print(f"\n{'=' * 60}")
    print(f"  DRAFT TRACES")
    print(f"{'=' * 60}")

    # Trace 1: committed player
    print(f"\n  --- Trace 1: Committed player ---")
    rng_trace = random.Random(777)
    trace1 = run_single_draft(rng_trace, "committed", SIBLING_RATES)
    print(format_trace(trace1, SIBLING_RATES))

    # Trace 2: signal reader
    print(f"\n  --- Trace 2: Signal reader ---")
    rng_trace = random.Random(888)
    trace2 = run_single_draft(rng_trace, "signal_reader", SIBLING_RATES)
    print(format_trace(trace2, SIBLING_RATES))

    # ==========================================================
    # Scorecard Summary
    # ==========================================================
    print(f"\n{'=' * 78}")
    print(f"  FULL SCORECARD SUMMARY")
    print(f"{'=' * 78}")

    mc = all_strategy_data["committed"]["metrics"]
    ms = all_strategy_data["signal_reader"]["metrics"]
    mp = all_strategy_data["power_chaser"]["metrics"]

    print(f"\n  {'Metric':<8} {'Committed':>10} {'Signal':>10} {'Power':>10} {'Target':>15}")
    print(f"  {'-' * 58}")
    rows = [
        ("M1", mc["M1"], ms["M1"], mp["M1"], ">= 3"),
        ("M2", mc["M2"], ms["M2"], mp["M2"], "<= 2"),
        ("M3", mc["M3"], ms["M3"], mp["M3"], ">= 2.0"),
        ("M4", mc["M4"], ms["M4"], mp["M4"], ">= 0.5"),
        ("M5", mc["M5"], ms["M5"], mp["M5"], "5-8"),
        ("M6", mc["M6"], ms["M6"], mp["M6"], "60-90%"),
        ("M7", mc["M7"], ms["M7"], mp["M7"], "< 40%"),
        ("M9", mc["M9"], ms["M9"], mp["M9"], ">= 0.8"),
        ("M10", mc["M10"], ms["M10"], mp["M10"], "<= 2"),
        ("M11'", mc["M11"], ms["M11"], mp["M11"], ">= 2.5"),
    ]
    for name, vc, vs, vp, tgt in rows:
        print(f"  {name:<8} {vc:>10.2f} {vs:>10.2f} {vp:>10.2f} {tgt:>15}")

    print(f"  {'M12':<8} {'':>10} {m12:>10.2f} {'':>10} {'>= 0.3':>15}")

    # ==========================================================
    # Self-Assessment
    # ==========================================================
    print(f"\n{'=' * 78}")
    print(f"  SELF-ASSESSMENT")
    print(f"{'=' * 78}")

    checks = [
        ("M1 >= 3", mc["M1"] >= 3.0),
        ("M2 <= 2", mc["M2"] <= 2.0),
        ("M3 >= 2.0 (committed)", mc["M3"] >= 2.0),
        ("M3 >= 2.0 (signal)", ms["M3"] >= 2.0),
        ("M4 >= 0.5", mc["M4"] >= 0.5),
        ("M5 in 5-8", 5 <= mc["M5"] <= 8),
        ("M6 in 60-90%", 60 <= mc["M6"] <= 90),
        ("M7 < 40%", mc["M7"] < 40),
        ("M8 no arch > 20%", mc["M8_max"] <= 20),
        ("M8 no arch < 5%", mc["M8_min"] >= 5),
        ("M9 >= 0.8", mc["M9"] >= 0.8),
        ("M10 <= 2 (avg)", mc["M10"] <= 2.0),
        ("M11' >= 2.5 (committed)", mc["M11"] >= 2.5),
        ("M11' >= 2.5 (signal)", ms["M11"] >= 2.5),
        ("M12 >= 0.3", m12 >= 0.3),
    ]

    passed = 0
    for name, result in checks:
        mark = "PASS" if result else "FAIL"
        if result:
            passed += 1
        print(f"    {mark:4s}  {name}")

    print(f"\n    Score: {passed}/{len(checks)} checks passed")
    print(f"\n{'=' * 78}")
    print("SIMULATION COMPLETE")
    print(f"{'=' * 78}")


if __name__ == "__main__":
    main()
