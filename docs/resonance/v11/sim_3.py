"""
Simulation Agent 3: SIM-3 — Hybrid A (Graduated Bias + Declining Volume)
V11 Round 4

Algorithm: 3 rounds x 10 picks, 120-card starting pool.
  - After Round 1: 70 cards added, 1.4x open-lane multiplier.
  - After Round 2: 48 cards added, 2.0x open-lane multiplier.
  - Full Design 5 information system (bars + snapshot + depletion trends).
  - 5 Level-0 AIs, each assigned one unique archetype. 3 open lanes.
  - AI pick logic: highest fitness for assigned archetype, saturation at 10.
  - 10% deviation to adjacent archetype.
  - Player strategies: committed, signal-reader (commits pick 5), power-chaser.
  - S/A evaluation: all on-archetype cards are S/A; sibling cards have
    graduated fitness chance. Matches V9 convention.

This is the primary "Standard" candidate for V11.
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
TOTAL_PICKS = 30
PACK_SIZE = 5
STARTING_POOL = 120
ROUNDS = 3
PICKS_PER_ROUND = 10
NUM_AIS = 5
NUM_OPEN_LANES = 3
AI_SATURATION_THRESHOLD = 10
AI_DEVIATION_RATE = 0.10

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

# Graduated Realistic fitness
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

# Adjacency on the archetype circle
ADJACENT = {}
for i, (name, _, _) in enumerate(ARCHETYPES):
    ADJACENT[name] = [ARCHETYPES[(i - 1) % 8][0], ARCHETYPES[(i + 1) % 8][0]]

# Refill schedule: (total_cards, open_lane_multiplier)
REFILL_SCHEDULE = [
    (70, 1.4),   # After Round 1
    (48, 2.0),   # After Round 2
]


def get_sibling(arch_name: str) -> Optional[str]:
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


_card_id_counter = 0


def next_card_id() -> int:
    global _card_id_counter
    _card_id_counter += 1
    return _card_id_counter


def reset_card_ids():
    global _card_id_counter
    _card_id_counter = 0


def make_archetype_card(archetype, r1, r2, is_dual=False):
    cid = next_card_id()
    symbols = [r1, r2] if is_dual else [r1]
    return SimCard(id=cid, visible_symbols=symbols, archetype=archetype,
                   power=random.uniform(3, 9), is_generic=False)


def make_generic_card():
    cid = next_card_id()
    return SimCard(id=cid, visible_symbols=[], archetype="Generic",
                   power=random.uniform(2, 7), is_generic=True)


def build_archetype_batch(archetype, count):
    _, r1, r2 = ARCH_BY_NAME[archetype]
    n_dual = max(1, round(count * 0.115))
    cards = []
    for _ in range(count - n_dual):
        cards.append(make_archetype_card(archetype, r1, r2, False))
    for _ in range(n_dual):
        cards.append(make_archetype_card(archetype, r1, r2, True))
    return cards


def build_pool():
    """120-card starting pool: 8 archetypes x 13 + 16 generics."""
    cards = []
    n_per_arch = 13
    for arch_name, _, _ in ARCHETYPES:
        cards.extend(build_archetype_batch(arch_name, n_per_arch))
    for _ in range(STARTING_POOL - n_per_arch * 8):
        cards.append(make_generic_card())
    return cards


def build_refill_batch(total_cards, open_mult, open_lanes, ai_lanes):
    """Build refill with open-lane bias."""
    n_generic = max(1, round(total_cards * 0.11))
    n_arch_total = total_cards - n_generic
    base = n_arch_total / 8.0
    total_open = len(open_lanes) * base * open_mult
    total_ai = n_arch_total - total_open
    per_ai = max(1.0, total_ai / max(1, len(ai_lanes)))

    cards = []
    for arch_name, _, _ in ARCHETYPES:
        n = max(1, round(base * open_mult if arch_name in open_lanes else per_ai))
        cards.extend(build_archetype_batch(arch_name, n))
    for _ in range(n_generic):
        cards.append(make_generic_card())
    return cards


# ============================================================
# S/A Evaluation (V9 convention)
# ============================================================
def precompute_sa(pool, player_archetype, fitness_model):
    """All on-archetype = S/A. Sibling has graduated fitness chance."""
    sibling = get_sibling(player_archetype)
    sa = {}
    for c in pool:
        if c.is_generic:
            sa[c.id] = False
        elif c.archetype == player_archetype:
            sa[c.id] = True
        elif sibling and c.archetype == sibling:
            rate = fitness_model.get((player_archetype, sibling), 0.0)
            sa[c.id] = random.random() < rate
        else:
            sa[c.id] = False
    return sa


def sa_single(card, player_archetype, fitness_model):
    """S/A check for one card (refill)."""
    if card.is_generic:
        return False
    if card.archetype == player_archetype:
        return True
    sibling = get_sibling(player_archetype)
    if sibling and card.archetype == sibling:
        return random.random() < fitness_model.get((player_archetype, sibling), 0.0)
    return False


# ============================================================
# AI Drafter
# ============================================================
@dataclass
class AIDrafter:
    archetype: str
    primary_res: str
    secondary_res: str
    cards_drafted: int = 0
    on_archetype_count: int = 0

    def pick_from_pool(self, pool):
        if not pool:
            return None
        saturated = self.on_archetype_count >= AI_SATURATION_THRESHOLD
        deviating = random.random() < AI_DEVIATION_RATE
        adj = ADJACENT[self.archetype]

        def score(c):
            if saturated:
                return (5.0 + c.power * 0.5) if c.is_generic else (
                    4.0 + c.power * 0.3 if c.archetype in adj else c.power * 0.2)
            if deviating and c.archetype in adj:
                return 8.0 + c.power * 0.3
            if c.archetype == self.archetype:
                return 10.0 + c.power * 0.5
            if self.primary_res in c.visible_symbols:
                return 2.0 + c.power * 0.1
            return c.power * (0.2 if c.is_generic else 0.1)

        best = max(pool, key=score)
        self.cards_drafted += 1
        if best.archetype == self.archetype:
            self.on_archetype_count += 1
        return best


# ============================================================
# Pack Construction
# ============================================================
def draw_pack(pool, pack_size=PACK_SIZE):
    """Draw pack weighted by archetype representation."""
    if len(pool) <= pack_size:
        return list(pool)

    arch_counts = Counter(c.archetype for c in pool)
    total = len(pool)
    weights = [arch_counts[c.archetype] / total for c in pool]
    wsum = sum(weights)
    weights = [w / wsum for w in weights]

    indices = list(range(len(pool)))
    chosen_set = set()
    result = []

    for _ in range(pack_size):
        adj = [weights[i] if i not in chosen_set else 0.0 for i in indices]
        ws = sum(adj)
        if ws <= 0:
            remaining = [i for i in indices if i not in chosen_set]
            if not remaining:
                break
            idx = random.choice(remaining)
        else:
            adj = [w / ws for w in adj]
            idx = random.choices(indices, weights=adj, k=1)[0]
        chosen_set.add(idx)
        result.append(pool[idx])

    return result


# ============================================================
# Player Strategies
# ============================================================
def player_select(pack, strategy, player_archetype, sa_cache, pick_number,
                  resonance_sig, pool_info):

    if strategy == "committed":
        _, r1, r2 = ARCH_BY_NAME[player_archetype]

        def committed_score(c):
            s = 10.0 if sa_cache.get(c.id, False) else 0.0
            for i, sym in enumerate(c.visible_symbols):
                if sym == r1:
                    s += 3.0 if i == 0 else 1.5
                elif sym == r2:
                    s += 2.0 if i == 0 else 1.0
            return s + c.power * 0.1
        return max(pack, key=committed_score)

    elif strategy == "power":
        return max(pack, key=lambda c: c.power)

    elif strategy == "signal":
        if pick_number <= 4:
            return max(pack, key=lambda c: c.power)

        # Commit at pick 5 to archetype with most pool presence
        arch_avail = pool_info.get("arch_counts", {})
        if pick_number == 5 and arch_avail:
            best = max((a for a in ARCHETYPE_NAMES if a in arch_avail),
                       key=lambda a: arch_avail.get(a, 0), default=player_archetype)
            pool_info["signal_committed_arch"] = best

        committed = pool_info.get("signal_committed_arch", player_archetype)
        info = ARCH_BY_NAME.get(committed)
        if info:
            _, r1, r2 = info

            def sig_score(c):
                s = 8.0 if sa_cache.get(c.id, False) else 0.0
                for i, sym in enumerate(c.visible_symbols):
                    if sym == r1:
                        s += 3.0 if i == 0 else 1.5
                    elif sym == r2:
                        s += 2.0 if i == 0 else 1.0
                return s + c.power * 0.15
            return max(pack, key=sig_score)

        return max(pack, key=lambda c: c.power)

    return random.choice(pack)


# ============================================================
# Core Draft Engine
# ============================================================
def run_draft(player_archetype, fitness_model, strategy, seed=None):
    if seed is not None:
        random.seed(seed)

    reset_card_ids()
    pool = build_pool()

    # Assign AI archetypes
    shuffled = list(ARCHETYPE_NAMES)
    random.shuffle(shuffled)
    ai_archetypes = shuffled[:NUM_AIS]
    open_lanes = set(shuffled[NUM_AIS:])
    ai_lanes = set(ai_archetypes)

    ais = [AIDrafter(a, ARCH_BY_NAME[a][1], ARCH_BY_NAME[a][2]) for a in ai_archetypes]
    sa_cache = precompute_sa(pool, player_archetype, fitness_model)

    drafted, history = [], []
    resonance_sig = {r: 0.0 for r in RESONANCE_TYPES}
    pool_info = {}
    pick_number = 0
    round_compositions = []

    for round_num in range(1, ROUNDS + 1):
        arch_counts = Counter(c.archetype for c in pool if not c.is_generic)
        info = {name: arch_counts.get(name, 0) for name in ARCHETYPE_NAMES}
        sa_pool = sum(1 for c in pool if sa_cache.get(c.id, False))

        round_compositions.append({
            "round": round_num, "pool_size": len(pool),
            "arch_counts": dict(info),
            "open_lane_total": sum(info.get(a, 0) for a in open_lanes),
            "ai_lane_total": sum(info.get(a, 0) for a in ai_lanes),
            "sa_count": sa_pool,
        })

        for _ in range(PICKS_PER_ROUND):
            pick_number += 1
            if len(pool) < PACK_SIZE:
                break

            pool_info_current = {
                "arch_counts": {n: sum(1 for c in pool if c.archetype == n)
                                for n in ARCHETYPE_NAMES}
            }
            # Carry over signal commitment
            if "signal_committed_arch" in pool_info:
                pool_info_current["signal_committed_arch"] = pool_info["signal_committed_arch"]

            pack = draw_pack(pool, PACK_SIZE)
            sa_in_pack = sum(1 for c in pack if sa_cache.get(c.id, False))

            chosen = player_select(pack, strategy, player_archetype, sa_cache,
                                   pick_number, resonance_sig, pool_info_current)
            drafted.append(chosen)
            pool_info = pool_info_current

            for i, sym in enumerate(chosen.visible_symbols):
                resonance_sig[sym] += 2.0 if i == 0 else 1.0

            pool = [c for c in pool if c.id != chosen.id]

            ai_picks_archs = []
            for ai in ais:
                if not pool:
                    break
                ai_pick = ai.pick_from_pool(pool)
                if ai_pick:
                    pool = [c for c in pool if c.id != ai_pick.id]
                    ai_picks_archs.append(ai_pick.archetype)

            history.append({
                "pick": pick_number, "round": round_num, "pack": pack,
                "chosen": chosen, "pool_size": len(pool),
                "sa_count": sa_in_pack, "ai_picks": ai_picks_archs,
                "open_lanes": list(open_lanes),
            })

        if round_num < ROUNDS:
            refill_total, open_mult = REFILL_SCHEDULE[round_num - 1]
            refill = build_refill_batch(refill_total, open_mult, open_lanes, ai_lanes)
            for c in refill:
                sa_cache[c.id] = sa_single(c, player_archetype, fitness_model)
            pool.extend(refill)

    return history, drafted, sa_cache, round_compositions, list(open_lanes)


# ============================================================
# Metrics
# ============================================================
def compute_metrics(all_results):
    m1_v, m2_v, m3_v, m4_v, m5_v = [], [], [], [], []
    m6_v, m9_v, m10_v, m11_v = [], [], [], []
    post_sa_all, consec_bad_all = [], []
    all_ids = []

    for history, drafted, sa_cache, _, _ in all_results:
        all_ids.append([c.id for c in drafted])

        # M1
        ea = [len(set(c.archetype for c in h["pack"] if sa_cache.get(c.id, False)))
              for h in history[:5]]
        m1_v.append(sum(ea) / max(1, len(ea)))

        # M2
        es = [h["sa_count"] for h in history[:5]]
        m2_v.append(sum(es) / max(1, len(es)))

        # M3: picks 6+
        ps = [h["sa_count"] for h in history[5:]]
        post_sa_all.extend(ps)
        if ps:
            m3_v.append(sum(ps) / len(ps))

        # M4
        po = [sum(1 for c in h["pack"] if not sa_cache.get(c.id, False))
              for h in history[5:]]
        if po:
            m4_v.append(sum(po) / len(po))

        # M5
        conv = TOTAL_PICKS
        for i in range(2, len(history)):
            w = [history[j]["sa_count"] for j in range(max(0, i - 2), i + 1)]
            if sum(w) / len(w) >= 1.5:
                conv = i + 1
                break
        m5_v.append(conv)

        # M6
        sa_d = sum(1 for c in drafted if sa_cache.get(c.id, False))
        m6_v.append(sa_d / max(1, len(drafted)))

        # M9
        if len(ps) > 1:
            mean = sum(ps) / len(ps)
            m9_v.append(math.sqrt(sum((x - mean) ** 2 for x in ps) / len(ps)))

        # M10
        mx = cur = 0
        for s in ps:
            if s < 1.5:
                cur += 1
                mx = max(mx, cur)
            else:
                cur = 0
        m10_v.append(mx)
        consec_bad_all.append(mx)

        # M11': picks 20+
        ls = [h["sa_count"] for h in history[19:]]
        if ls:
            m11_v.append(sum(ls) / len(ls))

    # M7
    m7 = []
    for i in range(1, len(all_ids)):
        p, c = set(all_ids[i - 1]), set(all_ids[i])
        m7.append(len(p & c) / max(1, len(p | c)))

    # M8
    af = Counter()
    for _, drafted, _, _, _ in all_results:
        ac = Counter(c.archetype for c in drafted if not c.is_generic)
        if ac:
            af[ac.most_common(1)[0][0]] += 1
    tr = max(1, len(all_results))
    m8_max = max(af.values()) / tr if af else 0
    m8_min = min(af.get(a, 0) for a in ARCHETYPE_NAMES) / tr

    # Pack percentiles
    pq = sorted(post_sa_all)
    n = len(pq)
    pcts = {p: pq[min(int(n * p / 100), n - 1)] if n > 0 else 0 for p in [10, 25, 50, 75, 90]}

    avg = lambda vs: sum(vs) / max(1, len(vs))
    return {
        "M1": avg(m1_v), "M2": avg(m2_v), "M3": avg(m3_v), "M4": avg(m4_v),
        "M5": avg(m5_v), "M6": avg(m6_v), "M7": avg(m7),
        "M8_max": m8_max, "M8_min": m8_min,
        "M9": avg(m9_v), "M10": avg(m10_v),
        "M10_max": max(m10_v) if m10_v else 0,
        "M11": avg(m11_v),
        "pack_pcts": pcts,
        "avg_consec_bad": avg(consec_bad_all),
        "worst_consec_bad": max(consec_bad_all) if consec_bad_all else 0,
    }


def run_aggregate(fm, strategy, n_drafts=NUM_DRAFTS, force_open=False):
    results = []
    for i in range(n_drafts):
        arch = ARCHETYPE_NAMES[i % 8]
        if force_open:
            while True:
                r = run_draft(arch, fm, strategy)
                if arch in r[4]:
                    break
            results.append(r)
        else:
            results.append(run_draft(arch, fm, strategy))
    return compute_metrics(results), results


def run_per_archetype(fm, strategy, n_per=125):
    results = {}
    for arch in ARCHETYPE_NAMES:
        hs = [run_draft(arch, fm, strategy) for _ in range(n_per)]
        results[arch] = compute_metrics(hs)
    return results


def collect_round_compositions(results):
    by_round = defaultdict(lambda: {"ps": [], "ot": [], "at": [], "sa": []})
    for _, _, _, rcs, _ in results:
        for c in rcs:
            r = c["round"]
            by_round[r]["ps"].append(c["pool_size"])
            by_round[r]["ot"].append(c["open_lane_total"])
            by_round[r]["at"].append(c["ai_lane_total"])
            by_round[r]["sa"].append(c["sa_count"])
    avg = lambda v: sum(v) / max(1, len(v))
    return {r: {
        "pool": avg(d["ps"]), "open": avg(d["ot"]), "ai": avg(d["at"]),
        "sa": avg(d["sa"]),
        "op_lane": avg(d["ot"]) / NUM_OPEN_LANES,
        "ai_lane": avg(d["at"]) / NUM_AIS,
        "grad": (avg(d["ot"]) / NUM_OPEN_LANES) / max(0.01, avg(d["at"]) / NUM_AIS),
    } for r, d in sorted(by_round.items())}


def sa_trajectory(results):
    by_pick = defaultdict(list)
    for h, _, _, _, _ in results:
        for e in h:
            by_pick[e["pick"]].append(e["sa_count"])
    return {p: sum(v) / len(v) for p, v in sorted(by_pick.items())}


def format_trace(result, player_arch):
    history, drafted, sa_cache, rcs, open_lanes = result
    lines = [f"=== Draft: {player_arch} (open: {', '.join(sorted(open_lanes))}) ==="]
    cr = 0
    for h in history:
        if h["round"] != cr:
            cr = h["round"]
            for c in rcs:
                if c["round"] == cr:
                    lines.append(f"\n  --- R{cr}: pool={c['pool_size']}, "
                                 f"open={c['open_lane_total']}, ai={c['ai_lane_total']}, "
                                 f"SA={c['sa_count']} ---")
                    break
        c = h["chosen"]
        sa = "S/A" if sa_cache.get(c.id, False) else "C/F"
        sym = "/".join(c.visible_symbols) or "Generic"
        ai = ", ".join(h["ai_picks"][:3])
        lines.append(f"  P{h['pick']:2d}: pool={h['pool_size']:3d} pkSA={h['sa_count']} "
                     f"[{c.archetype}:{sym}]({sa}) AI:[{ai}]")
    sd = sum(1 for c in drafted if sa_cache.get(c.id, False))
    lines.append(f"\n  Final: {sd}/{len(drafted)} SA = {sd / max(1, len(drafted)) * 100:.0f}%")
    return "\n".join(lines)


def consec_bad_dist(results):
    dist = Counter()
    for h, _, sa, _, _ in results:
        ps = [e["sa_count"] for e in h[5:]]
        mx = cur = 0
        for s in ps:
            if s < 1.5:
                cur += 1
                mx = max(mx, cur)
            else:
                cur = 0
        dist[mx] += 1
    return dist


# ============================================================
# Main
# ============================================================
def main():
    random.seed(42)
    fm = FITNESS_GRADUATED

    print("=" * 72)
    print("SIM-3: HYBRID A — Graduated Bias + Declining Volume")
    print("V11 Round 4 — Primary 'Standard' Candidate")
    print("=" * 72)

    # 1. All strategies
    print("\n" + "=" * 72)
    print("ALL STRATEGIES — Graduated Realistic (1000 drafts)")
    print("=" * 72)

    res, hist = {}, {}
    for s in ["committed", "signal", "power"]:
        random.seed(42)
        res[s], hist[s] = run_aggregate(fm, s)
        m = res[s]
        print(f"\n  {s}:")
        print(f"    M1={m['M1']:.2f}  M2={m['M2']:.2f}  M3={m['M3']:.2f}  "
              f"M4={m['M4']:.2f}  M5={m['M5']:.1f}")
        print(f"    M6={m['M6']:.2f}  M7={m['M7']:.4f}  M9={m['M9']:.2f}  "
              f"M10={m['M10']:.1f}  M11'={m['M11']:.2f}")
        pq = m['pack_pcts']
        print(f"    Pack (P10/25/50/75/90): {pq[10]}/{pq[25]}/{pq[50]}/{pq[75]}/{pq[90]}")

    m12 = res["signal"]["M3"] - res["committed"]["M3"]
    print(f"\n  M12 = {m12:.3f}")

    # 2. Forced open-lane committed (diagnostic)
    print("\n" + "=" * 72)
    print("COMMITTED FORCED OPEN LANE (500 drafts)")
    print("=" * 72)
    random.seed(42)
    res_fo, hist_fo = run_aggregate(fm, "committed", 500, force_open=True)
    m = res_fo
    print(f"  M3={m['M3']:.2f}  M5={m['M5']:.1f}  M6={m['M6']:.2f}  "
          f"M10={m['M10']:.1f}  M11'={m['M11']:.2f}")

    # 3. Per-archetype
    print("\n" + "=" * 72)
    print("PER-ARCHETYPE (committed, 125 drafts each)")
    print("=" * 72)
    random.seed(42)
    pa = run_per_archetype(fm, "committed")
    print(f"  {'Archetype':<16} {'M3':>6} {'M5':>6} {'M6':>6} {'M9':>6} {'M10':>6} {'M11':>6}")
    m3s = []
    for a in ARCHETYPE_NAMES:
        m = pa[a]
        m3s.append(m['M3'])
        print(f"  {a:<16} {m['M3']:6.2f} {m['M5']:6.1f} {m['M6']:6.2f} "
              f"{m['M9']:6.2f} {m['M10']:6.1f} {m['M11']:6.2f}")
    print(f"\n  Spread: {max(m3s) - min(m3s):.3f}")
    print(f"  Worst: {ARCHETYPE_NAMES[m3s.index(min(m3s))]}: {min(m3s):.2f}")
    print(f"  Best:  {ARCHETYPE_NAMES[m3s.index(max(m3s))]}: {max(m3s):.2f}")

    # 4. Per-archetype forced open lane
    print("\n" + "=" * 72)
    print("PER-ARCHETYPE FORCED OPEN LANE (committed, 125 drafts each)")
    print("=" * 72)
    random.seed(42)
    pa_fo = {}
    for a in ARCHETYPE_NAMES:
        hs = []
        for _ in range(125):
            while True:
                r = run_draft(a, fm, "committed")
                if a in r[4]:
                    break
            hs.append(r)
        pa_fo[a] = compute_metrics(hs)
    print(f"  {'Archetype':<16} {'M3':>6} {'M5':>6} {'M6':>6} {'M10':>6} {'M11':>6}")
    m3s_fo = []
    for a in ARCHETYPE_NAMES:
        m = pa_fo[a]
        m3s_fo.append(m['M3'])
        print(f"  {a:<16} {m['M3']:6.2f} {m['M5']:6.1f} {m['M6']:6.2f} "
              f"{m['M10']:6.1f} {m['M11']:6.2f}")
    print(f"\n  Forced-open spread: {max(m3s_fo) - min(m3s_fo):.3f}")

    # 5. Round compositions
    print("\n" + "=" * 72)
    print("ROUND-BY-ROUND POOL (committed, 1000 drafts avg)")
    print("=" * 72)
    rc = collect_round_compositions(hist["committed"])
    print(f"  {'R':>2} {'Pool':>5} {'Open':>6} {'AI':>6} {'O/L':>6} {'A/L':>6} {'Grad':>6} {'SA':>5}")
    for r, d in sorted(rc.items()):
        print(f"  {r:>2} {d['pool']:5.0f} {d['open']:6.1f} {d['ai']:6.1f} "
              f"{d['op_lane']:6.1f} {d['ai_lane']:6.1f} {d['grad']:5.2f}x {d['sa']:5.1f}")

    # 6. S/A trajectory
    print("\n" + "=" * 72)
    print("S/A TRAJECTORY (committed)")
    print("=" * 72)
    t = sa_trajectory(hist["committed"])
    for p in sorted(t.keys()):
        bar = "#" * int(t[p] * 10)
        mk = ""
        if p == 11:
            mk = " <-- R2 (+70, 1.4x)"
        elif p == 21:
            mk = " <-- R3 (+48, 2.0x)"
        print(f"  P{p:2d}: {t[p]:5.2f} {bar}{mk}")

    # Same for forced open lane
    print("\n  S/A trajectory (forced open lane, 500 drafts):")
    t_fo = sa_trajectory(hist_fo)
    for p in sorted(t_fo.keys()):
        bar = "#" * int(t_fo[p] * 10)
        print(f"  P{p:2d}: {t_fo[p]:5.2f} {bar}")

    # 7. Pack quality
    print("\n" + "=" * 72)
    print("PACK QUALITY (picks 6+, committed)")
    print("=" * 72)
    mc = res["committed"]
    pq = mc["pack_pcts"]
    print(f"  P10={pq[10]}  P25={pq[25]}  P50={pq[50]}  P75={pq[75]}  P90={pq[90]}")
    print(f"  Avg consec bad: {mc['avg_consec_bad']:.2f}")
    print(f"  Worst consec bad: {mc['worst_consec_bad']}")

    cd = consec_bad_dist(hist["committed"])
    td = sum(cd.values())
    print(f"\n  Distribution:")
    for k in sorted(cd.keys())[:10]:
        print(f"    {k}: {cd[k]} ({cd[k] / td * 100:.1f}%)")

    # Forced open lane pack quality
    print(f"\n  Forced open lane:")
    pq2 = res_fo["pack_pcts"]
    print(f"  P10={pq2[10]}  P25={pq2[25]}  P50={pq2[50]}  P75={pq2[75]}  P90={pq2[90]}")
    print(f"  Avg consec bad: {res_fo['avg_consec_bad']:.2f}  Worst: {res_fo['worst_consec_bad']}")

    # 8. Draft traces
    print("\n" + "=" * 72)
    print("DRAFT TRACES")
    print("=" * 72)

    random.seed(100)
    t1 = run_draft("Warriors", fm, "committed")
    print("\n" + format_trace(t1, "Warriors"))

    random.seed(200)
    t2 = run_draft("Storm", fm, "signal")
    print("\n" + format_trace(t2, "Storm"))

    # 9. Scorecard
    print("\n" + "=" * 72)
    print("SCORECARD — SIM-3 Hybrid A")
    print("=" * 72)

    mc = res["committed"]
    checks = [
        ("M1  (early variety >= 3.0)",      mc['M1'],     3.0,  ">="),
        ("M2  (early S/A <= 2.0)",          mc['M2'],     2.0,  "<="),
        ("M3  (picks 6+, S/A >= 2.0)",      mc['M3'],     2.0,  ">="),
        ("M4  (off-arch >= 0.5)",           mc['M4'],     0.5,  ">="),
        ("M5  (convergence 5-8)",           mc['M5'],     8.0,  "<="),
        ("M6  (concentration 60-90%)",      mc['M6'],     0.60, ">="),
        ("M7  (variety < 40%)",             mc['M7'],     0.40, "<="),
        ("M8  (no arch > 20%)",             mc['M8_max'], 0.20, "<="),
        ("M9  (stddev >= 0.8)",             mc['M9'],     0.8,  ">="),
        ("M10 (consec bad <= 2)",           mc['M10'],    2.0,  "<="),
        ("M11'(picks 20+, SA >= 2.5)",      mc['M11'],    2.5,  ">="),
        ("M12 (signal-committed >= 0.3)",   m12,          0.3,  ">="),
    ]
    ok = 0
    for name, val, tgt, d in checks:
        p = val >= tgt if d == ">=" else val <= tgt
        if p:
            ok += 1
        print(f"  {name:<40} {val:7.2f}  {'PASS' if p else 'FAIL'}")
    print(f"\n  Passes: {ok}/{len(checks)}")

    # Also show forced open lane scorecard
    mf = res_fo
    m12_fo = 0.0  # Not computed for forced
    print(f"\n  FORCED OPEN LANE scorecard:")
    checks_fo = [
        ("M3  (picks 6+, S/A >= 2.0)",      mf['M3'],     2.0,  ">="),
        ("M5  (convergence 5-8)",           mf['M5'],     8.0,  "<="),
        ("M6  (concentration 60-90%)",      mf['M6'],     0.60, ">="),
        ("M10 (consec bad <= 2)",           mf['M10'],    2.0,  "<="),
        ("M11'(picks 20+, SA >= 2.5)",      mf['M11'],    2.5,  ">="),
    ]
    for name, val, tgt, d in checks_fo:
        p = val >= tgt if d == ">=" else val <= tgt
        print(f"  {name:<40} {val:7.2f}  {'PASS' if p else 'FAIL'}")

    # 10. Strategy comparison
    print(f"\n  {'Strategy':<16} {'M3':>6} {'M6':>6} {'M11':>6}")
    for s in ["committed", "signal", "power"]:
        ms = res[s]
        print(f"  {s:<16} {ms['M3']:6.2f} {ms['M6']:6.2f} {ms['M11']:6.2f}")
    print(f"  {'committed(open)':<16} {res_fo['M3']:6.2f} {res_fo['M6']:6.2f} {res_fo['M11']:6.2f}")

    # 11. Comparison
    print("\n" + "=" * 72)
    print("COMPARISON")
    print("=" * 72)
    print(f"  {'Algorithm':<48} {'M3':>6} {'M10':>5} {'M11':>6} {'M6':>6}")
    print(f"  {'V9 Hybrid B':<48} {'2.70':>6} {'3.8':>5} {'3.25':>6} {'0.86':>6}")
    print(f"  {'V10 Hybrid X':<48} {'0.84':>6} {'--':>5} {'0.69':>6} {'--':>6}")
    print(f"  {'SIM-3 committed (all lanes)':<48} {mc['M3']:6.2f} {mc['M10']:5.1f} {mc['M11']:6.2f} {mc['M6']:6.2f}")
    print(f"  {'SIM-3 committed (open lanes only)':<48} {mf['M3']:6.2f} {mf['M10']:5.1f} {mf['M11']:6.2f} {mf['M6']:6.2f}")
    ms = res["signal"]
    print(f"  {'SIM-3 signal-reader':<48} {ms['M3']:6.2f} {ms['M10']:5.1f} {ms['M11']:6.2f} {ms['M6']:6.2f}")

    print("\nDone.")


if __name__ == "__main__":
    main()
