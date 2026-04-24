package statejson

import (
	"encoding/json"
	"fmt"
	"os"
	"strings"

	"dreamtides/prototypes/combat_solver/internal/model"
)

type File struct {
	Active    string                            `json:"active"`
	PlayerOne [model.TotalSlots]*CharacterInput `json:"player_one"`
	PlayerTwo [model.TotalSlots]*CharacterInput `json:"player_two"`
}

type CharacterInput struct {
	ID            string `json:"id"`
	CardID        string `json:"card_id"`
	Name          string `json:"name"`
	StoredSpark   *int   `json:"stored_spark"`
	CanReposition bool   `json:"can_reposition"`
}

func Load(path string, cards map[string]model.Card) (model.Board, error) {
	bytes, err := os.ReadFile(path)
	if err != nil {
		return model.Board{}, fmt.Errorf("read state JSON: %w", err)
	}

	var file File
	if err := json.Unmarshal(bytes, &file); err != nil {
		return model.Board{}, fmt.Errorf("decode state JSON: %w", err)
	}

	active, err := parseActive(file.Active)
	if err != nil {
		return model.Board{}, err
	}

	board := model.Board{
		Active:     active,
		Characters: map[string]model.Character{},
		Cards:      copyCards(cards),
	}

	if err := addPlayerCharacters(&board, model.PlayerOne, file.PlayerOne); err != nil {
		return model.Board{}, err
	}
	if err := addPlayerCharacters(&board, model.PlayerTwo, file.PlayerTwo); err != nil {
		return model.Board{}, err
	}

	return board, nil
}

func WriteResult(path string, value any) error {
	bytes, err := json.MarshalIndent(value, "", "  ")
	if err != nil {
		return fmt.Errorf("encode result JSON: %w", err)
	}

	if err := os.WriteFile(path, append(bytes, '\n'), 0o600); err != nil {
		return fmt.Errorf("write result JSON: %w", err)
	}

	return nil
}

func parseActive(value string) (model.Player, error) {
	switch strings.TrimSpace(strings.ToLower(value)) {
	case "", "player_one", "p1":
		return model.PlayerOne, nil
	case "player_two", "p2":
		return model.PlayerTwo, nil
	default:
		return model.PlayerOne, fmt.Errorf("unknown active player %q", value)
	}
}

func copyCards(cards map[string]model.Card) map[string]model.Card {
	copy := make(map[string]model.Card, len(cards))
	for id, card := range cards {
		copy[id] = card
	}
	return copy
}

func addPlayerCharacters(
	board *model.Board,
	player model.Player,
	inputs [model.TotalSlots]*CharacterInput,
) error {
	for slot, input := range inputs {
		if input == nil {
			continue
		}

		character, err := buildCharacter(*board, player, input)
		if err != nil {
			return fmt.Errorf("%s slot %s: %w", playerLabel(player), slotLabel(slot), err)
		}

		if _, ok := board.Characters[character.ID]; ok {
			return fmt.Errorf("%s slot %s: duplicate character ID %q", playerLabel(player), slotLabel(slot), character.ID)
		}

		board.Slots[player][slot] = character.ID
		board.Characters[character.ID] = character
	}

	return nil
}

func buildCharacter(board model.Board, player model.Player, input *CharacterInput) (model.Character, error) {
	if strings.TrimSpace(input.ID) == "" {
		return model.Character{}, fmt.Errorf("missing character ID")
	}
	if strings.TrimSpace(input.CardID) == "" {
		return model.Character{}, fmt.Errorf("missing card ID")
	}

	card, ok := board.Cards[input.CardID]
	if !ok {
		return model.Character{}, fmt.Errorf("unknown card ID %q", input.CardID)
	}

	name := input.Name
	if name == "" {
		name = card.Name
	}

	storedSpark := card.BaseSpark
	if input.StoredSpark != nil {
		storedSpark = *input.StoredSpark
	}

	return model.Character{
		ID:            input.ID,
		CardID:        input.CardID,
		Name:          name,
		Owner:         player,
		StoredSpark:   storedSpark,
		CanReposition: input.CanReposition,
	}, nil
}

func playerLabel(player model.Player) string {
	if player == model.PlayerTwo {
		return "player_two"
	}

	return "player_one"
}

func slotLabel(slot int) string {
	if model.IsFront(slot) {
		return fmt.Sprintf("F%d", slot)
	}

	return fmt.Sprintf("B%d", slot-model.FrontSlots)
}
