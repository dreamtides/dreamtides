package render

import (
	"fmt"
	"strings"

	"dreamtides/prototypes/combat_solver/internal/model"
)

const (
	cellWidth = 22
	cellGap   = 2
	rowOffset = (cellWidth + 2 + cellGap) / 2

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
	renderRow(builder, board, player, "Front", rowOffset, model.FrontSlots, model.FrontSlot)
	renderRow(builder, board, player, "Back ", 0, model.BackSlots, model.BackSlot)
}

func renderRow(
	builder *strings.Builder,
	board model.Board,
	player model.Player,
	label string,
	offset int,
	count int,
	slotFor func(int) int,
) {
	cells := make([]slotCell, 0, count)
	for index := range count {
		cells = append(cells, buildSlotCell(board, player, slotFor(index), index))
	}

	renderPrefix(builder, label, offset)
	renderBorder(builder, cells)
	renderPrefix(builder, strings.Repeat(" ", len(label)), offset)
	renderCellLine(builder, cells, 0)
	renderPrefix(builder, strings.Repeat(" ", len(label)), offset)
	renderCellLine(builder, cells, 1)
	renderPrefix(builder, strings.Repeat(" ", len(label)), offset)
	renderCellLine(builder, cells, 2)
	renderPrefix(builder, strings.Repeat(" ", len(label)), offset)
	renderBorder(builder, cells)
}

func renderPrefix(builder *strings.Builder, label string, offset int) {
	fmt.Fprintf(builder, "%s %s", label, strings.Repeat(" ", offset))
}

func renderBorder(builder *strings.Builder, cells []slotCell) {
	for _, cell := range cells {
		fmt.Fprintf(builder, "%s+%s+%s%s", cell.Color, strings.Repeat("-", cellWidth), reset, strings.Repeat(" ", cellGap))
	}
	fmt.Fprint(builder, "\n")
}

func renderCellLine(builder *strings.Builder, cells []slotCell, line int) {
	for _, cell := range cells {
		fmt.Fprintf(builder, "%s|%s|%s%s", cell.Color, pad(cell.Lines[line]), reset, strings.Repeat(" ", cellGap))
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
