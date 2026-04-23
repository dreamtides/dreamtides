package solver

import "testing"

func TestScoreCompareIsLexicographic(t *testing.T) {
	cases := []struct {
		name  string
		left  Score
		right Score
	}{
		{
			name:  "own survivor spark wins first",
			left:  Score{OwnSurvivors: []int{6}},
			right: Score{OwnSurvivors: []int{5, 99}},
		},
		{
			name:  "additional survivor wins after equal prefix",
			left:  Score{OwnSurvivors: []int{5, 1}},
			right: Score{OwnSurvivors: []int{5}},
		},
		{
			name: "opponent dissolved wins second",
			left: Score{
				OwnSurvivors:      []int{5},
				OpponentDissolved: []int{4},
			},
			right: Score{
				OwnSurvivors:      []int{5},
				OpponentDissolved: []int{3, 99},
			},
		},
		{
			name: "own points win third",
			left: Score{
				OwnSurvivors:      []int{5},
				OpponentDissolved: []int{4},
				OwnPoints:         2,
			},
			right: Score{
				OwnSurvivors:      []int{5},
				OpponentDissolved: []int{4},
				OwnPoints:         1,
			},
		},
		{
			name: "lower opponent points win last",
			left: Score{
				OwnSurvivors:       []int{5},
				OpponentDissolved:  []int{4},
				OwnPoints:          2,
				OpponentPoints:     1,
				OpponentPointsTerm: -1,
			},
			right: Score{
				OwnSurvivors:       []int{5},
				OpponentDissolved:  []int{4},
				OwnPoints:          2,
				OpponentPoints:     2,
				OpponentPointsTerm: -2,
			},
		},
	}

	for _, testCase := range cases {
		t.Run(testCase.name, func(t *testing.T) {
			if got := CompareScore(testCase.left, testCase.right); got <= 0 {
				t.Fatalf("CompareScore(left, right) = %d, want positive", got)
			}

			if got := CompareScore(testCase.right, testCase.left); got >= 0 {
				t.Fatalf("CompareScore(right, left) = %d, want negative", got)
			}
		})
	}

	equalLeft := Score{OwnSurvivors: []int{3}, OpponentPoints: 5}
	equalRight := Score{OwnSurvivors: []int{3}, OpponentPoints: 5, OpponentPointsTerm: -5}
	if got := CompareScore(equalLeft, equalRight); got != 0 {
		t.Fatalf("CompareScore(equalLeft, equalRight) = %d, want 0", got)
	}
}
