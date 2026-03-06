#!/usr/bin/env python3
"""Entry point for the draft simulator.

Parses CLI arguments, loads configuration, and dispatches to the selected
mode: single (default) runs one complete draft and prints per-seat results,
sweep runs batch parameter experiments, trace writes per-pick JSON output,
and demo runs the component demonstrations from earlier tasks. Stdlib-only,
no external dependencies.
"""

import argparse
import collections
import dataclasses
import json
import random
import sys
import traceback

import agents
import card_generator
import colors
import commitment
import config
import cube_manager
import deck_scorer
import draft_runner
import metrics
import output
import pack_generator
import pool_analyzer
import refill
import show_n
import sweep
import validation
from config import SimulatorConfig
from draft_models import CardDesign, CardInstance, CubeConsumptionMode, Pack
from utils import argmax

VERSION = "0.1.0"


def _resolve_copies_per_card(
    cards: list[CardDesign], cfg: SimulatorConfig
) -> int | dict[str, int]:
    """Return the copies_per_card value, using rarity map when enabled."""
    if cfg.rarity.enabled:
        return cube_manager.build_copies_map(cards, cfg.rarity)
    return cfg.cube.copies_per_card


def _unique_designs(card_pool: dict[int, CardInstance]) -> list[CardDesign]:
    """Extract unique CardDesign objects from a card pool."""
    seen: set[str] = set()
    designs: list[CardDesign] = []
    for inst in card_pool.values():
        if inst.design.card_id not in seen:
            seen.add(inst.design.card_id)
            designs.append(inst.design)
    return designs


def build_parser() -> argparse.ArgumentParser:
    """Build the argument parser for the draft simulator CLI."""
    parser = argparse.ArgumentParser(
        description="Draft simulator for Dreamtides.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "mode",
        nargs="?",
        default="single",
        choices=["single", "sweep", "trace", "demo", "explain", "analyze"],
        help="Simulation mode (default: single)",
    )
    parser.add_argument(
        "--seed",
        "-s",
        type=int,
        default=None,
        help="Base RNG seed (default: random)",
    )
    parser.add_argument(
        "--config",
        "-c",
        type=str,
        default=None,
        help="Path to TOML or JSON config file",
    )
    parser.add_argument(
        "--runs",
        type=int,
        default=None,
        help="Number of simulation runs (default: 1 for single, 1000 for sweep)",
    )
    parser.add_argument(
        "--output-dir",
        type=str,
        default="./draft_output/",
        help="Output directory (default: ./draft_output/)",
    )
    parser.add_argument(
        "--preset",
        type=str,
        default=None,
        choices=["easy", "hard"],
        help="Apply a difficulty preset",
    )
    parser.add_argument(
        "--param",
        action="append",
        default=None,
        help="Override a config parameter (repeatable, KEY=VALUE)",
    )
    parser.add_argument(
        "--describe-pool",
        action="store_true",
        default=False,
        help="Print a summary of the card pool before draft results",
    )
    parser.add_argument(
        "--quiet",
        "-q",
        action="store_true",
        default=False,
        help="Suppress incremental progress bars",
    )
    parser.add_argument(
        "--toml-path",
        type=str,
        default=None,
        help="Path to card-metadata TOML file (for analyze mode)",
    )
    return parser


def main() -> None:
    """Run the draft simulator."""
    parser = build_parser()
    args = parser.parse_args()

    # Load configuration
    cfg = config.load_config(
        config_path=args.config,
        overrides=args.param,
    )

    # Resolve seed: use CLI flag if provided, otherwise random
    seed: int = args.seed if args.seed is not None else random.randint(0, 2**32 - 1)
    cfg.sweep.base_seed = seed

    # Apply difficulty preset (overrides config values)
    if args.preset == "easy":
        cfg.agents.ai_optimality = 0.4
        cfg.agents.ai_signal_weight = 0.0
        cfg.draft.seat_count = 5
    elif args.preset == "hard":
        cfg.agents.ai_optimality = 0.9
        cfg.agents.ai_signal_weight = 0.8
        cfg.draft.seat_count = 8

    mode: str = args.mode
    output_dir: str = args.output_dir

    if mode == "single":
        runs = args.runs if args.runs is not None else 1
        _run_single(
            cfg,
            seed,
            describe_pool=args.describe_pool,
            runs=runs,
            preset=args.preset,
            quiet=args.quiet,
        )
    elif mode == "trace":
        _run_trace(cfg, seed, output_dir)
    elif mode == "demo":
        _run_demo(cfg, seed)
    elif mode == "explain":
        _run_explain()
    elif mode == "analyze":
        pool_analyzer.run_analyze(cfg, args.toml_path)
    elif mode == "sweep":
        runs = args.runs if args.runs is not None else cfg.sweep.runs_per_point
        _run_sweep(cfg, seed, runs, output_dir)


def _print_header(
    cfg: SimulatorConfig,
    seed: int,
    runs: int = 1,
    preset: str | None = None,
) -> None:
    """Print the richer header with configuration info."""
    line1 = (
        f"{colors.header(f'Draft Simulator v{VERSION}')} | "
        f"{colors.label('seed')}={colors.num(seed)}"
    )
    if runs > 1:
        line1 += f" | {colors.label('runs')}={colors.num(runs)}"
    print(line1)
    print(
        f"  {colors.label('seats')}={colors.num(cfg.draft.seat_count)}, "
        f"{colors.label('rounds')}={colors.num(cfg.draft.round_count)}, "
        f"{colors.label('pack_size')}={colors.num(cfg.draft.pack_size)}, "
        f"{colors.label('show_n')}={colors.num(cfg.agents.show_n)}"
    )
    print(
        f"  {colors.label('policy')}={colors.c(cfg.agents.policy, 'special')}, "
        f"{colors.label('show_n_strategy')}="
        f"{colors.c(cfg.agents.show_n_strategy, 'special')}, "
        f"{colors.label('ai_optimality')}={colors.num(f'{cfg.agents.ai_optimality:.2f}')}"
    )
    preset_str = preset if preset is not None else "none"
    print(
        f"  {colors.label('preset')}={colors.c(preset_str, 'special')}, "
        f"{colors.label('refill')}={colors.c(cfg.refill.strategy, 'special')}, "
        f"{colors.label('archetypes')}={colors.num(cfg.cards.archetype_count)}"
    )


def _run_explain() -> None:
    """Print plain-language explanations of all parameters and goal metrics."""
    print(colors.header(f"Draft Simulator v{VERSION} — Parameter & Metric Reference"))

    print(f"\n{colors.section('=== Configuration Parameters ===')}")

    _explain_section(
        "draft",
        "Draft Structure",
        [
            (
                "seat_count",
                "Number of players in the draft. More seats means more "
                "competition for cards.",
            ),
            (
                "round_count",
                "Number of rounds. Each round generates a fresh pack " "per seat.",
            ),
            (
                "picks_per_round",
                "Cards picked per round (list, one entry per "
                "round). Must sum to 30.",
            ),
            ("pack_size", "Number of cards in each newly generated pack."),
            (
                "alternate_direction",
                "Whether pack-passing direction alternates "
                "between rounds (left, right, left...).",
            ),
            (
                "human_seats",
                "Number of seats modeled as human (with show-N "
                "filtering). Usually 1.",
            ),
        ],
    )

    _explain_section(
        "cube",
        "Cube Construction",
        [
            ("distinct_cards", "Total number of unique card designs in the cube."),
            ("copies_per_card", "How many copies of each design exist in the " "cube."),
            (
                "consumption_mode",
                '"without_replacement" removes drawn cards from '
                'the cube; "with_replacement" allows redrawing.',
            ),
        ],
    )

    _explain_section(
        "pack_generation",
        "Pack Generation",
        [
            (
                "strategy",
                'Algorithm for building packs: "uniform" (random), '
                '"rarity_weighted", or "seeded_themed" (themed around specific '
                "archetypes).",
            ),
            (
                "archetype_target_count",
                "For seeded_themed: number of archetypes " "to emphasize per pack.",
            ),
            (
                "primary_density",
                "For seeded_themed: fraction of pack filled with "
                "primary-archetype cards.",
            ),
            (
                "bridge_density",
                "For seeded_themed: fraction of pack filled with "
                "bridge cards (cards that span two+ archetypes).",
            ),
            (
                "variance",
                "For seeded_themed: randomness in archetype emphasis " "across packs.",
            ),
        ],
    )

    _explain_section(
        "refill",
        "Pack Refill",
        [
            (
                "strategy",
                "How packs are replenished after each pick: "
                '"no_refill", "uniform_refill" (random card added), or '
                '"constrained_refill" (signal-preserving card added).',
            ),
            (
                "fingerprint_source",
                "Signal source for constrained refill: "
                '"pack_origin" uses the pack\'s own archetype profile; '
                '"round_environment" aggregates across all packs in the round.',
            ),
            (
                "fidelity",
                "How closely a constrained refill card matches the "
                "signal. 0 = random, 1 = perfect match.",
            ),
            (
                "commit_bias",
                "Bias toward high-commit (archetype-defining) cards "
                "in constrained refill selection.",
            ),
        ],
    )

    _explain_section(
        "cards",
        "Card Pool",
        [
            (
                "source",
                'Where card designs come from: "synthetic" (procedurally '
                'generated) or "file" (loaded from a data file).',
            ),
            ("file_path", 'Path to card data file when source="file".'),
            (
                "archetype_count",
                "Number of distinct draft archetypes in the card " "pool.",
            ),
            (
                "bridge_fraction",
                "Fraction of the card pool that bridges two or "
                "more archetypes (synthetic mode only).",
            ),
        ],
    )

    _explain_section(
        "agents",
        "Agent Behavior",
        [
            (
                "policy",
                'Card selection strategy for AI seats: "adaptive" '
                '(balances power, preference, and signals), "greedy" (maximizes '
                'deck value), "archetype_loyal" (follows best archetype), '
                '"force" (locks onto one archetype), or "signal_ignorant" '
                "(adaptive without signal reading).",
            ),
            ("show_n", "Number of cards revealed to the human seat per pick."),
            (
                "show_n_strategy",
                'How the N shown cards are selected: "uniform" '
                '(random), "power_biased" (favors strong cards), "curated" '
                '(guarantees on-plan + off-plan strong), "signal_rich" (favors '
                'archetype-defining cards), or "sharpened_preference" (heavily '
                "weights preference alignment).",
            ),
            (
                "ai_optimality",
                "Probability of making the best pick. 0 = fully "
                "random, 1 = always optimal. Controls AI difficulty.",
            ),
            (
                "ai_signal_weight",
                "Weight of supply-signal reading in the adaptive "
                "scoring formula. 0 = ignores signals, higher = reads signals more.",
            ),
            ("ai_power_weight", "Weight of raw card power in adaptive scoring."),
            (
                "ai_pref_weight",
                "Weight of archetype-preference alignment in " "adaptive scoring.",
            ),
            (
                "openness_window",
                "Number of recent packs used to estimate which "
                "archetypes are open (underdrafted).",
            ),
            (
                "learning_rate",
                "How quickly the preference vector updates after "
                "each pick. Higher = faster archetype commitment.",
            ),
            (
                "force_archetype",
                "Which archetype index to force when "
                'policy="force". Required for that policy.',
            ),
        ],
    )

    _explain_section(
        "scoring",
        "Deck Value Scoring",
        [
            ("weight_power", "Weight of raw card power in the deck value " "formula."),
            (
                "weight_coherence",
                "Weight of archetype coherence (how well cards "
                "fit the chosen archetype) in deck value.",
            ),
            (
                "weight_focus",
                "Weight of focus bonus (reward for high on-plan "
                "density) in deck value.",
            ),
            (
                "secondary_weight",
                "How much the second-best archetype contributes "
                "to coherence scoring.",
            ),
            (
                "focus_threshold",
                "Minimum fraction of on-plan cards needed to "
                "start earning focus bonus.",
            ),
            (
                "focus_saturation",
                "Fraction of on-plan cards at which focus bonus "
                "reaches its maximum.",
            ),
        ],
    )

    _explain_section(
        "commitment",
        "Commitment Detection",
        [
            (
                "commitment_threshold",
                "Minimum concentration ratio "
                "(max(w)/sum(w)) to trigger archetype commitment.",
            ),
            (
                "stability_window",
                "Number of consecutive picks that must favor "
                "the same archetype to confirm commitment.",
            ),
            (
                "entropy_threshold",
                "Shannon entropy threshold (in bits) for the "
                "secondary entropy-based commitment detector.",
            ),
        ],
    )

    _explain_section(
        "metrics",
        "Metrics Engine",
        [
            (
                "richness_gap",
                "Score tolerance for counting near-optimal cards. "
                'Cards scoring within this gap of the best are "near-optimal".',
            ),
            (
                "tau",
                "Softmax temperature for choice entropy. Lower = sharper "
                "probability distribution, more sensitive to score differences.",
            ),
            (
                "on_plan_threshold",
                "Minimum fitness for a card to count as "
                '"on-plan" when measuring convergence.',
            ),
            (
                "splash_power_threshold",
                "Minimum power for an off-plan card to " "be considered splashable.",
            ),
            (
                "splash_flex_threshold",
                "Minimum flex score for an off-plan card "
                "to be considered splashable.",
            ),
            (
                "exposure_threshold",
                "Minimum fitness for a card to count as "
                '"exposing" an archetype in early openness measurement.',
            ),
        ],
    )

    _explain_section(
        "sweep",
        "Sweep Execution",
        [
            (
                "runs_per_point",
                "Number of draft simulations per parameter "
                "combination in sweep mode.",
            ),
            ("base_seed", "Starting RNG seed for reproducible runs."),
            (
                "seeding_policy",
                "How seeds are assigned across runs: "
                '"sequential" means seed = base_seed + run_index.',
            ),
            (
                "trace_enabled",
                "Whether to collect per-pick trace data during " "sweep runs.",
            ),
            (
                "axes",
                "Dict of parameter paths to sweep over. Each key is a "
                "dot-notation parameter, each value is a list of values. Sweep "
                "runs the Cartesian product of all axes.",
            ),
        ],
    )

    print(f"\n{colors.section('=== Goal Metrics ===')}")
    print()

    _explain_metric(
        "Convergence (mid)",
        "Mean number of on-plan cards shown to the human per pick during the "
        "mid phase (picks 6-19), measured only after the human commits to an "
        "archetype. Answers: does the draft keep delivering relevant cards "
        "mid-draft?",
        ">= 2.0",
    )
    _explain_metric(
        "Convergence (late)",
        "Same as convergence (mid) but for the late phase (picks 20-29). "
        "Ensures archetype supply holds up through the end of the draft.",
        ">= 2.0",
    )
    _explain_metric(
        "Choice richness",
        "Mean count of near-optimal cards per pick across all picks (shown-N "
        "surface). A near-optimal card scores within 'richness_gap' of the "
        "best. Answers: does the drafter face meaningful decisions, or is "
        "each pick obvious?",
        ">= 1.5",
    )
    _explain_metric(
        "Forceability",
        "Ratio of forced-archetype deck value to adaptive deck value, taking "
        "the worst (highest) archetype. A value near 1.0 means forcing one "
        "archetype is as good as drafting adaptively. Requires multi-run "
        "data (--runs N).",
        "< 0.95",
    )
    _explain_metric(
        "Splashability",
        "Fraction of post-commitment picks that offer at least one viable "
        "off-plan card (high power or flex, low on-plan fitness). Answers: "
        "after committing, can the drafter still pick up generically strong "
        "cards?",
        ">= 0.40",
    )
    _explain_metric(
        "Early openness",
        "Number of distinct archetypes exposed in the first 5 picks via shown "
        "cards with fitness above the exposure threshold. Answers: do early "
        "packs let the drafter explore multiple paths before committing?",
        ">= 5.0",
    )
    _explain_metric(
        "Signal benefit",
        "Percentage improvement in deck value from reading supply signals "
        "(adaptive policy vs signal-ignorant policy, averaged across runs). "
        "Answers: is paying attention to what's being passed rewarded? "
        "Requires multi-run data (--runs N).",
        ">= 2%",
    )


def _explain_section(
    section_key: str,
    section_title: str,
    params: list[tuple[str, str]],
) -> None:
    """Print parameter explanations for one config section."""
    defaults = SimulatorConfig()
    section_obj = getattr(defaults, section_key)
    print(f"\n{colors.section(f'--- {section_title} ({section_key}.*) ---')}")
    for param_name, description in params:
        default_val = getattr(section_obj, param_name)
        print(
            f"  {colors.label(f'{section_key}.{param_name}')} "
            f"{colors.dim(f'(default: {default_val})')}\n"
            f"    {description}"
        )


def _explain_metric(name: str, description: str, target: str) -> None:
    """Print a goal metric explanation."""
    print(
        f"  {colors.label(name)}\n"
        f"    {description}\n"
        f"    {colors.dim(f'Target: {target}')}"
    )
    print()


def _run_single(
    cfg: SimulatorConfig,
    seed: int,
    describe_pool: bool = False,
    runs: int = 1,
    preset: str | None = None,
    quiet: bool = False,
) -> None:
    """Run one or more drafts and print per-seat results with metrics."""
    _print_header(cfg, seed, runs, preset)

    if runs == 1:
        _run_single_once(cfg, seed, describe_pool)
    else:
        _run_single_multi(cfg, seed, runs, describe_pool, quiet)


def _run_single_once(
    cfg: SimulatorConfig,
    seed: int,
    describe_pool: bool,
) -> None:
    """Run a single draft and print results (original behavior)."""
    result = draft_runner.run_draft(cfg, seed)

    if describe_pool:
        designs = _unique_designs(result.card_pool)
        source_label = (
            f"file ({cfg.cards.file_path})"
            if cfg.cards.source == "file"
            else "synthetic"
        )
        print()
        print(
            card_generator.describe_card_pool(
                designs, cfg.cards.archetype_count, source_label
            )
        )

    print()
    for seat_idx, sr in enumerate(result.seat_results):
        is_human = seat_idx < cfg.draft.human_seats
        seat_type = "human" if is_human else "ai"
        policy = cfg.agents.policy

        if sr.commitment_pick is not None:
            commit_str = f"pick {colors.num(sr.commitment_pick)}"
        else:
            commit_str = colors.dim("uncommitted")

        archetype = (
            sr.committed_archetype
            if sr.committed_archetype is not None
            else argmax(sr.final_w)
        )

        seat_type_str = (
            colors.c(seat_type, "tag") if is_human else colors.dim(seat_type)
        )
        label = f"Seat {colors.num(seat_idx)} ({seat_type_str}, {policy}):"
        print(
            f"{label:<35s} {colors.label('deck_value')}={colors.num(f'{sr.deck_value:.3f}')}, "
            f"{colors.label('archetype')}={colors.c(archetype, 'operator')}, "
            f"{colors.label('committed')}={commit_str}"
        )

    draft_metrics = metrics.compute_metrics(result, cfg)
    print()
    print(metrics.format_goal_metrics(draft_metrics))
    print()
    print(metrics.format_metrics(draft_metrics))


def _run_single_multi(
    cfg: SimulatorConfig,
    base_seed: int,
    runs: int,
    describe_pool: bool,
    quiet: bool = False,
) -> None:
    """Run multiple drafts and print averaged results."""
    seat_count = cfg.draft.seat_count
    all_metrics: list[metrics.DraftMetrics] = []
    seat_deck_values: list[list[float]] = [[] for _ in range(seat_count)]
    seat_archetypes: list[list[int | None]] = [[] for _ in range(seat_count)]
    seat_commitment_picks: list[list[int | None]] = [[] for _ in range(seat_count)]
    adaptive_all_dvs: list[float] = []
    described = False

    for run_i in range(runs):
        run_seed = base_seed + run_i
        result = draft_runner.run_draft(cfg, run_seed)

        if describe_pool and not described:
            designs = _unique_designs(result.card_pool)
            source_label = (
                f"file ({cfg.cards.file_path})"
                if cfg.cards.source == "file"
                else "synthetic"
            )
            print()
            print(
                card_generator.describe_card_pool(
                    designs, cfg.cards.archetype_count, source_label
                )
            )
            described = True

        adaptive_all_dvs.append(result.seat_results[0].deck_value)
        for seat_idx, sr in enumerate(result.seat_results):
            seat_deck_values[seat_idx].append(sr.deck_value)
            seat_archetypes[seat_idx].append(sr.committed_archetype)
            seat_commitment_picks[seat_idx].append(sr.commitment_pick)

        all_metrics.append(metrics.compute_metrics(result, cfg))

        if not quiet:
            done = run_i + 1
            print(
                colors.format_progress_bar(done, runs, use_color=sys.stderr.isatty()),
                end="",
                file=sys.stderr,
            )

    if not quiet:
        print(file=sys.stderr)  # newline after progress bar

    # Run comparison drafts for forceability and signal benefit
    (
        signal_benefit_val,
        forceability_val,
        forceability_arch,
        forceability_per_arch,
        per_run_signal_benefits,
        per_run_forceabilities,
    ) = _run_comparison_drafts(cfg, base_seed, runs, adaptive_all_dvs, quiet)

    # Per-seat summary
    print()
    for seat_idx in range(seat_count):
        is_human = seat_idx < cfg.draft.human_seats
        seat_type = "human" if is_human else "ai"
        policy = cfg.agents.policy

        mean_dv = sum(seat_deck_values[seat_idx]) / runs

        # Most common committed archetype
        arch_counts: collections.Counter[int | None] = collections.Counter(
            seat_archetypes[seat_idx]
        )
        most_common_arch = arch_counts.most_common(1)[0][0]
        uncommitted_count = arch_counts.get(None, 0)

        if uncommitted_count > runs / 2:
            commit_str = colors.dim("uncommitted")
        else:
            valid_picks = [p for p in seat_commitment_picks[seat_idx] if p is not None]
            mean_pick = sum(valid_picks) / len(valid_picks) if valid_picks else 0
            commit_str = f"pick {colors.num(f'{mean_pick:.1f}')}"

        archetype = most_common_arch if most_common_arch is not None else "?"

        seat_type_str = (
            colors.c(seat_type, "tag") if is_human else colors.dim(seat_type)
        )
        seat_label = f"Seat {colors.num(seat_idx)} ({seat_type_str}, {policy}):"
        print(
            f"{seat_label:<35s} {colors.label('deck_value')}="
            f"{colors.num(f'{mean_dv:.3f}')}, "
            f"{colors.label('archetype')}={colors.c(archetype, 'operator')}, "
            f"{colors.label('committed')}={commit_str}"
        )

    averaged = metrics.average_metrics(all_metrics)
    averaged = dataclasses.replace(
        averaged,
        forceability=forceability_val,
        forceability_archetype=forceability_arch,
        forceability_per_archetype=forceability_per_arch,
        signal_benefit=signal_benefit_val,
    )
    cis = metrics.compute_goal_cis(
        all_metrics, per_run_signal_benefits, per_run_forceabilities
    )
    print()
    print(metrics.format_goal_metrics(averaged, cis=cis))
    print()
    print(metrics.format_metrics(averaged))


def _run_comparison_drafts(
    cfg: SimulatorConfig,
    base_seed: int,
    runs: int,
    adaptive_all_dvs: list[float],
    quiet: bool = False,
) -> tuple[
    float | None,
    float | None,
    int | None,
    dict[int, float] | None,
    list[float],
    list[float],
]:
    """Run signal-ignorant and force-policy drafts for cross-run metrics.

    Returns (signal_benefit, forceability_max, forceability_archetype,
    forceability_per_archetype, per_run_signal_benefits,
    per_run_forceabilities).
    """
    archetype_count = cfg.cards.archetype_count
    comparison_total = (1 + archetype_count) * runs
    comparison_done = 0

    ignorant_all_dvs: list[float] = []
    force_all_dvs: dict[int, list[float]] = {a: [] for a in range(archetype_count)}
    force_per_run: dict[int, list[float]] = {a: [] for a in range(archetype_count)}

    for run_i in range(runs):
        run_seed = base_seed + run_i

        # Signal-ignorant comparison: only human seat switches policy
        ignorant_result = draft_runner.run_draft(
            cfg, run_seed, human_seat_policy="signal_ignorant"
        )
        ignorant_all_dvs.append(ignorant_result.seat_results[0].deck_value)
        comparison_done += 1
        if not quiet:
            print(
                colors.format_progress_bar(
                    comparison_done,
                    comparison_total,
                    use_color=sys.stderr.isatty(),
                    label="comparison runs",
                ),
                end="",
                file=sys.stderr,
            )

        # Force comparisons per archetype
        for arch in range(archetype_count):
            force_cfg = config.clone_config(cfg)
            force_cfg.agents.policy = "force"
            force_cfg.agents.force_archetype = arch
            force_result = draft_runner.run_draft(force_cfg, run_seed)
            seat_dvs = [sr.deck_value for sr in force_result.seat_results]
            force_all_dvs[arch].extend(seat_dvs)
            force_per_run[arch].append(
                sum(seat_dvs) / len(seat_dvs) if seat_dvs else 0.0
            )
            comparison_done += 1
            if not quiet:
                print(
                    colors.format_progress_bar(
                        comparison_done,
                        comparison_total,
                        use_color=sys.stderr.isatty(),
                        label="comparison runs",
                    ),
                    end="",
                    file=sys.stderr,
                )

    if not quiet:
        print(file=sys.stderr)  # newline after comparison progress

    mean_adaptive = (
        sum(adaptive_all_dvs) / len(adaptive_all_dvs) if adaptive_all_dvs else 0.0
    )
    mean_ignorant = (
        sum(ignorant_all_dvs) / len(ignorant_all_dvs) if ignorant_all_dvs else 0.0
    )

    # Signal benefit
    signal_benefit_val: float | None = None
    if mean_ignorant > 0:
        signal_benefit_val = ((mean_adaptive - mean_ignorant) / mean_ignorant) * 100.0

    # Forceability
    forceability_val: float | None = None
    forceability_arch: int | None = None
    forceability_per_arch: dict[int, float] | None = None
    if mean_adaptive > 0:
        per_arch: dict[int, float] = {}
        max_fi = 0.0
        max_arch = 0
        for arch in range(archetype_count):
            dvs = force_all_dvs[arch]
            fi = (sum(dvs) / len(dvs)) / mean_adaptive if dvs else 0.0
            per_arch[arch] = fi
            if fi > max_fi:
                max_fi = fi
                max_arch = arch
        forceability_val = max_fi
        forceability_arch = max_arch
        forceability_per_arch = per_arch

    # Per-run signal benefit for CI computation
    per_run_signal_benefits: list[float] = []
    for run_i in range(runs):
        if ignorant_all_dvs[run_i] > 0:
            sb = (
                (adaptive_all_dvs[run_i] - ignorant_all_dvs[run_i])
                / ignorant_all_dvs[run_i]
            ) * 100.0
            per_run_signal_benefits.append(sb)

    # Per-run forceability for CI computation
    per_run_forceabilities: list[float] = []
    for run_i in range(runs):
        if adaptive_all_dvs[run_i] > 0:
            max_fi = max(
                force_per_run[arch][run_i] / adaptive_all_dvs[run_i]
                for arch in range(archetype_count)
            )
            per_run_forceabilities.append(max_fi)

    return (
        signal_benefit_val,
        forceability_val,
        forceability_arch,
        forceability_per_arch,
        per_run_signal_benefits,
        per_run_forceabilities,
    )


def _run_trace(
    cfg: SimulatorConfig,
    seed: int,
    output_dir: str,
) -> None:
    """Run a single draft with per-pick trace output to JSON file."""
    output.ensure_output_dir(output_dir)

    result = draft_runner.run_draft(cfg, seed, trace_enabled=True)

    trace_path = output.write_trace_json(output_dir, result.traces, seed)
    print(f"{colors.label('Per-pick trace written to:')} {colors.filepath(trace_path)}")


def _run_sweep(
    cfg: SimulatorConfig,
    base_seed: int,
    runs_per_point: int,
    output_dir: str,
) -> None:
    """Run a parameter sweep experiment with output serialization."""
    output.ensure_output_dir(output_dir)

    run_records, aggregate_records = sweep.run_sweep(
        cfg, base_seed, runs_per_point, output_dir
    )

    run_path = output.write_run_level_csv(output_dir, run_records)
    agg_path = output.write_aggregate_csv(output_dir, aggregate_records)
    meta_path = output.write_config_metadata(output_dir, cfg)

    print(f"\n{colors.section('Results written to:')}")
    print(
        f"  {colors.filepath(f'{run_path:<40s}')} ({colors.num(len(run_records))} rows)"
    )
    print(
        f"  {colors.filepath(f'{agg_path:<40s}')} "
        f"({colors.num(len(aggregate_records))} rows)"
    )
    print(f"  {colors.filepath(meta_path)}")

    report = validation.run_validation(aggregate_records, run_records)
    print(validation.format_validation_report(report))


def _run_demo(cfg: SimulatorConfig, seed: int) -> None:
    """Run the component demonstration mode (legacy)."""
    rng = random.Random(seed)

    print(
        f"{colors.header(f'Draft Simulator v{VERSION}')} | "
        f"{colors.label('seed')}={colors.num(seed)} "
        f"| {colors.label('seats')}={colors.num(cfg.draft.seat_count)} "
        f"| {colors.label('rounds')}={colors.num(cfg.draft.round_count)} "
        f"| {colors.label('pack_size')}={colors.num(cfg.draft.pack_size)}"
    )

    cards = card_generator.generate_cards(cfg, rng)

    source_label = (
        f"file ({cfg.cards.file_path})" if cfg.cards.source == "file" else "synthetic"
    )
    print(f"\n{colors.label('Card source:')} {colors.c(source_label, 'special')}")

    card_generator.print_card_pool_stats(cards, cfg.cards.archetype_count)

    consumption_mode = (
        CubeConsumptionMode.WITH_REPLACEMENT
        if cfg.cube.consumption_mode == "with_replacement"
        else CubeConsumptionMode.WITHOUT_REPLACEMENT
    )
    copies = _resolve_copies_per_card(cards, cfg)
    cube = cube_manager.CubeManager(
        designs=cards,
        copies_per_card=copies,
        consumption_mode=consumption_mode,
    )

    cube_manager.validate_supply(cfg, cube.total_size)

    strategies = ["uniform", "rarity_weighted", "seeded_themed"]
    strategy_labels = {
        "uniform": "Uniform",
        "rarity_weighted": "Rarity-weighted",
        "seeded_themed": "Seeded/Themed",
    }

    for strategy in strategies:
        demo_cube = cube_manager.CubeManager(
            designs=cards,
            copies_per_card=copies,
            consumption_mode=consumption_mode,
        )
        pack = pack_generator.generate_pack(strategy, demo_cube, cfg, rng)
        _print_pack(pack, strategy_labels[strategy], cfg.cards.archetype_count)

    _demo_deck_scoring(cards, cfg, rng)
    _demo_commitment_detection(cards, cfg, rng)
    _demo_refill_strategies(cards, cfg, consumption_mode, rng)
    _demo_pick_policies(cards, cfg, consumption_mode, rng)
    _demo_show_n_strategies(cards, cfg, consumption_mode, rng)


def _print_pack(pack: Pack, pack_label: str, archetype_count: int) -> None:
    """Print pack contents for QA inspection."""
    print(
        f"\n{colors.section(f'--- {pack_label} Pack')} "
        f"({colors.label('id')}={colors.num(pack.pack_id)}) ---"
    )
    print(f"  {colors.label('Cards')} ({colors.num(len(pack.cards))}):")
    for card_inst in pack.cards:
        top_arch = 0
        top_val = card_inst.design.fitness[0] if card_inst.design.fitness else 0.0
        for i in range(1, min(len(card_inst.design.fitness), archetype_count)):
            if card_inst.design.fitness[i] > top_val:
                top_val = card_inst.design.fitness[i]
                top_arch = i
        print(
            f"    {colors.card(f'{card_inst.design.name:<30s}')} "
            f"{colors.label('top_arch')}={colors.c(top_arch, 'operator')}  "
            f"{colors.label('power')}={colors.num(f'{card_inst.design.power:.3f}')}"
        )
    profile_str = ", ".join(f"{colors.num(f'{v:.3f}')}" for v in pack.archetype_profile)
    print(f"  {colors.label('Archetype profile:')} [{profile_str}]")


def _demo_deck_scoring(
    cards: list[CardDesign],
    cfg: SimulatorConfig,
    rng: random.Random,
) -> None:
    """Generate a synthetic pool, build a preference vector, and print scoring."""
    pool_size = 30
    pool = rng.sample(cards, min(pool_size, len(cards)))

    w = commitment.initial_preference_vector(cfg.cards.archetype_count)
    for card in pool:
        w = commitment.update_preference_vector(
            w, card.fitness, cfg.agents.learning_rate
        )

    raw, coherence, focus, final = deck_scorer.deck_value_breakdown(
        pool, w, cfg.scoring
    )

    print(f"\n{colors.section('--- Deck Value Scoring ---')}")
    print(f"  {colors.label('Pool size:')} {colors.num(len(pool))}")
    print(f"  {colors.label('Effective archetype:')} {colors.c(argmax(w), 'operator')}")
    print(f"  {colors.label('Raw power:')}            {colors.num(f'{raw:.4f}')}")
    print(f"  {colors.label('Archetype coherence:')}  {colors.num(f'{coherence:.4f}')}")
    print(f"  {colors.label('Focus bonus:')}          {colors.num(f'{focus:.4f}')}")
    print(f"  {colors.label('Final score:')}          {colors.num(f'{final:.4f}')}")

    weighted = (
        cfg.scoring.weight_power * raw
        + cfg.scoring.weight_coherence * coherence
        + cfg.scoring.weight_focus * focus
    )
    print(
        f"  {colors.label('Weighted sum (before clamp):')} "
        f"{colors.num(f'{weighted:.4f}')} "
        f"({colors.label('weights:')} "
        f"{colors.num(cfg.scoring.weight_power)}/"
        f"{colors.num(cfg.scoring.weight_coherence)}/"
        f"{colors.num(cfg.scoring.weight_focus)})"
    )


def _demo_commitment_detection(
    cards: list[CardDesign],
    cfg: SimulatorConfig,
    rng: random.Random,
) -> None:
    """Simulate a sequence of picks and run commitment detection."""
    pool_size = 30

    w = commitment.initial_preference_vector(cfg.cards.archetype_count)
    w_history: list[list[float]] = []
    available = list(cards)
    picked: list[CardDesign] = []

    for _ in range(pool_size):
        if not available:
            break
        best_arch = argmax(w)
        scored = [(c.fitness[best_arch] + 0.3 * c.power, c) for c in available]
        scored.sort(key=lambda x: x[0], reverse=True)
        top_k = min(5, len(scored))
        pick_idx = rng.randint(0, top_k - 1)
        card = scored[pick_idx][1]
        picked.append(card)
        available.remove(card)
        w = commitment.update_preference_vector(
            w, card.fitness, cfg.agents.learning_rate
        )
        w_history.append(list(w))

    result = commitment.detect_commitment(w_history, cfg.commitment)

    print(f"\n{colors.section('--- Commitment Detection ---')}")
    print(f"  {colors.label('Picks simulated:')} {colors.num(len(picked))}")
    print(
        f"  {colors.label('Threshold:')} "
        f"{colors.num(cfg.commitment.commitment_threshold)}"
    )
    print(
        f"  {colors.label('Stability window:')} "
        f"{colors.num(cfg.commitment.stability_window)}"
    )

    print(f"\n  {colors.label('Primary (concentration-based):')}")
    if result.commitment_pick is not None:
        print(
            f"    {colors.label('Commitment pick:')} "
            f"{colors.num(result.commitment_pick)}"
        )
        print(
            f"    {colors.label('Committed archetype:')} "
            f"{colors.c(result.committed_archetype, 'operator')}"
        )
        w_at_pick = w_history[result.commitment_pick]
        print(
            f"    {colors.label('Concentration at pick:')} "
            f"{colors.num(f'{commitment.concentration(w_at_pick):.4f}')}"
        )
    else:
        print(
            f"    {colors.label('Commitment pick:')} {colors.dim('None (uncommitted)')}"
        )

    print(f"\n  {colors.label('Secondary (entropy-based):')}")
    print(
        f"    {colors.label('Entropy threshold:')} "
        f"{colors.num(f'{cfg.commitment.entropy_threshold} bits')}"
    )
    if result.entropy_commitment_pick is not None:
        print(
            f"    {colors.label('Entropy commitment pick:')} "
            f"{colors.num(result.entropy_commitment_pick)}"
        )
        print(
            f"    {colors.label('Entropy committed archetype:')} "
            f"{colors.c(result.entropy_committed_archetype, 'operator')}"
        )
        w_at_pick = w_history[result.entropy_commitment_pick]
        print(
            f"    {colors.label('Entropy at pick:')} "
            f"{colors.num(f'{commitment.shannon_entropy(w_at_pick):.4f} bits')}"
        )
    else:
        print(
            f"    {colors.label('Entropy commitment pick:')} "
            f"{colors.dim('None (uncommitted)')}"
        )

    print(f"\n  {colors.label('Uniform w test (should be uncommitted):')}")
    uniform_history = [
        commitment.initial_preference_vector(cfg.cards.archetype_count)
        for _ in range(pool_size)
    ]
    uniform_result = commitment.detect_commitment(uniform_history, cfg.commitment)
    print(
        f"    {colors.label('Commitment pick:')} "
        f"{colors.num(uniform_result.commitment_pick)}"
    )
    print(
        f"    {colors.label('Entropy commitment pick:')} "
        f"{colors.num(uniform_result.entropy_commitment_pick)}"
    )


def _print_pack_composition(comp_label: str, pack: Pack) -> None:
    """Print the card names in a pack for QA inspection."""
    names = [colors.card(inst.design.name) for inst in pack.cards]
    print(
        f"  {colors.label(comp_label)} ({colors.num(len(pack.cards))} cards): "
        f"{', '.join(names)}"
    )


def _demo_refill_strategies(
    cards: list[CardDesign],
    cfg: SimulatorConfig,
    consumption_mode: CubeConsumptionMode,
    rng: random.Random,
) -> None:
    """Demonstrate each refill strategy on a sample pack."""
    print(f"\n{colors.section('=== Refill Strategy Demonstrations ===')}")

    copies = _resolve_copies_per_card(cards, cfg)

    def _fresh_pack() -> tuple[cube_manager.CubeManager, Pack]:
        c = cube_manager.CubeManager(
            designs=cards,
            copies_per_card=copies,
            consumption_mode=consumption_mode,
        )
        p = pack_generator.generate_pack("uniform", c, cfg, rng)
        return c, p

    print(f"\n{colors.section('--- NoRefill ---')}")
    _, no_refill_pack = _fresh_pack()
    _print_pack_composition("Pack before", no_refill_pack)
    result = refill.no_refill()
    print(f"  {colors.label('Refill card:')} {colors.dim('None (no card added)')}")
    _print_pack_composition("Pack after", no_refill_pack)
    print(
        f"  {colors.label('Pack size before:')} {colors.num(len(no_refill_pack.cards))}"
    )
    print(
        f"  {colors.label('Pack size after:')}  {colors.num(len(no_refill_pack.cards))}"
    )

    print(f"\n{colors.section('--- UniformRefill ---')}")
    uniform_cube, uniform_pack = _fresh_pack()
    _print_pack_composition("Pack before", uniform_pack)
    size_before = len(uniform_pack.cards)
    uniform_card = refill.uniform_refill(uniform_cube, rng)
    uniform_pack.cards.append(uniform_card)
    print(f"  {colors.label('Refill card:')} {colors.card(uniform_card.design.name)}")
    _print_pack_composition("Pack after", uniform_pack)
    print(f"  {colors.label('Pack size before:')} {colors.num(size_before)}")
    print(
        f"  {colors.label('Pack size after:')}  {colors.num(len(uniform_pack.cards))}"
    )

    print(f"\n{colors.section('--- ConstrainedRefill ---')}")
    constrained_cube, constrained_pack = _fresh_pack()
    _print_pack_composition("Pack before", constrained_pack)
    size_before = len(constrained_pack.cards)

    if cfg.refill.fingerprint_source == "round_environment":
        all_packs = [constrained_pack]
        signal = refill.compute_round_environment_profile(all_packs)
        print(
            f"  {colors.label('Fingerprint source:')} "
            f"{colors.c('round_environment', 'special')}"
        )
    else:
        signal = constrained_pack.archetype_profile
        print(
            f"  {colors.label('Fingerprint source:')} "
            f"{colors.c('pack_origin', 'special')}"
        )

    signal_str = ", ".join(f"{colors.num(f'{v:.4f}')}" for v in signal)
    print(f"  {colors.label('Signal vector:')} [{signal_str}]")

    constrained_card = refill.constrained_refill(
        cube=constrained_cube,
        signal=signal,
        fidelity=cfg.refill.fidelity,
        commit_bias=cfg.refill.commit_bias,
        rng=rng,
    )
    constrained_pack.cards.append(constrained_card)
    similarity = refill.cosine_similarity(constrained_card.design.fitness, signal)
    print(
        f"  {colors.label('Refill card:')} "
        f"{colors.card(constrained_card.design.name)}"
    )
    print(
        f"  {colors.label('Cosine similarity to signal:')} "
        f"{colors.num(f'{similarity:.4f}')}"
    )
    _print_pack_composition("Pack after", constrained_pack)
    print(f"  {colors.label('Pack size before:')} {colors.num(size_before)}")
    print(
        f"  {colors.label('Pack size after:')}  "
        f"{colors.num(len(constrained_pack.cards))}"
    )

    print(f"\n  {colors.label('Round environment profile (across all demo packs):')}")
    demo_packs = [uniform_pack, constrained_pack]
    round_env = refill.compute_round_environment_profile(demo_packs)
    env_str = ", ".join(f"{colors.num(f'{v:.4f}')}" for v in round_env)
    print(f"    [{env_str}]")


def _demo_pick_policies(
    cards: list[CardDesign],
    cfg: SimulatorConfig,
    consumption_mode: CubeConsumptionMode,
    rng: random.Random,
) -> None:
    """Demonstrate each pick policy selecting a card from a sample pack."""
    demo_cube = cube_manager.CubeManager(
        designs=cards,
        copies_per_card=_resolve_copies_per_card(cards, cfg),
        consumption_mode=consumption_mode,
    )
    pack = pack_generator.generate_pack("uniform", demo_cube, cfg, rng)

    print(
        f"\n{colors.section('--- Pick Policy Demonstrations')} "
        f"({colors.label('pack_id')}={colors.num(pack.pack_id)}) ---"
    )
    print(f"  {colors.label('Pack contains')} {colors.num(len(pack.cards))} cards")

    policies = [
        ("greedy", "Greedy"),
        ("archetype_loyal", "Archetype-loyal"),
        ("force", "Force (target archetype=0)"),
        ("adaptive", "Adaptive"),
        ("signal_ignorant", "Signal-ignorant"),
    ]

    for policy_key, policy_label in policies:
        agent = agents.create_agent(cfg.cards.archetype_count)
        policy_rng = random.Random(rng.randint(0, 2**32))
        force_arch = 0 if policy_key == "force" else None

        pick = agents.pick_card(
            pack.cards,
            agent,
            policy_key,
            cfg.agents,
            cfg.scoring,
            policy_rng,
            force_archetype=force_arch,
        )

        score = _policy_score(pick, agent, policy_key, cfg, force_arch)
        top_arch = argmax(pick.design.fitness)
        top_fit = pick.design.fitness[top_arch]

        print(
            f"\n  {colors.label(f'{policy_label}:')}"
            f"\n    {colors.label('Selected:')} {colors.card(pick.design.name)}"
            f"  ({colors.label('power')}={colors.num(f'{pick.design.power:.3f}')}, "
            f"{colors.label(f'top_fitness[{top_arch}]')}="
            f"{colors.num(f'{top_fit:.3f}')})"
            f"\n    {colors.label('Score:')} {colors.num(f'{score:.4f}')}"
        )

        if policy_key == "force" and force_arch is not None:
            print(
                f"    {colors.label(f'Force archetype {force_arch} fitness:')} "
                f"{colors.num(f'{pick.design.fitness[force_arch]:.3f}')}"
            )


def _policy_score(
    card: CardInstance,
    agent: agents.AgentState,
    policy: str,
    cfg: SimulatorConfig,
    force_arch: int | None,
) -> float:
    """Compute the policy-specific score for the selected card."""
    if policy == "greedy":
        return agents.score_card_greedy(card, agent, cfg.scoring)
    elif policy == "archetype_loyal":
        best = argmax(agent.w)
        return card.design.fitness[best]
    elif policy == "force":
        arch = force_arch if force_arch is not None else 0
        return card.design.fitness[arch]
    elif policy == "adaptive":
        return agents.score_card_adaptive(card, agent, cfg.agents)
    elif policy == "signal_ignorant":
        return agents.score_card_signal_ignorant(card, agent, cfg.agents)
    return 0.0


def _demo_show_n_strategies(
    cards: list[CardDesign],
    cfg: SimulatorConfig,
    consumption_mode: CubeConsumptionMode,
    rng: random.Random,
) -> None:
    """Demonstrate each Show-N strategy selecting cards from a pack."""
    demo_cube = cube_manager.CubeManager(
        designs=cards,
        copies_per_card=_resolve_copies_per_card(cards, cfg),
        consumption_mode=consumption_mode,
    )
    pack = pack_generator.generate_pack("uniform", demo_cube, cfg, rng)

    human_w = commitment.initial_preference_vector(cfg.cards.archetype_count)
    human_w[0] = 2.0

    print(
        f"\n{colors.section('--- Show-N Strategy Demonstrations')} "
        f"({colors.label('pack_id')}={colors.num(pack.pack_id)}) ---"
    )
    print(
        f"  {colors.label('Pack contains')} {colors.num(len(pack.cards))} cards, "
        f"{colors.label('show_n')}={colors.num(cfg.agents.show_n)}"
    )
    print(
        f"  {colors.label('Human best archetype:')} "
        f"{colors.c(argmax(human_w), 'operator')}"
    )

    strategy_labels = {
        "uniform": "Uniform",
        "power_biased": "Power-biased",
        "curated": "Curated",
        "signal_rich": "Signal-rich",
    }

    for strategy_key, strategy_label in strategy_labels.items():
        strategy_rng = random.Random(rng.randint(0, 2**32))
        selected = show_n.select_cards(
            pack.cards,
            cfg.agents.show_n,
            strategy_key,
            strategy_rng,
            human_w=human_w,
        )

        print(
            f"\n  {colors.label(f'{strategy_label}')} "
            f"({colors.num(len(selected))} cards):"
        )

        for card_item in selected:
            top_arch = argmax(card_item.design.fitness)
            fitness_for_best = card_item.design.fitness[argmax(human_w)]
            tag = ""
            if strategy_key == "curated":
                if fitness_for_best >= 0.6:
                    tag = f" {colors.ok('[on-plan]')}"
                elif fitness_for_best < 0.3 and card_item.design.power >= 0.5:
                    tag = f" {colors.c('[off-plan strong]', 'warning')}"
            print(
                f"    {colors.card(f'{card_item.design.name:<30s}')} "
                f"{colors.label('power')}="
                f"{colors.num(f'{card_item.design.power:.3f}')}  "
                f"{colors.label(f'fitness[{argmax(human_w)}]')}="
                f"{colors.num(f'{fitness_for_best:.3f}')}"
                f"{tag}"
            )


def _run_with_error_handling() -> None:
    """Run main() with clean error handling for all exit paths."""
    try:
        main()
    except cube_manager.CubeSupplyError as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
    except KeyboardInterrupt:
        print("\n  Simulation interrupted.")
        sys.exit(0)
    except SystemExit:
        raise
    except Exception:
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    _run_with_error_handling()
