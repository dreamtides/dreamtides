"""
Simulation Agent 5: Double Enhancement
Resonance Draft System V6

One-sentence algorithm:
"Draw 4 random cards; if 2 or more share a primary resonance with your top
resonance, add 2 cards of that resonance to the pack."

Zero player decisions beyond picking 1 card from the pack.
"""

import random
import statistics
from collections import defaultdict
from dataclasses import dataclass, field
from typing import Optional

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

RESONANCES = ["Ember", "Stone", "Tide", "Zephyr"]

ARCHETYPES = [
    {"name": "Flash", "primary": "Zephyr", "secondary": "Ember"},
    {"name": "Blink", "primary": "Ember", "secondary": "Zephyr"},
    {"name": "Storm", "primary": "Ember", "secondary": "Stone"},
    {"name": "Self-Discard", "primary": "Stone", "secondary": "Ember"},
    {"name": "Self-Mill", "primary": "Stone", "secondary": "Tide"},
    {"name": "Sacrifice", "primary": "Tide", "secondary": "Stone"},
    {"name": "Warriors", "primary": "Tide", "secondary": "Zephyr"},
    {"name": "Ramp", "primary": "Zephyr", "secondary": "Tide"},
]

ARCHETYPE_NAMES = [a["name"] for a in ARCHETYPES]

NUM_DRAFTS = 1000
NUM_PICKS = 30
BASE_PACK_SIZE = 4
NUM_GENERIC = 36
MAX_DUAL_TYPE = 54

SEED = 42


# ---------------------------------------------------------------------------
# Data Model
# ---------------------------------------------------------------------------

@dataclass
class SimCard:
    id: int
    symbols: list  # ordered list of resonance strings
    home_archetype: str
    archetype_fitness: dict  # archetype_name -> tier (S/A/B/C/F)
    power: float
    is_generic: bool = False

    @property
    def primary_resonance(self) -> Optional[str]:
        return self.symbols[0] if self.symbols else None

    @property
    def resonance_types(self) -> set:
        return set(self.symbols)

    @property
    def is_dual_type(self) -> bool:
        return len(self.resonance_types) >= 2


# ---------------------------------------------------------------------------
# Fitness Helpers
# ---------------------------------------------------------------------------

def get_archetype_index(name: str) -> int:
    for i, a in enumerate(ARCHETYPES):
        if a["name"] == name:
            return i
    raise ValueError(f"Unknown archetype: {name}")


def circle_distance(a_idx: int, b_idx: int) -> int:
    d = abs(a_idx - b_idx)
    return min(d, 8 - d)


def compute_fitness(home_archetype: str, target_archetype: str) -> str:
    """Compute fitness tier of a card from home_archetype for target_archetype."""
    if home_archetype == target_archetype:
        return "S"
    h_idx = get_archetype_index(home_archetype)
    t_idx = get_archetype_index(target_archetype)
    h_info = ARCHETYPES[h_idx]
    t_info = ARCHETYPES[t_idx]

    # Adjacent sharing primary resonance -> A
    if h_info["primary"] == t_info["primary"] and circle_distance(h_idx, t_idx) == 1:
        return "A"

    # Shares a resonance in some capacity -> B
    if h_info["primary"] == t_info["secondary"] or h_info["secondary"] == t_info["primary"]:
        return "B"

    # Distance-based for remaining
    dist = circle_distance(h_idx, t_idx)
    if dist <= 2:
        return "C"
    return "F"


def fitness_value(tier: str) -> float:
    return {"S": 5.0, "A": 4.0, "B": 3.0, "C": 2.0, "F": 1.0}[tier]


def is_sa_tier(tier: str) -> bool:
    return tier in ("S", "A")


def is_cf_tier(tier: str) -> bool:
    return tier in ("C", "F")


# ---------------------------------------------------------------------------
# Card Pool Construction
# ---------------------------------------------------------------------------

def build_card_pool(rng: random.Random) -> list:
    """
    Build 360 cards:
    - 36 generic (B-tier in all archetypes)
    - 324 archetype cards (40 or 41 per archetype)
    - Max 54 dual-type cards (15%)

    4 archetypes get 41 cards, 4 get 40 cards -> 164 + 160 = 324

    Symbol distribution per archetype (for 40-card archetypes):
      - 8 mono-1: [pri]
      - 18 mono-2: [pri, pri]
      - 4 dual-2: [pri, sec]  -- dual-type
      - 7 mono-3: [pri, pri, pri]
      - 3 dual-3: [pri, pri, sec]  -- dual-type
    Total dual per 40-card archetype: 7

    For 41-card archetypes, add 1 mono-2.

    Dual budget: 8 * 7 = 56, trim 2 -> first 2 archetypes get 6 dual (3 dual-2 + 3 dual-3).
    """
    cards = []
    card_id = 0

    # Generic cards
    for _ in range(NUM_GENERIC):
        fitness = {a["name"]: "B" for a in ARCHETYPES}
        cards.append(SimCard(
            id=card_id, symbols=[], home_archetype="Generic",
            archetype_fitness=fitness, power=rng.uniform(3.0, 7.0),
            is_generic=True
        ))
        card_id += 1

    dual_type_used = 0

    for arch_idx, arch in enumerate(ARCHETYPES):
        pri = arch["primary"]
        sec = arch["secondary"]
        arch_name = arch["name"]

        # First 4 archetypes get 41 cards, last 4 get 40
        total_cards = 41 if arch_idx < 4 else 40

        # Dual-type budget: first 2 get 6, rest get 7
        if arch_idx < 2:
            dual_count = 6
        else:
            dual_count = 7
        # Ensure we don't exceed 54 total
        if dual_type_used + dual_count > MAX_DUAL_TYPE:
            dual_count = MAX_DUAL_TYPE - dual_type_used
        dual_type_used += dual_count

        dual_2 = (dual_count + 1) // 2
        dual_3 = dual_count - dual_2

        mono_1 = 8
        mono_2 = 18 + (1 if total_cards == 41 else 0)
        mono_3 = total_cards - mono_1 - mono_2 - dual_2 - dual_3

        def make_card(symbols, home=arch_name):
            nonlocal card_id
            fitness = {}
            for a in ARCHETYPES:
                fitness[a["name"]] = compute_fitness(home, a["name"])
            c = SimCard(
                id=card_id, symbols=symbols, home_archetype=home,
                archetype_fitness=fitness, power=rng.uniform(2.0, 9.0)
            )
            card_id += 1
            return c

        for _ in range(mono_1):
            cards.append(make_card([pri]))
        for _ in range(mono_2):
            cards.append(make_card([pri, pri]))
        for _ in range(dual_2):
            cards.append(make_card([pri, sec]))
        for _ in range(mono_3):
            cards.append(make_card([pri, pri, pri]))
        for _ in range(dual_3):
            cards.append(make_card([pri, pri, sec]))

    return cards


# ---------------------------------------------------------------------------
# Player Resonance Tracking
# ---------------------------------------------------------------------------

@dataclass
class PlayerState:
    drafted: list = field(default_factory=list)
    resonance_counts: dict = field(default_factory=lambda: defaultdict(float))
    target_archetype: Optional[str] = None

    def add_card(self, card: SimCard):
        self.drafted.append(card)
        for i, sym in enumerate(card.symbols):
            weight = 2.0 if i == 0 else 1.0
            self.resonance_counts[sym] += weight

    @property
    def top_resonance(self) -> Optional[str]:
        if not self.resonance_counts:
            return None
        return max(self.resonance_counts, key=self.resonance_counts.get)

    @property
    def top_resonance_count(self) -> float:
        if not self.resonance_counts:
            return 0
        return max(self.resonance_counts.values())

    @property
    def second_resonance(self) -> Optional[str]:
        if len(self.resonance_counts) < 2:
            return None
        sorted_res = sorted(self.resonance_counts.items(), key=lambda x: -x[1])
        return sorted_res[1][0]


# ---------------------------------------------------------------------------
# Double Enhancement Algorithm
# ---------------------------------------------------------------------------

def generate_pack_double_enhancement(
    pool: list,
    player: PlayerState,
    rng: random.Random,
    trigger_threshold: int = 2,
    bonus_count: int = 2,
    bonus_pool_mode: str = "primary",
    min_activation: float = 4.0,
) -> tuple:
    """
    Double Enhancement:
    1. Draw 4 random cards from pool.
    2. If player has >= min_activation weighted symbols in top resonance, AND
       trigger_threshold or more of the 4 cards have primary resonance == player's
       top resonance, add bonus_count cards with that primary resonance.
    3. Player picks 1.

    Zero decisions.
    """
    base_pack = rng.sample(pool, min(BASE_PACK_SIZE, len(pool)))

    top_res = player.top_resonance
    top_count = player.top_resonance_count

    if top_res is not None and top_count >= min_activation:
        matching = sum(1 for c in base_pack if c.primary_resonance == top_res)
        if matching >= trigger_threshold:
            if bonus_pool_mode == "dual_only":
                second_res = player.second_resonance
                if second_res:
                    bp = [c for c in pool if c.primary_resonance == top_res
                          and c.is_dual_type and second_res in c.resonance_types
                          and c not in base_pack]
                else:
                    bp = []
            else:
                bp = [c for c in pool if c.primary_resonance == top_res
                      and c not in base_pack]

            if bp:
                bonus_cards = [rng.choice(bp) for _ in range(bonus_count)]
                return base_pack + bonus_cards, True

    return base_pack, False


# ---------------------------------------------------------------------------
# Player Strategies
# ---------------------------------------------------------------------------

def pick_archetype_committed(pack, player, pick_num):
    """Picks highest fitness for strongest emerging archetype. Commits ~pick 5."""
    if player.target_archetype is None and pick_num >= 5 and player.drafted:
        arch_scores = defaultdict(float)
        for c in player.drafted:
            for a_name, tier in c.archetype_fitness.items():
                arch_scores[a_name] += fitness_value(tier)
        player.target_archetype = max(arch_scores, key=arch_scores.get)

    if player.target_archetype:
        return max(pack, key=lambda c: (
            fitness_value(c.archetype_fitness.get(player.target_archetype, "F")),
            c.power
        ))
    else:
        return max(pack, key=lambda c: c.power + len(c.symbols) * 0.5)


def pick_power_chaser(pack, player, pick_num):
    """Always picks highest raw power."""
    return max(pack, key=lambda c: c.power)


def pick_signal_reader(pack, player, pick_num):
    """Evaluates pack composition and drafts toward open archetype."""
    if player.target_archetype is None and pick_num >= 4:
        arch_scores = defaultdict(float)
        for c in player.drafted:
            for a_name, tier in c.archetype_fitness.items():
                arch_scores[a_name] += fitness_value(tier)
        for c in pack:
            for a_name, tier in c.archetype_fitness.items():
                arch_scores[a_name] += fitness_value(tier) * 0.3
        if arch_scores:
            player.target_archetype = max(arch_scores, key=arch_scores.get)

    if player.target_archetype:
        return max(pack, key=lambda c: (
            fitness_value(c.archetype_fitness.get(player.target_archetype, "F")),
            c.power * 0.3
        ))
    else:
        return max(pack, key=lambda c: max(
            fitness_value(t) for t in c.archetype_fitness.values()
        ))


STRATEGIES = {
    "committed": pick_archetype_committed,
    "power_chaser": pick_power_chaser,
    "signal_reader": pick_signal_reader,
}


# ---------------------------------------------------------------------------
# Metrics Collection
# ---------------------------------------------------------------------------

@dataclass
class DraftMetrics:
    sa_per_pack: list = field(default_factory=list)
    cf_per_pack: list = field(default_factory=list)
    unique_archs_with_sa: list = field(default_factory=list)
    pack_sizes: list = field(default_factory=list)
    trigger_fired: list = field(default_factory=list)
    convergence_pick: Optional[int] = None
    final_deck_sa_pct: float = 0.0
    final_archetype: Optional[str] = None
    card_ids_drafted: list = field(default_factory=list)


def evaluate_pack_archetype(pack, target_archetype):
    """Evaluate a pack at archetype level for a target archetype."""
    sa_count = 0
    cf_count = 0
    archs_with_sa = set()

    for card in pack:
        tier = card.archetype_fitness.get(target_archetype, "F")
        if is_sa_tier(tier):
            sa_count += 1
        if is_cf_tier(tier):
            cf_count += 1
        for arch_name, t in card.archetype_fitness.items():
            if is_sa_tier(t):
                archs_with_sa.add(arch_name)

    return {
        "sa_count": sa_count,
        "cf_count": cf_count,
        "unique_archs_with_sa": len(archs_with_sa),
    }


# ---------------------------------------------------------------------------
# Draft Simulation
# ---------------------------------------------------------------------------

def determine_target_archetype(player):
    """Determine the player's target archetype from drafted cards."""
    if player.target_archetype:
        return player.target_archetype
    arch_scores = defaultdict(float)
    for c in player.drafted:
        for a_name, tier in c.archetype_fitness.items():
            arch_scores[a_name] += fitness_value(tier)
    if arch_scores:
        return max(arch_scores, key=arch_scores.get)
    return ARCHETYPES[0]["name"]


def run_single_draft(
    pool, strategy_name, rng,
    trigger_threshold=2, bonus_count=2, bonus_pool_mode="primary",
    min_activation=4.0, trace=False,
):
    """Run a single 30-pick draft and return metrics."""
    player = PlayerState()
    metrics = DraftMetrics()
    strategy_fn = STRATEGIES[strategy_name]
    trace_lines = [] if trace else None

    for pick_num in range(NUM_PICKS):
        pack, triggered = generate_pack_double_enhancement(
            pool, player, rng,
            trigger_threshold=trigger_threshold,
            bonus_count=bonus_count,
            bonus_pool_mode=bonus_pool_mode,
            min_activation=min_activation,
        )

        metrics.pack_sizes.append(len(pack))
        metrics.trigger_fired.append(triggered)

        target = determine_target_archetype(player)
        pack_eval = evaluate_pack_archetype(pack, target)
        metrics.sa_per_pack.append(pack_eval["sa_count"])
        metrics.cf_per_pack.append(pack_eval["cf_count"])
        metrics.unique_archs_with_sa.append(pack_eval["unique_archs_with_sa"])

        chosen = strategy_fn(pack, player, pick_num)
        player.add_card(chosen)
        metrics.card_ids_drafted.append(chosen.id)

        if metrics.convergence_pick is None and pick_num >= 2:
            trailing = metrics.sa_per_pack[-3:]
            if statistics.mean(trailing) >= 2.0:
                metrics.convergence_pick = pick_num

        if trace:
            trace_lines.append(
                f"  Pick {pick_num+1}: pack={len(pack)}, trig={triggered}, "
                f"sa={pack_eval['sa_count']}, cf={pack_eval['cf_count']}, "
                f"top_res={player.top_resonance}({player.top_resonance_count:.0f}), "
                f"target={target}, chose={chosen.home_archetype}"
                f"{'['+chosen.primary_resonance+']' if chosen.primary_resonance else '[generic]'}"
            )

    final_target = determine_target_archetype(player)
    metrics.final_archetype = final_target
    sa_in_deck = sum(
        1 for c in player.drafted
        if is_sa_tier(c.archetype_fitness.get(final_target, "F"))
    )
    metrics.final_deck_sa_pct = sa_in_deck / len(player.drafted) if player.drafted else 0

    if trace:
        return metrics, trace_lines
    return metrics


# ---------------------------------------------------------------------------
# Aggregate Analysis
# ---------------------------------------------------------------------------

def run_experiment(
    pool, rng,
    trigger_threshold=2, bonus_count=2, bonus_pool_mode="primary",
    min_activation=4.0, num_drafts=NUM_DRAFTS, label="",
):
    """Run full experiment across all strategies."""
    all_metrics = defaultdict(list)

    for strategy_name in STRATEGIES:
        for _ in range(num_drafts):
            m = run_single_draft(
                pool, strategy_name, rng,
                trigger_threshold=trigger_threshold,
                bonus_count=bonus_count,
                bonus_pool_mode=bonus_pool_mode,
                min_activation=min_activation,
            )
            all_metrics[strategy_name].append(m)

    return aggregate_metrics(all_metrics, label)


def aggregate_metrics(all_metrics, label):
    """Compute all required metrics."""
    results = {"label": label, "strategies": {}}

    for strategy_name, drafts in all_metrics.items():
        n = len(drafts)

        # Picks 1-5 metrics
        early_unique_archs = []
        early_sa = []
        for d in drafts:
            for pick in range(min(5, len(d.unique_archs_with_sa))):
                early_unique_archs.append(d.unique_archs_with_sa[pick])
                early_sa.append(d.sa_per_pack[pick])

        # Picks 6+ metrics
        late_sa = []
        late_cf = []
        for d in drafts:
            for pick in range(5, len(d.sa_per_pack)):
                late_sa.append(d.sa_per_pack[pick])
                late_cf.append(d.cf_per_pack[pick])

        # Convergence
        convergence_picks = [d.convergence_pick for d in drafts if d.convergence_pick is not None]
        avg_convergence = statistics.mean(convergence_picks) if convergence_picks else 30
        convergence_rate = len(convergence_picks) / n

        # Deck concentration
        deck_concentrations = [d.final_deck_sa_pct for d in drafts]
        avg_concentration = statistics.mean(deck_concentrations)

        # Card overlap
        by_arch = defaultdict(list)
        for d in drafts:
            by_arch[d.final_archetype].append(set(d.card_ids_drafted))
        overlaps = []
        for arch, deck_sets in by_arch.items():
            limit = min(50, len(deck_sets))
            for i in range(limit):
                for j in range(i + 1, limit):
                    inter = len(deck_sets[i] & deck_sets[j])
                    overlaps.append(inter / 30.0)
        avg_overlap = statistics.mean(overlaps) if overlaps else 0

        # Archetype frequency
        arch_freq = defaultdict(int)
        for d in drafts:
            arch_freq[d.final_archetype] += 1
        arch_pcts = {a: arch_freq.get(a, 0) / n for a in ARCHETYPE_NAMES}

        # Variance
        late_sa_stddev = statistics.stdev(late_sa) if len(late_sa) >= 2 else 0

        # Trigger frequency by phase
        early_triggers = []
        mid_triggers = []
        late_triggers = []
        for d in drafts:
            for pick in range(len(d.trigger_fired)):
                val = 1 if d.trigger_fired[pick] else 0
                if pick < 5:
                    early_triggers.append(val)
                elif pick < 15:
                    mid_triggers.append(val)
                else:
                    late_triggers.append(val)

        # Per-archetype convergence
        arch_convergence = {}
        for arch_name in ARCHETYPE_NAMES:
            arch_drafts = [d for d in drafts if d.final_archetype == arch_name]
            if arch_drafts:
                conv_picks = [d.convergence_pick for d in arch_drafts
                              if d.convergence_pick is not None]
                arch_convergence[arch_name] = {
                    "avg_convergence": statistics.mean(conv_picks) if conv_picks else 30,
                    "count": len(arch_drafts),
                    "convergence_rate": len(conv_picks) / len(arch_drafts),
                }
            else:
                arch_convergence[arch_name] = {
                    "avg_convergence": 30, "count": 0, "convergence_rate": 0
                }

        # Pack sizes
        all_pack_sizes = []
        for d in drafts:
            all_pack_sizes.extend(d.pack_sizes)

        # S/A distribution for histogram
        late_sa_distribution = defaultdict(int)
        for v in late_sa:
            late_sa_distribution[v] += 1

        results["strategies"][strategy_name] = {
            "avg_early_unique_archs": statistics.mean(early_unique_archs) if early_unique_archs else 0,
            "avg_early_sa": statistics.mean(early_sa) if early_sa else 0,
            "avg_late_sa": statistics.mean(late_sa) if late_sa else 0,
            "avg_late_cf": statistics.mean(late_cf) if late_cf else 0,
            "avg_convergence_pick": avg_convergence,
            "convergence_rate": convergence_rate,
            "avg_concentration": avg_concentration,
            "avg_overlap": avg_overlap,
            "arch_frequency": arch_pcts,
            "late_sa_stddev": late_sa_stddev,
            "trigger_freq_early": statistics.mean(early_triggers) if early_triggers else 0,
            "trigger_freq_mid": statistics.mean(mid_triggers) if mid_triggers else 0,
            "trigger_freq_late": statistics.mean(late_triggers) if late_triggers else 0,
            "arch_convergence": arch_convergence,
            "avg_pack_size": statistics.mean(all_pack_sizes) if all_pack_sizes else 4,
            "late_sa_distribution": dict(late_sa_distribution),
        }

    return results


# ---------------------------------------------------------------------------
# Draft Traces
# ---------------------------------------------------------------------------

def run_traces(pool, rng, params):
    traces = []
    m, lines = run_single_draft(pool, "committed", rng, trace=True, **params)
    traces.append(("Early Committer (committed)", m, lines))
    m, lines = run_single_draft(pool, "power_chaser", rng, trace=True, **params)
    traces.append(("Flexible Player (power chaser)", m, lines))
    m, lines = run_single_draft(pool, "signal_reader", rng, trace=True, **params)
    traces.append(("Signal Reader", m, lines))
    return traces


# ---------------------------------------------------------------------------
# Parameter Sensitivity Sweep
# ---------------------------------------------------------------------------

def parameter_sweep(pool, base_rng_seed):
    """Sweep trigger_threshold, bonus_count, and bonus_pool_mode."""
    configs = [
        {"trigger_threshold": 1, "bonus_count": 1, "bonus_pool_mode": "primary",
         "label": "thresh=1, bonus=1, primary"},
        {"trigger_threshold": 1, "bonus_count": 2, "bonus_pool_mode": "primary",
         "label": "thresh=1, bonus=2, primary"},
        {"trigger_threshold": 1, "bonus_count": 2, "bonus_pool_mode": "dual_only",
         "label": "thresh=1, bonus=2, dual-only"},
        {"trigger_threshold": 2, "bonus_count": 1, "bonus_pool_mode": "primary",
         "label": "thresh=2, bonus=1, primary"},
        {"trigger_threshold": 2, "bonus_count": 2, "bonus_pool_mode": "primary",
         "label": "thresh=2, bonus=2, primary"},
        {"trigger_threshold": 2, "bonus_count": 2, "bonus_pool_mode": "dual_only",
         "label": "thresh=2, bonus=2, dual-only"},
        {"trigger_threshold": 2, "bonus_count": 3, "bonus_pool_mode": "primary",
         "label": "thresh=2, bonus=3, primary"},
        {"trigger_threshold": 3, "bonus_count": 2, "bonus_pool_mode": "primary",
         "label": "thresh=3, bonus=2, primary"},
    ]

    sweep_results = []
    for config in configs:
        label = config.pop("label")
        rng = random.Random(base_rng_seed)
        result = run_experiment(pool, rng, num_drafts=500, label=label, **config)
        c = result["strategies"]["committed"]
        sweep_results.append({
            "label": label,
            "late_sa": c["avg_late_sa"],
            "late_sa_stddev": c["late_sa_stddev"],
            "convergence": c["avg_convergence_pick"],
            "convergence_rate": c["convergence_rate"],
            "concentration": c["avg_concentration"],
            "trigger_mid": c["trigger_freq_mid"],
            "trigger_late": c["trigger_freq_late"],
            "avg_pack_size": c["avg_pack_size"],
            "early_unique": c["avg_early_unique_archs"],
            "early_sa": c["avg_early_sa"],
            "late_cf": c["avg_late_cf"],
            "overlap": c["avg_overlap"],
        })

    return sweep_results


# ---------------------------------------------------------------------------
# Report Generation
# ---------------------------------------------------------------------------

def format_results(main_results, sweep, traces, pool, best_label):
    lines = []

    lines.append("# Agent 5 Simulation Results: Double Enhancement")
    lines.append("")
    lines.append('**One-sentence algorithm:** "Draw 4 random cards; if 2 or more share a '
                 "primary resonance with your top resonance, add 2 cards of that resonance "
                 'to the pack."')
    lines.append("")
    lines.append("**One-sentence test:** Implementable from the sentence alone. The only "
                 "implicit state is 'top resonance' (weighted count of drafted symbols, "
                 "primary=2, others=1). Zero player decisions beyond picking 1 card.")
    lines.append("")

    dual_count = sum(1 for c in pool if c.is_dual_type)
    generic_count = sum(1 for c in pool if c.is_generic)
    lines.append(f"**Card pool:** {len(pool)} cards, {generic_count} generic, "
                 f"{dual_count} dual-type ({dual_count/len(pool)*100:.1f}%)")
    lines.append("")

    # Symbol distribution summary
    sym_counts = defaultdict(int)
    for c in pool:
        if c.is_generic:
            sym_counts["generic"] += 1
        elif c.is_dual_type:
            sym_counts[f"dual-{len(c.symbols)}sym"] += 1
        else:
            sym_counts[f"mono-{len(c.symbols)}sym"] += 1
    lines.append("**Symbol distribution:** " + ", ".join(
        f"{k}: {v}" for k, v in sorted(sym_counts.items())))
    lines.append("")

    # Main scorecard
    lines.append("## Scorecard (Champion config: thresh=2, bonus=2, primary pool)")
    lines.append("")
    lines.append("| Metric | Target | Committed | Power Chaser | Signal Reader |")
    lines.append("|--------|--------|-----------|--------------|---------------|")

    strats = main_results["strategies"]

    def row(metric_name, target, key, fmt=".2f"):
        vals = []
        for s in ["committed", "power_chaser", "signal_reader"]:
            v = strats[s][key]
            vals.append(f"{v:{fmt}}")
        return f"| {metric_name} | {target} | {' | '.join(vals)} |"

    lines.append(row("Picks 1-5: unique archs w/ S/A", ">= 3", "avg_early_unique_archs"))
    lines.append(row("Picks 1-5: S/A for emerging arch", "<= 2", "avg_early_sa"))
    lines.append(row("Picks 6+: S/A per pack", ">= 2.0", "avg_late_sa"))
    lines.append(row("Picks 6+: C/F per pack", ">= 0.5", "avg_late_cf"))
    lines.append(row("Convergence pick", "5-8", "avg_convergence_pick", ".1f"))
    lines.append(row("Convergence rate", "high", "convergence_rate", ".1%"))
    lines.append(row("Deck concentration", "60-90%", "avg_concentration", ".1%"))
    lines.append(row("Card overlap", "< 40%", "avg_overlap", ".1%"))
    lines.append(row("S/A stddev (picks 6+)", ">= 0.8", "late_sa_stddev"))
    lines.append("")

    # Archetype frequency
    lines.append("## Archetype Frequency (committed)")
    lines.append("")
    lines.append("| Archetype | Frequency | In range? |")
    lines.append("|-----------|-----------|-----------|")
    for arch in ARCHETYPE_NAMES:
        pct = strats["committed"]["arch_frequency"].get(arch, 0)
        flag = "OK" if 0.05 <= pct <= 0.20 else "OUT"
        lines.append(f"| {arch} | {pct:.1%} | {flag} |")
    lines.append("")

    # Per-archetype convergence
    lines.append("## Per-Archetype Convergence (committed)")
    lines.append("")
    lines.append("| Archetype | Avg Conv Pick | Conv Rate | Count |")
    lines.append("|-----------|---------------|-----------|-------|")
    for arch in ARCHETYPE_NAMES:
        ac = strats["committed"]["arch_convergence"][arch]
        lines.append(f"| {arch} | {ac['avg_convergence']:.1f} | "
                     f"{ac['convergence_rate']:.0%} | {ac['count']} |")
    lines.append("")

    # Trigger frequency
    lines.append("## Trigger Frequency by Phase (committed)")
    lines.append("")
    lines.append("| Phase | Trigger Rate |")
    lines.append("|-------|-------------|")
    c = strats["committed"]
    lines.append(f"| Picks 1-5 | {c['trigger_freq_early']:.1%} |")
    lines.append(f"| Picks 6-15 | {c['trigger_freq_mid']:.1%} |")
    lines.append(f"| Picks 16-30 | {c['trigger_freq_late']:.1%} |")
    lines.append(f"| Avg pack size | {c['avg_pack_size']:.2f} |")
    lines.append("")

    # Pack quality variance
    lines.append("## Pack-Quality Variance (committed, picks 6+)")
    lines.append("")
    lines.append(f"- Mean S/A per pack: {c['avg_late_sa']:.2f}")
    lines.append(f"- StdDev S/A per pack: {c['late_sa_stddev']:.2f}")
    lines.append(f"- Mean C/F per pack: {c['avg_late_cf']:.2f}")
    lines.append(f"- Avg pack size: {c['avg_pack_size']:.2f}")
    dist = c.get("late_sa_distribution", {})
    if dist:
        total_packs = sum(dist.values())
        lines.append(f"- S/A distribution: " + ", ".join(
            f"{k}={v/total_packs:.0%}" for k, v in sorted(dist.items())))
    lines.append("")

    # Parameter sweep
    lines.append("## Parameter Sensitivity (committed, 500 drafts each)")
    lines.append("")
    lines.append("| Config | Late S/A | StdDev | Conv | Conv% | Conc% | Trig Mid | Trig Late | Pack | C/F | Overlap |")
    lines.append("|--------|----------|--------|------|-------|-------|----------|-----------|------|-----|---------|")
    for s in sweep:
        lines.append(
            f"| {s['label']} | {s['late_sa']:.2f} | {s['late_sa_stddev']:.2f} | "
            f"{s['convergence']:.1f} | {s['convergence_rate']:.0%} | "
            f"{s['concentration']:.0%} | {s['trigger_mid']:.0%} | "
            f"{s['trigger_late']:.0%} | {s['avg_pack_size']:.2f} | "
            f"{s['late_cf']:.2f} | {s['overlap']:.1%} |"
        )
    lines.append("")

    # Best variant analysis
    lines.append(f"**Best variant:** {best_label}")
    lines.append("")

    # Traces
    lines.append("## Draft Traces")
    lines.append("")
    for title, metrics, trace_lines in traces:
        lines.append(f"### {title}")
        lines.append(f"Final archetype: {metrics.final_archetype}, "
                     f"Deck S/A: {metrics.final_deck_sa_pct:.0%}, "
                     f"Convergence: pick {metrics.convergence_pick or 'N/A'}")
        lines.append("```")
        show_picks = list(range(5)) + [9, 14, 19, 24, 29]
        for p in show_picks:
            if p < len(trace_lines):
                lines.append(trace_lines[p])
        lines.append("```")
        lines.append("")

    # Baseline comparison
    lines.append("## Baseline Comparison")
    lines.append("")
    lines.append("Agent 1 results not yet available. Reference points from prior versions:")
    lines.append("- V3 Lane Locking: 2.72 S/A, convergence pick 6.1")
    lines.append("- V4 Pack Widening: 3.35 S/A, convergence pick 6.0")
    lines.append("")

    # Self-assessment
    late_sa_val = strats["committed"]["avg_late_sa"]
    conv_val = strats["committed"]["avg_convergence_pick"]

    convergent_score = 8 if late_sa_val >= 2.0 else (6 if late_sa_val >= 1.5 else 4)

    lines.append("## Self-Assessment (1-10)")
    lines.append("")
    lines.append("| Goal | Score | Note |")
    lines.append("|------|-------|------|")
    lines.append(f"| 1. Simple | 8 | One sentence, one condition, one action |")
    lines.append(f"| 2. No actions | 10 | Zero player decisions verified in code |")
    lines.append(f"| 3. Not on rails | 7 | 4/6 pack split; untriggered packs stay diverse |")
    lines.append(f"| 4. No forced decks | 7 | Random base pack ensures variety |")
    lines.append(f"| 5. Flexible | 5 | Single-resonance trigger allows cross-archetype |")
    lines.append(f"| 6. Convergent | {convergent_score} | Late S/A = {late_sa_val:.2f} |")
    lines.append(f"| 7. Splashable | 6 | C/F = {strats['committed']['avg_late_cf']:.2f} per pack |")
    lines.append(f"| 8. Open early | 7 | Low early trigger rate ({strats['committed']['trigger_freq_early']:.0%}) |")
    lines.append(f"| 9. Signal reading | 5 | Enhancement signals resonance match |")
    lines.append("")

    # Zero-decision verification
    lines.append("## Zero-Decision Verification")
    lines.append("")
    lines.append("The implementation contains zero player decisions beyond card selection. "
                 "The trigger (2+ primary resonance match in base pack) is evaluated "
                 "automatically. Bonus cards are added automatically. The player's only "
                 "action is choosing 1 card from the presented pack (4 or 6 cards).")

    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    print("Building card pool...")
    rng = random.Random(SEED)
    pool = build_card_pool(rng)

    total = len(pool)
    dual_count = sum(1 for c in pool if c.is_dual_type)
    generic_count = sum(1 for c in pool if c.is_generic)
    print(f"Pool: {total} cards, {generic_count} generic, {dual_count} dual-type "
          f"({dual_count/total*100:.1f}%)")

    arch_counts = defaultdict(int)
    for c in pool:
        if not c.is_generic:
            arch_counts[c.home_archetype] += 1
    for a in ARCHETYPE_NAMES:
        print(f"  {a}: {arch_counts[a]} cards")

    # Verify fitness model
    print("\nFitness spot-check (Warriors card for each archetype):")
    warriors_card = next(c for c in pool if c.home_archetype == "Warriors")
    for a in ARCHETYPE_NAMES:
        print(f"  {a}: {warriors_card.archetype_fitness[a]}")

    # Resonance distribution in pool
    res_primary_count = defaultdict(int)
    for c in pool:
        if c.primary_resonance:
            res_primary_count[c.primary_resonance] += 1
    print("\nPrimary resonance distribution:")
    for r in RESONANCES:
        cnt = res_primary_count[r]
        print(f"  {r}: {cnt} cards ({cnt/total*100:.1f}%)")

    # ====================================================================
    # Champion configuration
    # ====================================================================
    champion_params = {
        "trigger_threshold": 2,
        "bonus_count": 2,
        "bonus_pool_mode": "primary",
        "min_activation": 4.0,
    }

    print("\n=== Main Experiment (1000 drafts x 3 strategies) ===")
    main_rng = random.Random(SEED + 1)
    main_results = run_experiment(pool, main_rng, label="Champion", **champion_params)

    for strat, data in main_results["strategies"].items():
        print(f"\n--- {strat} ---")
        print(f"  Late S/A: {data['avg_late_sa']:.2f} (stddev {data['late_sa_stddev']:.2f})")
        print(f"  Early unique archs: {data['avg_early_unique_archs']:.2f}")
        print(f"  Early S/A: {data['avg_early_sa']:.2f}")
        print(f"  Conv: pick {data['avg_convergence_pick']:.1f} (rate {data['convergence_rate']:.0%})")
        print(f"  Deck conc: {data['avg_concentration']:.1%}")
        print(f"  Overlap: {data['avg_overlap']:.1%}")
        print(f"  Late C/F: {data['avg_late_cf']:.2f}")
        print(f"  Triggers: early={data['trigger_freq_early']:.1%} "
              f"mid={data['trigger_freq_mid']:.1%} late={data['trigger_freq_late']:.1%}")
        print(f"  Avg pack: {data['avg_pack_size']:.2f}")

    # ====================================================================
    # Parameter sweep
    # ====================================================================
    print("\n=== Parameter Sweep (500 drafts each) ===")
    sweep = parameter_sweep(pool, SEED + 2)
    best_sweep = max(sweep, key=lambda s: s["late_sa"] if s["late_sa"] <= 3.5 else 0)
    for s in sweep:
        marker = " ***" if s["label"] == best_sweep["label"] else ""
        print(f"  {s['label']}: late_sa={s['late_sa']:.2f}, "
              f"conv={s['convergence']:.1f}, trig_late={s['trigger_late']:.0%}, "
              f"pack={s['avg_pack_size']:.2f}{marker}")

    # ====================================================================
    # Draft traces
    # ====================================================================
    print("\n=== Draft Traces ===")
    trace_rng = random.Random(SEED + 3)
    traces = run_traces(pool, trace_rng, champion_params)
    for title, metrics, tlines in traces:
        print(f"\n{title}: arch={metrics.final_archetype}, "
              f"deck_sa={metrics.final_deck_sa_pct:.0%}, "
              f"conv={metrics.convergence_pick}")

    # ====================================================================
    # Generate report
    # ====================================================================
    print("\nGenerating report...")
    report = format_results(main_results, sweep, traces, pool, best_sweep["label"])

    report_path = "/Users/dthurn/Documents/GoogleDrive/dreamtides/docs/resonance/v6/results_5.md"
    with open(report_path, "w") as f:
        f.write(report)
    print(f"Report written to {report_path}")


if __name__ == "__main__":
    main()
