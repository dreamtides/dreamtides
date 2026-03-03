#!/usr/bin/env python3
"""
Simulation Agent 5: D2 Sentinel Draft — Monte Carlo Simulation

Two variants:
  1. Sentinel (reactive): Phase 1 predetermined (picks 1 to staggered 8/9/10),
     Phase 2 pool-responsive (staggered start to pick 30).
  2. Level 0 control: Same 6 AIs, same culling, but Phase 2 continues with
     predetermined picks (no re-evaluation). Isolates reactivity's contribution.

Pool: 360 cards (40 per archetype x 8 + 40 generic)
6 AIs assigned to 6 of 8 archetypes randomly (2 lanes open)
Pack size: 4, 30 picks, 1000 drafts x 3 player strategies
5% supplemental culling per round

NOTE: The design specifies 6 AIs x 4 cards/round = 24 AI picks + 5% cull.
This exhausts the 360-card pool in ~10-12 rounds. The simulation faithfully
models this. When the pool falls below the minimum viable size, remaining
picks receive from whatever is left. This is a structural finding about the
D2 design's pool math.
"""

import random
import math
from dataclasses import dataclass, field
from typing import Optional
from collections import defaultdict

# ── Constants ──────────────────────────────────────────────────────────────

NUM_DRAFTS = 1000
NUM_PICKS = 30
PACK_SIZE = 4
POOL_SIZE = 360
CARDS_PER_ARCHETYPE = 40
GENERIC_CARDS = 40
NUM_AIS = 6
AI_PICKS_PER_ROUND = 4
CULL_RATE = 0.05
NUM_ARCHETYPES = 8

ARCHETYPES = [
    "Flash",        # 0: Zephyr/Ember
    "Blink",        # 1: Ember/Zephyr
    "Storm",        # 2: Ember/Stone
    "Self-Discard",  # 3: Stone/Ember
    "Self-Mill",    # 4: Stone/Tide
    "Sacrifice",    # 5: Tide/Stone
    "Warriors",     # 6: Tide/Zephyr
    "Ramp",         # 7: Zephyr/Tide
]

RESONANCES = ["Zephyr", "Ember", "Stone", "Tide"]

# Primary/secondary resonance for each archetype
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

# Sibling pairs: archetypes that share both resonances (neighbors on the circle)
# Each archetype has TWO neighbors. The sibling rate is per-pair.
SIBLING_PAIRS = {
    ("Flash", "Blink"): 0.30,       # share Zephyr+Ember
    ("Blink", "Storm"): 0.30,       # share Ember
    ("Storm", "Self-Discard"): 0.40,  # share Stone+Ember
    ("Self-Discard", "Self-Mill"): 0.40,  # share Stone
    ("Self-Mill", "Sacrifice"): 0.50,  # share Stone+Tide
    ("Sacrifice", "Warriors"): 0.50,  # share Tide
    ("Warriors", "Ramp"): 0.25,      # share Tide+Zephyr
    ("Ramp", "Flash"): 0.25,         # share Zephyr
}

# Fitness rates from spec (used for sibling A-tier probability)
SIBLING_RATES = {
    ("Warriors", "Sacrifice"): 0.50,
    ("Self-Discard", "Self-Mill"): 0.40,
    ("Blink", "Storm"): 0.30,
    ("Flash", "Ramp"): 0.25,
}


def get_sibling_rate(arch):
    """Get the sibling A-tier rate for an archetype's pair."""
    for (a, b), rate in SIBLING_RATES.items():
        if arch == a or arch == b:
            return rate
    return 0.30


def get_sibling(archetype):
    """Get the sibling archetype (same pair in SIBLING_RATES)."""
    for (a, b) in SIBLING_RATES:
        if archetype == a:
            return b
        if archetype == b:
            return a
    return None


def shares_resonance(arch_a, arch_b):
    """Check if two archetypes share at least one resonance."""
    res_a = set(ARCHETYPE_RESONANCES[arch_a])
    res_b = set(ARCHETYPE_RESONANCES[arch_b])
    return bool(res_a & res_b)


# ── Card Model ─────────────────────────────────────────────────────────────

@dataclass
class SimCard:
    id: int
    visible_symbols: list
    archetype: Optional[str]
    power: float
    # Pair affinity: how well this card fits its own archetype (0-1)
    pair_affinity: float
    # Cross affinity: how well it fits the sibling archetype (0-1)
    cross_affinity: float

    def __hash__(self):
        return self.id

    def __eq__(self, other):
        return isinstance(other, SimCard) and self.id == other.id

    def affinity_for(self, target_archetype):
        """How much an AI drafter targeting target_archetype wants this card.
        Returns a score 0-1 used for AI pick ordering."""
        if self.archetype is None:
            # Generic: mildly desirable based on power
            return 0.05 + self.power * 0.02

        if self.archetype == target_archetype:
            # Home card: very high affinity
            return 0.5 + 0.5 * self.pair_affinity

        # Sibling archetype
        sibling = get_sibling(target_archetype)
        if self.archetype == sibling:
            return 0.2 + 0.3 * self.cross_affinity

        # Adjacent (shares one resonance)
        if shares_resonance(self.archetype, target_archetype):
            return 0.1 + 0.15 * self.cross_affinity

        # Unrelated
        return 0.02 + self.power * 0.01


def fitness_tier(card, target_archetype):
    """Classify card fitness: S, A, B, C, or F for target archetype."""
    if card.archetype is None:
        # Generic cards: C-tier (B if very high power)
        return "B" if card.power >= 8 else "C"

    if card.archetype == target_archetype:
        # Home cards: distributed by pair_affinity
        # Top ~30% = S, next ~30% = A, rest = B
        if card.pair_affinity >= 0.70:
            return "S"
        elif card.pair_affinity >= 0.35:
            return "A"
        else:
            return "B"

    # Sibling archetype
    sibling = get_sibling(target_archetype)
    if card.archetype == sibling:
        rate = get_sibling_rate(target_archetype)
        # rate = probability of being A-tier from sibling
        if card.cross_affinity >= (1.0 - rate):
            return "A"
        elif card.cross_affinity >= (1.0 - rate * 2):
            return "B"
        else:
            return "C"

    # Adjacent archetype (shares a resonance)
    if shares_resonance(card.archetype, target_archetype):
        if card.power >= 8 and card.cross_affinity >= 0.8:
            return "B"
        return "C"

    # No relationship
    return "F"


def is_sa_tier(card, target_archetype):
    return fitness_tier(card, target_archetype) in ("S", "A")


# ── Pool Generation ────────────────────────────────────────────────────────

def generate_pool(rng):
    """Generate 360 cards: 40 per archetype + 40 generic."""
    cards = []
    cid = 0

    for arch in ARCHETYPES:
        primary_res, secondary_res = ARCHETYPE_RESONANCES[arch]
        for _ in range(CARDS_PER_ARCHETYPE):
            # Visible symbols
            roll = rng.random()
            if roll < 0.11:  # ~11% dual-symbol among archetype cards
                visible = [primary_res, secondary_res]
            elif roll < 0.55:
                visible = [primary_res]
            else:
                visible = [secondary_res]

            cards.append(SimCard(
                id=cid,
                visible_symbols=visible,
                archetype=arch,
                power=round(rng.uniform(2.0, 9.5), 1),
                pair_affinity=round(rng.random(), 3),
                cross_affinity=round(rng.random(), 3),
            ))
            cid += 1

    # 40 generic cards
    for _ in range(GENERIC_CARDS):
        cards.append(SimCard(
            id=cid,
            visible_symbols=[],
            archetype=None,
            power=round(rng.uniform(1.0, 7.0), 1),
            pair_affinity=round(rng.random(), 3),
            cross_affinity=round(rng.random(), 3),
        ))
        cid += 1

    return cards


# ── AI Drafter ─────────────────────────────────────────────────────────────

class AIDrafter:
    def __init__(self, archetype, phase2_start_pick, pool):
        self.archetype = archetype
        self.phase2_start_pick = phase2_start_pick
        # Generate predetermined list sorted by affinity descending
        scored = sorted(pool, key=lambda c: (-c.affinity_for(archetype), -c.power))
        self.predetermined_list = scored

    def pick_predetermined(self, pool_set, count):
        """Pick from predetermined list, skip cards already taken."""
        picks = []
        for card in self.predetermined_list:
            if card in pool_set:
                picks.append(card)
                if len(picks) >= count:
                    break
        return picks

    def pick_reactive(self, pool_list, count):
        """Re-evaluate remaining pool and take top cards by affinity."""
        scored = sorted(pool_list, key=lambda c: (-c.affinity_for(self.archetype), -c.power))
        return scored[:count]


def supplemental_cull(pool_list, ai_archetypes, cull_rate):
    """Remove bottom cull_rate fraction of pool by average AI desirability."""
    if not pool_list:
        return pool_list
    n_cull = max(1, int(len(pool_list) * cull_rate))

    def desirability(card):
        return sum(card.affinity_for(a) for a in ai_archetypes) / len(ai_archetypes) if ai_archetypes else card.power

    pool_sorted = sorted(pool_list, key=desirability)
    to_remove = set(id(c) for c in pool_sorted[:n_cull])
    return [c for c in pool_list if id(c) not in to_remove]


# ── Player Strategies ──────────────────────────────────────────────────────

def player_committed(pack, pick_num, state, rng):
    """Committed: explore early, commit at pick 5-6 to strongest archetype."""
    if not pack:
        return None
    if pick_num <= 4:
        # Track archetype signals from what we pick
        best = max(pack, key=lambda c: c.power + (3 if any(is_sa_tier(c, a) for a in ARCHETYPES) else 0))
        if best.archetype:
            state["signals"][best.archetype] = state["signals"].get(best.archetype, 0) + 1
        return best
    else:
        if "arch" not in state:
            # Commit to archetype with most signals
            if state["signals"]:
                state["arch"] = max(state["signals"], key=state["signals"].get)
            else:
                state["arch"] = rng.choice(ARCHETYPES)
        arch = state["arch"]
        # Pick best S/A card; fallback to highest power
        sa_cards = [c for c in pack if is_sa_tier(c, arch)]
        if sa_cards:
            return max(sa_cards, key=lambda c: c.power)
        return max(pack, key=lambda c: c.power)


def player_power_chaser(pack, pick_num, state, rng):
    """Always pick highest power card."""
    if not pack:
        return None
    return max(pack, key=lambda c: c.power)


def player_signal_reader(pack, pick_num, state, rng):
    """Read signals: track what's available, commit to most-seen archetype by pick 5."""
    if not pack:
        return None
    # Always track what appears
    for card in pack:
        if card.archetype:
            state["seen"][card.archetype] = state["seen"].get(card.archetype, 0) + 1

    if pick_num <= 4:
        return max(pack, key=lambda c: c.power)
    else:
        if "arch" not in state:
            if state["seen"]:
                state["arch"] = max(state["seen"], key=state["seen"].get)
            else:
                state["arch"] = rng.choice(ARCHETYPES)
        arch = state["arch"]
        sa_cards = [c for c in pack if is_sa_tier(c, arch)]
        if sa_cards:
            return max(sa_cards, key=lambda c: c.power)
        return max(pack, key=lambda c: c.power)


STRATEGIES = {
    "committed": player_committed,
    "power_chaser": player_power_chaser,
    "signal_reader": player_signal_reader,
}


# ── Single Draft ───────────────────────────────────────────────────────────

@dataclass
class DraftResult:
    player_picks: list = field(default_factory=list)
    packs_shown: list = field(default_factory=list)
    committed_archetype: Optional[str] = None
    sa_per_pack: list = field(default_factory=list)
    pool_sizes: list = field(default_factory=list)
    ai_removed_per_round: list = field(default_factory=list)
    culled_per_round: list = field(default_factory=list)
    open_archetypes: list = field(default_factory=list)
    ai_archetypes: list = field(default_factory=list)
    picks_completed: int = 0


def run_single_draft(rng, strategy_name, reactive=True):
    """Run one 30-pick draft. Returns (DraftResult, open_archetypes)."""
    pool = generate_pool(rng)
    pool_set = set(pool)

    # Assign 6 AIs to 6 of 8 archetypes
    arch_indices = list(range(NUM_ARCHETYPES))
    rng.shuffle(arch_indices)
    ai_arch_indices = arch_indices[:NUM_AIS]
    open_arch_indices = arch_indices[NUM_AIS:]

    ai_archetypes = [ARCHETYPES[i] for i in ai_arch_indices]
    open_archetypes = [ARCHETYPES[i] for i in open_arch_indices]

    # Create AI drafters with staggered phase 2 transitions
    phase2_starts = [8, 8, 9, 9, 10, 10]
    rng.shuffle(phase2_starts)
    ais = []
    for i, arch in enumerate(ai_archetypes):
        ai = AIDrafter(arch, phase2_starts[i], list(pool_set))
        ais.append(ai)

    strategy_fn = STRATEGIES[strategy_name]
    state = {"signals": {}, "seen": {}}
    result = DraftResult()
    result.open_archetypes = open_archetypes
    result.ai_archetypes = ai_archetypes

    for pick_num in range(1, NUM_PICKS + 1):
        if len(pool_set) < PACK_SIZE:
            break

        # ── AI picks ──
        ai_removed = 0
        for ai in ais:
            if not reactive or pick_num < ai.phase2_start_pick:
                picks = ai.pick_predetermined(pool_set, AI_PICKS_PER_ROUND)
            else:
                picks = ai.pick_reactive(list(pool_set), AI_PICKS_PER_ROUND)

            for card in picks:
                if card in pool_set:
                    pool_set.discard(card)
                    ai_removed += 1

        result.ai_removed_per_round.append(ai_removed)

        # ── Supplemental culling ──
        pre_cull = len(pool_set)
        pool_list = supplemental_cull(list(pool_set), ai_archetypes, CULL_RATE)
        pool_set = set(pool_list)
        result.culled_per_round.append(pre_cull - len(pool_set))

        result.pool_sizes.append(len(pool_set))

        if len(pool_set) < PACK_SIZE:
            break

        # ── Construct player pack ──
        candidates = list(pool_set)
        rng.shuffle(candidates)
        pack = candidates[:PACK_SIZE]
        result.packs_shown.append(pack)

        # ── Player picks ──
        chosen = strategy_fn(pack, pick_num, state, rng)
        if chosen:
            result.player_picks.append(chosen)
            pool_set.discard(chosen)

        result.picks_completed = pick_num

    # Determine committed archetype
    result.committed_archetype = state.get("arch", None)
    if result.committed_archetype is None and result.player_picks:
        arch_counts = defaultdict(int)
        for card in result.player_picks:
            if card.archetype:
                arch_counts[card.archetype] += 1
        if arch_counts:
            result.committed_archetype = max(arch_counts, key=arch_counts.get)

    # Compute S/A per pack
    if result.committed_archetype:
        for pack in result.packs_shown:
            sa = sum(1 for c in pack if is_sa_tier(c, result.committed_archetype))
            result.sa_per_pack.append(sa)

    return result, open_archetypes


# ── Metrics Computation ────────────────────────────────────────────────────

def compute_metrics(results, open_archs_list):
    """Compute all M1-M11 metrics from draft results."""
    m1_vals, m2_vals, m3_vals, m4_vals = [], [], [], []
    m5_vals, m6_vals, m9_vals, m10_vals, m11_vals = [], [], [], [], []
    per_arch_m3 = defaultdict(list)
    picks_completed = []

    for result in results:
        arch = result.committed_archetype
        if not arch:
            continue

        npacks = len(result.packs_shown)
        picks_completed.append(result.picks_completed)

        # M1: Picks 1-5, unique archetypes with S/A per pack
        for i in range(min(5, npacks)):
            pack = result.packs_shown[i]
            archs_with_sa = set()
            for c in pack:
                for a in ARCHETYPES:
                    if is_sa_tier(c, a):
                        archs_with_sa.add(a)
            m1_vals.append(len(archs_with_sa))

        # M2: Picks 1-5, S/A for committed archetype per pack
        for i in range(min(5, npacks)):
            m2_vals.append(sum(1 for c in result.packs_shown[i] if is_sa_tier(c, arch)))

        # M3: Picks 6+, S/A for committed archetype per pack
        p6_sa = []
        for i in range(5, npacks):
            sa = sum(1 for c in result.packs_shown[i] if is_sa_tier(c, arch))
            p6_sa.append(sa)
            m3_vals.append(sa)
        if p6_sa:
            per_arch_m3[arch].append(sum(p6_sa) / len(p6_sa))

        # M4: Picks 6+, off-archetype (C/F) cards per pack
        for i in range(5, npacks):
            off = sum(1 for c in result.packs_shown[i] if fitness_tier(c, arch) in ("C", "F"))
            m4_vals.append(off)

        # M5: Convergence pick (first 3-pick rolling avg >= 2.0 S/A)
        convergence = 30
        rolling = []
        for i in range(npacks):
            sa = sum(1 for c in result.packs_shown[i] if is_sa_tier(c, arch))
            rolling.append(sa)
            if len(rolling) >= 3 and sum(rolling[-3:]) / 3 >= 2.0:
                convergence = i + 1
                break
        m5_vals.append(convergence)

        # M6: Deck concentration
        if result.player_picks:
            sa_deck = sum(1 for c in result.player_picks if is_sa_tier(c, arch))
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

        # M11: Picks 15+, S/A per pack
        p15 = []
        for i in range(14, npacks):
            sa = sum(1 for c in result.packs_shown[i] if is_sa_tier(c, arch))
            p15.append(sa)
        if p15:
            m11_vals.append(sum(p15) / len(p15))

    def avg(lst):
        return sum(lst) / len(lst) if lst else 0.0

    metrics = {
        "M1": avg(m1_vals),
        "M2": avg(m2_vals),
        "M3": avg(m3_vals),
        "M4": avg(m4_vals),
        "M5": avg(m5_vals),
        "M6": avg(m6_vals) * 100,
        "M7": 0.0,
        "M9": avg(m9_vals),
        "M10": avg(m10_vals),
        "M10_p95": sorted(m10_vals)[int(len(m10_vals) * 0.95)] if m10_vals else 0,
        "M10_max": max(m10_vals) if m10_vals else 0,
        "M11": avg(m11_vals),
        "avg_picks": avg(picks_completed),
    }

    # M7: Card overlap between consecutive drafts
    overlaps = []
    for i in range(1, len(results)):
        a_ids = set(c.id for c in results[i - 1].player_picks)
        b_ids = set(c.id for c in results[i].player_picks)
        if a_ids and b_ids:
            overlaps.append(len(a_ids & b_ids) / max(len(a_ids), len(b_ids)))
    metrics["M7"] = avg(overlaps) * 100

    # M8: Archetype frequency
    arch_freq = defaultdict(int)
    total = 0
    for r in results:
        if r.committed_archetype:
            arch_freq[r.committed_archetype] += 1
            total += 1
    if total:
        freqs = [arch_freq.get(a, 0) / total * 100 for a in ARCHETYPES]
        metrics["M8_max"] = max(freqs)
        metrics["M8_min"] = min(freqs)
    else:
        metrics["M8_max"] = metrics["M8_min"] = 0

    # Per-archetype M3
    pa_m3 = {}
    for a in ARCHETYPES:
        v = per_arch_m3.get(a, [])
        pa_m3[a] = avg(v)

    # Pack quality distribution (picks 6+)
    all_sa = []
    for r in results:
        if r.committed_archetype and len(r.sa_per_pack) > 5:
            all_sa.extend(r.sa_per_pack[5:])
    pq = {}
    if all_sa:
        all_sa.sort()
        n = len(all_sa)
        for pct, label in [(0.10, "p10"), (0.25, "p25"), (0.50, "p50"),
                           (0.75, "p75"), (0.90, "p90")]:
            pq[label] = all_sa[min(int(n * pct), n - 1)]

    return metrics, pa_m3, pq


# ── Pessimistic Variant ────────────────────────────────────────────────────

def run_pessimistic(reactive, strategy):
    """Run 1000 drafts with pessimistic sibling rates (-10pp)."""
    global SIBLING_RATES
    orig = dict(SIBLING_RATES)
    SIBLING_RATES = {k: max(0.10, v - 0.10) for k, v in orig.items()}

    rng = random.Random(42)
    results = []
    oa_list = []
    for _ in range(NUM_DRAFTS):
        r, oa = run_single_draft(rng, strategy, reactive=reactive)
        results.append(r)
        oa_list.append(oa)

    SIBLING_RATES = orig
    m, _, _ = compute_metrics(results, oa_list)
    return m


# ── Draft Trace ────────────────────────────────────────────────────────────

def format_trace(rng, strategy_name, reactive):
    """Generate a human-readable trace of one draft (first 15 picks)."""
    pool = generate_pool(rng)
    pool_set = set(pool)

    arch_indices = list(range(NUM_ARCHETYPES))
    rng.shuffle(arch_indices)
    ai_archs = [ARCHETYPES[i] for i in arch_indices[:NUM_AIS]]
    open_archs = [ARCHETYPES[i] for i in arch_indices[NUM_AIS:]]

    phase2_starts = [8, 8, 9, 9, 10, 10]
    rng.shuffle(phase2_starts)
    ais = [AIDrafter(a, phase2_starts[i], list(pool_set)) for i, a in enumerate(ai_archs)]

    fn = STRATEGIES[strategy_name]
    state = {"signals": {}, "seen": {}}

    lines = [
        f"Strategy: {strategy_name} | Mode: {'Sentinel' if reactive else 'Level-0'}",
        f"AI lanes: {ai_archs}",
        f"Open lanes: {open_archs}",
        f"Pool: {len(pool_set)}",
        "",
    ]

    for pick in range(1, 16):
        if len(pool_set) < PACK_SIZE:
            lines.append(f"Pick {pick:2d} | Pool exhausted ({len(pool_set)} cards remain)")
            break

        ai_took = 0
        for ai in ais:
            if not reactive or pick < ai.phase2_start_pick:
                picks = ai.pick_predetermined(pool_set, AI_PICKS_PER_ROUND)
            else:
                picks = ai.pick_reactive(list(pool_set), AI_PICKS_PER_ROUND)
            for c in picks:
                if c in pool_set:
                    pool_set.discard(c)
                    ai_took += 1

        pre = len(pool_set)
        pl = supplemental_cull(list(pool_set), ai_archs, CULL_RATE)
        pool_set = set(pl)
        culled = pre - len(pool_set)

        if len(pool_set) < PACK_SIZE:
            lines.append(f"Pick {pick:2d} | Pool={len(pool_set)} after AI({ai_took})+cull({culled}) — exhausted")
            break

        cands = list(pool_set)
        rng.shuffle(cands)
        pack = cands[:PACK_SIZE]

        chosen = fn(pack, pick, state, rng)
        if chosen:
            pool_set.discard(chosen)

        arch = state.get("arch", "exploring")
        pack_desc = []
        for c in pack:
            t = fitness_tier(c, arch) if arch != "exploring" else "?"
            name = c.archetype or "Gen"
            pack_desc.append(f"{name[:4]}({t})")

        sa = sum(1 for c in pack if arch != "exploring" and is_sa_tier(c, arch))
        chosen_name = (chosen.archetype or "Gen")[:6] if chosen else "None"

        lines.append(
            f"Pick {pick:2d} | Pool:{len(pool_set):3d} | AI:{ai_took:2d} cull:{culled:2d} | "
            f"Pack:[{' '.join(pack_desc)}] S/A={sa} | Chose:{chosen_name} | Target:{arch}"
        )

    return "\n".join(lines)


# ── Main ───────────────────────────────────────────────────────────────────

def main():
    print("=" * 78)
    print("D2 SENTINEL DRAFT — Monte Carlo Simulation")
    print("1000 drafts x 30 picks x 3 strategies")
    print("Variants: Sentinel (Phase 2 reactive) + Level 0 (fully static control)")
    print("=" * 78)

    all_data = {}

    for vname, reactive in [("Sentinel", True), ("Level-0", False)]:
        print(f"\n{'━' * 78}")
        print(f"  VARIANT: {vname}")
        print(f"{'━' * 78}")

        vdata = {}
        for strat in ["committed", "power_chaser", "signal_reader"]:
            rng = random.Random(42)
            results = []
            oa_list = []
            for _ in range(NUM_DRAFTS):
                r, oa = run_single_draft(rng, strat, reactive=reactive)
                results.append(r)
                oa_list.append(oa)

            m, pa_m3, pq = compute_metrics(results, oa_list)
            vdata[strat] = {"metrics": m, "pa_m3": pa_m3, "pq": pq,
                            "results": results, "oa": oa_list}

            print(f"\n  Strategy: {strat}")
            print(f"  Avg picks completed: {m['avg_picks']:.1f} / 30")
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
            print(f"    M11 = {m['M11']:.2f}   (>= 3.0)")

            if pq:
                print(f"    PackQ: p10={pq['p10']} p25={pq['p25']} p50={pq['p50']} p75={pq['p75']} p90={pq['p90']}")

            print(f"    Per-Archetype M3:")
            for a in ARCHETYPES:
                print(f"      {a:15s} {pa_m3[a]:.2f}")

        all_data[vname] = vdata

    # ── Pessimistic ──────────────────────────────────────────────────────

    print(f"\n{'━' * 78}")
    print("  PESSIMISTIC FITNESS (sibling rates -10pp)")
    print(f"{'━' * 78}")

    for vname, reactive in [("Sentinel", True), ("Level-0", False)]:
        print(f"\n  {vname}:")
        for strat in ["committed", "signal_reader"]:
            m = run_pessimistic(reactive, strat)
            print(f"    {strat}: M3={m['M3']:.2f}  M10={m['M10']:.2f}  M11={m['M11']:.2f}")

    # ── Traces ───────────────────────────────────────────────────────────

    print(f"\n{'━' * 78}")
    print("  DRAFT TRACES")
    print(f"{'━' * 78}")

    for strat in ["committed", "signal_reader"]:
        print(f"\n── {strat} (Sentinel) ──")
        print(format_trace(random.Random(777), strat, True))

    # ── Head-to-Head Comparison ──────────────────────────────────────────

    print(f"\n{'━' * 78}")
    print("  COMPARISON: Sentinel vs Level-0 (avg of committed + signal_reader)")
    print(f"{'━' * 78}")

    print(f"\n  {'Metric':<8} {'Sentinel':>10} {'Level-0':>10} {'Delta':>8} {'Target':>12}")
    print(f"  {'-'*52}")

    for mk, tgt in [("M1", ">=3"), ("M2", "<=2"), ("M3", ">=2.0"),
                     ("M4", ">=0.5"), ("M5", "5-8"), ("M6", "60-90%"),
                     ("M9", ">=0.8"), ("M10", "<=2"), ("M11", ">=3.0")]:
        sv = sum(all_data["Sentinel"][s]["metrics"][mk] for s in ["committed", "signal_reader"]) / 2
        lv = sum(all_data["Level-0"][s]["metrics"][mk] for s in ["committed", "signal_reader"]) / 2
        d = sv - lv
        sign = "+" if d >= 0 else ""
        print(f"  {mk:<8} {sv:>10.2f} {lv:>10.2f} {sign}{d:>7.2f} {tgt:>12}")

    # ── Pool depletion analysis ──────────────────────────────────────────

    print(f"\n{'━' * 78}")
    print("  POOL DEPLETION ANALYSIS")
    print(f"{'━' * 78}")

    for vname in ["Sentinel", "Level-0"]:
        strat_data = all_data[vname]["committed"]
        rs = strat_data["results"]
        avg_pools = defaultdict(list)
        avg_ai = defaultdict(list)
        for r in rs:
            for i, ps in enumerate(r.pool_sizes):
                avg_pools[i + 1].append(ps)
            for i, ai in enumerate(r.ai_removed_per_round):
                avg_ai[i + 1].append(ai)

        print(f"\n  {vname} (committed strategy):")
        print(f"  {'Pick':>4} {'Pool avg':>10} {'AI avg':>10} {'Cull avg':>10}")
        for p in sorted(avg_pools.keys())[:15]:
            pool_avg = sum(avg_pools[p]) / len(avg_pools[p])
            ai_avg = sum(avg_ai[p]) / len(avg_ai[p]) if p in avg_ai else 0
            cull_data = [r.culled_per_round[p - 1] for r in rs if len(r.culled_per_round) >= p]
            cull_avg = sum(cull_data) / len(cull_data) if cull_data else 0
            print(f"  {p:>4} {pool_avg:>10.1f} {ai_avg:>10.1f} {cull_avg:>10.1f}")

    print(f"\n{'=' * 78}")
    print("SIMULATION COMPLETE")
    print(f"{'=' * 78}")


if __name__ == "__main__":
    main()
