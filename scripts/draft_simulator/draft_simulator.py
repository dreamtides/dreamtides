#!/usr/bin/env python3
"""Entry point for the draft simulator.

Parses CLI arguments, loads configuration, generates or loads the card
pool, and prints summary statistics. Stdlib-only, no external dependencies.
"""

import argparse
import random
import sys
import traceback

import card_generator
import config

VERSION = "0.1.0"


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
        choices=["single", "sweep", "trace"],
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
        "-n",
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


def _run_with_error_handling() -> None:
    """Run main() with clean error handling for all exit paths."""
    try:
        main()
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
