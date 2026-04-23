package solver

import (
	"reflect"
	"testing"

	"dreamtides/prototypes/combat_solver/internal/model"
)

func TestSolveChoosesPlacementThatPreservesCharacter(t *testing.T) {
	board := solverBoard()
	addSolverCharacter(&board, model.PlayerOne, model.FrontSlot(0), "ally", 1, true)
	addSolverCharacter(&board, model.PlayerTwo, model.FrontSlot(0), "enemy-0", 5, false)
	addSolverCharacter(&board, model.PlayerTwo, model.FrontSlot(1), "enemy-1", 5, false)
	addSolverCharacter(&board, model.PlayerTwo, model.FrontSlot(2), "enemy-2", 5, false)
	addSolverCharacter(&board, model.PlayerTwo, model.FrontSlot(3), "enemy-3", 5, false)

	result := Solve(board, Options{})

	if !result.Complete {
		t.Fatalf("Solve() Complete = false")
	}

	bestSlot := placementSlot(result.Best.Placement, "ally")
	if !model.IsBack(bestSlot) {
		t.Fatalf("best placement slot = %d, want back slot; placement=%+v", bestSlot, result.Best.Placement.Slots)
	}

	wantSurvivors := []int{1}
	if !reflect.DeepEqual(result.Best.Score.OwnSurvivors, wantSurvivors) {
		t.Fatalf("OwnSurvivors = %v, want %v", result.Best.Score.OwnSurvivors, wantSurvivors)
	}
}

func TestSolveUsesWorstCaseOpponentReply(t *testing.T) {
	board := solverBoard()
	addSolverCharacter(&board, model.PlayerOne, model.FrontSlot(0), "ally", 3, false)
	addSolverCharacter(&board, model.PlayerTwo, model.BackSlot(0), "enemy", 5, true)

	result := Solve(board, Options{})

	if !result.Complete {
		t.Fatalf("Solve() Complete = false")
	}

	if got := result.Best.Reply.Slots[model.FrontSlot(0)]; got != "enemy" {
		t.Fatalf("worst reply F0 = %q, want enemy; reply=%+v", got, result.Best.Reply.Slots)
	}

	if len(result.Best.Score.OwnSurvivors) != 0 {
		t.Fatalf("OwnSurvivors = %v, want empty after worst reply", result.Best.Score.OwnSurvivors)
	}
}

func TestSolveReturnsRankedCandidates(t *testing.T) {
	board := solverBoard()
	addSolverCharacter(&board, model.PlayerOne, model.FrontSlot(0), "ally", 2, true)

	result := Solve(board, Options{MaxRanked: 3})

	if !result.Complete {
		t.Fatalf("Solve() Complete = false")
	}

	if result.RootEvaluated != 9 {
		t.Fatalf("RootEvaluated = %d, want 9", result.RootEvaluated)
	}

	if result.ReplyEvaluated != 9 {
		t.Fatalf("ReplyEvaluated = %d, want 9", result.ReplyEvaluated)
	}

	if len(result.Ranked) != 3 {
		t.Fatalf("len(Ranked) = %d, want 3", len(result.Ranked))
	}

	if result.Best.Placement.Key() != result.Ranked[0].Placement.Key() {
		t.Fatalf("Best placement key = %q, want top ranked key %q",
			result.Best.Placement.Key(),
			result.Ranked[0].Placement.Key())
	}

	for index := 1; index < len(result.Ranked); index++ {
		if CompareScore(result.Ranked[index-1].Score, result.Ranked[index].Score) < 0 {
			t.Fatalf("Ranked[%d] score %v is better than Ranked[%d] score %v",
				index,
				result.Ranked[index].Score,
				index-1,
				result.Ranked[index-1].Score)
		}
	}
}

func solverBoard() model.Board {
	return model.Board{
		Active:     model.PlayerOne,
		Characters: map[string]model.Character{},
		Cards:      map[string]model.Card{},
	}
}

func addSolverCharacter(
	board *model.Board,
	player model.Player,
	slot int,
	id string,
	storedSpark int,
	canReposition bool,
) {
	board.Slots[player][slot] = id
	board.Characters[id] = model.Character{
		ID:            id,
		Name:          id,
		Owner:         player,
		StoredSpark:   storedSpark,
		CanReposition: canReposition,
	}
}

func placementSlot(placement Placement, characterID string) int {
	for slot, placedID := range placement.Slots {
		if placedID == characterID {
			return slot
		}
	}

	return -1
}
