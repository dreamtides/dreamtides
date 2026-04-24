package scenarios

import "dreamtides/prototypes/combat_solver/internal/model"

func All() map[string]model.Board {
	return map[string]model.Board{
		"support":        support(),
		"high-spark":     highSpark(),
		"sparse":         sparse(),
		"full-stress":    fullStress(),
		"mobility-mixed": mobilityMixed(),
	}
}

type characterSpec struct {
	player        model.Player
	slot          int
	id            string
	cardID        string
	storedSpark   int
	canReposition bool
}

func support() model.Board {
	return board(
		model.PlayerOne,
		characterSpec{
			player:        model.PlayerOne,
			slot:          model.FrontSlot(0),
			id:            "p1-ring",
			cardID:        "ring",
			storedSpark:   2,
			canReposition: true,
		},
		characterSpec{
			player:        model.PlayerOne,
			slot:          model.BackSlot(0),
			id:            "p1-strummer",
			cardID:        "strummer",
			storedSpark:   1,
			canReposition: true,
		},
		characterSpec{
			player:        model.PlayerOne,
			slot:          model.BackSlot(1),
			id:            "p1-witness",
			cardID:        "witness",
			storedSpark:   7,
			canReposition: true,
		},
		characterSpec{
			player:        model.PlayerTwo,
			slot:          model.FrontSlot(0),
			id:            "p2-direwolf",
			cardID:        "direwolf",
			storedSpark:   5,
			canReposition: false,
		},
		characterSpec{
			player:        model.PlayerTwo,
			slot:          model.BackSlot(2),
			id:            "p2-champion",
			cardID:        "champion",
			storedSpark:   3,
			canReposition: true,
		},
	)
}

func highSpark() model.Board {
	return board(
		model.PlayerOne,
		characterSpec{
			player:        model.PlayerOne,
			slot:          model.FrontSlot(1),
			id:            "p1-colossus",
			cardID:        "colossus",
			storedSpark:   10,
			canReposition: false,
		},
		characterSpec{
			player:        model.PlayerOne,
			slot:          model.BackSlot(2),
			id:            "p1-champion",
			cardID:        "champion",
			storedSpark:   3,
			canReposition: true,
		},
		characterSpec{
			player:        model.PlayerTwo,
			slot:          model.FrontSlot(1),
			id:            "p2-direwolf",
			cardID:        "direwolf",
			storedSpark:   5,
			canReposition: true,
		},
		characterSpec{
			player:        model.PlayerTwo,
			slot:          model.BackSlot(1),
			id:            "p2-strummer",
			cardID:        "strummer",
			storedSpark:   1,
			canReposition: true,
		},
	)
}

func sparse() model.Board {
	return board(
		model.PlayerOne,
		characterSpec{
			player:        model.PlayerOne,
			slot:          model.BackSlot(4),
			id:            "p1-witness",
			cardID:        "witness",
			storedSpark:   7,
			canReposition: true,
		},
		characterSpec{
			player:        model.PlayerTwo,
			slot:          model.FrontSlot(3),
			id:            "p2-ring",
			cardID:        "ring",
			storedSpark:   2,
			canReposition: true,
		},
	)
}

func fullStress() model.Board {
	return board(
		model.PlayerOne,
		characterSpec{player: model.PlayerOne, slot: model.FrontSlot(0), id: "p1-front-0", cardID: "ring", storedSpark: 2, canReposition: true},
		characterSpec{player: model.PlayerOne, slot: model.FrontSlot(1), id: "p1-front-1", cardID: "direwolf", storedSpark: 5, canReposition: false},
		characterSpec{player: model.PlayerOne, slot: model.FrontSlot(2), id: "p1-front-2", cardID: "champion", storedSpark: 3, canReposition: true},
		characterSpec{player: model.PlayerOne, slot: model.FrontSlot(3), id: "p1-front-3", cardID: "colossus", storedSpark: 10, canReposition: false},
		characterSpec{player: model.PlayerOne, slot: model.BackSlot(0), id: "p1-back-0", cardID: "strummer", storedSpark: 1, canReposition: true},
		characterSpec{player: model.PlayerOne, slot: model.BackSlot(1), id: "p1-back-1", cardID: "witness", storedSpark: 7, canReposition: true},
		characterSpec{player: model.PlayerOne, slot: model.BackSlot(2), id: "p1-back-2", cardID: "ring", storedSpark: 3, canReposition: true},
		characterSpec{player: model.PlayerOne, slot: model.BackSlot(3), id: "p1-back-3", cardID: "direwolf", storedSpark: 5, canReposition: false},
		characterSpec{player: model.PlayerOne, slot: model.BackSlot(4), id: "p1-back-4", cardID: "witness", storedSpark: 7, canReposition: true},
		characterSpec{player: model.PlayerTwo, slot: model.FrontSlot(0), id: "p2-front-0", cardID: "direwolf", storedSpark: 5, canReposition: true},
		characterSpec{player: model.PlayerTwo, slot: model.FrontSlot(1), id: "p2-front-1", cardID: "ring", storedSpark: 3, canReposition: false},
		characterSpec{player: model.PlayerTwo, slot: model.FrontSlot(2), id: "p2-front-2", cardID: "colossus", storedSpark: 10, canReposition: true},
		characterSpec{player: model.PlayerTwo, slot: model.FrontSlot(3), id: "p2-front-3", cardID: "champion", storedSpark: 3, canReposition: false},
		characterSpec{player: model.PlayerTwo, slot: model.BackSlot(0), id: "p2-back-0", cardID: "witness", storedSpark: 7, canReposition: true},
		characterSpec{player: model.PlayerTwo, slot: model.BackSlot(1), id: "p2-back-1", cardID: "strummer", storedSpark: 1, canReposition: true},
		characterSpec{player: model.PlayerTwo, slot: model.BackSlot(2), id: "p2-back-2", cardID: "direwolf", storedSpark: 6, canReposition: true},
		characterSpec{player: model.PlayerTwo, slot: model.BackSlot(3), id: "p2-back-3", cardID: "ring", storedSpark: 2, canReposition: false},
		characterSpec{player: model.PlayerTwo, slot: model.BackSlot(4), id: "p2-back-4", cardID: "witness", storedSpark: 7, canReposition: true},
	)
}

func mobilityMixed() model.Board {
	return board(
		model.PlayerTwo,
		characterSpec{
			player:        model.PlayerOne,
			slot:          model.FrontSlot(0),
			id:            "p1-fixed-ring",
			cardID:        "ring",
			storedSpark:   3,
			canReposition: false,
		},
		characterSpec{
			player:        model.PlayerOne,
			slot:          model.FrontSlot(3),
			id:            "p1-mobile-direwolf",
			cardID:        "direwolf",
			storedSpark:   5,
			canReposition: true,
		},
		characterSpec{
			player:        model.PlayerOne,
			slot:          model.BackSlot(3),
			id:            "p1-mobile-strummer",
			cardID:        "strummer",
			storedSpark:   1,
			canReposition: true,
		},
		characterSpec{
			player:        model.PlayerTwo,
			slot:          model.FrontSlot(2),
			id:            "p2-fixed-colossus",
			cardID:        "colossus",
			storedSpark:   10,
			canReposition: false,
		},
		characterSpec{
			player:        model.PlayerTwo,
			slot:          model.BackSlot(1),
			id:            "p2-mobile-champion",
			cardID:        "champion",
			storedSpark:   3,
			canReposition: true,
		},
		characterSpec{
			player:        model.PlayerTwo,
			slot:          model.BackSlot(4),
			id:            "p2-mobile-witness",
			cardID:        "witness",
			storedSpark:   7,
			canReposition: true,
		},
	)
}

func board(active model.Player, characters ...characterSpec) model.Board {
	board := model.Board{
		Active:     active,
		Characters: map[string]model.Character{},
		Cards:      fixtureCards(),
	}

	for _, character := range characters {
		addCharacter(&board, character)
	}

	return board
}

func addCharacter(board *model.Board, spec characterSpec) {
	board.Slots[spec.player][spec.slot] = spec.id
	board.Characters[spec.id] = model.Character{
		ID:            spec.id,
		CardID:        spec.cardID,
		Name:          board.Cards[spec.cardID].Name,
		Owner:         spec.player,
		StoredSpark:   spec.storedSpark,
		CanReposition: spec.canReposition,
	}
}

func fixtureCards() map[string]model.Card {
	return map[string]model.Card{
		"champion": {
			ID:            "champion",
			Name:          "Runebound Champion",
			RenderedText:  "At end of turn, each supporting character gains +1 spark.",
			BaseSpark:     3,
			SupportEffect: model.SupportRuneboundChampion,
		},
		"colossus": {
			ID:        "colossus",
			Name:      "Wildflower Colossus",
			BaseSpark: 10,
		},
		"direwolf": {
			ID:        "direwolf",
			Name:      "Marked Direwolf",
			BaseSpark: 5,
		},
		"ring": {
			ID:        "ring",
			Name:      "Ringwatcher",
			BaseSpark: 2,
		},
		"strummer": {
			ID:            "strummer",
			Name:          "Nocturne Strummer",
			RenderedText:  "Supported characters gain +2 spark.",
			BaseSpark:     1,
			SupportEffect: model.SupportNocturneStrummer,
		},
		"witness": {
			ID:        "witness",
			Name:      "Final Witness",
			BaseSpark: 7,
		},
	}
}
