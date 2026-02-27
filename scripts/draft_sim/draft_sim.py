#!/usr/bin/env python3
"""Draft resonance simulation script.

Validates the resonance weighting algorithm by simulating quest drafts
with synthetic cards and measuring convergence dynamics.
"""

import argparse
import random
import sys

from models import AlgorithmParams, PoolParams, QuestParams, Strategy, StrategyParams
from interactive import run_interactive
from output import print_aggregate, print_evolution, print_sweep, print_trace
from simulation import simulate_quest

DEFAULT_SWEEP_RANGES: dict[str, list[float]] = {
    "exponent": [1.0, 1.2, 1.4, 1.6, 1.8, 2.0],
    "floor_weight": [0.1, 0.3, 0.5, 0.7, 1.0],
    "neutral_base": [1.0, 2.0, 3.0, 4.0, 5.0],
    "staleness_factor": [0.0, 0.1, 0.3, 0.5, 0.8],
    "dreamcaller_bonus": [2, 3, 4, 5, 6],
    "power_weight": [0.5, 1.0, 1.5, 2.0],
    "fit_weight": [0.5, 1.0, 1.5, 2.0, 2.5],
}


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Draft resonance simulation for Dreamtides",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--mode",
        choices=["trace", "aggregate", "sweep", "evolution", "interactive"],
        default="aggregate",
        help="Output mode (default: aggregate)",
    )

    algo = parser.add_argument_group("Algorithm Parameters")
    algo.add_argument("--exponent", type=float, default=1.4)
    algo.add_argument("--floor-weight", type=float, default=0.5)
    algo.add_argument("--neutral-base", type=float, default=3.0)
    algo.add_argument("--staleness-factor", type=float, default=0.3)

    strat = parser.add_argument_group("Player Strategy")
    strat.add_argument(
        "--strategy",
        choices=["synergy", "power_chaser", "rigid"],
        default="synergy",
    )
    strat.add_argument("--power-weight", type=float, default=1.0)
    strat.add_argument("--fit-weight", type=float, default=1.5)

    quest = parser.add_argument_group("Quest Structure")
    quest.add_argument("--dreamcaller-bonus", type=int, default=4)
    quest.add_argument("--mono-dreamcaller", action="store_true")

    sweep = parser.add_argument_group("Sweep Mode")
    sweep.add_argument("--sweep-param", default="exponent")
    sweep.add_argument("--sweep-values", type=float, nargs="+")
    sweep.add_argument("--sweep-strategies", action="store_true")

    out = parser.add_argument_group("Output")
    out.add_argument("--runs", "-n", type=int, default=1000)
    out.add_argument("--seed", "-s", type=int)

    return parser


def make_params(args: argparse.Namespace):
    algo = AlgorithmParams(
        exponent=args.exponent,
        floor_weight=args.floor_weight,
        neutral_base=args.neutral_base,
        staleness_factor=args.staleness_factor,
    )
    pool = PoolParams()
    quest = QuestParams(
        dreamcaller_bonus=args.dreamcaller_bonus,
        mono_dreamcaller=args.mono_dreamcaller,
    )
    strat = StrategyParams(
        strategy=Strategy(args.strategy),
        power_weight=args.power_weight,
        fit_weight=args.fit_weight,
    )
    return algo, pool, quest, strat


def run_batch(algo, pool, quest, strat, n, rng):
    return [
        simulate_quest(algo, pool, quest, strat, random.Random(rng.randint(0, 2**32)))
        for _ in range(n)
    ]


def mode_trace(args):
    algo, pool, quest, strat = make_params(args)
    seed = args.seed if args.seed is not None else 42
    rng = random.Random(seed)
    result = simulate_quest(algo, pool, quest, strat, rng)
    print_trace(result)


def mode_aggregate(args):
    algo, pool, quest, strat = make_params(args)
    seed = args.seed if args.seed is not None else 0
    rng = random.Random(seed)
    results = run_batch(algo, pool, quest, strat, args.runs, rng)
    print_aggregate(results)


def mode_sweep(args):
    param_name = args.sweep_param
    values = args.sweep_values or DEFAULT_SWEEP_RANGES.get(
        param_name, [1.0, 1.5, 2.0]
    )
    seed = args.seed if args.seed is not None else 0

    strategies = (
        [Strategy.SYNERGY, Strategy.POWER_CHASER, Strategy.RIGID]
        if args.sweep_strategies
        else [Strategy(args.strategy)]
    )

    sweep_results = []
    for val in values:
        for strat_enum in strategies:
            # Override the swept parameter
            overrides = {param_name: val}
            algo, pool, quest, strat = make_params(args)

            # Apply override to the correct param group
            if param_name in ("exponent", "floor_weight", "neutral_base", "staleness_factor"):
                algo = AlgorithmParams(**{
                    **{f.name: getattr(algo, f.name) for f in algo.__dataclass_fields__.values()},
                    param_name: val,
                })
            elif param_name == "dreamcaller_bonus":
                quest = QuestParams(dreamcaller_bonus=int(val), mono_dreamcaller=quest.mono_dreamcaller)
            elif param_name in ("power_weight", "fit_weight"):
                strat = StrategyParams(
                    strategy=strat_enum,
                    power_weight=val if param_name == "power_weight" else strat.power_weight,
                    fit_weight=val if param_name == "fit_weight" else strat.fit_weight,
                )
            else:
                strat = StrategyParams(strategy=strat_enum, power_weight=strat.power_weight, fit_weight=strat.fit_weight)

            if param_name not in ("power_weight", "fit_weight"):
                strat = StrategyParams(strategy=strat_enum, power_weight=strat.power_weight, fit_weight=strat.fit_weight)

            rng = random.Random(seed)
            results = run_batch(algo, pool, quest, strat, args.runs, rng)
            sweep_results.append((f"{val:.2f}" if isinstance(val, float) else str(val), strat_enum.value, results))

    print_sweep(sweep_results, param_name)


def mode_evolution(args):
    algo, pool, quest, strat = make_params(args)
    seed = args.seed if args.seed is not None else 0
    rng = random.Random(seed)
    results = run_batch(algo, pool, quest, strat, args.runs, rng)
    print_evolution(results)


def mode_interactive(args):
    algo, pool, quest, strat = make_params(args)
    seed = args.seed if args.seed is not None else random.randint(0, 2**32)
    rng = random.Random(seed)
    result = simulate_quest(algo, pool, quest, strat, rng)
    run_interactive(result, strat.strategy.value, strat)


def main():
    parser = build_parser()
    args = parser.parse_args()

    dispatch = {
        "trace": mode_trace,
        "aggregate": mode_aggregate,
        "sweep": mode_sweep,
        "evolution": mode_evolution,
        "interactive": mode_interactive,
    }
    dispatch[args.mode](args)


if __name__ == "__main__":
    main()
