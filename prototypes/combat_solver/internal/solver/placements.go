package solver

import (
	"sort"
	"strings"

	"dreamtides/prototypes/combat_solver/internal/model"
)

type Placement struct {
	Player model.Player
	Slots  [model.TotalSlots]string
}

func GeneratePlacements(board model.Board, player model.Player) []Placement {
	base := Placement{Player: player}
	usedSlots := map[int]bool{}
	var movable []model.Character

	for slot := range model.TotalSlots {
		character, ok := board.CharacterAt(player, slot)
		if !ok {
			continue
		}
		if character.CanReposition {
			movable = append(movable, character)
			continue
		}
		base.Slots[slot] = character.ID
		usedSlots[slot] = true
	}

	sort.SliceStable(movable, func(left int, right int) bool {
		if movable[left].Name != movable[right].Name {
			return movable[left].Name < movable[right].Name
		}
		return movable[left].ID < movable[right].ID
	})

	placements := []Placement{}
	assignPlacements(base, usedSlots, movable, 0, &placements)
	return dedupePlacements(placements)
}

func ApplyPlacement(board model.Board, placement Placement) model.Board {
	next := board.Clone()
	next.Slots[placement.Player] = placement.Slots
	return next
}

func (p Placement) Key() string {
	return strings.Join(p.Slots[:], "|")
}

func assignPlacements(
	current Placement,
	usedSlots map[int]bool,
	movable []model.Character,
	index int,
	placements *[]Placement,
) {
	if index == len(movable) {
		*placements = append(*placements, current)
		return
	}

	character := movable[index]
	for slot := range model.TotalSlots {
		if usedSlots[slot] {
			continue
		}
		next := current
		next.Slots[slot] = character.ID
		nextUsed := copyUsedSlots(usedSlots)
		nextUsed[slot] = true
		assignPlacements(next, nextUsed, movable, index+1, placements)
	}
}

func copyUsedSlots(used map[int]bool) map[int]bool {
	next := make(map[int]bool, len(used))
	for slot, value := range used {
		next[slot] = value
	}
	return next
}

func dedupePlacements(placements []Placement) []Placement {
	seen := map[string]bool{}
	result := make([]Placement, 0, len(placements))
	for _, placement := range placements {
		key := placement.Key()
		if seen[key] {
			continue
		}
		seen[key] = true
		result = append(result, placement)
	}
	return result
}
