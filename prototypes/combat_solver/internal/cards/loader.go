package cards

import (
	"fmt"
	"os"
	"strconv"
	"strings"

	"github.com/pelletier/go-toml/v2"

	"dreamtides/prototypes/combat_solver/internal/model"
)

const (
	baseCardIDListType = "BaseCardId"
	characterCardType  = "Character"
	core11ListName     = "Core 11"
)

type cardListsDocument struct {
	CardLists []cardListEntry `toml:"card-lists"`
}

type cardListEntry struct {
	CardID   string `toml:"card-id"`
	ListName string `toml:"list-name"`
	ListType string `toml:"list-type"`
}

type renderedCardsDocument struct {
	Cards []renderedCardEntry `toml:"cards"`
}

type renderedCardEntry struct {
	ID           string `toml:"id"`
	Name         string `toml:"name"`
	RenderedText string `toml:"rendered-text"`
	CardType     string `toml:"card-type"`
	Spark        any    `toml:"spark"`
}

// LoadCore11Characters loads Core 11 character cards from Tabula TOML files.
func LoadCore11Characters(cardListsPath string, renderedCardsPath string) (map[string]model.Card, error) {
	core11IDs, err := loadCore11IDs(cardListsPath)
	if err != nil {
		return nil, err
	}

	renderedCards, err := loadRenderedCards(renderedCardsPath)
	if err != nil {
		return nil, err
	}

	cards := make(map[string]model.Card)
	for _, renderedCard := range renderedCards {
		if _, ok := core11IDs[renderedCard.ID]; !ok || renderedCard.CardType != characterCardType {
			continue
		}

		baseSpark, err := parseSpark(renderedCard.Spark)
		if err != nil {
			return nil, fmt.Errorf("parse spark for %q (%s): %w", renderedCard.Name, renderedCard.ID, err)
		}

		cards[renderedCard.ID] = model.Card{
			ID:            renderedCard.ID,
			Name:          renderedCard.Name,
			RenderedText:  renderedCard.RenderedText,
			BaseSpark:     baseSpark,
			SupportEffect: supportEffect(renderedCard),
		}
	}

	return cards, nil
}

func loadCore11IDs(cardListsPath string) (map[string]struct{}, error) {
	bytes, err := os.ReadFile(cardListsPath)
	if err != nil {
		return nil, fmt.Errorf("read card lists: %w", err)
	}

	var document cardListsDocument
	if err := toml.Unmarshal(bytes, &document); err != nil {
		return nil, fmt.Errorf("decode card lists: %w", err)
	}

	ids := make(map[string]struct{})
	for _, entry := range document.CardLists {
		if entry.ListName == core11ListName && entry.ListType == baseCardIDListType && entry.CardID != "" {
			ids[entry.CardID] = struct{}{}
		}
	}

	return ids, nil
}

func loadRenderedCards(renderedCardsPath string) ([]renderedCardEntry, error) {
	bytes, err := os.ReadFile(renderedCardsPath)
	if err != nil {
		return nil, fmt.Errorf("read rendered cards: %w", err)
	}

	var document renderedCardsDocument
	if err := toml.Unmarshal(bytes, &document); err != nil {
		return nil, fmt.Errorf("decode rendered cards: %w", err)
	}

	return document.Cards, nil
}

func parseSpark(value any) (int, error) {
	switch value := value.(type) {
	case int64:
		return int(value), nil
	case string:
		spark := strings.TrimSpace(value)
		if spark == "" {
			return 0, fmt.Errorf("empty spark")
		}

		parsed, err := strconv.Atoi(spark)
		if err != nil {
			return 0, err
		}

		return parsed, nil
	default:
		return 0, fmt.Errorf("unsupported spark value %v", value)
	}
}

func supportEffect(card renderedCardEntry) model.SupportEffect {
	switch {
	case card.Name == "Nocturne Strummer" || strings.TrimSpace(card.RenderedText) == "Supported characters gain +2 spark.":
		return model.SupportNocturneStrummer
	case card.Name == "Runebound Champion" || strings.Contains(strings.ToLower(card.RenderedText), "each supporting character gains +1 spark"):
		return model.SupportRuneboundChampion
	default:
		return model.SupportNone
	}
}
