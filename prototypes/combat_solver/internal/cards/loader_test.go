package cards

import (
	"os"
	"path/filepath"
	"testing"

	"dreamtides/prototypes/combat_solver/internal/model"
)

func TestLoadCore11CharactersFiltersEventsAndTagsNocturne(t *testing.T) {
	dir := t.TempDir()
	cardListsPath := writeFixture(t, dir, "card-lists.toml", `
[[card-lists]]
list-name = "Core 11"
list-type = "BaseCardId"
card-id = "character-id"
copies = 1

[[card-lists]]
list-name = "Core 11"
list-type = "BaseCardId"
card-id = "event-id"
copies = 1
`)
	renderedCardsPath := writeFixture(t, dir, "rendered-cards.toml", `
[[cards]]
name = "Nocturne Strummer"
id = "character-id"
rendered-text = "Custom text still tags by name."
card-type = "Character"
spark = 1

[[cards]]
name = "Ignored Event"
id = "event-id"
rendered-text = "Ignored."
card-type = "Event"
spark = ""
`)

	cards, err := LoadCore11Characters(cardListsPath, renderedCardsPath)
	if err != nil {
		t.Fatalf("LoadCore11Characters() error = %v", err)
	}

	if len(cards) != 1 {
		t.Fatalf("len(cards) = %d, want 1", len(cards))
	}

	card, ok := cards["character-id"]
	if !ok {
		t.Fatalf("cards[character-id] missing")
	}

	if card.BaseSpark != 1 {
		t.Errorf("BaseSpark = %d, want 1", card.BaseSpark)
	}

	if card.SupportEffect != model.SupportNocturneStrummer {
		t.Errorf("SupportEffect = %v, want %v", card.SupportEffect, model.SupportNocturneStrummer)
	}
}

func TestLoadCore11CharactersTagsRuneboundByRenderedText(t *testing.T) {
	dir := t.TempDir()
	cardListsPath := writeFixture(t, dir, "card-lists.toml", `
[[card-lists]]
list-name = "Core 11"
list-type = "BaseCardId"
card-id = "character-id"
copies = 1
`)
	renderedCardsPath := writeFixture(t, dir, "rendered-cards.toml", `
[[cards]]
name = "Rune Student"
id = "character-id"
rendered-text = "At end of turn, each supporting character gains +1 spark."
card-type = "Character"
spark = "3"
`)

	cards, err := LoadCore11Characters(cardListsPath, renderedCardsPath)
	if err != nil {
		t.Fatalf("LoadCore11Characters() error = %v", err)
	}

	card := cards["character-id"]
	if card.BaseSpark != 3 {
		t.Errorf("BaseSpark = %d, want 3", card.BaseSpark)
	}

	if card.SupportEffect != model.SupportRuneboundChampion {
		t.Errorf("SupportEffect = %v, want %v", card.SupportEffect, model.SupportRuneboundChampion)
	}
}

func TestLoadCore11CharactersRealRepoTOML(t *testing.T) {
	root, ok := findRepoRoot(t)
	if !ok {
		t.Skip("repo rules_engine Tabula TOML files not found")
	}

	cards, err := LoadCore11Characters(
		filepath.Join(root, "rules_engine", "tabula", "card-lists.toml"),
		filepath.Join(root, "rules_engine", "tabula", "rendered-cards.toml"),
	)
	if err != nil {
		t.Fatalf("LoadCore11Characters() with repo TOML error = %v", err)
	}

	if len(cards) == 0 {
		t.Fatalf("len(cards) = 0, want Core 11 characters")
	}

	assertSupportEffectPresent(t, cards, model.SupportNocturneStrummer)
	assertSupportEffectPresent(t, cards, model.SupportRuneboundChampion)
}

func writeFixture(t *testing.T, dir string, name string, content string) string {
	t.Helper()

	path := filepath.Join(dir, name)
	if err := os.WriteFile(path, []byte(content), 0o600); err != nil {
		t.Fatalf("write fixture %s: %v", path, err)
	}

	return path
}

func findRepoRoot(t *testing.T) (string, bool) {
	t.Helper()

	dir, err := os.Getwd()
	if err != nil {
		t.Fatalf("Getwd(): %v", err)
	}

	for {
		cardListsPath := filepath.Join(dir, "rules_engine", "tabula", "card-lists.toml")
		renderedCardsPath := filepath.Join(dir, "rules_engine", "tabula", "rendered-cards.toml")
		if _, err := os.Stat(cardListsPath); err == nil {
			if _, err := os.Stat(renderedCardsPath); err == nil {
				return dir, true
			}
		}

		parent := filepath.Dir(dir)
		if parent == dir {
			return "", false
		}

		dir = parent
	}
}

func assertSupportEffectPresent(t *testing.T, cards map[string]model.Card, effect model.SupportEffect) {
	t.Helper()

	for _, card := range cards {
		if card.SupportEffect == effect {
			return
		}
	}

	t.Fatalf("support effect %v missing from loaded Core 11 cards", effect)
}
