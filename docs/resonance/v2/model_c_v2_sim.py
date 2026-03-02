"""
Model C v2 Simulation: Tiered Weighted Sampling with Soft Floors

Revised from v1 based on Round 3 debate findings:
- Dropped carousel in favor of simple weighted sampling
- N=8 with 2 suppressed per run (from Model D)
- Fixed commitment detection: pick >= 6, 3+ S/A, 2+ lead
- Soft floor instead of hard anchor slot
- ~25% multi-archetype cards as baseline
"""

import random
from collections import defaultdict
from typing import Optional

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

NUM_ARCHETYPES = 8
NUM_CARDS = 360
PACK_SIZE = 4
NUM_PICKS = 30
NUM_DRAFTS = 1000

COMMITMENT_MIN_PICK = 5      # Allow commitment from pick 5 (debate said 5-8 window)
COMMITMENT_THRESHOLD = 3
COMMITMENT_LEAD_REQUIRED = 1  # Reduced from 2: with N=8 and multi-arch cards, 2+ lead is too harsh

SUPPRESSED_PER_RUN = 2
SUPPRESSION_COPY_FACTOR = 0.50

WEIGHT_RAMP = [(5, 10, 7.0), (11, 20, 8.0), (21, 30, 9.0)]  # Strong ramp to reliably hit 2+ fitting

RARITY_DIST = [("common", 0.55), ("uncommon", 0.25), ("rare", 0.15), ("legendary", 0.05)]
RARITY_COPIES = {"common": 4, "uncommon": 3, "rare": 2, "legendary": 1}

TIER_VALUES = {"S": 5, "A": 4, "B": 2, "C": 1, "F": 0}

NEIGHBORS = {
    0: [1, 7], 1: [0, 2], 2: [1, 3], 3: [2, 4],
    4: [3, 5], 5: [4, 6], 6: [5, 7], 7: [6, 0],
}


# ---------------------------------------------------------------------------
# Card generation
# ---------------------------------------------------------------------------

def generate_card_pool(rng, multi_archetype_pct=None):
    """Generate 360 unique cards. Returns list of (id, rarity, power, fitness_dict)."""
    cards = []
    archetypes = list(range(NUM_ARCHETYPES))

    rarities = []
    for rarity, pct in RARITY_DIST:
        rarities.extend([rarity] * int(NUM_CARDS * pct))
    while len(rarities) < NUM_CARDS:
        rarities.append("common")
    rng.shuffle(rarities)

    if multi_archetype_pct is not None:
        target_multi = int(NUM_CARDS * multi_archetype_pct)
        narrow_count = NUM_CARDS - target_multi
        # Distribute multi proportionally: splash 45%, generalist 25%, multi-star 20%, universal 10%
        splash_count = int(target_multi * 0.45)
        gen_count = int(target_multi * 0.25)
        star_count = int(target_multi * 0.20)
        uni_count = target_multi - splash_count - gen_count - star_count
    else:
        # Default: ~30% multi-archetype (S/A in 2+ archetypes)
        # 60% narrow (S in 1 only), 15% splash (S+A), 8% multi-star (S+S),
        # 12% generalist (A in 2-3), 5% universal (S in 3+)
        narrow_count = 216  # 60%
        splash_count = 54   # 15%
        star_count = 29     # 8%
        gen_count = 43      # 12%
        uni_count = 18      # 5%

    s_counts = defaultdict(int)

    def pick_primary():
        min_c = min(s_counts.get(a, 0) for a in archetypes)
        cands = [a for a in archetypes if s_counts.get(a, 0) == min_c]
        return rng.choice(cands)

    ri = [0]
    def next_rarity():
        r = rarities[ri[0] % len(rarities)]
        ri[0] += 1
        return r

    def power_for(rarity, lo=None, hi=None):
        if lo is not None:
            return rng.uniform(lo, hi)
        base = {"common": 4.0, "uncommon": 5.5, "rare": 7.0, "legendary": 8.5}[rarity]
        return max(1.0, min(10.0, base + rng.gauss(0, 0.8)))

    card_id = 0

    # 1. Narrow Specialists
    for _ in range(narrow_count):
        p = pick_primary()
        s_counts[p] += 1
        fit = {}
        for a in archetypes:
            if a == p: fit[a] = "S"
            elif a in NEIGHBORS[p] and rng.random() < 0.35: fit[a] = "B"
            elif rng.random() < 0.15: fit[a] = "C"
            else: fit[a] = "F"
        r = next_rarity()
        cards.append((card_id, r, power_for(r), fit))
        card_id += 1

    # 2. Specialists with Splash
    for _ in range(splash_count):
        p = pick_primary()
        s_counts[p] += 1
        at = rng.choice(NEIGHBORS[p])
        fit = {}
        for a in archetypes:
            if a == p: fit[a] = "S"
            elif a == at: fit[a] = "A"
            elif a in NEIGHBORS[p] or rng.random() < 0.2: fit[a] = "B"
            else: fit[a] = rng.choice(["C", "F"])
        r = next_rarity()
        cards.append((card_id, r, power_for(r), fit))
        card_id += 1

    # 3. Multi-Archetype Stars
    for _ in range(star_count):
        p = pick_primary()
        s_counts[p] += 1
        sec = rng.choice(NEIGHBORS[p])
        s_counts[sec] += 1
        fit = {}
        for a in archetypes:
            if a in (p, sec): fit[a] = "S"
            elif a in NEIGHBORS[p] or a in NEIGHBORS[sec]:
                fit[a] = "B" if rng.random() < 0.4 else "C"
            else: fit[a] = rng.choice(["C", "F"])
        r = next_rarity()
        cards.append((card_id, r, power_for(r, 5.0, 8.0), fit))
        card_id += 1

    # 4. Generalists
    for _ in range(gen_count):
        ac = rng.choice([2, 3])
        at = rng.sample(archetypes, ac)
        rem = [a for a in archetypes if a not in at]
        bc = min(rng.choice([3, 4]), len(rem))
        bt = rng.sample(rem, bc)
        fit = {}
        for a in archetypes:
            if a in at: fit[a] = "A"
            elif a in bt: fit[a] = "B"
            else: fit[a] = "C"
        r = next_rarity()
        cards.append((card_id, r, power_for(r, 5.0, 7.5), fit))
        card_id += 1

    # 5. Universal Stars
    for _ in range(uni_count):
        sc = rng.choice([3, 4, 5])
        st = rng.sample(archetypes, sc)
        for a in st: s_counts[a] += 1
        fit = {}
        for a in archetypes:
            if a in st: fit[a] = "S"
            else: fit[a] = rng.choice(["A", "B"])
        r = next_rarity()
        cards.append((card_id, r, power_for(r, 7.5, 10.0), fit))
        card_id += 1

    return cards


# ---------------------------------------------------------------------------
# Fast pool representation
# ---------------------------------------------------------------------------

def build_run_pool(cards, suppressed, rng):
    """Build a pool as parallel arrays for speed. Returns (ids, powers, fitnesses, sa_masks).

    sa_masks[i] is a set of archetypes where card i is S/A.
    """
    ids = []
    powers = []
    fitnesses = []
    sa_masks = []
    rarities = []

    for cid, rarity, power, fit in cards:
        base_copies = RARITY_COPIES[rarity]
        copies = base_copies + rng.choice([-1, 0, 0, 1])
        copies = max(1, copies)

        s_archs = [a for a, f in fit.items() if f == "S"]
        sa_archs = set(a for a, f in fit.items() if f in ("S", "A"))

        primary_suppressed = any(a in suppressed for a in s_archs) and len(s_archs) <= 2
        if primary_suppressed:
            copies = max(1, int(copies * SUPPRESSION_COPY_FACTOR))

        for _ in range(copies):
            ids.append(cid)
            powers.append(power)
            fitnesses.append(fit)
            sa_masks.append(sa_archs)
            rarities.append(rarity)

    return ids, powers, fitnesses, sa_masks


def get_weight_mult(pick_num):
    for lo, hi, mult in WEIGHT_RAMP:
        if lo <= pick_num <= hi:
            return mult
    return 1.0


# ---------------------------------------------------------------------------
# Draft simulation
# ---------------------------------------------------------------------------

def run_draft(cards, strategy_name, rng, trace=False, cfg=None):
    """Run one complete draft. cfg allows overriding params for sensitivity."""
    min_pick = cfg.get("min_pick", COMMITMENT_MIN_PICK) if cfg else COMMITMENT_MIN_PICK
    weight_ramp = cfg.get("weight_ramp", WEIGHT_RAMP) if cfg else WEIGHT_RAMP

    suppressed = rng.sample(range(NUM_ARCHETYPES), SUPPRESSED_PER_RUN)
    active = [a for a in range(NUM_ARCHETYPES) if a not in suppressed]
    active_set = set(active)

    pool_ids, pool_powers, pool_fits, pool_sa = build_run_pool(cards, suppressed, rng)
    pool_size = len(pool_ids)

    # Shuffle pool
    indices = list(range(pool_size))
    rng.shuffle(indices)
    pool_ids = [pool_ids[i] for i in indices]
    pool_powers = [pool_powers[i] for i in indices]
    pool_fits = [pool_fits[i] for i in indices]
    pool_sa = [pool_sa[i] for i in indices]

    # Track which pool indices are alive
    alive = [True] * pool_size

    # Starting card signal
    starting_cands = [i for i in range(pool_size)
                      if any(pool_fits[i].get(a) == "S" for a in active)]
    if len(starting_cands) >= 3:
        start_opts = rng.sample(starting_cands, 3)
    else:
        start_opts = rng.sample(range(pool_size), min(3, pool_size))

    def card_active_score(idx):
        return max(TIER_VALUES[pool_fits[idx].get(a, "F")] + pool_powers[idx] * 0.1 for a in active)

    start_idx = max(start_opts, key=card_active_score)
    start_id = pool_ids[start_idx]
    start_fit = pool_sa[start_idx]

    # Track drafted card ids (unique) and archetype counts
    drafted_ids = set()
    drafted_ids.add(start_id)
    arch_counts = defaultdict(int)
    for a in active:
        if a in start_fit:
            arch_counts[a] += 1

    committed_arch = None
    commitment_pick = None
    drafted_count = 1  # starting card

    # Metrics
    early_unique_archs = []
    early_fitting = []
    late_fitting = []
    late_off_arch = []
    convergence_pick = None
    soft_floor_fires = 0

    trace_log = []

    for pick_num in range(1, NUM_PICKS + 1):
        # Build pack
        pack_indices = []
        used_card_ids = set()

        if committed_arch is not None and pick_num >= min_pick:
            mult = 1.0
            for lo, hi, m in weight_ramp:
                if lo <= pick_num <= hi:
                    mult = m
                    break

            # Draw 3 weighted cards
            for _ in range(3):
                candidates = []
                weights = []
                for i in range(pool_size):
                    if not alive[i] or pool_ids[i] in used_card_ids:
                        continue
                    w = 1.0
                    if committed_arch in pool_sa[i]:
                        w *= mult
                    candidates.append(i)
                    weights.append(w)
                if not candidates:
                    break
                total = sum(weights)
                if total == 0:
                    break
                r = rng.uniform(0, total)
                cum = 0
                chosen = candidates[0]
                for ci, wi in zip(candidates, weights):
                    cum += wi
                    if r <= cum:
                        chosen = ci
                        break
                pack_indices.append(chosen)
                used_card_ids.add(pool_ids[chosen])

            # Draw 1 splash card (off-archetype, power-biased)
            splash_cands = []
            splash_weights = []
            for i in range(pool_size):
                if not alive[i] or pool_ids[i] in used_card_ids:
                    continue
                if committed_arch not in pool_sa[i]:
                    sw = pool_powers[i]
                    for a in active:
                        if a != committed_arch and pool_fits[i].get(a) == "S":
                            sw += 5.0
                            break
                    splash_cands.append(i)
                    splash_weights.append(sw)

            if not splash_cands:
                # Fall back to any available
                for i in range(pool_size):
                    if not alive[i] or pool_ids[i] in used_card_ids:
                        continue
                    splash_cands.append(i)
                    splash_weights.append(pool_powers[i])

            if splash_cands:
                total = sum(splash_weights)
                if total > 0:
                    r = rng.uniform(0, total)
                    cum = 0
                    chosen = splash_cands[0]
                    for ci, wi in zip(splash_cands, splash_weights):
                        cum += wi
                        if r <= cum:
                            chosen = ci
                            break
                    pack_indices.append(chosen)
                    used_card_ids.add(pool_ids[chosen])

            # Soft floor: if 0 fitting cards, replace lowest-power with a fitting card
            fitting_count = sum(1 for pi in pack_indices if committed_arch in pool_sa[pi])
            if fitting_count == 0 and pack_indices:
                fitting_pool = [i for i in range(pool_size)
                               if alive[i] and pool_ids[i] not in used_card_ids
                               and committed_arch in pool_sa[i]]
                if fitting_pool:
                    replacement = rng.choice(fitting_pool)
                    worst = min(range(len(pack_indices)), key=lambda j: pool_powers[pack_indices[j]])
                    pack_indices[worst] = replacement
                    soft_floor_fires += 1

        else:
            # Pre-commitment: uniform random
            available = [i for i in range(pool_size) if alive[i] and pool_ids[i] not in used_card_ids]
            if len(available) >= PACK_SIZE:
                pack_indices = rng.sample(available, PACK_SIZE)
            else:
                pack_indices = available[:]

        # Fill if needed
        while len(pack_indices) < PACK_SIZE:
            available = [i for i in range(pool_size)
                        if alive[i] and pool_ids[i] not in used_card_ids
                        and i not in pack_indices]
            if not available:
                break
            pack_indices.append(rng.choice(available))
            used_card_ids.add(pool_ids[pack_indices[-1]])

        if not pack_indices:
            break

        # --- Metrics ---
        if pick_num <= 5:
            archs_in_pack = set()
            for pi in pack_indices:
                for a in active:
                    if a in pool_sa[pi]:
                        archs_in_pack.add(a)
            early_unique_archs.append(len(archs_in_pack))

            if committed_arch is not None:
                fitting = sum(1 for pi in pack_indices if committed_arch in pool_sa[pi])
            elif arch_counts:
                emerging = max(arch_counts, key=arch_counts.get)
                fitting = sum(1 for pi in pack_indices if emerging in pool_sa[pi])
            else:
                fitting = 0
            early_fitting.append(fitting)

        if pick_num >= 6 and committed_arch is not None:
            fitting = sum(1 for pi in pack_indices if committed_arch in pool_sa[pi])
            late_fitting.append(fitting)

            off = 0
            for pi in pack_indices:
                if committed_arch not in pool_sa[pi]:
                    if pool_powers[pi] >= 7.0:
                        off += 1
                    elif any(pool_fits[pi].get(a) == "S" for a in active if a != committed_arch):
                        off += 1
            late_off_arch.append(off)

        if committed_arch is not None and convergence_pick is None:
            fitting = sum(1 for pi in pack_indices if committed_arch in pool_sa[pi])
            if fitting >= 2:
                convergence_pick = pick_num

        # --- Player picks ---
        if strategy_name == "committed":
            if committed_arch is not None:
                chosen_pi = max(pack_indices, key=lambda pi: TIER_VALUES[pool_fits[pi].get(committed_arch, "F")] + pool_powers[pi] * 0.1)
            else:
                chosen_pi = max(pack_indices, key=lambda pi: max(TIER_VALUES[pool_fits[pi].get(a, "F")] + pool_powers[pi] * 0.1 for a in active))
        elif strategy_name == "power_chaser":
            chosen_pi = max(pack_indices, key=lambda pi: pool_powers[pi])
        else:  # signal_reader
            if committed_arch is not None:
                chosen_pi = max(pack_indices, key=lambda pi: TIER_VALUES[pool_fits[pi].get(committed_arch, "F")] + pool_powers[pi] * 0.1)
            elif arch_counts:
                target = max(arch_counts, key=arch_counts.get)
                chosen_pi = max(pack_indices, key=lambda pi: TIER_VALUES[pool_fits[pi].get(target, "F")] + pool_powers[pi] * 0.1)
            else:
                chosen_pi = max(pack_indices, key=lambda pi: pool_powers[pi])

        # Record trace
        if trace:
            trace_log.append({
                "pick": pick_num,
                "pack": [(pool_ids[pi], pool_fits[pi].get(committed_arch if committed_arch is not None
                          else (max(arch_counts, key=arch_counts.get) if arch_counts else 0), "F"),
                          round(pool_powers[pi], 1))
                         for pi in pack_indices],
                "picked_id": pool_ids[chosen_pi],
                "committed": committed_arch,
            })

        picked_cid = pool_ids[chosen_pi]
        drafted_ids.add(picked_cid)
        drafted_count += 1

        # Update archetype counts
        for a in active:
            if a in pool_sa[chosen_pi]:
                arch_counts[a] += 1

        # Remove all copies of picked card
        for i in range(pool_size):
            if pool_ids[i] == picked_cid:
                alive[i] = False

        # Check commitment
        if committed_arch is None and pick_num >= min_pick:
            if arch_counts:
                sorted_a = sorted(arch_counts.items(), key=lambda x: -x[1])
                best_a, best_c = sorted_a[0]
                runner_c = sorted_a[1][1] if len(sorted_a) > 1 else 0
                if best_c >= COMMITMENT_THRESHOLD and (best_c - runner_c) >= COMMITMENT_LEAD_REQUIRED:
                    committed_arch = best_a
                    commitment_pick = pick_num

    # Deck concentration
    if committed_arch is not None and drafted_count > 0:
        # Count S/A cards for committed arch among all drafted unique card ids
        sa_count = 0
        for cid, _, _, fit in cards:
            if cid in drafted_ids and fit.get(committed_arch, "F") in ("S", "A"):
                sa_count += 1
        # Include starting card
        deck_concentration = sa_count / drafted_count
    else:
        deck_concentration = 0.0

    return {
        "committed_arch": committed_arch,
        "commitment_pick": commitment_pick,
        "deck_concentration": deck_concentration,
        "picked_ids": drafted_ids,
        "early_unique_archs": early_unique_archs,
        "early_fitting": early_fitting,
        "late_fitting": late_fitting,
        "late_off_arch": late_off_arch,
        "convergence_pick": convergence_pick,
        "soft_floor_fires": soft_floor_fires,
        "trace": trace_log if trace else None,
        "suppressed": suppressed,
        "active": active,
    }


# ---------------------------------------------------------------------------
# Batch simulation
# ---------------------------------------------------------------------------

def run_simulation(multi_archetype_pct=None, num_drafts=NUM_DRAFTS,
                   trace_count=0, cfg=None):
    """Run full simulation batch across all 3 strategies."""
    master_rng = random.Random(42)
    cards = generate_card_pool(master_rng, multi_archetype_pct)

    actual_multi = sum(1 for _, _, _, fit in cards
                      if sum(1 for f in fit.values() if f in ("S", "A")) >= 2) / len(cards)

    all_results = {"multi_arch_pct": actual_multi}

    for strat_name in ["committed", "power_chaser", "signal_reader"]:
        results = defaultdict(list)

        for i in range(num_drafts):
            draft_rng = random.Random(master_rng.randint(0, 2**32))
            do_trace = (i < trace_count)
            r = run_draft(cards, strat_name, draft_rng, trace=do_trace, cfg=cfg)

            results["early_unique_archs"].extend(r["early_unique_archs"])
            results["early_fitting"].extend(r["early_fitting"])
            results["late_fitting"].extend(r["late_fitting"])
            results["late_off_arch"].extend(r["late_off_arch"])
            if r["convergence_pick"] is not None:
                results["convergence_picks"].append(r["convergence_pick"])
            results["deck_concentrations"].append(r["deck_concentration"])
            results["picked_id_sets"].append(r["picked_ids"])
            if r["committed_arch"] is not None:
                results["committed_archetypes"].append(r["committed_arch"])
            if r["commitment_pick"] is not None:
                results["commitment_picks"].append(r["commitment_pick"])
            if r["trace"]:
                results["traces"].append(r["trace"])
            results["soft_floor_fires"].append(r["soft_floor_fires"])

        all_results[strat_name] = dict(results)

    return all_results


def safe_mean(lst):
    return sum(lst) / len(lst) if lst else 0


def compute_metrics(all_results):
    """Compute target metrics from simulation results."""
    m = {}
    c = all_results["committed"]

    m["early_unique_archs"] = safe_mean(c.get("early_unique_archs", []))
    m["early_fitting"] = safe_mean(c.get("early_fitting", []))
    m["late_fitting"] = safe_mean(c.get("late_fitting", []))
    m["late_off_arch"] = safe_mean(c.get("late_off_arch", []))
    m["convergence_pick"] = safe_mean(c.get("convergence_picks", [])) if c.get("convergence_picks") else float("inf")
    m["commitment_pick"] = safe_mean(c.get("commitment_picks", [])) if c.get("commitment_picks") else float("inf")
    m["soft_floor_pct"] = safe_mean(c.get("soft_floor_fires", [])) / max(NUM_PICKS - COMMITMENT_MIN_PICK + 1, 1) if c.get("soft_floor_fires") else 0

    concs = [x for x in c.get("deck_concentrations", []) if x > 0]
    m["deck_concentration"] = safe_mean(concs)

    # Overlap
    id_sets = c.get("picked_id_sets", [])
    overlaps = []
    for i in range(0, min(200, len(id_sets)) - 1, 2):
        s1, s2 = id_sets[i], id_sets[i+1]
        if s1 and s2:
            overlaps.append(len(s1 & s2) / max(len(s1 | s2), 1))
    m["card_overlap"] = safe_mean(overlaps)

    # Archetype frequency
    arch_counts = defaultdict(int)
    total = len(c.get("committed_archetypes", []))
    for a in c.get("committed_archetypes", []):
        arch_counts[a] += 1
    if total > 0:
        m["arch_freq_max"] = max(arch_counts.values()) / total
        all_freqs = [arch_counts.get(a, 0) / total for a in range(NUM_ARCHETYPES)]
        m["arch_freq_min"] = min(all_freqs)
    else:
        m["arch_freq_max"] = 0
        m["arch_freq_min"] = 0

    m["num_committed"] = len(c.get("committed_archetypes", []))

    # Per-strategy
    for strat in ["committed", "power_chaser", "signal_reader"]:
        sr = all_results.get(strat, {})
        p = strat + "_"
        m[p + "late_fitting"] = safe_mean(sr.get("late_fitting", []))
        concs = [x for x in sr.get("deck_concentrations", []) if x > 0]
        m[p + "deck_conc"] = safe_mean(concs)
        m[p + "commitment_pick"] = safe_mean(sr.get("commitment_picks", [])) if sr.get("commitment_picks") else float("inf")
        m[p + "num_committed"] = len(sr.get("committed_archetypes", []))

    return m


# ---------------------------------------------------------------------------
# Reporting
# ---------------------------------------------------------------------------

def print_scorecard(m, label=""):
    if label:
        print(f"\n{'='*75}")
        print(f"  {label}")
        print(f"{'='*75}")
    else:
        print(f"\n{'='*75}")
        print(f"  TARGET SCORECARD (committed strategy)")
        print(f"{'='*75}")

    targets = [
        ("Picks 1-5: unique archs per pack", ">= 3", m["early_unique_archs"], m["early_unique_archs"] >= 3),
        ("Picks 1-5: fitting cards per pack", "<= 2", m["early_fitting"], m["early_fitting"] <= 2),
        ("Picks 6+: fitting cards per pack", ">= 2", m["late_fitting"], m["late_fitting"] >= 2),
        ("Picks 6+: off-archetype per pack", ">= 0.5", m["late_off_arch"], m["late_off_arch"] >= 0.5),
        ("Convergence pick", "5-8", m["convergence_pick"], 5 <= m["convergence_pick"] <= 8),
        ("Deck concentration", "85-95%*", m["deck_concentration"], 0.85 <= m["deck_concentration"] <= 0.95),
        ("Run-to-run card overlap", "< 40%", m["card_overlap"], m["card_overlap"] < 0.40),
        ("Archetype freq max", "<= 20%", m["arch_freq_max"], m["arch_freq_max"] <= 0.20),
        ("Archetype freq min", ">= 5%", m["arch_freq_min"], m["arch_freq_min"] >= 0.05),
    ]

    print(f"  {'Metric':<44} {'Target':<12} {'Actual':<10} {'Result'}")
    print(f"  {'-'*72}")
    for name, target, actual, passed in targets:
        if isinstance(actual, float):
            astr = f"{actual:.1%}" if actual < 1 else f"{actual:.2f}"
        else:
            astr = str(actual)
        print(f"  {name:<44} {target:<12} {astr:<10} {'PASS' if passed else 'FAIL'}")

    passes = sum(1 for _, _, _, p in targets if p)
    print(f"\n  Total: {passes}/{len(targets)} passed")
    print(f"  * Concentration target relaxed to 85-95% per debate consensus")
    return passes, len(targets)


def print_strategy_breakdown(m):
    print(f"\n{'='*75}")
    print(f"  PER-STRATEGY BREAKDOWN")
    print(f"{'='*75}")
    print(f"  {'Strategy':<20} {'Late Fit':<10} {'Deck Conc':<12} {'Commit Pick':<14} {'#Committed'}")
    print(f"  {'-'*66}")
    for strat in ["committed", "power_chaser", "signal_reader"]:
        p = strat + "_"
        lf = m.get(p + "late_fitting", 0)
        dc = m.get(p + "deck_conc", 0)
        cp = m.get(p + "commitment_pick", float("inf"))
        nc = m.get(p + "num_committed", 0)
        cp_str = f"{cp:.1f}" if cp < float("inf") else "N/A"
        print(f"  {strat:<20} {lf:<10.2f} {dc:<12.1%} {cp_str:<14} {nc}")


# ---------------------------------------------------------------------------
# Story traces
# ---------------------------------------------------------------------------

def run_story_traces(cards):
    print(f"\n{'='*75}")
    print(f"  DRAFT STORY TRACES")
    print(f"{'='*75}")

    # Story 1: Early committer
    print(f"\n  --- Story 1: Early Committer ---")
    for seed in range(200):
        rng = random.Random(seed + 1000)
        r = run_draft(cards, "committed", rng, trace=True)
        if r["commitment_pick"] is not None and r["commitment_pick"] <= 7:
            print_story(r, "Early committer (committed strategy)")
            break

    # Story 2: Flexible player (8+ picks before commitment)
    print(f"\n  --- Story 2: Flexible Player ---")
    for seed in range(200):
        rng = random.Random(seed + 2000)
        r = run_draft(cards, "committed", rng, trace=True)
        if r["commitment_pick"] is not None and r["commitment_pick"] >= 9:
            print_story(r, "Flexible player (late commitment)")
            break

    # Story 3: Signal reader
    print(f"\n  --- Story 3: Signal Reader ---")
    for seed in range(200):
        rng = random.Random(seed + 3000)
        r = run_draft(cards, "signal_reader", rng, trace=True)
        if r["committed_arch"] is not None:
            print_story(r, "Signal reader strategy")
            break


def print_story(r, label):
    print(f"  {label}")
    print(f"  Committed to archetype {r['committed_arch']} at pick {r['commitment_pick']}")
    print(f"  Suppressed: {r['suppressed']}, Active: {r['active']}")
    print(f"  Deck concentration: {r['deck_concentration']:.1%}")
    print(f"  Soft floor fires: {r['soft_floor_fires']}")

    if r["trace"]:
        for entry in r["trace"][:15]:
            pk = entry["pick"]
            ca = entry["committed"]
            phase = "PRE " if ca is None else f"POST(a={ca})"
            parts = []
            for cid, fit, pwr in entry["pack"]:
                marker = "*" if cid == entry["picked_id"] else " "
                parts.append(f"[{marker}]id={cid} fit={fit} pwr={pwr}")
            print(f"    Pick {pk:2d} {phase:14s}: {'  '.join(parts)}")
        if len(r["trace"]) > 15:
            print(f"    ... ({len(r['trace']) - 15} more picks)")


# ---------------------------------------------------------------------------
# Sensitivity sweeps
# ---------------------------------------------------------------------------

def run_multi_arch_sweep():
    print(f"\n{'='*75}")
    print(f"  SENSITIVITY: Multi-Archetype Card Percentage")
    print(f"{'='*75}")

    pcts = [0.10, 0.15, 0.20, 0.25, 0.30, 0.40, 0.50]
    print(f"  {'MA%':<7} {'Actual':<8} {'EarlyUniq':<10} {'EarlyFit':<10} "
          f"{'LateFit':<9} {'LateOff':<9} {'ConvPk':<8} {'DeckConc':<10} "
          f"{'Overlap':<9} {'ArchMax':<8}")
    print(f"  {'-'*92}")

    for pct in pcts:
        res = run_simulation(multi_archetype_pct=pct, num_drafts=400)
        m = compute_metrics(res)
        print(f"  {pct:<7.0%} {res['multi_arch_pct']:<8.0%} "
              f"{m['early_unique_archs']:<10.2f} {m['early_fitting']:<10.2f} "
              f"{m['late_fitting']:<9.2f} {m['late_off_arch']:<9.2f} "
              f"{m['convergence_pick']:<8.1f} {m['deck_concentration']:<10.1%} "
              f"{m['card_overlap']:<9.1%} {m['arch_freq_max']:<8.1%}")


def run_weight_ramp_sweep():
    print(f"\n{'='*75}")
    print(f"  SENSITIVITY: Weight Ramp Multiplier")
    print(f"{'='*75}")

    ramps = [
        ("Gentle (2/3/4)", [(6,10,2.0),(11,20,3.0),(21,30,4.0)]),
        ("Default (3/4/5)", [(6,10,3.0),(11,20,4.0),(21,30,5.0)]),
        ("Strong (5/6/7)", [(6,10,5.0),(11,20,6.0),(21,30,7.0)]),
        ("Aggressive (8/9/10)", [(6,10,8.0),(11,20,9.0),(21,30,10.0)]),
    ]

    print(f"  {'Ramp':<22} {'LateFit':<9} {'LateOff':<9} {'ConvPk':<8} {'DeckConc':<10}")
    print(f"  {'-'*58}")

    for name, ramp in ramps:
        res = run_simulation(num_drafts=400, cfg={"weight_ramp": ramp})
        m = compute_metrics(res)
        print(f"  {name:<22} {m['late_fitting']:<9.2f} {m['late_off_arch']:<9.2f} "
              f"{m['convergence_pick']:<8.1f} {m['deck_concentration']:<10.1%}")


def run_commitment_timing_sweep():
    print(f"\n{'='*75}")
    print(f"  SENSITIVITY: Commitment Detection Min Pick")
    print(f"{'='*75}")

    min_picks = [4, 5, 6, 7, 8, 9, 10]
    print(f"  {'MinPick':<9} {'CommitPk':<10} {'LateFit':<9} {'DeckConc':<10} {'ConvPk':<8} {'#Committed'}")
    print(f"  {'-'*56}")

    for mp in min_picks:
        res = run_simulation(num_drafts=400, cfg={"min_pick": mp})
        m = compute_metrics(res)
        print(f"  {mp:<9} {m['commitment_pick']:<10.1f} {m['late_fitting']:<9.2f} "
              f"{m['deck_concentration']:<10.1%} {m['convergence_pick']:<8.1f} {m['num_committed']}")


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    print("=" * 75)
    print("  MODEL C v2 SIMULATION")
    print("  Tiered Weighted Sampling with Soft Floors")
    print("  N=8, 2 suppressed/run, ~25% multi-archetype cards")
    print("=" * 75)

    # Main simulation
    print("\n  Running main simulation (1000 drafts x 3 strategies)...")
    main_results = run_simulation(trace_count=3)
    main_metrics = compute_metrics(main_results)

    print(f"\n  Multi-archetype card %: {main_results['multi_arch_pct']:.1%}")
    print_scorecard(main_metrics)
    print_strategy_breakdown(main_metrics)

    # Story traces
    story_rng = random.Random(42)
    story_cards = generate_card_pool(story_rng)
    run_story_traces(story_cards)

    # Sensitivity sweeps
    print("\n  Running sensitivity sweeps...")
    run_multi_arch_sweep()
    run_weight_ramp_sweep()
    run_commitment_timing_sweep()

    print(f"\n{'='*75}")
    print(f"  Simulation complete.")
    print(f"{'='*75}")
