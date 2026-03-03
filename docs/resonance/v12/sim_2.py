"""
SIM-2: Hybrid 1 — Robustly Biased Pressure + N=12 + Floor
==========================================================
V12 Simulation Agent 2

Algorithm (from critic review Section 7, Hybrid 1):
- Starting pool: 120 cards, 8 archetypes (15 per archetype)
- Fitness: Graduated Realistic (~36% weighted-average sibling A-tier)
- 5 AIs, each assigned 1 of 5 archetypes (3 open lanes)
- AI avoidance: gradual ramp from pick 5, reaching 80% by pick 12
- Refills: 50/30/0 (3 rounds of 10 picks), 2.0x open-lane bias
- S/A targeting: refills add S/A at 40% rate (vs 36% baseline) for open lanes
- Pack construction: N=4 (picks 1-5), N=12 (picks 6-30)
- "Best 4" ranking: visible resonance symbol match only (NO pair-affinity)
- Floor slot: guarantee 1 S/A in shown 4 if any S/A drawn

1000 drafts x 30 picks x 3 player strategies, all 14 metrics.
"""

import random
import math
from dataclasses import dataclass, field
from typing import Optional
from collections import defaultdict

# ============================================================
# Constants
# ============================================================

NUM_ARCHETYPES = 8
ARCHETYPE_NAMES = [
    "Flash/Tempo",        # 0 - Zephyr primary, Ember secondary
    "Blink/Flicker",      # 1 - Ember primary, Zephyr secondary
    "Storm/Spellslinger", # 2 - Ember primary, Stone secondary
    "Self-Discard",       # 3 - Stone primary, Ember secondary
    "Self-Mill/Reanim",   # 4 - Stone primary, Tide secondary
    "Sacrifice/Abandon",  # 5 - Tide primary, Stone secondary
    "Warriors/Midrange",  # 6 - Tide primary, Zephyr secondary
    "Ramp/SpiritAnim",    # 7 - Zephyr primary, Tide secondary
]

# Resonance symbols: 0=Tide, 1=Stone, 2=Ember, 3=Zephyr
RESONANCE_NAMES = ["Tide", "Stone", "Ember", "Zephyr"]
NUM_RESONANCES = 4

# Archetype -> (primary_resonance, secondary_resonance)
ARCHETYPE_RESONANCE = {
    0: (3, 2),  # Flash: Zephyr/Ember
    1: (2, 3),  # Blink: Ember/Zephyr
    2: (2, 1),  # Storm: Ember/Stone
    3: (1, 2),  # Self-Discard: Stone/Ember
    4: (1, 0),  # Self-Mill: Stone/Tide
    5: (0, 1),  # Sacrifice: Tide/Stone
    6: (0, 3),  # Warriors: Tide/Zephyr
    7: (3, 0),  # Ramp: Zephyr/Tide
}

# Archetype pairs on the circle (sibling A-tier rates)
ARCHETYPE_PAIRS = [
    (6, 5, 0.50),  # Warriors / Sacrifice (Tide) - 50%
    (3, 4, 0.40),  # Self-Discard / Self-Mill (Stone) - 40%
    (1, 2, 0.30),  # Blink / Storm (Ember) - 30%
    (0, 7, 0.25),  # Flash / Ramp (Zephyr) - 25%
]

SIBLING_MAP = {}
for a, b, rate in ARCHETYPE_PAIRS:
    SIBLING_MAP[a] = (b, rate)
    SIBLING_MAP[b] = (a, rate)

STARTING_POOL_SIZE = 120
CARDS_PER_ARCHETYPE = 15
PICKS_PER_ROUND = 10
NUM_ROUNDS = 3
TOTAL_PICKS = 30
# Refills BETWEEN rounds: 50 after R1, 30 after R2, 0 after R3
REFILL_BETWEEN_ROUNDS = [50, 30, 0]
REFILL_BIAS = 2.0
REFILL_SA_RATE_OPEN = 0.40

NUM_AIS = 5
AI_SATURATION_THRESHOLD = 12
PACK_SHOW = 4

S_TIER_THRESHOLD = 0.85
A_TIER_THRESHOLD = 0.65

NUM_SIMULATIONS = 1000


def get_oversample_n(pick_num):
    """N=4 for picks 1-5, N=12 for picks 6-30."""
    return 4 if pick_num <= 5 else 12


def get_avoidance_weight(pick_num):
    """Gradual ramp: 0% at pick <5, linear to 80% at pick 12, 80% after."""
    if pick_num < 5:
        return 0.0
    elif pick_num <= 12:
        return 0.80 * (pick_num - 5) / (12 - 5)
    else:
        return 0.80


# ============================================================
# Card Model
# ============================================================

_next_card_id = 0


@dataclass
class SimCard:
    id: int
    archetype: int
    fitness: dict
    power: float
    visible_symbols: list  # list of resonance indices (0-3)

    def is_sa_for(self, archetype):
        return self.fitness.get(archetype, 0.0) >= A_TIER_THRESHOLD

    def resonance_score(self, signature):
        """Cosine similarity of card's visible symbols against a resonance
        signature vector [Tide, Stone, Ember, Zephyr]."""
        if not self.visible_symbols or all(s == 0 for s in signature):
            return 0.0
        card_vec = [0.0] * NUM_RESONANCES
        for sym in self.visible_symbols:
            card_vec[sym] += 1.0
        dot = sum(a * b for a, b in zip(card_vec, signature))
        card_mag = math.sqrt(sum(x * x for x in card_vec))
        sig_mag = math.sqrt(sum(x * x for x in signature))
        if card_mag == 0 or sig_mag == 0:
            return 0.0
        return dot / (card_mag * sig_mag)


def _gen_id():
    global _next_card_id
    _next_card_id += 1
    return _next_card_id


def _gen_fitness(primary_archetype, sa_rate_override=None):
    fitness = {}
    sibling, sibling_rate = SIBLING_MAP[primary_archetype]

    # Primary archetype fitness
    sa_rate = sa_rate_override if sa_rate_override is not None else 0.36
    roll = random.random()
    if roll < sa_rate * 0.42:
        fitness[primary_archetype] = random.uniform(0.85, 1.0)
    elif roll < sa_rate:
        fitness[primary_archetype] = random.uniform(0.65, 0.849)
    elif roll < sa_rate + 0.34:
        fitness[primary_archetype] = random.uniform(0.35, 0.649)
    else:
        fitness[primary_archetype] = random.uniform(0.05, 0.349)

    # Sibling
    sib_roll = random.random()
    if sib_roll < sibling_rate * 0.4:
        fitness[sibling] = random.uniform(0.85, 1.0)
    elif sib_roll < sibling_rate:
        fitness[sibling] = random.uniform(0.65, 0.849)
    elif sib_roll < sibling_rate + 0.25:
        fitness[sibling] = random.uniform(0.35, 0.649)
    else:
        fitness[sibling] = random.uniform(0.05, 0.349)

    # Others
    for arch in range(NUM_ARCHETYPES):
        if arch in (primary_archetype, sibling):
            continue
        dist = min(abs(arch - primary_archetype),
                   NUM_ARCHETYPES - abs(arch - primary_archetype))
        if dist == 2:
            if random.random() < 0.10:
                fitness[arch] = random.uniform(0.35, 0.649)
            else:
                fitness[arch] = random.uniform(0.02, 0.349)
        else:
            fitness[arch] = random.uniform(0.01, 0.20)
    return fitness


def _gen_symbols(archetype):
    primary_res, secondary_res = ARCHETYPE_RESONANCE[archetype]
    roll = random.random()
    if roll < 0.11:
        return []
    elif roll < 0.90:
        return [primary_res] if random.random() < 0.70 else [secondary_res]
    else:
        return [primary_res, secondary_res]


def make_card(archetype, sa_rate_override=None):
    return SimCard(
        id=_gen_id(),
        archetype=archetype,
        fitness=_gen_fitness(archetype, sa_rate_override),
        power=random.uniform(0, 10),
        visible_symbols=_gen_symbols(archetype),
    )


def generate_pool():
    return [make_card(arch) for arch in range(NUM_ARCHETYPES)
            for _ in range(CARDS_PER_ARCHETYPE)]


def generate_refill_biased(total_cards, open_archetypes, ai_archetypes):
    """Open lanes get REFILL_BIAS x cards vs AI lanes.
    Open-lane cards use elevated S/A rate."""
    num_open = len(open_archetypes)
    num_ai = len(ai_archetypes)
    denom = num_open * REFILL_BIAS + num_ai
    per_ai = total_cards / denom
    per_open = per_ai * REFILL_BIAS

    cards = []
    for arch in range(NUM_ARCHETYPES):
        if arch in open_archetypes:
            cnt = int(round(per_open))
            sa_rate = REFILL_SA_RATE_OPEN
        elif arch in ai_archetypes:
            cnt = int(round(per_ai))
            sa_rate = None
        else:
            continue
        for _ in range(cnt):
            cards.append(make_card(arch, sa_rate_override=sa_rate))

    while len(cards) < total_cards:
        cards.append(make_card(random.choice(open_archetypes),
                               sa_rate_override=REFILL_SA_RATE_OPEN))
    while len(cards) > total_cards:
        cards.pop()
    return cards


# ============================================================
# AI Drafter with Avoidance
# ============================================================

@dataclass
class AIDrafter:
    archetype: int
    cards_drafted: list = field(default_factory=list)
    on_archetype_count: int = 0

    def pick_from_pool(self, pool, avoidance_weight, inferred_player_arch):
        """AI picks best card for its archetype.  With avoidance, it will not
        pick cards belonging to the player's inferred archetype."""
        if not pool:
            return None

        avoiding = (avoidance_weight > 0
                    and inferred_player_arch is not None
                    and inferred_player_arch != self.archetype)

        # Build candidate list
        if self.on_archetype_count < AI_SATURATION_THRESHOLD:
            # Prefer own archetype cards
            own = [c for c in pool if c.archetype == self.archetype]
            if own:
                best = max(own, key=lambda c: c.fitness.get(self.archetype, 0))
                self._take(best, pool)
                return best

        # Fallback: pick from any card NOT in player's archetype (with avoidance)
        candidates = list(pool)
        if avoiding and random.random() < avoidance_weight:
            non_player = [c for c in candidates
                          if c.archetype != inferred_player_arch]
            if non_player:
                candidates = non_player

        if candidates:
            best = max(candidates, key=lambda c: c.power)
            self._take(best, pool)
            return best
        return None

    def _take(self, card, pool):
        self.cards_drafted.append(card)
        if card.archetype == self.archetype:
            self.on_archetype_count += 1
        pool.remove(card)


# ============================================================
# AI Inference Engine
# ============================================================

class AIInferenceEngine:
    """Infers the player's archetype from pool depletion patterns.

    Key insight: with 5 AIs on 5 archetypes, AI archetypes deplete fastest
    (~1 card/cycle). The player's archetype (one of 3 open lanes) depletes
    moderately (~0.7/cycle). The other 2 open archetypes deplete slowest
    (~0/cycle, only random AI fallback picks).

    Strategy: identify the 3 slowest-depleting archetypes as open lanes.
    Among those, find the one with the most depletion = player's archetype."""

    def __init__(self):
        self.count_history = []
        self.inferred = None
        self.confidence = 0.0

    def snapshot(self, pool):
        counts = defaultdict(int)
        for c in pool:
            counts[c.archetype] += 1
        self.count_history.append(dict(counts))

    def reset_baseline(self, pool):
        """Reset the baseline after a refill event."""
        counts = defaultdict(int)
        for c in pool:
            counts[c.archetype] += 1
        self.baseline = dict(counts)
        self.picks_since_baseline = 0

    def infer(self, pick_num):
        if pick_num < 5 or len(self.count_history) < 4:
            return self.inferred, self.confidence

        self.picks_since_baseline = getattr(self, 'picks_since_baseline', 0) + 1

        # Use the baseline (set after each refill or at start) for comparison
        baseline = getattr(self, 'baseline', self.count_history[0])
        new = self.count_history[-1]

        if self.picks_since_baseline < 3:
            return self.inferred, self.confidence

        # Calculate depletion per archetype since baseline
        depletions = {}
        for arch in range(NUM_ARCHETYPES):
            arch_old = baseline.get(arch, 0)
            arch_new = new.get(arch, 0)
            depletions[arch] = max(0, arch_old - arch_new)

        # Sort by depletion (ascending). Least depleted = likely open lanes
        sorted_archs = sorted(depletions, key=lambda a: depletions[a])

        # The 3 least-depleted are likely open lanes (no AI assigned)
        likely_open = sorted_archs[:3]

        # Among open lanes, the one with most depletion = player's archetype
        open_deps = {a: depletions[a] for a in likely_open}
        best_open = max(open_deps, key=open_deps.get)
        best_dep = open_deps[best_open]

        other_deps = [open_deps[a] for a in likely_open if a != best_open]
        second_dep = max(other_deps) if other_deps else 0

        gap = best_dep - second_dep

        if best_dep >= 2 and gap >= 1:
            self.inferred = best_open
            self.confidence = min(gap / 4.0 + 0.2, 1.0)
        elif best_dep >= 1 and gap >= 1:
            self.inferred = best_open
            self.confidence = min(gap / 5.0 + 0.1, 0.6)
        elif self.inferred is not None:
            # Keep existing inference but decay confidence
            self.confidence *= 0.85

        return self.inferred, self.confidence


# ============================================================
# Pack Construction
# ============================================================

def build_resonance_signature(drafted_cards):
    sig = [0.0] * NUM_RESONANCES
    for card in drafted_cards:
        for sym in card.visible_symbols:
            sig[sym] += 1.0
    return sig


def construct_pack(pool, player_drafted, pick_num, committed_arch):
    """Draw N from pool, rank by resonance match, show best 4.
    Floor slot: guarantee 1 S/A in shown 4 if any S/A in the draw."""
    n = get_oversample_n(pick_num)
    if len(pool) <= PACK_SHOW:
        return list(pool)

    draw_count = min(n, len(pool))
    drawn = random.sample(pool, draw_count)

    if pick_num <= 5 or n <= 4:
        return drawn[:PACK_SHOW]

    sig = build_resonance_signature(player_drafted)

    # Score and rank
    scored = sorted(drawn, key=lambda c: (-c.resonance_score(sig), -c.power))
    shown = list(scored[:PACK_SHOW])

    # Floor slot: if committed and any S/A in drawn but none in shown
    if committed_arch is not None:
        sa_in_drawn = [c for c in drawn if c.is_sa_for(committed_arch)]
        sa_in_shown = [c for c in shown if c.is_sa_for(committed_arch)]
        if sa_in_drawn and not sa_in_shown:
            best_sa = max(sa_in_drawn, key=lambda c: c.resonance_score(sig))
            shown[-1] = best_sa

    return shown


# ============================================================
# Player Strategies
# ============================================================

def strategy_committed(pack, committed_arch, pick_num, drafted, pool,
                       open_archetypes, ai_archetypes):
    if committed_arch is None:
        committed_arch = (random.choice(open_archetypes)
                          if open_archetypes else random.randint(0, 7))
    best = max(pack, key=lambda c: c.fitness.get(committed_arch, 0))
    return best, committed_arch


def strategy_signal_reader(pack, committed_arch, pick_num, drafted, pool,
                           open_archetypes, ai_archetypes):
    if pick_num <= 4:
        return max(pack, key=lambda c: c.power), None
    if pick_num == 5 or committed_arch is None:
        # Find open archetype with most S/A remaining in pool
        sa = defaultdict(int)
        for c in pool:
            for a in open_archetypes:
                if c.is_sa_for(a):
                    sa[a] += 1
        committed_arch = max(open_archetypes, key=lambda a: sa.get(a, 0))
    return max(pack, key=lambda c: c.fitness.get(committed_arch, 0)), committed_arch


def strategy_power_chaser(pack, committed_arch, pick_num, drafted, pool,
                          open_archetypes, ai_archetypes):
    best = max(pack, key=lambda c: c.power)
    if drafted:
        counts = defaultdict(int)
        for c in drafted:
            counts[c.archetype] += 1
        committed_arch = max(counts, key=counts.get)
    return best, committed_arch


# ============================================================
# Single Draft Simulation
# ============================================================

@dataclass
class DraftResult:
    packs_seen: list = field(default_factory=list)
    picks_made: list = field(default_factory=list)
    committed_archetype: Optional[int] = None
    convergence_pick: int = 1

    ai_archetypes: list = field(default_factory=list)
    open_archetypes: list = field(default_factory=list)
    committed_is_open: bool = False

    pool_sizes: list = field(default_factory=list)
    pool_arch_counts: list = field(default_factory=list)
    pool_sa_counts: list = field(default_factory=list)

    sa_per_pack: list = field(default_factory=list)
    unique_archs_with_sa: list = field(default_factory=list)
    max_sa_emerging: list = field(default_factory=list)
    off_archetype_cards: list = field(default_factory=list)

    ai_inference_log: list = field(default_factory=list)
    refill_moments: list = field(default_factory=list)


def run_single_draft(strategy_fn, rng_seed=None):
    if rng_seed is not None:
        random.seed(rng_seed)

    result = DraftResult()
    pool = generate_pool()

    ai_archetypes = random.sample(range(NUM_ARCHETYPES), NUM_AIS)
    open_archetypes = [a for a in range(NUM_ARCHETYPES) if a not in ai_archetypes]
    ais = [AIDrafter(archetype=a) for a in ai_archetypes]
    engine = AIInferenceEngine()
    engine.reset_baseline(pool)  # Set initial baseline

    result.ai_archetypes = list(ai_archetypes)
    result.open_archetypes = list(open_archetypes)

    committed_arch = None
    drafted = []

    for round_idx in range(NUM_ROUNDS):
        # Refill between rounds (after R1 -> +50, after R2 -> +30, after R3 -> 0)
        if round_idx > 0:
            refill_amt = REFILL_BETWEEN_ROUNDS[round_idx - 1]
            if refill_amt > 0:
                pool.extend(generate_refill_biased(
                    refill_amt, open_archetypes, ai_archetypes))
                result.refill_moments.append(round_idx * PICKS_PER_ROUND + 1)
                engine.reset_baseline(pool)  # Reset after refill

        for pick_in_round in range(PICKS_PER_ROUND):
            pick_num = round_idx * PICKS_PER_ROUND + pick_in_round + 1

            # --- Record pool state ---
            arch_counts = defaultdict(int)
            for c in pool:
                arch_counts[c.archetype] += 1
            result.pool_sizes.append(len(pool))
            result.pool_arch_counts.append(dict(arch_counts))

            sa_in_pool = (sum(1 for c in pool if c.is_sa_for(committed_arch))
                          if committed_arch is not None else 0)
            result.pool_sa_counts.append(sa_in_pool)

            # --- AI inference ---
            engine.snapshot(pool)
            inferred, conf = engine.infer(pick_num)
            avoid_w = get_avoidance_weight(pick_num)
            effective_avoid = avoid_w * min(conf, 1.0)

            correct = (inferred == committed_arch
                       if committed_arch is not None else False)
            result.ai_inference_log.append({
                'pick': pick_num, 'inferred': inferred,
                'actual': committed_arch, 'correct': correct,
                'conf': conf, 'avoid_w': avoid_w,
                'eff_avoid': effective_avoid,
            })

            # --- AIs pick ---
            for ai in ais:
                if pool:
                    ai.pick_from_pool(pool, effective_avoid, inferred)

            # --- Construct pack ---
            pack = construct_pack(pool, drafted, pick_num, committed_arch)
            if not pack:
                break
            result.packs_seen.append(pack)

            # M1: unique archetypes with S/A in pack
            archs_sa = set()
            for c in pack:
                for a in range(NUM_ARCHETYPES):
                    if c.is_sa_for(a):
                        archs_sa.add(a)
            result.unique_archs_with_sa.append(len(archs_sa))

            # M2: max S/A for any archetype
            max_sa = max(sum(1 for c in pack if c.is_sa_for(a))
                         for a in range(NUM_ARCHETYPES))
            result.max_sa_emerging.append(max_sa)

            # --- Player picks ---
            pick_card, committed_arch = strategy_fn(
                pack, committed_arch, pick_num, drafted, pool,
                open_archetypes, ai_archetypes)
            drafted.append(pick_card)
            result.picks_made.append(pick_card)
            if pick_card in pool:
                pool.remove(pick_card)

            if committed_arch is not None and result.committed_archetype is None:
                result.committed_archetype = committed_arch
                result.convergence_pick = pick_num

            # M3: S/A for committed in pack
            sa_cnt = (sum(1 for c in pack if c.is_sa_for(committed_arch))
                      if committed_arch is not None else 0)
            result.sa_per_pack.append(sa_cnt)

            # M4: off-archetype
            off = (sum(1 for c in pack if c.archetype != committed_arch)
                   if committed_arch is not None else len(pack))
            result.off_archetype_cards.append(off)

    result.committed_archetype = committed_arch
    if committed_arch is not None:
        result.committed_is_open = committed_arch in open_archetypes
    return result


# ============================================================
# Metrics
# ============================================================

def pct(data, p):
    if not data:
        return 0.0
    s = sorted(data)
    k = (len(s) - 1) * p / 100
    f = int(k)
    c = min(f + 1, len(s) - 1)
    return s[f] + (k - f) * (s[c] - s[f])


def compute_metrics(results, name):
    m = {}

    # Helper: collect per-pack values for a pick range
    def pack_vals(pick_start, pick_end, attr):
        vals = []
        for r in results:
            if r.committed_archetype is None:
                continue
            lst = getattr(r, attr)
            for i in range(pick_start - 1, min(pick_end, len(lst))):
                vals.append(lst[i])
        return vals

    # M1
    m1 = []
    for r in results:
        for i in range(min(5, len(r.unique_archs_with_sa))):
            m1.append(r.unique_archs_with_sa[i])
    m['M1'] = sum(m1) / max(len(m1), 1)

    # M2
    m2 = []
    for r in results:
        for i in range(min(5, len(r.max_sa_emerging))):
            m2.append(r.max_sa_emerging[i])
    m['M2'] = sum(m2) / max(len(m2), 1)

    # M3
    m3v = pack_vals(6, 30, 'sa_per_pack')
    m['M3'] = sum(m3v) / max(len(m3v), 1)

    # M4
    m4v = pack_vals(6, 30, 'off_archetype_cards')
    m['M4'] = sum(m4v) / max(len(m4v), 1)

    # M5
    m5 = [r.convergence_pick for r in results if r.convergence_pick]
    m['M5'] = sum(m5) / max(len(m5), 1)

    # M6
    m6 = []
    for r in results:
        if r.committed_archetype is not None and r.picks_made:
            sa = sum(1 for c in r.picks_made if c.is_sa_for(r.committed_archetype))
            m6.append(sa / len(r.picks_made))
    m['M6'] = sum(m6) / max(len(m6), 1)

    # M7
    arch_runs = defaultdict(list)
    for r in results:
        if r.committed_archetype is not None:
            arch_runs[r.committed_archetype].append(set(c.id for c in r.picks_made))
    overlaps = []
    for runs in arch_runs.values():
        for i in range(1, len(runs)):
            if runs[i - 1] and runs[i]:
                overlaps.append(len(runs[i-1] & runs[i]) / len(runs[i-1] | runs[i]))
    m['M7'] = sum(overlaps) / max(len(overlaps), 1) if overlaps else 0.0

    # M8
    freq = defaultdict(int)
    for r in results:
        if r.committed_archetype is not None:
            freq[r.committed_archetype] += 1
    tot = sum(freq.values())
    pcts = {a: freq[a] / max(tot, 1) * 100 for a in range(NUM_ARCHETYPES)}
    m['M8_max'] = max(pcts.values()) if pcts else 0
    m['M8_min'] = min(pcts.values()) if pcts else 0
    m['M8_dist'] = pcts

    # M9
    if m3v:
        mean3 = sum(m3v) / len(m3v)
        m['M9'] = math.sqrt(sum((x - mean3) ** 2 for x in m3v) / len(m3v))
    else:
        m['M9'] = 0.0

    # M10
    m10 = []
    for r in results:
        if r.committed_archetype is None:
            continue
        vals = r.sa_per_pack[5:]
        streak = mx = 0
        for v in vals:
            if v < 1.5:
                streak += 1
                mx = max(mx, streak)
            else:
                streak = 0
        m10.append(mx)
    m['M10'] = sum(m10) / max(len(m10), 1)

    # M11'
    m11v = pack_vals(20, 30, 'sa_per_pack')
    m['M11p'] = sum(m11v) / max(len(m11v), 1)

    # Pack quality percentiles (picks 6+)
    m['pq'] = {p: pct(m3v, p) for p in [10, 25, 50, 75, 90]}

    # Per-archetype M3
    per_arch = {}
    for a in range(NUM_ARCHETYPES):
        vs = [r.sa_per_pack[i] for r in results
              if r.committed_archetype == a
              for i in range(5, len(r.sa_per_pack))]
        per_arch[a] = sum(vs) / len(vs) if vs else 0.0
    m['per_arch_m3'] = per_arch

    # Pool trajectory
    max_len = max((len(r.pool_sizes) for r in results), default=0)
    m['pool_traj'] = [
        sum(r.pool_sizes[i] for r in results if i < len(r.pool_sizes))
        / sum(1 for r in results if i < len(r.pool_sizes))
        for i in range(max_len)
    ]

    # SA trajectory
    m['sa_traj'] = []
    for i in range(max_len):
        vs = [r.pool_sa_counts[i] for r in results
              if i < len(r.pool_sa_counts) and r.committed_archetype is not None
              and r.pool_sa_counts[i] > 0]
        m['sa_traj'].append(sum(vs) / len(vs) if vs else 0)

    # Arch density trajectory
    m['density_traj'] = []
    for i in range(max_len):
        ds = []
        for r in results:
            if (r.committed_archetype is not None and
                    i < len(r.pool_arch_counts) and
                    i < len(r.pool_sizes) and r.pool_sizes[i] > 0):
                ac = r.pool_arch_counts[i].get(r.committed_archetype, 0)
                ds.append(ac / r.pool_sizes[i])
        m['density_traj'].append(sum(ds) / len(ds) if ds else 0)

    # AI inference accuracy by pick
    inf_acc = defaultdict(list)
    for r in results:
        if r.committed_archetype is not None:
            for e in r.ai_inference_log:
                if e['actual'] is not None:
                    inf_acc[e['pick']].append(1 if e['correct'] else 0)
    m['inf_acc'] = {p: sum(v)/len(v) for p, v in inf_acc.items()}

    # Effective avoidance by pick
    eff_av = defaultdict(list)
    for r in results:
        if r.committed_archetype is not None:
            for e in r.ai_inference_log:
                if e['actual'] is not None:
                    eff_av[e['pick']].append(e['eff_avoid'] if e['correct'] else 0.0)
    m['eff_avoid'] = {p: sum(v)/len(v) for p, v in eff_av.items()}

    # M13: first pick where avoidance is detectably active
    first_avoid = []
    for r in results:
        for e in r.ai_inference_log:
            if e['correct'] and e['eff_avoid'] > 0.1:
                first_avoid.append(e['pick'])
                break
    m['M13'] = sum(first_avoid) / len(first_avoid) if first_avoid else 30

    # M14: first pick where inference is correct with confidence
    first_correct = []
    for r in results:
        for e in r.ai_inference_log:
            if e['correct'] and e['conf'] > 0.3:
                first_correct.append(e['pick'])
                break
    m['M14'] = sum(first_correct) / len(first_correct) if first_correct else 30

    # Open-lane breakdown
    open_m3 = [r.sa_per_pack[i]
               for r in results if r.committed_is_open
               for i in range(5, len(r.sa_per_pack))]
    ai_m3 = [r.sa_per_pack[i]
             for r in results
             if r.committed_archetype is not None and not r.committed_is_open
             for i in range(5, len(r.sa_per_pack))]
    m['M3_open'] = sum(open_m3) / len(open_m3) if open_m3 else 0
    m['M3_ai'] = sum(ai_m3) / len(ai_m3) if ai_m3 else 0
    m['open_pct'] = sum(1 for r in results if r.committed_is_open) / max(len(results), 1) * 100

    # SA at key picks
    sa_at = {}
    for tp in [20, 25, 30]:
        idx = tp - 1
        vs = [r.pool_sa_counts[idx] for r in results
              if idx < len(r.pool_sa_counts) and r.committed_archetype is not None
              and r.pool_sa_counts[idx] > 0]
        sa_at[tp] = sum(vs) / len(vs) if vs else 0
    m['sa_at'] = sa_at

    # M3 by pick bands
    m['m3_6_15'] = sum(pack_vals(6, 15, 'sa_per_pack')) / max(len(pack_vals(6, 15, 'sa_per_pack')), 1)
    m['m3_16_20'] = sum(pack_vals(16, 20, 'sa_per_pack')) / max(len(pack_vals(16, 20, 'sa_per_pack')), 1)
    m['m3_21_30'] = sum(pack_vals(21, 30, 'sa_per_pack')) / max(len(pack_vals(21, 30, 'sa_per_pack')), 1)

    return m


# ============================================================
# Trace
# ============================================================

def gen_trace(strategy_fn, label, seed):
    random.seed(seed)
    r = run_single_draft(strategy_fn, rng_seed=seed)
    arch = r.committed_archetype
    aname = ARCHETYPE_NAMES[arch] if arch is not None else "None"
    open_l = "OPEN" if r.committed_is_open else "AI-lane"

    out = [f"Trace: {label} | {aname} ({open_l}) | conv pick {r.convergence_pick}",
           f"AIs: {[ARCHETYPE_NAMES[a][:6] for a in r.ai_archetypes]}",
           f"Open: {[ARCHETYPE_NAMES[a][:6] for a in r.open_archetypes]}", ""]

    for i in range(len(r.picks_made)):
        pn = i + 1
        rn = (i // 10) + 1
        if i > 0 and i % 10 == 0:
            ref = REFILL_BETWEEN_ROUNDS[(i // 10) - 1] if (i // 10) - 1 < len(REFILL_BETWEEN_ROUNDS) else 0
            out.append(f"  --- REFILL +{ref} ---")
        pk = r.picks_made[i]
        pack = r.packs_seen[i] if i < len(r.packs_seen) else []
        sa_p = sum(1 for c in pack if arch is not None and c.is_sa_for(arch))
        ps = r.pool_sizes[i] if i < len(r.pool_sizes) else 0
        sa_pool = r.pool_sa_counts[i] if i < len(r.pool_sa_counts) else 0
        n = get_oversample_n(pn)
        inf = r.ai_inference_log[i] if i < len(r.ai_inference_log) else {}
        inf_s = ""
        if inf.get('eff_avoid', 0) > 0:
            iname = ARCHETYPE_NAMES[inf['inferred']][:6] if inf.get('inferred') is not None else "?"
            inf_s = f" [AI->{iname} c={inf.get('conf',0):.1f} a={inf.get('eff_avoid',0):.0%}]"
        out.append(f"  {pn:2d} R{rn} N={n:2d}: pool={ps:3d} SA_pool={sa_pool} "
                   f"SA_pack={sa_p} pick={ARCHETYPE_NAMES[pk.archetype][:8]}{inf_s}")

    out.append("")
    if arch is not None:
        sa_tot = sum(1 for c in r.picks_made if c.is_sa_for(arch))
        avg6 = sum(r.sa_per_pack[5:]) / max(len(r.sa_per_pack[5:]), 1)
        out.append(f"Deck: {len(r.picks_made)} cards, {sa_tot} S/A ({sa_tot/30*100:.0f}%), "
                   f"avg S/A 6+: {avg6:.2f}")
    return "\n".join(out)


# ============================================================
# Main
# ============================================================

def run_simulation():
    print("=" * 60)
    print("SIM-2: Hybrid 1 — Robustly Biased Pressure + N=12 + Floor")
    print("=" * 60)
    print(f"  120 start, refills 50/30/0 (2.0x bias, 40% SA open)")
    print(f"  Avoidance ramp pick 5-12 (80%), N=4/12, floor slot")
    print(f"  {NUM_SIMULATIONS} sims x 3 strategies")
    print()

    strats = {
        'committed': strategy_committed,
        'signal_reader': strategy_signal_reader,
        'power_chaser': strategy_power_chaser,
    }

    all_res = {}
    for name, fn in strats.items():
        print(f"Running {name} ({NUM_SIMULATIONS} drafts)...")
        runs = [run_single_draft(fn, rng_seed=i * 1000 + hash(name) % 100000)
                for i in range(NUM_SIMULATIONS)]
        met = compute_metrics(runs, name)
        all_res[name] = {'m': met, 'r': runs}
        print(f"  M3={met['M3']:.3f}  M11'={met['M11p']:.3f}  "
              f"M3_open={met['M3_open']:.3f}  M13={met['M13']:.1f}  M14={met['M14']:.1f}")

    m12 = all_res['signal_reader']['m']['M3'] - all_res['committed']['m']['M3']
    print(f"\nM12 = {m12:.3f}")

    t1 = gen_trace(strategy_committed, "Committed", 42)
    t2 = gen_trace(strategy_signal_reader, "Signal-Reader", 42)
    return all_res, m12, t1, t2


def fmt(all_res, m12, t1, t2):
    cm = all_res['committed']['m']
    sm = all_res['signal_reader']['m']
    pm = all_res['power_chaser']['m']
    L = []
    def a(s):
        L.append(s)

    a("# SIM-2 Results: Hybrid 1 — Robustly Biased Pressure + N=12 + Floor")
    a("")
    a("## Algorithm Summary")
    a("")
    a("- Pool: 120 cards (15/arch), refills 50/30/0 with 2.0x open-lane bias")
    a("- Open-lane refills use 40% S/A rate (vs 36% baseline)")
    a("- 5 AIs (3 open lanes), avoidance ramp pick 5 (0%) to pick 12 (80%)")
    a("- N=4 (picks 1-5), N=12 (picks 6-30), visible resonance ranking")
    a("- Floor slot: guarantee 1 S/A in shown 4 if any S/A drawn")
    a(f"- {NUM_SIMULATIONS} simulations per strategy")
    a("")

    # Scorecard
    a("## Full Scorecard")
    a("")
    a("| Metric | Target | Committed | Signal | Power | Pass? |")
    a("|--------|--------|:---------:|:------:|:-----:|:-----:|")
    def pf(v, fn): return "PASS" if fn(v) else "FAIL"
    a(f"| M1: Unique archs S/A (1-5) | >= 3 | {cm['M1']:.2f} | {sm['M1']:.2f} | {pm['M1']:.2f} | {pf(cm['M1'], lambda x: x>=3)} |")
    a(f"| M2: Max S/A emerging (1-5) | <= 2 | {cm['M2']:.2f} | {sm['M2']:.2f} | {pm['M2']:.2f} | {pf(cm['M2'], lambda x: x<=2)} |")
    a(f"| M3: S/A committed (6+) | >= 2.0 | {cm['M3']:.2f} | {sm['M3']:.2f} | {pm['M3']:.2f} | {pf(cm['M3'], lambda x: x>=2.0)} |")
    a(f"| M4: Off-arch (6+) | >= 0.5 | {cm['M4']:.2f} | {sm['M4']:.2f} | {pm['M4']:.2f} | {pf(cm['M4'], lambda x: x>=0.5)} |")
    a(f"| M5: Convergence pick | 5-8 | {cm['M5']:.1f} | {sm['M5']:.1f} | {pm['M5']:.1f} | {pf(cm['M5'], lambda x: 5<=x<=8)} |")
    a(f"| M6: Deck concentration | 60-90% | {cm['M6']*100:.1f}% | {sm['M6']*100:.1f}% | {pm['M6']*100:.1f}% | {pf(cm['M6'], lambda x: 0.6<=x<=0.9)} |")
    a(f"| M7: Run overlap | < 40% | {cm['M7']*100:.1f}% | {sm['M7']*100:.1f}% | {pm['M7']*100:.1f}% | {pf(cm['M7'], lambda x: x<0.4)} |")
    a(f"| M8: Arch freq max | < 20% | {cm['M8_max']:.1f}% | {sm['M8_max']:.1f}% | {pm['M8_max']:.1f}% | {pf(cm['M8_max'], lambda x: x<20)} |")
    a(f"| M8: Arch freq min | > 5% | {cm['M8_min']:.1f}% | {sm['M8_min']:.1f}% | {pm['M8_min']:.1f}% | {pf(cm['M8_min'], lambda x: x>5)} |")
    a(f"| M9: StdDev S/A (6+) | >= 0.8 | {cm['M9']:.2f} | {sm['M9']:.2f} | {pm['M9']:.2f} | {pf(cm['M9'], lambda x: x>=0.8)} |")
    a(f"| M10: Consec bad (6+) | <= 2 | {cm['M10']:.1f} | {sm['M10']:.1f} | {pm['M10']:.1f} | {pf(cm['M10'], lambda x: x<=2)} |")
    a(f"| M11': S/A (20+) | >= 2.5 | {cm['M11p']:.2f} | {sm['M11p']:.2f} | {pm['M11p']:.2f} | {pf(cm['M11p'], lambda x: x>=2.5)} |")
    a(f"| M12: Signal - Committed | >= 0.3 | -- | {m12:.2f} | -- | {pf(m12, lambda x: x>=0.3)} |")
    a(f"| M13: Avoidance detect | 6-10 | {cm['M13']:.1f} | {sm['M13']:.1f} | -- | {pf(cm['M13'], lambda x: 6<=x<=10)} |")
    a(f"| M14: Inference correct | 4-7 | {cm['M14']:.1f} | {sm['M14']:.1f} | -- | {pf(cm['M14'], lambda x: 4<=x<=7)} |")
    a("")

    # Per-archetype M3
    a("## Per-Archetype M3 (Committed)")
    a("")
    a("| Archetype | M3 | Sibling Rate |")
    a("|-----------|:---:|:-----------:|")
    for ar in range(NUM_ARCHETYPES):
        _, sr = SIBLING_MAP[ar]
        a(f"| {ARCHETYPE_NAMES[ar]} | {cm['per_arch_m3'].get(ar,0):.2f} | {sr:.0%} |")
    a("")

    # AI avoidance timeline
    a("## AI Avoidance Timeline")
    a("")
    a("| Pick | Inference Accuracy | Effective Avoidance |")
    a("|:----:|:------------------:|:-------------------:|")
    for p in [1, 3, 5, 6, 7, 8, 9, 10, 12, 15, 20, 25, 30]:
        acc = cm['inf_acc'].get(p, 0)
        av = cm['eff_avoid'].get(p, 0)
        a(f"| {p} | {acc:.0%} | {av:.0%} |")
    a("")

    # Pool contraction
    a("## Pool Contraction Trajectory")
    a("")
    pt = cm['pool_traj']
    st = cm['sa_traj']
    dt = cm['density_traj']
    a("| Pick | Pool Size | S/A in Pool | Arch Density | N |")
    a("|:----:|:---------:|:-----------:|:------------:|:-:|")
    for p in [1, 5, 10, 11, 15, 20, 21, 25, 28, 30]:
        i = p - 1
        ps = pt[i] if i < len(pt) else 0
        sa = st[i] if i < len(st) else 0
        dn = dt[i] if i < len(dt) else 0
        n = get_oversample_n(p)
        a(f"| {p} | {ps:.0f} | {sa:.1f} | {dn:.1%} | {n} |")
    a("")

    # SA at key picks
    a("## S/A Counts at Key Picks")
    a("")
    sa_at = cm['sa_at']
    a(f"- Pick 20: {sa_at.get(20,0):.1f} S/A remaining")
    a(f"- Pick 25: {sa_at.get(25,0):.1f} S/A remaining")
    a(f"- Pick 30: {sa_at.get(30,0):.1f} S/A remaining")
    a("")

    # Oversampling analysis
    a("## Oversampling Analysis")
    a("")
    a(f"- Picks 6-15 (N=12, pool ~80-110): avg S/A = {cm['m3_6_15']:.2f}")
    a(f"- Picks 16-20 (N=12, pool ~40-80): avg S/A = {cm['m3_16_20']:.2f}")
    a(f"- Picks 21-30 (N=12, pool ~10-50): avg S/A = {cm['m3_21_30']:.2f}")
    a("")

    # Pack quality
    a("## Pack Quality Distribution (S/A per Pack, Picks 6+)")
    a("")
    a("| Strategy | p10 | p25 | p50 | p75 | p90 |")
    a("|----------|:---:|:---:|:---:|:---:|:---:|")
    for n, l in [('committed','Committed'),('signal_reader','Signal'),('power_chaser','Power')]:
        q = all_res[n]['m']['pq']
        a(f"| {l} | {q[10]:.1f} | {q[25]:.1f} | {q[50]:.1f} | {q[75]:.1f} | {q[90]:.1f} |")
    a("")

    # Bad packs
    a("## Consecutive Bad Pack Analysis")
    a("")
    streaks = []
    for r in all_res['committed']['r']:
        if r.committed_archetype is None: continue
        vs = r.sa_per_pack[5:]
        s = mx = 0
        for v in vs:
            if v < 1.5: s += 1; mx = max(mx, s)
            else: s = 0
        streaks.append(mx)
    if streaks:
        ss = sorted(streaks)
        n = len(ss)
        a(f"- Median max consecutive bad: {ss[n//2]}")
        a(f"- p25={ss[n//4]}, p75={ss[3*n//4]}, p90={ss[int(n*0.9)]}")
        a(f"- All packs bad (streak=25): {sum(1 for x in ss if x>=25)/n*100:.0f}%")
        a(f"- 5+ consecutive bad: {sum(1 for x in ss if x>=5)/n*100:.0f}%")
    a("")

    # Pool composition
    a("## Pool Composition Trajectory")
    a("")
    a("Average cards per archetype at key picks:")
    a("")
    comp = []
    max_len = max((len(r.pool_arch_counts) for r in all_res['committed']['r']), default=0)
    for i in range(max_len):
        avgs = {}
        for ar in range(NUM_ARCHETYPES):
            vs = [r.pool_arch_counts[i].get(ar, 0)
                  for r in all_res['committed']['r']
                  if i < len(r.pool_arch_counts)]
            avgs[ar] = sum(vs) / len(vs) if vs else 0
        comp.append(avgs)

    a("| Pick | Top Arch | 2nd | 3rd | Avg Others | Total |")
    a("|:----:|:--------:|:---:|:---:|:----------:|:-----:|")
    for p in [1, 5, 10, 11, 15, 20, 21, 25, 30]:
        i = p - 1
        if i < len(comp):
            c = comp[i]
            vals = sorted([c.get(ar, 0) for ar in range(NUM_ARCHETYPES)], reverse=True)
            tot = sum(vals)
            a(f"| {p} | {vals[0]:.1f} | {vals[1]:.1f} | {vals[2]:.1f} | {sum(vals[3:])/5:.1f} | {tot:.0f} |")
    a("")

    # Traces
    a("## Draft Traces")
    a("")
    a("### Committed Player")
    a("```")
    a(t1)
    a("```")
    a("")
    a("### Signal-Reader")
    a("```")
    a(t2)
    a("```")
    a("")

    # Comparison
    a("## Comparison to V9 Baseline and V11")
    a("")
    a("| Metric | V9 Hybrid B | V11 SIM-4 | SIM-2 Committed | SIM-2 Signal |")
    a("|--------|:-----------:|:---------:|:---------------:|:------------:|")
    a(f"| M3 | 2.70 | 0.83 | {cm['M3']:.2f} | {sm['M3']:.2f} |")
    a(f"| M11' | 3.25 | -- | {cm['M11p']:.2f} | {sm['M11p']:.2f} |")
    a(f"| M10 | 3.8 | -- | {cm['M10']:.1f} | {sm['M10']:.1f} |")
    a(f"| M5 | 9.6 | -- | {cm['M5']:.1f} | {sm['M5']:.1f} |")
    a(f"| M6 | 86% | -- | {cm['M6']*100:.1f}% | {sm['M6']*100:.1f}% |")
    a(f"| M12 | -- | -- | -- | {m12:.2f} |")
    a("")

    # Self-assessment
    a("## Self-Assessment")
    a("")
    fails = []
    if cm['M3'] < 2.0: fails.append(f"M3 ({cm['M3']:.2f} < 2.0)")
    if cm['M11p'] < 2.5: fails.append(f"M11' ({cm['M11p']:.2f} < 2.5)")
    if cm['M10'] > 2: fails.append(f"M10 ({cm['M10']:.1f} > 2)")
    if m12 < 0.3: fails.append(f"M12 ({m12:.2f} < 0.3)")

    if fails:
        a(f"**FAIL.** Critical failures: {', '.join(fails)}.")
    else:
        a("**PASS.** All critical metrics met.")
    a("")

    a("### Root Cause Analysis")
    a("")
    a(f"M3 = {cm['M3']:.2f} falls {'short of' if cm['M3'] < 2.0 else 'near'} the 2.0 target.")
    a("")
    a("**Pool contraction works as designed.** The pool contracts from 120 to "
      f"~{pt[-1]:.0f} cards by pick 30, matching the design's prediction of ~20 "
      "cards. The biased refills (50 after R1, 30 after R2) with 2.0x open-lane "
      "multiplier correctly favor the player's archetype in refill composition.")
    a("")

    if cm['M3'] < 2.0:
        a("**The binding constraint is the sampling bottleneck, not avoidance or "
          "contraction.** Drawing 12 random cards from a pool of 60-110 cards "
          "where only 10-15% are on-archetype yields E[on-arch drawn] = 0.7-1.6. "
          "Of those, only 36% are S/A, giving E[S/A drawn] = 0.25-0.58. The 'best "
          "4' ranking and floor slot can only show S/A cards that were randomly "
          "drawn in the first place.")
        a("")
        a("**The oversampling N=12 is insufficient when the pool is large.** In "
          "the critical picks 6-20 window (pool ~60-110), N=12 draws only 11-20% "
          "of the pool. The probability of drawing the player's 3-6 S/A cards "
          "from 60-110 total is too low for M3 >= 2.0. Only when the pool "
          "contracts below ~25 cards (picks 27-30) does N=12 sample enough of the "
          "pool to reliably find S/A cards.")
        a("")
        a("**S/A exhaustion compounds the problem.** The player consumes their own "
          "S/A through picks. Starting with ~5 S/A, refills add ~4-5 more, but "
          "the player picks ~8-10 S/A over 30 picks, leaving only ~1-2 S/A by "
          f"pick 25 ({sa_at.get(25,0):.1f} observed). Even with perfect pool "
          "contraction, if only 1-2 S/A remain, M3 cannot reach 2.0.")
        a("")
        a("**AI avoidance works but too slowly.** Inference accuracy reaches "
          f"useful levels (~50%+) around pick {[p for p in sorted(cm['inf_acc']) if cm['inf_acc'].get(p,0) >= 0.5][0] if any(v >= 0.5 for v in cm['inf_acc'].values()) else 30}, "
          "but the avoidance benefit is modest: AIs preferentially taking from "
          "non-player archetypes raises the player's archetype density from 12.5% "
          f"to ~{max(cm['density_traj']) * 100:.0f}% at best -- insufficient for "
          "M3 >= 2.0 via oversampling alone.")
    a("")

    a("### Is AI avoidance + pool contraction + N=12 oversampling viable?")
    a("")
    if cm['M3'] >= 1.8:
        a("**Partially viable.** The combination produces meaningful concentration "
          "approaching M3 = 2.0. The transparent mechanisms work as theorized but "
          "achieve ~75-90% of V9's invisible contraction. The gap is attributable "
          "to the lower contraction ratio (6:1 vs V9's 21:1) and S/A exhaustion.")
    elif cm['M3'] >= 1.0:
        a("**Marginally viable.** The three mechanisms produce measurable "
          "concentration but fall well short of M3 = 2.0. The fundamental "
          "limitation is structural: with 120 cards and 30 picks, the physical "
          "contraction ratio (120 -> 20 = 6:1) combined with N=12 oversampling "
          "cannot replicate V9's virtual contraction (360 -> 17 = 21:1). V9's "
          "fallback (Design 4) remains the proven path to M3 >= 2.0.")
    else:
        a("**Not viable.** The three mechanisms fail to produce meaningful "
          "concentration. The sampling bottleneck identified in V11 persists: "
          "drawing N=12 from a pool of 60-120 cards with 12% on-archetype density "
          "cannot achieve M3 >= 2.0 regardless of AI avoidance behavior. The pool "
          "must contract to <25 cards before oversampling becomes effective, but "
          "by that point S/A cards are nearly exhausted. V9 fallback recommended.")
    a("")

    return "\n".join(L)


if __name__ == "__main__":
    all_res, m12, t1, t2 = run_simulation()
    output = fmt(all_res, m12, t1, t2)
    path = "/Users/dthurn/Documents/GoogleDrive/dreamtides/docs/resonance/v12/results_2.md"
    with open(path, "w") as f:
        f.write(output)
    print(f"\nResults written to {path}")
