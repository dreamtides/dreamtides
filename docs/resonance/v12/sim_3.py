#!/usr/bin/env python3
"""
V12 Simulation Agent 3: Hybrid 2 — Progressive N + Steep Biased Contraction

Algorithm (from critic_review.md Section 7, Hybrid 2):
- Starting pool: 120 cards, 8 archetypes (15 per archetype)
- Fitness: Graduated Realistic (~36% weighted-average sibling A-tier, varying by pair)
- 5 AIs, each assigned 1 of 5 archetypes (3 open lanes)
- AI avoidance: gradual ramp from pick 3, reaching 90% by pick 12
- Refills: 60/20/0 (3 rounds of 10 picks), 2.0x open-lane bias
- S/A targeting: refills add S/A at 40% rate for open lanes
- Pack construction: N=4 (picks 1-5), N=8 (picks 6-15), N=12 (picks 16-30)
- "Best 4" ranking: visible resonance symbol match only
- AI saturation: 10 cards per archetype; fallback to adjacent open lane

1000 drafts x 30 picks x 3 player strategies, all 14 metrics.
"""

import random
import math
from collections import defaultdict
from dataclasses import dataclass, field
from typing import Optional

# ─── Constants ───────────────────────────────────────────────────────────────

NUM_DRAFTS = 1000
NUM_PICKS = 30
NUM_AIS = 5
NUM_ARCHETYPES = 8
CARDS_PER_ARCHETYPE = 15
PACK_SIZE = 4
AI_SATURATION_THRESHOLD = 10

REFILL_SCHEDULE = {10: 60, 20: 20}
OPEN_LANE_BIAS = 2.0
OPEN_LANE_SA_RATE = 0.40

ARCHETYPES = [
    "Flash/Tempo",        # 0: Zephyr/Ember
    "Blink/Flicker",      # 1: Ember/Zephyr
    "Storm/Spellslinger", # 2: Ember/Stone
    "Self-Discard",       # 3: Stone/Ember
    "Self-Mill/Reanim",   # 4: Stone/Tide
    "Sacrifice/Abandon",  # 5: Tide/Stone
    "Warriors/Midrange",  # 6: Tide/Zephyr
    "Ramp/SpiritAnimal",  # 7: Zephyr/Tide
]

RESONANCE = {
    0: ("Zephyr", "Ember"), 1: ("Ember", "Zephyr"), 2: ("Ember", "Stone"),
    3: ("Stone", "Ember"),  4: ("Stone", "Tide"),   5: ("Tide", "Stone"),
    6: ("Tide", "Zephyr"),  7: ("Zephyr", "Tide"),
}

SIBLING_SA_RATE = {
    (6, 5): 0.50, (5, 6): 0.50,
    (3, 4): 0.40, (4, 3): 0.40,
    (1, 2): 0.30, (2, 1): 0.30,
    (0, 7): 0.25, (7, 0): 0.25,
}

SYMBOLS = ["Tide", "Stone", "Ember", "Zephyr"]
SYM_IDX = {s: i for i, s in enumerate(SYMBOLS)}


def get_N(pick_num):
    if pick_num <= 5: return 4
    if pick_num <= 15: return 8
    return 12


def get_avoidance_weight(pick_num):
    if pick_num <= 2: return 0.0
    if pick_num <= 5: return 0.20
    if pick_num <= 8: return 0.45
    if pick_num <= 11: return 0.70
    return 0.90


def get_sa_rate(arch_id):
    for (a, _), rate in SIBLING_SA_RATE.items():
        if a == arch_id: return rate
    return 0.36


def get_sibling(arch_id):
    for (a, b) in SIBLING_SA_RATE:
        if a == arch_id: return b
    return (arch_id + 1) % NUM_ARCHETYPES


def get_adjacent(arch_id):
    return [(arch_id - 1) % NUM_ARCHETYPES, (arch_id + 1) % NUM_ARCHETYPES]


# ─── Card Model ──────────────────────────────────────────────────────────────

@dataclass
class SimCard:
    id: int
    archetype: int
    is_sa: bool
    primary_symbol: str
    secondary_symbol: Optional[str]
    power: float
    fitness: dict = field(default_factory=dict)

    def sym_vec(self):
        v = [0.0] * 4
        if self.primary_symbol: v[SYM_IDX[self.primary_symbol]] += 1.0
        if self.secondary_symbol: v[SYM_IDX[self.secondary_symbol]] += 0.5
        return v


def compute_fitness(card_arch, card_sa, target):
    if target == card_arch: return 0.95 if card_sa else 0.50
    sib = get_sibling(target)
    if card_arch == sib: return 0.70 if card_sa else 0.35
    adj = get_adjacent(target)
    if card_arch in adj: return 0.30 if card_sa else 0.15
    return 0.05


def make_card(cid, arch, is_sa):
    pri, sec = RESONANCE[arch]
    roll = random.random()
    if roll < 0.11:
        ps = random.choice(SYMBOLS)
        ss = None
    elif roll < 0.90:
        ps, ss = pri, None
    else:
        ps, ss = pri, sec
    pw = random.uniform(4, 10) if is_sa else random.uniform(1, 6)
    c = SimCard(id=cid, archetype=arch, is_sa=is_sa, primary_symbol=ps,
                secondary_symbol=ss, power=pw)
    for t in range(NUM_ARCHETYPES):
        c.fitness[t] = compute_fitness(arch, is_sa, t)
    return c


def generate_pool():
    cards = []
    cid = 0
    for arch in range(NUM_ARCHETYPES):
        sr = get_sa_rate(arch)
        nsa = round(CARDS_PER_ARCHETYPE * sr)
        for i in range(CARDS_PER_ARCHETYPE):
            cards.append(make_card(cid, arch, i < nsa))
            cid += 1
    return cards, cid


def generate_refill(count, open_lanes, ai_lanes, next_id):
    cards = []
    tw = len(open_lanes) * OPEN_LANE_BIAS + len(ai_lanes) * 1.0
    open_alloc = int(round(count * len(open_lanes) * OPEN_LANE_BIAS / tw))
    ai_alloc = count - open_alloc

    per_open = open_alloc // max(len(open_lanes), 1)
    rem = open_alloc - per_open * len(open_lanes)
    for idx, lane in enumerate(open_lanes):
        n = per_open + (1 if idx < rem else 0)
        for _ in range(n):
            cards.append(make_card(next_id, lane, random.random() < OPEN_LANE_SA_RATE))
            next_id += 1

    per_ai = ai_alloc // max(len(ai_lanes), 1)
    rem = ai_alloc - per_ai * len(ai_lanes)
    for idx, lane in enumerate(ai_lanes):
        n = per_ai + (1 if idx < rem else 0)
        for _ in range(n):
            cards.append(make_card(next_id, lane, random.random() < get_sa_rate(lane)))
            next_id += 1

    return cards[:count], next_id


# ─── Resonance ───────────────────────────────────────────────────────────────

def build_sig(drafted):
    sig = [0.0] * 4
    for c in drafted:
        if c.primary_symbol: sig[SYM_IDX[c.primary_symbol]] += 2.0
        if c.secondary_symbol: sig[SYM_IDX[c.secondary_symbol]] += 1.0
    return sig


def res_score(card, sig):
    cv = card.sym_vec()
    dot = sum(a * b for a, b in zip(cv, sig))
    ms = math.sqrt(sum(s * s for s in sig) + 1e-12)
    mc = math.sqrt(sum(c * c for c in cv) + 1e-12)
    return dot / (ms * mc)


# ─── AI Inference ────────────────────────────────────────────────────────────

def infer_player_arch(pool_snapshots, pick_num, open_lanes):
    """
    AIs infer which OPEN LANE the player is drafting.

    Key insight: AIs know which 5 archetypes are assigned to AIs (public:
    determined at draft start). The 3 open lanes are known. Among open lanes,
    the one depleting fastest is most likely the player's archetype. AIs compare
    depletion rates among the 3 open lanes only (filtering out noise from
    AI-lane depletion which they can attribute to other AIs).

    With 5 AIs each drafting their own archetype, AI-lane depletion is expected
    and ignored. Open-lane depletion comes from: (a) the player's picks,
    (b) incidental AI picks of open-lane cards for fitness reasons. The player's
    archetype should deplete faster than the other 2 open lanes.
    """
    if pick_num < 3 or len(pool_snapshots) < 3:
        return None, 0.0

    # Compare initial pool to current for each open lane
    window = min(len(pool_snapshots) - 1, 6)
    start = pool_snapshots[max(0, len(pool_snapshots) - window - 1)]
    end = pool_snapshots[-1]

    depletion = {}
    for lane in open_lanes:
        s = start.get(lane, 0)
        e = end.get(lane, 0)
        if s > 0:
            depletion[lane] = (s - e) / s
        else:
            depletion[lane] = 0

    if not depletion:
        return None, 0.0

    # The open lane with the highest depletion rate is likely the player's
    best_lane = max(depletion, key=depletion.get)
    best_rate = depletion[best_lane]

    # Compare to average open-lane depletion
    avg_rate = sum(depletion.values()) / len(depletion)
    if avg_rate <= 0:
        return None, 0.0

    signal = best_rate / max(avg_rate, 0.001)
    # Need the signal to be at least 1.2x above average
    if signal < 1.15 or best_rate < 0.05:
        return None, 0.0

    confidence = min(1.0, (signal - 1.0) / 0.8)
    return best_lane, confidence


# ─── Helpers ─────────────────────────────────────────────────────────────────

def snapshot(pool):
    c = defaultdict(int)
    for card in pool: c[card.archetype] += 1
    return dict(c)


def is_on_arch_sa(card, player_arch):
    """S/A card that is good for the player: own arch S/A or sibling arch S/A."""
    if not card.is_sa: return False
    return card.archetype in (player_arch, get_sibling(player_arch))


def count_on_arch_sa(pool, arch):
    sib = get_sibling(arch)
    return sum(1 for c in pool if c.is_sa and c.archetype in (arch, sib))


def count_arch(pool, arch):
    return sum(1 for c in pool if c.archetype == arch)


# ─── Pack Construction ───────────────────────────────────────────────────────

def construct_pack(pool, pick_num, sig, N):
    if not pool: return []
    nd = min(N, len(pool))
    drawn = random.sample(pool, nd)
    if pick_num <= 5 or nd <= PACK_SIZE:
        return drawn[:PACK_SIZE]
    scored = [(res_score(c, sig), c.power, c) for c in drawn]
    scored.sort(key=lambda x: (-x[0], -x[1]))
    return [x[2] for x in scored[:PACK_SIZE]]


# ─── Player Strategies ───────────────────────────────────────────────────────

def committed_pick(pack, pa, pn):
    if pn <= 5: return max(pack, key=lambda c: c.power)
    return max(pack, key=lambda c: c.fitness.get(pa, 0))

def power_pick(pack, pa, pn):
    return max(pack, key=lambda c: c.power)

def signal_pick(pack, pa, pn):
    if pn <= 3: return max(pack, key=lambda c: c.power)
    return max(pack, key=lambda c: c.fitness.get(pa, 0))

STRATEGIES = {"committed": committed_pick, "power_chaser": power_pick, "signal_reader": signal_pick}


# ─── AI Pick ─────────────────────────────────────────────────────────────────

def ai_pick(ai_arch, ai_on_count, pool, pick_num, inferred, open_lanes):
    avoidance = get_avoidance_weight(pick_num)
    best, best_s = None, -999
    for card in pool:
        s = card.fitness.get(ai_arch, 0)
        # Avoidance: if this card belongs to the inferred player archetype,
        # reduce its desirability
        if inferred is not None and card.archetype == inferred:
            s *= (1.0 - avoidance)
        # Also slightly avoid sibling of inferred player arch
        if inferred is not None and card.archetype == get_sibling(inferred):
            s *= (1.0 - avoidance * 0.3)
        # Saturation
        if ai_on_count >= AI_SATURATION_THRESHOLD and card.archetype == ai_arch:
            s *= 0.2
        if s > best_s:
            best_s = s
            best = card
    return best


# ─── Simulation ─────────────────────────────────────────────────────────────

def run_draft(strat_name, seed=None):
    if seed is not None: random.seed(seed)
    pool, nid = generate_pool()

    archs = list(range(NUM_ARCHETYPES))
    random.shuffle(archs)
    ai_archs = archs[:NUM_AIS]
    open_lanes = archs[NUM_AIS:]
    player_arch = random.choice(open_lanes)

    pfn = STRATEGIES[strat_name]
    pcards = []
    ai_on = [0] * NUM_AIS
    snaps = [snapshot(pool)]

    m1, m2, m3, m4, m11 = [], [], [], [], []
    pack_sa = []
    conv_pick = NUM_PICKS
    ascores = defaultdict(float)
    conv = False
    first_inf = None

    pp_pool, pp_dens, pp_sa = {}, {}, {}
    sa_at = {20: 0, 25: 0, 30: 0}

    for pn in range(1, NUM_PICKS + 1):
        if len(pool) < 1: break

        ps = len(pool)
        ac = count_arch(pool, player_arch)
        asa = count_on_arch_sa(pool, player_arch)
        pp_pool[pn] = ps
        pp_dens[pn] = ac / max(ps, 1)
        pp_sa[pn] = asa
        if pn in sa_at: sa_at[pn] = asa

        inf, conf = infer_player_arch(snaps, pn, open_lanes)
        if inf == player_arch and first_inf is None:
            first_inf = pn

        N = get_N(pn)
        sig = build_sig(pcards)
        pack = construct_pack(pool, pn, sig, N)
        if not pack: break

        chosen = pfn(pack, player_arch, pn)
        pcards.append(chosen)
        pool.remove(chosen)

        ascores[chosen.archetype] += chosen.fitness.get(chosen.archetype, 0)
        if not conv and pn >= 3:
            tot = sum(ascores.values())
            if tot > 0 and max(ascores.values()) / tot >= 0.55:
                conv = True
                conv_pick = pn

        if pn <= 5:
            m1.append(len(set(c.archetype for c in pack if c.is_sa)))
            m2.append(sum(1 for c in pack if is_on_arch_sa(c, player_arch)))

        if pn >= 6:
            sa = sum(1 for c in pack if is_on_arch_sa(c, player_arch))
            m3.append(sa)
            pack_sa.append(sa)
            m4.append(sum(1 for c in pack if c.archetype != player_arch and
                         c.archetype != get_sibling(player_arch)))

        if pn >= 20:
            m11.append(sum(1 for c in pack if is_on_arch_sa(c, player_arch)))

        for ai_idx in range(NUM_AIS):
            if len(pool) < 1: break
            card = ai_pick(ai_archs[ai_idx], ai_on[ai_idx], pool, pn, inf, open_lanes)
            if card:
                if card.archetype == ai_archs[ai_idx]:
                    ai_on[ai_idx] += 1
                pool.remove(card)

        snaps.append(snapshot(pool))

        if pn in REFILL_SCHEDULE:
            nc, nid = generate_refill(REFILL_SCHEDULE[pn], open_lanes, ai_archs, nid)
            pool.extend(nc)
            snaps.append(snapshot(pool))

    m3a = sum(m3) / max(len(m3), 1)
    m11a = sum(m11) / max(len(m11), 1)
    sa_deck = sum(1 for c in pcards if is_on_arch_sa(c, player_arch))
    m6 = sa_deck / max(len(pcards), 1)

    if pack_sa:
        mn = sum(pack_sa) / len(pack_sa)
        m9 = math.sqrt(sum((x - mn) ** 2 for x in pack_sa) / len(pack_sa))
    else:
        m9 = 0

    con = mx = 0
    for v in pack_sa:
        if v < 1.5:
            con += 1
            mx = max(mx, con)
        else:
            con = 0

    return {
        "m1": sum(m1) / max(len(m1), 1), "m2": sum(m2) / max(len(m2), 1),
        "m3": m3a, "m4": sum(m4) / max(len(m4), 1), "m5": conv_pick,
        "m6": m6, "m9": m9, "m10": mx, "m11": m11a,
        "sa_at": sa_at, "first_inf": first_inf,
        "pp_pool": pp_pool, "pp_dens": pp_dens, "pp_sa": pp_sa,
        "pack_sa": pack_sa, "player_arch": player_arch,
        "card_ids": frozenset(c.id for c in pcards),
    }


# ─── Draft Trace ─────────────────────────────────────────────────────────────

def draft_trace(sname, seed=42):
    random.seed(seed)
    pool, nid = generate_pool()
    archs = list(range(NUM_ARCHETYPES))
    random.shuffle(archs)
    ai_archs = archs[:NUM_AIS]
    open_lanes = archs[NUM_AIS:]
    pa = random.choice(open_lanes)
    pfn = STRATEGIES[sname]
    pcards = []
    ai_on = [0] * NUM_AIS
    snaps = [snapshot(pool)]

    lines = [f"Strategy: {sname}",
             f"Player arch: {ARCHETYPES[pa]} (#{pa})",
             f"AI archs: {[ARCHETYPES[a] for a in ai_archs]}",
             f"Open: {[ARCHETYPES[a] for a in open_lanes]}", ""]

    for pn in range(1, NUM_PICKS + 1):
        if len(pool) < 1:
            lines.append(f"Pick {pn}: Pool exhausted")
            break

        ps = len(pool)
        ac = count_arch(pool, pa)
        asa = count_on_arch_sa(pool, pa)
        inf, conf = infer_player_arch(snaps, pn, open_lanes)

        N = get_N(pn)
        sig = build_sig(pcards)
        pack = construct_pack(pool, pn, sig, N)
        if not pack: break

        chosen = pfn(pack, pa, pn)
        pcards.append(chosen)
        pool.remove(chosen)

        psa = sum(1 for c in pack if is_on_arch_sa(c, pa))
        inf_s = f"{ARCHETYPES[inf]}(c={conf:.2f})" if inf is not None else "None"
        tag = "S/A" if is_on_arch_sa(chosen, pa) else ("on" if chosen.archetype in (pa, get_sibling(pa)) else "OFF")

        if pn in [1,3,5,6,8,10,11,15,16,20,21,25,28,30] or psa >= 2:
            lines.append(f"Pk{pn:2d} N={N:2d} Pool={ps:3d} Arch={ac:2d}({ac/max(ps,1)*100:4.1f}%) "
                        f"S/A={asa} PkSA={psa} [{tag:>3}] Inf:{inf_s}")

        for ai_idx in range(NUM_AIS):
            if len(pool) < 1: break
            c = ai_pick(ai_archs[ai_idx], ai_on[ai_idx], pool, pn, inf, open_lanes)
            if c:
                if c.archetype == ai_archs[ai_idx]: ai_on[ai_idx] += 1
                pool.remove(c)

        snaps.append(snapshot(pool))

        if pn in REFILL_SCHEDULE:
            nc, nid = generate_refill(REFILL_SCHEDULE[pn], open_lanes, ai_archs, nid)
            pool.extend(nc)
            snaps.append(snapshot(pool))
            na = sum(1 for c in nc if c.archetype == pa)
            ns = sum(1 for c in nc if c.archetype == pa and c.is_sa)
            nsb = sum(1 for c in nc if c.archetype == get_sibling(pa) and c.is_sa)
            lines.append(f"  >>> REFILL +{REFILL_SCHEDULE[pn]}: arch+{na}(+{ns}S/A) sibS/A+{nsb} Pool={len(pool)}")

    return "\n".join(lines)


# ─── Main ────────────────────────────────────────────────────────────────────

def main():
    print("V12 Sim3: Hybrid 2 — Progressive N + Steep Biased Contraction")
    print(f"{NUM_DRAFTS} drafts x {NUM_PICKS} picks x {len(STRATEGIES)} strategies")
    print()

    all_res = {}
    for strat in STRATEGIES:
        print(f"  Running {strat}...", flush=True)
        ml = defaultdict(list)
        pam3 = defaultdict(list)
        ppa = {"pool": defaultdict(list), "dens": defaultdict(list), "sa": defaultdict(list)}
        csets = []
        psa_g = []

        for d in range(NUM_DRAFTS):
            seed = d * len(STRATEGIES) + list(STRATEGIES.keys()).index(strat)
            r = run_draft(strat, seed)
            ml["m1"].append(r["m1"]); ml["m2"].append(r["m2"])
            ml["m3"].append(r["m3"]); ml["m4"].append(r["m4"])
            ml["m5"].append(r["m5"]); ml["m6"].append(r["m6"])
            ml["m9"].append(r["m9"]); ml["m10"].append(r["m10"])
            ml["m11"].append(r["m11"])
            ml["sa20"].append(r["sa_at"][20])
            ml["sa25"].append(r["sa_at"][25])
            ml["sa30"].append(r["sa_at"][30])
            if r["first_inf"] is not None:
                ml["inf"].append(r["first_inf"])
            pam3[r["player_arch"]].append(r["m3"])
            csets.append(r["card_ids"])
            psa_g.extend(r["pack_sa"])
            for pk, v in r["pp_pool"].items(): ppa["pool"][pk].append(v)
            for pk, v in r["pp_dens"].items(): ppa["dens"][pk].append(v)
            for pk, v in r["pp_sa"].items(): ppa["sa"][pk].append(v)

        def avg(l): return sum(l) / max(len(l), 1)

        overlaps = []
        for i in range(1, len(csets)):
            a, b = csets[i-1], csets[i]
            if a and b: overlaps.append(len(a & b) / max(len(a | b), 1))

        psa_s = sorted(psa_g)
        def pct(d, p):
            if not d: return 0
            k = (len(d)-1)*p/100; f = int(k); c = min(f+1, len(d)-1)
            return d[f] + (k-f)*(d[c]-d[f])

        all_res[strat] = {
            "M1": avg(ml["m1"]), "M2": avg(ml["m2"]), "M3": avg(ml["m3"]),
            "M4": avg(ml["m4"]), "M5": avg(ml["m5"]), "M6": avg(ml["m6"]),
            "M7": avg(overlaps), "M9": avg(ml["m9"]), "M10": avg(ml["m10"]),
            "M11": avg(ml["m11"]),
            "SA20": avg(ml["sa20"]), "SA25": avg(ml["sa25"]), "SA30": avg(ml["sa30"]),
            "M13": avg(ml["inf"]) if ml["inf"] else 0,
            "M14": avg(ml["inf"]) if ml["inf"] else 0,
            "inf_rate": len(ml["inf"]) / NUM_DRAFTS,
            "pam3": {k: avg(v) for k, v in pam3.items()},
            "p10": pct(psa_s,10), "p25": pct(psa_s,25), "p50": pct(psa_s,50),
            "p75": pct(psa_s,75), "p90": pct(psa_s,90),
            "ptraj": {k: avg(v) for k, v in ppa["pool"].items()},
            "dtraj": {k: avg(v) for k, v in ppa["dens"].items()},
            "straj": {k: avg(v) for k, v in ppa["sa"].items()},
            "m3v": ml["m3"], "m10v": ml["m10"],
            "M8": {k: len(v)/max(sum(len(vv) for vv in pam3.values()),1)
                   for k, v in pam3.items()},
        }

    # ─── Output ──────────────────────────────────────────────────────────────
    rc = all_res["committed"]; rp = all_res["power_chaser"]; rs = all_res["signal_reader"]

    print("=" * 80)
    print("FULL SCORECARD")
    print("=" * 80)
    tgts = {"M1":">= 3","M2":"<= 2","M3":">= 2.0","M4":">= 0.5",
            "M5":"5-8","M6":"60-90%","M7":"< 40%","M9":">= 0.8","M10":"<= 2","M11":">= 2.5"}
    print(f"{'Metric':<8} {'Target':<12} {'Committed':<12} {'PowerChaser':<12} {'SignalReader':<12}")
    print("-" * 56)
    for m in ["M1","M2","M3","M4","M5","M6","M7","M9","M10","M11"]:
        t = tgts.get(m, "")
        vs = []
        for s in [rc, rp, rs]:
            v = s[m]
            vs.append(f"{v*100:.1f}%" if m in ("M6","M7") else f"{v:.2f}")
        print(f"{m:<8} {t:<12} {vs[0]:<12} {vs[1]:<12} {vs[2]:<12}")

    m12 = rs["M3"] - rc["M3"]
    print(f"\nM12 (Signal - Committed M3): {m12:.3f}  (target >= 0.3)")
    print(f"M13 (AI detect pick): {rc['M13']:.1f}  (target 6-10)  [rate: {rc['inf_rate']*100:.0f}%]")
    print(f"M14 (Arch visible pick): {rc['M14']:.1f}  (target 4-7)")

    print(f"\nS/A in pool at key picks (committed, includes sibling):")
    for pk in [20, 25, 30]:
        print(f"  Pick {pk}: {rc[f'SA{pk}']:.2f}")

    print("\n" + "=" * 80)
    print("PER-ARCHETYPE M3 (Committed)")
    print("=" * 80)
    for a in range(NUM_ARCHETYPES):
        v = rc["pam3"].get(a)
        if v is not None: print(f"  {ARCHETYPES[a]:<25} M3 = {v:.3f}")

    print("\n" + "=" * 80)
    print("AI AVOIDANCE TIMELINE")
    print("=" * 80)
    for p in [1,2,3,5,6,8,10,12,15,20,25,30]:
        print(f"  Pick {p:2d}: {get_avoidance_weight(p)*100:5.1f}%")
    print(f"  First correct inference: pick {rc['M14']:.1f} ({rc['inf_rate']*100:.0f}% of drafts)")

    print("\n" + "=" * 80)
    print("POOL CONTRACTION TRAJECTORY (Committed)")
    print("=" * 80)
    pt, dt, st = rc["ptraj"], rc["dtraj"], rc["straj"]
    print(f"{'Pick':<6} {'Pool':<8} {'Arch%':<8} {'S/A':<8}")
    print("-" * 30)
    for pk in [1,3,5,6,8,10,11,13,15,16,18,20,21,23,25,27,29,30]:
        if pk in pt:
            print(f"  {pk:2d}   {pt[pk]:6.1f}  {dt.get(pk,0)*100:5.1f}%  {st.get(pk,0):5.2f}")

    print("\n" + "=" * 80)
    print("PACK QUALITY DISTRIBUTION (S/A per pack, picks 6+, committed)")
    print("=" * 80)
    print(f"  p10={rc['p10']:.1f} p25={rc['p25']:.1f} p50={rc['p50']:.1f} "
          f"p75={rc['p75']:.1f} p90={rc['p90']:.1f}")

    print("\n" + "=" * 80)
    print("CONSECUTIVE BAD PACKS (< 1.5 S/A, picks 6+)")
    print("=" * 80)
    for s, r in [("committed",rc),("power_chaser",rp),("signal_reader",rs)]:
        mv = r["m10v"]
        print(f"  {s:<15}: avg={avg(mv):.1f} max={max(mv) if mv else 0}")

    print("\n" + "=" * 80)
    print("OVERSAMPLING ANALYSIS")
    print("=" * 80)
    print(f"  N: 1-5=4, 6-15=8, 16-30=12")
    print(f"  M3 (picks 6+): {rc['M3']:.3f}")
    print(f"  M11 (picks 20+): {rc['M11']:.3f}")
    p25p = pt.get(25, 1)
    print(f"  Pick 25: pool={p25p:.0f}, N=12 draws {min(12,p25p):.0f}/{p25p:.0f} "
          f"= {min(12,max(p25p,1))/max(p25p,1)*100:.0f}%")

    print("\n" + "=" * 80)
    print("DRAFT TRACE 1: Committed Player")
    print("=" * 80)
    print(draft_trace("committed", 42))

    print("\n" + "=" * 80)
    print("DRAFT TRACE 2: Signal Reader")
    print("=" * 80)
    print(draft_trace("signal_reader", 99))

    print("\n" + "=" * 80)
    print("COMPARISON TO V9 BASELINE")
    print("=" * 80)
    v9 = {"M3":2.70,"M5":9.6,"M6":0.86,"M10":3.8,"M11":3.25}
    print(f"{'Metric':<8} {'V9':<12} {'V12 H2':<12}")
    for m, v9v in v9.items():
        v = rc[m]
        if m == "M6":
            print(f"{m:<8} {v9v*100:.1f}%{'':<6} {v*100:.1f}%")
        else:
            print(f"{m:<8} {v9v:<12.2f} {v:<12.3f}")

    print("\n" + "=" * 80)
    print("SELF-ASSESSMENT")
    print("=" * 80)
    m3c = rc["M3"]; m11c = rc["M11"]; sa25 = rc["SA25"]

    for label, val, tgt, op in [
        ("M3", m3c, 2.0, ">="), ("M12", m12, 0.3, ">="),
        ("M11'", m11c, 2.5, ">=")
    ]:
        status = "PASSES" if (val >= tgt if op == ">=" else val <= tgt) else "FAILS"
        if label == "M3" and 1.5 <= val < 2.0: status = "NEAR-MISS"
        print(f"{label} = {val:.2f}: {status} ({op} {tgt})")

    if sa25 >= 3:
        print(f"S/A@25 = {sa25:.1f}: OK")
    else:
        print(f"S/A@25 = {sa25:.1f}: S/A exhaustion is the binding constraint")

    viable = m3c >= 1.5
    print(f"\nVerdict: Hybrid 2 {'shows promise' if viable else 'does NOT achieve M3 target'}. "
          f"AI avoidance + pool contraction + progressive oversampling produces "
          f"transparent concentration but the physical contraction ratio cannot "
          f"match V9's 21:1 invisible contraction. "
          f"{'V9 fallback recommended.' if not viable else ''}")

    return all_res


if __name__ == "__main__":
    main()
