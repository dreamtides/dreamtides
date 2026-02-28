#!/usr/bin/env python3
"""Entry point for the interactive quest simulator.

Parses CLI arguments, loads all data files, initializes quest state,
creates the dream atlas, and launches the main quest flow.
"""

import argparse
import random
import sys
import traceback
from typing import Any

import atlas
import data_loader
import flow
import input_handler
import pool
import render
from jsonl_log import SessionLogger
from models import AlgorithmParams
from quest_state import QuestState
from site_dispatch import SiteData


def build_parser() -> argparse.ArgumentParser:
    """Build the argument parser for the quest simulator CLI."""
    parser = argparse.ArgumentParser(
        description="Interactive quest simulator for Dreamtides.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--seed", "-s",
        type=int,
        default=None,
        help="Random seed (default: random)",
    )
    parser.add_argument(
        "--exponent",
        type=float,
        default=None,
        help="Resonance affinity exponent",
    )
    parser.add_argument(
        "--floor-weight",
        type=float,
        default=None,
        help="Minimum weight for resonance cards",
    )
    parser.add_argument(
        "--neutral-base",
        type=float,
        default=None,
        help="Weight for neutral cards",
    )
    parser.add_argument(
        "--staleness-factor",
        type=float,
        default=None,
        help="Staleness penalty factor",
    )
    return parser


def main() -> None:
    """Run the quest simulator."""
    parser = build_parser()
    args = parser.parse_args()

    # Determine seed
    seed: int = args.seed if args.seed is not None else random.randint(0, 2**32)
    rng = random.Random(seed)

    # Load all data
    all_cards = data_loader.load_cards()
    algorithm_params, draft_params, pool_params, extra_config = (
        data_loader.load_config()
    )
    dreamcallers = data_loader.load_dreamcallers()
    dreamsigns = data_loader.load_dreamsigns()
    journeys = data_loader.load_journeys()
    offers = data_loader.load_offers()
    banes = data_loader.load_banes()
    bosses = data_loader.load_bosses()

    # Override algorithm params with CLI args if provided
    algorithm_params = AlgorithmParams(
        exponent=(
            args.exponent
            if args.exponent is not None
            else algorithm_params.exponent
        ),
        floor_weight=(
            args.floor_weight
            if args.floor_weight is not None
            else algorithm_params.floor_weight
        ),
        neutral_base=(
            args.neutral_base
            if args.neutral_base is not None
            else algorithm_params.neutral_base
        ),
        staleness_factor=(
            args.staleness_factor
            if args.staleness_factor is not None
            else algorithm_params.staleness_factor
        ),
    )

    # Build draft pool
    variance = pool.generate_variance(rng, pool_params)
    draft_pool = pool.build_pool(all_cards, pool_params, variance)

    # Read quest config values
    quest_config: dict[str, Any] = extra_config.get("quest", {})
    starting_essence: int = int(quest_config.get("starting_essence", 250))
    max_deck: int = int(quest_config.get("max_deck", 50))
    min_deck: int = int(quest_config.get("min_deck", 25))
    max_dreamsigns: int = int(quest_config.get("max_dreamsigns", 12))
    total_battles: int = int(quest_config.get("total_battles", 7))

    # Initialize quest state
    state = QuestState(
        essence=starting_essence,
        pool=draft_pool,
        rng=rng,
        all_cards=all_cards,
        pool_variance=variance,
        max_deck=max_deck,
        min_deck=min_deck,
        max_dreamsigns=max_dreamsigns,
    )

    # Assemble data bundle for site dispatch
    data = SiteData(
        dreamcallers=dreamcallers,
        dreamsigns=dreamsigns,
        journeys=journeys,
        offers=offers,
        banes=banes,
        bosses=bosses,
        algorithm_params=algorithm_params,
        draft_params=draft_params,
        pool_params=pool_params,
        config=extra_config,
    )

    # Create initial atlas
    nodes = atlas.initialize_atlas(rng)

    # Create JSONL logger
    logger = SessionLogger(seed)

    # Log session start
    logger.log_session_start(seed, algorithm_params, nodes)

    # Display quest start banner
    banner = render.quest_start_banner(
        seed=seed,
        starting_essence=starting_essence,
        pool_size=len(draft_pool),
        unique_cards=len(all_cards),
    )
    print(banner)

    # Wait for user to continue
    input_handler.wait_for_continue(prompt="")

    # Launch main quest flow
    try:
        flow.run_quest(
            state=state,
            nodes=nodes,
            data=data,
            total_battles=total_battles,
            logger=logger,
        )
    finally:
        logger.close()


def _run_with_error_handling() -> None:
    """Run main() with terminal restoration in all exit paths."""
    try:
        main()
    except KeyboardInterrupt:
        input_handler.ensure_terminal_restored()
        print("\n  Quest abandoned.")
        sys.exit(0)
    except SystemExit:
        input_handler.ensure_terminal_restored()
        raise
    except Exception:
        input_handler.ensure_terminal_restored()
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    _run_with_error_handling()
