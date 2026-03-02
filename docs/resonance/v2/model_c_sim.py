"""
Model C Simulation: Sub-Pool Carousel with Guaranteed Floors

Non-standard pack construction using archetype sub-pools, slot roles,
carousel pre-commitment exploration, and anchor post-commitment convergence.
"""

import random
import math
from collections import defaultdict
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

NUM_ARCHETYPES = 7
NUM_CARDS = 360
PACK_SIZE = 4
NUM_PICKS = 30
NUM_DRAFTS = 1000
COMMITMENT_THRESHOLD = 3  # S/A picks in one archetype to trigger commitment
COMMITMENT_PICK = 5       # earliest pick commitment logic activates

# Rarity system
RARITY_DIST = {"common": 0.55, "uncommon": 0.25, "rare": 0.15, "legendary": 0.05}
RARITY_COPIES = {"common": 4, "uncommon": 3, "rare": 2, "legendary": 1}

# Fitness tiers with numeric values for power approximation
FITNESS_VALUES = {"S": 5, "A": 4, "B": 3, "C": 2, "F": 0}

# Card type distribution
NARROW_SPECIALIST_PCT = 0.35
SPLASH_SPECIALIST_PCT = 0.30
MULTI_STAR_PCT = 0.10
GENERALIST_PCT = 0.20
UNIVERSAL_STAR_PCT = 0.05

# Neighbor topology: each archetype has 2-3 neighbors
# Using a ring + cross-connections for 7 archetypes
NEIGHBORS = {
    0: [1, 6, 3],
    1: [0, 2, 4],
    2: [1, 3, 5],
    3: [2, 4, 0],
    4: [3, 5, 1],
    5: [4, 6, 2],
    6: [5, 0, 3],
}

# Pool restriction: suppress 1-2 archetypes per run to 60% pool
NUM_SUPPRESSED = 1
SUPPRESSION_FACTOR = 0.60


# ---------------------------------------------------------------------------
# Data model
# ---------------------------------------------------------------------------

@dataclass
class SimCard:
    id: int
    rarity: str
    power: float
    archetype_fitness: dict  # archetype_id -> "S"/"A"/"B"/"C"/"F"

    def fitness_in(self, arch: int) -> str:
        return self.archetype_fitness.get(arch, "F")

    def fitness_value(self, arch: int) -> int:
        return FITNESS_VALUES[self.fitness_in(arch)]

    def is_fitting(self, arch: int) -> bool:
        """S or A tier in the given archetype."""
        return self.fitness_in(arch) in ("S", "A")

    def best_archetype(self) -> int:
        """Return the archetype with highest fitness (tie-break: lowest id)."""
        return max(self.archetype_fitness, key=lambda a: FITNESS_VALUES[self.archetype_fitness[a]])

    def best_fitness_value(self) -> int:
        return max(FITNESS_VALUES[f] for f in self.archetype_fitness.values())

    def s_tier_archetypes(self) -> list:
        return [a for a, f in self.archetype_fitness.items() if f == "S"]

    def sa_tier_archetypes(self) -> list:
        return [a for a, f in self.archetype_fitness.items() if f in ("S", "A")]

    def num_sa_archetypes(self) -> int:
        return len(self.sa_tier_archetypes())


# ---------------------------------------------------------------------------
# Card generation
# ---------------------------------------------------------------------------

def generate_card_pool(multi_archetype_pct: Optional[float] = None) -> list:
    """Generate 360 unique cards with the specified fitness distribution.

    If multi_archetype_pct is provided, it overrides the default distribution
    to set the fraction of cards that are S/A in 2+ archetypes (for sensitivity).
    """
    cards = []
    card_id = 0

    # Assign rarities
    rarities = []
    for rarity, pct in RARITY_DIST.items():
        count = int(NUM_CARDS * pct)
        rarities.extend([rarity] * count)
    # Fill any remainder with common
    while len(rarities) < NUM_CARDS:
        rarities.append("common")
    random.shuffle(rarities)

    # Determine card type counts
    if multi_archetype_pct is not None:
        # Adjust distribution so that multi_archetype_pct of cards are S/A in 2+
        # Multi-archetype cards = splash_specialist + multi_star + generalist + universal_star
        # Narrow specialists are NOT multi-archetype (S in 1, B in 1 only)
        target_multi = int(NUM_CARDS * multi_archetype_pct)
        narrow_count = NUM_CARDS - target_multi
        # Distribute multi-archetype across types proportionally
        base_multi = SPLASH_SPECIALIST_PCT + MULTI_STAR_PCT + GENERALIST_PCT + UNIVERSAL_STAR_PCT
        splash_count = int(target_multi * (SPLASH_SPECIALIST_PCT / base_multi))
        star_count = int(target_multi * (MULTI_STAR_PCT / base_multi))
        gen_count = int(target_multi * (GENERALIST_PCT / base_multi))
        uni_count = target_multi - splash_count - star_count - gen_count
    else:
        narrow_count = int(NUM_CARDS * NARROW_SPECIALIST_PCT)
        splash_count = int(NUM_CARDS * SPLASH_SPECIALIST_PCT)
        star_count = int(NUM_CARDS * MULTI_STAR_PCT)
        gen_count = int(NUM_CARDS * GENERALIST_PCT)
        uni_count = NUM_CARDS - narrow_count - splash_count - star_count - gen_count

    archetypes = list(range(NUM_ARCHETYPES))

    def make_card(rarity, fitness_dict, power=None):
        nonlocal card_id
        if power is None:
            # Base power from rarity
            base = {"common": 4.0, "uncommon": 5.5, "rare": 7.0, "legendary": 8.5}[rarity]
            power = base + random.gauss(0, 0.8)
            power = max(1.0, min(10.0, power))
        c = SimCard(id=card_id, rarity=rarity, power=power, archetype_fitness=fitness_dict)
        card_id += 1
        return c

    rarity_idx = 0

    def next_rarity():
        nonlocal rarity_idx
        r = rarities[rarity_idx]
        rarity_idx += 1
        return r

    # 1. Narrow Specialists: S in 1, B in 1 neighbor, C/F elsewhere
    for i in range(narrow_count):
        primary = archetypes[i % NUM_ARCHETYPES]
        neighbor = random.choice(NEIGHBORS[primary])
        fitness = {}
        for a in archetypes:
            if a == primary:
                fitness[a] = "S"
            elif a == neighbor:
                fitness[a] = "B"
            else:
                fitness[a] = random.choice(["C", "F"])
        cards.append(make_card(next_rarity(), fitness))

    # 2. Specialists with Splash: S in 1, A in 1-2 neighbors, B/C elsewhere
    for i in range(splash_count):
        primary = archetypes[i % NUM_ARCHETYPES]
        neighbors = NEIGHBORS[primary]
        a_targets = random.sample(neighbors, min(random.choice([1, 2]), len(neighbors)))
        fitness = {}
        for a in archetypes:
            if a == primary:
                fitness[a] = "S"
            elif a in a_targets:
                fitness[a] = "A"
            else:
                fitness[a] = random.choice(["B", "C"])
        cards.append(make_card(next_rarity(), fitness))

    # 3. Multi-Archetype Stars: S in 2 neighboring archetypes, B in 1-2
    for i in range(star_count):
        primary = archetypes[i % NUM_ARCHETYPES]
        second = random.choice(NEIGHBORS[primary])
        b_targets = random.sample([a for a in archetypes if a not in (primary, second)],
                                  min(random.randint(1, 2), NUM_ARCHETYPES - 2))
        fitness = {}
        for a in archetypes:
            if a in (primary, second):
                fitness[a] = "S"
            elif a in b_targets:
                fitness[a] = "B"
            else:
                fitness[a] = random.choice(["C", "F"])
        cards.append(make_card(next_rarity(), fitness))

    # 4. Generalists: A in 2-3, B in 2-3, no S
    for i in range(gen_count):
        a_count = random.choice([2, 3])
        a_targets = random.sample(archetypes, a_count)
        remaining = [a for a in archetypes if a not in a_targets]
        b_count = min(random.choice([2, 3]), len(remaining))
        b_targets = random.sample(remaining, b_count)
        fitness = {}
        for a in archetypes:
            if a in a_targets:
                fitness[a] = "A"
            elif a in b_targets:
                fitness[a] = "B"
            else:
                fitness[a] = "C"
        cards.append(make_card(next_rarity(), fitness))

    # 5. Universal Stars: S in 3+ archetypes, high power
    for i in range(uni_count):
        s_count = random.choice([3, 4])
        s_targets = random.sample(archetypes, s_count)
        fitness = {}
        for a in archetypes:
            if a in s_targets:
                fitness[a] = "S"
            else:
                fitness[a] = random.choice(["A", "B"])
        r = next_rarity()
        base_power = 8.0 + random.gauss(0, 0.5)
        cards.append(make_card(r, fitness, power=max(7.0, min(10.0, base_power))))

    return cards


# ---------------------------------------------------------------------------
# Pool entry (card + copies)
# ---------------------------------------------------------------------------

@dataclass
class PoolEntry:
    card: SimCard
    copies_remaining: int


def build_pool(cards: list, suppressed_archetypes: list) -> dict:
    """Build archetype sub-pools and a full pool.

    Returns dict with:
      - 'sub_pools': {arch_id: [PoolEntry, ...]} -- cards with S/A in that arch
      - 'full_pool': [PoolEntry, ...] -- all cards
      - 'pool_entries': {card_id: PoolEntry} -- for depletion tracking
    """
    pool_entries = {}
    for card in cards:
        copies = RARITY_COPIES[card.rarity]
        # Apply suppression
        for arch in suppressed_archetypes:
            if card.fitness_in(arch) == "S":
                copies = max(1, int(copies * SUPPRESSION_FACTOR))
                break
        pool_entries[card.id] = PoolEntry(card=card, copies_remaining=copies)

    sub_pools = defaultdict(list)
    full_pool = []
    for pe in pool_entries.values():
        full_pool.append(pe)
        for arch in range(NUM_ARCHETYPES):
            if pe.card.is_fitting(arch):
                sub_pools[arch].append(pe)

    return {
        'sub_pools': dict(sub_pools),
        'full_pool': full_pool,
        'pool_entries': pool_entries,
    }


def draw_from_pool(pool_list: list, exclude_ids: set, n: int = 1) -> list:
    """Draw n cards from a pool list, weighted by copies_remaining, excluding already-drawn ids."""
    available = [pe for pe in pool_list if pe.copies_remaining > 0 and pe.card.id not in exclude_ids]
    if not available:
        return []

    drawn = []
    for _ in range(n):
        if not available:
            break
        weights = [pe.copies_remaining for pe in available]
        total = sum(weights)
        if total == 0:
            break
        chosen = random.choices(available, weights=weights, k=1)[0]
        drawn.append(chosen)
        available = [pe for pe in available if pe.card.id != chosen.card.id]
    return drawn


# ---------------------------------------------------------------------------
# Pack construction
# ---------------------------------------------------------------------------

def construct_pack_precommit(pool_data: dict, carousel_sequence: list, pick_num: int) -> list:
    """Pre-commitment pack (picks 1-5): carousel spotlight slots + random."""
    sub_pools = pool_data['sub_pools']
    full_pool = pool_data['full_pool']

    pack_cards = []
    used_ids = set()

    # Slot 1: Spotlight from carousel archetype
    arch1 = carousel_sequence[pick_num % len(carousel_sequence)]
    drawn = draw_from_pool(sub_pools.get(arch1, []), used_ids, 1)
    if drawn:
        pack_cards.append(drawn[0].card)
        used_ids.add(drawn[0].card.id)

    # Slot 2: Spotlight from a DIFFERENT carousel archetype
    arch2_idx = (pick_num + 1) % len(carousel_sequence)
    arch2 = carousel_sequence[arch2_idx]
    if arch2 == arch1:
        arch2 = carousel_sequence[(arch2_idx + 1) % len(carousel_sequence)]
    drawn = draw_from_pool(sub_pools.get(arch2, []), used_ids, 1)
    if drawn:
        pack_cards.append(drawn[0].card)
        used_ids.add(drawn[0].card.id)

    # Slots 3-4: Random from full pool
    while len(pack_cards) < PACK_SIZE:
        drawn = draw_from_pool(full_pool, used_ids, 1)
        if drawn:
            pack_cards.append(drawn[0].card)
            used_ids.add(drawn[0].card.id)
        else:
            break

    return pack_cards[:PACK_SIZE]


def construct_pack_committed(pool_data: dict, committed_arch: int) -> list:
    """Post-commitment pack (picks 6+): anchor + neighbor + splash + wild."""
    sub_pools = pool_data['sub_pools']
    full_pool = pool_data['full_pool']

    pack_cards = []
    used_ids = set()

    # Slot 1: Anchor - draw from committed archetype sub-pool
    drawn = draw_from_pool(sub_pools.get(committed_arch, []), used_ids, 1)
    if drawn:
        pack_cards.append(drawn[0].card)
        used_ids.add(drawn[0].card.id)

    # Slot 2: Committed (60%) or neighbor (40%)
    if random.random() < 0.60:
        source = sub_pools.get(committed_arch, [])
    else:
        neighbor = random.choice(NEIGHBORS[committed_arch])
        source = sub_pools.get(neighbor, [])
    drawn = draw_from_pool(source, used_ids, 1)
    if drawn:
        pack_cards.append(drawn[0].card)
        used_ids.add(drawn[0].card.id)

    # Slot 3: Splash - random non-committed archetype sub-pool
    non_committed = [a for a in range(NUM_ARCHETYPES) if a != committed_arch]
    splash_arch = random.choice(non_committed)
    drawn = draw_from_pool(sub_pools.get(splash_arch, []), used_ids, 1)
    if drawn:
        pack_cards.append(drawn[0].card)
        used_ids.add(drawn[0].card.id)

    # Slot 4: Wild - full pool
    drawn = draw_from_pool(full_pool, used_ids, 1)
    if drawn:
        pack_cards.append(drawn[0].card)
        used_ids.add(drawn[0].card.id)

    # Fill any remaining slots
    while len(pack_cards) < PACK_SIZE:
        drawn = draw_from_pool(full_pool, used_ids, 1)
        if drawn:
            pack_cards.append(drawn[0].card)
            used_ids.add(drawn[0].card.id)
        else:
            break

    return pack_cards[:PACK_SIZE]


# ---------------------------------------------------------------------------
# Player strategies
# ---------------------------------------------------------------------------

def detect_commitment(drafted_cards: list) -> Optional[int]:
    """Detect if the player has committed to an archetype (3+ S/A in one arch)."""
    arch_counts = defaultdict(int)
    for card in drafted_cards:
        for arch in card.sa_tier_archetypes():
            arch_counts[arch] += 1

    best_arch = None
    best_count = 0
    for arch, count in arch_counts.items():
        if count > best_count:
            best_count = count
            best_arch = arch

    if best_count >= COMMITMENT_THRESHOLD:
        return best_arch
    return None


def strategy_committed(pack: list, drafted: list, committed_arch: Optional[int]) -> SimCard:
    """Pick the card with highest fitness in the committed archetype.
    If not committed, pick the card with best single-archetype fitness."""
    if committed_arch is not None:
        return max(pack, key=lambda c: (c.fitness_value(committed_arch), c.power))
    else:
        # Pick best card by its best archetype fitness
        return max(pack, key=lambda c: (c.best_fitness_value(), c.power))


def strategy_power_chaser(pack: list, drafted: list, committed_arch: Optional[int]) -> SimCard:
    """Pick the highest raw power card regardless of archetype."""
    return max(pack, key=lambda c: c.power)


def strategy_signal_reader(pack: list, drafted: list, committed_arch: Optional[int]) -> SimCard:
    """Track which archetypes appear most frequently, lean toward the 'open' one."""
    if committed_arch is not None:
        return max(pack, key=lambda c: (c.fitness_value(committed_arch), c.power))

    # Count archetype frequencies across all seen packs (approximated by drafted cards)
    arch_freq = defaultdict(int)
    for card in drafted:
        for arch in card.sa_tier_archetypes():
            arch_freq[arch] += 1

    # Also count from current pack
    for card in pack:
        for arch in card.sa_tier_archetypes():
            arch_freq[arch] += 0.5  # lower weight for current

    if not arch_freq:
        return max(pack, key=lambda c: c.power)

    # Pick the card whose best archetype has the highest frequency (most "open")
    def card_score(c):
        best_arch_score = 0
        for arch in c.sa_tier_archetypes():
            freq = arch_freq.get(arch, 0)
            best_arch_score = max(best_arch_score, freq * c.fitness_value(arch))
        if best_arch_score == 0:
            return c.power
        return best_arch_score * 10 + c.power

    return max(pack, key=card_score)


# ---------------------------------------------------------------------------
# Draft simulation
# ---------------------------------------------------------------------------

@dataclass
class DraftResult:
    strategy: str
    drafted_cards: list
    committed_arch: Optional[int]
    commitment_pick: Optional[int]
    pack_history: list  # list of (pick_num, pack_cards, picked_card)

    # Computed metrics
    early_unique_archetypes: list = field(default_factory=list)  # per pack, picks 1-5
    early_fitting_count: list = field(default_factory=list)       # per pack, picks 1-5
    late_fitting_count: list = field(default_factory=list)        # per pack, picks 6+
    late_off_archetype_count: list = field(default_factory=list)  # per pack, picks 6+
    deck_concentration: float = 0.0


def run_draft(cards: list, strategy_fn, strategy_name: str,
              suppressed_archetypes: list, trace: bool = False) -> DraftResult:
    """Run a single draft of 30 picks."""
    pool_data = build_pool(cards, suppressed_archetypes)

    # Create carousel sequence (shuffled archetypes)
    carousel = list(range(NUM_ARCHETYPES))
    random.shuffle(carousel)

    drafted = []
    committed_arch = None
    commitment_pick = None
    pack_history = []

    for pick_num in range(NUM_PICKS):
        # Construct pack
        if pick_num < COMMITMENT_PICK or committed_arch is None:
            pack = construct_pack_precommit(pool_data, carousel, pick_num)
        else:
            pack = construct_pack_committed(pool_data, committed_arch)

        if len(pack) == 0:
            break

        # Pick a card
        picked = strategy_fn(pack, drafted, committed_arch)
        drafted.append(picked)
        pack_history.append((pick_num, [c for c in pack], picked))

        # Deplete the picked card from pool
        if picked.id in pool_data['pool_entries']:
            pe = pool_data['pool_entries'][picked.id]
            pe.copies_remaining = max(0, pe.copies_remaining - 1)

        # Check for commitment (only after pick 3)
        if committed_arch is None and len(drafted) >= COMMITMENT_THRESHOLD:
            committed_arch = detect_commitment(drafted)
            if committed_arch is not None and commitment_pick is None:
                commitment_pick = pick_num

    result = DraftResult(
        strategy=strategy_name,
        drafted_cards=drafted,
        committed_arch=committed_arch,
        commitment_pick=commitment_pick,
        pack_history=pack_history,
    )

    # Compute metrics
    compute_metrics(result)

    return result


def compute_metrics(result: DraftResult):
    """Compute all 8 target metrics for a draft result."""
    committed = result.committed_arch

    for pick_num, pack, picked in result.pack_history:
        if pick_num < 5:
            # Early metrics (picks 1-5)
            # Unique archetypes: count archetypes where at least 1 card is S-tier
            represented = set()
            for card in pack:
                for arch in card.s_tier_archetypes():
                    represented.add(arch)
                # Also count A-tier for broader representation
                for arch in card.sa_tier_archetypes():
                    represented.add(arch)
            result.early_unique_archetypes.append(len(represented))

            # Cards fitting emerging archetype (use committed if known, else best guess)
            if committed is not None:
                fitting = sum(1 for c in pack if c.is_fitting(committed))
            else:
                # Use the most common archetype in drafted so far
                if result.drafted_cards:
                    arch_counts = defaultdict(int)
                    for c in result.drafted_cards[:pick_num]:
                        for a in c.sa_tier_archetypes():
                            arch_counts[a] += 1
                    if arch_counts:
                        emerging = max(arch_counts, key=arch_counts.get)
                        fitting = sum(1 for c in pack if c.is_fitting(emerging))
                    else:
                        fitting = 0
                else:
                    fitting = 0
            result.early_fitting_count.append(fitting)

        elif committed is not None:
            # Late metrics (picks 6+, committed)
            fitting = sum(1 for c in pack if c.is_fitting(committed))
            result.late_fitting_count.append(fitting)

            # Strong off-archetype: S-tier in a DIFFERENT archetype or power >= 7
            off_arch = 0
            for c in pack:
                if not c.is_fitting(committed):
                    s_others = [a for a in c.s_tier_archetypes() if a != committed]
                    if s_others or c.power >= 7.0:
                        off_arch += 1
            result.late_off_archetype_count.append(off_arch)

    # Deck concentration
    if committed is not None and result.drafted_cards:
        sa_count = sum(1 for c in result.drafted_cards if c.is_fitting(committed))
        result.deck_concentration = sa_count / len(result.drafted_cards)
    else:
        result.deck_concentration = 0.0


# ---------------------------------------------------------------------------
# Aggregate metrics
# ---------------------------------------------------------------------------

def aggregate_metrics(results: list) -> dict:
    """Aggregate metrics across many draft results."""
    metrics = {
        'early_unique_archetypes': [],
        'early_fitting': [],
        'late_fitting': [],
        'late_off_archetype': [],
        'convergence_pick': [],
        'deck_concentration': [],
    }

    for r in results:
        if r.early_unique_archetypes:
            metrics['early_unique_archetypes'].append(sum(r.early_unique_archetypes) / len(r.early_unique_archetypes))
        if r.early_fitting_count:
            metrics['early_fitting'].append(sum(r.early_fitting_count) / len(r.early_fitting_count))
        if r.late_fitting_count:
            metrics['late_fitting'].append(sum(r.late_fitting_count) / len(r.late_fitting_count))
        if r.late_off_archetype_count:
            metrics['late_off_archetype'].append(sum(r.late_off_archetype_count) / len(r.late_off_archetype_count))
        if r.commitment_pick is not None:
            metrics['convergence_pick'].append(r.commitment_pick)
        if r.deck_concentration > 0:
            metrics['deck_concentration'].append(r.deck_concentration)

    agg = {}
    for key, values in metrics.items():
        if values:
            agg[key] = {
                'mean': sum(values) / len(values),
                'min': min(values),
                'max': max(values),
                'count': len(values),
            }
        else:
            agg[key] = {'mean': 0, 'min': 0, 'max': 0, 'count': 0}

    return agg


def compute_variety_metrics(results: list) -> dict:
    """Compute run-to-run variety and archetype frequency metrics."""
    # Card overlap between consecutive runs with same strategy
    overlaps = []
    for i in range(1, len(results)):
        ids_a = set(c.id for c in results[i-1].drafted_cards)
        ids_b = set(c.id for c in results[i].drafted_cards)
        if ids_a and ids_b:
            overlap = len(ids_a & ids_b) / len(ids_a | ids_b)
            overlaps.append(overlap)

    # Archetype frequency
    arch_counts = defaultdict(int)
    total_committed = 0
    for r in results:
        if r.committed_arch is not None:
            arch_counts[r.committed_arch] += 1
            total_committed += 1

    arch_freq = {}
    if total_committed > 0:
        for arch in range(NUM_ARCHETYPES):
            arch_freq[arch] = arch_counts[arch] / total_committed

    return {
        'card_overlap_mean': sum(overlaps) / len(overlaps) if overlaps else 0,
        'arch_frequency': arch_freq,
        'arch_freq_max': max(arch_freq.values()) if arch_freq else 0,
        'arch_freq_min': min(arch_freq.values()) if arch_freq else 0,
    }


# ---------------------------------------------------------------------------
# Convergence pick measurement
# ---------------------------------------------------------------------------

def measure_convergence_pick(results: list) -> float:
    """Find the average pick number where players regularly see 2+ archetype cards.

    "Regularly" = in 60%+ of packs from that pick onward.
    """
    convergence_picks = []
    for r in results:
        if r.committed_arch is None:
            continue
        committed = r.committed_arch
        # Check each pick from 3 onward
        found = False
        for start_pick in range(3, NUM_PICKS):
            # Count packs from start_pick onward that have 2+ fitting
            subsequent = [(pn, pack, _) for pn, pack, _ in r.pack_history if pn >= start_pick]
            if len(subsequent) < 3:
                break
            count_2plus = sum(1 for _, pack, _ in subsequent
                            if sum(1 for c in pack if c.is_fitting(committed)) >= 2)
            if count_2plus / len(subsequent) >= 0.60:
                convergence_picks.append(start_pick)
                found = True
                break
        if not found:
            convergence_picks.append(NUM_PICKS)  # never converged

    return sum(convergence_picks) / len(convergence_picks) if convergence_picks else NUM_PICKS


# ---------------------------------------------------------------------------
# Draft trace printer
# ---------------------------------------------------------------------------

def print_draft_trace(result: DraftResult, label: str):
    """Print a detailed pick-by-pick trace of a draft."""
    print(f"\n{'='*70}")
    print(f"DRAFT TRACE: {label}")
    print(f"Strategy: {result.strategy}")
    print(f"Committed archetype: {result.committed_arch} (at pick {result.commitment_pick})")
    print(f"Deck concentration: {result.deck_concentration:.1%}")
    print(f"{'='*70}")

    for pick_num, pack, picked in result.pack_history:
        phase = "PRE" if (pick_num < 5 or result.committed_arch is None) else "POST"
        committed_str = f" [committed: arch {result.committed_arch}]" if result.committed_arch is not None and pick_num >= (result.commitment_pick or 99) else ""

        print(f"\nPick {pick_num+1} ({phase}{committed_str}):")
        for card in pack:
            marker = " <-- PICKED" if card.id == picked.id else ""
            sa_archs = card.sa_tier_archetypes()
            s_archs = card.s_tier_archetypes()
            fitness_str = f"S:{s_archs} A:{[a for a in sa_archs if a not in s_archs]}"
            print(f"  Card {card.id:3d} | pwr {card.power:.1f} | {fitness_str} | {card.rarity}{marker}")

    print(f"\nFinal deck ({len(result.drafted_cards)} cards):")
    if result.committed_arch is not None:
        sa = sum(1 for c in result.drafted_cards if c.is_fitting(result.committed_arch))
        print(f"  S/A in arch {result.committed_arch}: {sa}/{len(result.drafted_cards)} ({sa/len(result.drafted_cards):.0%})")


# ---------------------------------------------------------------------------
# Main simulation
# ---------------------------------------------------------------------------

def run_simulation(multi_archetype_pct: Optional[float] = None, num_drafts: int = NUM_DRAFTS,
                   trace: bool = False, label: str = "") -> dict:
    """Run the full simulation with all 3 strategies."""
    cards = generate_card_pool(multi_archetype_pct)

    # Verify multi-archetype distribution
    multi_count = sum(1 for c in cards if c.num_sa_archetypes() >= 2)
    actual_multi_pct = multi_count / len(cards)

    strategies = [
        ("committed", strategy_committed),
        ("power_chaser", strategy_power_chaser),
        ("signal_reader", strategy_signal_reader),
    ]

    all_results = {}
    trace_results = {}

    for strat_name, strat_fn in strategies:
        results = []
        for i in range(num_drafts):
            # Random suppression each run
            suppressed = random.sample(range(NUM_ARCHETYPES), NUM_SUPPRESSED)
            do_trace = trace and i < 1  # trace first run of each strategy
            r = run_draft(cards, strat_fn, strat_name, suppressed, trace=do_trace)
            results.append(r)
            if do_trace:
                trace_results[strat_name] = r

        all_results[strat_name] = results

    # Aggregate metrics per strategy
    output = {"label": label, "multi_archetype_pct": actual_multi_pct}

    for strat_name, results in all_results.items():
        agg = aggregate_metrics(results)
        variety = compute_variety_metrics(results)
        conv_pick = measure_convergence_pick(results)

        output[strat_name] = {
            'agg': agg,
            'variety': variety,
            'convergence_pick': conv_pick,
            'num_committed': sum(1 for r in results if r.committed_arch is not None),
        }

    output['trace_results'] = trace_results
    return output


def print_results(output: dict):
    """Print formatted results."""
    label = output.get('label', '')
    if label:
        print(f"\n{'#'*70}")
        print(f"# {label}")
        print(f"# Multi-archetype card %: {output['multi_archetype_pct']:.1%}")
        print(f"{'#'*70}")

    for strat_name in ["committed", "power_chaser", "signal_reader"]:
        if strat_name not in output:
            continue
        data = output[strat_name]
        agg = data['agg']
        variety = data['variety']

        print(f"\n--- Strategy: {strat_name} ---")
        print(f"  Drafts committed: {data['num_committed']}/{NUM_DRAFTS}")
        print(f"  Early unique archetypes (picks 1-5): {agg['early_unique_archetypes']['mean']:.2f}")
        print(f"  Early fitting per pack (picks 1-5):  {agg['early_fitting']['mean']:.2f}")
        print(f"  Late fitting per pack (picks 6+):    {agg['late_fitting']['mean']:.2f}")
        print(f"  Late off-archetype per pack:         {agg['late_off_archetype']['mean']:.2f}")
        print(f"  Convergence pick (2+ regular):       {data['convergence_pick']:.1f}")
        print(f"  Deck concentration (S/A):            {agg['deck_concentration']['mean']:.1%}")
        print(f"  Run-to-run card overlap:             {variety['card_overlap_mean']:.1%}")
        print(f"  Archetype freq max:                  {variety['arch_freq_max']:.1%}")
        print(f"  Archetype freq min:                  {variety['arch_freq_min']:.1%}")


def print_scorecard(output: dict):
    """Print the target scorecard for the committed strategy (primary target)."""
    data = output['committed']
    agg = data['agg']
    variety = data['variety']

    print(f"\n{'='*70}")
    print("TARGET SCORECARD (committed strategy)")
    print(f"{'='*70}")

    targets = [
        ("Early unique archetypes (picks 1-5)", ">= 3", agg['early_unique_archetypes']['mean'], agg['early_unique_archetypes']['mean'] >= 3),
        ("Early fitting per pack (picks 1-5)", "<= 2", agg['early_fitting']['mean'], agg['early_fitting']['mean'] <= 2),
        ("Late fitting per pack (picks 6+)", ">= 2", agg['late_fitting']['mean'], agg['late_fitting']['mean'] >= 2),
        ("Late off-archetype per pack", ">= 0.5", agg['late_off_archetype']['mean'], agg['late_off_archetype']['mean'] >= 0.5),
        ("Convergence pick", "5-8", data['convergence_pick'], 5 <= data['convergence_pick'] <= 8),
        ("Deck concentration (S/A)", "60-80%", agg['deck_concentration']['mean'], 0.60 <= agg['deck_concentration']['mean'] <= 0.80),
        ("Run-to-run card overlap", "< 40%", variety['card_overlap_mean'], variety['card_overlap_mean'] < 0.40),
        ("Archetype freq (no arch > 20%)", "<= 20%", variety['arch_freq_max'], variety['arch_freq_max'] <= 0.20),
    ]

    # Also check min frequency
    arch_min_pass = variety['arch_freq_min'] >= 0.05 if variety['arch_freq_min'] > 0 else False
    targets.append(("Archetype freq (no arch < 5%)", ">= 5%", variety['arch_freq_min'], arch_min_pass))

    print(f"{'Metric':<45} {'Target':<10} {'Actual':<10} {'Result':<8}")
    print("-" * 73)
    for name, target, actual, passed in targets:
        status = "PASS" if passed else "FAIL"
        if isinstance(actual, float):
            if actual < 1:
                actual_str = f"{actual:.1%}"
            else:
                actual_str = f"{actual:.2f}"
        else:
            actual_str = str(actual)
        print(f"{name:<45} {target:<10} {actual_str:<10} {status:<8}")

    passes = sum(1 for _, _, _, p in targets if p)
    print(f"\nTotal: {passes}/{len(targets)} passed")


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    random.seed(42)

    print("=" * 70)
    print("MODEL C SIMULATION: Sub-Pool Carousel with Guaranteed Floors")
    print("=" * 70)

    # Main simulation
    print("\n\n### MAIN SIMULATION (default distribution) ###")
    output = run_simulation(trace=True, label="Default Distribution (~40% multi-archetype)")
    print_results(output)
    print_scorecard(output)

    # Print traces
    for strat_name, result in output['trace_results'].items():
        print_draft_trace(result, f"{strat_name} strategy")

    # Sensitivity analysis: vary multi-archetype %
    print("\n\n" + "=" * 70)
    print("MULTI-ARCHETYPE SENSITIVITY ANALYSIS")
    print("=" * 70)

    for ma_pct in [0.10, 0.20, 0.30, 0.40, 0.50, 0.60]:
        random.seed(42 + int(ma_pct * 100))
        out = run_simulation(multi_archetype_pct=ma_pct, num_drafts=500, label=f"Multi-arch {ma_pct:.0%}")

        d = out['committed']
        agg = d['agg']
        var = d['variety']
        print(f"\n  MA%={out['multi_archetype_pct']:.0%}: "
              f"early_uniq={agg['early_unique_archetypes']['mean']:.1f}, "
              f"early_fit={agg['early_fitting']['mean']:.2f}, "
              f"late_fit={agg['late_fitting']['mean']:.2f}, "
              f"late_off={agg['late_off_archetype']['mean']:.2f}, "
              f"conv_pick={d['convergence_pick']:.1f}, "
              f"deck_conc={agg['deck_concentration']['mean']:.0%}, "
              f"overlap={var['card_overlap_mean']:.0%}, "
              f"arch_max={var['arch_freq_max']:.0%}")

    print("\n\nSimulation complete.")
