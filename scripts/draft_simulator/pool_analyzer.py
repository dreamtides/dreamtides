"""Pool analysis for comparing real card designs against ideal draft targets.

Loads card designs from a TOML metadata file, infers rarity tiers,
classifies cards by archetype role, and produces a gap analysis showing
what remains to fill. Stdlib-only, no external dependencies.
"""

import math
import sys
from dataclasses import dataclass

import colors
from config import RarityConfig, SimulatorConfig
from draft_models import CardDesign

ARCHETYPE_NAMES: list[str] = [
    "flash",
    "awaken",
    "flicker",
    "ignite",
    "shatter",
    "endure",
    "submerge",
    "surge",
]


# ---------------------------------------------------------------------------
# Data models
# ---------------------------------------------------------------------------


@dataclass
class PoolAnalysis:
    """Result of analyzing a real card pool."""

    cards: list[CardDesign]
    primary_count: int
    bridge_count: int
    neutral_count: int
    per_archetype_primaries: list[int]
    per_archetype_coverage: list[int]
    bridge_pairs: dict[tuple[int, int], int]
    rarity_counts: dict[str, int]
    powers: list[float]
    power_per_rarity: dict[str, list[float]]


@dataclass
class IdealTargets:
    """Ideal pool composition targets derived from config."""

    pool_size: int
    bridge_count: int
    primary_count: int
    primaries_per_archetype: int
    coverage_per_archetype: int
    per_tier_total: dict[str, int]
    per_tier_bridge: dict[str, int]
    per_tier_primary: dict[str, int]
    per_tier_primaries_per_archetype: dict[str, int]


@dataclass
class GapAnalysis:
    """What's missing between current pool and ideal targets."""

    remaining_total: int
    per_archetype_coverage_needed: list[int]
    bridge_pairs_needed: dict[tuple[int, int], int]
    per_rarity_needed: dict[str, int]
    per_rarity_power_range: dict[str, tuple[float, float]]


# ---------------------------------------------------------------------------
# TOML loading
# ---------------------------------------------------------------------------


def load_cards_from_toml(path: str) -> list[CardDesign]:
    """Load card designs from a card-metadata TOML file."""
    if sys.version_info < (3, 11):
        raise RuntimeError("Requires Python 3.11+ for tomllib")
    import tomllib

    with open(path, "rb") as f:
        data = tomllib.load(f)

    entries = data.get("card-metadata", [])
    cards: list[CardDesign] = []
    for entry in entries:
        fitness = [float(entry.get(name, 0.0)) for name in ARCHETYPE_NAMES]
        power = float(entry.get("power", 0.0))
        commit = float(entry.get("commit", 0.0))
        flex = float(entry.get("flex", 0.0))
        card_id = entry.get("card-id", "")
        cards.append(
            CardDesign(
                card_id=card_id,
                name=card_id,
                fitness=fitness,
                power=power,
                commit=commit,
                flex=flex,
            )
        )
    return cards


# ---------------------------------------------------------------------------
# Rarity inference
# ---------------------------------------------------------------------------


def infer_rarity(cards: list[CardDesign], rarity_cfg: RarityConfig) -> list[CardDesign]:
    """Assign rarity tiers to cards based on power and balance targets."""
    total = len(cards)
    if total == 0:
        return []

    tier_design_counts = list(rarity_cfg.tier_design_counts)
    tier_total = sum(tier_design_counts)
    # Target fractions from design counts
    target_fracs = [c / tier_total for c in tier_design_counts]

    sorted_cards = sorted(cards, key=lambda c: c.power)
    tier_counts = [0] * len(rarity_cfg.tiers)
    result: list[CardDesign] = []

    for card in sorted_cards:
        best_tier = 0
        best_score = -1.0

        for ti in range(len(rarity_cfg.tiers)):
            lo, hi = rarity_cfg.tier_power_ranges[ti]
            # Fit score: 1.0 if in range, linear decay outside
            if lo <= card.power <= hi:
                fit_score = 1.0
            else:
                dist = min(abs(card.power - lo), abs(card.power - hi))
                fit_score = max(0.0, 1.0 - dist * 5.0)

            # Balance score: favor under-filled tiers
            current_frac = tier_counts[ti] / total if total > 0 else 0.0
            balance_score = max(0.0, target_fracs[ti] - current_frac)

            score = fit_score + balance_score
            if score > best_score:
                best_score = score
                best_tier = ti

        tier_counts[best_tier] += 1
        result.append(
            CardDesign(
                card_id=card.card_id,
                name=card.name,
                fitness=card.fitness,
                power=card.power,
                commit=card.commit,
                flex=card.flex,
                rarity=rarity_cfg.tiers[best_tier],
            )
        )

    return result


# ---------------------------------------------------------------------------
# Classification
# ---------------------------------------------------------------------------


def _classify_card(card: CardDesign) -> tuple[str, list[int]]:
    """Classify a card as primary, bridge, or neutral."""
    strong = [i for i, f in enumerate(card.fitness) if f > 0.5]
    if len(strong) == 0:
        return "neutral", strong
    elif len(strong) == 1:
        return "primary", strong
    else:
        return "bridge", strong


# ---------------------------------------------------------------------------
# Pool analysis
# ---------------------------------------------------------------------------


def analyze_pool(cards: list[CardDesign], cfg: SimulatorConfig) -> PoolAnalysis:
    """Analyze the composition of a card pool."""
    archetype_count = cfg.cards.archetype_count
    primary_count = 0
    bridge_count = 0
    neutral_count = 0
    per_archetype_primaries = [0] * archetype_count
    per_archetype_coverage = [0] * archetype_count
    bridge_pairs: dict[tuple[int, int], int] = {}

    for card in cards:
        role, strong_archs = _classify_card(card)
        if role == "primary":
            primary_count += 1
            per_archetype_primaries[strong_archs[0]] += 1
        elif role == "bridge":
            bridge_count += 1
            for i in range(len(strong_archs)):
                for j in range(i + 1, len(strong_archs)):
                    pair = (strong_archs[i], strong_archs[j])
                    bridge_pairs[pair] = bridge_pairs.get(pair, 0) + 1
        else:
            neutral_count += 1
        # Coverage: every archetype where fitness > 0.5
        for i in range(min(archetype_count, len(card.fitness))):
            if card.fitness[i] > 0.5:
                per_archetype_coverage[i] += 1

    rarity_counts: dict[str, int] = {}
    power_per_rarity: dict[str, list[float]] = {}
    for card in cards:
        r = card.rarity or "unassigned"
        rarity_counts[r] = rarity_counts.get(r, 0) + 1
        power_per_rarity.setdefault(r, []).append(card.power)

    return PoolAnalysis(
        cards=cards,
        primary_count=primary_count,
        bridge_count=bridge_count,
        neutral_count=neutral_count,
        per_archetype_primaries=per_archetype_primaries,
        per_archetype_coverage=per_archetype_coverage,
        bridge_pairs=bridge_pairs,
        rarity_counts=rarity_counts,
        powers=[c.power for c in cards],
        power_per_rarity=power_per_rarity,
    )


# ---------------------------------------------------------------------------
# Ideal targets
# ---------------------------------------------------------------------------


def compute_ideal_targets(cfg: SimulatorConfig) -> IdealTargets:
    """Compute ideal pool composition from config (pure arithmetic).

    Coverage target: in the simulator, each primary covers 1 archetype and
    each bridge covers ~2.3 on average. Total archetype-slots =
    primary_count + bridge_count * 2.3, divided evenly across archetypes.
    """
    pool_size = cfg.cube.distinct_cards
    archetype_count = cfg.cards.archetype_count
    bridge_count = int(pool_size * cfg.cards.bridge_fraction)
    primary_count = pool_size - bridge_count
    primaries_per_archetype = primary_count // archetype_count

    # Coverage: each card with fitness > 0.5 in an archetype counts.
    # Bridges average ~2.3 archetypes, primaries 1, neutrals 0.
    avg_bridge_archetypes = 2.3
    total_coverage_slots = primary_count + bridge_count * avg_bridge_archetypes
    coverage_per_archetype = round(total_coverage_slots / archetype_count)

    per_tier_total: dict[str, int] = {}
    per_tier_bridge: dict[str, int] = {}
    per_tier_primary: dict[str, int] = {}
    per_tier_primaries_per_archetype: dict[str, int] = {}

    if cfg.rarity.enabled:
        for ti, tier_name in enumerate(cfg.rarity.tiers):
            tier_count = cfg.rarity.tier_design_counts[ti]
            tier_bridge = int(tier_count * cfg.cards.bridge_fraction)
            tier_primary = tier_count - tier_bridge
            per_tier_total[tier_name] = tier_count
            per_tier_bridge[tier_name] = tier_bridge
            per_tier_primary[tier_name] = tier_primary
            per_tier_primaries_per_archetype[tier_name] = (
                tier_primary // archetype_count
            )

    return IdealTargets(
        pool_size=pool_size,
        bridge_count=bridge_count,
        primary_count=primary_count,
        primaries_per_archetype=primaries_per_archetype,
        coverage_per_archetype=coverage_per_archetype,
        per_tier_total=per_tier_total,
        per_tier_bridge=per_tier_bridge,
        per_tier_primary=per_tier_primary,
        per_tier_primaries_per_archetype=per_tier_primaries_per_archetype,
    )


# ---------------------------------------------------------------------------
# Gap analysis
# ---------------------------------------------------------------------------


def compute_gaps(
    analysis: PoolAnalysis, targets: IdealTargets, cfg: SimulatorConfig
) -> GapAnalysis:
    """Compute what's missing between current pool and ideal.

    Uses coverage-based targets: scales the ideal coverage_per_archetype
    proportionally to the current pool size, so the gap reflects what's
    needed at the current scale, not only at the full 360-card target.
    """
    card_count = len(analysis.cards)
    remaining_total = max(0, targets.pool_size - card_count)
    archetype_count = cfg.cards.archetype_count

    # Coverage gap against full target — shows where to focus remaining cards
    per_archetype_coverage_needed = [
        max(0, targets.coverage_per_archetype - analysis.per_archetype_coverage[i])
        for i in range(archetype_count)
    ]

    # Bridge pair needs — only show uncovered pairs, don't enforce counts
    bridge_pairs_needed: dict[tuple[int, int], int] = {}
    for i in range(archetype_count):
        for j in range(i + 1, archetype_count):
            pair = (i, j)
            current = analysis.bridge_pairs.get(pair, 0)
            if current == 0:
                bridge_pairs_needed[pair] = 1

    # Per-rarity needs scaled to remaining slots
    per_rarity_needed: dict[str, int] = {}
    per_rarity_power_range: dict[str, tuple[float, float]] = {}
    if cfg.rarity.enabled and remaining_total > 0:
        for ti, tier_name in enumerate(cfg.rarity.tiers):
            tier_target = cfg.rarity.tier_design_counts[ti]
            current_tier = analysis.rarity_counts.get(tier_name, 0)
            per_rarity_needed[tier_name] = max(0, tier_target - current_tier)
            lo, hi = cfg.rarity.tier_power_ranges[ti]
            per_rarity_power_range[tier_name] = (lo, hi)

    return GapAnalysis(
        remaining_total=remaining_total,
        per_archetype_coverage_needed=per_archetype_coverage_needed,
        bridge_pairs_needed=bridge_pairs_needed,
        per_rarity_needed=per_rarity_needed,
        per_rarity_power_range=per_rarity_power_range,
    )


# ---------------------------------------------------------------------------
# Formatting helpers
# ---------------------------------------------------------------------------


def _mean(values: list[float]) -> float:
    if not values:
        return 0.0
    return sum(values) / len(values)


def _percentile(values: list[float], pct: float) -> float:
    if not values:
        return 0.0
    s = sorted(values)
    k = (pct / 100.0) * (len(s) - 1)
    lo = int(k)
    hi = min(lo + 1, len(s) - 1)
    frac = k - lo
    return s[lo] + frac * (s[hi] - s[lo])


def _histogram(
    values: list[float], indent: str, bins: int = 6, lo: float = 0.0, hi: float = 1.0
) -> str:
    width = (hi - lo) / bins
    counts = [0] * bins
    for v in values:
        idx = min(int((v - lo) / width), bins - 1)
        idx = max(0, idx)
        counts[idx] += 1
    max_count = max(counts) if counts else 1
    lines: list[str] = []
    for i, count in enumerate(counts):
        if count == 0:
            continue
        edge = lo + i * width
        bar_len = max(1, round(count / max_count * 20))
        bar = "\u2588" * bar_len
        lines.append(
            f"{indent}[{edge:.2f}-{edge + width:.2f}] "
            f"{colors.c(bar, 'accent')} {colors.num(count)}"
        )
    return "\n".join(lines)


def _bar_chart(label: str, value: int, max_value: int, width: int = 20) -> str:
    """Render a single horizontal bar chart row."""
    bar_len = max(1, round(value / max_value * width)) if max_value > 0 else 0
    bar = "\u2588" * bar_len
    return (
        f"  {colors.label(f'{label:<12s}')} "
        f"{colors.c(bar, 'accent')} {colors.num(value)}"
    )


# ---------------------------------------------------------------------------
# Output formatting
# ---------------------------------------------------------------------------


def format_analysis(
    analysis: PoolAnalysis,
    targets: IdealTargets,
    gaps: GapAnalysis,
    cfg: SimulatorConfig,
    source_path: str,
) -> str:
    """Format the full 3-section analysis report."""
    lines: list[str] = []
    archetype_count = cfg.cards.archetype_count
    card_count = len(analysis.cards)
    arch_names = ARCHETYPE_NAMES[:archetype_count]

    # ===== HEADER =====
    lines.append(colors.c("=" * 64, "ui"))
    lines.append(colors.header("POOL ANALYSIS — Real Cards vs Ideal Targets"))
    lines.append(colors.c("=" * 64, "ui"))
    lines.append(
        f"{colors.label('Source:')} {colors.filepath(source_path)}  |  "
        f"{colors.num(card_count)} cards loaded"
    )
    lines.append("")

    # ===== SECTION A: Current Pool Summary =====
    lines.append(colors.section("━━━ A. Current Pool Summary ━━━"))
    lines.append("")
    lines.append(f"  {colors.label('Total cards:')}  {colors.num(card_count)}")

    # Rarity distribution (inferred)
    if analysis.rarity_counts:
        lines.append(f"\n  {colors.label('Rarity Distribution (inferred):')}")
        for tier in cfg.rarity.tiers:
            count = analysis.rarity_counts.get(tier, 0)
            pct = 100 * count / card_count if card_count > 0 else 0
            lines.append(
                f"    {colors.label(f'{tier.title() + ':':<12s}')} "
                f"{colors.num(f'{count:>4d}')}  "
                f"({colors.num(f'{pct:.1f}')}%)"
            )

    # Composition
    lines.append(f"\n  {colors.label('Composition:')}")
    for role, count in [
        ("Primary", analysis.primary_count),
        ("Bridge", analysis.bridge_count),
        ("Neutral", analysis.neutral_count),
    ]:
        pct = 100 * count / card_count if card_count > 0 else 0
        lines.append(
            f"    {colors.label(f'{role + ':':<12s}')} "
            f"{colors.num(f'{count:>4d}')}  "
            f"({colors.num(f'{pct:.1f}')}%)"
        )

    # Per-archetype coverage (all cards with fitness > 0.5)
    lines.append(f"\n  {colors.label('Archetype coverage (fitness > 0.5):')}")
    for i, name in enumerate(arch_names):
        cov = analysis.per_archetype_coverage[i]
        pri = analysis.per_archetype_primaries[i]
        bridge_contrib = cov - pri
        lines.append(
            f"    {colors.label(f'{name.title() + ':':<12s}')} "
            f"{colors.num(f'{cov:>4d}')}  "
            f"{colors.dim(f'({pri} primary + {bridge_contrib} bridge)')}"
        )

    # Power distribution
    if analysis.powers:
        lines.append(f"\n  {colors.label('Power Distribution:')}")
        lines.append(
            f"    {colors.dim('min=')}{colors.num(f'{min(analysis.powers):.3f}')}  "
            f"{colors.dim('p25=')}{colors.num(f'{_percentile(analysis.powers, 25):.3f}')}  "
            f"{colors.dim('median=')}{colors.num(f'{_percentile(analysis.powers, 50):.3f}')}  "
            f"{colors.dim('p75=')}{colors.num(f'{_percentile(analysis.powers, 75):.3f}')}  "
            f"{colors.dim('max=')}{colors.num(f'{max(analysis.powers):.3f}')}"
        )
        lines.append(_histogram(analysis.powers, "    "))
    lines.append("")

    # ===== SECTION B: Comparison vs Ideal =====
    lines.append(colors.section("━━━ B. Comparison vs Ideal ━━━"))
    lines.append("")

    # Side-by-side table header
    lines.append(
        f"  {colors.label(f'{'Metric':<28s}')}  "
        f"{colors.label(f'{'Current':>8s}')}  "
        f"{colors.label(f'{'Target':>8s}')}  "
        f"{colors.label(f'{'Delta':>8s}')}"
    )
    lines.append(f"  {colors.dim('─' * 58)}")

    def _row(metric: str, current: int, target: int) -> str:
        delta = current - target
        delta_str = f"{delta:+d}"
        delta_color = "vcs_added" if delta >= 0 else "error"
        return (
            f"  {colors.dim(f'{metric:<28s}')}  "
            f"{colors.num(f'{current:>8d}')}  "
            f"{colors.num(f'{target:>8d}')}  "
            f"{colors.c(f'{delta_str:>8s}', delta_color)}"
        )

    lines.append(_row("Total cards", card_count, targets.pool_size))
    lines.append("")
    lines.append(
        f"  {colors.label('Composition:')}  "
        f"{colors.num(analysis.primary_count)} primary  "
        f"{colors.num(analysis.bridge_count)} bridge  "
        f"{colors.num(analysis.neutral_count)} neutral  "
        f"{colors.dim(f'(sim default: {cfg.cards.bridge_fraction:.0%} bridge)')}"
    )
    lines.append("")

    # Per-archetype coverage
    lines.append(f"  {colors.label('Per-archetype coverage (fitness > 0.5):')}")
    lines.append(
        f"  {colors.label(f'{'Archetype':<12s}')}  "
        f"{colors.label(f'{'Current':>8s}')}  "
        f"{colors.label(f'{'Target':>8s}')}  "
        f"{colors.label(f'{'Delta':>8s}')}"
    )
    lines.append(f"  {colors.dim('─' * 44)}")
    for i, name in enumerate(arch_names):
        lines.append(
            _row(
                f"  {name.title()}",
                analysis.per_archetype_coverage[i],
                targets.coverage_per_archetype,
            )
        )
    lines.append("")

    # Rarity comparison
    if targets.per_tier_total:
        lines.append(f"  {colors.label('Rarity distribution:')}")
        lines.append(
            f"  {colors.label(f'{'Tier':<12s}')}  "
            f"{colors.label(f'{'Current':>8s}')}  "
            f"{colors.label(f'{'Target':>8s}')}  "
            f"{colors.label(f'{'Delta':>8s}')}"
        )
        lines.append(f"  {colors.dim('─' * 44)}")
        for tier in cfg.rarity.tiers:
            current_tier = analysis.rarity_counts.get(tier, 0)
            target_tier = targets.per_tier_total.get(tier, 0)
            lines.append(_row(f"  {tier.title()}", current_tier, target_tier))
    lines.append("")

    # ===== SECTION C: Gap Analysis =====
    lines.append(colors.section("━━━ C. Gap Analysis ━━━"))
    lines.append("")

    lines.append(
        f"  {colors.label('Remaining cards needed:')} "
        f"{colors.num(gaps.remaining_total)}"
    )
    lines.append("")

    # Per-archetype coverage needs (sorted by need descending)
    lines.append(f"  {colors.label('Coverage needed per archetype:')}")
    arch_needs = sorted(
        [
            (arch_names[i], gaps.per_archetype_coverage_needed[i])
            for i in range(archetype_count)
        ],
        key=lambda x: -x[1],
    )
    max_need = max(n for _, n in arch_needs) if arch_needs else 1
    if max_need > 0:
        for name, need in arch_needs:
            lines.append(_bar_chart(name.title(), need, max_need))
    else:
        lines.append(f"    {colors.ok('All archetypes at or above coverage target')}")
    lines.append("")

    # Uncovered archetype pairs (no bridge card linking them)
    if gaps.bridge_pairs_needed:
        lines.append(f"  {colors.label('Uncovered archetype pairs (no bridge):')}")
        sorted_pairs = sorted(gaps.bridge_pairs_needed.items(), key=lambda x: x[0])
        for (a, b), _need in sorted_pairs:
            a_name = arch_names[a].title() if a < len(arch_names) else str(a)
            b_name = arch_names[b].title() if b < len(arch_names) else str(b)
            lines.append(f"    {colors.dim(f'{a_name} — {b_name}')}")
        lines.append("")

    # Rarity gap
    if gaps.per_rarity_needed:
        lines.append(f"  {colors.label('Remaining cards by rarity:')}")
        for tier in cfg.rarity.tiers:
            need = gaps.per_rarity_needed.get(tier, 0)
            lo, hi = gaps.per_rarity_power_range.get(tier, (0.0, 1.0))
            lines.append(
                f"    {colors.label(f'{tier.title() + ':':<12s}')} "
                f"{colors.num(f'{need:>4d}')} cards  "
                f"{colors.dim(f'(power {lo:.2f}-{hi:.2f})')}"
            )
    lines.append("")
    lines.append(colors.c("=" * 64, "ui"))

    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------


def run_analyze(cfg: SimulatorConfig, toml_path: str | None) -> None:
    """Run the full analysis pipeline and print results."""
    if toml_path is None:
        # Default path relative to the script location
        import os

        script_dir = os.path.dirname(os.path.abspath(__file__))
        toml_path = os.path.join(
            script_dir, "..", "..", "rules_engine", "tabula", "card-metadata.toml"
        )

    cards = load_cards_from_toml(toml_path)
    if not cards:
        print(f"{colors.fail('No cards found in')} {toml_path}", file=sys.stderr)
        return

    # Infer rarity based on power
    cards = infer_rarity(cards, cfg.rarity)

    analysis = analyze_pool(cards, cfg)
    targets = compute_ideal_targets(cfg)
    gaps = compute_gaps(analysis, targets, cfg)

    print(format_analysis(analysis, targets, gaps, cfg, toml_path))
