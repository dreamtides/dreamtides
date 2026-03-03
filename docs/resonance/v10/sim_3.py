#!/usr/bin/env python3
"""
Simulation Agent 3: D4 Escalating Aggression
Monte Carlo simulation of the Resonance Draft System V10.

Algorithm (Post-Critique Revision):
- 7 AI drafters assigned to 7 of 8 archetypes (1 open lane)
- Escalating picks per AI per round:
    Phase 1 (picks 1-5):  2 cards/AI/round, 70% archetype focus
    Phase 2 (picks 6-10): 3 cards/AI/round, 75% archetype focus
    Phase 3 (picks 11-15): 4 cards/AI/round, 90% archetype focus
    Phase 4 (picks 16+):  5 cards/AI/round, 95% archetype focus
- Per-AI phase-shift offset of {-1, 0, +1} for staggered escalation
- Market culling: 8 lowest-power cards removed per round
- Level 0 reactivity (fully predetermined)
- Pair-affinity card evaluation (8-bit)

Fitness Model (Graduated Realistic):
Each archetype's "S/A" pool includes:
  - Own archetype cards at power >= 3.5 (S if >= 7, A otherwise)
  - Sibling archetype cards at the sibling rate (50/40/30/25%)
  - Shared-resonance bridge cards at ~20% rate with power >= 5
  - High-power generics at ~15% rate

This gives each archetype roughly 40 own + 10-20 sibling + 10-15 bridge
+ 5-6 generic = 65-80 S/A-tier cards in a 360-card pool. A 4-card pack
from a full pool has ~0.72-0.89 S/A cards per archetype baseline; the
convergence algorithm must raise this above 2.0 by pick 6.
"""

import random
import math
import statistics
from dataclasses import dataclass, field
from collections import defaultdict
from typing import Optional

# ─── Constants ───────────────────────────────────────────────────────────────

NUM_DRAFTS = 1000
NUM_PICKS = 30
POOL_SIZE = 360
CARDS_PER_ARCHETYPE = 40
GENERIC_CARDS = 40
PACK_SIZE = 4
POOL_FLOOR = 30
MARKET_CULL_COUNT = 8
NUM_AIS = 7
NUM_ARCHETYPES = 8

ARCHETYPES = [
    "Flash", "Blink", "Storm", "Self-Discard",
    "Self-Mill", "Sacrifice", "Warriors", "Ramp",
]

RESONANCES = ["Zephyr", "Ember", "Stone", "Tide"]

ARCHETYPE_RESONANCES = {
    "Flash":        ("Zephyr", "Ember"),
    "Blink":        ("Ember", "Zephyr"),
    "Storm":        ("Ember", "Stone"),
    "Self-Discard": ("Stone", "Ember"),
    "Self-Mill":    ("Stone", "Tide"),
    "Sacrifice":    ("Tide", "Stone"),
    "Warriors":     ("Tide", "Zephyr"),
    "Ramp":         ("Zephyr", "Tide"),
}

SIBLING_PAIRS = {
    "Warriors": "Sacrifice", "Sacrifice": "Warriors",
    "Self-Discard": "Self-Mill", "Self-Mill": "Self-Discard",
    "Blink": "Storm", "Storm": "Blink",
    "Flash": "Ramp", "Ramp": "Flash",
}

SIBLING_RATES = {
    "Warriors": 0.50, "Sacrifice": 0.50,
    "Self-Discard": 0.40, "Self-Mill": 0.40,
    "Blink": 0.30, "Storm": 0.30,
    "Flash": 0.25, "Ramp": 0.25,
}

def shares_resonance(arch1, arch2):
    if arch1 is None or arch2 is None:
        return False
    r1 = set(ARCHETYPE_RESONANCES.get(arch1, ()))
    r2 = set(ARCHETYPE_RESONANCES.get(arch2, ()))
    return bool(r1 & r2)

PHASES = [
    (1, 5, 2, 0.70),
    (6, 10, 3, 0.75),
    (11, 15, 4, 0.90),
    (16, 999, 5, 0.95),
]

def get_phase(pick_num, offset):
    adjusted = pick_num - offset
    for min_p, max_p, cards, focus in PHASES:
        if min_p <= adjusted <= max_p:
            return cards, focus
    if adjusted < 1:
        return PHASES[0][2], PHASES[0][3]
    return PHASES[-1][2], PHASES[-1][3]


# ─── Card Model ──────────────────────────────────────────────────────────────

@dataclass
class SimCard:
    id: int
    archetype: Optional[str]
    visible_symbols: list
    power: float
    pair_affinity: float       # affinity to own archetype (0-1)
    bridge_affinity: float     # affinity as bridge card to related archetypes (0-1)
    fitness: dict = field(default_factory=dict)


def build_fitness(card, sibling_rates_map):
    """
    Assign S/A/B/C/F fitness tiers.

    For each target archetype:
      Own-archetype cards:  power >= 7 -> S, power >= 3.5 -> A, else B
      Sibling cards:        top `rate` by bridge_affinity -> A (pwr>=5) or B, else C
      Shared-res bridge:    top 20% by bridge_affinity and pwr >= 5 -> A, else B/C
      Generic:              power >= 6 -> B, else C
      No connection:        power >= 8.5 -> C, else F
    """
    fitness = {}
    for target in ARCHETYPES:
        if card.archetype is None:
            # Generic
            if card.power >= 6.0:
                fitness[target] = "B"
            else:
                fitness[target] = "C"
        elif card.archetype == target:
            # Own archetype
            if card.power >= 7.0:
                fitness[target] = "S"
            elif card.power >= 3.5:
                fitness[target] = "A"
            else:
                fitness[target] = "B"
        elif SIBLING_PAIRS.get(target) == card.archetype:
            # Sibling
            rate = sibling_rates_map.get(target, 0.25)
            if card.bridge_affinity >= (1.0 - rate):
                if card.power >= 5.0:
                    fitness[target] = "A"
                else:
                    fitness[target] = "B"
            else:
                if card.power >= 6.0:
                    fitness[target] = "B"
                else:
                    fitness[target] = "C"
        elif shares_resonance(target, card.archetype):
            # Shared resonance (bridge cards)
            if card.bridge_affinity >= 0.80 and card.power >= 5.0:
                fitness[target] = "A"
            elif card.power >= 5.0:
                fitness[target] = "B"
            else:
                fitness[target] = "C"
        else:
            # No connection
            if card.power >= 8.5:
                fitness[target] = "C"
            else:
                fitness[target] = "F"
    return fitness


def generate_pool(rng, sibling_rates_map=None):
    """Generate 360 cards."""
    if sibling_rates_map is None:
        sibling_rates_map = SIBLING_RATES
    cards = []
    card_id = 0

    for arch in ARCHETYPES:
        primary_res, secondary_res = ARCHETYPE_RESONANCES[arch]
        for _ in range(CARDS_PER_ARCHETYPE):
            r = rng.random()
            if r < 0.10:
                visible = [primary_res, secondary_res]
            elif r < 0.87:
                visible = [primary_res]
            else:
                visible = [secondary_res]

            power = rng.uniform(1.0, 10.0)
            pair_aff = rng.random()
            bridge_aff = rng.random()

            card = SimCard(
                id=card_id, archetype=arch,
                visible_symbols=visible, power=power,
                pair_affinity=pair_aff, bridge_affinity=bridge_aff,
            )
            card.fitness = build_fitness(card, sibling_rates_map)
            cards.append(card)
            card_id += 1

    for _ in range(GENERIC_CARDS):
        r = rng.random()
        visible = [] if r < 0.5 else [rng.choice(RESONANCES)]
        power = rng.uniform(2.0, 8.0)

        card = SimCard(
            id=card_id, archetype=None,
            visible_symbols=visible, power=power,
            pair_affinity=0.0, bridge_affinity=0.0,
        )
        card.fitness = build_fitness(card, sibling_rates_map)
        cards.append(card)
        card_id += 1

    return cards


# ─── AI Drafter ──────────────────────────────────────────────────────────────

@dataclass
class AIDrafter:
    archetype: str
    phase_offset: int

    def evaluate_card(self, card):
        """Score for AI's archetype preference. Uses pair_affinity for own
        archetype, bridge_affinity for related archetypes."""
        if card.archetype == self.archetype:
            return 1.0 + card.pair_affinity * 0.5 + card.power / 10.0 * 0.3
        sib = SIBLING_PAIRS.get(self.archetype)
        if sib and card.archetype == sib:
            return 0.5 + card.bridge_affinity * 0.3 + card.power / 10.0 * 0.2
        if card.archetype and shares_resonance(self.archetype, card.archetype):
            return 0.2 + card.power / 10.0 * 0.2
        return card.power / 10.0 * 0.1

    def pick_cards(self, pool, pick_num, rng):
        num_cards, arch_focus = get_phase(pick_num, self.phase_offset)
        picked = []
        for _ in range(num_cards):
            if len(pool) <= POOL_FLOOR:
                break
            if rng.random() < arch_focus:
                best_idx = max(range(len(pool)), key=lambda i: self.evaluate_card(pool[i]))
            else:
                best_idx = max(range(len(pool)), key=lambda i: pool[i].power)
            picked.append(pool.pop(best_idx))
        return picked


# ─── Market Culling ──────────────────────────────────────────────────────────

def market_cull(pool, count):
    removable = len(pool) - POOL_FLOOR
    if removable <= 0:
        return []
    count = min(count, removable)
    pool.sort(key=lambda c: c.power)
    culled = pool[:count]
    del pool[:count]
    return culled


# ─── Player ──────────────────────────────────────────────────────────────────

TIER_VAL = {"S": 4, "A": 3, "B": 2, "C": 1, "F": 0}

def card_score(card, arch):
    return TIER_VAL[card.fitness.get(arch, "F")] * 2.5 + card.power / 10.0

def is_sa(card, arch):
    return card.fitness.get(arch, "F") in ("S", "A")


def pick_committed(pack, arch, picked, pick_num):
    if arch is None:
        best_card, best_score, best_arch = None, -1, None
        for card in pack:
            for a in ARCHETYPES:
                s = card_score(card, a)
                if s > best_score:
                    best_score, best_card, best_arch = s, card, a
        return best_card, best_arch
    return max(pack, key=lambda c: card_score(c, arch)), arch


def pick_power(pack, arch, picked, pick_num):
    card = max(pack, key=lambda c: c.power)
    if arch is None and pick_num >= 10 and picked:
        ac = defaultdict(int)
        for c in picked:
            if c.archetype:
                ac[c.archetype] += 1
        if ac:
            arch = max(ac, key=ac.get)
    return card, arch


def pick_signal(pack, arch, picked, pick_num, recent):
    if arch is None and pick_num <= 5:
        avail = defaultdict(float)
        for pp in recent:
            for c in pp:
                for a in ARCHETYPES:
                    if is_sa(c, a):
                        avail[a] += 1
        for c in pack:
            for a in ARCHETYPES:
                if is_sa(c, a):
                    avail[a] += 2
        if avail:
            arch = max(avail, key=avail.get)
        if arch:
            return max(pack, key=lambda c: card_score(c, arch)), arch
        return max(pack, key=lambda c: c.power), arch
    if arch is None:
        ac = defaultdict(int)
        for c in picked:
            if c.archetype:
                ac[c.archetype] += 1
        arch = max(ac, key=ac.get) if ac else ARCHETYPES[0]
    return max(pack, key=lambda c: card_score(c, arch)), arch


# ─── Metrics ─────────────────────────────────────────────────────────────────

def compute_metrics(log, arch, picked):
    m = {}
    # M1
    m1v = []
    for p in range(1, 6):
        if p in log:
            sa_archs = set()
            for c in log[p]["pack"]:
                for a in ARCHETYPES:
                    if is_sa(c, a):
                        sa_archs.add(a)
            m1v.append(len(sa_archs))
    m["M1"] = statistics.mean(m1v) if m1v else 0

    # M2
    m2v = []
    for p in range(1, 6):
        if p in log:
            best = max(sum(1 for c in log[p]["pack"] if is_sa(c, a)) for a in ARCHETYPES)
            m2v.append(best)
    m["M2"] = statistics.mean(m2v) if m2v else 0

    # M3
    m3v = []
    for p in range(6, NUM_PICKS + 1):
        if p in log and arch:
            m3v.append(sum(1 for c in log[p]["pack"] if is_sa(c, arch)))
    m["M3"] = statistics.mean(m3v) if m3v else 0

    # M4
    m4v = []
    for p in range(6, NUM_PICKS + 1):
        if p in log and arch:
            m4v.append(sum(1 for c in log[p]["pack"] if c.fitness.get(arch, "F") in ("C", "F")))
    m["M4"] = statistics.mean(m4v) if m4v else 0

    # M5
    if arch:
        run = []
        conv = NUM_PICKS
        for p in range(1, NUM_PICKS + 1):
            if p in log:
                run.append(sum(1 for c in log[p]["pack"] if is_sa(c, arch)))
                if len(run) >= 3 and statistics.mean(run[-3:]) >= 2.0:
                    conv = p - 2
                    break
        m["M5"] = conv
    else:
        m["M5"] = NUM_PICKS

    # M6
    if arch and picked:
        m["M6"] = sum(1 for c in picked if is_sa(c, arch)) / len(picked)
    else:
        m["M6"] = 0

    # M9
    m["M9"] = statistics.stdev(m3v) if len(m3v) > 1 else 0

    # M10
    mx, cur = 0, 0
    for v in m3v:
        if v < 1.5:
            cur += 1
            mx = max(mx, cur)
        else:
            cur = 0
    m["M10"] = mx

    # M11
    m11v = []
    for p in range(15, NUM_PICKS + 1):
        if p in log and arch:
            m11v.append(sum(1 for c in log[p]["pack"] if is_sa(c, arch)))
    m["M11"] = statistics.mean(m11v) if m11v else 0

    m["sa_per_pack_6plus"] = m3v
    m["sa_per_pack_15plus"] = m11v
    return m


# ─── Draft ───────────────────────────────────────────────────────────────────

def run_draft(rng, strategy="committed", sibling_rates_map=None, trace=False):
    if sibling_rates_map is None:
        sibling_rates_map = SIBLING_RATES

    pool = generate_pool(rng, sibling_rates_map)

    ai_archs = rng.sample(ARCHETYPES, NUM_AIS)
    open_arch = [a for a in ARCHETYPES if a not in ai_archs][0]

    ais = [AIDrafter(a, rng.choice([-1, 0, 1])) for a in ai_archs]

    arch = None
    picked = []
    log = {}
    recent = []
    ai_phase_picks = defaultdict(int)
    ai_arch_picks = defaultdict(int)
    trace_data = [] if trace else None

    for pn in range(1, NUM_PICKS + 1):
        if len(pool) <= 0:
            break

        # AI picks
        ai_total = 0
        if len(pool) > POOL_FLOOR:
            for ai in ais:
                if len(pool) <= POOL_FLOOR:
                    break
                removed = ai.pick_cards(pool, pn, rng)
                ai_total += len(removed)
                for c in removed:
                    if c.archetype:
                        ai_arch_picks[c.archetype] += 1

        # Phase tracking
        ph = 0
        for i, (lo, hi, _, _) in enumerate(PHASES):
            if lo <= pn <= hi:
                ph = i
                break
        if pn > PHASES[-2][1]:
            ph = len(PHASES) - 1
        ai_phase_picks[ph] += ai_total

        # Cull
        culled = 0
        if len(pool) > POOL_FLOOR + MARKET_CULL_COUNT:
            culled = len(market_cull(pool, MARKET_CULL_COUNT))

        # Pack
        if len(pool) <= 0:
            break
        ps = min(PACK_SIZE, len(pool))
        pack = rng.sample(pool, ps)
        log[pn] = {"pack": list(pack), "pool_size": len(pool)}

        # Player pick
        if strategy == "committed":
            if arch is None and pn >= 5 and picked:
                scores = defaultdict(float)
                for c in picked:
                    for a in ARCHETYPES:
                        if is_sa(c, a):
                            scores[a] += 1
                if scores:
                    arch = max(scores, key=scores.get)
            chosen, cand = pick_committed(pack, arch, picked, pn)
            if arch is None and pn >= 5:
                arch = cand
        elif strategy == "power":
            chosen, arch = pick_power(pack, arch, picked, pn)
        else:
            chosen, arch = pick_signal(pack, arch, picked, pn, recent)

        if chosen:
            picked.append(chosen)
            if chosen in pool:
                pool.remove(chosen)

        recent.append(pack)
        if len(recent) > 5:
            recent.pop(0)

        if trace:
            sa = sum(1 for c in pack if arch and is_sa(c, arch)) if arch else -1
            # Count remaining S/A cards in pool for player's archetype
            pool_sa = sum(1 for c in pool if arch and is_sa(c, arch)) if arch else -1
            trace_data.append({
                "pick": pn, "pool": len(pool), "ai": ai_total, "cull": culled,
                "sa": sa, "pool_sa": pool_sa,
                "arch": arch,
                "chosen": chosen.archetype if chosen else None,
                "pwr": round(chosen.power, 1) if chosen else 0,
                "tier": chosen.fitness.get(arch, "?") if (chosen and arch) else "?",
            })

    if arch is None and picked:
        ac = defaultdict(int)
        for c in picked:
            if c.archetype:
                ac[c.archetype] += 1
        if ac:
            arch = max(ac, key=ac.get)

    met = compute_metrics(log, arch, picked)
    met["player_arch"] = arch
    met["open_arch"] = open_arch
    met["ai_phase_picks"] = dict(ai_phase_picks)
    met["ai_arch_picks"] = dict(ai_arch_picks)
    met["picked_cards"] = picked
    met["in_open"] = (arch == open_arch)
    met["logged"] = len(log)
    if trace:
        met["trace"] = trace_data
    return met


# ─── Helpers ─────────────────────────────────────────────────────────────────

def pct(data, p):
    if not data:
        return 0
    k = (len(data) - 1) * p / 100
    f, c = math.floor(k), math.ceil(k)
    if f == c:
        return data[int(k)]
    return data[f] * (c - k) + data[c] * (k - f)


# ─── Main ────────────────────────────────────────────────────────────────────

def main():
    rng = random.Random(42)

    # First: verify baseline S/A density in a fresh pool
    test_pool = generate_pool(random.Random(0))
    for test_arch in ARCHETYPES:
        sa_count = sum(1 for c in test_pool if is_sa(c, test_arch))
        print(f"  Baseline S/A for {test_arch}: {sa_count}/360 = {sa_count/360*100:.1f}%")
    baseline_sa = statistics.mean(
        sum(1 for c in test_pool if is_sa(c, a)) for a in ARCHETYPES
    )
    print(f"  Average baseline S/A per archetype: {baseline_sa:.1f}/360")
    expected_pack = baseline_sa / 360 * 4
    print(f"  Expected S/A per 4-card pack (no contraction): {expected_pack:.2f}")
    print()

    strategies = ["committed", "power", "signal"]
    all_results = {}

    for strategy in strategies:
        print(f"Running {strategy} ({NUM_DRAFTS} drafts)... ", end="", flush=True)
        results = []
        for i in range(NUM_DRAFTS):
            do_trace = (i < 2 and strategy in ("committed", "signal"))
            results.append(run_draft(rng, strategy=strategy, trace=do_trace))
        all_results[strategy] = results
        print("done.")

    # Pessimistic
    print("Running pessimistic (500 drafts)... ", end="", flush=True)
    pess_rng = random.Random(99)
    pess_rates = {
        "Warriors": 0.40, "Sacrifice": 0.40,
        "Self-Discard": 0.30, "Self-Mill": 0.30,
        "Blink": 0.20, "Storm": 0.20,
        "Flash": 0.15, "Ramp": 0.15,
    }
    pess_results = []
    for _ in range(500):
        pess_results.append(run_draft(pess_rng, strategy="committed", sibling_rates_map=pess_rates))
    print("done.\n")

    # ─── Aggregation ─────────────────────────────────────────────────────────
    com = all_results["committed"]

    agg = {}
    for k in ["M1", "M2", "M3", "M4", "M5", "M6", "M9", "M10", "M11"]:
        agg[k] = statistics.mean([m[k] for m in com])

    # Variety
    overlaps = []
    for i in range(1, len(com)):
        p1 = set(c.id for c in com[i-1]["picked_cards"])
        p2 = set(c.id for c in com[i]["picked_cards"])
        if p1 and p2:
            overlaps.append(len(p1 & p2) / max(len(p1), len(p2)))
    m7 = statistics.mean(overlaps) if overlaps else 0

    ac = defaultdict(int)
    for m in com:
        if m["player_arch"]:
            ac[m["player_arch"]] += 1
    total_ac = sum(ac.values())
    m8_max = max(ac.values()) / total_ac if total_ac else 0
    m8_min = min(ac.values()) / total_ac if total_ac else 0

    # Per-arch M3
    pam3 = defaultdict(list)
    pam3_open = defaultdict(list)
    pam3_cont = defaultdict(list)
    for m in com:
        a = m["player_arch"]
        if a:
            pam3[a].append(m["M3"])
            if m["in_open"]:
                pam3_open[a].append(m["M3"])
            else:
                pam3_cont[a].append(m["M3"])

    # Pack dist
    all_sa = sorted(v for m in com for v in m["sa_per_pack_6plus"])

    # Consec
    consec = defaultdict(int)
    for m in com:
        consec[m["M10"]] += 1

    # AI behavior
    pp = defaultdict(list)
    ap = defaultdict(list)
    for m in com:
        for k, v in m["ai_phase_picks"].items():
            pp[k].append(v)
        for k, v in m["ai_arch_picks"].items():
            ap[k].append(v)

    avg_logged = statistics.mean([m["logged"] for m in com])
    open_count = sum(1 for m in com if m["in_open"])
    open_m3 = statistics.mean([m["M3"] for m in com if m["in_open"]]) if open_count else 0
    cont_m3 = statistics.mean([m["M3"] for m in com if not m["in_open"]]) if (len(com) - open_count) else 0

    # Pessimistic
    pa = {k: statistics.mean([m[k] for m in pess_results]) for k in ["M3", "M10", "M11"]}

    # ─── Print ───────────────────────────────────────────────────────────────
    print("=" * 72)
    print("D4 ESCALATING AGGRESSION — SIMULATION RESULTS")
    print("=" * 72)
    print(f"\n{NUM_DRAFTS} drafts x {NUM_PICKS} picks | Pool {POOL_SIZE} | "
          f"AIs {NUM_AIS} | Cull {MARKET_CULL_COUNT}/rnd | Floor {POOL_FLOOR}")
    print(f"Avg picks logged: {avg_logged:.1f}")
    print(f"Open-lane drafts: {open_count}/{len(com)} ({open_count/len(com)*100:.1f}%)")
    print(f"Open-lane M3: {open_m3:.2f} | Contested M3: {cont_m3:.2f}")

    print("\n--- FULL SCORECARD (Committed, Graduated Realistic) ---")
    rows = [
        ("M1",  f"{agg['M1']:.2f}",  ">= 3",   agg["M1"] >= 3),
        ("M2",  f"{agg['M2']:.2f}",  "<= 2",   agg["M2"] <= 2),
        ("M3",  f"{agg['M3']:.2f}",  ">= 2.0", agg["M3"] >= 2.0),
        ("M4",  f"{agg['M4']:.2f}",  ">= 0.5", agg["M4"] >= 0.5),
        ("M5",  f"{agg['M5']:.1f}",  "5-8",    5 <= agg["M5"] <= 8),
        ("M6",  f"{agg['M6']*100:.1f}%", "60-90%", 60 <= agg["M6"]*100 <= 90),
        ("M7",  f"{m7*100:.1f}%",    "< 40%",  m7 < 0.40),
        ("M8",  f"{m8_max*100:.1f}/{m8_min*100:.1f}%", "<20/>5%",
         m8_max < 0.20 and m8_min > 0.05),
        ("M9",  f"{agg['M9']:.2f}",  ">= 0.8", agg["M9"] >= 0.8),
        ("M10", f"{agg['M10']:.2f}", "<= 2",   agg["M10"] <= 2),
        ("M11", f"{agg['M11']:.2f}", ">= 3.0", agg["M11"] >= 3.0),
    ]
    passes = 0
    for name, val, tgt, ok in rows:
        s = "PASS" if ok else "FAIL"
        if ok:
            passes += 1
        print(f"  {name:<5s} {val:>12s}  target {tgt:<10s} {s}")
    print(f"\n  Result: {passes}/{len(rows)} pass")

    print("\n--- PESSIMISTIC (Sibling -10pp) ---")
    for k in ["M3", "M10", "M11"]:
        print(f"  {k}: {pa[k]:.2f}")

    print("\n--- PER-ARCHETYPE M3 ---")
    print(f"  {'Archetype':<15s} {'All':>6s} {'Open':>6s} {'Contest':>8s} {'N':>5s}")
    for a in ARCHETYPES:
        ov = statistics.mean(pam3[a]) if pam3[a] else 0
        op = statistics.mean(pam3_open[a]) if pam3_open.get(a) else 0
        ct = statistics.mean(pam3_cont[a]) if pam3_cont.get(a) else 0
        print(f"  {a:<15s} {ov:>6.2f} {op:>6.2f} {ct:>8.2f} {len(pam3[a]):>5d}")

    print("\n--- PACK QUALITY (Picks 6+, S/A count) ---")
    if all_sa:
        for p in [10, 25, 50, 75, 90]:
            print(f"  p{p}: {pct(all_sa, p):.2f}")

    print("\n--- CONSECUTIVE BAD PACKS ---")
    for s in sorted(consec):
        print(f"  {s:2d}: {consec[s]:4d} ({consec[s]/len(com)*100:.1f}%)")

    print("\n--- AI BEHAVIOR ---")
    plabels = ["Phase 1 (1-5)", "Phase 2 (6-10)", "Phase 3 (11-15)", "Phase 4 (16+)"]
    for i in range(4):
        if i in pp:
            avg = statistics.mean(pp[i])
            per = avg / (5 if i < 3 else 15)
            print(f"  {plabels[i]}: {avg:.0f} total ({per:.1f}/pick)")
        else:
            print(f"  {plabels[i]}: 0 (pool at floor)")

    print("\n  Per-archetype AI removal (avg):")
    for a in ARCHETYPES:
        print(f"    {a:<15s}: {statistics.mean(ap[a]) if a in ap else 0:.1f}")

    print("\n--- STRATEGY COMPARISON ---")
    for st in strategies:
        r = all_results[st]
        print(f"  {st:<12s}: M3={statistics.mean([x['M3'] for x in r]):.2f}  "
              f"M6={statistics.mean([x['M6'] for x in r])*100:.1f}%  "
              f"M10={statistics.mean([x['M10'] for x in r]):.2f}  "
              f"M11={statistics.mean([x['M11'] for x in r]):.2f}")

    # Traces
    for label, st in [("COMMITTED", "committed"), ("SIGNAL READER", "signal")]:
        for m in all_results[st]:
            if "trace" in m:
                print(f"\n--- TRACE: {label} ---")
                print(f"  Arch: {m['player_arch']} | Open: {m['open_arch']} | "
                      f"In open: {m['in_open']}")
                print(f"  {'Pk':>3s} {'Pool':>5s} {'AI':>4s} {'Cl':>3s} {'SA':>3s} "
                      f"{'PoolSA':>6s} {'Chosen':>14s} {'Pwr':>5s} {'T':>2s}")
                for t in m["trace"]:
                    sa_s = str(t["sa"]) if t["sa"] >= 0 else "—"
                    psa_s = str(t["pool_sa"]) if t["pool_sa"] >= 0 else "—"
                    print(f"  {t['pick']:3d} {t['pool']:5d} {t['ai']:4d} "
                          f"{t['cull']:3d} {sa_s:>3s} {psa_s:>6s} "
                          f"{str(t['chosen']):>14s} {t['pwr']:5.1f} {t['tier']:>2s}")
                break

    # V9 comparison
    print("\n--- V9 COMPARISON ---")
    v9 = {"M3": 2.70, "M5": 9.6, "M10": 3.8, "M11": 3.25}
    print(f"  {'Met':<5s} {'D4':>8s} {'V9':>8s} {'Target':>8s}")
    for k, tgt in [("M3", ">= 2.0"), ("M5", "5-8"), ("M10", "<= 2"), ("M11", ">= 3.0")]:
        d4v = agg[k]
        v9v = v9.get(k, "—")
        if k == "M5":
            print(f"  {k:<5s} {d4v:>8.1f} {v9v:>8.1f} {tgt:>8s}")
        else:
            print(f"  {k:<5s} {d4v:>8.2f} {v9v:>8.2f} {tgt:>8s}")

    print("\n" + "=" * 72)


if __name__ == "__main__":
    main()
