package render

import (
	"fmt"
	"strings"

	"dreamtides/prototypes/combat_solver/internal/model"
)

func Board(board model.Board) string {
	var builder strings.Builder

	fmt.Fprintf(&builder, "Active: %s\n", playerName(board.Active))
	renderPlayer(&builder, board, model.PlayerOne)
	renderPlayer(&builder, board, model.PlayerTwo)

	return builder.String()
}

func renderPlayer(builder *strings.Builder, board model.Board, player model.Player) {
	fmt.Fprintf(builder, "%s\n", playerName(player))
	fmt.Fprint(builder, "  Front:")
	for slot := range model.FrontSlots {
		fmt.Fprintf(builder, " F%d=%s", slot, renderSlot(board, player, model.FrontSlot(slot)))
	}
	fmt.Fprint(builder, "\n")

	fmt.Fprint(builder, "  Back:")
	for slot := range model.BackSlots {
		fmt.Fprintf(builder, " B%d=%s", slot, renderSlot(board, player, model.BackSlot(slot)))
	}
	fmt.Fprint(builder, "\n")
}

func renderSlot(board model.Board, player model.Player, slot int) string {
	character, ok := board.CharacterAt(player, slot)
	if !ok {
		return "-"
	}

	name := character.Name
	if name == "" {
		name = character.ID
	}

	return fmt.Sprintf("%s(spark=%d)", name, character.StoredSpark)
}

func playerName(player model.Player) string {
	if player == model.PlayerTwo {
		return "Player Two"
	}

	return "Player One"
}
