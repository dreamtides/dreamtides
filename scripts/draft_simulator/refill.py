"""Refill strategies for the draft simulator.

Implements three strategies for adding cards to packs during draft rotation:
NoRefill (no card added), UniformRefill (one card sampled uniformly), and
ConstrainedRefill (one card sampled with weights conditioned on a signal
vector). Stdlib-only, no external dependencies.
"""

import math
import random

import cube_manager
from draft_models import CardInstance, Pack


def no_refill() -> None:
    """No-refill strategy: no card is added to the pack."""
    return None


def uniform_refill(
    cube: cube_manager.CubeManager,
    rng: random.Random,
) -> CardInstance:
    """Uniform refill: draw one card uniformly from the cube supply."""
    drawn = cube.draw(1, rng)
    return drawn[0]


def constrained_refill(
    cube: cube_manager.CubeManager,
    signal: list[float],
    fidelity: float,
    commit_bias: float,
    rng: random.Random,
) -> CardInstance:
    """Constrained refill: draw one card weighted by cosine similarity to signal.

    Computes per-card weights based on cosine similarity between each
    candidate's fitness vector and the signal vector, modulated by
    fidelity and commit_bias parameters.
    """
    supply = cube.supply
    weights = compute_constrained_weights(supply, signal, fidelity, commit_bias)
    drawn = cube.draw(1, rng, weights=weights)
    return drawn[0]


def compute_constrained_weights(
    candidates: list[CardInstance],
    signal: list[float],
    fidelity: float,
    commit_bias: float,
) -> list[float]:
    """Compute sampling weights for constrained refill.

    For each candidate card c:
        similarity = dot(c.fitness, signal) / (norm(c.fitness) * norm(signal) + eps)
        weight = (1 - fidelity) + fidelity * similarity
        weight *= (1 - commit_bias) + commit_bias * c.commit
    """
    eps = 1e-10
    signal_norm = _l2_norm(signal)
    weights: list[float] = []

    for inst in candidates:
        fitness = inst.design.fitness
        dot = _dot_product(fitness, signal)
        fitness_norm = _l2_norm(fitness)
        similarity = dot / (fitness_norm * signal_norm + eps)
        weight = (1.0 - fidelity) + fidelity * similarity
        if fidelity > 0.0:
            weight *= (1.0 - commit_bias) + commit_bias * inst.design.commit
        weights.append(max(weight, 0.001))

    return weights


def cosine_similarity(a: list[float], b: list[float]) -> float:
    """Compute cosine similarity between two vectors."""
    eps = 1e-10
    dot = _dot_product(a, b)
    norm_a = _l2_norm(a)
    norm_b = _l2_norm(b)
    return dot / (norm_a * norm_b + eps)


def compute_round_environment_profile(packs: list[Pack]) -> list[float]:
    """Compute mean archetype profile across all packs in a round.

    Returns the element-wise mean of all pack archetype profiles,
    used as the signal vector when fingerprint_source is round_environment.
    """
    if not packs:
        return []

    archetype_count = len(packs[0].archetype_profile)
    profile = [0.0] * archetype_count

    for pack in packs:
        for i in range(archetype_count):
            profile[i] += pack.archetype_profile[i]

    n = len(packs)
    return [v / n for v in profile]


def _dot_product(a: list[float], b: list[float]) -> float:
    """Compute dot product of two vectors."""
    total = 0.0
    for i in range(min(len(a), len(b))):
        total += a[i] * b[i]
    return total


def _l2_norm(v: list[float]) -> float:
    """Compute L2 norm of a vector."""
    return math.sqrt(sum(x * x for x in v))
