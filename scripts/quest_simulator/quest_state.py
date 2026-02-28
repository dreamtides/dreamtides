"""Mutable quest state management.

Tracks the player's deck, dreamsigns, dreamcaller, essence, resonance
and tag profiles, draft pool, and completion level. All site interactions
mutate state through this module's methods.
"""

import random
from typing import Optional

from models import (
    Card,
    DeckCard,
    Dreamcaller,
    Dreamsign,
    PoolEntry,
    Rarity,
    Resonance,
    ResonanceProfile,
    TagProfile,
)


class QuestState:
    """Central mutable state object for a quest run."""

    def __init__(
        self,
        essence: int,
        pool: list[PoolEntry],
        rng: random.Random,
        all_cards: list[Card],
        pool_variance: dict[Resonance, float],
        max_deck: int = 50,
        min_deck: int = 25,
        max_dreamsigns: int = 12,
    ) -> None:
        self.deck: list[DeckCard] = []
        self.dreamsigns: list[Dreamsign] = []
        self.dreamcaller: Optional[Dreamcaller] = None
        self.essence: int = essence
        self.resonance_profile: ResonanceProfile = ResonanceProfile()
        self.tag_profile: TagProfile = TagProfile()
        self.pool: list[PoolEntry] = pool
        self.completion_level: int = 0
        self.rng: random.Random = rng
        self.all_cards: list[Card] = all_cards
        self.pool_variance: dict[Resonance, float] = pool_variance
        self.max_deck: int = max_deck
        self.min_deck: int = min_deck
        self.max_dreamsigns: int = max_dreamsigns

    def add_card(self, card: Card) -> None:
        """Add a card to the deck and update resonance and tag profiles."""
        self.deck.append(DeckCard(card=card))
        for r in card.resonances:
            self.resonance_profile.add(r)
        for t in card.tags:
            self.tag_profile.add(t)

    def add_bane_card(self, card: Card) -> None:
        """Add a bane card to the deck. Bane cards do not affect profiles."""
        self.deck.append(DeckCard(card=card, is_bane=True))

    def remove_card(self, deck_card: DeckCard) -> None:
        """Remove a card from the deck and update resonance and tag profiles."""
        self.deck.remove(deck_card)
        if not deck_card.is_bane:
            for r in deck_card.card.resonances:
                self.resonance_profile.remove(r)
            for t in deck_card.card.tags:
                self.tag_profile.remove(t)

    def set_dreamcaller(self, dreamcaller: Dreamcaller) -> None:
        """Set the dreamcaller and apply resonance, tag, and essence bonuses.

        If a dreamcaller is already set, its bonuses are removed first so that
        calling this method is idempotent with respect to profile and essence
        state.
        """
        if self.dreamcaller is not None:
            old = self.dreamcaller
            for resonance_name, amount in old.resonance_bonus.items():
                self.resonance_profile.remove(Resonance(resonance_name), amount)
            for tag, amount in old.tag_bonus.items():
                self.tag_profile.remove(tag, amount)
            self.essence -= old.essence_bonus
        self.dreamcaller = dreamcaller
        for resonance_name, amount in dreamcaller.resonance_bonus.items():
            self.resonance_profile.add(Resonance(resonance_name), amount)
        for tag, amount in dreamcaller.tag_bonus.items():
            self.tag_profile.add(tag, amount)
        self.essence += dreamcaller.essence_bonus

    def add_dreamsign(self, dreamsign: Dreamsign) -> None:
        """Add a dreamsign and update resonance and tag profiles."""
        self.dreamsigns.append(dreamsign)
        self.resonance_profile.add(dreamsign.resonance)
        for t in dreamsign.tags:
            self.tag_profile.add(t)

    def remove_dreamsign(self, dreamsign: Dreamsign) -> None:
        """Remove a dreamsign and update resonance and tag profiles."""
        self.dreamsigns.remove(dreamsign)
        self.resonance_profile.remove(dreamsign.resonance)
        for t in dreamsign.tags:
            self.tag_profile.remove(t)

    def spend_essence(self, amount: int) -> None:
        """Subtract from essence. Raises ValueError if balance would go negative."""
        if self.essence < amount:
            raise ValueError(
                f"Cannot spend {amount} essence with only {self.essence} available"
            )
        self.essence -= amount

    def gain_essence(self, amount: int) -> None:
        """Add to essence."""
        self.essence += amount

    def increment_completion(self) -> None:
        """Increment the completion level by 1."""
        self.completion_level += 1

    def is_over_deck_limit(self) -> bool:
        """Return True if deck exceeds the maximum size."""
        return len(self.deck) > self.max_deck

    def is_under_deck_limit(self) -> bool:
        """Return True if deck is below the minimum size."""
        return len(self.deck) < self.min_deck

    def is_over_dreamsign_limit(self) -> bool:
        """Return True if dreamsigns are at or above the maximum count."""
        return len(self.dreamsigns) >= self.max_dreamsigns

    def auto_fill_deck(self) -> None:
        """Duplicate the whole deck repeatedly until deck count exceeds minimum."""
        if not self.deck:
            return
        original_cards = [dc.card for dc in self.deck]
        while len(self.deck) <= self.min_deck:
            for card in original_cards:
                self.add_card(card)

    def deck_count(self) -> int:
        """Return the number of cards in the deck."""
        return len(self.deck)

    def dreamsign_count(self) -> int:
        """Return the number of active dreamsigns."""
        return len(self.dreamsigns)

    def deck_by_rarity(self) -> dict[Rarity, int]:
        """Return a count of cards per rarity in the deck."""
        counts: dict[Rarity, int] = {r: 0 for r in Rarity}
        for dc in self.deck:
            counts[dc.card.rarity] += 1
        return counts
