package scenarios

import (
	"testing"

	"dreamtides/prototypes/combat_solver/internal/model"
)

func TestGeneratedScenariosAreNamedAndNonEmpty(t *testing.T) {
	scenarios := All()
	names := []string{"support", "high-spark", "sparse", "full-stress", "mobility-mixed"}

	for _, name := range names {
		board, ok := scenarios[name]
		if !ok {
			t.Fatalf("scenario %q missing", name)
		}
		if len(board.Characters) == 0 {
			t.Fatalf("scenario %q has no characters", name)
		}
		if len(board.Cards) != len(fixtureCards()) {
			t.Fatalf("scenario %q has %d fixture cards, want %d", name, len(board.Cards), len(fixtureCards()))
		}
		if !hasOccupiedSlot(board) {
			t.Fatalf("scenario %q has no occupied slots", name)
		}
	}
}

func TestGeneratedScenariosIncludeCompleteFixtureCards(t *testing.T) {
	for name, board := range All() {
		for cardID := range fixtureCards() {
			if _, ok := board.Cards[cardID]; !ok {
				t.Fatalf("scenario %q missing fixture card %q", name, cardID)
			}
		}
	}
}

func hasOccupiedSlot(board model.Board) bool {
	for player := range board.Slots {
		for slot := range board.Slots[player] {
			if board.Slots[player][slot] != "" {
				return true
			}
		}
	}

	return false
}
