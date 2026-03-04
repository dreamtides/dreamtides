"""Agent state and pick policies for the draft simulator.

Each agent maintains a preference vector, drafted pool, pick history,
and openness estimates. Five pick policies determine how agents select
cards from packs: Greedy, Archetype-loyal, Force, Adaptive, and
Signal-ignorant. AI noise is controlled via epsilon-greedy.
Stdlib-only, no external dependencies.
"""

import random
from dataclasses import dataclass, field
from typing import Optional

import commitment
import deck_scorer
from config import AgentsConfig, ScoringConfig
from draft_models import CardInstance


@dataclass
class AgentState:
    """Mutable state maintained by a drafting agent."""

    w: list[float]
    drafted: list[CardInstance] = field(default_factory=list)
    pick_history: list[tuple[int, int, str, str]] = field(default_factory=list)
    openness: list[float] = field(default_factory=list)
    openness_window: list[list[float]] = field(default_factory=list)


def create_agent(archetype_count: int) -> AgentState:
    """Create a fresh agent with uniform preference and openness vectors."""
    w = commitment.initial_preference_vector(archetype_count)
    openness = commitment.initial_preference_vector(archetype_count)
    return AgentState(w=w, openness=openness)


def update_agent_after_pick(
    agent: AgentState,
    card: CardInstance,
    pack_cards: list[CardInstance],
    pick_index: int,
    round_index: int,
    pack_id: str,
    learning_rate: float,
    openness_window_size: int,
) -> None:
    """Update agent state after a pick.

    Updates the preference vector additively, appends to drafted pool
    and pick history, and recomputes the openness estimate from the
    rolling window of recent pack supply signals.
    """
    agent.w = commitment.update_preference_vector(
        agent.w, card.design.fitness, learning_rate
    )
    agent.drafted.append(card)
    agent.pick_history.append((pick_index, round_index, card.design.card_id, pack_id))

    # Compute supply signal from this pack
    supply_signal = _compute_supply_signal(pack_cards, len(agent.w))
    agent.openness_window.append(supply_signal)

    # Trim window to configured size
    while len(agent.openness_window) > openness_window_size:
        agent.openness_window.pop(0)

    # Update openness as mean of supply signals in window
    agent.openness = _compute_openness(agent.openness_window, len(agent.w))


def pick_card(
    candidates: list[CardInstance],
    agent: AgentState,
    policy: str,
    agents_cfg: AgentsConfig,
    scoring_cfg: ScoringConfig,
    rng: random.Random,
    force_archetype: Optional[int] = None,
) -> CardInstance:
    """Select a card from candidates using the specified policy.

    Applies epsilon-greedy noise: with probability (1 - ai_optimality),
    picks a random card instead of using the policy's scoring.
    """
    if not candidates:
        raise ValueError("Cannot pick from an empty candidate set")

    # Epsilon-greedy noise
    if rng.random() >= agents_cfg.ai_optimality:
        return candidates[rng.randrange(len(candidates))]

    if policy == "greedy":
        return _pick_greedy(candidates, agent, scoring_cfg)
    elif policy == "archetype_loyal":
        return _pick_archetype_loyal(candidates, agent)
    elif policy == "force":
        return _pick_force(candidates, force_archetype)
    elif policy == "adaptive":
        return _pick_adaptive(candidates, agent, agents_cfg)
    elif policy == "signal_ignorant":
        return _pick_signal_ignorant(candidates, agent, agents_cfg)
    else:
        raise ValueError(f"Unknown pick policy: {policy!r}")


def score_card_greedy(
    card: CardInstance,
    agent: AgentState,
    scoring_cfg: ScoringConfig,
) -> float:
    """Score a card using the Greedy policy (deck_value with card added)."""
    trial_pool: list[CardInstance] = list(agent.drafted) + [card]
    return deck_scorer.deck_value(trial_pool, agent.w, scoring_cfg)


def score_card_adaptive(
    card: CardInstance,
    agent: AgentState,
    agents_cfg: AgentsConfig,
) -> float:
    """Score a card using the Adaptive formula.

    Score = alpha * power + beta * dot(fitness, normalize(w))
          + gamma * dot(fitness, openness).
    """
    power_weight = agents_cfg.ai_power_weight
    pref_weight = agents_cfg.ai_pref_weight
    signal_weight = agents_cfg.ai_signal_weight

    fitness = card.design.fitness
    w_norm = _normalize(agent.w)

    power_term = power_weight * card.design.power
    pref_term = pref_weight * _dot(fitness, w_norm)
    signal_term = signal_weight * _dot(fitness, agent.openness)

    return power_term + pref_term + signal_term


def score_card_signal_ignorant(
    card: CardInstance,
    agent: AgentState,
    agents_cfg: AgentsConfig,
) -> float:
    """Score a card using the Signal-ignorant formula.

    Identical to Adaptive but uses a uniform openness vector instead
    of the agent's actual openness estimate.
    """
    power_weight = agents_cfg.ai_power_weight
    pref_weight = agents_cfg.ai_pref_weight
    signal_weight = agents_cfg.ai_signal_weight

    fitness = card.design.fitness
    w_norm = _normalize(agent.w)
    uniform_openness = commitment.initial_preference_vector(len(agent.w))

    power_term = power_weight * card.design.power
    pref_term = pref_weight * _dot(fitness, w_norm)
    signal_term = signal_weight * _dot(fitness, uniform_openness)

    return power_term + pref_term + signal_term


def _pick_greedy(
    candidates: list[CardInstance],
    agent: AgentState,
    scoring_cfg: ScoringConfig,
) -> CardInstance:
    """Pick the card that maximizes deck_value(current_pool + [candidate])."""
    best_card = candidates[0]
    best_score = score_card_greedy(candidates[0], agent, scoring_cfg)

    for card in candidates[1:]:
        score = score_card_greedy(card, agent, scoring_cfg)
        if score > best_score:
            best_score = score
            best_card = card

    return best_card


def _pick_archetype_loyal(
    candidates: list[CardInstance],
    agent: AgentState,
) -> CardInstance:
    """Pick the card with highest fitness for argmax(w), break ties by power."""
    from utils import argmax

    best_arch = argmax(agent.w)
    best_card = candidates[0]
    best_fitness = candidates[0].design.fitness[best_arch]
    best_power = candidates[0].design.power

    for card in candidates[1:]:
        fitness = card.design.fitness[best_arch]
        power = card.design.power
        if fitness > best_fitness or (fitness == best_fitness and power > best_power):
            best_card = card
            best_fitness = fitness
            best_power = power

    return best_card


def _pick_force(
    candidates: list[CardInstance],
    force_archetype: Optional[int],
) -> CardInstance:
    """Pick the card with highest fitness for the target archetype.

    Always uses the fixed target archetype regardless of w.
    Breaks ties by power.
    """
    if force_archetype is None:
        raise ValueError("Force policy requires a target archetype index")

    archetype_count = len(candidates[0].design.fitness)
    if force_archetype < 0 or force_archetype >= archetype_count:
        raise ValueError(
            f"force_archetype={force_archetype} out of range " f"[0, {archetype_count})"
        )

    best_card = candidates[0]
    best_fitness = candidates[0].design.fitness[force_archetype]
    best_power = candidates[0].design.power

    for card in candidates[1:]:
        fitness = card.design.fitness[force_archetype]
        power = card.design.power
        if fitness > best_fitness or (fitness == best_fitness and power > best_power):
            best_card = card
            best_fitness = fitness
            best_power = power

    return best_card


def _pick_adaptive(
    candidates: list[CardInstance],
    agent: AgentState,
    agents_cfg: AgentsConfig,
) -> CardInstance:
    """Pick the card with the highest Adaptive score."""
    best_card = candidates[0]
    best_score = score_card_adaptive(candidates[0], agent, agents_cfg)

    for card in candidates[1:]:
        score = score_card_adaptive(card, agent, agents_cfg)
        if score > best_score:
            best_score = score
            best_card = card

    return best_card


def _pick_signal_ignorant(
    candidates: list[CardInstance],
    agent: AgentState,
    agents_cfg: AgentsConfig,
) -> CardInstance:
    """Pick the card with the highest Signal-ignorant score."""
    best_card = candidates[0]
    best_score = score_card_signal_ignorant(candidates[0], agent, agents_cfg)

    for card in candidates[1:]:
        score = score_card_signal_ignorant(card, agent, agents_cfg)
        if score > best_score:
            best_score = score
            best_card = card

    return best_card


def _compute_supply_signal(
    pack_cards: list[CardInstance],
    archetype_count: int,
) -> list[float]:
    """Compute the per-archetype supply signal from a pack.

    Supply signal for archetype a is the mean fitness-for-archetype-a
    of all cards in the pack.
    """
    if not pack_cards:
        return [0.0] * archetype_count

    signal = [0.0] * archetype_count
    for card in pack_cards:
        for a in range(min(len(card.design.fitness), archetype_count)):
            signal[a] += card.design.fitness[a]

    n = len(pack_cards)
    return [s / n for s in signal]


def _compute_openness(
    window: list[list[float]],
    archetype_count: int,
) -> list[float]:
    """Compute openness estimates as mean of supply signals in the window."""
    if not window:
        return commitment.initial_preference_vector(archetype_count)

    openness = [0.0] * archetype_count
    for signal in window:
        for a in range(min(len(signal), archetype_count)):
            openness[a] += signal[a]

    n = len(window)
    return [o / n for o in openness]


def _normalize(w: list[float]) -> list[float]:
    """Normalize a vector so its elements sum to 1.0."""
    total = sum(w)
    if total <= 0.0:
        return [1.0 / len(w)] * len(w) if w else []
    return [v / total for v in w]


def _dot(a: list[float], b: list[float]) -> float:
    """Compute the dot product of two vectors of equal length."""
    return sum(x * y for x, y in zip(a, b))
