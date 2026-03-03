"""
Simulation Agent 2: SIM-2 -- Static Open-Lane Bias
===================================================

Algorithm per Design 4 Proposal B + Design 5 information layer:

STRUCTURE:
  - 3 rounds x 10 picks = 30 total picks
  - 120-card starting pool, 8 archetypes x 15 cards each
  - 60-card refills after rounds 1 and 2

REFILL BIAS (Level 0):
  - Open-lane archetypes (3): ~10.7 cards per refill (1.7x base)
  - AI-lane archetypes (5): ~4.3 cards per refill (~0.58x base)
  - Bias fixed at draft init from static AI assignments

AI DRAFTERS:
  - 5 AIs, each assigned one unique archetype
  - Level 0: pick highest fitness for assigned archetype
  - Saturation at 10 on-archetype cards (then generics/adjacents)
  - 10% chance per pick of taking adjacent archetype card

PACK CONSTRUCTION:
  - Player sees 5 cards drawn from pool, weighted by archetype presence
  - Player picks 1

CARD MODEL:
  - Each card has a primary archetype (one of 8) and per-archetype fitness
  - S/A tier for a given player archetype is pre-rolled at draft start:
    * Card is its own archetype: always S/A
    * Card is sibling archetype: S/A with graduated realistic probability
    * Card is any other archetype or generic: never S/A
  - ~36% weighted average S/A rate across sibling pairs
  - Visible symbols: ~11% generic, ~79% single, ~10% dual

PLAYER STRATEGIES:
  - Committed: picks random open-lane archetype at pick 1
  - Signal-reader: commits at pick 5 to open archetype with most pool cards
  - Power-chaser: picks highest raw power regardless of archetype
"""

import random
import math
from dataclasses import dataclass, field
from collections import Counter, defaultdict

# ============================================================
# Constants
# ============================================================
NUM_DRAFTS = 1000
NUM_PICKS = 30
PACK_SIZE = 5
POOL_SIZE = 120
CARDS_PER_ARCHETYPE = 15
NUM_ROUNDS = 3
PICKS_PER_ROUND = 10
REFILL_SIZE = 60
OPEN_LANE_MULTIPLIER = 1.7
NUM_AIS = 5
AI_SATURATION_THRESHOLD = 10
AI_ADJACENT_PROB = 0.10
NUM_ARCHETYPES = 8

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
ARCH_INDEX = {a[0]: i for i, a in enumerate(ARCHETYPES)}

# Sibling pairs share primary resonance
SIBLING_PAIRS = {}
for i, a in enumerate(ARCHETYPES):
    for j, b in enumerate(ARCHETYPES):
        if i != j and a[1] == b[1]:
            SIBLING_PAIRS[a[0]] = b[0]

# Adjacent archetypes on circle
ADJACENT_ARCHS = {}
for i, a in enumerate(ARCHETYPES):
    adj = set()
    adj.add(ARCHETYPES[(i - 1) % 8][0])
    adj.add(ARCHETYPES[(i + 1) % 8][0])
    if a[0] in SIBLING_PAIRS:
        adj.add(SIBLING_PAIRS[a[0]])
    adj.discard(a[0])
    ADJACENT_ARCHS[a[0]] = list(adj)

# Graduated Realistic sibling A-tier rates
SIBLING_RATES = {
    ("Warriors",     "Sacrifice"):    0.50,
    ("Sacrifice",    "Warriors"):     0.50,
    ("Self-Discard", "Self-Mill"):    0.40,
    ("Self-Mill",    "Self-Discard"): 0.40,
    ("Blink",        "Storm"):        0.30,
    ("Storm",        "Blink"):        0.30,
    ("Flash",        "Ramp"):         0.25,
    ("Ramp",         "Flash"):        0.25,
}


# ============================================================
# Card Model
# ============================================================
_next_card_id = 0


@dataclass
class SimCard:
    id: int
    visible_symbols: list      # 0-2 resonance symbols
    archetype: str             # primary archetype
    power: float               # raw power 0-10
    is_generic: bool = False

    def __hash__(self):
        return self.id

    def __eq__(self, other):
        if not isinstance(other, SimCard):
            return False
        return self.id == other.id


def make_card(archetype_name):
    """Generate a single card for the given archetype."""
    global _next_card_id
    cid = _next_card_id
    _next_card_id += 1

    arch = ARCH_BY_NAME[archetype_name]
    r1, r2 = arch[1], arch[2]

    # Visible symbol distribution: 11% generic, 79% single, 10% dual
    roll = random.random()
    if roll < 0.11:
        visible_symbols = []
        is_generic = True
    elif roll < 0.90:
        visible_symbols = [r1]
        is_generic = False
    else:
        visible_symbols = [r1, r2]
        is_generic = False

    power = random.uniform(2, 10)

    return SimCard(
        id=cid,
        visible_symbols=visible_symbols,
        archetype=archetype_name,
        power=power,
        is_generic=is_generic,
    )


def precompute_sa(pool, player_archetype, fitness_rates):
    """
    Pre-roll S/A tier for every card relative to a specific player archetype.

    Rules:
    - Card's primary archetype == player archetype: always S/A
    - Card's primary archetype == sibling of player archetype: S/A with
      graduated realistic probability
    - Generic or any other archetype: never S/A
    """
    sa_map = {}
    sibling = SIBLING_PAIRS.get(player_archetype)

    for c in pool:
        if c.is_generic:
            sa_map[c.id] = False
        elif c.archetype == player_archetype:
            sa_map[c.id] = True
        elif sibling and c.archetype == sibling:
            rate = fitness_rates.get((player_archetype, sibling), 0.0)
            sa_map[c.id] = random.random() < rate
        else:
            sa_map[c.id] = False
    return sa_map


def generate_pool(n_per_archetype):
    """Generate a pool of n_per_archetype cards per archetype."""
    cards = []
    for arch_name in ARCHETYPE_NAMES:
        for _ in range(n_per_archetype):
            cards.append(make_card(arch_name))
    return cards


def generate_biased_refill(refill_total, open_archetypes, ai_archetypes):
    """Generate a refill batch with open-lane bias."""
    base = refill_total / NUM_ARCHETYPES  # 7.5
    open_per = base * OPEN_LANE_MULTIPLIER  # ~12.75
    open_total_alloc = open_per * len(open_archetypes)
    ai_total_alloc = refill_total - open_total_alloc
    ai_per = ai_total_alloc / len(ai_archetypes) if ai_archetypes else 0

    cards = []
    counts = {}
    for arch_name in ARCHETYPE_NAMES:
        if arch_name in open_archetypes:
            n = int(round(open_per))
        else:
            n = int(round(ai_per))
        counts[arch_name] = n
        for _ in range(n):
            cards.append(make_card(arch_name))

    # Adjust to exact refill_total
    while len(cards) < refill_total:
        arch = random.choice(list(open_archetypes))
        cards.append(make_card(arch))
    while len(cards) > refill_total:
        cards.pop()

    return cards


# ============================================================
# AI Drafter
# ============================================================
@dataclass
class AIDrafter:
    archetype: str
    on_archetype_count: int = 0

    def pick_from_pool(self, pool):
        """AI picks a card from the pool. Returns the chosen card or None."""
        if not pool:
            return None

        saturated = self.on_archetype_count >= AI_SATURATION_THRESHOLD
        take_adjacent = random.random() < AI_ADJACENT_PROB

        if saturated or take_adjacent:
            adj = ADJACENT_ARCHS.get(self.archetype, [])
            candidates = [c for c in pool
                          if c.is_generic or c.archetype in adj]
            if not candidates:
                candidates = pool
        else:
            candidates = [c for c in pool if c.archetype == self.archetype]
            if not candidates:
                adj = ADJACENT_ARCHS.get(self.archetype, [])
                candidates = [c for c in pool
                              if c.is_generic or c.archetype in adj]
            if not candidates:
                candidates = pool

        # AI scores: prefer own archetype cards, then power
        def ai_score(c):
            if c.archetype == self.archetype:
                return 100 + c.power
            elif c.archetype in ADJACENT_ARCHS.get(self.archetype, []):
                return 50 + c.power
            else:
                return c.power
        chosen = max(candidates, key=ai_score)

        if chosen.archetype == self.archetype:
            self.on_archetype_count += 1

        return chosen


# ============================================================
# Pack Construction
# ============================================================
def construct_pack(pool, pack_size=PACK_SIZE):
    """
    Construct a pack from the pool.
    Weighted by archetype presence (more cards = more likely to appear).
    """
    if len(pool) <= pack_size:
        return list(pool)

    # Simple weighted sampling: each card equally likely (uniform from pool)
    # This naturally weights toward archetypes with more cards in pool
    return random.sample(pool, pack_size)


# ============================================================
# Player Strategies
# ============================================================
def committed_pick(pack, player_archetype, sa_map):
    """Pick highest-scoring card for committed archetype."""
    arch = ARCH_BY_NAME[player_archetype]
    r1, r2 = arch[1], arch[2]

    def score(c):
        s = 10.0 if sa_map.get(c.id, False) else 0.0
        for sym in c.visible_symbols:
            if sym == r1:
                s += 3.0
            elif sym == r2:
                s += 1.5
        s += c.power * 0.05
        return s

    return max(pack, key=score)


def power_pick(pack):
    """Pick highest raw power."""
    return max(pack, key=lambda c: c.power)


def signal_reader_pick(pack, committed_arch, sa_map):
    """Before commitment: pick highest power. After: pick like committed."""
    if committed_arch is None:
        return power_pick(pack)
    return committed_pick(pack, committed_arch, sa_map)


# ============================================================
# Draft Engine
# ============================================================
def run_draft(fitness_rates, strategy, specific_archetype=None):
    """
    Run a single draft.

    Returns dict with all results needed for metrics.
    """
    # Generate initial pool
    pool = generate_pool(CARDS_PER_ARCHETYPE)

    # Assign AI archetypes (5 random unique archetypes)
    ai_arch_names = random.sample(ARCHETYPE_NAMES, NUM_AIS)
    open_arch_names = set(ARCHETYPE_NAMES) - set(ai_arch_names)
    ai_arch_set = set(ai_arch_names)

    # Create AI drafters
    ais = [AIDrafter(archetype=a) for a in ai_arch_names]

    # Determine player archetype
    if strategy == "committed":
        if specific_archetype:
            player_archetype = specific_archetype
        else:
            # Committed player picks a random open-lane archetype
            player_archetype = random.choice(list(open_arch_names))
    elif strategy == "signal":
        player_archetype = None  # Determined at pick 5
    elif strategy == "power":
        if specific_archetype:
            player_archetype = specific_archetype
        else:
            # Power chaser evaluates relative to a random archetype
            player_archetype = random.choice(list(open_arch_names))
    else:
        player_archetype = random.choice(list(open_arch_names))

    # Pre-compute S/A for the entire draft pool plus future refills
    # We need to do this per-card as they enter the pool
    # Start with initial pool
    if player_archetype:
        sa_map = precompute_sa(pool, player_archetype, fitness_rates)
    else:
        sa_map = {}

    drafted = []
    history = []
    pool_snapshots = []
    pick_num = 0

    for round_num in range(1, NUM_ROUNDS + 1):
        # Record pool snapshot at round start
        pool_snapshots.append({
            "round": round_num,
            "total": len(pool),
            "per_archetype": dict(Counter(c.archetype for c in pool)),
        })

        for pick_in_round in range(1, PICKS_PER_ROUND + 1):
            pick_num += 1

            # AI picks first
            for ai in ais:
                chosen_ai = ai.pick_from_pool(pool)
                if chosen_ai and chosen_ai in pool:
                    pool.remove(chosen_ai)

            # Signal-reader commitment at pick 5
            if strategy == "signal" and pick_num == 5:
                # Commit to open archetype with most cards in pool
                arch_counts = Counter(c.archetype for c in pool
                                      if not c.is_generic)
                open_counts = {a: arch_counts.get(a, 0)
                               for a in open_arch_names}
                player_archetype = max(open_counts, key=open_counts.get)
                # Now compute SA map for all existing pool cards + drafted
                sa_map = precompute_sa(pool + drafted, player_archetype,
                                       fitness_rates)

            # Construct pack
            pack = construct_pack(pool)
            if not pack:
                break

            # Player picks
            if strategy == "committed":
                chosen = committed_pick(pack, player_archetype, sa_map)
            elif strategy == "signal":
                chosen = signal_reader_pick(pack, player_archetype, sa_map)
            elif strategy == "power":
                chosen = power_pick(pack)
            else:
                chosen = random.choice(pack)

            drafted.append(chosen)
            if chosen in pool:
                pool.remove(chosen)

            # Count S/A in pack for player's archetype
            if player_archetype and sa_map:
                sa_in_pack = sum(1 for c in pack if sa_map.get(c.id, False))
            else:
                sa_in_pack = 0

            history.append({
                "pick": pick_num,
                "round": round_num,
                "pick_in_round": pick_in_round,
                "pack": pack,
                "chosen": chosen,
                "pool_size": len(pool),
                "sa_count": sa_in_pack,
                "player_archetype": player_archetype,
            })

        # Refill between rounds
        if round_num < NUM_ROUNDS:
            refill = generate_biased_refill(
                REFILL_SIZE, open_arch_names, ai_arch_set)
            # Pre-compute SA for refill cards
            if player_archetype:
                refill_sa = precompute_sa(refill, player_archetype,
                                          fitness_rates)
                sa_map.update(refill_sa)
            pool.extend(refill)

    return {
        "history": history,
        "drafted": drafted,
        "player_archetype": player_archetype,
        "pool_snapshots": pool_snapshots,
        "open_archs": open_arch_names,
        "ai_archs": ai_arch_set,
        "sa_map": sa_map,
    }


# ============================================================
# Metrics
# ============================================================
def compute_metrics(all_results):
    """Compute all 12 metrics."""
    m1_v, m2_v, m3_v, m4_v, m5_v = [], [], [], [], []
    m6_v, m9_v, m10_v, m11_v = [], [], [], []
    post_sa_all = []

    for r in all_results:
        history = r["history"]
        drafted = r["drafted"]
        pa = r["player_archetype"]
        sa_map = r["sa_map"]

        if not pa or not history:
            continue

        # M1: Picks 1-5: unique archetypes with S/A cards per pack
        early = history[:5]
        m1_per_pack = []
        for h in early:
            archs_with_sa = set()
            for c in h["pack"]:
                if sa_map.get(c.id, False):
                    archs_with_sa.add(c.archetype)
            m1_per_pack.append(len(archs_with_sa))
        m1_v.append(sum(m1_per_pack) / max(1, len(m1_per_pack)))

        # M2: Picks 1-5: S/A for emerging archetype per pack
        early_sa = [h["sa_count"] for h in early]
        m2_v.append(sum(early_sa) / max(1, len(early_sa)))

        # M3: Picks 6+: S/A for committed archetype per pack
        post = history[5:]
        post_sa = [h["sa_count"] for h in post]
        post_sa_all.extend(post_sa)
        if post_sa:
            m3_v.append(sum(post_sa) / len(post_sa))

        # M4: Picks 6+: off-archetype cards per pack
        post_off = [sum(1 for c in h["pack"]
                        if not sa_map.get(c.id, False))
                    for h in post]
        if post_off:
            m4_v.append(sum(post_off) / len(post_off))

        # M5: Convergence pick
        conv = NUM_PICKS
        for i in range(2, len(history)):
            w = [history[j]["sa_count"] for j in range(i - 2, i + 1)]
            if sum(w) / 3 >= 1.5:
                conv = history[i]["pick"]
                break
        m5_v.append(conv)

        # M6: Deck S/A concentration
        sa_d = sum(1 for c in drafted if sa_map.get(c.id, False))
        m6_v.append(sa_d / max(1, len(drafted)))

        # M9: StdDev of S/A per pack (picks 6+)
        if len(post_sa) > 1:
            mu = sum(post_sa) / len(post_sa)
            var = sum((x - mu) ** 2 for x in post_sa) / len(post_sa)
            m9_v.append(math.sqrt(var))

        # M10: Max consecutive packs below 1.5 S/A (picks 6+)
        max_c, cur_c = 0, 0
        for sa in post_sa:
            if sa < 1.5:
                cur_c += 1
                max_c = max(max_c, cur_c)
            else:
                cur_c = 0
        m10_v.append(max_c)

        # M11': Picks 20+: S/A for committed archetype per pack
        late = [h["sa_count"] for h in history if h["pick"] >= 20]
        if late:
            m11_v.append(sum(late) / len(late))

    # M7: Run-to-run card overlap
    m7_v = []
    for i in range(1, len(all_results)):
        a_ids = {c.id for c in all_results[i - 1]["drafted"]}
        b_ids = {c.id for c in all_results[i]["drafted"]}
        if a_ids or b_ids:
            m7_v.append(len(a_ids & b_ids) / max(1, len(a_ids | b_ids)))

    # Pack quality percentiles
    pq = sorted(post_sa_all)
    n = len(pq)
    pcts = {}
    for p in [10, 25, 50, 75, 90]:
        idx = min(int(n * p / 100), n - 1) if n > 0 else 0
        pcts[p] = pq[idx] if n > 0 else 0

    avg = lambda vs: sum(vs) / max(1, len(vs))
    return {
        "M1": avg(m1_v), "M2": avg(m2_v), "M3": avg(m3_v),
        "M4": avg(m4_v), "M5": avg(m5_v), "M6": avg(m6_v),
        "M7": avg(m7_v), "M9": avg(m9_v), "M10": avg(m10_v),
        "M10_max": max(m10_v) if m10_v else 0,
        "M11": avg(m11_v), "pack_pcts": pcts,
    }


# ============================================================
# Per-Archetype Analysis
# ============================================================
def run_per_archetype(fitness_rates, n_per=125):
    results = {}
    for arch in ARCHETYPE_NAMES:
        all_r = []
        for _ in range(n_per):
            all_r.append(run_draft(fitness_rates, "committed",
                                   specific_archetype=arch))
        results[arch] = compute_metrics(all_r)
    return results


# ============================================================
# Instrumented Pool Tracking
# ============================================================
def run_pool_tracking(fitness_rates, n_drafts=200):
    """Track pool composition at each pick with proper SA measurement."""
    pick_stats = defaultdict(lambda: {
        "open_total": [], "ai_total": [], "pool_total": [],
        "open_sa": [], "ai_sa": [], "total_sa": [],
    })
    refill_stats = defaultdict(lambda: {
        "open_pre": [], "ai_pre": [], "open_post": [], "ai_post": [],
    })

    for _ in range(n_drafts):
        pool = generate_pool(CARDS_PER_ARCHETYPE)
        ai_arch_names = random.sample(ARCHETYPE_NAMES, NUM_AIS)
        open_arch_names = set(ARCHETYPE_NAMES) - set(ai_arch_names)
        ai_arch_set = set(ai_arch_names)
        player_archetype = random.choice(list(open_arch_names))
        sa_map = precompute_sa(pool, player_archetype, fitness_rates)
        ais = [AIDrafter(archetype=a) for a in ai_arch_names]

        pick_num = 0
        for round_num in range(1, NUM_ROUNDS + 1):
            for pick_in_round in range(1, PICKS_PER_ROUND + 1):
                pick_num += 1

                # Record pool state BEFORE picks
                open_cards = [c for c in pool
                              if c.archetype in open_arch_names]
                ai_cards = [c for c in pool if c.archetype in ai_arch_set]

                open_sa = sum(1 for c in open_cards
                              if sa_map.get(c.id, False))
                ai_sa = sum(1 for c in ai_cards
                            if sa_map.get(c.id, False))

                pick_stats[pick_num]["open_total"].append(len(open_cards))
                pick_stats[pick_num]["ai_total"].append(len(ai_cards))
                pick_stats[pick_num]["pool_total"].append(len(pool))
                pick_stats[pick_num]["open_sa"].append(open_sa)
                pick_stats[pick_num]["ai_sa"].append(ai_sa)
                pick_stats[pick_num]["total_sa"].append(open_sa + ai_sa)

                # AI picks
                for ai in ais:
                    chosen_ai = ai.pick_from_pool(pool)
                    if chosen_ai and chosen_ai in pool:
                        pool.remove(chosen_ai)

                # Player picks (simplified committed)
                pack = construct_pack(pool)
                if pack:
                    chosen = committed_pick(pack, player_archetype, sa_map)
                    if chosen in pool:
                        pool.remove(chosen)

            # Refill
            if round_num < NUM_ROUNDS:
                open_pre = len([c for c in pool
                                if c.archetype in open_arch_names])
                ai_pre = len([c for c in pool
                              if c.archetype in ai_arch_set])

                refill = generate_biased_refill(
                    REFILL_SIZE, open_arch_names, ai_arch_set)
                refill_sa = precompute_sa(refill, player_archetype,
                                          fitness_rates)
                sa_map.update(refill_sa)
                pool.extend(refill)

                open_post = len([c for c in pool
                                 if c.archetype in open_arch_names])
                ai_post = len([c for c in pool
                               if c.archetype in ai_arch_set])

                refill_stats[round_num]["open_pre"].append(open_pre)
                refill_stats[round_num]["ai_pre"].append(ai_pre)
                refill_stats[round_num]["open_post"].append(open_post)
                refill_stats[round_num]["ai_post"].append(ai_post)

    # Average
    avg_pick = {}
    for pick in sorted(pick_stats.keys()):
        avg_pick[pick] = {k: sum(v) / len(v) if v else 0
                          for k, v in pick_stats[pick].items()}
    avg_refill = {}
    for rn in sorted(refill_stats.keys()):
        avg_refill[rn] = {k: sum(v) / len(v) if v else 0
                          for k, v in refill_stats[rn].items()}

    return avg_pick, avg_refill


# ============================================================
# Consecutive Bad Pack Analysis
# ============================================================
def consec_bad_packs(all_results):
    counts = Counter()
    total = 0
    for r in all_results:
        if not r["player_archetype"]:
            continue
        total += 1
        post = r["history"][5:]
        max_c, cur_c = 0, 0
        for h in post:
            if h["sa_count"] < 1.5:
                cur_c += 1
                max_c = max(max_c, cur_c)
            else:
                cur_c = 0
        counts[max_c] += 1
    return counts, total


# ============================================================
# S/A Density Trajectory
# ============================================================
def sa_trajectory(all_results, n_sample=500):
    pick_densities = defaultdict(list)
    for r in all_results[:n_sample]:
        if not r["player_archetype"]:
            continue
        sa_map = r["sa_map"]
        for h in r["history"]:
            sa = h["sa_count"]
            ps = len(h["pack"])
            if ps > 0:
                pick_densities[h["pick"]].append(sa / ps)
    traj = {}
    for pick in sorted(pick_densities.keys()):
        vals = pick_densities[pick]
        traj[pick] = sum(vals) / len(vals) if vals else 0
    return traj


# ============================================================
# Draft Trace Formatter
# ============================================================
def format_trace(result, label):
    history = result["history"]
    drafted = result["drafted"]
    pa = result["player_archetype"]
    sa_map = result["sa_map"]
    open_a = result["open_archs"]
    ai_a = result["ai_archs"]

    lines = [f"=== Draft Trace: {pa} ({label}) ==="]
    lines.append(f"Open lanes: {sorted(open_a)}")
    lines.append(f"AI lanes: {sorted(ai_a)}")
    lines.append("")

    for h in history:
        pick = h["pick"]
        rnd = h["round"]
        sa = h["sa_count"]
        psz = h["pool_size"]
        c = h["chosen"]
        sym = "/".join(c.visible_symbols) if c.visible_symbols else "Generic"
        is_sa = "S/A" if sa_map.get(c.id, False) else "C/F"

        if h["pick_in_round"] == 1 and rnd > 1:
            lines.append(f"  --- Round {rnd} (refill applied) ---")

        lines.append(
            f"  Pick {pick:2d} (R{rnd}): pool={psz:3d}, pack_SA={sa}, "
            f"[{c.archetype}:{sym}] {is_sa} pwr={c.power:.1f}"
        )

    sa_d = sum(1 for c in drafted if sa_map.get(c.id, False))
    lines.append(f"\n  Final: {sa_d}/{len(drafted)} S/A = "
                 f"{sa_d / max(1, len(drafted)) * 100:.0f}%")
    return "\n".join(lines)


# ============================================================
# Main
# ============================================================
def main():
    random.seed(42)

    print("=" * 72)
    print("SIM-2: STATIC OPEN-LANE BIAS -- Simulation Agent 2")
    print("3 rounds x 10 picks, 120-card pool, 60-card biased refills")
    print("Open-lane multiplier: 1.7x | AI-lane: ~0.58x | Level 0")
    print("=" * 72)

    # ---- Run primary simulations ----
    print("\n[1/7] Graduated Realistic, committed (1000 drafts)...")
    committed_results = [run_draft(SIBLING_RATES, "committed")
                         for _ in range(NUM_DRAFTS)]
    m_c = compute_metrics(committed_results)

    print("[2/7] Graduated Realistic, signal (1000 drafts)...")
    signal_results = [run_draft(SIBLING_RATES, "signal")
                      for _ in range(NUM_DRAFTS)]
    m_s = compute_metrics(signal_results)

    print("[3/7] Graduated Realistic, power (1000 drafts)...")
    power_results = [run_draft(SIBLING_RATES, "power")
                     for _ in range(NUM_DRAFTS)]
    m_p = compute_metrics(power_results)

    m12 = m_s["M3"] - m_c["M3"]

    print("[4/7] Per-archetype M3 (125 per archetype)...")
    per_arch = run_per_archetype(SIBLING_RATES, n_per=125)

    print("[5/7] Detailed pool tracking (200 drafts)...")
    avg_pick, avg_refill = run_pool_tracking(SIBLING_RATES, n_drafts=200)

    print("[6/7] S/A density trajectory...")
    sa_traj = sa_trajectory(committed_results)

    print("[7/7] Consecutive bad pack analysis...")
    cc, cc_total = consec_bad_packs(committed_results)

    # ---- Draft traces ----
    random.seed(101)
    trace_committed = run_draft(SIBLING_RATES, "committed")

    random.seed(202)
    trace_signal = run_draft(SIBLING_RATES, "signal")

    # ==========================
    # PRINT ALL RESULTS
    # ==========================

    print("\n" + "=" * 72)
    print("FULL SCORECARD -- SIM-2: Static Open-Lane Bias")
    print("=" * 72)
    m = m_c
    rows = [
        ("M1",   m["M1"],   ">= 3.0",  m["M1"] >= 3.0),
        ("M2",   m["M2"],   "<= 2.0",  m["M2"] <= 2.0),
        ("M3",   m["M3"],   ">= 2.0",  m["M3"] >= 2.0),
        ("M4",   m["M4"],   ">= 0.5",  m["M4"] >= 0.5),
        ("M5",   m["M5"],   "5-8",     5 <= m["M5"] <= 8),
        ("M6",   m["M6"],   "60-90%",  0.60 <= m["M6"] <= 0.90),
        ("M7",   m["M7"],   "< 0.40",  m["M7"] < 0.40),
        ("M9",   m["M9"],   ">= 0.8",  m["M9"] >= 0.8),
        ("M10",  m["M10"],  "<= 2",    m["M10"] <= 2.0),
        ("M11'", m["M11"],  ">= 2.5",  m["M11"] >= 2.5),
        ("M12",  m12,       ">= 0.3",  m12 >= 0.3),
    ]
    print(f"{'Metric':<8} {'Value':>8} {'Target':>10} {'Status':>7}")
    print("-" * 38)
    for name, val, tgt, ok in rows:
        status = "PASS" if ok else "FAIL"
        if name in ("M6", "M7"):
            print(f"{name:<8} {val:8.1%} {tgt:>10} {status:>7}")
        else:
            print(f"{name:<8} {val:8.2f} {tgt:>10} {status:>7}")

    # ---- Strategy comparison ----
    print("\n" + "=" * 72)
    print("STRATEGY COMPARISON (Graduated Realistic, 1000 drafts each)")
    print("=" * 72)
    print(f"{'Strategy':<14} {'M3':>6} {'M11':>6} {'M5':>6} "
          f"{'M6':>8} {'M9':>6} {'M10':>6}")
    for lab, mx in [("committed", m_c), ("signal", m_s), ("power", m_p)]:
        print(f"{lab:<14} {mx['M3']:6.2f} {mx['M11']:6.2f} "
              f"{mx['M5']:6.1f} {mx['M6']:8.1%} {mx['M9']:6.2f} "
              f"{mx['M10']:6.2f}")
    print(f"\nM12 (signal M3 - committed M3): {m12:.2f}")

    # ---- Per-archetype ----
    print("\n" + "=" * 72)
    print("PER-ARCHETYPE M3 TABLE (committed, 125 drafts each)")
    print("=" * 72)
    print(f"{'Archetype':<16} {'M3':>6} {'M11':>6} {'M5':>6} "
          f"{'M6':>8} {'M10':>6}")
    for arch in ARCHETYPE_NAMES:
        mx = per_arch[arch]
        print(f"{arch:<16} {mx['M3']:6.2f} {mx['M11']:6.2f} "
              f"{mx['M5']:6.1f} {mx['M6']:8.1%} {mx['M10']:6.2f}")
    worst_m3 = min(per_arch[a]["M3"] for a in ARCHETYPE_NAMES)
    best_m3 = max(per_arch[a]["M3"] for a in ARCHETYPE_NAMES)
    print(f"  Range: worst={worst_m3:.2f}  best={best_m3:.2f}  "
          f"spread={best_m3 - worst_m3:.2f}")

    # ---- Pool composition ----
    print("\n" + "=" * 72)
    print("POOL COMPOSITION AT EACH PICK (avg 200 drafts)")
    print("=" * 72)
    print(f"{'Pick':>4} {'Pool':>6} {'Open':>8} {'AI':>8} "
          f"{'OpenSA':>8} {'AI_SA':>8} {'TotalSA':>8} "
          f"{'SA%open':>8} {'SA%ai':>8}")
    for pick in sorted(avg_pick.keys()):
        s = avg_pick[pick]
        sa_pct_open = s["open_sa"] / max(1, s["open_total"]) * 100
        sa_pct_ai = s["ai_sa"] / max(1, s["ai_total"]) * 100
        print(f"{pick:4d} {s['pool_total']:6.0f} "
              f"{s['open_total']:8.1f} {s['ai_total']:8.1f} "
              f"{s['open_sa']:8.1f} {s['ai_sa']:8.1f} "
              f"{s['total_sa']:8.1f} "
              f"{sa_pct_open:7.1f}% {sa_pct_ai:7.1f}%")

    # Refill boundary
    print("\nRefill boundary summary:")
    for rn in sorted(avg_refill.keys()):
        s = avg_refill[rn]
        print(f"  After Round {rn}: Open {s['open_pre']:.0f} -> "
              f"{s['open_post']:.0f}  |  AI {s['ai_pre']:.0f} -> "
              f"{s['ai_post']:.0f}")

    # ---- Pack quality distribution ----
    print("\n" + "=" * 72)
    print("PACK QUALITY DISTRIBUTION (S/A per pack, picks 6+, committed)")
    print("=" * 72)
    pq = m_c["pack_pcts"]
    print(f"P10={pq[10]}  P25={pq[25]}  P50={pq[50]}  "
          f"P75={pq[75]}  P90={pq[90]}")

    # ---- Consecutive bad packs ----
    print("\n" + "=" * 72)
    print("CONSECUTIVE BAD PACK ANALYSIS (packs < 1.5 S/A, picks 6+)")
    print("=" * 72)
    print(f"{'MaxConsec':>10} {'Drafts':>8} {'%':>8}")
    for k in sorted(cc.keys()):
        pct = cc[k] / cc_total * 100
        print(f"{k:10d} {cc[k]:8d} {pct:7.1f}%")

    # ---- S/A density trajectory ----
    print("\n" + "=" * 72)
    print("S/A DENSITY TRAJECTORY (fraction of pack that is S/A)")
    print("=" * 72)
    print(f"{'Pick':>4} {'SA Frac':>10}")
    for pick in sorted(sa_traj.keys()):
        print(f"{pick:4d} {sa_traj[pick]:10.1%}")

    # ---- Draft traces ----
    print("\n" + "=" * 72)
    print("DRAFT TRACES")
    print("=" * 72)
    print("\n" + format_trace(trace_committed, "committed"))
    print("\n" + format_trace(trace_signal, "signal-reader"))

    # ---- Comparison ----
    print("\n" + "=" * 72)
    print("COMPARISON TO BASELINES")
    print("=" * 72)
    print(f"{'Algorithm':<30} {'M3':>6} {'M10':>6} {'M11':>6} {'M12':>6}")
    print(f"{'V9 Hybrid B':.<30} {'2.70':>6} {'3.80':>6} {'3.25':>6} "
          f"{'N/A':>6}")
    print(f"{'V10 Hybrid X':.<30} {'0.84':>6} {'---':>6} {'0.69':>6} "
          f"{'N/A':>6}")
    print(f"{'SIM-1 expected':.<30} {'~1.35':>6} {'---':>6} {'---':>6} "
          f"{'---':>6}")
    print(f"{'SIM-2 (this)':.<30} {m_c['M3']:6.2f} "
          f"{m_c['M10']:6.2f} {m_c['M11']:6.2f} {m12:6.2f}")

    print("\nDone.")


if __name__ == "__main__":
    main()
