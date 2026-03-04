#!/usr/bin/env python3
"""Entry point for the draft simulator.

Parses CLI arguments, loads configuration, generates or loads the card
pool, creates a cube, generates packs using each strategy, and prints
summary statistics. Stdlib-only, no external dependencies.
"""

import argparse
import random
import sys
import traceback

import card_generator
import commitment
import config
import cube_manager
import deck_scorer
import pack_generator
from config import SimulatorConfig
from draft_models import CardDesign, CubeConsumptionMode, Pack

VERSION = "0.1.0"


def build_parser() -> argparse.ArgumentParser:
    """Build the argument parser for the draft simulator CLI."""
    parser = argparse.ArgumentParser(
        description="Draft simulator for Dreamtides.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
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
    return parser


def main() -> None:
    """Run the draft simulator."""
    parser = build_parser()
    args = parser.parse_args()

    seed: int = args.seed
    rng = random.Random(seed)

    # Load configuration
    cfg = config.load_config(
        config_path=args.config,
        overrides=args.param,
    )

    # Print banner
    print(
        f"Draft Simulator v{VERSION} | seed={seed} "
        f"| seats={cfg.draft.seat_count} "
        f"| rounds={cfg.draft.round_count} "
        f"| pack_size={cfg.draft.pack_size}"
    )

    # Generate or load card pool
    cards = card_generator.generate_cards(cfg, rng)

    source_label = (
        f"file ({cfg.cards.file_path})" if cfg.cards.source == "file" else "synthetic"
    )
    print(f"\nCard source: {source_label}")

    # Print card pool statistics
    card_generator.print_card_pool_stats(cards, cfg.cards.archetype_count)

    # Create cube and validate supply
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

    # Generate one pack using each strategy and print contents.
    # Each strategy gets a fresh cube so draws are independent.
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

    # Demonstrate deck value scoring
    _demo_deck_scoring(cards, cfg, rng)

    # Demonstrate commitment detection
    _demo_commitment_detection(cards, cfg, rng)


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

    # Build a plausible preference vector by simulating picks
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
    print(f"  Effective archetype: {_argmax(w)}")
    print(f"  Raw power:            {raw:.4f}")
    print(f"  Archetype coherence:  {coherence:.4f}")
    print(f"  Focus bonus:          {focus:.4f}")
    print(f"  Final score:          {final:.4f}")

    # Verify weighted sum
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
    """Simulate a sequence of picks and run commitment detection.

    Uses an archetype-biased pick strategy to demonstrate commitment:
    picks cards with higher fitness for the agent's best archetype,
    similar to how a real agent would draft.
    """
    pool_size = 30

    # Build w history by simulating biased picks from the card pool
    w = commitment.initial_preference_vector(cfg.cards.archetype_count)
    w_history: list[list[float]] = [list(w)]
    available = list(cards)
    picked: list[CardDesign] = []

    for _ in range(pool_size):
        if not available:
            break
        # Score candidates by fitness for current best archetype + power
        best_arch = _argmax(w)
        scored = [(c.fitness[best_arch] + 0.3 * c.power, c) for c in available]
        scored.sort(key=lambda x: x[0], reverse=True)
        # Pick from top candidates with some randomness
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

    # Also demonstrate None case with uniform w history
    print("\n  Uniform w test (should be uncommitted):")
    uniform_history = [
        commitment.initial_preference_vector(cfg.cards.archetype_count)
        for _ in range(pool_size + 1)
    ]
    uniform_result = commitment.detect_commitment(uniform_history, cfg.commitment)
    print(f"    Commitment pick: {uniform_result.commitment_pick}")
    print(f"    Entropy commitment pick: {uniform_result.entropy_commitment_pick}")


def _argmax(values: list[float]) -> int:
    """Return the index of the maximum value."""
    best_index = 0
    best_value = values[0]
    for i in range(1, len(values)):
        if values[i] > best_value:
            best_value = values[i]
            best_index = i
    return best_index


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
