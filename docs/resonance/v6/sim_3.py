#!/usr/bin/env python3
"""
Simulation Agent 3: Threshold-Triggered Soft Locks

One-sentence algorithm:
"Each pack slot starts random; when your top resonance count crosses 3,
6, and 9, one more slot begins showing a resonance-matched card 75% of
the time (first two slots target top resonance, third targets second-
highest), while slot 4 stays fully random."

This simulation tests the soft-lock variant (75% probability) as primary,
with parameter sweeps covering binary locks (100%) and other variants.
"""

import random
import math
from dataclasses import dataclass, field
from collections import Counter, defaultdict

# ============================================================================
# Constants
# ============================================================================

NUM_DRAFTS = 1000
NUM_PICKS = 30
PACK_SIZE = 4
NUM_ARCHETYPES = 8
TOTAL_CARDS = 360
GENERIC_CARDS = 36
ARCHETYPE_CARDS = 324

RESONANCES = ["Zephyr", "Ember", "Stone", "Tide"]

# Archetype definitions: (name, primary_resonance, secondary_resonance)
# Arranged in a circle: 0-1-2-3-4-5-6-7-0
ARCHETYPES = [
    ("Flash",         "Zephyr", "Ember"),   # 0
    ("Blink",         "Ember",  "Zephyr"),  # 1
    ("Storm",         "Ember",  "Stone"),   # 2
    ("Self-Discard",  "Stone",  "Ember"),   # 3
    ("Self-Mill",     "Stone",  "Tide"),    # 4
    ("Sacrifice",     "Tide",   "Stone"),   # 5
    ("Warriors",      "Tide",   "Zephyr"),  # 6
    ("Ramp",          "Zephyr", "Tide"),    # 7
]


def circle_distance(a, b):
    """Distance between two archetypes on the circle (0..4)."""
    d = abs(a - b)
    return min(d, NUM_ARCHETYPES - d)


def compute_fitness_map():
    """Compute fitness tier for each (card_archetype, eval_archetype) pair.

    Rules from orchestration plan:
    - S in home archetype
    - A in the adjacent archetype sharing primary resonance
    - B in archetypes sharing secondary resonance
    - C/F in distant archetypes (C at distance 2-3, F at distance 4)
    """
    fitness = {}
    for card_idx in range(NUM_ARCHETYPES):
        _, card_pri, card_sec = ARCHETYPES[card_idx]
        for eval_idx in range(NUM_ARCHETYPES):
            _, eval_pri, eval_sec = ARCHETYPES[eval_idx]
            dist = circle_distance(card_idx, eval_idx)

            if card_idx == eval_idx:
                fitness[(card_idx, eval_idx)] = "S"
            elif card_pri == eval_pri:
                # Adjacent archetype sharing primary resonance (dist=1 always)
                fitness[(card_idx, eval_idx)] = "A"
            elif dist == 1:
                # Adjacent but not sharing primary -- shares secondary
                fitness[(card_idx, eval_idx)] = "B"
            elif card_pri == eval_sec or card_sec == eval_pri or card_sec == eval_sec:
                # Shares some resonance at distance 2+
                fitness[(card_idx, eval_idx)] = "B"
            elif dist <= 3:
                fitness[(card_idx, eval_idx)] = "C"
            else:
                fitness[(card_idx, eval_idx)] = "F"
    return fitness


FITNESS_MAP = compute_fitness_map()


# ============================================================================
# Card Pool Construction
# ============================================================================

@dataclass
class SimCard:
    id: int
    symbols: list
    archetype_idx: int  # -1 for generic
    archetype_name: str
    power: float
    rarity: str

    @property
    def is_generic(self):
        return self.archetype_idx == -1

    @property
    def is_dual_type(self):
        return len(set(self.symbols)) == 2

    def fitness_for(self, archetype_idx):
        if self.is_generic:
            return "B"
        return FITNESS_MAP.get((self.archetype_idx, archetype_idx), "F")

    def is_sa(self, archetype_idx):
        return self.fitness_for(archetype_idx) in ("S", "A")

    def is_cf(self, archetype_idx):
        return self.fitness_for(archetype_idx) in ("C", "F")

    def weighted_symbols(self):
        """Return dict of resonance -> weighted count for this card."""
        counts = defaultdict(int)
        for i, sym in enumerate(self.symbols):
            counts[sym] += 2 if i == 0 else 1
        return dict(counts)


def build_card_pool(dual_type_count=54):
    """Build the 360-card pool."""
    cards = []
    card_id = 0

    # Generic cards
    for _ in range(GENERIC_CARDS):
        cards.append(SimCard(
            id=card_id, symbols=[], archetype_idx=-1,
            archetype_name="Generic",
            power=round(random.uniform(3, 7), 1), rarity="common"
        ))
        card_id += 1

    dual_per_arch = dual_type_count // NUM_ARCHETYPES
    dual_remainder = dual_type_count % NUM_ARCHETYPES
    cards_per_archetype = ARCHETYPE_CARDS // NUM_ARCHETYPES

    for arch_idx in range(NUM_ARCHETYPES):
        arch_name, pri_res, sec_res = ARCHETYPES[arch_idx]
        n_cards = cards_per_archetype + (1 if arch_idx < ARCHETYPE_CARDS % NUM_ARCHETYPES else 0)
        n_dual = dual_per_arch + (1 if arch_idx < dual_remainder else 0)
        n_mono = n_cards - n_dual

        # Dual-type cards
        n_dual_2sym = (n_dual + 1) // 2
        n_dual_3sym = n_dual - n_dual_2sym

        for _ in range(n_dual_2sym):
            cards.append(SimCard(
                id=card_id, symbols=[pri_res, sec_res],
                archetype_idx=arch_idx, archetype_name=arch_name,
                power=round(random.uniform(3, 9), 1),
                rarity=random.choice(["common", "common", "uncommon", "rare"])
            ))
            card_id += 1

        for _ in range(n_dual_3sym):
            cards.append(SimCard(
                id=card_id, symbols=[pri_res, pri_res, sec_res],
                archetype_idx=arch_idx, archetype_name=arch_name,
                power=round(random.uniform(3, 9), 1),
                rarity=random.choice(["common", "common", "uncommon", "rare"])
            ))
            card_id += 1

        # Mono-type cards: 20% 1-sym, 50% 2-sym, 30% 3-sym
        n_mono_1 = max(1, round(n_mono * 0.20))
        n_mono_3 = max(1, round(n_mono * 0.30))
        n_mono_2 = n_mono - n_mono_1 - n_mono_3

        for _ in range(n_mono_1):
            cards.append(SimCard(
                id=card_id, symbols=[pri_res],
                archetype_idx=arch_idx, archetype_name=arch_name,
                power=round(random.uniform(2, 8), 1), rarity="common"
            ))
            card_id += 1

        for _ in range(n_mono_2):
            cards.append(SimCard(
                id=card_id, symbols=[pri_res, pri_res],
                archetype_idx=arch_idx, archetype_name=arch_name,
                power=round(random.uniform(3, 9), 1),
                rarity=random.choice(["common", "common", "uncommon"])
            ))
            card_id += 1

        for _ in range(n_mono_3):
            cards.append(SimCard(
                id=card_id, symbols=[pri_res, pri_res, pri_res],
                archetype_idx=arch_idx, archetype_name=arch_name,
                power=round(random.uniform(4, 10), 1),
                rarity=random.choice(["common", "uncommon", "rare"])
            ))
            card_id += 1

    return cards


# ============================================================================
# Threshold-Triggered Soft Locks Algorithm
# ============================================================================

@dataclass
class SoftLockState:
    """Tracks per-player soft lock state."""
    resonance_counts: dict = field(default_factory=lambda: defaultdict(int))
    slot_locks: list = field(default_factory=lambda: [None, None, None, None])
    thresholds_triggered: int = 0

    def add_card_symbols(self, card):
        wsym = card.weighted_symbols()
        for res, count in wsym.items():
            self.resonance_counts[res] += count

    def top_resonance(self):
        if not self.resonance_counts:
            return None
        return max(self.resonance_counts, key=lambda r: self.resonance_counts[r])

    def second_resonance(self):
        if len(self.resonance_counts) < 2:
            return None
        sorted_res = sorted(self.resonance_counts,
                            key=lambda r: self.resonance_counts[r], reverse=True)
        return sorted_res[1] if len(sorted_res) >= 2 else None

    def check_thresholds(self, thresholds):
        """Check and trigger soft locks."""
        top_res = self.top_resonance()
        if top_res is None:
            return
        top_count = self.resonance_counts[top_res]

        if self.thresholds_triggered < 1 and top_count >= thresholds[0]:
            self.slot_locks[0] = top_res
            self.thresholds_triggered = 1

        if self.thresholds_triggered < 2 and top_count >= thresholds[1]:
            self.slot_locks[1] = top_res
            self.thresholds_triggered = 2

        # Third lock targets SECOND resonance (split-resonance targeting)
        if self.thresholds_triggered < 3 and top_count >= thresholds[2]:
            second_res = self.second_resonance()
            self.slot_locks[2] = second_res if second_res else top_res
            self.thresholds_triggered = 3


def generate_pack(pool, state, lock_probability=0.75):
    """Generate a pack of 4 cards using the soft lock algorithm."""
    pack = []
    used_ids = set()

    for slot_idx in range(PACK_SIZE):
        locked_res = state.slot_locks[slot_idx]
        card = None

        if locked_res is not None and random.random() < lock_probability:
            candidates = [c for c in pool if c.id not in used_ids
                          and len(c.symbols) > 0 and c.symbols[0] == locked_res]
            if candidates:
                card = random.choice(candidates)

        if card is None:
            candidates = [c for c in pool if c.id not in used_ids]
            if candidates:
                card = random.choice(candidates)

        if card is not None:
            pack.append(card)
            used_ids.add(card.id)

    return pack


# ============================================================================
# Player Strategies
# ============================================================================

def determine_committed_archetype(drafted):
    """Determine which archetype the drafted cards most belong to.

    Uses S-tier count as primary tiebreaker to distinguish between archetypes
    sharing the same resonance.
    """
    if not drafted:
        return random.randint(0, NUM_ARCHETYPES - 1)
    best_arch = 0
    best_score = (-1, -1)
    for arch_idx in range(NUM_ARCHETYPES):
        sa = sum(1 for c in drafted if c.is_sa(arch_idx))
        s_only = sum(1 for c in drafted if c.fitness_for(arch_idx) == "S")
        score = (sa, s_only)
        if score > best_score:
            best_score = score
            best_arch = arch_idx
    return best_arch


def pick_archetype_committed(pack, drafted, pick_num):
    """Picks highest fitness for strongest archetype. Commits around pick 5-6."""
    tier_val = {"S": 5, "A": 4, "B": 3, "C": 2, "F": 1}
    if pick_num < 5:
        # Early: pick best card considering emerging archetype leanings
        # Score each card by its best fitness + slight bias toward invested archetypes
        arch_investment = defaultdict(float)
        for d in drafted:
            for aidx in range(NUM_ARCHETYPES):
                if d.fitness_for(aidx) == "S":
                    arch_investment[aidx] += 2.0
                elif d.fitness_for(aidx) == "A":
                    arch_investment[aidx] += 1.0

        best_card = None
        best_score = -999
        for card in pack:
            card_score = 0
            for aidx in range(NUM_ARCHETYPES):
                t = tier_val.get(card.fitness_for(aidx), 0)
                bonus = arch_investment[aidx] * 0.2
                card_score = max(card_score, t + bonus)
            card_score += card.power * 0.05
            if card_score > best_score:
                best_score = card_score
                best_card = card
        return best_card if best_card else pack[0]
    else:
        committed = determine_committed_archetype(drafted)
        return max(pack, key=lambda c: (
            tier_val.get(c.fitness_for(committed), 0), c.power
        ))


def pick_power_chaser(pack, drafted, pick_num):
    """Picks highest raw power regardless of archetype."""
    return max(pack, key=lambda c: c.power)


def pick_signal_reader(pack, drafted, pick_num):
    """Evaluates which archetype seems most available and drafts toward it."""
    tier_val = {"S": 5, "A": 4, "B": 3, "C": 2, "F": 1}
    if pick_num < 3:
        return max(pack, key=lambda c: c.power)

    # Evaluate archetype availability in current pack
    arch_avail = defaultdict(float)
    for card in pack:
        for aidx in range(NUM_ARCHETYPES):
            if card.is_sa(aidx):
                arch_avail[aidx] += 1

    # Combine with investment from drafted cards
    arch_invest = defaultdict(float)
    for d in drafted:
        for aidx in range(NUM_ARCHETYPES):
            if d.fitness_for(aidx) == "S":
                arch_invest[aidx] += 1.5
            elif d.fitness_for(aidx) == "A":
                arch_invest[aidx] += 0.8

    best_arch = max(range(NUM_ARCHETYPES),
                    key=lambda a: arch_avail[a] * 0.5 + arch_invest[a] * 0.5)

    return max(pack, key=lambda c: (
        tier_val.get(c.fitness_for(best_arch), 0), c.power
    ))


STRATEGIES = {
    "archetype_committed": pick_archetype_committed,
    "power_chaser": pick_power_chaser,
    "signal_reader": pick_signal_reader,
}


# ============================================================================
# Simulation Engine
# ============================================================================

@dataclass
class DraftRecord:
    """Records everything about a single draft for post-hoc analysis."""
    packs: list = field(default_factory=list)  # list of (pick_num, pack_cards)
    picks: list = field(default_factory=list)  # list of picked cards
    lock_states: list = field(default_factory=list)  # thresholds_triggered at each pick
    convergence_pick: int = 30


def run_single_draft(pool, strategy_fn, thresholds=(3, 6, 9),
                     lock_probability=0.75):
    """Run a single 30-pick draft."""
    state = SoftLockState()
    record = DraftRecord()
    drafted = []

    for pick_num in range(NUM_PICKS):
        pack = generate_pack(pool, state, lock_probability)
        if len(pack) < PACK_SIZE:
            break

        record.packs.append(pack)
        record.lock_states.append(state.thresholds_triggered)

        card = strategy_fn(pack, drafted, pick_num)
        drafted.append(card)
        record.picks.append(card)

        state.add_card_symbols(card)
        state.check_thresholds(thresholds)

        # Check convergence: first pick >= 5 where committed arch gets 2+ S/A
        if pick_num >= 4 and record.convergence_pick == 30:
            committed = determine_committed_archetype(drafted)
            sa_in_pack = sum(1 for c in pack if c.is_sa(committed))
            if sa_in_pack >= 2:
                record.convergence_pick = pick_num + 1

    return record


def run_simulation(pool, thresholds=(3, 6, 9), lock_probability=0.75,
                   num_drafts=NUM_DRAFTS):
    """Run full simulation across all strategies."""
    all_results = {}
    for strat_name, strat_fn in STRATEGIES.items():
        records = []
        for _ in range(num_drafts):
            rec = run_single_draft(pool, strat_fn, thresholds, lock_probability)
            records.append(rec)
        all_results[strat_name] = records
    return all_results


# ============================================================================
# Metrics Computation (all at ARCHETYPE level, post-hoc)
# ============================================================================

def compute_metrics(all_results, label=""):
    """Compute and return all 9 metrics + supporting data."""
    lines = []

    def out(s=""):
        lines.append(s)

    out(f"\n{'='*70}")
    out(f"  METRICS: {label}")
    out(f"{'='*70}")

    summary = {}  # strategy -> dict of metric values

    for strat_name, records in all_results.items():
        out(f"\n--- Strategy: {strat_name} ({len(records)} drafts) ---")
        sm = {}

        # Determine committed archetype for each draft
        committed_archs = []
        for rec in records:
            arch = determine_committed_archetype(rec.picks)
            committed_archs.append(arch)

        # M1: Picks 1-5: unique archetypes with S/A cards per pack (>= 3)
        early_unique = []
        for rec in records:
            for pick_num in range(min(5, len(rec.packs))):
                pack = rec.packs[pick_num]
                archs = set()
                for card in pack:
                    for aidx in range(NUM_ARCHETYPES):
                        if card.is_sa(aidx):
                            archs.add(aidx)
                early_unique.append(len(archs))
        sm["m1"] = sum(early_unique) / max(len(early_unique), 1)
        out(f"  M1 Picks 1-5: unique archs with S/A per pack: {sm['m1']:.2f} (target >= 3)")

        # M2: Picks 1-5: S/A for emerging archetype per pack (<= 2)
        early_sa = []
        for i, rec in enumerate(records):
            for pick_num in range(min(5, len(rec.packs))):
                pack = rec.packs[pick_num]
                # "Emerging" = best archetype from cards drafted so far
                if pick_num > 0:
                    emerging = determine_committed_archetype(rec.picks[:pick_num])
                else:
                    emerging = random.randint(0, NUM_ARCHETYPES - 1)
                sa = sum(1 for c in pack if c.is_sa(emerging))
                early_sa.append(sa)
        sm["m2"] = sum(early_sa) / max(len(early_sa), 1)
        out(f"  M2 Picks 1-5: S/A for emerging arch per pack: {sm['m2']:.2f} (target <= 2)")

        # M3: Picks 6+: S/A for committed archetype per pack (>= 2)
        late_sa = []
        for i, rec in enumerate(records):
            arch = committed_archs[i]
            for pick_num in range(5, len(rec.packs)):
                pack = rec.packs[pick_num]
                sa = sum(1 for c in pack if c.is_sa(arch))
                late_sa.append(sa)
        sm["m3"] = sum(late_sa) / max(len(late_sa), 1)
        out(f"  M3 Picks 6+: S/A for committed arch per pack: {sm['m3']:.2f} (target >= 2)")

        # M4: Picks 6+: off-archetype (C/F) cards per pack (>= 0.5)
        late_cf = []
        for i, rec in enumerate(records):
            arch = committed_archs[i]
            for pick_num in range(5, len(rec.packs)):
                pack = rec.packs[pick_num]
                cf = sum(1 for c in pack if c.is_cf(arch))
                late_cf.append(cf)
        sm["m4"] = sum(late_cf) / max(len(late_cf), 1)
        out(f"  M4 Picks 6+: C/F cards per pack: {sm['m4']:.2f} (target >= 0.5)")

        # M5: Convergence pick (5-8)
        conv_picks = [rec.convergence_pick for rec in records]
        sm["m5"] = sum(conv_picks) / len(conv_picks)
        out(f"  M5 Convergence pick: {sm['m5']:.2f} (target 5-8)")

        # M6: Deck archetype concentration (60-90%)
        concs = []
        for i, rec in enumerate(records):
            arch = committed_archs[i]
            if rec.picks:
                sa = sum(1 for c in rec.picks if c.is_sa(arch))
                concs.append(sa / len(rec.picks))
        sm["m6"] = sum(concs) / len(concs) * 100 if concs else 0
        out(f"  M6 Deck archetype concentration: {sm['m6']:.1f}% (target 60-90%)")

        # M7: Run-to-run card overlap (< 40%)
        overlaps = []
        for i in range(0, len(records) - 1, 2):
            ids1 = set(c.id for c in records[i].picks)
            ids2 = set(c.id for c in records[i + 1].picks)
            if ids1 | ids2:
                overlaps.append(len(ids1 & ids2) / len(ids1 | ids2))
        sm["m7"] = sum(overlaps) / max(len(overlaps), 1) * 100
        out(f"  M7 Run-to-run overlap: {sm['m7']:.1f}% (target < 40%)")

        # M8: Archetype frequency (no > 20%, no < 5%)
        arch_counts = Counter(committed_archs)
        out(f"  M8 Archetype frequency:")
        total = len(records)
        for aidx in range(NUM_ARCHETYPES):
            freq = arch_counts[aidx] / total * 100
            flag = " *** OUT OF RANGE" if freq > 20 or freq < 5 else ""
            out(f"      {ARCHETYPES[aidx][0]:15s}: {freq:.1f}%{flag}")

        # M9: StdDev S/A per pack picks 6+ (>= 0.8)
        if late_sa:
            mean_sa = sum(late_sa) / len(late_sa)
            var_sa = sum((x - mean_sa) ** 2 for x in late_sa) / len(late_sa)
            sm["m9"] = math.sqrt(var_sa)
        else:
            sm["m9"] = 0
        out(f"  M9 StdDev S/A per pack (picks 6+): {sm['m9']:.2f} (target >= 0.8)")

        summary[strat_name] = sm

    return lines, summary


def compute_per_archetype_convergence(all_results):
    """Per-archetype convergence table."""
    lines = []
    lines.append("\n--- Per-Archetype Convergence Table ---")
    lines.append(f"{'Archetype':15s} | {'Avg Conv':>9s} | {'Count':>6s} | {'Avg Conc':>9s}")
    lines.append("-" * 50)

    for strat_name, records in all_results.items():
        lines.append(f"\nStrategy: {strat_name}")
        arch_conv = defaultdict(list)
        arch_conc = defaultdict(list)
        for rec in records:
            arch = determine_committed_archetype(rec.picks)
            arch_conv[arch].append(rec.convergence_pick)
            if rec.picks:
                sa = sum(1 for c in rec.picks if c.is_sa(arch))
                arch_conc[arch].append(sa / len(rec.picks) * 100)

        for aidx in range(NUM_ARCHETYPES):
            name = ARCHETYPES[aidx][0]
            convs = arch_conv[aidx]
            concs = arch_conc[aidx]
            if convs:
                avg_c = sum(convs) / len(convs)
                avg_n = sum(concs) / len(concs) if concs else 0
                lines.append(f"{name:15s} | {avg_c:9.2f} | {len(convs):6d} | {avg_n:8.1f}%")
            else:
                lines.append(f"{name:15s} | {'N/A':>9s} | {0:6d} | {'N/A':>9s}")

    return lines


def compute_pack_variance(all_results):
    """Pack-quality variance report."""
    lines = []
    lines.append("\n--- Pack-Quality Variance Report ---")

    for strat_name, records in all_results.items():
        lines.append(f"\nStrategy: {strat_name}")
        late_sa = []
        for rec in records:
            arch = determine_committed_archetype(rec.picks)
            for pick_num in range(5, len(rec.packs)):
                pack = rec.packs[pick_num]
                sa = sum(1 for c in pack if c.is_sa(arch))
                late_sa.append(sa)

        if late_sa:
            mean_v = sum(late_sa) / len(late_sa)
            var_v = sum((x - mean_v) ** 2 for x in late_sa) / len(late_sa)
            std_v = math.sqrt(var_v)
            dist = Counter(late_sa)
            lines.append(f"  Mean S/A per pack (picks 6+): {mean_v:.2f}")
            lines.append(f"  StdDev: {std_v:.2f}")
            lines.append(f"  Distribution:")
            for k in sorted(dist.keys()):
                pct = dist[k] / len(late_sa) * 100
                lines.append(f"    {k} S/A cards: {pct:.1f}%")

    return lines


# ============================================================================
# Draft Traces
# ============================================================================

def run_draft_trace(pool, strategy_fn, strategy_name, thresholds=(3, 6, 9),
                    lock_probability=0.75):
    """Run a single draft with detailed trace output."""
    state = SoftLockState()
    drafted = []
    lines = []

    lines.append(f"\n=== Draft Trace: {strategy_name} ===")
    lines.append(f"    Thresholds: {thresholds}, Lock prob: {lock_probability}")

    for pick_num in range(min(NUM_PICKS, 12)):
        pack = generate_pack(pool, state, lock_probability)
        if len(pack) < PACK_SIZE:
            break

        committed = determine_committed_archetype(drafted) if drafted else 0

        top = state.top_resonance()
        sec = state.second_resonance()
        top_ct = state.resonance_counts.get(top, 0) if top else 0
        sec_ct = state.resonance_counts.get(sec, 0) if sec else 0

        lines.append(f"\n  Pick {pick_num + 1} (locks: {state.thresholds_triggered}, "
                      f"top: {top}={top_ct}, 2nd: {sec}={sec_ct}):")
        lines.append(f"    Slot locks: {state.slot_locks}")

        for i, card in enumerate(pack):
            fit = card.fitness_for(committed)
            locked = "LOCK" if state.slot_locks[i] else "rand"
            lines.append(f"    [{locked:4s}] {card.archetype_name:15s} "
                          f"sym={str(card.symbols):30s} pow={card.power:.1f} fit={fit}")

        card = strategy_fn(pack, drafted, pick_num)
        drafted.append(card)
        state.add_card_symbols(card)
        state.check_thresholds(thresholds)

        lines.append(f"    -> Picked: {card.archetype_name} {card.symbols} pow={card.power:.1f}")

    final_arch = determine_committed_archetype(drafted)
    sa_count = sum(1 for c in drafted if c.is_sa(final_arch))
    lines.append(f"\n  Final: {ARCHETYPES[final_arch][0]} "
                  f"({sa_count}/{len(drafted)} = {sa_count/len(drafted)*100:.1f}% S/A)")

    return lines


# ============================================================================
# Compact Sweep Helper
# ============================================================================

def sweep_metrics(pool, thresholds, lock_prob, num_drafts=500):
    """Run a sweep and return key metrics for archetype_committed only."""
    records = []
    for _ in range(num_drafts):
        rec = run_single_draft(pool, pick_archetype_committed, thresholds, lock_prob)
        records.append(rec)

    late_sa = []
    conv_picks = []
    concs = []
    early_unique = []
    committed_archs = []

    for rec in records:
        arch = determine_committed_archetype(rec.picks)
        committed_archs.append(arch)
        conv_picks.append(rec.convergence_pick)
        if rec.picks:
            sa = sum(1 for c in rec.picks if c.is_sa(arch))
            concs.append(sa / len(rec.picks))
        for pick_num in range(5, len(rec.packs)):
            pack = rec.packs[pick_num]
            sa = sum(1 for c in pack if c.is_sa(arch))
            late_sa.append(sa)
        for pick_num in range(min(5, len(rec.packs))):
            pack = rec.packs[pick_num]
            archs = set()
            for card in pack:
                for aidx in range(NUM_ARCHETYPES):
                    if card.is_sa(aidx):
                        archs.add(aidx)
            early_unique.append(len(archs))

    mean_sa = sum(late_sa) / max(len(late_sa), 1)
    std_sa = math.sqrt(sum((x - mean_sa)**2 for x in late_sa) / max(len(late_sa), 1)) if late_sa else 0
    mean_conv = sum(conv_picks) / len(conv_picks)
    mean_conc = sum(concs) / len(concs) * 100 if concs else 0
    mean_early = sum(early_unique) / max(len(early_unique), 1)

    # Archetype balance
    arch_counts = Counter(committed_archs)
    arch_freqs = {ARCHETYPES[a][0]: arch_counts[a] / len(records) * 100
                  for a in range(NUM_ARCHETYPES)}

    return {
        "sa": mean_sa, "std": std_sa, "conv": mean_conv,
        "conc": mean_conc, "early_div": mean_early,
        "arch_freqs": arch_freqs
    }


# ============================================================================
# Main
# ============================================================================

def main():
    random.seed(42)

    print("Building card pool...")
    pool = build_card_pool(dual_type_count=54)

    # Validate pool
    dual_count = sum(1 for c in pool if c.is_dual_type)
    generic_count = sum(1 for c in pool if c.is_generic)
    print(f"Pool: {len(pool)} cards, {generic_count} generic, {dual_count} dual-type "
          f"({dual_count / len(pool) * 100:.1f}%)")

    per_arch = Counter(c.archetype_name for c in pool)
    print("Per archetype:", dict(per_arch))

    # Validate fitness map
    print("\nFitness map sample (Warriors card -> each archetype):")
    warriors_idx = 6
    for aidx in range(NUM_ARCHETYPES):
        print(f"  Warriors -> {ARCHETYPES[aidx][0]:15s}: {FITNESS_MAP[(warriors_idx, aidx)]}")

    # ==================================================================
    # PRIMARY SIMULATION
    # ==================================================================
    print("\n" + "=" * 70)
    print("  PRIMARY: Thresholds (3,6,9), Lock Prob 0.75")
    print("=" * 70)

    primary_results = run_simulation(pool, thresholds=(3, 6, 9),
                                      lock_probability=0.75)
    metric_lines, primary_summary = compute_metrics(primary_results,
                                                     "Primary (3/6/9, 75%)")
    for line in metric_lines:
        print(line)

    conv_lines = compute_per_archetype_convergence(primary_results)
    for line in conv_lines:
        print(line)

    var_lines = compute_pack_variance(primary_results)
    for line in var_lines:
        print(line)

    # ==================================================================
    # SWEEP 1: Lock probability
    # ==================================================================
    print("\n" + "=" * 70)
    print("  SWEEP 1: Lock Probability")
    print("=" * 70)

    for prob in [0.50, 0.65, 0.75, 0.85, 1.00]:
        m = sweep_metrics(pool, (3, 6, 9), prob)
        print(f"  Prob={prob:.2f}: S/A={m['sa']:.2f} (std={m['std']:.2f}), "
              f"Conv={m['conv']:.2f}, Conc={m['conc']:.1f}%")

    # ==================================================================
    # SWEEP 2: Thresholds
    # ==================================================================
    print("\n" + "=" * 70)
    print("  SWEEP 2: Threshold Variants")
    print("=" * 70)

    for thresh in [(2, 4, 7), (3, 6, 9), (4, 8, 12), (5, 10, 15)]:
        m = sweep_metrics(pool, thresh, 0.75)
        print(f"  Thresh={thresh}: S/A={m['sa']:.2f} (std={m['std']:.2f}), "
              f"Conv={m['conv']:.2f}, EarlyDiv={m['early_div']:.2f}")

    # ==================================================================
    # SWEEP 3: Split-resonance targeting variants
    # ==================================================================
    print("\n" + "=" * 70)
    print("  SWEEP 3: Split-Resonance Variants")
    print("=" * 70)

    for variant_name, split_mode in [("all_top", 0), ("2top_1sec", 1), ("1top_2sec", 2)]:
        variant_records = []
        for _ in range(500):
            state = SoftLockState()
            rec = DraftRecord()
            drafted = []
            for pick_num in range(NUM_PICKS):
                pack = generate_pack(pool, state, 0.75)
                if len(pack) < PACK_SIZE:
                    break
                rec.packs.append(pack)
                card = pick_archetype_committed(pack, drafted, pick_num)
                drafted.append(card)
                rec.picks.append(card)

                state.add_card_symbols(card)
                top = state.top_resonance()
                sec = state.second_resonance()
                if top:
                    top_ct = state.resonance_counts[top]
                    thresholds = (3, 6, 9)

                    if state.thresholds_triggered < 1 and top_ct >= thresholds[0]:
                        state.slot_locks[0] = top
                        state.thresholds_triggered = 1
                    if state.thresholds_triggered < 2 and top_ct >= thresholds[1]:
                        if split_mode >= 2:
                            state.slot_locks[1] = sec if sec else top
                        else:
                            state.slot_locks[1] = top
                        state.thresholds_triggered = 2
                    if state.thresholds_triggered < 3 and top_ct >= thresholds[2]:
                        if split_mode == 0:
                            state.slot_locks[2] = top
                        else:
                            state.slot_locks[2] = sec if sec else top
                        state.thresholds_triggered = 3

                if pick_num >= 4 and rec.convergence_pick == 30:
                    committed = determine_committed_archetype(drafted)
                    sa_in_pack = sum(1 for c in pack if c.is_sa(committed))
                    if sa_in_pack >= 2:
                        rec.convergence_pick = pick_num + 1

            variant_records.append(rec)

        # Compute key metrics
        late_sa = []
        conv_picks = []
        for rec in variant_records:
            arch = determine_committed_archetype(rec.picks)
            conv_picks.append(rec.convergence_pick)
            for pn in range(5, len(rec.packs)):
                pack = rec.packs[pn]
                sa = sum(1 for c in pack if c.is_sa(arch))
                late_sa.append(sa)

        mean_sa = sum(late_sa) / max(len(late_sa), 1)
        std_sa = math.sqrt(sum((x - mean_sa)**2 for x in late_sa) / max(len(late_sa), 1)) if late_sa else 0
        mean_conv = sum(conv_picks) / len(conv_picks)
        print(f"  {variant_name}: S/A={mean_sa:.2f} (std={std_sa:.2f}), Conv={mean_conv:.2f}")

    # ==================================================================
    # DRAFT TRACES
    # ==================================================================
    print("\n" + "=" * 70)
    print("  DETAILED DRAFT TRACES")
    print("=" * 70)

    random.seed(100)
    tp = build_card_pool(54)
    for line in run_draft_trace(tp, pick_archetype_committed,
                                 "Early Committer (archetype_committed)"):
        print(line)

    random.seed(200)
    tp = build_card_pool(54)
    for line in run_draft_trace(tp, pick_power_chaser,
                                 "Flexible Player (power_chaser)"):
        print(line)

    random.seed(300)
    tp = build_card_pool(54)
    for line in run_draft_trace(tp, pick_signal_reader, "Signal Reader"):
        print(line)

    # ==================================================================
    # VERIFICATION
    # ==================================================================
    print("\n" + "=" * 70)
    print("  ONE-SENTENCE CLAIM TEST")
    print("=" * 70)
    print("""
  One-sentence: "Each pack slot starts random; when your top resonance
  count crosses 3, 6, and 9, one more slot begins showing a resonance-
  matched card 75% of the time (first two target top resonance, third
  targets second-highest), while slot 4 stays fully random."

  - Slots start random: YES (slot_locks = [None]*4)
  - Thresholds at 3/6/9: YES (check_thresholds)
  - 75% probability: YES (lock_probability parameter)
  - First two target top: YES (thresholds 1,2 lock to top_resonance)
  - Third targets second: YES (threshold 3 locks to second_resonance)
  - Slot 4 random: YES (only slots 0-2 lockable)
  - No player decisions: YES (all automatic from symbols)
  VERDICT: Fully reconstructable from one sentence.
    """)

    print("=" * 70)
    print("  ZERO PLAYER DECISIONS: VERIFIED")
    print("  Only action = pick 1 card from pack of 4. All else automatic.")
    print("=" * 70)
    print("\n  NOTE: Compare to Agent 1 baselines when available.")


if __name__ == "__main__":
    main()
