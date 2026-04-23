package solver

import (
	"sort"

	"dreamtides/prototypes/combat_solver/internal/engine"
	"dreamtides/prototypes/combat_solver/internal/model"
)

type Score struct {
	OwnSurvivors       []int `json:"own_survivors"`
	OpponentDissolved  []int `json:"opponent_dissolved"`
	OwnPoints          int   `json:"own_points"`
	OpponentPoints     int   `json:"opponent_points"`
	OpponentPointsTerm int   `json:"opponent_points_term"`
}

type Tally struct {
	Points    [2]int
	Dissolved [2][]engine.DissolvedCharacter
}

func FinalScore(board model.Board, perspective model.Player, tally Tally) Score {
	opponent := perspective.Opponent()
	score := Score{
		OwnSurvivors:      engine.SurvivingEffectiveSparks(board, perspective),
		OpponentDissolved: dissolvedSparks(tally.Dissolved[opponent]),
		OwnPoints:         tally.Points[perspective],
		OpponentPoints:    tally.Points[opponent],
	}
	score.OpponentPointsTerm = -score.OpponentPoints
	return score
}

func CompareScore(left Score, right Score) int {
	if result := compareIntSlices(left.OwnSurvivors, right.OwnSurvivors); result != 0 {
		return result
	}

	if result := compareIntSlices(left.OpponentDissolved, right.OpponentDissolved); result != 0 {
		return result
	}

	if result := compareInt(left.OwnPoints, right.OwnPoints); result != 0 {
		return result
	}

	return compareInt(scoreOpponentPointsTerm(left), scoreOpponentPointsTerm(right))
}

func AddOutcome(tally *Tally, outcome engine.Outcome) {
	for player := range tally.Points {
		tally.Points[player] += outcome.Points[player]
		tally.Dissolved[player] = append(tally.Dissolved[player], outcome.Dissolved[player]...)
	}
}

func compareIntSlices(left []int, right []int) int {
	limit := min(len(left), len(right))
	for index := range limit {
		if result := compareInt(left[index], right[index]); result != 0 {
			return result
		}
	}

	return compareInt(len(left), len(right))
}

func compareInt(left int, right int) int {
	switch {
	case left < right:
		return -1
	case left > right:
		return 1
	default:
		return 0
	}
}

func dissolvedSparks(dissolved []engine.DissolvedCharacter) []int {
	sparks := make([]int, 0, len(dissolved))
	for _, character := range dissolved {
		sparks = append(sparks, character.Spark)
	}

	sort.Sort(sort.Reverse(sort.IntSlice(sparks)))
	return sparks
}

func scoreOpponentPointsTerm(score Score) int {
	if score.OpponentPointsTerm != 0 || score.OpponentPoints == 0 {
		return score.OpponentPointsTerm
	}

	return -score.OpponentPoints
}
