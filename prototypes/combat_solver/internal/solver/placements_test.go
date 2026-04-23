package solver

import (
	"testing"

	"dreamtides/prototypes/combat_solver/internal/model"
)

func TestGeneratePlacementsKeepsImmovableCharactersFixed(t *testing.T) {
	board := placementBoard()
	addPlacementCharacter(&board, model.PlayerOne, model.FrontSlot(0), "fixed", "Fixed", false)
	addPlacementCharacter(&board, model.PlayerOne, model.FrontSlot(1), "mover", "Mover", true)

	placements := GeneratePlacements(board, model.PlayerOne)

	if len(placements) != 8 {
		t.Fatalf("len(placements) = %d, want 8", len(placements))
	}
	for _, placement := range placements {
		if placement.Slots[model.FrontSlot(0)] != "fixed" {
			t.Fatalf("immovable character moved in %+v", placement.Slots)
		}
	}
}

func TestGeneratePlacementsCountsPartialBoardChoices(t *testing.T) {
	board := placementBoard()
	addPlacementCharacter(&board, model.PlayerOne, model.BackSlot(0), "alpha", "Alpha", true)
	addPlacementCharacter(&board, model.PlayerOne, model.BackSlot(1), "beta", "Beta", true)

	placements := GeneratePlacements(board, model.PlayerOne)

	if len(placements) != 72 {
		t.Fatalf("len(placements) = %d, want 72", len(placements))
	}
	if placements[0].Slots[model.FrontSlot(0)] != "alpha" ||
		placements[0].Slots[model.FrontSlot(1)] != "beta" {
		t.Fatalf("first placement is not deterministic by slot order: %+v", placements[0].Slots)
	}
}

func TestApplyPlacementMovesOnlyChosenPlayer(t *testing.T) {
	board := placementBoard()
	addPlacementCharacter(&board, model.PlayerOne, model.FrontSlot(0), "ally", "Ally", true)
	addPlacementCharacter(&board, model.PlayerTwo, model.FrontSlot(0), "enemy", "Enemy", true)
	placement := Placement{Player: model.PlayerOne}
	placement.Slots[model.BackSlot(4)] = "ally"

	next := ApplyPlacement(board, placement)

	if next.Slots[model.PlayerOne][model.BackSlot(4)] != "ally" {
		t.Fatalf("ally not moved: %+v", next.Slots[model.PlayerOne])
	}
	if next.Slots[model.PlayerTwo][model.FrontSlot(0)] != "enemy" {
		t.Fatalf("opponent slots changed: %+v", next.Slots[model.PlayerTwo])
	}
	if board.Slots[model.PlayerOne][model.FrontSlot(0)] != "ally" {
		t.Fatalf("input board mutated: %+v", board.Slots[model.PlayerOne])
	}
}

func TestPlacementKeyIsStable(t *testing.T) {
	placement := Placement{Player: model.PlayerOne}
	placement.Slots[model.FrontSlot(0)] = "a"
	placement.Slots[model.BackSlot(4)] = "b"

	if placement.Key() != "a||||||||b" {
		t.Fatalf("Key() = %q", placement.Key())
	}
}

func placementBoard() model.Board {
	return model.Board{Characters: map[string]model.Character{}}
}

func addPlacementCharacter(
	board *model.Board,
	player model.Player,
	slot int,
	id string,
	name string,
	canReposition bool,
) {
	board.Slots[player][slot] = id
	board.Characters[id] = model.Character{
		ID:            id,
		Name:          name,
		Owner:         player,
		CanReposition: canReposition,
	}
}
