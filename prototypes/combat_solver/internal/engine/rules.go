package engine

import (
	"sort"

	"dreamtides/prototypes/combat_solver/internal/model"
)

type DissolvedCharacter struct {
	ID    string
	Name  string
	Spark int
}

type Outcome struct {
	Points    [2]int
	Dissolved [2][]DissolvedCharacter
}

func EffectiveSpark(board model.Board, player model.Player, slot int) int {
	character, ok := board.CharacterAt(player, slot)
	if !ok {
		return 0
	}

	spark := character.StoredSpark
	if !model.IsFront(slot) {
		return spark
	}

	for _, backSlot := range model.SupportingBackSlots(slot) {
		supportingCharacter, ok := board.CharacterAt(player, backSlot)
		if !ok {
			continue
		}

		if supportEffect(board, supportingCharacter) == model.SupportNocturneStrummer {
			spark += 2
		}
	}

	return spark
}

func ApplyEndOfTurnSupportGains(board *model.Board, player model.Player) {
	for frontIndex := range model.FrontSlots {
		frontSlot := model.FrontSlot(frontIndex)
		character, ok := board.CharacterAt(player, frontSlot)
		if !ok {
			continue
		}

		if supportEffect(*board, character) != model.SupportRuneboundChampion {
			continue
		}

		for _, backSlot := range model.SupportingBackSlots(frontSlot) {
			supportingCharacter, ok := board.CharacterAt(player, backSlot)
			if !ok {
				continue
			}

			supportingCharacter.StoredSpark++
			board.Characters[supportingCharacter.ID] = supportingCharacter
		}
	}
}

func ResolveJudgment(board *model.Board, attacker model.Player) Outcome {
	var outcome Outcome
	defender := attacker.Opponent()

	for frontIndex := range model.FrontSlots {
		frontSlot := model.FrontSlot(frontIndex)
		attackerCharacter, attackerPresent := board.CharacterAt(attacker, frontSlot)
		defenderCharacter, defenderPresent := board.CharacterAt(defender, frontSlot)

		if !attackerPresent {
			continue
		}

		attackerSpark := EffectiveSpark(*board, attacker, frontSlot)
		if !defenderPresent {
			outcome.Points[attacker] += attackerSpark
			continue
		}

		defenderSpark := EffectiveSpark(*board, defender, frontSlot)
		switch {
		case attackerSpark < defenderSpark:
			dissolve(board, &outcome, attacker, frontSlot, attackerCharacter, attackerSpark)
		case attackerSpark > defenderSpark:
			dissolve(board, &outcome, defender, frontSlot, defenderCharacter, defenderSpark)
		default:
			dissolve(board, &outcome, attacker, frontSlot, attackerCharacter, attackerSpark)
			dissolve(board, &outcome, defender, frontSlot, defenderCharacter, defenderSpark)
		}
	}

	return outcome
}

func SurvivingEffectiveSparks(board model.Board, player model.Player) []int {
	sparks := make([]int, 0, model.TotalSlots)
	for slot := range model.TotalSlots {
		if _, ok := board.CharacterAt(player, slot); ok {
			sparks = append(sparks, EffectiveSpark(board, player, slot))
		}
	}

	sort.Sort(sort.Reverse(sort.IntSlice(sparks)))
	return sparks
}

func dissolve(
	board *model.Board,
	outcome *Outcome,
	player model.Player,
	slot int,
	character model.Character,
	spark int,
) {
	outcome.Dissolved[player] = append(outcome.Dissolved[player], DissolvedCharacter{
		ID:    character.ID,
		Name:  character.Name,
		Spark: spark,
	})
	board.Slots[player][slot] = ""
	delete(board.Characters, character.ID)
}

func supportEffect(board model.Board, character model.Character) model.SupportEffect {
	card, ok := board.Cards[character.CardID]
	if !ok {
		return model.SupportNone
	}

	return card.SupportEffect
}
