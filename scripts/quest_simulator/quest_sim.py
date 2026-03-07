#!/usr/bin/env python3
"""Entry point for the interactive quest simulator.

Parses CLI arguments, initializes the draft engine with synthetic cards,
creates agents and cube, loads quest data files, and launches the main
quest flow.
"""

import argparse
import os
import random
import sys
import traceback
from pathlib import Path
from typing import Any

# Add draft_simulator to sys.path for cross-module imports
_DRAFT_SIM_DIR = str(Path(__file__).resolve().parent.parent / "draft_simulator")
if _DRAFT_SIM_DIR not in sys.path:
    sys.path.insert(0, _DRAFT_SIM_DIR)

import agents
import card_generator
import cube_manager
from config import SimulatorConfig
from draft_models import CubeConsumptionMode

import atlas
import data_loader
import flow
import input_handler
import render
from jsonl_log import SessionLogger
from quest_state import QuestState
from site_dispatch import SiteData


def build_parser() -> argparse.ArgumentParser:
    """Build the argument parser for the quest simulator CLI."""
    parser = argparse.ArgumentParser(
        description="Interactive quest simulator for Dreamtides.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--seed",
        "-s",
        type=int,
        default=None,
        help="Random seed (default: random)",
    )
    parser.add_argument(
        "--synthetic",
        action="store_true",
        default=False,
        help="Use synthetic cards instead of real TOML cards",
    )
    parser.add_argument(
        "--real-only",
        action="store_true",
        default=False,
        help="Fill pool to 360 by duplicating real cards instead of adding synthetics",
    )
    return parser


def _build_draft_config(
    synthetic: bool = False, real_only: bool = False
) -> SimulatorConfig:
    """Construct a SimulatorConfig for quest mode without validation."""
    cfg = SimulatorConfig()
    cfg.draft.seat_count = 6
    cfg.draft.pack_size = 20
    cfg.draft.human_seats = 1
    cfg.draft.alternate_direction = False
    cfg.agents.show_n = 4
    cfg.agents.show_n_strategy = "sharpened_preference"
    cfg.agents.policy = "adaptive"
    cfg.agents.ai_optimality = 0.80
    cfg.agents.learning_rate = 3.0
    cfg.agents.openness_window = 3
    cfg.cards.archetype_count = 8
    cfg.cube.distinct_cards = 360
    cfg.cube.copies_per_card = 1
    cfg.cube.consumption_mode = "with_replacement"
    cfg.rarity.enabled = True
    cfg.refill.strategy = "no_refill"
    cfg.pack_generation.strategy = "seeded_themed"

    if synthetic:
        cfg.cards.source = "synthetic"
    else:
        cfg.cards.source = "toml"
        cfg.cards.real_only = real_only
        script_dir = os.path.dirname(os.path.abspath(__file__))
        cfg.cards.rendered_toml_path = os.path.join(
            script_dir, "data", "rendered-cards.toml"
        )
        cfg.cards.metadata_toml_path = os.path.join(
            script_dir, "..", "..", "rules_engine", "tabula", "card-metadata.toml"
        )

    return cfg


def draft_config_summary(cfg: SimulatorConfig) -> dict[str, object]:
    """Return a JSON-serializable summary of the draft config for logging."""
    return {
        "seat_count": cfg.draft.seat_count,
        "pack_size": cfg.draft.pack_size,
        "human_seats": cfg.draft.human_seats,
        "archetype_count": cfg.cards.archetype_count,
        "distinct_cards": cfg.cube.distinct_cards,
        "consumption_mode": cfg.cube.consumption_mode,
    }


def main() -> None:
    """Run the quest simulator."""
    parser = build_parser()
    args = parser.parse_args()

    # Determine seed
    seed: int = args.seed if args.seed is not None else random.randint(0, 2**32)
    rng = random.Random(seed)

    # Build draft engine configuration
    cfg = _build_draft_config(synthetic=args.synthetic, real_only=args.real_only)

    # Generate card pool
    cards = card_generator.generate_cards(cfg, rng)

    # Create cube with rarity-based copy counts and replacement mode
    copies_per_card: int | dict[str, int] = cube_manager.build_copies_map(
        cards, cfg.rarity
    )
    cube = cube_manager.CubeManager(
        designs=cards,
        copies_per_card=copies_per_card,
        consumption_mode=CubeConsumptionMode.WITH_REPLACEMENT,
    )

    # Create agents: 1 human (seat 0) + 5 AI
    human_agent = agents.create_agent(archetype_count=cfg.cards.archetype_count)
    ai_agents = [
        agents.create_agent(archetype_count=cfg.cards.archetype_count)
        for _ in range(cfg.draft.seat_count - 1)
    ]

    real_count = sum(1 for c in cards if c.is_real)
    synth_count = len(cards) - real_count
    if real_count > 0:
        print(
            f"  Draft engine initialized: {real_count} real + "
            f"{synth_count} synthetic cards, "
            f"cube size {cube.total_size}, "
            f"{1 + len(ai_agents)} agents created"
        )
    else:
        print(
            f"  Draft engine initialized: {len(cards)} cards generated, "
            f"cube size {cube.total_size}, "
            f"{1 + len(ai_agents)} agents created"
        )

    # Load all quest data
    config = data_loader.load_config()
    dreamcallers = data_loader.load_dreamcallers()
    dreamsigns = data_loader.load_dreamsigns()
    journeys = data_loader.load_journeys()
    offers = data_loader.load_offers()
    banes = data_loader.load_banes()
    bosses = data_loader.load_bosses()

    # Read quest config values
    quest_config: dict[str, Any] = config.get("quest", {})
    starting_essence: int = int(quest_config.get("starting_essence", 250))
    max_deck: int = int(quest_config.get("max_deck", 50))
    min_deck: int = int(quest_config.get("min_deck", 25))
    max_dreamsigns: int = int(quest_config.get("max_dreamsigns", 12))
    total_battles: int = int(quest_config.get("total_battles", 7))

    # Initialize quest state with draft engine fields
    state = QuestState(
        essence=starting_essence,
        rng=rng,
        human_agent=human_agent,
        ai_agents=ai_agents,
        cube=cube,
        draft_cfg=cfg,
        packs=None,
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
        config=config,
    )

    # Create initial atlas
    nodes = atlas.initialize_atlas(rng)

    # Create JSONL logger
    logger = SessionLogger(seed)

    # Log session start with draft config summary
    logger.log_session_start(
        seed,
        nodes,
        draft_config=draft_config_summary(cfg),
    )

    # Display quest start banner
    banner = render.quest_start_banner(
        seed=seed,
        starting_essence=starting_essence,
        card_count=len(cards),
        real_card_count=real_count,
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
