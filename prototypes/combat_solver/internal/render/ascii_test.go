package render

import (
	"strings"
	"testing"

	"dreamtides/prototypes/combat_solver/internal/model"
)

func TestBoardIncludesPlayersRowsNamesAndSpark(t *testing.T) {
	board := model.Board{
		Active: model.PlayerTwo,
		Characters: map[string]model.Character{
			"ally": {
				ID:          "ally",
				Name:        "Ally",
				Owner:       model.PlayerOne,
				StoredSpark: 3,
			},
			"enemy": {
				ID:          "enemy",
				Name:        "Enemy",
				Owner:       model.PlayerTwo,
				StoredSpark: 5,
			},
		},
	}
	board.Slots[model.PlayerOne][model.FrontSlot(0)] = "ally"
	board.Slots[model.PlayerTwo][model.BackSlot(4)] = "enemy"

	output := Board(board)
	for _, want := range []string{
		"Active: Player Two",
		"Player One",
		"Player Two",
		"F0=Ally(spark=3)",
		"F3=-",
		"B0=-",
		"B4=Enemy(spark=5)",
	} {
		if !strings.Contains(output, want) {
			t.Fatalf("Board() missing %q in:\n%s", want, output)
		}
	}
}
