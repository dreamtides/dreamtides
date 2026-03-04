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
import config
import cube_manager
import pack_generator
from draft_models import CubeConsumptionMode, Pack

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
