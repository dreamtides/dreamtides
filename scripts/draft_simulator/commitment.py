"""Commitment detection for draft agent preference vectors.

Identifies when a seat locks into an archetype during the draft by
analyzing the history of preference vectors. Supports both
concentration-based (primary) and entropy-based (secondary) detection
methods. Stdlib-only, no external dependencies.
"""

import math
from dataclasses import dataclass
from typing import Optional

from config import CommitmentConfig


@dataclass(frozen=True)
class CommitmentResult:
    """Result of commitment detection for a single seat."""

    commitment_pick: Optional[int]
    committed_archetype: Optional[int]
    entropy_commitment_pick: Optional[int]
    entropy_committed_archetype: Optional[int]


def detect_commitment(
    w_history: list[list[float]],
    commitment_cfg: CommitmentConfig,
) -> CommitmentResult:
    """Detect the commitment pick from a history of preference vectors.

    Analyzes both concentration-based (primary) and entropy-based
    (secondary) commitment. Returns a CommitmentResult with both.
    """
    primary_pick, primary_arch = _detect_concentration_commitment(
        w_history,
        commitment_cfg.commitment_threshold,
        commitment_cfg.stability_window,
    )
    entropy_pick, entropy_arch = _detect_entropy_commitment(
        w_history,
        commitment_cfg.entropy_threshold,
        commitment_cfg.stability_window,
    )
    return CommitmentResult(
        commitment_pick=primary_pick,
        committed_archetype=primary_arch,
        entropy_commitment_pick=entropy_pick,
        entropy_committed_archetype=entropy_arch,
    )


def update_preference_vector(
    w: list[float],
    card_fitness: list[float],
    learning_rate: float,
) -> list[float]:
    """Update a preference vector after picking a card.

    Adds learning_rate * fitness[a] to w[a] for each archetype a.
    The vector is NOT renormalized. Returns a new list (does not
    mutate the input).
    """
    return [w[a] + learning_rate * card_fitness[a] for a in range(len(w))]


def initial_preference_vector(archetype_count: int) -> list[float]:
    """Create the initial uniform preference vector."""
    return [1.0 / archetype_count] * archetype_count


def concentration(w: list[float]) -> float:
    """Compute concentration C(w) = max(w) / sum(w)."""
    total = sum(w)
    if total <= 0.0:
        return 0.0
    return max(w) / total


def shannon_entropy(w: list[float]) -> float:
    """Compute Shannon entropy H(normalized_w) in bits."""
    total = sum(w)
    if total <= 0.0:
        return 0.0

    entropy = 0.0
    for value in w:
        p = value / total
        if p > 0.0:
            entropy -= p * math.log2(p)
    return entropy


def _detect_concentration_commitment(
    w_history: list[list[float]],
    threshold: float,
    stability_window: int,
) -> tuple[Optional[int], Optional[int]]:
    """Detect commitment using the concentration-based method.

    A pick qualifies if concentration >= threshold and argmax(w) stays
    the same with concentration remaining above threshold for the next
    stability_window consecutive picks.
    """
    num_picks = len(w_history)
    if num_picks == 0:
        return (None, None)

    for i in range(num_picks):
        w = w_history[i]
        if concentration(w) < threshold:
            continue

        arch = _argmax(w)
        stable = True

        for j in range(1, stability_window + 1):
            if i + j >= num_picks:
                stable = False
                break
            w_next = w_history[i + j]
            if concentration(w_next) < threshold:
                stable = False
                break
            if _argmax(w_next) != arch:
                stable = False
                break

        if stable:
            return (i, arch)

    return (None, None)


def _detect_entropy_commitment(
    w_history: list[list[float]],
    entropy_threshold: float,
    stability_window: int,
) -> tuple[Optional[int], Optional[int]]:
    """Detect commitment using the entropy-based method.

    A pick qualifies if Shannon entropy < entropy_threshold and argmax(w)
    stays the same with entropy remaining below threshold for the next
    stability_window consecutive picks.
    """
    num_picks = len(w_history)
    if num_picks == 0:
        return (None, None)

    for i in range(num_picks):
        w = w_history[i]
        if shannon_entropy(w) >= entropy_threshold:
            continue

        arch = _argmax(w)
        stable = True

        for j in range(1, stability_window + 1):
            if i + j >= num_picks:
                stable = False
                break
            w_next = w_history[i + j]
            if shannon_entropy(w_next) >= entropy_threshold:
                stable = False
                break
            if _argmax(w_next) != arch:
                stable = False
                break

        if stable:
            return (i, arch)

    return (None, None)


def _argmax(values: list[float]) -> int:
    """Return the index of the maximum value."""
    best_index = 0
    best_value = values[0]
    for i in range(1, len(values)):
        if values[i] > best_value:
            best_value = values[i]
            best_index = i
    return best_index
