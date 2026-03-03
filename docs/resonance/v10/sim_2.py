#!/usr/bin/env python3
"""
Simulation Agent 2: Hybrid X (Open Table + Saturation)
V10 Resonance Draft System — Monte Carlo Simulation

Algorithm: D1 Open Table + D3 Saturation
 - 5 AIs randomly from 8 archetypes, 3 open lanes
 - AI picks: 4 cards/round using pair-affinity (pre-sat 85/5/10, post-sat 50/30/20)
 - Saturation at 16 archetype cards
 - Market culling: removes lowest-power cards per round (V9-like contraction)
 - Level 0 reactivity

Key concentration mechanism: AI picks directionally deplete contested lanes
while market culling removes low-quality filler. The combination of AI directional
depletion + general power culling concentrates the pool toward high-quality
open-lane cards over time.

1000 drafts x 30 picks x 3 player strategies
"""

import random
import statistics
from dataclasses import dataclass, field
from collections import defaultdict
from typing import Optional

# ── Constants ────────────────────────────────────────────────────────────────

NUM_DRAFTS = 1000
NUM_PICKS = 30
PACK_SIZE = 4
POOL_SIZE = 360
CARDS_PER_ARCHETYPE = 40
NUM_GENERIC = 40
NUM_AIS = 5
NUM_ARCHETYPES = 8
SATURATION_THRESHOLD = 16
POOL_MIN = 20

# Contraction rate per pick (V9 equivalent)
CONTRACTION_RATE = 0.12

PRE_SAT_ARCH = 0.85
PRE_SAT_ADJ = 0.05
PRE_SAT_GEN = 0.10
POST_SAT_ARCH = 0.50
POST_SAT_ADJ = 0.30
POST_SAT_GEN = 0.20

SEED = 42

ARCHETYPES = ["Flash", "Blink", "Storm", "Self-Discard",
              "Self-Mill", "Sacrifice", "Warriors", "Ramp"]
ARCH_RES = {
    0: ("Zephyr", "Ember"), 1: ("Ember", "Zephyr"),
    2: ("Ember", "Stone"),  3: ("Stone", "Ember"),
    4: ("Stone", "Tide"),   5: ("Tide", "Stone"),
    6: ("Tide", "Zephyr"),  7: ("Zephyr", "Tide"),
}
SIBLING = {0: 7, 7: 0, 1: 2, 2: 1, 3: 4, 4: 3, 5: 6, 6: 5}
SIB_RATES = {
    (6,5): 0.50, (5,6): 0.50, (3,4): 0.40, (4,3): 0.40,
    (1,2): 0.30, (2,1): 0.30, (0,7): 0.25, (7,0): 0.25,
}
PESS_SIB_RATES = {k: max(0.0, v - 0.10) for k, v in SIB_RATES.items()}

def _adj(a):
    p, s = ARCH_RES[a]
    return [o for o in range(NUM_ARCHETYPES) if o != a and
            (p in ARCH_RES[o] or s in ARCH_RES[o])]
ADJ = {i: _adj(i) for i in range(NUM_ARCHETYPES)}

# ── Cards ────────────────────────────────────────────────────────────────────

@dataclass
class Card:
    id: int
    archetype: Optional[int]
    vis: list
    power: float
    aff: dict  # archetype -> float
    sib_fit: dict  # archetype -> bool
    sib_fit_p: dict  # pessimistic

def gen_pool(rng):
    cards = []
    cid = 0
    duals = [4,4,5,5,4,4,5,5]
    for arch in range(NUM_ARCHETYPES):
        p, s = ARCH_RES[arch]
        sb = SIBLING.get(arch)
        for i in range(CARDS_PER_ARCHETYPE):
            pw = rng.uniform(1, 10)
            vis = [p, s] if i < duals[arch] else [p]
            af = {}
            for a in range(NUM_ARCHETYPES):
                if a == arch: af[a] = rng.uniform(0.70, 1.0)
                elif sb is not None and a == sb: af[a] = rng.uniform(0.35, 0.65)
                elif a in ADJ[arch]: af[a] = rng.uniform(0.10, 0.30)
                else: af[a] = rng.uniform(0, 0.12)
            sf, sfp = {}, {}
            for a in range(NUM_ARCHETYPES):
                soa = SIBLING.get(a)
                if soa == arch:
                    r = rng.random()
                    sf[a] = r < SIB_RATES.get((a, arch), 0)
                    sfp[a] = r < PESS_SIB_RATES.get((a, arch), 0)
                else:
                    sf[a] = sfp[a] = False
            cards.append(Card(cid, arch, vis, pw, af, sf, sfp))
            cid += 1
    for _ in range(NUM_GENERIC):
        pw = rng.uniform(1, 10)
        af = {a: rng.uniform(0.10, 0.25) for a in range(NUM_ARCHETYPES)}
        sf = {a: False for a in range(NUM_ARCHETYPES)}
        cards.append(Card(cid, None, [], pw, af, sf, sf))
        cid += 1
    return cards

# ── Fitness ──────────────────────────────────────────────────────────────────

def is_sa(c, pa, pess=False):
    if c.archetype is None: return False
    if c.archetype == pa: return True
    sb = SIBLING.get(pa)
    if c.archetype == sb:
        return (c.sib_fit_p if pess else c.sib_fit).get(pa, False)
    return False

def cnt_sa(pack, pa, pess=False):
    return sum(1 for c in pack if is_sa(c, pa, pess))

def cnt_off(pack, pa):
    return sum(1 for c in pack if not is_sa(c, pa))

def pscore(c, pa):
    if is_sa(c, pa): return 100 + c.power
    if c.archetype is not None and c.archetype in ADJ[pa]: return 30 + c.power
    if c.archetype is None: return 20 + c.power
    return c.power

# ── AI ───────────────────────────────────────────────────────────────────────

class AI:
    def __init__(self, arch):
        self.arch = arch
        self.ac = 0  # archetype count
    @property
    def sat(self): return self.ac >= SATURATION_THRESHOLD
    def pick(self, pids, pbi, n, rng):
        picked = []
        av = set(pids)
        for _ in range(n):
            if not av: break
            ap = POST_SAT_ARCH if self.sat else PRE_SAT_ARCH
            adjp = POST_SAT_ADJ if self.sat else PRE_SAT_ADJ
            r = rng.random()
            if r < ap:
                c = max(av, key=lambda x: pbi[x].aff.get(self.arch, 0))
            elif r < ap + adjp:
                adj = ADJ[self.arch]
                if adj:
                    t = rng.choice(adj)
                    cds = [x for x in av if pbi[x].archetype is not None and pbi[x].archetype in adj]
                    c = max(cds, key=lambda x: pbi[x].aff.get(t, 0)) if cds else max(av, key=lambda x: pbi[x].power)
                else:
                    c = max(av, key=lambda x: pbi[x].power)
            else:
                c = max(av, key=lambda x: pbi[x].power)
            picked.append(c)
            av.discard(c)
            if pbi[c].archetype == self.arch: self.ac += 1
        return picked

# ── Draft ────────────────────────────────────────────────────────────────────

@dataclass
class Result:
    strat: str
    ca: Optional[int]
    conv: int
    drafted: list
    sa6: list
    sa6p: list
    off6: list
    m1v: list
    m2v: list
    ai_archs: list
    ai_sat: list

def run_draft(rng, strat="committed", trace=False):
    pool = gen_pool(rng)
    pbi = {c.id: c for c in pool}
    pids = set(c.id for c in pool)
    ai_archs = sorted(rng.sample(range(NUM_ARCHETYPES), NUM_AIS))
    ais = [AI(a) for a in ai_archs]
    drafted, ca, conv = [], None, NUM_PICKS
    sa6, sa6p, off6, m1v, m2v = [], [], [], [], []
    ai_sat = [None] * NUM_AIS
    tlog = [] if trace else None
    seen_packs = []

    for pn in range(NUM_PICKS):
        pool_sz = len(pids)
        if pool_sz < PACK_SIZE + 4: break

        # Compute total removal for this round: pool_sz * CONTRACTION_RATE
        total_remove = max(2, int(pool_sz * CONTRACTION_RATE))

        # AI picks: each AI gets a share. Target ~65% of removal from AIs.
        ai_budget = max(0, int(total_remove * 0.65))
        ai_per = min(4, max(0, ai_budget // NUM_AIS))
        actual_ai = ai_per * NUM_AIS

        # Remaining removal from culling
        cull_n = max(0, total_remove - actual_ai)

        # Safety: don't over-deplete
        if actual_ai + cull_n + 1 > pool_sz - POOL_MIN:
            excess = actual_ai + cull_n + 1 - (pool_sz - POOL_MIN)
            cull_n = max(0, cull_n - excess)
            if cull_n < 0:
                ai_per = max(0, ai_per - 1)
                actual_ai = ai_per * NUM_AIS

        # AI picks
        for idx, ai in enumerate(ais):
            ws = ai.sat
            picked = ai.pick(pids, pbi, ai_per, rng)
            pids -= set(picked)
            if not ws and ai.sat and ai_sat[idx] is None:
                ai_sat[idx] = pn + 1

        # Market culling: remove cards least relevant to the open archetypes.
        # For each card, compute max pair_affinity across the 3 open archetypes.
        # Cards with lowest scores are culled first.
        # This is Level 0: based on which AIs are present, not player behavior.
        # Narrative: "cards that nobody at the open seats wants get cleared."
        if cull_n > 0 and len(pids) > POOL_MIN + cull_n:
            open_archs = [a for a in range(NUM_ARCHETYPES) if a not in ai_archs]
            def open_relevance(cid):
                c = pbi[cid]
                return max(c.aff.get(a, 0) for a in open_archs) if open_archs else 0
            rem = sorted(pids, key=open_relevance)
            pids -= set(rem[:cull_n])

        if len(pids) < PACK_SIZE: break
        pack = [pbi[c] for c in rng.sample(list(pids), PACK_SIZE)]
        seen_packs.append(pack)

        # Player pick
        nc = None
        if strat == "committed":
            if ca is not None:
                chosen = max(pack, key=lambda c: pscore(c, ca))
            elif pn >= 5:
                # Commit: count S/A across all SEEN PACKS for each archetype
                asc = defaultdict(int)
                for pk in seen_packs:
                    for c in pk:
                        for a in range(NUM_ARCHETYPES):
                            if is_sa(c, a): asc[a] += 1
                nc = max(asc, key=asc.get) if asc else 0
                chosen = max(pack, key=lambda c: pscore(c, nc))
            else:
                chosen = max(pack, key=lambda c: c.power + max(c.aff.values()) * 3)
        elif strat == "power":
            chosen = max(pack, key=lambda c: c.power)
        elif strat == "signal":
            if ca is not None:
                chosen = max(pack, key=lambda c: pscore(c, ca))
            elif pn >= 5:
                avails = {a: sum(1 for c in pids if is_sa(pbi[c], a)) for a in range(NUM_ARCHETYPES)}
                nc = max(avails, key=avails.get)
                chosen = max(pack, key=lambda c: pscore(c, nc))
            else:
                chosen = max(pack, key=lambda c: c.power)

        if nc is not None and ca is None: ca = nc; conv = pn + 1
        drafted.append(chosen)
        pids.discard(chosen.id)

        if pn < 5:
            arcs = set()
            for c in pack:
                for a in range(NUM_ARCHETYPES):
                    if is_sa(c, a): arcs.add(a)
            m1v.append(len(arcs))
            m2v.append(max(cnt_sa(pack, a) for a in range(NUM_ARCHETYPES)))

        if ca is not None:
            sa6.append(cnt_sa(pack, ca)) if pn >= 5 else None
            sa6p.append(cnt_sa(pack, ca, True)) if pn >= 5 else None
            off6.append(cnt_off(pack, ca)) if pn >= 5 else None

        if trace:
            tlog.append({
                'pk': pn+1, 'pool': len(pids),
                'sa': cnt_sa(pack, ca) if ca is not None else -1,
                'ai': ai_per, 'cul': cull_n,
                'ch': ARCHETYPES[chosen.archetype] if chosen.archetype is not None else "Generic",
                'ca': ARCHETYPES[ca] if ca is not None else "---",
                'ns': sum(1 for a in ais if a.sat),
            })

    if strat == "power" and ca is None:
        ac = defaultdict(int)
        for c in drafted:
            if c.archetype is not None: ac[c.archetype] += 1
        ca = max(ac, key=ac.get) if ac else 0
        conv = NUM_PICKS

    r = Result(strat, ca, conv, drafted, sa6, sa6p, off6, m1v, m2v, ai_archs, ai_sat)
    if trace: r.trace_log = tlog
    return r

# ── Metrics ──────────────────────────────────────────────────────────────────

def metrics(results, sn):
    rs = [r for r in results if r.strat == sn]
    if not rs: return {}
    m = {}
    m['M1'] = statistics.mean([v for r in rs for v in r.m1v]) or 0
    m['M2'] = statistics.mean([v for r in rs for v in r.m2v]) or 0
    a6 = [v for r in rs for v in r.sa6]
    m['M3'] = statistics.mean(a6) if a6 else 0
    a6p = [v for r in rs for v in r.sa6p]
    m['M3p'] = statistics.mean(a6p) if a6p else 0
    ao = [v for r in rs for v in r.off6]
    m['M4'] = statistics.mean(ao) if ao else 0
    m['M5'] = statistics.mean([r.conv for r in rs])
    concs = []
    for r in rs:
        if r.ca is not None and r.drafted:
            n = sum(1 for c in r.drafted if is_sa(c, r.ca))
            concs.append(n / len(r.drafted) * 100)
    m['M6'] = statistics.mean(concs) if concs else 0
    ad = defaultdict(list)
    for r in rs:
        if r.ca is not None: ad[r.ca].append(set(c.id for c in r.drafted))
    ovl = []
    sr = random.Random(12345)
    for a, sets in ad.items():
        if len(sets) >= 2:
            for _ in range(min(200, len(sets)*(len(sets)-1)//2)):
                i, j = sr.sample(range(len(sets)), 2)
                if sets[i] and sets[j]:
                    ovl.append(len(sets[i]&sets[j])/max(len(sets[i]),len(sets[j]))*100)
    m['M7'] = statistics.mean(ovl) if ovl else 0
    af = defaultdict(int); tc = 0
    for r in rs:
        if r.ca is not None: af[r.ca] += 1; tc += 1
    fp = {ARCHETYPES[a]: n/tc*100 for a, n in af.items()} if tc else {}
    m['M8max'] = max(fp.values()) if fp else 0
    m['M8min'] = min(fp.values()) if len(fp) == NUM_ARCHETYPES else 0
    m['M8f'] = fp
    m['M9'] = statistics.stdev(a6) if len(a6) > 1 else 0
    cl = []
    for r in rs:
        if r.sa6:
            mx, cur = 0, 0
            for s in r.sa6:
                if s < 1.5: cur += 1; mx = max(mx, cur)
                else: cur = 0
            cl.append(mx)
    m['M10'] = statistics.mean(cl) if cl else 0
    clp = []
    for r in rs:
        if r.sa6p:
            mx, cur = 0, 0
            for s in r.sa6p:
                if s < 1.5: cur += 1; mx = max(mx, cur)
                else: cur = 0
            clp.append(mx)
    m['M10p'] = statistics.mean(clp) if clp else 0
    late, latep = [], []
    for r in rs:
        if r.ca is not None:
            for i, s in enumerate(r.sa6):
                if i + 6 >= 14: late.append(s)
            for i, s in enumerate(r.sa6p):
                if i + 6 >= 14: latep.append(s)
    m['M11'] = statistics.mean(late) if late else 0
    m['M11p'] = statistics.mean(latep) if latep else 0
    am3 = {}
    for a in range(NUM_ARCHETYPES):
        asa = [v for r in rs if r.ca == a for v in r.sa6]
        am3[ARCHETYPES[a]] = statistics.mean(asa) if asa else 0
    m['am3'] = am3
    if a6:
        s = sorted(a6); n = len(s)
        m['pd'] = {'p10': s[int(n*0.10)], 'p25': s[int(n*0.25)],
                   'p50': s[int(n*0.50)], 'p75': s[int(n*0.75)],
                   'p90': s[min(int(n*0.90), n-1)]}
    else:
        m['pd'] = {'p10':0,'p25':0,'p50':0,'p75':0,'p90':0}
    if cl:
        m['cd'] = {str(k): sum(1 for x in cl if x == k)/len(cl)*100 for k in range(5)}
        m['cd']['4+'] = sum(1 for x in cl if x >= 4)/len(cl)*100
    else:
        m['cd'] = {}
    sp = []; nv = 0; tot = 0
    for r in rs:
        for s in r.ai_sat:
            tot += 1
            if s is not None: sp.append(s)
            else: nv += 1
    m['avg_sat'] = statistics.mean(sp) if sp else 0
    m['pct_nsat'] = (nv/tot*100) if tot else 0
    if sp:
        ss = sorted(sp); n = len(ss)
        m['sd'] = {'p10': ss[int(n*0.10)], 'p25': ss[int(n*0.25)],
                   'p50': ss[int(n*0.50)], 'p75': ss[int(n*0.75)],
                   'p90': ss[min(int(n*0.90), n-1)]}
    else:
        m['sd'] = {}
    return m

# ── Main ─────────────────────────────────────────────────────────────────────

def main():
    rng = random.Random(SEED)
    print("="*70)
    print("Hybrid X (Open Table + Saturation) — V10 Sim")
    print("="*70)
    print(f"Drafts={NUM_DRAFTS} Picks={NUM_PICKS} AIs={NUM_AIS}/{NUM_ARCHETYPES}")
    print(f"Sat={SATURATION_THRESHOLD} Contraction={CONTRACTION_RATE:.0%}")
    print()

    all_res = []
    for st in ["committed", "power", "signal"]:
        print(f"{st} ({NUM_DRAFTS})...", flush=True)
        sr = random.Random(rng.randint(0, 2**32))
        for i in range(NUM_DRAFTS):
            all_res.append(run_draft(sr, strat=st))
            if (i+1) % 500 == 0: print(f"  {i+1}", flush=True)

    t1 = run_draft(random.Random(SEED+1001), "committed", trace=True)
    t2 = run_draft(random.Random(SEED+2002), "signal", trace=True)

    for st in ["committed", "power", "signal"]:
        m = metrics(all_res, st)
        print(f"\n{'─'*60}\n {st.upper()}\n{'─'*60}")
        for k, t in [('M1','>=3'),('M2','<=2'),('M3','>=2.0'),('M3p','pess'),
                     ('M4','>=0.5'),('M5','5-8'),('M6','60-90%'),('M7','<40%'),
                     ('M9','>=0.8'),('M10','<=2'),('M10p','pess'),
                     ('M11','>=3.0'),('M11p','pess')]:
            v = m[k]; sf = '%' if k in ('M6','M7') else ''
            print(f"  {k:5s} {v:.2f}{sf}  [{t}]")
        print(f"  M8    max={m['M8max']:.1f}% min={m['M8min']:.1f}%")
        if st == "committed":
            print(f"\n  Per-Arch M3:")
            for nm, v in sorted(m['am3'].items()):
                print(f"    {nm:15s}: {v:.2f}")
            pd = m['pd']
            print(f"\n  PackDist: p10={pd['p10']:.0f} p25={pd['p25']:.0f} p50={pd['p50']:.0f} p75={pd['p75']:.0f} p90={pd['p90']:.0f}")
            print(f"\n  ConsecBad:")
            for k in ['0','1','2','3','4+']:
                if k in m['cd']: print(f"    {k}: {m['cd'][k]:.1f}%")
            print(f"\n  Saturation: avg={m['avg_sat']:.1f} never={m['pct_nsat']:.1f}%")
            if m['sd']:
                sd = m['sd']
                print(f"    p10={sd['p10']} p25={sd['p25']} p50={sd['p50']} p75={sd['p75']} p90={sd['p90']}")
            print(f"\n  ArchFreq:")
            for nm, f in sorted(m['M8f'].items(), key=lambda x:-x[1]):
                print(f"    {nm:15s}: {f:.1f}%")

    for lb, t in [("COMMITTED", t1), ("SIGNAL", t2)]:
        print(f"\n{'='*70}\nTRACE: {lb}\n{'='*70}")
        print(f"AI: {', '.join(ARCHETYPES[a] for a in t.ai_archs)}")
        print(f"Open: {', '.join(ARCHETYPES[a] for a in range(NUM_ARCHETYPES) if a not in t.ai_archs)}")
        print(f"Committed: {ARCHETYPES[t.ca] if t.ca is not None else '---'} @ {t.conv}")
        if hasattr(t, 'trace_log'):
            print(f"{'Pk':>3} {'Pl':>4} {'SA':>3} {'AI':>3} {'Cu':>3} {'Chosen':>15} {'CA':>15} {'#S':>3}")
            for e in t.trace_log:
                sa = str(e['sa']) if e['sa'] >= 0 else "---"
                print(f"{e['pk']:3d} {e['pool']:4d} {sa:>3} {e['ai']:3d} {e['cul']:3d} {e['ch']:>15} {e['ca']:>15} {e['ns']:3d}")

    print(f"\n{'='*70}\nCOMPLETE")

if __name__ == "__main__":
    main()
