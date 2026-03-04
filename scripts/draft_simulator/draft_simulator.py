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
import refill
import show_n
import sweep
import validation
from config import SimulatorConfig
from draft_models import CardDesign, CardInstance, CubeConsumptionMode, Pack
from utils import argmax

VERSION = "0.1.0"


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
        choices=["single", "sweep", "trace", "demo"],
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
            cfg, seed, describe_pool=args.describe_pool, runs=runs, preset=args.preset
        )
    elif mode == "trace":
        _run_trace(cfg, seed, output_dir)
    elif mode == "demo":
        _run_demo(cfg, seed)
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


def _run_single(
    cfg: SimulatorConfig,
    seed: int,
    describe_pool: bool = False,
    runs: int = 1,
    preset: str | None = None,
) -> None:
    """Run one or more drafts and print per-seat results with metrics."""
    _print_header(cfg, seed, runs, preset)

    if runs == 1:
        _run_single_once(cfg, seed, describe_pool)
    else:
        _run_single_multi(cfg, seed, runs, describe_pool)


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
) -> None:
    """Run multiple drafts and print averaged results."""
    seat_count = cfg.draft.seat_count
    all_metrics: list[metrics.DraftMetrics] = []
    seat_deck_values: list[list[float]] = [[] for _ in range(seat_count)]
    seat_archetypes: list[list[int | None]] = [[] for _ in range(seat_count)]
    seat_commitment_picks: list[list[int | None]] = [[] for _ in range(seat_count)]
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

        for seat_idx, sr in enumerate(result.seat_results):
            seat_deck_values[seat_idx].append(sr.deck_value)
            seat_archetypes[seat_idx].append(sr.committed_archetype)
            seat_commitment_picks[seat_idx].append(sr.commitment_pick)

        all_metrics.append(metrics.compute_metrics(result, cfg))

        # Progress bar on stderr
        done = run_i + 1
        print(
            colors.format_progress_bar(done, runs, use_color=sys.stderr.isatty()),
            end="",
            file=sys.stderr,
        )

    print(file=sys.stderr)  # newline after progress bar

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
    print()
    print(metrics.format_goal_metrics(averaged))
    print()
    print(metrics.format_metrics(averaged))


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
    cube = cube_manager.CubeManager(
        designs=cards,
        copies_per_card=cfg.cube.copies_per_card,
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
            copies_per_card=cfg.cube.copies_per_card,
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

    def _fresh_pack() -> tuple[cube_manager.CubeManager, Pack]:
        c = cube_manager.CubeManager(
            designs=cards,
            copies_per_card=cfg.cube.copies_per_card,
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
        copies_per_card=cfg.cube.copies_per_card,
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
        copies_per_card=cfg.cube.copies_per_card,
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
