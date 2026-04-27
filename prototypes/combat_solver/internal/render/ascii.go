package render

import (
	"fmt"
	"strings"

	"dreamtides/prototypes/combat_solver/internal/model"
)

const (
	cellWidth = 22

	reset     = "\x1b[0m"
	playerOne = "\x1b[38;5;39m"
	playerTwo = "\x1b[38;5;208m"
	active    = "\x1b[1;38;5;120m"
	empty     = "\x1b[38;5;244m"
)

func Board(board model.Board) string {
	var builder strings.Builder

	fmt.Fprintf(&builder, "%sActive: %s%s\n", active, playerName(board.Active), reset)
	renderPlayer(&builder, board, model.PlayerTwo)
	fmt.Fprint(&builder, "\n")
	renderPlayer(&builder, board, model.PlayerOne)

	return builder.String()
}

func renderPlayer(builder *strings.Builder, board model.Board, player model.Player) {
	fmt.Fprintf(builder, "%s%s%s\n", playerColor(player), playerName(player), reset)
	renderRow(builder, board, player, "Front", model.FrontSlots, model.FrontSlot)
	renderRow(builder, board, player, "Back ", model.BackSlots, model.BackSlot)
}

func renderRow(
	builder *strings.Builder,
	board model.Board,
	player model.Player,
	label string,
	count int,
	slotFor func(int) int,
) {
	cells := make([]slotCell, 0, count)
	for index := range count {
		cells = append(cells, buildSlotCell(board, player, slotFor(index), index))
	}

	fmt.Fprintf(builder, "%s ", label)
	renderBorder(builder, cells)
	fmt.Fprintf(builder, "%s ", strings.Repeat(" ", len(label)))
	renderCellLine(builder, cells, 0)
	fmt.Fprintf(builder, "%s ", strings.Repeat(" ", len(label)))
	renderCellLine(builder, cells, 1)
	fmt.Fprintf(builder, "%s ", strings.Repeat(" ", len(label)))
	renderCellLine(builder, cells, 2)
	fmt.Fprintf(builder, "%s ", strings.Repeat(" ", len(label)))
	renderBorder(builder, cells)
}

func renderBorder(builder *strings.Builder, cells []slotCell) {
	for _, cell := range cells {
		fmt.Fprintf(builder, "%s+%s+%s ", cell.Color, strings.Repeat("-", cellWidth), reset)
	}
	fmt.Fprint(builder, "\n")
}

func renderCellLine(builder *strings.Builder, cells []slotCell, line int) {
	for _, cell := range cells {
		fmt.Fprintf(builder, "%s|%s|%s ", cell.Color, pad(cell.Lines[line]), reset)
	}
	fmt.Fprint(builder, "\n")
}

type slotCell struct {
	Color string
	Lines [3]string
}

func buildSlotCell(board model.Board, player model.Player, slot int, rowIndex int) slotCell {
	rank := "B"
	if model.IsFront(slot) {
		rank = "F"
	}

	character, ok := board.CharacterAt(player, slot)
	if !ok {
		return slotCell{
			Color: empty,
			Lines: [3]string{
				fmt.Sprintf("%s%d", rank, rowIndex),
				"--",
				"",
			},
		}
	}

	name := character.Name
	if name == "" {
		name = character.ID
	}

	return slotCell{
		Color: playerColor(player),
		Lines: [3]string{
			fmt.Sprintf("%s%d", rank, rowIndex),
			name,
			fmt.Sprintf("spark %d", character.StoredSpark),
		},
	}
}

func pad(value string) string {
	if len(value) > cellWidth {
		value = value[:cellWidth-1] + "~"
	}
	return value + strings.Repeat(" ", cellWidth-len(value))
}

func playerName(player model.Player) string {
	if player == model.PlayerTwo {
		return "Player Two"
	}

	return "Player One"
}

func playerColor(player model.Player) string {
	if player == model.PlayerTwo {
		return playerTwo
	}

	return playerOne
}
