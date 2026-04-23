package engine

import (
	"reflect"
	"testing"

	"dreamtides/prototypes/combat_solver/internal/model"
)

func TestNocturneStrummerAddsEffectiveSpark(t *testing.T) {
	board := testBoard()
	addCharacter(&board, model.PlayerOne, model.FrontSlot(1), "front", "front-card", "Front", 3)
	addCharacter(&board, model.PlayerOne, model.BackSlot(1), "left-support", "nocturne-card", "Left Support", 1)
	addCharacter(&board, model.PlayerOne, model.BackSlot(2), "right-support", "nocturne-card", "Right Support", 1)
	addCharacter(&board, model.PlayerOne, model.BackSlot(3), "other-support", "nocturne-card", "Other Support", 1)

	got := EffectiveSpark(board, model.PlayerOne, model.FrontSlot(1))
	if got != 7 {
		t.Fatalf("EffectiveSpark() = %d, want 7", got)
	}

	backGot := EffectiveSpark(board, model.PlayerOne, model.BackSlot(1))
	if backGot != 1 {
		t.Fatalf("back EffectiveSpark() = %d, want 1", backGot)
	}
}

func TestRuneboundChampionEndOfTurnGain(t *testing.T) {
	board := testBoard()
	addCharacter(&board, model.PlayerOne, model.FrontSlot(2), "runebound", "runebound-card", "Runebound Champion", 4)
	addCharacter(&board, model.PlayerOne, model.BackSlot(2), "left-support", "front-card", "Left Support", 1)
	addCharacter(&board, model.PlayerOne, model.BackSlot(3), "right-support", "front-card", "Right Support", 2)
	addCharacter(&board, model.PlayerOne, model.BackSlot(4), "other-support", "front-card", "Other Support", 3)

	ApplyEndOfTurnSupportGains(&board, model.PlayerOne)

	if got := board.Characters["left-support"].StoredSpark; got != 2 {
		t.Errorf("left support StoredSpark = %d, want 2", got)
	}

	if got := board.Characters["right-support"].StoredSpark; got != 3 {
		t.Errorf("right support StoredSpark = %d, want 3", got)
	}

	if got := board.Characters["other-support"].StoredSpark; got != 3 {
		t.Errorf("other support StoredSpark = %d, want 3", got)
	}
}

func TestJudgmentDissolvesLowerSparkAndScoresUnblocked(t *testing.T) {
	board := testBoard()
	addCharacter(&board, model.PlayerOne, model.FrontSlot(0), "attacker-low", "front-card", "Attacker Low", 2)
	addCharacter(&board, model.PlayerTwo, model.FrontSlot(0), "defender-high", "front-card", "Defender High", 4)
	addCharacter(&board, model.PlayerOne, model.FrontSlot(1), "attacker-unblocked", "front-card", "Attacker Unblocked", 5)

	outcome := ResolveJudgment(&board, model.PlayerOne)

	if got := outcome.Points[model.PlayerOne]; got != 5 {
		t.Fatalf("attacker points = %d, want 5", got)
	}

	wantDissolved := []DissolvedCharacter{{ID: "attacker-low", Name: "Attacker Low", Spark: 2}}
	if !reflect.DeepEqual(outcome.Dissolved[model.PlayerOne], wantDissolved) {
		t.Fatalf("attacker dissolved = %v, want %v", outcome.Dissolved[model.PlayerOne], wantDissolved)
	}

	if _, ok := board.Characters["attacker-low"]; ok {
		t.Fatalf("attacker-low still present in Characters")
	}

	if got := board.Slots[model.PlayerOne][model.FrontSlot(0)]; got != "" {
		t.Fatalf("attacker-low slot = %q, want empty", got)
	}

	if _, ok := board.Characters["defender-high"]; !ok {
		t.Fatalf("defender-high missing from Characters")
	}

	if _, ok := board.Characters["attacker-unblocked"]; !ok {
		t.Fatalf("attacker-unblocked missing from Characters")
	}
}

func TestJudgmentTieDissolvesBoth(t *testing.T) {
	board := testBoard()
	addCharacter(&board, model.PlayerOne, model.FrontSlot(3), "attacker", "front-card", "Attacker", 4)
	addCharacter(&board, model.PlayerTwo, model.FrontSlot(3), "defender", "front-card", "Defender", 4)

	outcome := ResolveJudgment(&board, model.PlayerOne)

	wantAttackerDissolved := []DissolvedCharacter{{ID: "attacker", Name: "Attacker", Spark: 4}}
	if !reflect.DeepEqual(outcome.Dissolved[model.PlayerOne], wantAttackerDissolved) {
		t.Fatalf("attacker dissolved = %v, want %v", outcome.Dissolved[model.PlayerOne], wantAttackerDissolved)
	}

	wantDefenderDissolved := []DissolvedCharacter{{ID: "defender", Name: "Defender", Spark: 4}}
	if !reflect.DeepEqual(outcome.Dissolved[model.PlayerTwo], wantDefenderDissolved) {
		t.Fatalf("defender dissolved = %v, want %v", outcome.Dissolved[model.PlayerTwo], wantDefenderDissolved)
	}

	if _, ok := board.Characters["attacker"]; ok {
		t.Fatalf("attacker still present in Characters")
	}

	if _, ok := board.Characters["defender"]; ok {
		t.Fatalf("defender still present in Characters")
	}
}

func testBoard() model.Board {
	return model.Board{
		Characters: map[string]model.Character{},
		Cards: map[string]model.Card{
			"front-card": {
				ID:            "front-card",
				Name:          "Front",
				SupportEffect: model.SupportNone,
			},
			"nocturne-card": {
				ID:            "nocturne-card",
				Name:          "Nocturne Strummer",
				SupportEffect: model.SupportNocturneStrummer,
			},
			"runebound-card": {
				ID:            "runebound-card",
				Name:          "Runebound Champion",
				SupportEffect: model.SupportRuneboundChampion,
			},
		},
	}
}

func addCharacter(
	board *model.Board,
	player model.Player,
	slot int,
	id string,
	cardID string,
	name string,
	storedSpark int,
) {
	board.Slots[player][slot] = id
	board.Characters[id] = model.Character{
		ID:          id,
		CardID:      cardID,
		Name:        name,
		Owner:       player,
		StoredSpark: storedSpark,
	}
}
