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
from typing import Any, Optional

_PROMPT_PATH = Path(".logs/quest_ai_prompt.json")
_RESPONSE_PATH = Path(".logs/quest_ai_response.json")

# Add draft_simulator_v2 to sys.path for cross-module imports
_DRAFT_SIM_DIR = str(Path(__file__).resolve().parent.parent / "draft_simulator_v2")
if _DRAFT_SIM_DIR not in sys.path:
    sys.path.insert(0, _DRAFT_SIM_DIR)

import agents
import card_generator
import cube_manager
from config import SimulatorConfig, _apply_dict, _apply_override, _load_file
from draft_models import CubeConsumptionMode

import atlas
import data_loader
import flow
import input_handler
import render
import resonance_filter
from draft_strategy import ArchetypeDraftStrategy, RankDraftStrategy, SixSeatDraftStrategy
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
        "--debug",
        action="store_true",
        default=False,
        help="Show AI draft bot debug panel in dreamscape site-selection menu",
    )
    parser.add_argument(
        "--ai",
        action="store_true",
        default=False,
        help="Enable AI turn protocol mode for sub-agent play-testing",
    )
    parser.add_argument(
        "--web",
        action="store_true",
        default=False,
        help="Launch a localhost web UI instead of the terminal interface",
    )
    parser.add_argument(
        "--port",
        type=int,
        default=8080,
        help="Port for the web UI server (default: 8080)",
    )
    parser.add_argument(
        "--archetype-draft",
        action="store_true",
        default=False,
        help="Use simple archetype-based draft instead of 6-seat AI draft",
    )
    parser.add_argument(
        "--num-archetypes",
        type=int,
        default=3,
        help="Number of archetypes to include in archetype draft (default: 3)",
    )
    parser.add_argument(
        "--no-rarity",
        action="store_true",
        default=False,
        help="Disable rarity weighting in archetype draft (all cards at equal weight)",
    )
    parser.add_argument(
        "--original-cards",
        action="store_true",
        default=False,
        help="Restrict card pool to the original 220 cards (archetype draft only)",
    )
    parser.add_argument(
        "--allied",
        action="store_true",
        default=False,
        help="Select contiguous archetypes from the alliance circle (archetype draft only)",
    )
    parser.add_argument(
        "--config",
        type=str,
        default=None,
        help="Path to TOML/JSON config file for draft parameters",
    )
    parser.add_argument(
        "--set",
        action="append",
        default=[],
        dest="overrides",
        metavar="KEY=VALUE",
        help="Override config field (dot-notation, repeatable). E.g. --set pack_generation.strategy=uniform",
    )
    parser.add_argument(
        "--rank-draft",
        action="store_true",
        default=False,
        help="Use rank-based draft: cards with w1-rank below threshold, no resonance/archetype UI",
    )
    parser.add_argument(
        "--rank-threshold",
        type=int,
        default=100,
        help="w1-rank threshold for rank draft (default: 100, cards with rank < threshold are eligible)",
    )
    parser.add_argument(
        "--no-resonance-filter",
        action="store_true",
        default=False,
        help="Disable dual-resonance filtering (all cards eligible regardless of resonance)",
    )
    return parser


def _build_draft_config(
    config_path: Optional[str] = None,
    overrides: Optional[list[str]] = None,
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
    cfg.pack_generation.strategy = "seeded_themed"

    script_dir = os.path.dirname(os.path.abspath(__file__))
    cfg.cards.rendered_toml_path = os.path.join(
        script_dir, "..", "..", "rules_engine", "tabula", "rendered-cards.toml"
    )

    if config_path is not None:
        raw = _load_file(config_path)
        _apply_dict(cfg, raw)

    if overrides:
        for override in overrides:
            _apply_override(cfg, override)

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

    # Launch web UI if requested
    if args.web:
        from web.server import run_web_server

        run_web_server(args)
        return

    # Set up AI mode if requested
    if args.ai:
        input_handler.set_ai_mode(True)
        input_handler.install_output_capture()
        # Clean up stale files from previous runs
        for stale in (_PROMPT_PATH, _RESPONSE_PATH):
            try:
                stale.unlink()
            except OSError:
                pass

    # Determine seed
    seed: int = args.seed if args.seed is not None else random.randint(0, 2**32)
    rng = random.Random(seed)

    # Build draft engine configuration
    cfg = _build_draft_config(
        config_path=args.config,
        overrides=args.overrides if args.overrides else None,
    )

    # Generate card pool
    if args.archetype_draft or args.rank_draft:
        if cfg.cards.rendered_toml_path is None:
            raise ValueError(
                "--archetype-draft/--rank-draft requires cards.rendered_toml_path to be set"
            )
        cards = card_generator.load_cards(
            cfg.cards.rendered_toml_path,
            original_only=args.original_cards,
        )
    else:
        cards = card_generator.generate_cards(cfg, rng)

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

    # Initialize quest state
    state = QuestState(
        essence=starting_essence,
        rng=rng,
        max_deck=max_deck,
        min_deck=min_deck,
        max_dreamsigns=max_dreamsigns,
        debug=args.debug,
    )

    # Wire up the draft strategy
    if args.archetype_draft:
        state.archetype_draft = True
        archetype_strategy = ArchetypeDraftStrategy(
            rng=rng,
            all_cards=cards,
            num_archetypes=args.num_archetypes,
            no_rarity=args.no_rarity,
            allied=args.allied,
        )
        archetype_names = [
            render.ARCHETYPE_NAMES[i] for i in archetype_strategy.selected_archetypes
        ]
        print(
            f"  Archetype draft initialized: {', '.join(archetype_names)}, "
            f"pool size {archetype_strategy.pool_size} instances "
            f"from {len(cards)} card designs"
        )
        state.draft_strategy = archetype_strategy
    elif args.rank_draft:
        state.rank_draft = True
        rank_strategy = RankDraftStrategy(
            rng=rng,
            all_cards=cards,
            rank_threshold=args.rank_threshold,
        )
        print(
            f"  Rank draft initialized: threshold {args.rank_threshold}, "
            f"pool size {rank_strategy.pool_size} instances "
            f"from {len(cards)} card designs"
        )
        state.draft_strategy = rank_strategy
    else:
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

        print(
            f"  Draft engine initialized: {len(cards)} card designs, "
            f"cube size {cube.total_size}, "
            f"{1 + len(ai_agents)} agents created"
        )

        if args.no_resonance_filter:
            resonance_pair_fn = lambda: None
            cfg.agents.ai_resonance_commit_pick = 9999
        else:
            resonance_pair_fn = lambda: resonance_filter.human_resonance_pair(state)

        state.draft_strategy = SixSeatDraftStrategy(
            rng=rng,
            human_agent=human_agent,
            ai_agents=ai_agents,
            cube=cube,
            draft_cfg=cfg,
            resonance_pair_fn=resonance_pair_fn,
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
