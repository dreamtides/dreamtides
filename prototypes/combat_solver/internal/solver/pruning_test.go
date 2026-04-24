package solver

import (
	"testing"
	"time"

	"dreamtides/prototypes/combat_solver/internal/model"
	"dreamtides/prototypes/combat_solver/internal/scenarios"
)

func TestSolveReportsIncompleteWhenBudgetExpires(t *testing.T) {
	result := Solve(scenarios.All()["full-stress"], Options{
		Budget:    time.Nanosecond,
		MaxRanked: 3,
	})

	if result.Complete {
		t.Fatalf("Solve() Complete = true, want false")
	}

	if !result.TimedOutAtRoot && !result.TimedOutAtReply {
		t.Fatalf("Solve() did not report timeout location: %+v", result)
	}

	if result.Elapsed == "" {
		t.Fatalf("Solve() Elapsed is empty")
	}
}

func TestFullStressReturnsBestCandidateWithHundredMillisecondBudget(t *testing.T) {
	result := Solve(scenarios.All()["full-stress"], Options{
		Budget:    100 * time.Millisecond,
		MaxRanked: 3,
	})

	if result.RootEvaluated == 0 {
		t.Fatalf("RootEvaluated = 0, want at least one evaluated root")
	}

	if result.ReplyEvaluated == 0 {
		t.Fatalf("ReplyEvaluated = 0, want at least one evaluated reply")
	}

	if len(result.Ranked) == 0 {
		t.Fatalf("Ranked is empty, want best candidate")
	}

	if len(result.Ranked) > 3 {
		t.Fatalf("len(Ranked) = %d, want at most 3", len(result.Ranked))
	}

	if result.Best.Placement.Key() != result.Ranked[0].Placement.Key() {
		t.Fatalf("Best placement key = %q, want top ranked key %q",
			result.Best.Placement.Key(),
			result.Ranked[0].Placement.Key())
	}

	if result.Elapsed == "" {
		t.Fatalf("Solve() Elapsed is empty")
	}
}

func TestRootReplySearchStopsWhenOpponentCanForceWorseThanBest(t *testing.T) {
	board := solverBoard()
	addSolverCharacter(&board, model.PlayerOne, model.FrontSlot(0), "ally", 3, true)
	addSolverCharacter(&board, model.PlayerTwo, model.BackSlot(0), "enemy", 5, true)

	rootPlacement := Placement{Player: model.PlayerOne}
	rootPlacement.Slots[model.FrontSlot(0)] = "ally"
	bestRoot := Score{}

	evaluation := evaluateRoot(
		board,
		model.PlayerOne,
		rootPlacement,
		time.Now().Add(time.Hour),
		&bestRoot,
	)

	if !evaluation.Complete {
		t.Fatalf("evaluateRoot() Complete = false")
	}

	if !evaluation.Evaluated {
		t.Fatalf("evaluateRoot() Evaluated = false")
	}

	if evaluation.ReplyEvaluated != 1 {
		t.Fatalf("ReplyEvaluated = %d, want cutoff after first reply", evaluation.ReplyEvaluated)
	}

	if len(evaluation.Candidate.Score.OwnSurvivors) != 0 {
		t.Fatalf("OwnSurvivors = %v, want empty after forced reply",
			evaluation.Candidate.Score.OwnSurvivors)
	}
}
