"""
Simulation Agent 6: SIM-6 -- Small Pool Concentrated Bias (Design 2 Modified)

5 rounds x 6 picks, 66-card pool, declining refills (70%/55%/40%/none),
70% open-lane refill bias. Bars only. Level 1 pool-reactive.
Competitive check against 3-round designs.

Algorithm spec from critic_review.md Section 6, SIM-6:
  - 5 rounds x 6 picks = 30 total picks
  - 66-card starting pool (8 archetypes x ~8.25)
  - 5 AIs (Level 0), 3 open lanes
  - 6 drafters total: 36 cards removed per round
  - Declining refills after rounds 1-3:
      Round 1: +25 cards (70% of 36)
      Round 2: +20 cards (55% of 36)
      Round 3: +14 cards (40% of 36)
      Round 4: no refill (round 5 drafts from residual)
  - Open-lane bias: 70% of refill to 3 cold (open) lanes, 30% to 5 hot (AI) lanes
  - Cold/hot detection: depletion rate per archetype since last refill
  - AI saturation at 8 on-archetype picks
  - Graduated Realistic fitness model (primary), Pessimistic (secondary)
  - 1000 drafts x 3 strategies
"""

import random
import math
from dataclasses import dataclass, field
from collections import defaultdict
from typing import Optional, List, Dict, Tuple

# ============================================================
# Constants
# ============================================================
NUM_DRAFTS = 1000
NUM_PICKS = 30
PACK_SIZE = 5
POOL_START = 66
NUM_ROUNDS = 5
PICKS_PER_ROUND = 6
NUM_AIS = 5
NUM_OPEN_LANES = 3
RESONANCE_TYPES = ["Ember", "Stone", "Tide", "Zephyr"]

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
ARCH_BY_NAME = {a[0]: a for a in ARCHETYPES}

# Graduated Realistic fitness per co-primary pair
FITNESS_GRADUATED = {
    ("Warriors", "Sacrifice"): 0.50,
    ("Sacrifice", "Warriors"): 0.50,
    ("Self-Discard", "Self-Mill"): 0.40,
    ("Self-Mill", "Self-Discard"): 0.40,
    ("Blink", "Storm"): 0.30,
    ("Storm", "Blink"): 0.30,
    ("Flash", "Ramp"): 0.25,
    ("Ramp", "Flash"): 0.25,
}

FITNESS_PESSIMISTIC = {
    ("Warriors", "Sacrifice"): 0.35,
    ("Sacrifice", "Warriors"): 0.35,
    ("Self-Discard", "Self-Mill"): 0.25,
    ("Self-Mill", "Self-Discard"): 0.25,
    ("Blink", "Storm"): 0.15,
    ("Storm", "Blink"): 0.15,
    ("Flash", "Ramp"): 0.10,
    ("Ramp", "Flash"): 0.10,
}

FITNESS_MODELS = {
    "Graduated": FITNESS_GRADUATED,
    "Pessimistic": FITNESS_PESSIMISTIC,
}

# Refill schedule: cards added after each round (index = round_just_completed - 1)
# After round 1: +25, after round 2: +20, after round 3: +14, after round 4: 0
REFILL_SCHEDULE = [25, 20, 14, 0]

# Open-lane bias: 70% of refill goes to cold (open) lanes
OPEN_LANE_BIAS = 0.70


def get_sibling(arch_name: str) -> Optional[str]:
    """Return co-primary sibling archetype name (shares primary resonance)."""
    r1 = ARCH_BY_NAME[arch_name][1]
    for a in ARCHETYPES:
        if a[0] != arch_name and a[1] == r1:
            return a[0]
    return None


# ============================================================
# Card Model
# ============================================================
@dataclass
class SimCard:
    id: int
    visible_symbols: List[str]
    archetype: str
    power: float
    is_generic: bool = False
    tier: str = "C"  # S, A, C, F


# ============================================================
# S/A Tier Assignment
# ============================================================
def assign_tier(card: SimCard, player_archetype: str, fitness_model: dict) -> str:
    """Determine if a card is S/A tier for a given player archetype."""
    if card.is_generic:
        return "F"
    if card.archetype == player_archetype:
        return "S"
    sibling = get_sibling(player_archetype)
    if card.archetype == sibling:
        rate = fitness_model.get((player_archetype, sibling), 0.0)
        return "A" if random.random() < rate else "C"
    return "F"


# ============================================================
# Pool Construction
# ============================================================
_card_id_counter = 0


def _next_id():
    global _card_id_counter
    _card_id_counter += 1
    return _card_id_counter


def make_cards_for_archetype(arch_name: str, count: int) -> List[SimCard]:
    """Create cards for a specific archetype with proper symbol distribution."""
    r1 = ARCH_BY_NAME[arch_name][1]
    r2 = ARCH_BY_NAME[arch_name][2]
    cards = []

    # ~11% generic, ~79% single, ~10% dual per archetype batch
    n_dual = max(1, round(count * 0.10))
    n_generic = max(1, round(count * 0.11))
    n_single = count - n_dual - n_generic

    for _ in range(n_single):
        cards.append(SimCard(
            id=_next_id(),
            visible_symbols=[r1],
            archetype=arch_name,
            power=random.uniform(3, 9),
        ))

    for _ in range(n_dual):
        cards.append(SimCard(
            id=_next_id(),
            visible_symbols=[r1, r2],
            archetype=arch_name,
            power=random.uniform(4, 9),
        ))

    for _ in range(n_generic):
        cards.append(SimCard(
            id=_next_id(),
            visible_symbols=[],
            archetype=arch_name,
            power=random.uniform(3, 7),
            is_generic=True,
        ))

    return cards


def build_starting_pool() -> List[SimCard]:
    """Build the 66-card starting pool (8 archetypes x ~8.25)."""
    global _card_id_counter
    _card_id_counter = 0

    cards = []
    # Distribute 66 cards: 8 archetypes get 8 each = 64, then 2 extra
    base_per = 8
    extras = POOL_START - (base_per * 8)  # 2 extra cards

    for i, (arch_name, r1, r2) in enumerate(ARCHETYPES):
        count = base_per + (1 if i < extras else 0)
        cards.extend(make_cards_for_archetype(arch_name, count))

    random.shuffle(cards)
    return cards


def build_refill_cards(num_cards: int, hot_archetypes: List[str],
                       cold_archetypes: List[str]) -> List[SimCard]:
    """
    Build refill cards with open-lane (cold) bias.
    70% of cards go to cold (open) archetypes, 30% to hot (AI) archetypes.
    """
    n_cold = round(num_cards * OPEN_LANE_BIAS)
    n_hot = num_cards - n_cold

    cards = []

    # Distribute cold cards evenly among cold archetypes
    if cold_archetypes:
        per_cold = n_cold // len(cold_archetypes)
        remainder = n_cold - per_cold * len(cold_archetypes)
        for i, arch in enumerate(cold_archetypes):
            count = per_cold + (1 if i < remainder else 0)
            cards.extend(make_cards_for_archetype(arch, count))

    # Distribute hot cards evenly among hot archetypes
    if hot_archetypes:
        per_hot = n_hot // len(hot_archetypes)
        remainder = n_hot - per_hot * len(hot_archetypes)
        for i, arch in enumerate(hot_archetypes):
            count = per_hot + (1 if i < remainder else 0)
            cards.extend(make_cards_for_archetype(arch, count))

    random.shuffle(cards)
    return cards


# ============================================================
# AI Drafter
# ============================================================
@dataclass
class AIDrafter:
    archetype: str
    on_archetype_picks: int = 0
    saturation_threshold: int = 8
    total_picks: int = 0

    def pick_card(self, pool: List[SimCard]) -> Optional[SimCard]:
        """AI picks the best card from the pool for its archetype."""
        if not pool:
            return None

        r1 = ARCH_BY_NAME[self.archetype][1]
        r2 = ARCH_BY_NAME[self.archetype][2]
        saturated = self.on_archetype_picks >= self.saturation_threshold

        def ai_score(c: SimCard) -> float:
            if saturated:
                # After saturation: pick generics and adjacents by power
                if c.is_generic:
                    return c.power + 5
                if c.archetype == self.archetype:
                    return c.power * 0.3  # avoid own archetype
                # Adjacent = shares a resonance symbol
                for sym in c.visible_symbols:
                    if sym == r1 or sym == r2:
                        return c.power + 2
                return c.power
            else:
                # Pre-saturation: strongly prefer own archetype
                if c.archetype == self.archetype:
                    return c.power + 20
                # Adjacent archetype (shares primary resonance)
                for sym in c.visible_symbols:
                    if sym == r1:
                        return c.power + 5
                    elif sym == r2:
                        return c.power + 2
                if c.is_generic:
                    return c.power + 1
                return c.power * 0.5

        best = max(pool, key=ai_score)

        if best.archetype == self.archetype:
            self.on_archetype_picks += 1
        self.total_picks += 1

        return best


# ============================================================
# Depletion Tracking (Level 1 Pool-Reactive)
# ============================================================
def detect_hot_cold(depletion_counts: Dict[str, int]) -> Tuple[List[str], List[str]]:
    """
    Classify archetypes as hot (AI lanes, fast-depleting) or cold (open, slow-depleting).
    Returns (hot_archetypes, cold_archetypes) where 5 are hot and 3 are cold.
    """
    sorted_archs = sorted(ARCHETYPE_NAMES,
                          key=lambda a: depletion_counts.get(a, 0))
    # 3 slowest-depleting = cold (open lanes)
    cold = sorted_archs[:NUM_OPEN_LANES]
    # 5 fastest-depleting = hot (AI lanes)
    hot = sorted_archs[NUM_OPEN_LANES:]
    return hot, cold


# ============================================================
# Pack Construction
# ============================================================
def draw_pack(pool: List[SimCard], pack_size: int = PACK_SIZE) -> List[SimCard]:
    """
    Draw a pack of cards from the pool, weighted by archetype presence.
    Simulates seeing a representative sample of what's available.
    """
    if len(pool) <= pack_size:
        return list(pool)
    return random.sample(pool, pack_size)


# ============================================================
# Player Pick Logic
# ============================================================
def player_pick(pack: List[SimCard], player_archetype: str,
                strategy: str, pick_num: int, sa_cache: Dict[int, bool],
                signature: Dict[str, float]) -> SimCard:
    """Select a card from the pack based on strategy."""
    arch = ARCH_BY_NAME[player_archetype]
    r1, r2 = arch[1], arch[2]

    if strategy == "committed":
        def score(c):
            s = 0
            if sa_cache.get(c.id, False):
                s += 10
            for i, sym in enumerate(c.visible_symbols):
                if sym == r1:
                    s += 3 if i == 0 else 1
                elif sym == r2:
                    s += 2 if i == 0 else 1
            s += c.power * 0.1
            return s
        return max(pack, key=score)

    elif strategy == "power":
        return max(pack, key=lambda c: c.power)

    elif strategy == "signal":
        if pick_num <= 5:
            # Before commitment: pick best power with slight resonance preference
            def score(c):
                s = c.power
                for sym in c.visible_symbols:
                    if sym == r1:
                        s += 1
                return s
            return max(pack, key=score)
        # After pick 5: commit to strongest observed resonance
        top_res = max(RESONANCE_TYPES, key=lambda r: signature.get(r, 0))
        def score(c):
            s = 0
            for i, sym in enumerate(c.visible_symbols):
                if sym == top_res:
                    s += 3 if i == 0 else 1
            s += c.power * 0.1
            if sa_cache.get(c.id, False):
                s += 5
            return s
        return max(pack, key=score)

    return random.choice(pack)


# ============================================================
# Core Draft Simulation
# ============================================================
def run_one_draft(player_archetype: str, ai_archetypes: List[str],
                  fitness_model: dict, strategy: str):
    """
    Run a single 5-round draft with declining refills and open-lane bias.

    Returns (history, drafted_cards, sa_cache, pool_snapshots, sa_density_trace).
    """
    pool = build_starting_pool()

    # Pre-compute S/A status for all cards that could appear
    sa_cache = {}
    for c in pool:
        sa_cache[c.id] = (assign_tier(c, player_archetype, fitness_model) in ("S", "A"))

    # Initialize AIs
    ais = [AIDrafter(archetype=arch) for arch in ai_archetypes]

    drafted = []
    history = []
    signature = {r: 0.0 for r in RESONANCE_TYPES}

    # Tracking for depletion detection
    depletion_counts = {a: 0 for a in ARCHETYPE_NAMES}
    pool_snapshots = []  # (round, pool_size, per_arch_counts, sa_density)
    sa_density_trace = []  # per-pick SA density in pool

    pick_num = 0

    for rnd in range(1, NUM_ROUNDS + 1):
        # Round-start snapshot
        arch_counts = defaultdict(int)
        for c in pool:
            arch_counts[c.archetype] += 1
        sa_in_pool = sum(1 for c in pool if sa_cache.get(c.id, False))
        sa_dens = sa_in_pool / max(1, len(pool))
        pool_snapshots.append({
            "round": rnd,
            "pool_size": len(pool),
            "arch_counts": dict(arch_counts),
            "sa_density": sa_dens,
            "sa_count": sa_in_pool,
        })

        # Reset depletion tracking for this round
        round_depletion = {a: 0 for a in ARCHETYPE_NAMES}

        for pick_in_round in range(1, PICKS_PER_ROUND + 1):
            pick_num += 1

            # Track SA density before this pick
            sa_in_pool = sum(1 for c in pool if sa_cache.get(c.id, False))
            sa_density_trace.append({
                "pick": pick_num,
                "round": rnd,
                "pool_size": len(pool),
                "sa_in_pool": sa_in_pool,
                "sa_density": sa_in_pool / max(1, len(pool)),
            })

            # AI picks first (each AI picks 1 card from pool)
            for ai in ais:
                if pool:
                    ai_pick = ai.pick_card(pool)
                    if ai_pick:
                        pool = [c for c in pool if c.id != ai_pick.id]
                        round_depletion[ai_pick.archetype] += 1
                        depletion_counts[ai_pick.archetype] += 1

            # Player gets a pack from remaining pool
            pack = draw_pack(pool)

            if not pack:
                break

            # Player picks
            chosen = player_pick(pack, player_archetype, strategy,
                                 pick_num, sa_cache, signature)
            drafted.append(chosen)
            pool = [c for c in pool if c.id != chosen.id]
            round_depletion[chosen.archetype] += 1
            depletion_counts[chosen.archetype] += 1

            # Update signature
            for i, sym in enumerate(chosen.visible_symbols):
                if i == 0:
                    signature[sym] += 2.0
                else:
                    signature[sym] += 1.0

            # Record history
            sa_count = sum(1 for c in pack if sa_cache.get(c.id, False))
            history.append({
                "pick": pick_num,
                "round": rnd,
                "pack": pack,
                "chosen": chosen,
                "pool_size": len(pool),
                "sa_count": sa_count,
            })

        # End of round: apply refill
        if rnd <= len(REFILL_SCHEDULE) and REFILL_SCHEDULE[rnd - 1] > 0:
            num_refill = REFILL_SCHEDULE[rnd - 1]

            # Detect hot/cold using accumulated depletion
            hot, cold = detect_hot_cold(depletion_counts)

            refill_cards = build_refill_cards(num_refill, hot, cold)

            # Assign S/A status for new cards
            for c in refill_cards:
                sa_cache[c.id] = (assign_tier(c, player_archetype, fitness_model)
                                  in ("S", "A"))

            pool.extend(refill_cards)
            random.shuffle(pool)

    return history, drafted, sa_cache, pool_snapshots, sa_density_trace


# ============================================================
# Metrics Computation
# ============================================================
def compute_metrics(all_results):
    """Compute all 12 metrics from draft results."""
    m1_vals, m2_vals, m3_vals, m4_vals = [], [], [], []
    m5_vals, m6_vals, m9_vals, m10_vals = [], [], [], []
    m11_vals = []
    post_commit_sa = []
    late_draft_sa = []
    consec_bad_list = []

    for (history, drafted, sa_cache, _, _) in all_results:
        # M1: picks 1-5, unique archetypes per pack
        early_arch = []
        for h in history[:5]:
            archs = set()
            for c in h["pack"]:
                if not c.is_generic:
                    archs.add(c.archetype)
            early_arch.append(len(archs))
        m1_vals.append(sum(early_arch) / max(1, len(early_arch)))

        # M2: picks 1-5, S/A per pack
        early_sa = []
        for h in history[:5]:
            sa = sum(1 for c in h["pack"] if sa_cache.get(c.id, False))
            early_sa.append(sa)
        m2_vals.append(sum(early_sa) / max(1, len(early_sa)))

        # M3: picks 6+, S/A per pack
        post_sa = []
        for h in history[5:]:
            sa = h["sa_count"]
            post_sa.append(sa)
            post_commit_sa.append(sa)
        if post_sa:
            m3_vals.append(sum(post_sa) / len(post_sa))

        # M4: picks 6+, off-archetype cards per pack
        post_off = []
        for h in history[5:]:
            off = sum(1 for c in h["pack"] if not sa_cache.get(c.id, False))
            post_off.append(off)
        if post_off:
            m4_vals.append(sum(post_off) / len(post_off))

        # M5: convergence pick (first pick where rolling 3-pack avg >= 1.5 SA)
        conv_pick = NUM_PICKS
        all_sa = [h["sa_count"] for h in history]
        for i in range(2, len(all_sa)):
            window = all_sa[i-2:i+1]
            if sum(window) / 3 >= 1.5:
                conv_pick = i + 1
                break
        m5_vals.append(conv_pick)

        # M6: deck concentration (fraction S/A of all drafted)
        sa_drafted = sum(1 for c in drafted if sa_cache.get(c.id, False))
        m6_vals.append(sa_drafted / max(1, len(drafted)))

        # M9: stddev of S/A per pack (picks 6+)
        if len(post_sa) > 1:
            mean_sa = sum(post_sa) / len(post_sa)
            var = sum((x - mean_sa) ** 2 for x in post_sa) / len(post_sa)
            m9_vals.append(math.sqrt(var))

        # M10: max consecutive packs with S/A < 1.5 (picks 6+)
        max_c = 0
        cur_c = 0
        for sa in post_sa:
            if sa < 1.5:
                cur_c += 1
                max_c = max(max_c, cur_c)
            else:
                cur_c = 0
        m10_vals.append(max_c)
        consec_bad_list.append(max_c)

        # M11': picks 20+, S/A per pack
        late_sa_picks = []
        for h in history[19:]:  # picks 20+
            sa = h["sa_count"]
            late_sa_picks.append(sa)
            late_draft_sa.append(sa)
        if late_sa_picks:
            m11_vals.append(sum(late_sa_picks) / len(late_sa_picks))

    # M7: run-to-run card overlap
    m7_overlaps = []
    for i in range(1, len(all_results)):
        ids_prev = set(c.id for c in all_results[i-1][1])
        ids_curr = set(c.id for c in all_results[i][1])
        overlap = len(ids_prev & ids_curr) / max(1, len(ids_prev | ids_curr))
        m7_overlaps.append(overlap)

    # M8: archetype frequency
    arch_counts = defaultdict(int)
    total = len(all_results)
    for (history, drafted, sa_cache, _, _) in all_results:
        if drafted:
            counts = defaultdict(int)
            for c in drafted:
                if not c.is_generic:
                    counts[c.archetype] += 1
            if counts:
                top_arch = max(counts, key=counts.get)
                arch_counts[top_arch] += 1
    m8_max = max(arch_counts.values()) / max(1, total) if arch_counts else 0
    m8_min = min(arch_counts.values()) / max(1, total) if arch_counts else 0

    # Pack quality percentiles (picks 6+)
    pq_sorted = sorted(post_commit_sa)
    n = len(pq_sorted)
    pcts = {}
    for p in [10, 25, 50, 75, 90]:
        idx = min(int(n * p / 100), n - 1) if n > 0 else 0
        pcts[p] = pq_sorted[idx] if n > 0 else 0

    # Late draft percentiles (picks 20+)
    late_sorted = sorted(late_draft_sa)
    nl = len(late_sorted)
    late_pcts = {}
    for p in [10, 25, 50, 75, 90]:
        idx = min(int(nl * p / 100), nl - 1) if nl > 0 else 0
        late_pcts[p] = late_sorted[idx] if nl > 0 else 0

    avg = lambda vs: sum(vs) / max(1, len(vs))

    return {
        "M1": avg(m1_vals),
        "M2": avg(m2_vals),
        "M3": avg(m3_vals),
        "M4": avg(m4_vals),
        "M5": avg(m5_vals),
        "M6": avg(m6_vals),
        "M7": avg(m7_overlaps),
        "M8_max": m8_max,
        "M8_min": m8_min,
        "M9": avg(m9_vals),
        "M10": avg(m10_vals),
        "M10_worst": max(m10_vals) if m10_vals else 0,
        "M11": avg(m11_vals),
        "pack_pcts": pcts,
        "late_pcts": late_pcts,
        "avg_consec_bad": avg(consec_bad_list),
        "worst_consec_bad": max(consec_bad_list) if consec_bad_list else 0,
    }


# ============================================================
# M12: Signal-reader advantage
# ============================================================
def compute_m12(signal_m3, committed_m3):
    """M12 = signal-reader M3 minus committed M3."""
    return signal_m3 - committed_m3


# ============================================================
# Runners
# ============================================================
def assign_ai_archetypes(player_archetype: str) -> List[str]:
    """Assign 5 AI archetypes (all except player's and 2 other open lanes)."""
    # Player's archetype is one open lane
    # Find the 2 archetypes most distant from the player's on the circle
    # For simplicity: pick 5 archetypes that are NOT the player's archetype
    # and not the player's sibling (to keep sibling as an open lane option)
    all_archs = list(ARCHETYPE_NAMES)
    sibling = get_sibling(player_archetype)

    # Open lanes: player's archetype, sibling, and one other
    # We pick the "other" open lane to be the archetype 4 steps away (opposite side)
    player_idx = ARCHETYPE_NAMES.index(player_archetype)
    opposite_idx = (player_idx + 4) % 8
    other_open = ARCHETYPE_NAMES[opposite_idx]

    open_lanes = {player_archetype, sibling, other_open}
    ai_archs = [a for a in all_archs if a not in open_lanes]

    # If we have more than 5 (shouldn't happen with 8 - 3 = 5), trim
    return ai_archs[:NUM_AIS]


def run_aggregate(fitness_name: str, strategy: str, n_drafts: int = NUM_DRAFTS):
    """Run aggregate drafts cycling through player archetypes."""
    fitness_model = FITNESS_MODELS[fitness_name]
    all_results = []

    for i in range(n_drafts):
        arch_name = ARCHETYPE_NAMES[i % 8]
        ai_archs = assign_ai_archetypes(arch_name)
        result = run_one_draft(arch_name, ai_archs, fitness_model, strategy)
        all_results.append(result)

    return compute_metrics(all_results), all_results


def run_per_archetype(fitness_name: str, strategy: str, n_per: int = 125):
    """Run per-archetype analysis."""
    fitness_model = FITNESS_MODELS[fitness_name]
    results = {}

    for arch_name in ARCHETYPE_NAMES:
        histories = []
        ai_archs = assign_ai_archetypes(arch_name)
        for _ in range(n_per):
            result = run_one_draft(arch_name, ai_archs, fitness_model, strategy)
            histories.append(result)
        results[arch_name] = compute_metrics(histories)

    return results


# ============================================================
# Draft Trace Formatting
# ============================================================
def format_trace(history, drafted, sa_cache, player_archetype, pool_snapshots):
    """Format a detailed draft trace with round boundaries."""
    lines = [f"=== Draft Trace: {player_archetype} ==="]

    current_round = 0
    for h in history:
        pick = h["pick"]
        rnd = h["round"]
        sa = h["sa_count"]
        pool_sz = h["pool_size"]
        chosen = h["chosen"]
        chosen_sa = "S/A" if sa_cache.get(chosen.id, False) else "C/F"
        sym_str = "/".join(chosen.visible_symbols) if chosen.visible_symbols else "Generic"

        if rnd != current_round:
            current_round = rnd
            # Find snapshot for this round
            snap = None
            for s in pool_snapshots:
                if s["round"] == rnd:
                    snap = s
                    break
            if snap:
                lines.append(
                    f"\n  --- Round {rnd} start: pool={snap['pool_size']}, "
                    f"SA density={snap['sa_density']:.1%}, "
                    f"SA count={snap['sa_count']} ---"
                )

        lines.append(
            f"  Pick {pick:2d} (R{rnd}): pool={pool_sz:3d}, pack S/A={sa}, "
            f"chose [{chosen.archetype}:{sym_str}] ({chosen_sa})"
        )

    sa_d = sum(1 for c in drafted if sa_cache.get(c.id, False))
    lines.append(f"\n  Final: {sa_d}/{len(drafted)} S/A = "
                 f"{sa_d / max(1, len(drafted)) * 100:.0f}%")
    return "\n".join(lines)


# ============================================================
# Pool Composition Analysis
# ============================================================
def analyze_pool_composition(all_results):
    """Analyze round-by-round pool composition across all drafts."""
    round_data = defaultdict(lambda: {
        "pool_sizes": [],
        "sa_densities": [],
        "sa_counts": [],
        "arch_counts": defaultdict(list),
    })

    for (history, drafted, sa_cache, pool_snapshots, sa_density_trace) in all_results:
        for snap in pool_snapshots:
            rnd = snap["round"]
            round_data[rnd]["pool_sizes"].append(snap["pool_size"])
            round_data[rnd]["sa_densities"].append(snap["sa_density"])
            round_data[rnd]["sa_counts"].append(snap["sa_count"])
            for arch, count in snap["arch_counts"].items():
                round_data[rnd]["arch_counts"][arch].append(count)

    return round_data


def analyze_sa_density_trajectory(all_results):
    """Analyze SA density trajectory across all picks."""
    pick_data = defaultdict(lambda: {"densities": [], "pool_sizes": [], "sa_counts": []})

    for (history, drafted, sa_cache, pool_snapshots, sa_density_trace) in all_results:
        for pt in sa_density_trace:
            pick = pt["pick"]
            pick_data[pick]["densities"].append(pt["sa_density"])
            pick_data[pick]["pool_sizes"].append(pt["pool_size"])
            pick_data[pick]["sa_counts"].append(pt["sa_in_pool"])

    return pick_data


# ============================================================
# Main
# ============================================================
def main():
    random.seed(42)

    print("=" * 80)
    print("SIM-6: Small Pool Concentrated Bias (Design 2 Modified)")
    print("=" * 80)
    print("5 rounds x 6 picks, 66-card pool, declining refills, 70% open-lane bias")
    print("Level 1 pool-reactive, bars only")
    print()

    # =====================================================
    # Primary: Graduated Realistic, all 3 strategies
    # =====================================================
    print("=" * 80)
    print("PRIMARY RUN: Graduated Realistic Fitness")
    print("=" * 80)

    grad_results = {}
    for strategy in ["committed", "power", "signal"]:
        print(f"  Running strategy: {strategy} ({NUM_DRAFTS} drafts)...")
        m, results = run_aggregate("Graduated", strategy, n_drafts=NUM_DRAFTS)
        grad_results[strategy] = (m, results)

    print()
    print(f"{'Strategy':<12} {'M1':>5} {'M2':>5} {'M3':>5} {'M4':>5} "
          f"{'M5':>5} {'M6':>5} {'M7':>6} {'M9':>5} {'M10':>5} {'M11':>6}")
    for strategy in ["committed", "power", "signal"]:
        m = grad_results[strategy][0]
        print(f"{strategy:<12} {m['M1']:5.2f} {m['M2']:5.2f} {m['M3']:5.2f} "
              f"{m['M4']:5.2f} {m['M5']:5.1f} {m['M6']:5.2f} {m['M7']:6.3f} "
              f"{m['M9']:5.2f} {m['M10']:5.1f} {m['M11']:6.2f}")

    # M8
    m_committed = grad_results["committed"][0]
    print(f"\nM8 (archetype frequency): max={m_committed['M8_max']:.1%}, "
          f"min={m_committed['M8_min']:.1%}")

    # M12
    m12 = compute_m12(grad_results["signal"][0]["M3"],
                      grad_results["committed"][0]["M3"])
    print(f"M12 (signal - committed M3): {m12:.3f}")

    # =====================================================
    # Secondary: Pessimistic fitness
    # =====================================================
    print()
    print("=" * 80)
    print("SECONDARY RUN: Pessimistic Fitness (committed strategy)")
    print("=" * 80)
    m_pess, _ = run_aggregate("Pessimistic", "committed", n_drafts=NUM_DRAFTS)
    print(f"  M3={m_pess['M3']:.2f} M10={m_pess['M10']:.1f} "
          f"M11'={m_pess['M11']:.2f} M6={m_pess['M6']:.2f}")

    # =====================================================
    # Per-archetype M3 table
    # =====================================================
    print()
    print("=" * 80)
    print("PER-ARCHETYPE M3 (Graduated Realistic, committed)")
    print("=" * 80)
    pa = run_per_archetype("Graduated", "committed", n_per=125)
    print(f"  {'Archetype':<16} {'M3':>6} {'M5':>6} {'M6':>6} {'M9':>6} "
          f"{'M10':>5} {'M11':>6}")
    m3_per_arch = []
    for arch in ARCHETYPE_NAMES:
        m = pa[arch]
        m3_per_arch.append(m["M3"])
        print(f"  {arch:<16} {m['M3']:6.2f} {m['M5']:6.1f} {m['M6']:6.2f} "
              f"{m['M9']:6.2f} {m['M10']:5.1f} {m['M11']:6.2f}")
    m3_spread = max(m3_per_arch) - min(m3_per_arch)
    print(f"  Spread (max-min): {m3_spread:.2f}")
    print(f"  Worst: {ARCHETYPE_NAMES[m3_per_arch.index(min(m3_per_arch))]} = "
          f"{min(m3_per_arch):.2f}")
    print(f"  Best:  {ARCHETYPE_NAMES[m3_per_arch.index(max(m3_per_arch))]} = "
          f"{max(m3_per_arch):.2f}")

    # =====================================================
    # Pack quality distribution
    # =====================================================
    print()
    print("=" * 80)
    print("PACK QUALITY DISTRIBUTION (picks 6+, committed)")
    print("=" * 80)
    m_c = grad_results["committed"][0]
    pq = m_c["pack_pcts"]
    lp = m_c["late_pcts"]
    print(f"  Picks 6+  P10={pq[10]}  P25={pq[25]}  P50={pq[50]}  "
          f"P75={pq[75]}  P90={pq[90]}")
    print(f"  Picks 20+ P10={lp[10]}  P25={lp[25]}  P50={lp[50]}  "
          f"P75={lp[75]}  P90={lp[90]}")

    # =====================================================
    # Consecutive bad pack analysis
    # =====================================================
    print()
    print("=" * 80)
    print("CONSECUTIVE BAD PACK ANALYSIS (picks 6+, committed, Graduated)")
    print("=" * 80)
    _, results_c = grad_results["committed"]
    consec_dist = defaultdict(int)
    for (h, d, cache, _, _) in results_c:
        post_sa = [hh["sa_count"] for hh in h[5:]]
        max_c, cur_c = 0, 0
        for sa in post_sa:
            if sa < 1.5:
                cur_c += 1
                max_c = max(max_c, cur_c)
            else:
                cur_c = 0
        consec_dist[max_c] += 1
    print(f"  {'Streak':<8} {'Count':>8} {'Pct':>8}")
    for k in sorted(consec_dist.keys()):
        pct = consec_dist[k] / NUM_DRAFTS * 100
        print(f"  {k:<8} {consec_dist[k]:>8} {pct:>7.1f}%")

    # =====================================================
    # Round-by-round pool composition
    # =====================================================
    print()
    print("=" * 80)
    print("ROUND-BY-ROUND POOL COMPOSITION (committed, Graduated)")
    print("=" * 80)
    round_data = analyze_pool_composition(results_c)
    print(f"  {'Round':<8} {'Avg Pool':>10} {'Avg SA':>8} {'SA Density':>12}")
    for rnd in sorted(round_data.keys()):
        rd = round_data[rnd]
        avg_pool = sum(rd["pool_sizes"]) / len(rd["pool_sizes"])
        avg_sa = sum(rd["sa_counts"]) / len(rd["sa_counts"])
        avg_dens = sum(rd["sa_densities"]) / len(rd["sa_densities"])
        print(f"  {rnd:<8} {avg_pool:10.1f} {avg_sa:8.1f} {avg_dens:11.1%}")

    # =====================================================
    # SA density trajectory
    # =====================================================
    print()
    print("=" * 80)
    print("S/A DENSITY TRAJECTORY (committed, Graduated)")
    print("=" * 80)
    sa_traj = analyze_sa_density_trajectory(results_c)
    print(f"  {'Pick':<6} {'Avg Pool':>10} {'Avg SA':>8} {'SA Density':>12}")
    for pick in sorted(sa_traj.keys()):
        pt = sa_traj[pick]
        avg_pool = sum(pt["pool_sizes"]) / len(pt["pool_sizes"])
        avg_sa = sum(pt["sa_counts"]) / len(pt["sa_counts"])
        avg_dens = sum(pt["densities"]) / len(pt["densities"])
        if pick % 3 == 1 or pick <= 6 or pick >= 25:
            print(f"  {pick:<6} {avg_pool:10.1f} {avg_sa:8.1f} {avg_dens:11.1%}")

    # =====================================================
    # Draft Traces
    # =====================================================
    print()
    print("=" * 80)
    print("DRAFT TRACES")
    print("=" * 80)

    # Trace 1: Committed player, Warriors
    random.seed(100)
    ai_archs_1 = assign_ai_archetypes("Warriors")
    h1, d1, c1, ps1, _ = run_one_draft(
        "Warriors", ai_archs_1, FITNESS_GRADUATED, "committed")
    print("\n" + format_trace(h1, d1, c1, "Warriors (committed)", ps1))

    # Trace 2: Signal reader, Sacrifice
    random.seed(200)
    ai_archs_2 = assign_ai_archetypes("Sacrifice")
    h2, d2, c2, ps2, _ = run_one_draft(
        "Sacrifice", ai_archs_2, FITNESS_GRADUATED, "signal")
    print("\n" + format_trace(h2, d2, c2, "Sacrifice (signal reader)", ps2))

    # =====================================================
    # Thin rounds 4-5 analysis
    # =====================================================
    print()
    print("=" * 80)
    print("THIN ROUNDS 4-5 ANALYSIS (committed, Graduated)")
    print("=" * 80)
    round_45_sa = []
    round_45_pool = []
    round_45_bad_streak = []
    for (h, d, cache, _, _) in results_c:
        r45_packs = [hh for hh in h if hh["round"] >= 4]
        for hh in r45_packs:
            round_45_sa.append(hh["sa_count"])
            round_45_pool.append(hh["pool_size"])
        # Consecutive bad in rounds 4-5 only
        r45_sa = [hh["sa_count"] for hh in r45_packs]
        max_c, cur_c = 0, 0
        for sa in r45_sa:
            if sa < 1.5:
                cur_c += 1
                max_c = max(max_c, cur_c)
            else:
                cur_c = 0
        round_45_bad_streak.append(max_c)

    if round_45_sa:
        avg_sa_45 = sum(round_45_sa) / len(round_45_sa)
        avg_pool_45 = sum(round_45_pool) / len(round_45_pool)
        avg_bad_45 = sum(round_45_bad_streak) / len(round_45_bad_streak)
        worst_bad_45 = max(round_45_bad_streak)
        zero_sa_pct = sum(1 for s in round_45_sa if s == 0) / len(round_45_sa) * 100
        print(f"  Avg pool size (rounds 4-5): {avg_pool_45:.1f}")
        print(f"  Avg S/A per pack: {avg_sa_45:.2f}")
        print(f"  Zero S/A packs: {zero_sa_pct:.1f}%")
        print(f"  Avg consecutive bad streak: {avg_bad_45:.2f}")
        print(f"  Worst consecutive bad streak: {worst_bad_45}")

        # Percentiles for rounds 4-5
        sorted_45 = sorted(round_45_sa)
        n45 = len(sorted_45)
        print(f"  S/A distribution: P10={sorted_45[int(n45*0.1)]} "
              f"P25={sorted_45[int(n45*0.25)]} P50={sorted_45[int(n45*0.5)]} "
              f"P75={sorted_45[int(n45*0.75)]} P90={sorted_45[int(n45*0.9)]}")

    # =====================================================
    # Summary Scorecard
    # =====================================================
    print()
    print("=" * 80)
    print("SUMMARY SCORECARD: SIM-6 (Graduated Realistic, committed)")
    print("=" * 80)
    m = grad_results["committed"][0]
    targets = {
        "M1 >= 3.0": (m["M1"], m["M1"] >= 3.0),
        "M2 <= 2.0": (m["M2"], m["M2"] <= 2.0),
        "M3 >= 2.0": (m["M3"], m["M3"] >= 2.0),
        "M4 >= 0.5": (m["M4"], m["M4"] >= 0.5),
        "M5 pick 5-8": (m["M5"], 5.0 <= m["M5"] <= 8.0),
        "M6 60-90%": (m["M6"], 0.60 <= m["M6"] <= 0.90),
        "M7 < 40%": (m["M7"], m["M7"] < 0.40),
        "M8 max < 20%": (m["M8_max"], m["M8_max"] < 0.20),
        "M8 min > 5%": (m["M8_min"], m["M8_min"] > 0.05),
        "M9 >= 0.8": (m["M9"], m["M9"] >= 0.8),
        "M10 <= 2": (m["M10"], m["M10"] <= 2.0),
        "M11' >= 2.5": (m["M11"], m["M11"] >= 2.5),
        "M12 >= 0.3": (m12, m12 >= 0.3),
    }
    passes = 0
    for label, (val, passed) in targets.items():
        status = "PASS" if passed else "FAIL"
        passes += int(passed)
        print(f"  {status} {label:<18}: {val:.2f}")
    print(f"\n  Overall: {passes}/{len(targets)} metrics pass")

    # =====================================================
    # Comparison Table
    # =====================================================
    print()
    print("=" * 80)
    print("COMPARISON: SIM-6 vs V9 vs V10")
    print("=" * 80)
    print(f"  {'Algorithm':<35} {'M3':>6} {'M10':>5} {'M11':>6} {'M6':>5}")
    print(f"  {'V9 Hybrid B':<35} {'2.70':>6} {'3.8':>5} {'3.25':>6} {'0.86':>5}")
    print(f"  {'V10 Hybrid X':<35} {'0.84':>6} {'--':>5} {'0.69':>6} {'--':>5}")
    print(f"  {'SIM-6 (this)':<35} {m['M3']:6.2f} "
          f"{m['M10']:5.1f} {m['M11']:6.2f} {m['M6']:5.2f}")

    print("\nDone.")


if __name__ == "__main__":
    main()
