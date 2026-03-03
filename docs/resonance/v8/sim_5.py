#!/usr/bin/env python3
"""
Simulation Agent 5: Symbol-Weighted Graduated Escalation

Algorithm (post-discussion refinement): Track weighted pair counters from
3-symbol cards. AAB cards earn +3 for primary resonance, +1 for secondary.
ABC cards earn +2/+1/+1. Pair-matched slots unlock progressively:
  - 1 slot at primary counter >= T1
  - 2 slots at primary counter >= T2
  - 3 slots at primary counter >= T3
Remaining slots filled randomly. No surge/floor binary -- smooth graduated
escalation replaces bimodal delivery.

Pools tested:
  - V7 Standard (15% dual-res, 1-2 symbols per card)
  - 40% Enriched (40% dual-res, 1-2 symbols per card)
  - Symbol-Rich (84.5% dual-res, 3 symbols per card with repetition)

Fitness models tested:
  - Optimistic (100% uniform)
  - Graduated Realistic (per-pair avg 36%)
  - Pessimistic (per-pair avg 21%)
  - Hostile (8% uniform)
"""

import random
import math
from collections import defaultdict
from dataclasses import dataclass, field

# ── Constants ────────────────────────────────────────────────────────────────

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
RESONANCES = ["Ember", "Stone", "Tide", "Zephyr"]

SIBLING_MAP = {}
for i, (name, pri, sec) in enumerate(ARCHETYPES):
    for j, (name2, pri2, sec2) in enumerate(ARCHETYPES):
        if i != j and pri == pri2:
            SIBLING_MAP[name] = name2

FITNESS_MODELS = {
    "Optimistic": {
        ("Warriors", "Sacrifice"): 1.0, ("Sacrifice", "Warriors"): 1.0,
        ("Self-Discard", "Self-Mill"): 1.0, ("Self-Mill", "Self-Discard"): 1.0,
        ("Blink", "Storm"): 1.0, ("Storm", "Blink"): 1.0,
        ("Flash", "Ramp"): 1.0, ("Ramp", "Flash"): 1.0,
    },
    "Graduated": {
        ("Warriors", "Sacrifice"): 0.50, ("Sacrifice", "Warriors"): 0.50,
        ("Self-Discard", "Self-Mill"): 0.40, ("Self-Mill", "Self-Discard"): 0.40,
        ("Blink", "Storm"): 0.30, ("Storm", "Blink"): 0.30,
        ("Flash", "Ramp"): 0.25, ("Ramp", "Flash"): 0.25,
    },
    "Pessimistic": {
        ("Warriors", "Sacrifice"): 0.35, ("Sacrifice", "Warriors"): 0.35,
        ("Self-Discard", "Self-Mill"): 0.25, ("Self-Mill", "Self-Discard"): 0.25,
        ("Blink", "Storm"): 0.15, ("Storm", "Blink"): 0.15,
        ("Flash", "Ramp"): 0.10, ("Ramp", "Flash"): 0.10,
    },
    "Hostile": {
        ("Warriors", "Sacrifice"): 0.08, ("Sacrifice", "Warriors"): 0.08,
        ("Self-Discard", "Self-Mill"): 0.08, ("Self-Mill", "Self-Discard"): 0.08,
        ("Blink", "Storm"): 0.08, ("Storm", "Blink"): 0.08,
        ("Flash", "Ramp"): 0.08, ("Ramp", "Flash"): 0.08,
    },
}

NUM_DRAFTS = 1000
NUM_PICKS = 30
PACK_SIZE = 4

DEFAULT_T1 = 3
DEFAULT_T2 = 6
DEFAULT_T3 = 9

# ── Data Structures ──────────────────────────────────────────────────────────

@dataclass
class SimCard:
    card_id: int
    symbols: list
    archetype: str
    power: float
    tiers: dict = field(default_factory=dict)


def assign_tiers(card, fitness_model):
    card.tiers = {}
    home = card.archetype
    if home == "Generic":
        for a in ARCHETYPE_NAMES:
            card.tiers[a] = "C"
        return
    for a in ARCHETYPE_NAMES:
        if a == home:
            card.tiers[a] = "S"
        elif a == SIBLING_MAP.get(home):
            rate = fitness_model.get((home, a), 0.0)
            card.tiers[a] = "A" if random.random() < rate else "B"
        else:
            card.tiers[a] = "B" if random.random() < 0.05 else random.choice(["C", "F"])


def is_sa(tier):
    return tier in ("S", "A")


# ── Pool Construction ────────────────────────────────────────────────────────

def build_pool(pool_type, fitness_model):
    cards = []
    cid = 0
    all_res = RESONANCES[:]

    if pool_type == "standard":
        for arch_name, pri, sec in ARCHETYPES:
            for i in range(40):
                symbols = [pri, sec] if i < 7 else [pri]
                c = SimCard(cid, symbols, arch_name, random.uniform(3, 8))
                assign_tiers(c, fitness_model)
                cards.append(c); cid += 1
        for _ in range(40):
            c = SimCard(cid, [], "Generic", random.uniform(2, 6))
            assign_tiers(c, fitness_model)
            cards.append(c); cid += 1

    elif pool_type == "enriched":
        for arch_name, pri, sec in ARCHETYPES:
            for i in range(40):
                symbols = [pri, sec] if i < 16 else [pri]
                c = SimCard(cid, symbols, arch_name, random.uniform(3, 8))
                assign_tiers(c, fitness_model)
                cards.append(c); cid += 1
        for _ in range(40):
            c = SimCard(cid, [], "Generic", random.uniform(2, 6))
            assign_tiers(c, fitness_model)
            cards.append(c); cid += 1

    elif pool_type == "symbol_rich":
        for arch_name, pri, sec in ARCHETYPES:
            others = [r for r in all_res if r != pri and r != sec]
            for i in range(40):
                frac = i / 40
                if frac < 0.55:
                    symbols = [pri, pri, sec]  # AAB
                elif frac < 0.75:
                    symbols = [pri, sec, sec]  # ABB
                elif frac < 0.95:
                    symbols = [pri, sec, random.choice(others)]  # ABC
                else:
                    symbols = [pri, pri, pri]  # AAA
                c = SimCard(cid, symbols, arch_name, random.uniform(3, 8))
                assign_tiers(c, fitness_model)
                cards.append(c); cid += 1
        for _ in range(40):
            c = SimCard(cid, [], "Generic", random.uniform(2, 6))
            assign_tiers(c, fitness_model)
            cards.append(c); cid += 1

    return cards


# ── Algorithm ────────────────────────────────────────────────────────────────

def get_symbol_weights(card):
    weights = defaultdict(int)
    for idx, sym in enumerate(card.symbols):
        weights[sym] += 2 if idx == 0 else 1
    return weights


def identify_pair(counters):
    sorted_res = sorted(counters.items(), key=lambda x: -x[1])
    if len(sorted_res) < 2:
        return (sorted_res[0][0], sorted_res[0][0]) if sorted_res else (None, None)
    return (sorted_res[0][0], sorted_res[1][0])


def pair_score(card, pri_res, sec_res):
    if not card.symbols:
        return 0
    score = 0
    if card.symbols[0] == pri_res:
        score += 3
    if len(card.symbols) >= 2:
        if card.symbols[0] == pri_res and card.symbols[1] in (pri_res, sec_res):
            score += 2
        if card.symbols[1] == sec_res:
            score += 1
    for s in card.symbols:
        if s == pri_res:
            score += 0.5
        elif s == sec_res:
            score += 0.25
    return score


def build_pack(pool, counters, pick_num, params, rng):
    t1, t2, t3 = params["t1"], params["t2"], params["t3"]
    pri_res, sec_res = identify_pair(counters)
    pri_count = counters.get(pri_res, 0) if pri_res else 0

    if pri_count >= t3:
        pair_slots = 3
    elif pri_count >= t2:
        pair_slots = 2
    elif pri_count >= t1:
        pair_slots = 1
    else:
        pair_slots = 0

    pack = []
    used_ids = set()

    if pair_slots > 0 and pri_res is not None:
        candidates = []
        for c in pool:
            ps = pair_score(c, pri_res, sec_res)
            if ps >= 3:
                candidates.append((ps, random.random(), c))
        candidates.sort(key=lambda x: (-x[0], x[1]))
        top_n = min(len(candidates), pair_slots * 4)
        top_pool = candidates[:top_n]
        rng.shuffle(top_pool)
        for _, _, c in top_pool:
            if len(pack) >= pair_slots:
                break
            if c.card_id not in used_ids:
                pack.append(c)
                used_ids.add(c.card_id)
        # Fallback to R1 if not enough pair candidates
        if len(pack) < pair_slots:
            r1 = [c for c in pool if c.card_id not in used_ids
                  and c.symbols and c.symbols[0] == pri_res]
            rng.shuffle(r1)
            for c in r1:
                if len(pack) >= pair_slots:
                    break
                pack.append(c)
                used_ids.add(c.card_id)

    # Fill remaining with random
    remaining = [c for c in pool if c.card_id not in used_ids]
    rng.shuffle(remaining)
    needed = PACK_SIZE - len(pack)
    pack.extend(remaining[:needed])
    for c in pack[len(pack) - needed:]:
        used_ids.add(c.card_id)

    return pack[:PACK_SIZE], pair_slots


# ── Player Strategies ────────────────────────────────────────────────────────

def pick_committed(pack, arch):
    tier_v = {"S": 5, "A": 4, "B": 3, "C": 2, "F": 1}
    return max(pack, key=lambda c: tier_v.get(c.tiers.get(arch, "F"), 0) + c.power / 100)


def pick_power(pack):
    return max(pack, key=lambda c: c.power)


def pick_signal(pack, counters, pick_num):
    if pick_num <= 3:
        return max(pack, key=lambda c: c.power)
    pri_res, sec_res = identify_pair(counters)
    target = None
    for name, pri, sec in ARCHETYPES:
        if pri == pri_res and sec == sec_res:
            target = name; break
    if not target:
        for name, pri, sec in ARCHETYPES:
            if pri == pri_res:
                target = name; break
    return pick_committed(pack, target) if target else max(pack, key=lambda c: c.power)


# ── Single Draft ─────────────────────────────────────────────────────────────

@dataclass
class DraftResult:
    archetype: str
    strategy: str
    picks: list
    packs: list  # list of 4-card packs for M1 measurement
    pack_sa: list  # S/A count per pack for committed archetype
    pair_slots_hist: list
    counters_hist: list


def simulate_draft(pool, strategy, params, rng, target_arch=None):
    counters = defaultdict(float)
    picks, packs, pack_sa_list, ps_hist, c_hist = [], [], [], [], []

    if strategy == "committed" and target_arch is None:
        target_arch = rng.choice(ARCHETYPE_NAMES)

    for pn in range(1, NUM_PICKS + 1):
        pack, ps = build_pack(pool, counters, pn, params, rng)
        packs.append(pack)
        ps_hist.append(ps)

        # Determine eval archetype
        if strategy == "committed":
            eval_arch = target_arch
        elif strategy == "signal":
            if pn <= 3:
                eval_arch = None
            else:
                p, s = identify_pair(counters)
                eval_arch = None
                for name, pri, sec in ARCHETYPES:
                    if pri == p and sec == s:
                        eval_arch = name; break
                if not eval_arch:
                    for name, pri, sec in ARCHETYPES:
                        if pri == p:
                            eval_arch = name; break
        else:
            p, s = identify_pair(counters)
            eval_arch = None
            for name, pri, sec in ARCHETYPES:
                if pri == p and sec == s:
                    eval_arch = name; break

        sa_count = sum(1 for c in pack if eval_arch and is_sa(c.tiers.get(eval_arch, "F")))
        pack_sa_list.append(sa_count)

        # Pick
        if strategy == "committed":
            chosen = pick_committed(pack, target_arch)
        elif strategy == "power":
            chosen = pick_power(pack)
        else:
            chosen = pick_signal(pack, counters, pn)

        picks.append(chosen)
        for res, w in get_symbol_weights(chosen).items():
            counters[res] += w
        c_hist.append(dict(counters))

        # Signal reader commits at pick 3
        if strategy == "signal" and pn == 3:
            p, s = identify_pair(counters)
            for name, pri, sec in ARCHETYPES:
                if pri == p and sec == s:
                    target_arch = name; break
            if target_arch is None:
                target_arch = rng.choice(ARCHETYPE_NAMES)

    # Resolve final archetype for non-committed
    if target_arch is None:
        p, s = identify_pair(counters)
        for name, pri, sec in ARCHETYPES:
            if pri == p and sec == s:
                target_arch = name; break
        if target_arch is None:
            target_arch = ARCHETYPE_NAMES[0]

    # Re-evaluate pack_sa for signal/power with final archetype
    if strategy != "committed":
        for i, pack in enumerate(packs):
            pack_sa_list[i] = sum(1 for c in pack if is_sa(c.tiers.get(target_arch, "F")))

    return DraftResult(target_arch, strategy, picks, packs, pack_sa_list,
                       ps_hist, c_hist)


# ── Metrics ──────────────────────────────────────────────────────────────────

def compute_metrics(results):
    m = {}
    committed = [r for r in results if r.strategy == "committed"]
    committed_signal = [r for r in results if r.strategy in ("committed", "signal")]

    # M1: Picks 1-5, unique archetypes with S/A per pack (avg across packs)
    m1_vals = []
    for r in results:
        for pi in range(min(5, len(r.packs))):
            pack = r.packs[pi]
            arches_seen = set()
            for c in pack:
                for a in ARCHETYPE_NAMES:
                    if is_sa(c.tiers.get(a, "F")):
                        arches_seen.add(a)
            m1_vals.append(len(arches_seen))
    m["M1"] = sum(m1_vals) / max(len(m1_vals), 1)

    # M2: Picks 1-5, S/A for emerging archetype per pack
    m2_vals = []
    for r in committed:
        for pi in range(min(5, len(r.pack_sa))):
            m2_vals.append(r.pack_sa[pi])
    m["M2"] = sum(m2_vals) / max(len(m2_vals), 1)

    # M3: Picks 6+, S/A for committed archetype per pack
    m3_vals = []
    for r in committed_signal:
        m3_vals.extend(r.pack_sa[5:])
    m["M3"] = sum(m3_vals) / max(len(m3_vals), 1)

    # M3 per archetype (committed only for clean measurement)
    m3_arch = defaultdict(list)
    for r in committed:
        m3_arch[r.archetype].extend(r.pack_sa[5:])
    m["M3_arch"] = {a: sum(v)/max(len(v), 1) for a, v in m3_arch.items()}

    # M4: Off-archetype cards per pack, picks 6+
    m4_vals = [PACK_SIZE - sa for sa in m3_vals]
    m["M4"] = sum(m4_vals) / max(len(m4_vals), 1)

    # M5: Convergence pick
    m5_vals = []
    for r in committed:
        conv = NUM_PICKS
        for start in range(len(r.pack_sa)):
            rest = r.pack_sa[start:]
            if len(rest) >= 3 and sum(rest) / len(rest) >= 1.5:
                conv = start + 1; break
        m5_vals.append(conv)
    m["M5"] = sum(m5_vals) / max(len(m5_vals), 1)

    # M6: Deck concentration
    m6_vals = []
    for r in committed:
        sa_n = sum(1 for c in r.picks if is_sa(c.tiers.get(r.archetype, "F")))
        m6_vals.append(sa_n / NUM_PICKS)
    m["M6"] = sum(m6_vals) / max(len(m6_vals), 1)

    # M7: Run-to-run overlap
    arch_sets = defaultdict(list)
    for r in committed:
        arch_sets[r.archetype].append(set(c.card_id for c in r.picks))
    overlaps = []
    for arch, sets in arch_sets.items():
        for i in range(min(len(sets) - 1, 50)):
            s1, s2 = sets[i], sets[i + 1]
            if s1 | s2:
                overlaps.append(len(s1 & s2) / len(s1 | s2))
    m["M7"] = sum(overlaps) / max(len(overlaps), 1) if overlaps else 0

    # M8: Archetype frequency
    freq = defaultdict(int)
    for r in results:
        freq[r.archetype] += 1
    total = max(sum(freq.values()), 1)
    m["M8_max"] = max(freq.values()) / total if freq else 0
    m["M8_min"] = min(freq.values()) / total if freq else 0

    # M9: StdDev of S/A per pack, picks 6+
    if m3_vals:
        mean = m["M3"]
        m["M9"] = math.sqrt(sum((v - mean)**2 for v in m3_vals) / len(m3_vals))
    else:
        m["M9"] = 0

    # M10: Max consecutive packs < 1.5 S/A, committed only, picks 6+
    m10_vals = []
    for r in committed:
        post = r.pack_sa[5:]
        mx, cur = 0, 0
        for sa in post:
            if sa < 1.5:
                cur += 1; mx = max(mx, cur)
            else:
                cur = 0
        m10_vals.append(mx)
    m["M10_avg"] = sum(m10_vals) / max(len(m10_vals), 1)
    m["M10_worst"] = max(m10_vals) if m10_vals else 0
    m["M10_p90"] = sorted(m10_vals)[int(len(m10_vals) * 0.90)] if m10_vals else 0

    # Pack quality distribution (picks 6+, committed)
    pq = []
    for r in committed:
        pq.extend(r.pack_sa[5:])
    if pq:
        pq_s = sorted(pq)
        n = len(pq_s)
        m["PQ_p10"] = pq_s[int(n * 0.10)]
        m["PQ_p25"] = pq_s[int(n * 0.25)]
        m["PQ_p50"] = pq_s[int(n * 0.50)]
        m["PQ_p75"] = pq_s[int(n * 0.75)]
        m["PQ_p90"] = pq_s[min(int(n * 0.90), n - 1)]
    else:
        for k in ["PQ_p10", "PQ_p25", "PQ_p50", "PQ_p75", "PQ_p90"]:
            m[k] = 0

    # Consecutive bad pack analysis
    streaks = []
    for r in committed:
        post = r.pack_sa[5:]
        cur = 0
        for sa in post:
            if sa < 1.5:
                cur += 1
            else:
                if cur > 0:
                    streaks.append(cur)
                cur = 0
        if cur > 0:
            streaks.append(cur)
    m["bad_avg"] = sum(streaks) / max(len(streaks), 1) if streaks else 0
    m["bad_worst"] = max(streaks) if streaks else 0

    return m


# ── Trace Formatting ─────────────────────────────────────────────────────────

def fmt_trace(r, label):
    lines = [f"\n### {label} ({r.archetype}, {r.strategy})"]
    for i in range(len(r.picks)):
        c = r.picks[i]
        sym = "/".join(c.symbols) if c.symbols else "none"
        tier = c.tiers.get(r.archetype, "?")
        sa = r.pack_sa[i]
        ps = r.pair_slots_hist[i]
        ct = r.counters_hist[i]
        top2 = sorted(ct.items(), key=lambda x: -x[1])[:2]
        cs = ", ".join(f"{k}={v:.0f}" for k, v in top2)
        lines.append(f"Pick {i+1:2d}: [{sym:20s}] tier={tier} packS/A={sa} "
                      f"pairSlots={ps} [{cs}]")
    sa_final = sum(1 for c in r.picks if is_sa(c.tiers.get(r.archetype, "F")))
    lines.append(f"Final S/A: {sa_final}/30")
    return "\n".join(lines)


# ── Main ─────────────────────────────────────────────────────────────────────

def run_all():
    default_p = {"t1": DEFAULT_T1, "t2": DEFAULT_T2, "t3": DEFAULT_T3}
    pools = ["standard", "enriched", "symbol_rich"]
    fitnesses = ["Optimistic", "Graduated", "Pessimistic", "Hostile"]
    strategies = ["committed", "power", "signal"]

    all_m = {}
    traces = []

    print("=" * 72)
    print("AGENT 5: Symbol-Weighted Graduated Escalation")
    print("=" * 72)

    for pool in pools:
        for fit in fitnesses:
            key = f"{pool}_{fit}"
            print(f"Running {key} ...", flush=True)
            rng = random.Random(42)
            fitness_model = FITNESS_MODELS[fit]
            card_pool = build_pool(pool, fitness_model)

            results = []
            for di in range(NUM_DRAFTS):
                strat = strategies[di % 3]
                ta = ARCHETYPE_NAMES[di % 8] if strat == "committed" else None
                results.append(simulate_draft(card_pool, strat, default_p, rng, ta))

            all_m[key] = compute_metrics(results)

            # Collect traces for primary condition
            if pool == "symbol_rich" and fit == "Graduated":
                war = [r for r in results if r.archetype == "Warriors" and r.strategy == "committed"]
                if war:
                    traces.append(fmt_trace(war[0], "Early Committer (Warriors)"))
                pwr = [r for r in results if r.strategy == "power"]
                if pwr:
                    traces.append(fmt_trace(pwr[0], "Power Chaser"))
                sig = [r for r in results if r.strategy == "signal"]
                if sig:
                    traces.append(fmt_trace(sig[0], "Signal Reader"))

    # Parameter sweeps
    sweep_m = {}
    sweep_configs = [
        ("Fast 2/4/6",    {"t1": 2, "t2": 4, "t3": 6}),
        ("Default 3/6/9", {"t1": 3, "t2": 6, "t3": 9}),
        ("Slow 4/8/12",   {"t1": 4, "t2": 8, "t3": 12}),
        ("Rapid 2/3/5",   {"t1": 2, "t2": 3, "t3": 5}),
    ]
    for label, params in sweep_configs:
        print(f"Sweep: {label} ...", flush=True)
        rng = random.Random(42)
        card_pool = build_pool("symbol_rich", FITNESS_MODELS["Graduated"])
        results = []
        for di in range(NUM_DRAFTS):
            strat = strategies[di % 3]
            ta = ARCHETYPE_NAMES[di % 8] if strat == "committed" else None
            results.append(simulate_draft(card_pool, strat, params, rng, ta))
        sweep_m[label] = compute_metrics(results)

    # ── Output ───────────────────────────────────────────────────────────────

    print("\n" + "=" * 72)
    print("RESULTS")
    print("=" * 72)

    for key in sorted(all_m):
        v = all_m[key]
        print(f"\n--- {key} ---")
        print(f"  M1={v['M1']:.2f}  M2={v['M2']:.2f}  M3={v['M3']:.2f}  "
              f"M4={v['M4']:.2f}  M5={v['M5']:.1f}")
        print(f"  M6={v['M6']:.1%}  M7={v['M7']:.1%}  "
              f"M8={v['M8_min']:.1%}-{v['M8_max']:.1%}  M9={v['M9']:.2f}")
        print(f"  M10 avg={v['M10_avg']:.1f}  p90={v['M10_p90']}  worst={v['M10_worst']}")
        print(f"  PQ: p10={v['PQ_p10']} p25={v['PQ_p25']} p50={v['PQ_p50']} "
              f"p75={v['PQ_p75']} p90={v['PQ_p90']}")
        print(f"  Bad streaks: avg={v['bad_avg']:.1f} worst={v['bad_worst']}")
        if v.get("M3_arch"):
            for a in ARCHETYPE_NAMES:
                val = v["M3_arch"].get(a, 0)
                mark = " *" if val < 2.0 else ""
                print(f"    {a:15s}: {val:.2f}{mark}")

    print("\n" + "=" * 72)
    print("PARAMETER SENSITIVITY (symbol_rich, Graduated)")
    print("=" * 72)
    for label, v in sweep_m.items():
        print(f"  {label:15s}: M3={v['M3']:.2f}  M5={v['M5']:.1f}  "
              f"M6={v['M6']:.1%}  M9={v['M9']:.2f}  M10avg={v['M10_avg']:.1f}  "
              f"M10worst={v['M10_worst']}  bad_worst={v['bad_worst']}")

    print("\n" + "=" * 72)
    print("FITNESS DEGRADATION CURVE")
    print("=" * 72)
    for pool in pools:
        print(f"\n  {pool}:")
        for fit in fitnesses:
            key = f"{pool}_{fit}"
            print(f"    {fit:15s}: M3={all_m[key]['M3']:.2f}  "
                  f"M10avg={all_m[key]['M10_avg']:.1f}")

    print("\n" + "=" * 72)
    print("PER-ARCHETYPE CONVERGENCE (symbol_rich, Graduated)")
    print("=" * 72)
    v = all_m["symbol_rich_Graduated"]
    print(f"{'Archetype':15s} | {'M3':>5s} | Pass")
    print("-" * 35)
    for a in ARCHETYPE_NAMES:
        val = v["M3_arch"].get(a, 0)
        print(f"{a:15s} | {val:5.2f} | {'YES' if val >= 2.0 else 'no'}")

    for t in traces:
        print(t)

    return all_m, sweep_m


if __name__ == "__main__":
    all_m, sweep_m = run_all()
