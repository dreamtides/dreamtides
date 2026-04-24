package statejson

import (
	"os"
	"path/filepath"
	"testing"

	"dreamtides/prototypes/combat_solver/internal/model"
)

func TestLoadBuildsBoardFromJSON(t *testing.T) {
	spark := 0
	path := writeStateFixture(t, `{
  "active": "p2",
  "player_one": [
    {
      "id": "ally",
      "card_id": "ring",
      "can_reposition": true
    }
  ],
  "player_two": [
    null,
    {
      "id": "enemy",
      "card_id": "direwolf",
      "name": "Custom Direwolf",
      "stored_spark": 0
    }
  ]
}`)

	board, err := Load(path, stateCards())
	if err != nil {
		t.Fatalf("Load() error = %v", err)
	}

	if board.Active != model.PlayerTwo {
		t.Fatalf("Active = %v, want PlayerTwo", board.Active)
	}

	ally, ok := board.CharacterAt(model.PlayerOne, model.FrontSlot(0))
	if !ok {
		t.Fatalf("player one F0 missing")
	}
	if ally.Name != "Ring Bearer" {
		t.Fatalf("ally.Name = %q, want card name default", ally.Name)
	}
	if ally.StoredSpark != 2 {
		t.Fatalf("ally.StoredSpark = %d, want card base spark", ally.StoredSpark)
	}
	if !ally.CanReposition {
		t.Fatalf("ally.CanReposition = false, want true")
	}

	enemy, ok := board.CharacterAt(model.PlayerTwo, model.FrontSlot(1))
	if !ok {
		t.Fatalf("player two F1 missing")
	}
	if enemy.Name != "Custom Direwolf" {
		t.Fatalf("enemy.Name = %q, want custom name", enemy.Name)
	}
	if enemy.StoredSpark != spark {
		t.Fatalf("enemy.StoredSpark = %d, want %d", enemy.StoredSpark, spark)
	}
}

func TestLoadDefaultsActiveToPlayerOne(t *testing.T) {
	path := writeStateFixture(t, `{}`)

	board, err := Load(path, stateCards())
	if err != nil {
		t.Fatalf("Load() error = %v", err)
	}

	if board.Active != model.PlayerOne {
		t.Fatalf("Active = %v, want PlayerOne", board.Active)
	}
}

func TestLoadRejectsUnknownCardID(t *testing.T) {
	path := writeStateFixture(t, `{
  "player_one": [
    {
      "id": "ally",
      "card_id": "missing"
    }
  ]
}`)

	if _, err := Load(path, stateCards()); err == nil {
		t.Fatalf("Load() error = nil, want unknown card ID error")
	}
}

func TestWriteResultWritesIndentedJSON(t *testing.T) {
	path := filepath.Join(t.TempDir(), "result.json")

	if err := WriteResult(path, map[string]int{"value": 1}); err != nil {
		t.Fatalf("WriteResult() error = %v", err)
	}

	bytes, err := os.ReadFile(path)
	if err != nil {
		t.Fatalf("ReadFile() error = %v", err)
	}

	if string(bytes) != "{\n  \"value\": 1\n}\n" {
		t.Fatalf("result JSON = %q", string(bytes))
	}
}

func writeStateFixture(t *testing.T, content string) string {
	t.Helper()

	path := filepath.Join(t.TempDir(), "state.json")
	if err := os.WriteFile(path, []byte(content), 0o600); err != nil {
		t.Fatalf("WriteFile() error = %v", err)
	}

	return path
}

func stateCards() map[string]model.Card {
	return map[string]model.Card{
		"direwolf": {
			ID:        "direwolf",
			Name:      "Direwolf",
			BaseSpark: 5,
		},
		"ring": {
			ID:        "ring",
			Name:      "Ring Bearer",
			BaseSpark: 2,
		},
	}
}
