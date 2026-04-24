package solver

import (
	"math"
	"sort"
	"time"

	"dreamtides/prototypes/combat_solver/internal/engine"
	"dreamtides/prototypes/combat_solver/internal/model"
)

const defaultBudget = 100 * time.Millisecond

type Options struct {
	Budget    time.Duration
	MaxRanked int
}

type Candidate struct {
	Placement Placement `json:"placement"`
	Reply     Placement `json:"reply"`
	Score     Score     `json:"score"`
}

type Result struct {
	Complete        bool        `json:"complete"`
	Elapsed         string      `json:"elapsed"`
	RootEvaluated   int         `json:"root_evaluated"`
	ReplyEvaluated  int         `json:"reply_evaluated"`
	Best            Candidate   `json:"best"`
	Ranked          []Candidate `json:"ranked"`
	TimedOutAtRoot  bool        `json:"timed_out_at_root"`
	TimedOutAtReply bool        `json:"timed_out_at_reply"`
}

type rootEvaluation struct {
	Candidate      Candidate
	ReplyEvaluated int
	Complete       bool
	Evaluated      bool
}

func Solve(board model.Board, options Options) Result {
	start := time.Now()
	deadline := start.Add(resolveBudget(options.Budget))
	perspective := board.Active
	rootPlacements := GeneratePlacements(board, perspective)
	candidates := make([]Candidate, 0, len(rootPlacements))
	result := Result{Complete: true}
	var bestRoot *Score

	for _, rootPlacement := range rootPlacements {
		if timedOut(deadline) {
			result.Complete = false
			result.TimedOutAtRoot = true
			break
		}

		evaluation := evaluateRoot(board, perspective, rootPlacement, deadline, bestRoot)
		result.ReplyEvaluated += evaluation.ReplyEvaluated
		if !evaluation.Complete {
			result.Complete = false
			result.TimedOutAtReply = true
			break
		}
		if !evaluation.Evaluated {
			continue
		}

		candidate := evaluation.Candidate
		candidates = append(candidates, candidate)
		result.RootEvaluated++
		if result.RootEvaluated == 1 || compareCandidates(candidate, result.Best) < 0 {
			result.Best = candidate
			bestRoot = &result.Best.Score
		}
	}

	result.Ranked = rankedCandidates(candidates, options.MaxRanked)
	if len(result.Ranked) > 0 {
		result.Best = result.Ranked[0]
	}
	result.Elapsed = time.Since(start).String()
	return result
}

func evaluateRoot(
	board model.Board,
	perspective model.Player,
	rootPlacement Placement,
	deadline time.Time,
	bestRoot *Score,
) rootEvaluation {
	afterRoot := ApplyPlacement(board, rootPlacement)
	engine.ApplyEndOfTurnSupportGains(&afterRoot, perspective)

	tally := Tally{}
	AddOutcome(&tally, engine.ResolveJudgment(&afterRoot, perspective.Opponent()))

	if bestRoot != nil && hasCharacters(afterRoot, perspective.Opponent()) {
		upperBound := optimisticRootScore(afterRoot, perspective, tally)
		if CompareScore(upperBound, *bestRoot) <= 0 {
			return rootEvaluation{Complete: true}
		}
	}

	opponentPlacements := GeneratePlacements(afterRoot, perspective.Opponent())
	var worst Candidate
	replyEvaluated := 0
	for replyIndex, replyPlacement := range opponentPlacements {
		if timedOut(deadline) {
			return rootEvaluation{
				ReplyEvaluated: replyEvaluated,
				Complete:       false,
			}
		}

		afterReply := ApplyPlacement(afterRoot, replyPlacement)
		replyTally := cloneTally(tally)
		engine.ApplyEndOfTurnSupportGains(&afterReply, perspective.Opponent())
		AddOutcome(&replyTally, engine.ResolveJudgment(&afterReply, perspective))
		candidate := Candidate{
			Placement: rootPlacement,
			Reply:     replyPlacement,
			Score:     FinalScore(afterReply, perspective, replyTally),
		}
		replyEvaluated++

		if replyIndex == 0 || compareOpponentReply(candidate, worst) < 0 {
			worst = candidate
		}
		if bestRoot != nil && CompareScore(worst.Score, *bestRoot) <= 0 {
			return rootEvaluation{
				Candidate:      worst,
				ReplyEvaluated: replyEvaluated,
				Complete:       true,
				Evaluated:      true,
			}
		}
	}

	return rootEvaluation{
		Candidate:      worst,
		ReplyEvaluated: replyEvaluated,
		Complete:       true,
		Evaluated:      true,
	}
}

func optimisticRootScore(board model.Board, perspective model.Player, tally Tally) Score {
	score := Score{
		OwnSurvivors:      engine.SurvivingEffectiveSparks(board, perspective),
		OpponentDissolved: optimisticOpponentDissolved(board, perspective.Opponent(), tally),
		OwnPoints:         tally.Points[perspective] + optimisticOwnPointGain(board, perspective),
		OpponentPoints:    tally.Points[perspective.Opponent()],
	}
	score.OpponentPointsTerm = -score.OpponentPoints
	return score
}

func optimisticOpponentDissolved(
	board model.Board,
	opponent model.Player,
	tally Tally,
) []int {
	sparks := dissolvedSparks(tally.Dissolved[opponent])
	for _, character := range board.Characters {
		if character.Owner != opponent {
			continue
		}
		sparks = append(sparks, optimisticDissolvedSpark(character))
	}

	sort.Sort(sort.Reverse(sort.IntSlice(sparks)))
	return sparks
}

func optimisticDissolvedSpark(character model.Character) int {
	if character.StoredSpark > math.MaxInt-4 {
		return math.MaxInt
	}

	return character.StoredSpark + 4
}

func optimisticOwnPointGain(board model.Board, perspective model.Player) int {
	points := 0
	for frontIndex := range model.FrontSlots {
		frontSlot := model.FrontSlot(frontIndex)
		if _, ok := board.CharacterAt(perspective, frontSlot); !ok {
			continue
		}
		points += engine.EffectiveSpark(board, perspective, frontSlot)
	}

	return points
}

func hasCharacters(board model.Board, player model.Player) bool {
	for _, character := range board.Characters {
		if character.Owner == player {
			return true
		}
	}

	return false
}

func rankedCandidates(candidates []Candidate, maxRanked int) []Candidate {
	ranked := append([]Candidate(nil), candidates...)
	sort.SliceStable(ranked, func(left int, right int) bool {
		return compareCandidates(ranked[left], ranked[right]) < 0
	})

	if maxRanked > 0 && len(ranked) > maxRanked {
		return ranked[:maxRanked]
	}

	return ranked
}

func compareCandidates(left Candidate, right Candidate) int {
	if result := CompareScore(left.Score, right.Score); result != 0 {
		return -result
	}

	if result := compareString(left.Placement.Key(), right.Placement.Key()); result != 0 {
		return result
	}

	return compareString(left.Reply.Key(), right.Reply.Key())
}

func compareOpponentReply(left Candidate, right Candidate) int {
	if result := CompareScore(left.Score, right.Score); result != 0 {
		return result
	}

	return compareString(left.Reply.Key(), right.Reply.Key())
}

func compareString(left string, right string) int {
	switch {
	case left < right:
		return -1
	case left > right:
		return 1
	default:
		return 0
	}
}

func cloneTally(tally Tally) Tally {
	clone := tally
	for player := range tally.Dissolved {
		clone.Dissolved[player] = append([]engine.DissolvedCharacter(nil), tally.Dissolved[player]...)
	}
	return clone
}

func resolveBudget(budget time.Duration) time.Duration {
	if budget <= 0 {
		return defaultBudget
	}

	return budget
}

func timedOut(deadline time.Time) bool {
	return !time.Now().Before(deadline)
}
