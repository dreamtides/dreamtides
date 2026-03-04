#!/usr/bin/env python3
"""Entry point for the draft simulator.

Parses CLI arguments, loads configuration, and dispatches to the selected
mode: single (default) runs one complete draft and prints per-seat results,
sweep runs batch parameter experiments, trace writes per-pick JSON output,
and demo runs the component demonstrations from earlier tasks. Stdlib-only,
no external dependencies.
"""

import argparse
import json
import random
import sys
import traceback

import agents
import card_generator
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
        default=42,
        help="Base RNG seed (default: 42)",
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
        help="Number of runs (default: 1000 for sweep, 1 for single)",
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

    # CLI --seed overrides config base_seed; write back so metadata is accurate
    seed: int = args.seed
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
        _run_single(cfg, seed, describe_pool=args.describe_pool)
    elif mode == "trace":
        _run_trace(cfg, seed, output_dir)
    elif mode == "demo":
        _run_demo(cfg, seed)
    elif mode == "sweep":
        runs = args.runs if args.runs is not None else cfg.sweep.runs_per_point
        _run_sweep(cfg, seed, runs, output_dir)


def _run_single(
    cfg: SimulatorConfig,
    seed: int,
    describe_pool: bool = False,
) -> None:
    """Run a single draft and print per-seat results with metrics."""
    print(
        f"Draft Simulator v{VERSION} | seed={seed} "
        f"| seats={cfg.draft.seat_count} "
        f"| rounds={cfg.draft.round_count} "
        f"| pack_size={cfg.draft.pack_size}"
    )

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
            commit_str = f"pick {sr.commitment_pick}"
        else:
            commit_str = "uncommitted"

        archetype = (
            sr.committed_archetype
            if sr.committed_archetype is not None
            else argmax(sr.final_w)
        )

        label = f"Seat {seat_idx} ({seat_type}, {policy}):"
        print(
            f"{label:<35s} deck_value={sr.deck_value:.3f}, "
            f"archetype={archetype}, committed={commit_str}"
        )

    draft_metrics = metrics.compute_metrics(result, cfg)
    print()
    print(metrics.format_metrics(draft_metrics))


def _run_trace(
    cfg: SimulatorConfig,
    seed: int,
    output_dir: str,
) -> None:
    """Run a single draft with per-pick trace output to JSON file."""
    output.ensure_output_dir(output_dir)

    result = draft_runner.run_draft(cfg, seed, trace_enabled=True)

    trace_path = output.write_trace_json(output_dir, result.traces, seed)
    print(f"Per-pick trace written to: {trace_path}")


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

    print(f"\nResults written to:")
    print(f"  {run_path:<40s} ({len(run_records)} rows)")
    print(f"  {agg_path:<40s} ({len(aggregate_records)} rows)")
    print(f"  {meta_path}")

    report = validation.run_validation(aggregate_records, run_records)
    print(validation.format_validation_report(report))


def _run_demo(cfg: SimulatorConfig, seed: int) -> None:
    """Run the component demonstration mode (legacy)."""
    rng = random.Random(seed)

    print(
        f"Draft Simulator v{VERSION} | seed={seed} "
        f"| seats={cfg.draft.seat_count} "
        f"| rounds={cfg.draft.round_count} "
        f"| pack_size={cfg.draft.pack_size}"
    )

    cards = card_generator.generate_cards(cfg, rng)

    source_label = (
        f"file ({cfg.cards.file_path})" if cfg.cards.source == "file" else "synthetic"
    )
    print(f"\nCard source: {source_label}")

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


def _print_pack(pack: Pack, label: str, archetype_count: int) -> None:
    """Print pack contents for QA inspection."""
    print(f"\n--- {label} Pack (id={pack.pack_id}) ---")
    print(f"  Cards ({len(pack.cards)}):")
    for card in pack.cards:
        top_arch = 0
        top_val = card.design.fitness[0] if card.design.fitness else 0.0
        for i in range(1, min(len(card.design.fitness), archetype_count)):
            if card.design.fitness[i] > top_val:
                top_val = card.design.fitness[i]
                top_arch = i
        print(
            f"    {card.design.name:<30s} "
            f"top_arch={top_arch}  power={card.design.power:.3f}"
        )
    profile_str = ", ".join(f"{v:.3f}" for v in pack.archetype_profile)
    print(f"  Archetype profile: [{profile_str}]")


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

    print("\n--- Deck Value Scoring ---")
    print(f"  Pool size: {len(pool)}")
    print(f"  Effective archetype: {argmax(w)}")
    print(f"  Raw power:            {raw:.4f}")
    print(f"  Archetype coherence:  {coherence:.4f}")
    print(f"  Focus bonus:          {focus:.4f}")
    print(f"  Final score:          {final:.4f}")

    weighted = (
        cfg.scoring.weight_power * raw
        + cfg.scoring.weight_coherence * coherence
        + cfg.scoring.weight_focus * focus
    )
    print(
        f"  Weighted sum (before clamp): {weighted:.4f} "
        f"(weights: {cfg.scoring.weight_power}/{cfg.scoring.weight_coherence}"
        f"/{cfg.scoring.weight_focus})"
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

    print("\n--- Commitment Detection ---")
    print(f"  Picks simulated: {len(picked)}")
    print(f"  Threshold: {cfg.commitment.commitment_threshold}")
    print(f"  Stability window: {cfg.commitment.stability_window}")

    print(f"\n  Primary (concentration-based):")
    if result.commitment_pick is not None:
        print(f"    Commitment pick: {result.commitment_pick}")
        print(f"    Committed archetype: {result.committed_archetype}")
        w_at_pick = w_history[result.commitment_pick]
        print(f"    Concentration at pick: {commitment.concentration(w_at_pick):.4f}")
    else:
        print("    Commitment pick: None (uncommitted)")

    print(f"\n  Secondary (entropy-based):")
    print(f"    Entropy threshold: {cfg.commitment.entropy_threshold} bits")
    if result.entropy_commitment_pick is not None:
        print(f"    Entropy commitment pick: {result.entropy_commitment_pick}")
        print(f"    Entropy committed archetype: {result.entropy_committed_archetype}")
        w_at_pick = w_history[result.entropy_commitment_pick]
        print(f"    Entropy at pick: {commitment.shannon_entropy(w_at_pick):.4f} bits")
    else:
        print("    Entropy commitment pick: None (uncommitted)")

    print("\n  Uniform w test (should be uncommitted):")
    uniform_history = [
        commitment.initial_preference_vector(cfg.cards.archetype_count)
        for _ in range(pool_size)
    ]
    uniform_result = commitment.detect_commitment(uniform_history, cfg.commitment)
    print(f"    Commitment pick: {uniform_result.commitment_pick}")
    print(f"    Entropy commitment pick: {uniform_result.entropy_commitment_pick}")


def _print_pack_composition(label: str, pack: Pack) -> None:
    """Print the card names in a pack for QA inspection."""
    names = [c.design.name for c in pack.cards]
    print(f"  {label} ({len(names)} cards): {', '.join(names)}")


def _demo_refill_strategies(
    cards: list[CardDesign],
    cfg: SimulatorConfig,
    consumption_mode: CubeConsumptionMode,
    rng: random.Random,
) -> None:
    """Demonstrate each refill strategy on a sample pack."""
    print("\n=== Refill Strategy Demonstrations ===")

    def _fresh_pack() -> tuple[cube_manager.CubeManager, Pack]:
        c = cube_manager.CubeManager(
            designs=cards,
            copies_per_card=cfg.cube.copies_per_card,
            consumption_mode=consumption_mode,
        )
        p = pack_generator.generate_pack("uniform", c, cfg, rng)
        return c, p

    print("\n--- NoRefill ---")
    _, no_refill_pack = _fresh_pack()
    _print_pack_composition("Pack before", no_refill_pack)
    result = refill.no_refill()
    print(f"  Refill card: None (no card added)")
    _print_pack_composition("Pack after", no_refill_pack)
    print(f"  Pack size before: {len(no_refill_pack.cards)}")
    print(f"  Pack size after:  {len(no_refill_pack.cards)}")

    print("\n--- UniformRefill ---")
    uniform_cube, uniform_pack = _fresh_pack()
    _print_pack_composition("Pack before", uniform_pack)
    size_before = len(uniform_pack.cards)
    uniform_card = refill.uniform_refill(uniform_cube, rng)
    uniform_pack.cards.append(uniform_card)
    print(f"  Refill card: {uniform_card.design.name}")
    _print_pack_composition("Pack after", uniform_pack)
    print(f"  Pack size before: {size_before}")
    print(f"  Pack size after:  {len(uniform_pack.cards)}")

    print("\n--- ConstrainedRefill ---")
    constrained_cube, constrained_pack = _fresh_pack()
    _print_pack_composition("Pack before", constrained_pack)
    size_before = len(constrained_pack.cards)

    if cfg.refill.fingerprint_source == "round_environment":
        all_packs = [constrained_pack]
        signal = refill.compute_round_environment_profile(all_packs)
        print(f"  Fingerprint source: round_environment")
    else:
        signal = constrained_pack.archetype_profile
        print(f"  Fingerprint source: pack_origin")

    signal_str = ", ".join(f"{v:.4f}" for v in signal)
    print(f"  Signal vector: [{signal_str}]")

    constrained_card = refill.constrained_refill(
        cube=constrained_cube,
        signal=signal,
        fidelity=cfg.refill.fidelity,
        commit_bias=cfg.refill.commit_bias,
        rng=rng,
    )
    constrained_pack.cards.append(constrained_card)
    similarity = refill.cosine_similarity(constrained_card.design.fitness, signal)
    print(f"  Refill card: {constrained_card.design.name}")
    print(f"  Cosine similarity to signal: {similarity:.4f}")
    _print_pack_composition("Pack after", constrained_pack)
    print(f"  Pack size before: {size_before}")
    print(f"  Pack size after:  {len(constrained_pack.cards)}")

    print("\n  Round environment profile (across all demo packs):")
    demo_packs = [uniform_pack, constrained_pack]
    round_env = refill.compute_round_environment_profile(demo_packs)
    env_str = ", ".join(f"{v:.4f}" for v in round_env)
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

    print(f"\n--- Pick Policy Demonstrations (pack_id={pack.pack_id}) ---")
    print(f"  Pack contains {len(pack.cards)} cards")

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
            f"\n  {policy_label}:"
            f"\n    Selected: {pick.design.name}"
            f"  (power={pick.design.power:.3f}, "
            f"top_fitness[{top_arch}]={top_fit:.3f})"
            f"\n    Score: {score:.4f}"
        )

        if policy_key == "force" and force_arch is not None:
            print(
                f"    Force archetype {force_arch} fitness: "
                f"{pick.design.fitness[force_arch]:.3f}"
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

    print(f"\n--- Show-N Strategy Demonstrations (pack_id={pack.pack_id}) ---")
    print(f"  Pack contains {len(pack.cards)} cards, show_n={cfg.agents.show_n}")
    print(f"  Human best archetype: {argmax(human_w)}")

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

        print(f"\n  {strategy_label} ({len(selected)} cards):")

        for card in selected:
            top_arch = argmax(card.design.fitness)
            fitness_for_best = card.design.fitness[argmax(human_w)]
            label = ""
            if strategy_key == "curated":
                if fitness_for_best >= 0.6:
                    label = " [on-plan]"
                elif fitness_for_best < 0.3 and card.design.power >= 0.5:
                    label = " [off-plan strong]"
            print(
                f"    {card.design.name:<30s} "
                f"power={card.design.power:.3f}  "
                f"fitness[{argmax(human_w)}]={fitness_for_best:.3f}"
                f"{label}"
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
