package render

import (
	"strings"
	"testing"

	"dreamtides/prototypes/combat_solver/internal/model"
)

func TestBoardIncludesPlayersRowsNamesAndSpark(t *testing.T) {
	board := model.Board{
		Active: model.PlayerTwo,
		Characters: map[string]model.Character{
			"ally": {
				ID:          "ally",
				Name:        "Ally",
				Owner:       model.PlayerOne,
				StoredSpark: 3,
			},
			"enemy": {
				ID:          "enemy",
				Name:        "Enemy",
				Owner:       model.PlayerTwo,
				StoredSpark: 5,
			},
		},
	}
	board.Slots[model.PlayerOne][model.FrontSlot(0)] = "ally"
	board.Slots[model.PlayerTwo][model.BackSlot(4)] = "enemy"

	output := Board(board)
	for _, want := range []string{
		"\x1b[",
		"Active: Player Two",
		"Player One",
		"Player Two",
		"+----------------------+",
		"F0",
		"Ally",
		"spark 3",
		"F3",
		"--",
		"B4",
		"Enemy",
		"spark 5",
	} {
		if !strings.Contains(output, want) {
			t.Fatalf("Board() missing %q in:\n%s", want, output)
		}
	}
}

func TestBoardStaggersBackRowBetweenFrontSlots(t *testing.T) {
	output := stripANSI(Board(model.Board{}))
	lines := strings.Split(output, "\n")

	frontLine := firstLineContaining(lines, "Front")
	backLine := firstLineContaining(lines, "Back ")
	if frontLine == "" || backLine == "" {
		t.Fatalf("Board() missing front or back row:\n%s", output)
	}

	frontStart := strings.Index(frontLine, "+")
	backStart := strings.Index(backLine, "+")
	if frontStart-backStart != rowOffset {
		t.Fatalf("front row offset = %d, want %d\nfront: %q\nback:  %q", frontStart-backStart, rowOffset, frontLine, backLine)
	}
}

func firstLineContaining(lines []string, text string) string {
	for _, line := range lines {
		if strings.Contains(line, text) {
			return line
		}
	}

	return ""
}

func stripANSI(value string) string {
	var builder strings.Builder
	for index := 0; index < len(value); index++ {
		if value[index] == '\x1b' && index+1 < len(value) && value[index+1] == '[' {
			index += 2
			for index < len(value) && (value[index] < '@' || value[index] > '~') {
				index++
			}
			continue
		}
		builder.WriteByte(value[index])
	}

	return builder.String()
}
