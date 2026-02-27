"""Quest simulation loop and player strategy implementations."""

import random

from models import (
    AlgorithmParams,
    PickContext,
    PickRecord,
    PoolParams,
    QuestParams,
    QuestResult,
    Rarity,
    Resonance,
    ResonanceProfile,
    SimCard,
    Strategy,
    StrategyParams,
)
from algorithm import generate_pool, select_cards, PoolEntry


def simulate_quest(
    algo_params: AlgorithmParams,
    pool_params: PoolParams,
    quest_params: QuestParams,
    strat_params: StrategyParams,
    rng: random.Random,
) -> QuestResult:
    """Simulate a full quest draft and return the result."""
    # Choose dreamcaller resonances
    all_res = list(Resonance)
    if quest_params.mono_dreamcaller:
        dc_res = frozenset([rng.choice(all_res)])
    else:
        pair = rng.sample(all_res, 2)
        dc_res = frozenset(pair)

    # Initialize profile with dreamcaller bonus
    profile = ResonanceProfile()
    for r in dc_res:
        profile.add(r, quest_params.dreamcaller_bonus)

    # Generate pool
    pool, pool_variance = generate_pool(pool_params, rng)

    picks: list[PickRecord] = []
    deck: list[SimCard] = []
    pick_number = 0
    shop_count = 0

    # 7 dreamscapes
    for dreamscape in range(7):
        # Draft sites: completion 0-1 = 2 sites, 2-3 = 1, 4+ = 0
        if dreamscape <= 1:
            num_draft_sites = 2
        elif dreamscape <= 3:
            num_draft_sites = 1
        else:
            num_draft_sites = 0

        for site_idx in range(num_draft_sites):
            is_shop = rng.random() < quest_params.shop_chance

            if is_shop:
                shop_count += 1
                # Shop: offer 6 cards, strategy decides which to buy
                offered = select_cards(pool, 6, profile, algo_params, rng)
                if not offered:
                    continue

                pick_number += 1
                bought_entries, buy_reasons = _shop_buy(
                    offered, profile, strat_params
                )

                # Update profile and pool for all bought cards
                for entry in bought_entries:
                    for r in entry.card.resonances:
                        profile.add(r)
                    _remove_from_pool(pool, entry)
                    deck.append(entry.card)

                # Staleness on cards not bought
                bought_set = set(id(e) for e in bought_entries)
                for entry, _ in offered:
                    if id(entry) not in bought_set:
                        entry.staleness += 1

                # Use first bought card as primary pick for compatibility
                primary = bought_entries[0] if bought_entries else offered[0][0]
                primary_reason = buy_reasons[0] if buy_reasons else "no buy"

                picks.append(PickRecord(
                    pick_number=pick_number,
                    context=PickContext(
                        dreamscape=dreamscape,
                        site=site_idx,
                        position=None,
                        is_battle_reward=False,
                        is_shop=True,
                    ),
                    offered=[e.card for e, _ in offered],
                    weights=[w for _, w in offered],
                    picked=primary.card if bought_entries else offered[0][0].card,
                    pick_reason=primary_reason,
                    profile_after=profile.snapshot(),
                    bought=[e.card for e in bought_entries],
                    buy_reasons=buy_reasons,
                ))

            else:
                # Draft site: 5 picks of 4 cards
                for pos in range(5):
                    offered = select_cards(pool, 4, profile, algo_params, rng)
                    if not offered:
                        continue

                    pick_number += 1
                    picked_entry, picked_weight = _pick_card(
                        offered, profile, strat_params, rng
                    )
                    reason = _pick_reason(
                        picked_entry.card, offered, profile, strat_params
                    )

                    # Update profile
                    for r in picked_entry.card.resonances:
                        profile.add(r)

                    # Update staleness on unpicked cards
                    for entry, _ in offered:
                        if entry is not picked_entry:
                            entry.staleness += 1

                    # Remove picked card from pool
                    _remove_from_pool(pool, picked_entry)
                    deck.append(picked_entry.card)

                    picks.append(PickRecord(
                        pick_number=pick_number,
                        context=PickContext(
                            dreamscape=dreamscape,
                            site=site_idx,
                            position=pos,
                            is_battle_reward=False,
                            is_shop=False,
                        ),
                        offered=[e.card for e, _ in offered],
                        weights=[w for _, w in offered],
                        picked=picked_entry.card,
                        pick_reason=reason,
                        profile_after=profile.snapshot(),
                    ))

        # Battle reward: 1 rare+ pick per dreamscape
        rare_offered = select_cards(
            pool, 3, profile, algo_params, rng, rare_only=True
        )
        if rare_offered:
            pick_number += 1
            picked_entry, picked_weight = _pick_card(
                rare_offered, profile, strat_params, rng
            )
            reason = _pick_reason(
                picked_entry.card, rare_offered, profile, strat_params
            )

            for r in picked_entry.card.resonances:
                profile.add(r)
            for entry, _ in rare_offered:
                if entry is not picked_entry:
                    entry.staleness += 1
            _remove_from_pool(pool, picked_entry)
            deck.append(picked_entry.card)

            picks.append(PickRecord(
                pick_number=pick_number,
                context=PickContext(
                    dreamscape=dreamscape,
                    site=None,
                    position=None,
                    is_battle_reward=True,
                    is_shop=False,
                ),
                offered=[e.card for e, _ in rare_offered],
                weights=[w for _, w in rare_offered],
                picked=picked_entry.card,
                pick_reason=reason + " [battle reward]",
                profile_after=profile.snapshot(),
            ))

        # Decay staleness at end of dreamscape
        for entry in pool:
            entry.staleness = max(0, entry.staleness - 1)

    return QuestResult(
        picks=picks,
        final_profile=profile,
        deck=deck,
        dreamcaller_resonances=dc_res,
        pool_variance=pool_variance,
        shop_count=shop_count,
    )


def _shop_buy(
    offered: list[tuple[PoolEntry, float]],
    profile: ResonanceProfile,
    strat: StrategyParams,
) -> tuple[list[PoolEntry], list[str]]:
    """Decide which shop cards to buy based on strategy.

    Returns (bought_entries, buy_reasons).
    """
    top2_res = {r for r, c in profile.top_n(2) if c > 0}
    bought: list[PoolEntry] = []
    reasons: list[str] = []

    for entry, weight in offered:
        card = entry.card
        fit = _resonance_fit(card, top2_res)
        res_str = "+".join(
            r.value for r in sorted(card.resonances, key=lambda r: r.value)
        )
        if not res_str:
            res_str = "neutral"

        if strat.strategy == Strategy.POWER_CHASER:
            # Buy everything
            bought.append(entry)
            reasons.append(f"buy power={card.power} ({res_str})")

        elif strat.strategy == Strategy.RIGID:
            # Buy only fully on-color or neutral
            if not card.resonances or card.resonances <= top2_res:
                bought.append(entry)
                reasons.append(f"buy on-color power={card.power} ({res_str})")

        else:
            # Synergy: buy if fit >= 0.5 (partial match, full match, or neutral)
            if fit >= 0.5:
                score = (
                    card.power * strat.power_weight
                    + fit * strat.fit_weight * 10
                )
                bought.append(entry)
                reasons.append(
                    f"buy score={score:.1f} power={card.power} "
                    f"fit={fit:.1f} ({res_str})"
                )

    return bought, reasons


def _pick_card(
    offered: list[tuple[PoolEntry, float]],
    profile: ResonanceProfile,
    strat: StrategyParams,
    rng: random.Random,
) -> tuple[PoolEntry, float]:
    """Choose a card based on the player strategy."""
    if strat.strategy == Strategy.POWER_CHASER:
        return max(offered, key=lambda x: x[0].card.power)

    if strat.strategy == Strategy.RIGID:
        top2_res = {r for r, c in profile.top_n(2) if c > 0}
        on_color = [
            (e, w) for e, w in offered
            if e.card.resonances and e.card.resonances <= top2_res
        ]
        if on_color:
            return max(on_color, key=lambda x: x[0].card.power)
        # Accept neutrals before truly off-color
        neutrals = [(e, w) for e, w in offered if not e.card.resonances]
        if neutrals:
            return max(neutrals, key=lambda x: x[0].card.power)
        # Last resort: highest power
        return max(offered, key=lambda x: x[0].card.power)

    # Synergy strategy
    top2_res = {r for r, c in profile.top_n(2) if c > 0}
    best_score = -1.0
    best = offered[0]
    for entry, weight in offered:
        fit = _resonance_fit(entry.card, top2_res)
        score = entry.card.power * strat.power_weight + fit * strat.fit_weight * 10
        if score > best_score:
            best_score = score
            best = (entry, weight)
    return best


def _resonance_fit(card: SimCard, top2: set[Resonance]) -> float:
    """Fraction of card's resonances matching player's top-2."""
    if not card.resonances:
        return 0.5  # Neutrals get 0.5 fit
    matching = len(card.resonances & top2)
    return matching / len(card.resonances)


def _pick_reason(
    card: SimCard,
    offered: list[tuple[PoolEntry, float]],
    profile: ResonanceProfile,
    strat: StrategyParams,
) -> str:
    """Generate a human-readable reason for a pick."""
    res_str = "+".join(r.value for r in sorted(card.resonances, key=lambda r: r.value))
    if not res_str:
        res_str = "neutral"

    if strat.strategy == Strategy.POWER_CHASER:
        return f"power={card.power} ({res_str})"
    if strat.strategy == Strategy.RIGID:
        return f"on-color power={card.power} ({res_str})"

    top2 = {r for r, c in profile.top_n(2) if c > 0}
    fit = _resonance_fit(card, top2)
    score = card.power * strat.power_weight + fit * strat.fit_weight * 10
    return f"score={score:.1f} power={card.power} fit={fit:.1f} ({res_str})"


def _remove_from_pool(pool: list[PoolEntry], entry: PoolEntry):
    """Remove a specific pool entry."""
    for i, e in enumerate(pool):
        if e is entry:
            pool.pop(i)
            return
