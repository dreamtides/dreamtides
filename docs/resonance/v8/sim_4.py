#!/usr/bin/env python3
"""
Simulation Agent 4: GPE-45 Graduated Pair-Escalation
Two-phase pair-escalation with smoothed transition, guaranteed floor,
and R1 fallback for non-pair slots.

Key mechanism: Each drafted dual-resonance card increments an ordered-pair
counter. Per non-guaranteed slot, P(pair-matched card) ramps from
min(count/8, 0.35) in phase 1 to min(count/5, 0.55) in phase 2, with a
smooth linear transition over picks 11-15. From pick 3+, one slot per pack
is guaranteed pair-matched. Non-pair-matched slots fall back to R1 (primary
resonance) filtering rather than full-pool random.
"""

import random
import math
from collections import defaultdict
from dataclasses import dataclass, field

# ─── Constants ────────────────────────────────────────────────────────────────

RESONANCES = ["Ember", "Stone", "Tide", "Zephyr"]

ARCHETYPES = [
    {"name": "Flash",        "primary": "Zephyr", "secondary": "Ember"},
    {"name": "Blink",        "primary": "Ember",  "secondary": "Zephyr"},
    {"name": "Storm",        "primary": "Ember",  "secondary": "Stone"},
    {"name": "SelfDiscard",  "primary": "Stone",  "secondary": "Ember"},
    {"name": "SelfMill",     "primary": "Stone",  "secondary": "Tide"},
    {"name": "Sacrifice",    "primary": "Tide",   "secondary": "Stone"},
    {"name": "Warriors",     "primary": "Tide",   "secondary": "Zephyr"},
    {"name": "Ramp",         "primary": "Zephyr", "secondary": "Tide"},
]

ARCHETYPE_NAMES = [a["name"] for a in ARCHETYPES]
ARCH_INFO = {a["name"]: a for a in ARCHETYPES}

CO_PRIMARY_PAIRS = {
    "Flash": "Ramp", "Ramp": "Flash",
    "Blink": "Storm", "Storm": "Blink",
    "SelfDiscard": "SelfMill", "SelfMill": "SelfDiscard",
    "Sacrifice": "Warriors", "Warriors": "Sacrifice",
}

# ─── Fitness Models ───────────────────────────────────────────────────────────

FITNESS_MODELS = {
    "Optimistic": {
        ("Warriors", "Sacrifice"): 1.0, ("Sacrifice", "Warriors"): 1.0,
        ("SelfDiscard", "SelfMill"): 1.0, ("SelfMill", "SelfDiscard"): 1.0,
        ("Blink", "Storm"): 1.0, ("Storm", "Blink"): 1.0,
        ("Flash", "Ramp"): 1.0, ("Ramp", "Flash"): 1.0,
    },
    "GraduatedRealistic": {
        ("Warriors", "Sacrifice"): 0.50, ("Sacrifice", "Warriors"): 0.50,
        ("SelfDiscard", "SelfMill"): 0.40, ("SelfMill", "SelfDiscard"): 0.40,
        ("Blink", "Storm"): 0.30, ("Storm", "Blink"): 0.30,
        ("Flash", "Ramp"): 0.25, ("Ramp", "Flash"): 0.25,
    },
    "Pessimistic": {
        ("Warriors", "Sacrifice"): 0.35, ("Sacrifice", "Warriors"): 0.35,
        ("SelfDiscard", "SelfMill"): 0.25, ("SelfMill", "SelfDiscard"): 0.25,
        ("Blink", "Storm"): 0.15, ("Storm", "Blink"): 0.15,
        ("Flash", "Ramp"): 0.10, ("Ramp", "Flash"): 0.10,
    },
    "Hostile": {
        ("Warriors", "Sacrifice"): 0.08, ("Sacrifice", "Warriors"): 0.08,
        ("SelfDiscard", "SelfMill"): 0.08, ("SelfMill", "SelfDiscard"): 0.08,
        ("Blink", "Storm"): 0.08, ("Storm", "Blink"): 0.08,
        ("Flash", "Ramp"): 0.08, ("Ramp", "Flash"): 0.08,
    },
}

# ─── Card & Pool ──────────────────────────────────────────────────────────────

@dataclass
class SimCard:
    card_id: int
    symbols: list
    archetype: str
    power: float


def compute_tier(card, archetype, fitness_model):
    """Compute tier stochastically (call once per card/archetype/draft)."""
    home = card.archetype
    if home == archetype:
        return "S"
    if home == "Generic":
        return "B"
    sibling = CO_PRIMARY_PAIRS.get(archetype)
    if home == sibling:
        rate = fitness_model.get((archetype, sibling), 0.0)
        return "A" if random.random() < rate else "B"
    arch_info = ARCH_INFO.get(archetype)
    home_info = ARCH_INFO.get(home)
    if (arch_info and home_info and
        (home_info["primary"] == arch_info["primary"] or
         home_info["primary"] == arch_info["secondary"] or
         home_info["secondary"] == arch_info["primary"])):
        return "C"
    return "F"


def build_pool(dual_res_pct=0.40, total_cards=360, generic_count=40):
    cards = []
    card_id = 0
    resonance_cards = total_cards - generic_count
    per_archetype = resonance_cards // len(ARCHETYPES)
    total_dual = int(dual_res_pct * resonance_cards)
    dual_per_arch = total_dual // len(ARCHETYPES)
    single_per_arch = per_archetype - dual_per_arch

    for arch in ARCHETYPES:
        for _ in range(single_per_arch):
            cards.append(SimCard(card_id, [arch["primary"]], arch["name"],
                                 random.uniform(4.0, 8.0)))
            card_id += 1
        for _ in range(dual_per_arch):
            cards.append(SimCard(card_id, [arch["primary"], arch["secondary"]],
                                 arch["name"], random.uniform(4.0, 8.0)))
            card_id += 1

    for _ in range(generic_count):
        cards.append(SimCard(card_id, [], "Generic", random.uniform(3.0, 7.0)))
        card_id += 1

    return cards


def build_pool_indices(pool):
    pair_pools = defaultdict(list)
    r1_pools = defaultdict(list)
    for card in pool:
        if len(card.symbols) >= 1:
            r1_pools[card.symbols[0]].append(card)
        if len(card.symbols) >= 2:
            pair_pools[(card.symbols[0], card.symbols[1])].append(card)
    return pair_pools, r1_pools


# ─── Tier Map: computed once per draft ────────────────────────────────────────

def compute_tier_map(pool, target_archetype, fitness_model):
    """Compute and freeze tier assignments for all cards for this draft."""
    tier_map = {}
    for card in pool:
        tier_map[card.card_id] = compute_tier(card, target_archetype,
                                               fitness_model)
    return tier_map


def is_sa(tier_map, card):
    return tier_map.get(card.card_id, "F") in ("S", "A")


# ─── GPE-45 Algorithm ─────────────────────────────────────────────────────────

def get_pair_prob(pair_count, pick_num,
                  phase1_div=8, phase1_cap=0.35,
                  phase2_div=5, phase2_cap=0.55,
                  trans_start=11, trans_end=15):
    p1 = min(pair_count / max(phase1_div, 1), phase1_cap)
    p2 = min(pair_count / max(phase2_div, 1), phase2_cap)
    if pick_num <= trans_start:
        return p1
    if pick_num >= trans_end:
        return p2
    t = (pick_num - trans_start) / (trans_end - trans_start)
    return p1 + t * (p2 - p1)


def build_pack(pool, pair_pools, r1_pools, pair_counters, pick_num,
               pack_size=4, floor_start=3, params=None):
    if params is None:
        params = {}

    top_pair = None
    top_count = 0
    for pair, count in pair_counters.items():
        if count > top_count:
            top_count = count
            top_pair = pair

    if top_pair is None:
        return random.sample(pool, min(pack_size, len(pool)))

    primary = top_pair[0]
    p = get_pair_prob(top_count, pick_num,
                      params.get("phase1_div", 8),
                      params.get("phase1_cap", 0.35),
                      params.get("phase2_div", 5),
                      params.get("phase2_cap", 0.55))

    pair_pool = pair_pools.get(top_pair, [])
    r1_pool = r1_pools.get(primary, [])
    r1_fallback = params.get("r1_fallback", True)

    pack = []
    used = set()
    guaranteed = 1 if (pick_num >= floor_start and pair_pool) else 0

    for slot in range(pack_size):
        card = None
        if slot < guaranteed:
            cands = [c for c in pair_pool if c.card_id not in used]
            if cands:
                card = random.choice(cands)
        elif random.random() < p:
            cands = [c for c in pair_pool if c.card_id not in used]
            if cands:
                card = random.choice(cands)

        if card is None and r1_fallback and r1_pool and pick_num >= floor_start:
            cands = [c for c in r1_pool if c.card_id not in used]
            if cands:
                card = random.choice(cands)

        if card is None:
            cands = [c for c in pool if c.card_id not in used]
            if cands:
                card = random.choice(cands)

        if card:
            pack.append(card)
            used.add(card.card_id)

    return pack


# ─── Player Strategies ────────────────────────────────────────────────────────

def committed_pick(pack, tier_map):
    best_card = None
    best_score = -1
    for card in pack:
        t = tier_map.get(card.card_id, "F")
        if t == "S":   score = 10 + card.power
        elif t == "A": score = 7 + card.power
        elif t == "B": score = 4 + card.power
        elif t == "C": score = 2 + card.power
        else:          score = card.power * 0.5
        if score > best_score:
            best_score = score
            best_card = card
    return best_card


def power_pick(pack):
    return max(pack, key=lambda c: c.power)


def signal_pick(pack, pair_counters):
    best_card = None
    best_score = -1
    for card in pack:
        score = card.power
        if len(card.symbols) >= 2:
            pair = (card.symbols[0], card.symbols[1])
            score += pair_counters.get(pair, 0) * 1.5 + 3.0
        if score > best_score:
            best_score = score
            best_card = card
    return best_card


# ─── Draft ────────────────────────────────────────────────────────────────────

@dataclass
class DraftResult:
    picks: list
    sa_per_pack: list
    archetype: str
    pair_counters_history: list
    pack_contents: list
    tier_map: dict


def simulate_draft(pool, pair_pools, r1_pools, fitness_model,
                   strategy="committed", num_picks=30, pack_size=4,
                   params=None):
    target = random.choice(ARCHETYPES)["name"]
    tier_map = compute_tier_map(pool, target, fitness_model)

    pair_counters = defaultdict(int)
    picks = []
    sa_per_pack = []
    pair_history = []
    pack_contents = []

    for pick_num in range(1, num_picks + 1):
        pack = build_pack(pool, pair_pools, r1_pools, pair_counters,
                          pick_num, pack_size, params=params)
        pack_contents.append(list(pack))

        sa_count = sum(1 for c in pack if is_sa(tier_map, c))
        sa_per_pack.append(sa_count)

        if strategy == "committed":
            chosen = committed_pick(pack, tier_map)
        elif strategy == "power":
            chosen = power_pick(pack)
        elif strategy == "signal":
            chosen = signal_pick(pack, pair_counters)
        else:
            chosen = random.choice(pack)

        picks.append(chosen)
        if len(chosen.symbols) >= 2:
            pair = (chosen.symbols[0], chosen.symbols[1])
            pair_counters[pair] += 1
        pair_history.append(dict(pair_counters))

    return DraftResult(picks, sa_per_pack, target, pair_history,
                       pack_contents, tier_map)


# ─── Metrics ──────────────────────────────────────────────────────────────────

def compute_metrics(results, fitness_model):
    m1s, m2s, m3s, m4s, m5s, m6s, m9s, m10s = ([] for _ in range(8))
    m7_sets = []
    m8_counts = defaultdict(int)
    pq6 = []
    consec_bads = []
    per_arch_m3 = defaultdict(list)

    for dr in results:
        arch = dr.archetype
        tm = dr.tier_map
        m8_counts[arch] += 1

        # M1: unique archetypes with S/A in early packs
        early_arch_counts = []
        for pi in range(min(5, len(dr.pack_contents))):
            pack = dr.pack_contents[pi]
            # For M1, we need per-archetype tier maps -- expensive.
            # Approximate: count unique card.archetype values for S/A cards
            # A card is S-tier for its home archetype always
            archs = set()
            for c in pack:
                archs.add(c.archetype)  # always S-tier for home
                # Also check target archetype
                if is_sa(tm, c):
                    archs.add(arch)
            early_arch_counts.append(len(archs))
        if early_arch_counts:
            m1s.append(sum(early_arch_counts) / len(early_arch_counts))

        # M2: early S/A for target archetype
        early_sa = dr.sa_per_pack[:5]
        if early_sa:
            m2s.append(sum(early_sa) / len(early_sa))

        # M3: late S/A
        late_sa = dr.sa_per_pack[5:]
        if late_sa:
            m3_val = sum(late_sa) / len(late_sa)
            m3s.append(m3_val)
            per_arch_m3[arch].append(m3_val)

        pq6.extend(late_sa)

        # Consecutive bad packs
        max_c = cur_c = 0
        for sa in late_sa:
            if sa < 1.5:
                cur_c += 1
                max_c = max(max_c, cur_c)
            else:
                cur_c = 0
        consec_bads.append(max_c)

        # M4: off-archetype
        cf_counts = []
        for pi in range(5, len(dr.pack_contents)):
            pack = dr.pack_contents[pi]
            cf = sum(1 for c in pack
                     if tm.get(c.card_id, "F") in ("C", "F"))
            cf_counts.append(cf)
        if cf_counts:
            m4s.append(sum(cf_counts) / len(cf_counts))

        # M5: convergence
        conv = 30
        for i in range(2, len(dr.sa_per_pack)):
            avg3 = sum(dr.sa_per_pack[i-2:i+1]) / 3
            if avg3 >= 1.5:
                conv = i + 1
                break
        m5s.append(conv)

        # M6: deck concentration
        sa_in_deck = sum(1 for c in dr.picks if is_sa(tm, c))
        m6s.append(sa_in_deck / len(dr.picks))

        # M7
        m7_sets.append((arch, set(c.card_id for c in dr.picks)))

        # M9
        if late_sa and len(late_sa) > 1:
            mean = sum(late_sa) / len(late_sa)
            var = sum((x - mean)**2 for x in late_sa) / len(late_sa)
            m9s.append(math.sqrt(var))

        m10s.append(max_c)

    # Aggregate
    def avg(lst):
        return sum(lst) / len(lst) if lst else 0

    metrics = {
        "M1": avg(m1s), "M2": avg(m2s), "M3": avg(m3s), "M4": avg(m4s),
        "M5": avg(m5s), "M6": avg(m6s), "M9": avg(m9s),
        "M10": avg(m10s), "M10_max": max(m10s) if m10s else 0,
    }

    # M7
    arch_runs = defaultdict(list)
    for arch, cs in m7_sets:
        arch_runs[arch].append(cs)
    overlaps = []
    for arch, runs in arch_runs.items():
        for i in range(len(runs)):
            for j in range(i+1, min(i+5, len(runs))):
                inter = len(runs[i] & runs[j])
                union = len(runs[i] | runs[j])
                if union > 0:
                    overlaps.append(inter / union)
    metrics["M7"] = avg(overlaps)

    # M8
    total = sum(m8_counts.values())
    metrics["M8_max"] = max(m8_counts.values()) / total if total else 0
    metrics["M8_min"] = min(m8_counts.values()) / total if total else 0

    # Per-archetype M3
    arch_m3 = {}
    for a in ARCHETYPE_NAMES:
        vals = per_arch_m3.get(a, [])
        arch_m3[a] = avg(vals)
    metrics["per_archetype_m3"] = arch_m3

    # Pack quality percentiles
    pqs = sorted(pq6)
    n = len(pqs)
    if n > 0:
        metrics["pq_10"] = pqs[int(n*0.10)]
        metrics["pq_25"] = pqs[int(n*0.25)]
        metrics["pq_50"] = pqs[int(n*0.50)]
        metrics["pq_75"] = pqs[int(n*0.75)]
        metrics["pq_90"] = pqs[min(int(n*0.90), n-1)]
    else:
        for k in ["pq_10","pq_25","pq_50","pq_75","pq_90"]:
            metrics[k] = 0

    metrics["consec_bad_avg"] = avg(consec_bads)
    metrics["consec_bad_max"] = max(consec_bads) if consec_bads else 0

    return metrics


# ─── Output Formatting ───────────────────────────────────────────────────────

def fmt(metrics, label=""):
    lines = []
    if label:
        lines.append(f"--- {label} ---")
    lines.append(f"  M1  (early variety):      {metrics['M1']:.2f}")
    lines.append(f"  M2  (early S/A):          {metrics['M2']:.2f}")
    lines.append(f"  M3  (late S/A avg):        {metrics['M3']:.2f}")
    lines.append(f"  M4  (off-arch/pack):       {metrics['M4']:.2f}")
    lines.append(f"  M5  (convergence pick):    {metrics['M5']:.1f}")
    lines.append(f"  M6  (deck concentration):  {metrics['M6']:.1%}")
    lines.append(f"  M7  (run overlap):         {metrics['M7']:.1%}")
    lines.append(f"  M8  (arch freq max/min):   {metrics['M8_max']:.1%}/{metrics['M8_min']:.1%}")
    lines.append(f"  M9  (S/A stddev):          {metrics['M9']:.2f}")
    lines.append(f"  M10 (consec <1.5):         {metrics['M10']:.1f} avg, {metrics['M10_max']} worst")
    lines.append(f"  Pack quality (10/25/50/75/90): "
                 f"{metrics['pq_10']}/{metrics['pq_25']}/{metrics['pq_50']}/"
                 f"{metrics['pq_75']}/{metrics['pq_90']}")
    lines.append(f"  Consec bad avg/max:        "
                 f"{metrics['consec_bad_avg']:.1f}/{metrics['consec_bad_max']}")
    am = metrics.get("per_archetype_m3", {})
    if am:
        lines.append("  Per-archetype M3:")
        for a in ARCHETYPE_NAMES:
            lines.append(f"    {a:15s}: {am.get(a,0):.2f}")
    return "\n".join(lines)


def trace_draft_str(pool, pair_pools, r1_pools, fitness_model,
                    strategy, params, label):
    dr = simulate_draft(pool, pair_pools, r1_pools, fitness_model,
                        strategy=strategy, params=params)
    tm = dr.tier_map
    lines = [f"\n=== {label} ({dr.archetype}) ==="]
    for i in range(len(dr.picks)):
        pn = i + 1
        card = dr.picks[i]
        sa = dr.sa_per_pack[i]
        pc = max(dr.pair_counters_history[i].values()) if dr.pair_counters_history[i] else 0
        tp = max(dr.pair_counters_history[i].items(), key=lambda x: x[1])[0] if dr.pair_counters_history[i] else ("?","?")
        tier = tm.get(card.card_id, "?")
        syms = ','.join(card.symbols) if card.symbols else 'gen'
        lines.append(f"  Pick {pn:2d}: S/A={sa} [{syms}] "
                     f"({card.archetype} {tier}) pair={tp} cnt={pc}")
    late_sa = dr.sa_per_pack[5:]
    lines.append(f"  Final M3: {sum(late_sa)/max(1,len(late_sa)):.2f}")
    return "\n".join(lines)


# ─── Main ─────────────────────────────────────────────────────────────────────

def run_sim(pool_config, fitness_model, strategy="committed",
            n_drafts=1000, params=None):
    pool = build_pool(**pool_config)
    pp, r1p = build_pool_indices(pool)
    results = []
    for _ in range(n_drafts):
        dr = simulate_draft(pool, pp, r1p, fitness_model,
                            strategy=strategy, params=params)
        results.append(dr)
    return compute_metrics(results, fitness_model), results


def main():
    random.seed(42)

    pools = {
        "V7_15pct": {"dual_res_pct": 0.15, "total_cards": 360, "generic_count": 40},
        "Enriched_40pct": {"dual_res_pct": 0.40, "total_cards": 360, "generic_count": 40},
    }

    champion = {"phase1_div": 8, "phase1_cap": 0.35,
                "phase2_div": 5, "phase2_cap": 0.55,
                "r1_fallback": True}

    out = []
    out.append("=" * 70)
    out.append("GPE-45 GRADUATED PAIR-ESCALATION SIMULATION")
    out.append("=" * 70)

    # ── Main grid ──
    out.append("\n## MAIN RESULTS: Committed Strategy")
    out.append("-" * 60)

    grid = {}
    for pn, pc in pools.items():
        for fn, fm in FITNESS_MODELS.items():
            label = f"{pn} / {fn}"
            print(f"Running: {label}...")
            m, _ = run_sim(pc, fm, strategy="committed",
                           n_drafts=1000, params=champion)
            grid[(pn, fn)] = m
            out.append(fmt(m, label=label))
            out.append("")

    # ── Strategy comparison ──
    out.append("\n## STRATEGY COMPARISON (Enriched 40%, Graduated Realistic)")
    out.append("-" * 60)
    for strat in ["committed", "power", "signal"]:
        label = f"Strategy: {strat}"
        print(f"Running: {label}...")
        m, _ = run_sim(pools["Enriched_40pct"],
                       FITNESS_MODELS["GraduatedRealistic"],
                       strategy=strat, n_drafts=1000, params=champion)
        out.append(fmt(m, label=label))
        out.append("")

    # ── Parameter Sensitivity ──
    out.append("\n## PARAMETER SENSITIVITY")
    out.append("-" * 60)
    sweeps = [
        ("Conservative", {"phase1_div": 10, "phase1_cap": 0.30,
                          "phase2_div": 6, "phase2_cap": 0.50, "r1_fallback": True}),
        ("Champion",     champion),
        ("Aggressive",   {"phase1_div": 6, "phase1_cap": 0.40,
                          "phase2_div": 4, "phase2_cap": 0.60, "r1_fallback": True}),
    ]

    for fn_name in ["GraduatedRealistic", "Pessimistic"]:
        out.append(f"\n  === Sweep: {fn_name} ===")
        for slabel, sparams in sweeps:
            m, _ = run_sim(pools["Enriched_40pct"],
                           FITNESS_MODELS[fn_name],
                           strategy="committed", n_drafts=500,
                           params=sparams)
            worst = min(m["per_archetype_m3"].values())
            out.append(f"  {slabel:15s}: M3={m['M3']:.2f}, M5={m['M5']:.1f}, "
                       f"M6={m['M6']:.1%}, M9={m['M9']:.2f}, "
                       f"M10={m['M10']:.1f}/{m['M10_max']}, worst={worst:.2f}")

    # ── R1 Fallback Ablation ──
    out.append("\n\n## R1 FALLBACK ABLATION (Enriched 40%, Graduated Realistic)")
    out.append("-" * 60)
    no_r1 = dict(champion)
    no_r1["r1_fallback"] = False
    m_off, _ = run_sim(pools["Enriched_40pct"],
                       FITNESS_MODELS["GraduatedRealistic"],
                       strategy="committed", n_drafts=500, params=no_r1)
    out.append(fmt(m_off, label="R1 fallback OFF"))
    out.append("")
    m_on, _ = run_sim(pools["Enriched_40pct"],
                      FITNESS_MODELS["GraduatedRealistic"],
                      strategy="committed", n_drafts=500, params=champion)
    out.append(fmt(m_on, label="R1 fallback ON"))

    # ── Draft Traces ──
    out.append("\n\n## DRAFT TRACES")
    out.append("-" * 60)
    trace_pool = build_pool(**pools["Enriched_40pct"])
    tp_pp, tp_r1 = build_pool_indices(trace_pool)
    fm_gr = FITNESS_MODELS["GraduatedRealistic"]

    random.seed(100)
    out.append(trace_draft_str(trace_pool, tp_pp, tp_r1, fm_gr,
                               "committed", champion, "Trace 1: Early Committer"))
    random.seed(200)
    out.append(trace_draft_str(trace_pool, tp_pp, tp_r1, fm_gr,
                               "signal", champion, "Trace 2: Signal Reader"))
    random.seed(300)
    out.append(trace_draft_str(trace_pool, tp_pp, tp_r1, fm_gr,
                               "committed", champion, "Trace 3: Committed (diff seed)"))

    # ── Fitness Degradation Curve ──
    out.append("\n\n## FITNESS DEGRADATION CURVE (Enriched 40%)")
    out.append("-" * 60)
    fit_order = ["Optimistic", "GraduatedRealistic", "Pessimistic", "Hostile"]
    out.append(f"  {'Fitness':<25s} {'M3':>6s} {'M5':>5s} {'M6':>7s} "
               f"{'M9':>5s} {'M10':>5s} {'WorstM3':>8s}")
    for fn in fit_order:
        m = grid[("Enriched_40pct", fn)]
        worst = min(m["per_archetype_m3"].values()) if m["per_archetype_m3"] else 0
        out.append(f"  {fn:<25s} {m['M3']:6.2f} {m['M5']:5.1f} {m['M6']:6.1%} "
                   f"{m['M9']:5.2f} {m['M10']:5.1f} {worst:8.2f}")

    out.append(f"\n  {'Fitness':<25s} {'M3':>6s} {'M5':>5s} {'M6':>7s} "
               f"{'M9':>5s} {'M10':>5s} {'WorstM3':>8s}")
    for fn in fit_order:
        m = grid[("V7_15pct", fn)]
        worst = min(m["per_archetype_m3"].values()) if m["per_archetype_m3"] else 0
        out.append(f"  {fn:<25s} {m['M3']:6.2f} {m['M5']:5.1f} {m['M6']:6.1%} "
                   f"{m['M9']:5.2f} {m['M10']:5.1f} {worst:8.2f}")

    output_text = "\n".join(out)
    print(output_text)
    with open("/Users/dthurn/Documents/GoogleDrive/dreamtides/docs/resonance/v8/sim_4_output.txt", "w") as f:
        f.write(output_text)

    return grid


if __name__ == "__main__":
    main()
