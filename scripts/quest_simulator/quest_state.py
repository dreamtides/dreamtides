"""Mutable quest state management.

Tracks the player's deck, dreamsigns, dreamcaller, essence, and
a reference to the draft strategy. All site interactions mutate
state through this module's methods.
"""

import random
from typing import TYPE_CHECKING, Any, Optional

from models import (
    DeckCard,
    Dreamcaller,
    Dreamsign,
)

if TYPE_CHECKING:
    from draft_strategy import DraftStrategy


class QuestState:
    """Central mutable state object for a quest run."""

    def __init__(
        self,
        essence: int,
        rng: random.Random,
        max_deck: int = 50,
        min_deck: int = 25,
        max_dreamsigns: int = 12,
        debug: bool = False,
    ) -> None:
        self.deck: list[DeckCard] = []
        self.dreamsigns: list[Dreamsign] = []
        self.dreamcaller: Optional[Dreamcaller] = None
        self.essence: int = essence
        self.completion_level: int = 0
        self.rng: random.Random = rng
        # pyre-ignore[8]: Two-phase init; set immediately after construction.
        self.draft_strategy: DraftStrategy = None  # type: ignore[assignment]
        self.max_deck: int = max_deck
        self.min_deck: int = min_deck
        self.max_dreamsigns: int = max_dreamsigns
        self.debug: bool = debug
        self.archetype_draft: bool = False
        self.bane_instance_counter: int = 0

    def add_card(self, card_instance: Any) -> None:
        """Add a card to the deck from a draft CardInstance."""
        self.deck.append(DeckCard(instance=card_instance))

    def add_bane_card(self, card_instance: Any) -> None:
        """Add a bane card to the deck."""
        self.deck.append(DeckCard(instance=card_instance, is_bane=True))

    def duplicate_card(self, deck_card: DeckCard) -> None:
        """Add a copy of a deck card, preserving all metadata flags."""
        self.deck.append(
            DeckCard(
                instance=deck_card.instance,
                is_transfigured=deck_card.is_transfigured,
                is_bane=deck_card.is_bane,
                transfig_note=deck_card.transfig_note,
            )
        )

    def remove_card(self, deck_card: DeckCard) -> None:
        """Remove a card from the deck."""
        self.deck.remove(deck_card)

    def set_dreamcaller(self, dreamcaller: Dreamcaller) -> None:
        """Set the dreamcaller and apply essence bonus.

        If a dreamcaller is already set, its essence bonus is removed first
        so that calling this method is idempotent with respect to essence.
        """
        if self.dreamcaller is not None:
            self.essence -= self.dreamcaller.essence_bonus
        self.dreamcaller = dreamcaller
        self.essence += dreamcaller.essence_bonus

    def add_dreamsign(self, dreamsign: Dreamsign) -> None:
        """Add a dreamsign."""
        self.dreamsigns.append(dreamsign)

    def remove_dreamsign(self, dreamsign: Dreamsign) -> None:
        """Remove a dreamsign."""
        self.dreamsigns.remove(dreamsign)

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
        original_instances = [dc.instance for dc in self.deck]
        while len(self.deck) <= self.min_deck:
            for inst in original_instances:
                self.deck.append(DeckCard(instance=inst))

    def deck_count(self) -> int:
        """Return the number of cards in the deck."""
        return len(self.deck)

    def dreamsign_count(self) -> int:
        """Return the number of active dreamsigns."""
        return len(self.dreamsigns)
