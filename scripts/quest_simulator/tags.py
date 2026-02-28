"""Tag system for theme selection in discovery drafts and specialty shops.

Provides weighted theme selection from a player's tag profile and the
current draft pool, plus pool filtering by selected tag.
"""

import math
import random
from typing import Optional

from models import PoolEntry, TagProfile


def compute_theme_score(
    tag_count: int,
    pool_depth: int,
    tag_scale: float,
    relevance_boost: float,
    depth_factor: float,
) -> float:
    """Compute the theme score for a single tag.

    Score formula: tag_affinity * relevance_boost + pool_depth * depth_factor
    where tag_affinity = ln(1 + tag_count) * tag_scale.
    """
    tag_affinity = math.log(1 + tag_count) * tag_scale
    return tag_affinity * relevance_boost + pool_depth * depth_factor


def select_theme(
    pool: list[PoolEntry],
    profile: TagProfile,
    rng: random.Random,
    min_theme_cards: int,
    tag_scale: float,
    relevance_boost: float,
    depth_factor: float,
) -> Optional[str]:
    """Select a theme tag via weighted random selection.

    Filters to tags with at least min_theme_cards matching pool entries,
    scores each by affinity and pool depth, then selects via weighted
    random. Falls back to uniform random when no tags have affinity
    (cold start). Returns None if no tags are eligible.
    """
    tag_depths: dict[str, int] = {}
    for entry in pool:
        for tag in sorted(entry.card.tags):
            tag_depths[tag] = tag_depths.get(tag, 0) + 1

    eligible_tags = sorted(
        tag for tag, depth in tag_depths.items() if depth >= min_theme_cards
    )

    if not eligible_tags:
        return None

    counts = profile.snapshot()
    has_affinity = any(counts.get(tag, 0) > 0 for tag in eligible_tags)

    if not has_affinity:
        return rng.choice(eligible_tags)

    scores = [
        compute_theme_score(
            tag_count=counts.get(tag, 0),
            pool_depth=tag_depths[tag],
            tag_scale=tag_scale,
            relevance_boost=relevance_boost,
            depth_factor=depth_factor,
        )
        for tag in eligible_tags
    ]

    if sum(scores) <= 0:
        return rng.choice(eligible_tags)

    selected = rng.choices(eligible_tags, weights=scores, k=1)
    return selected[0]


def filter_pool_by_tag(pool: list[PoolEntry], tag: str) -> list[PoolEntry]:
    """Return pool entries whose card has the given tag."""
    return [entry for entry in pool if tag in entry.card.tags]
