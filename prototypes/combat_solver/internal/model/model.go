package model

type Player int

const (
	PlayerOne Player = iota
	PlayerTwo
)

func (p Player) Opponent() Player {
	if p == PlayerOne {
		return PlayerTwo
	}

	return PlayerOne
}

type SupportEffect int

const (
	SupportNone SupportEffect = iota
	SupportNocturneStrummer
	SupportRuneboundChampion
)

type Card struct {
	ID            string
	Name          string
	RenderedText  string
	BaseSpark     int
	SupportEffect SupportEffect
}

type Character struct {
	ID            string
	CardID        string
	Name          string
	Owner         Player
	StoredSpark   int
	CanReposition bool
}

type Board struct {
	Active     Player
	Slots      [2][TotalSlots]string
	Characters map[string]Character
	Cards      map[string]Card
}

func (b Board) Clone() Board {
	clone := b

	if b.Characters != nil {
		clone.Characters = make(map[string]Character, len(b.Characters))
		for id, character := range b.Characters {
			clone.Characters[id] = character
		}
	}

	if b.Cards != nil {
		clone.Cards = make(map[string]Card, len(b.Cards))
		for id, card := range b.Cards {
			clone.Cards[id] = card
		}
	}

	return clone
}

func (b Board) CharacterAt(player Player, slot int) (Character, bool) {
	if player != PlayerOne && player != PlayerTwo {
		return Character{}, false
	}

	if slot < 0 || slot >= TotalSlots {
		return Character{}, false
	}

	characterID := b.Slots[player][slot]
	if characterID == "" {
		return Character{}, false
	}

	character, ok := b.Characters[characterID]
	return character, ok
}
