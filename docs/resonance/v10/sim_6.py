"""
Simulation Agent 6: Design 3 — Competitive Pressure with Safety Valve (revised)
V10 Round 4

Algorithm: 7 AI drafters with saturation mechanic, Level 0 reactivity.
- 7 AIs assigned to 7 of 8 archetypes randomly (1 lane open per game)
- Each AI picks cards per round using pair-affinity scores
- Accumulation phase (< threshold archetype cards): 85% archetype, 15% generic
- Saturation phase (>= threshold archetype cards): 50% archetype, 30% adjacent, 20% generic
- Supplemental culling: lowest-power cards removed per round
- Level 0 reactivity throughout (no player awareness)
- Pack: 4 cards drawn randomly from remaining pool after AI picks + culling

NOTE: The design specifies 2 cards/AI/round + 10 cull = 25/round, which exhausts
the 360-card pool by pick 14. This is a structural flaw acknowledged in the
post-critique revision. We simulate both:
  Config A: "As designed" (2/AI + 10 cull = 25/round, pool exhausts ~pick 14)
  Config B: "Calibrated" (1/AI + 4 cull = 12/round, supports full 30 picks)
Config B is the primary reporting configuration.
"""

import random
import math
from dataclasses import dataclass, field
from collections import defaultdict, Counter
from typing import Optional, List, Dict, Tuple, Set

# ============================================================
# Constants
# ============================================================
NUM_DRAFTS = 1000
NUM_PICKS = 30
PACK_SIZE = 4
POOL_SIZE = 360
CARDS_PER_ARCHETYPE = 40
GENERIC_CARDS = 40
RESONANCE_TYPES = ["Ember", "Stone", "Tide", "Zephyr"]
NUM_AIS = 7

# Config A: "As designed" (pool exhausts at ~pick 14)
CONFIG_A = {
    "name": "As-Designed",
    "ai_picks_per_round": 2,
    "cull_count": 10,
    "saturation_threshold": 16,
}

# Config B: "Calibrated" (supports 30 picks)
CONFIG_B = {
    "name": "Calibrated",
    "ai_picks_per_round": 1,
    "cull_count": 4,
    "saturation_threshold": 10,
}

# AI pick probabilities (same for both configs)
ACCUM_ARCHETYPE_RATE = 0.85
ACCUM_GENERIC_RATE = 0.15
SATUR_ARCHETYPE_RATE = 0.50
SATUR_ADJACENT_RATE = 0.30
SATUR_GENERIC_RATE = 0.20

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


def compute_adjacent(arch_name: str) -> List[str]:
    """Return list of archetype names that share at least one resonance."""
    _, r1, r2 = ARCH_BY_NAME[arch_name]
    adj = []
    for name, ar1, ar2 in ARCHETYPES:
        if name == arch_name:
            continue
        if r1 in (ar1, ar2) or r2 in (ar1, ar2):
            adj.append(name)
    return adj


ADJACENCY = {name: compute_adjacent(name) for name in ARCHETYPE_NAMES}

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

# Pessimistic fitness per co-primary pair (sibling rates -10pp)
FITNESS_PESSIMISTIC = {
    ("Warriors", "Sacrifice"): 0.40,
    ("Sacrifice", "Warriors"): 0.40,
    ("Self-Discard", "Self-Mill"): 0.30,
    ("Self-Mill", "Self-Discard"): 0.30,
    ("Blink", "Storm"): 0.20,
    ("Storm", "Blink"): 0.20,
    ("Flash", "Ramp"): 0.15,
    ("Ramp", "Flash"): 0.15,
}

FITNESS_MODELS = {
    "Graduated": FITNESS_GRADUATED,
    "Pessimistic": FITNESS_PESSIMISTIC,
}


def get_sibling(arch_name: str) -> Optional[str]:
    """Return co-primary sibling archetype name (shares primary resonance)."""
    r1 = ARCH_BY_NAME[arch_name][1]
    for a in ARCHETYPES:
        if a[0] != arch_name and a[1] == r1:
            return a[0]
    return None


# ============================================================
# Card & Pool Construction
# ============================================================
@dataclass
class SimCard:
    id: int
    visible_symbols: List[str]
    archetype: str
    power: float
    pair_affinity_0: float
    pair_affinity_1: float
    is_generic: bool = False


def build_pool() -> List[SimCard]:
    cards = []
    card_id = 0
    dual_per_arch = [4, 4, 4, 4, 5, 5, 5, 5]

    for arch_idx, (arch_name, r1, r2) in enumerate(ARCHETYPES):
        n_dual = dual_per_arch[arch_idx]
        n_single = CARDS_PER_ARCHETYPE - n_dual

        for _ in range(n_single):
            cards.append(SimCard(
                id=card_id,
                visible_symbols=[r1],
                archetype=arch_name,
                power=random.uniform(3, 9),
                pair_affinity_0=random.uniform(0.5, 1.0),
                pair_affinity_1=random.uniform(0.0, 0.5),
            ))
            card_id += 1

        for _ in range(n_dual):
            cards.append(SimCard(
                id=card_id,
                visible_symbols=[r1, r2],
                archetype=arch_name,
                power=random.uniform(4, 9),
                pair_affinity_0=random.uniform(0.6, 1.0),
                pair_affinity_1=random.uniform(0.3, 0.7),
            ))
            card_id += 1

    for _ in range(GENERIC_CARDS):
        cards.append(SimCard(
            id=card_id,
            visible_symbols=[],
            archetype="Generic",
            power=random.uniform(3, 7),
            pair_affinity_0=0.0,
            pair_affinity_1=0.0,
            is_generic=True,
        ))
        card_id += 1

    assert len(cards) == POOL_SIZE
    return cards


def precompute_sa_tiers(
    pool: List[SimCard],
    player_archetype: str,
    fitness_model: Dict,
) -> Dict[int, bool]:
    sibling = get_sibling(player_archetype)
    sa_map = {}
    for c in pool:
        if c.is_generic:
            sa_map[c.id] = False
        elif c.archetype == player_archetype:
            sa_map[c.id] = True
        elif sibling and c.archetype == sibling:
            rate = fitness_model.get((player_archetype, sibling), 0.0)
            sa_map[c.id] = (random.random() < rate)
        else:
            sa_map[c.id] = False
    return sa_map


# ============================================================
# AI Drafter
# ============================================================
@dataclass
class AIDrafter:
    archetype: str
    primary_res: str
    secondary_res: str
    adjacent_archetypes: List[str]
    saturation_threshold: int = 10
    drafted_archetype_count: int = 0
    total_drafted: int = 0
    saturated: bool = False
    saturation_pick: int = -1

    def is_saturated(self) -> bool:
        return self.drafted_archetype_count >= self.saturation_threshold

    def pick_cards(self, pool: List[SimCard], pick_num: int,
                   picks_per_round: int) -> List[SimCard]:
        picks = []
        available = list(pool)

        for _ in range(picks_per_round):
            if not available:
                break

            saturated = self.is_saturated()
            if saturated and not self.saturated:
                self.saturated = True
                self.saturation_pick = pick_num

            roll = random.random()
            if saturated:
                if roll < SATUR_ARCHETYPE_RATE:
                    category = "archetype"
                elif roll < SATUR_ARCHETYPE_RATE + SATUR_ADJACENT_RATE:
                    category = "adjacent"
                else:
                    category = "generic"
            else:
                if roll < ACCUM_ARCHETYPE_RATE:
                    category = "archetype"
                else:
                    category = "generic"

            chosen = self._pick_from_category(available, category)
            if chosen is not None:
                picks.append(chosen)
                available = [c for c in available if c.id != chosen.id]
                if chosen.archetype == self.archetype:
                    self.drafted_archetype_count += 1
                self.total_drafted += 1

        return picks

    def _pick_from_category(self, available: List[SimCard],
                            category: str) -> Optional[SimCard]:
        if category == "archetype":
            arch_cards = [c for c in available if c.archetype == self.archetype]
            if arch_cards:
                return max(arch_cards,
                           key=lambda c: c.pair_affinity_0 + c.power * 0.05)
            res_cards = [c for c in available
                         if self.primary_res in c.visible_symbols
                         and not c.is_generic]
            if res_cards:
                return max(res_cards,
                           key=lambda c: c.pair_affinity_0 + c.power * 0.05)
            return max(available, key=lambda c: c.power) if available else None

        elif category == "adjacent":
            adj_cards = [c for c in available
                         if c.archetype in self.adjacent_archetypes]
            if adj_cards:
                return max(adj_cards,
                           key=lambda c: c.pair_affinity_1 + c.power * 0.05)
            res_cards = [c for c in available
                         if self.secondary_res in c.visible_symbols
                         and not c.is_generic]
            if res_cards:
                return max(res_cards,
                           key=lambda c: c.pair_affinity_1 + c.power * 0.05)
            return max(available, key=lambda c: c.power) if available else None

        else:
            generics = [c for c in available if c.is_generic]
            if generics:
                return max(generics, key=lambda c: c.power)
            return max(available, key=lambda c: c.power) if available else None


def create_ai_drafters(excluded_archetype: str,
                       saturation_threshold: int) -> List[AIDrafter]:
    drafters = []
    for name, r1, r2 in ARCHETYPES:
        if name == excluded_archetype:
            continue
        drafters.append(AIDrafter(
            archetype=name,
            primary_res=r1,
            secondary_res=r2,
            adjacent_archetypes=ADJACENCY[name],
            saturation_threshold=saturation_threshold,
        ))
    return drafters


# ============================================================
# Supplemental Culling
# ============================================================
def cull_lowest_power(pool: List[SimCard], n_cull: int) -> List[SimCard]:
    if len(pool) <= n_cull:
        return pool
    sorted_pool = sorted(pool, key=lambda c: c.power)
    to_remove = set(c.id for c in sorted_pool[:n_cull])
    return [c for c in pool if c.id not in to_remove]


# ============================================================
# Draft Engine
# ============================================================
def run_draft(
    pool: List[SimCard],
    player_archetype: str,
    fitness_model: Dict,
    strategy: str,
    config: Dict,
    open_archetype: Optional[str] = None,
) -> Tuple[List[Dict], List[SimCard], Dict[int, bool], List[AIDrafter], str]:
    ai_picks_per_round = config["ai_picks_per_round"]
    cull_count = config["cull_count"]
    saturation_threshold = config["saturation_threshold"]

    if open_archetype is None:
        open_archetype = random.choice(ARCHETYPE_NAMES)

    ai_drafters = create_ai_drafters(open_archetype, saturation_threshold)
    active_pool = list(pool)
    drafted: List[SimCard] = []
    history: List[Dict] = []
    sa_cache = precompute_sa_tiers(pool, player_archetype, fitness_model)

    for pick in range(1, NUM_PICKS + 1):
        if len(active_pool) < PACK_SIZE:
            break

        pool_start = len(active_pool)

        # Phase 1: AI drafters pick
        round_ai_picks = []
        for ai in ai_drafters:
            ai_picked = ai.pick_cards(active_pool, pick, ai_picks_per_round)
            round_ai_picks.extend(ai_picked)
            picked_ids = set(c.id for c in ai_picked)
            active_pool = [c for c in active_pool if c.id not in picked_ids]

        # Phase 2: Supplemental culling
        pool_before_cull = len(active_pool)
        active_pool = cull_lowest_power(active_pool, cull_count)
        actual_culled = pool_before_cull - len(active_pool)

        if len(active_pool) < PACK_SIZE:
            break

        # Phase 3: Player pack
        pack = random.sample(active_pool, min(PACK_SIZE, len(active_pool)))

        # Phase 4: Player picks
        chosen = select_card(pack, player_archetype, drafted, strategy,
                             pick, sa_cache)
        drafted.append(chosen)
        active_pool = [c for c in active_pool if c.id != chosen.id]

        sa_count = sum(1 for c in pack if sa_cache[c.id])
        history.append({
            "pick": pick,
            "pack": pack,
            "chosen": chosen,
            "pool_size": pool_start,
            "pool_after": len(active_pool),
            "sa_count": sa_count,
            "ai_picks": len(round_ai_picks),
            "culled": actual_culled,
            "open_archetype": open_archetype,
            "saturated_count": sum(1 for ai in ai_drafters if ai.saturated),
        })

    return history, drafted, sa_cache, ai_drafters, open_archetype


def select_card(
    pack: List[SimCard],
    player_archetype: str,
    drafted: List[SimCard],
    strategy: str,
    pick: int,
    sa_cache: Dict[int, bool],
) -> SimCard:
    arch = ARCH_BY_NAME[player_archetype]
    r1, r2 = arch[1], arch[2]

    if strategy == "committed":
        def score(c: SimCard) -> float:
            s = 0.0
            if sa_cache.get(c.id, False):
                s += 10.0
            for i, sym in enumerate(c.visible_symbols):
                if sym == r1:
                    s += 3.0 if i == 0 else 1.0
                elif sym == r2:
                    s += 2.0 if i == 0 else 1.0
            s += c.power * 0.1
            return s
        return max(pack, key=score)

    elif strategy == "power":
        return max(pack, key=lambda c: c.power)

    elif strategy == "signal":
        if pick <= 3:
            return max(pack, key=lambda c: c.power)
        res_counts = Counter()
        for c in drafted:
            for sym in c.visible_symbols:
                res_counts[sym] += 1
        for c in pack:
            for sym in c.visible_symbols:
                res_counts[sym] += 0.5

        if not res_counts:
            return max(pack, key=lambda c: c.power)

        top_res = max(RESONANCE_TYPES, key=lambda r: res_counts.get(r, 0))

        def signal_score(c: SimCard) -> float:
            s = 0.0
            for i, sym in enumerate(c.visible_symbols):
                if sym == top_res:
                    s += 3.0 if i == 0 else 1.0
            s += c.power * 0.1
            if sa_cache.get(c.id, False):
                s += 5.0
            return s
        return max(pack, key=signal_score)

    return random.choice(pack)


# ============================================================
# Metrics
# ============================================================
def compute_metrics(all_results: List[Tuple]) -> Dict:
    m1_vals, m2_vals, m3_vals, m4_vals = [], [], [], []
    m5_vals, m6_vals, m9_vals, m10_vals = [], [], [], []
    m11_vals = []
    post_commit_sa = []
    consec_bad_list = []
    all_drafted_ids: List[List[int]] = []
    pick_counts = []

    for history, drafted, sa_cache, _, _ in all_results:
        pick_counts.append(len(history))
        all_drafted_ids.append([c.id for c in drafted])

        # M1: picks 1-5
        early_arch = []
        for h in history[:5]:
            archs = set(c.archetype for c in h["pack"] if not c.is_generic)
            early_arch.append(len(archs))
        if early_arch:
            m1_vals.append(sum(early_arch) / len(early_arch))

        # M2: picks 1-5
        early_sa = []
        for h in history[:5]:
            sa = sum(1 for c in h["pack"] if sa_cache[c.id])
            early_sa.append(sa)
        if early_sa:
            m2_vals.append(sum(early_sa) / len(early_sa))

        # M3: picks 6+
        post_sa = []
        for h in history[5:]:
            sa = h["sa_count"]
            post_sa.append(sa)
            post_commit_sa.append(sa)
        if post_sa:
            m3_vals.append(sum(post_sa) / len(post_sa))

        # M4: picks 6+
        post_off = []
        for h in history[5:]:
            off = sum(1 for c in h["pack"] if not sa_cache[c.id])
            post_off.append(off)
        if post_off:
            m4_vals.append(sum(post_off) / len(post_off))

        # M5: convergence
        conv_pick = NUM_PICKS
        all_sa = [h["sa_count"] for h in history]
        for i in range(2, len(all_sa)):
            window = all_sa[max(0, i - 2):i + 1]
            if sum(window) / len(window) >= 1.5:
                conv_pick = i + 1
                break
        m5_vals.append(conv_pick)

        # M6: deck concentration
        sa_drafted = sum(1 for c in drafted if sa_cache[c.id])
        m6_vals.append(sa_drafted / max(1, len(drafted)))

        # M9: stddev
        if len(post_sa) > 1:
            mean_sa = sum(post_sa) / len(post_sa)
            var = sum((x - mean_sa) ** 2 for x in post_sa) / len(post_sa)
            m9_vals.append(math.sqrt(var))

        # M10: consecutive bad
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

        # M11: picks 15+
        late_sa_picks = []
        for h in history[14:]:
            sa = h["sa_count"]
            late_sa_picks.append(sa)
        if late_sa_picks:
            m11_vals.append(sum(late_sa_picks) / len(late_sa_picks))

    # M7: overlap
    m7_overlaps = []
    for i in range(1, len(all_drafted_ids)):
        ids_prev = set(all_drafted_ids[i - 1])
        ids_curr = set(all_drafted_ids[i])
        if ids_prev | ids_curr:
            overlap = len(ids_prev & ids_curr) / len(ids_prev | ids_curr)
            m7_overlaps.append(overlap)

    pq_sorted = sorted(post_commit_sa)
    n = len(pq_sorted)
    pcts = {}
    for p in [10, 25, 50, 75, 90]:
        idx = min(int(n * p / 100), n - 1)
        pcts[p] = pq_sorted[idx] if n > 0 else 0

    avg = lambda vs: sum(vs) / max(1, len(vs))

    return {
        "M1": avg(m1_vals),
        "M2": avg(m2_vals),
        "M3": avg(m3_vals),
        "M4": avg(m4_vals),
        "M5": avg(m5_vals),
        "M6": avg(m6_vals),
        "M7": avg(m7_overlaps) if m7_overlaps else 0.0,
        "M9": avg(m9_vals),
        "M10": avg(m10_vals),
        "M10_max": max(m10_vals) if m10_vals else 0,
        "M11": avg(m11_vals) if m11_vals else 0.0,
        "pack_pcts": pcts,
        "avg_consec_bad": avg(consec_bad_list),
        "worst_consec_bad": max(consec_bad_list) if consec_bad_list else 0,
        "avg_picks": avg(pick_counts),
        "min_picks": min(pick_counts) if pick_counts else 0,
    }


# ============================================================
# Runners
# ============================================================
def run_aggregate(fitness_model, strategy, config, n_drafts=NUM_DRAFTS):
    all_results = []
    for i in range(n_drafts):
        arch_name = ARCHETYPE_NAMES[i % 8]
        pool = build_pool()
        open_arch = random.choice(ARCHETYPE_NAMES)
        result = run_draft(pool, arch_name, fitness_model, strategy, config,
                           open_arch)
        all_results.append(result)
    return compute_metrics(all_results), all_results


def run_per_archetype(fitness_model, strategy, config, n_per=125):
    results = {}
    for arch_name in ARCHETYPE_NAMES:
        histories = []
        for _ in range(n_per):
            pool = build_pool()
            open_arch = random.choice(ARCHETYPE_NAMES)
            result = run_draft(pool, arch_name, fitness_model, strategy,
                               config, open_arch)
            histories.append(result)
        results[arch_name] = compute_metrics(histories)
    return results


def run_open_vs_contested(fitness_model, strategy, config, n_drafts=500):
    open_results = []
    contested_results = []
    for i in range(n_drafts):
        arch_name = ARCHETYPE_NAMES[i % 8]

        pool = build_pool()
        result_open = run_draft(pool, arch_name, fitness_model, strategy,
                                config, arch_name)
        open_results.append(result_open)

        pool2 = build_pool()
        other_archs = [a for a in ARCHETYPE_NAMES if a != arch_name]
        open_other = random.choice(other_archs)
        result_contested = run_draft(pool2, arch_name, fitness_model, strategy,
                                     config, open_other)
        contested_results.append(result_contested)

    return compute_metrics(open_results), compute_metrics(contested_results)


# ============================================================
# Draft Trace Formatter
# ============================================================
def format_trace(history, drafted, sa_cache, ai_drafters,
                 player_archetype, open_archetype):
    lines = [
        f"=== Draft Trace: Player={player_archetype}, "
        f"Open Lane={open_archetype} ===",
        f"    AI lanes: {', '.join(ai.archetype for ai in ai_drafters)}",
        f"    Player {'IN OPEN LANE' if player_archetype == open_archetype else 'IN CONTESTED LANE'}",
    ]

    for h in history:
        pick = h["pick"]
        sa = h["sa_count"]
        pool_sz = h.get("pool_size", 0)
        pool_after = h.get("pool_after", 0)
        chosen = h["chosen"]
        chosen_sa = "S/A" if sa_cache[chosen.id] else "C/F"
        sym_str = ("/".join(chosen.visible_symbols)
                   if chosen.visible_symbols else "Generic")
        sat_count = h.get("saturated_count", 0)

        lines.append(
            f"  Pick {pick:2d}: pool={pool_sz:3d}->{pool_after:3d} "
            f"S/A={sa} sat={sat_count}/7 "
            f"[{chosen.archetype}:{sym_str}] ({chosen_sa})"
        )

    sa_d = sum(1 for c in drafted if sa_cache[c.id])
    lines.append(
        f"  Final: {sa_d}/{len(drafted)} S/A = "
        f"{sa_d / max(1, len(drafted)) * 100:.0f}%"
    )

    lines.append(f"\n  AI Saturation Summary:")
    for ai in ai_drafters:
        sat_str = (f"saturated at pick {ai.saturation_pick}"
                   if ai.saturated else "never saturated")
        lines.append(
            f"    {ai.archetype:<14}: {ai.drafted_archetype_count} arch cards, "
            f"{ai.total_drafted} total, {sat_str}"
        )

    return "\n".join(lines)


# ============================================================
# Main
# ============================================================
def main():
    random.seed(42)

    print("=" * 72)
    print("D3 COMPETITIVE PRESSURE WITH SAFETY VALVE (revised)")
    print("V10 Simulation Agent 6")
    print("=" * 72)

    fitness_grad = FITNESS_MODELS["Graduated"]
    fitness_pess = FITNESS_MODELS["Pessimistic"]

    # We use Config B (calibrated) as primary
    config = CONFIG_B
    print(f"\nPrimary Config: {config['name']}")
    print(f"  AIs: {NUM_AIS}, Picks/AI/round: {config['ai_picks_per_round']}")
    print(f"  Saturation threshold: {config['saturation_threshold']} arch cards")
    print(f"  Supplemental cull: {config['cull_count']}/round")
    total_per_round = (NUM_AIS * config['ai_picks_per_round']
                       + config['cull_count'] + 1)
    print(f"  Total removal/round: ~{total_per_round}")
    print(f"  Expected picks: ~{POOL_SIZE // total_per_round}")

    # =====================================================
    # Config A quick summary (as-designed, pool exhausts early)
    # =====================================================
    print("\n" + "=" * 72)
    print("CONFIG A: AS-DESIGNED (2/AI + 10 cull = 25/round)")
    print("=" * 72)
    random.seed(42)
    m_a, _ = run_aggregate(fitness_grad, "committed", CONFIG_A, n_drafts=200)
    print(f"  Avg picks per draft: {m_a['avg_picks']:.1f} "
          f"(min: {m_a['min_picks']})")
    print(f"  M3={m_a['M3']:.2f}  M5={m_a['M5']:.1f}  "
          f"M10={m_a['M10']:.1f}  M11={m_a['M11']:.2f}")
    print(f"  VERDICT: Pool exhausts too quickly. Not viable for 30-pick draft.")

    # =====================================================
    # Config B: Calibrated — all 3 strategies
    # =====================================================
    random.seed(42)
    print("\n" + "=" * 72)
    print(f"CONFIG B: CALIBRATED ({config['ai_picks_per_round']}/AI + "
          f"{config['cull_count']} cull = {total_per_round}/round)")
    print("GRADUATED REALISTIC FITNESS -- All Strategies")
    print("=" * 72)

    results_grad = {}
    all_results_grad = {}
    for strat in ["committed", "power", "signal"]:
        m, results = run_aggregate(fitness_grad, strat, config)
        results_grad[strat] = m
        all_results_grad[strat] = results
        print(f"\nStrategy: {strat} (avg picks: {m['avg_picks']:.1f})")
        print(f"  M1={m['M1']:.2f}  M2={m['M2']:.2f}  M3={m['M3']:.2f}  "
              f"M4={m['M4']:.2f}  M5={m['M5']:.1f}")
        print(f"  M6={m['M6']:.2f}  M7={m['M7']:.3f}  M9={m['M9']:.2f}  "
              f"M10={m['M10']:.1f}  M11={m['M11']:.2f}")
        pq = m['pack_pcts']
        print(f"  Pack pcts (P10/P25/P50/P75/P90): "
              f"{pq[10]}/{pq[25]}/{pq[50]}/{pq[75]}/{pq[90]}")

    # =====================================================
    # Pessimistic
    # =====================================================
    print("\n" + "=" * 72)
    print("PESSIMISTIC FITNESS -- Committed Strategy")
    print("=" * 72)

    m_pess, _ = run_aggregate(fitness_pess, "committed", config)
    print(f"  M3={m_pess['M3']:.2f}  M10={m_pess['M10']:.1f}  "
          f"M11={m_pess['M11']:.2f}  M6={m_pess['M6']:.2f}")
    pq = m_pess['pack_pcts']
    print(f"  Pack pcts: P10={pq[10]} P25={pq[25]} P50={pq[50]} "
          f"P75={pq[75]} P90={pq[90]}")

    # =====================================================
    # Per-archetype
    # =====================================================
    print("\n" + "=" * 72)
    print("PER-ARCHETYPE: Graduated Realistic, committed")
    print("=" * 72)

    pa_results = run_per_archetype(fitness_grad, "committed", config)
    print(f"  {'Archetype':<16} {'M3':>6} {'M5':>6} {'M6':>6} "
          f"{'M9':>6} {'M10':>6} {'M11':>6}")
    m3_vals = []
    for arch in ARCHETYPE_NAMES:
        m = pa_results[arch]
        m3_vals.append(m['M3'])
        print(f"  {arch:<16} {m['M3']:6.2f} {m['M5']:6.1f} "
              f"{m['M6']:6.2f} {m['M9']:6.2f} {m['M10']:6.1f} "
              f"{m['M11']:6.2f}")
    spread = max(m3_vals) - min(m3_vals)
    print(f"\n  M3 spread (max-min): {spread:.3f}")
    print(f"  M3 worst: {ARCHETYPE_NAMES[m3_vals.index(min(m3_vals))]}: "
          f"{min(m3_vals):.2f}")
    print(f"  M3 best:  {ARCHETYPE_NAMES[m3_vals.index(max(m3_vals))]}: "
          f"{max(m3_vals):.2f}")

    # =====================================================
    # Open vs Contested
    # =====================================================
    print("\n" + "=" * 72)
    print("OPEN LANE vs CONTESTED LANE (Graduated, committed)")
    print("=" * 72)

    m_open, m_contested = run_open_vs_contested(fitness_grad, "committed",
                                                 config)
    print(f"  {'Condition':<16} {'M3':>6} {'M10':>6} {'M11':>6} {'M6':>6}")
    print(f"  {'Open lane':<16} {m_open['M3']:6.2f} {m_open['M10']:6.1f} "
          f"{m_open['M11']:6.2f} {m_open['M6']:6.2f}")
    print(f"  {'Contested':<16} {m_contested['M3']:6.2f} "
          f"{m_contested['M10']:6.1f} {m_contested['M11']:6.2f} "
          f"{m_contested['M6']:6.2f}")

    # =====================================================
    # AI Behavior Summary
    # =====================================================
    print("\n" + "=" * 72)
    print("AI BEHAVIOR SUMMARY (committed strategy runs)")
    print("=" * 72)

    sat_picks = []
    never_sat = 0
    total_ai_cards = []
    total_arch_cards = []
    for history, drafted, sa_cache, ai_drafters, open_arch in \
            all_results_grad["committed"]:
        for ai in ai_drafters:
            total_ai_cards.append(ai.total_drafted)
            total_arch_cards.append(ai.drafted_archetype_count)
            if ai.saturated:
                sat_picks.append(ai.saturation_pick)
            else:
                never_sat += 1

    total_ais = len(all_results_grad["committed"]) * NUM_AIS
    print(f"  Total AIs across all drafts: {total_ais}")
    print(f"  AIs that saturated: {len(sat_picks)} "
          f"({len(sat_picks) / total_ais * 100:.1f}%)")
    print(f"  AIs that never saturated: {never_sat} "
          f"({never_sat / total_ais * 100:.1f}%)")
    if sat_picks:
        print(f"  Saturation timing (of those that did):")
        print(f"    Mean: {sum(sat_picks) / len(sat_picks):.1f}")
        print(f"    Min: {min(sat_picks)}, Max: {max(sat_picks)}")
        print(f"    Median: {sorted(sat_picks)[len(sat_picks) // 2]}")

        sat_counter = Counter()
        for sp in sat_picks:
            bucket = (sp - 1) // 5 * 5 + 1
            sat_counter[f"pick {bucket}-{bucket + 4}"] += 1
        print(f"  Saturation timing distribution:")
        for k in sorted(sat_counter.keys()):
            pct = sat_counter[k] / len(sat_picks) * 100
            print(f"    {k}: {sat_counter[k]} ({pct:.1f}%)")

    avg_ai_cards = (sum(total_ai_cards) / len(total_ai_cards)
                    if total_ai_cards else 0)
    avg_arch_cards = (sum(total_arch_cards) / len(total_arch_cards)
                      if total_arch_cards else 0)
    print(f"  Avg cards per AI: {avg_ai_cards:.1f}")
    print(f"  Avg archetype cards per AI: {avg_arch_cards:.1f}")

    # =====================================================
    # Pack quality distribution
    # =====================================================
    print("\n" + "=" * 72)
    print("PACK QUALITY DISTRIBUTION (picks 6+, Graduated, committed)")
    print("=" * 72)
    m = results_grad["committed"]
    pq = m["pack_pcts"]
    print(f"  P10={pq[10]}  P25={pq[25]}  P50={pq[50]}  "
          f"P75={pq[75]}  P90={pq[90]}")
    print(f"  Avg consecutive bad packs (<1.5 S/A): {m['avg_consec_bad']:.2f}")
    print(f"  Worst consecutive bad: {m['worst_consec_bad']}")

    consec_dist = Counter()
    for history, drafted, sa_cache, _, _ in all_results_grad["committed"]:
        post_sa = [h["sa_count"] for h in history[5:]]
        max_c = 0
        cur_c = 0
        for sa in post_sa:
            if sa < 1.5:
                cur_c += 1
                max_c = max(max_c, cur_c)
            else:
                cur_c = 0
        consec_dist[max_c] += 1

    print(f"\n  Max consecutive bad pack distribution:")
    for k in sorted(consec_dist.keys())[:12]:
        pct = consec_dist[k] / sum(consec_dist.values()) * 100
        print(f"    {k}: {consec_dist[k]} drafts ({pct:.1f}%)")

    # =====================================================
    # Draft traces
    # =====================================================
    print("\n" + "=" * 72)
    print("DRAFT TRACES")
    print("=" * 72)

    # Trace 1: Committed player in OPEN lane (Warriors)
    random.seed(100)
    pool_t1 = build_pool()
    h1, d1, c1, ais1, open1 = run_draft(
        pool_t1, "Warriors", fitness_grad, "committed", config, "Warriors"
    )
    print("\n" + format_trace(h1, d1, c1, ais1, "Warriors", open1))

    # Trace 2: Signal reader in CONTESTED lane (Storm)
    random.seed(200)
    pool_t2 = build_pool()
    h2, d2, c2, ais2, open2 = run_draft(
        pool_t2, "Storm", fitness_grad, "signal", config, "Ramp"
    )
    print("\n" + format_trace(h2, d2, c2, ais2, "Storm", open2))

    # =====================================================
    # Pool evolution
    # =====================================================
    print("\n" + "=" * 72)
    print("POOL EVOLUTION (from Trace 1)")
    print("=" * 72)
    for h in h1:
        pick = h["pick"]
        pool_sz = h.get("pool_size", 0)
        pool_after = h.get("pool_after", 0)
        removal = pool_sz - pool_after
        contraction = removal / pool_sz * 100 if pool_sz > 0 else 0
        print(f"  Pick {pick:2d}: {pool_sz:3d} -> {pool_after:3d} "
              f"(removed {removal}, {contraction:.1f}%)")

    # =====================================================
    # Scorecard
    # =====================================================
    print("\n" + "=" * 72)
    print("SCORECARD SUMMARY")
    print("=" * 72)

    m = results_grad["committed"]
    targets = [
        ("M1  (early variety >= 3.0)",      m['M1'],  3.0,  ">="),
        ("M2  (early S/A <= 2.0)",          m['M2'],  2.0,  "<="),
        ("M3  (post-commit S/A >= 2.0)",    m['M3'],  2.0,  ">="),
        ("M4  (off-arch >= 0.5)",           m['M4'],  0.5,  ">="),
        ("M5  (convergence 5-8)",           m['M5'],  8.0,  "<="),
        ("M6  (concentration 60-90%)",      m['M6'],  0.60, ">="),
        ("M7  (variety < 40% overlap)",     m['M7'],  0.40, "<="),
        ("M9  (stddev >= 0.8)",             m['M9'],  0.8,  ">="),
        ("M10 (consec bad <= 2)",           m['M10'], 2.0,  "<="),
        ("M11 (late S/A >= 3.0)",           m['M11'], 3.0,  ">="),
    ]

    for name, val, target, direction in targets:
        if direction == ">=":
            passed = val >= target
        else:
            passed = val <= target
        status = "PASS" if passed else "FAIL"
        print(f"  {name:<38} {val:6.2f}  {status}")

    m5_pass = 5.0 <= m['M5'] <= 8.0
    print(f"\n  M5 range check [5,8]:                  {m['M5']:6.1f}  "
          f"{'PASS' if m5_pass else 'FAIL'}")
    m6_upper = m['M6'] <= 0.90
    print(f"  M6 upper bound <= 90%:                 {m['M6']:6.2f}  "
          f"{'PASS' if m6_upper else 'FAIL'}")

    print(f"\n  Pessimistic (committed):")
    print(f"    M3:  {m_pess['M3']:.2f}  M10: {m_pess['M10']:.1f}  "
          f"M11: {m_pess['M11']:.2f}")

    print(f"\n  V9 Hybrid B Comparison:")
    print(f"    {'Metric':<8} {'V9 HybB':>10} {'D3':>10} {'Delta':>8}")
    print(f"    {'M3':<8} {'2.70':>10} {m['M3']:10.2f} "
          f"{m['M3'] - 2.70:+8.2f}")
    print(f"    {'M10':<8} {'3.8':>10} {m['M10']:10.1f} "
          f"{m['M10'] - 3.8:+8.1f}")
    print(f"    {'M11':<8} {'3.25':>10} {m['M11']:10.2f} "
          f"{m['M11'] - 3.25:+8.2f}")
    print(f"    {'M5':<8} {'9.6':>10} {m['M5']:10.1f} "
          f"{m['M5'] - 9.6:+8.1f}")

    print("\nDone.")


if __name__ == "__main__":
    main()
