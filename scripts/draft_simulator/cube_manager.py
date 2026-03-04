"""Cube management for the draft simulator.

Manages the drawable card supply with support for both with-replacement
and without-replacement consumption modes. Creates physical card instances
from card designs and provides weighted and uniform draw operations.
Stdlib-only, no external dependencies.
"""

import random

import config
from draft_models import CardDesign, CardInstance, CubeConsumptionMode


class CubeSupplyError(Exception):
    """Raised when the cube supply is exhausted or insufficient."""


class CubeManager:
    """Manages the master card pool for drafting.

    Tracks available card instances and supports drawing cards with
    or without replacement. All RNG operations require an explicit
    random.Random parameter.
    """

    def __init__(
        self,
        designs: list[CardDesign],
        copies_per_card: int | dict[str, int],
        consumption_mode: CubeConsumptionMode,
    ) -> None:
        self._consumption_mode: CubeConsumptionMode = consumption_mode
        self._all_instances: list[CardInstance] = _create_instances(
            designs, copies_per_card
        )
        self._supply: list[CardInstance] = list(self._all_instances)

    @property
    def supply(self) -> list[CardInstance]:
        """Read-only snapshot of the current supply for external queries."""
        return list(self._supply)

    @property
    def remaining(self) -> int:
        """Number of card instances currently available in the supply."""
        return len(self._supply)

    @property
    def total_size(self) -> int:
        """Total number of card instances created for this cube."""
        return len(self._all_instances)

    def draw(
        self,
        n: int,
        rng: random.Random,
        weights: list[float] | None = None,
    ) -> list[CardInstance]:
        """Draw n card instances from the supply.

        When weights is provided, it must be parallel to the current
        supply list (same length, same order). Sampling is weighted
        without replacement. When weights is None, sampling is uniform.

        In without_replacement mode, drawn cards are removed from the
        supply. In with_replacement mode, the supply is not depleted.
        """
        if n <= 0:
            return []

        available = len(self._supply)
        if available == 0:
            raise CubeSupplyError("Cube supply is empty, cannot draw")

        if n > available:
            raise CubeSupplyError(
                f"Cannot draw {n} cards: only {available} remain in supply"
            )

        if weights is not None and len(weights) != available:
            raise ValueError(
                f"weights length {len(weights)} != supply size {available}"
            )

        if weights is not None:
            drawn = _weighted_sample_without_replacement(self._supply, weights, n, rng)
        else:
            drawn = _uniform_sample_without_replacement(self._supply, n, rng)

        if self._consumption_mode == CubeConsumptionMode.WITHOUT_REPLACEMENT:
            drawn_ids = {inst.instance_id for inst in drawn}
            self._supply = [
                inst for inst in self._supply if inst.instance_id not in drawn_ids
            ]

        return drawn


def validate_supply(cfg: config.SimulatorConfig, cube_size: int) -> None:
    """Pre-validate that cube supply is sufficient for the draft's demand.

    Computes max_demand = seat_count * pack_size * round_count + total_refills.
    When refill is active, total_refills = seat_count * sum(picks_per_round)
    (each pick causes one refill card per pack). When refill is no_refill,
    total_refills = 0.

    Raises CubeSupplyError with a diagnostic message if max_demand > cube_size
    and consumption mode is without_replacement.
    """
    if cfg.cube.consumption_mode != "without_replacement":
        return

    seat_count = cfg.draft.seat_count
    pack_size = cfg.draft.pack_size
    round_count = cfg.draft.round_count
    picks_per_round = cfg.draft.picks_per_round

    base_demand = seat_count * pack_size * round_count

    if cfg.refill.strategy != "no_refill":
        total_refills = seat_count * sum(picks_per_round)
    else:
        total_refills = 0

    max_demand = base_demand + total_refills

    if max_demand > cube_size:
        raise CubeSupplyError(
            f"Cube too small: demand={max_demand} exceeds supply={cube_size}. "
            f"(base={seat_count}*{pack_size}*{round_count}={base_demand}, "
            f"refills={total_refills})"
        )


def _create_instances(
    designs: list[CardDesign], copies_per_card: int | dict[str, int]
) -> list[CardInstance]:
    """Create CardInstance objects with unique instance_ids.

    When copies_per_card is an int, every design gets that many copies.
    When it is a dict mapping card_id to int, each design gets its
    per-card count (defaulting to 1 for designs not in the map).
    """
    instances: list[CardInstance] = []
    instance_id = 0
    for design in designs:
        if isinstance(copies_per_card, dict):
            count = copies_per_card.get(design.card_id, 1)
        else:
            count = copies_per_card
        for _ in range(count):
            instances.append(CardInstance(instance_id=instance_id, design=design))
            instance_id += 1
    return instances


def _uniform_sample_without_replacement(
    pool: list[CardInstance],
    n: int,
    rng: random.Random,
) -> list[CardInstance]:
    """Sample n items uniformly without replacement from pool."""
    indices = list(range(len(pool)))
    selected: list[CardInstance] = []

    for _ in range(min(n, len(pool))):
        pos = rng.randrange(len(indices))
        chosen_idx = indices[pos]
        selected.append(pool[chosen_idx])
        indices[pos] = indices[-1]
        indices.pop()

    return selected


def _weighted_sample_without_replacement(
    pool: list[CardInstance],
    weights: list[float],
    n: int,
    rng: random.Random,
) -> list[CardInstance]:
    """Sample n items via weighted sampling without replacement.

    Follows the pattern from quest_simulator/algorithm.py: maintain
    a list of active indices and cumulative weight selection.
    """
    selected: list[CardInstance] = []
    indices = list(range(len(pool)))
    remaining_weights = list(weights)

    for _ in range(min(n, len(pool))):
        total = sum(remaining_weights[i] for i in indices)
        if total <= 0:
            break
        r = rng.uniform(0, total)
        cumulative = 0.0
        chosen_pos = 0
        for pos, idx in enumerate(indices):
            cumulative += remaining_weights[idx]
            if cumulative >= r:
                chosen_pos = pos
                break
        chosen_idx = indices[chosen_pos]
        selected.append(pool[chosen_idx])
        indices.pop(chosen_pos)

    return selected
