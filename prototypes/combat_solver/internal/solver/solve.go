package solver

import (
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

func Solve(board model.Board, options Options) Result {
	start := time.Now()
	deadline := start.Add(resolveBudget(options.Budget))
	perspective := board.Active
	rootPlacements := GeneratePlacements(board, perspective)
	candidates := make([]Candidate, 0, len(rootPlacements))
	result := Result{Complete: true}

	for _, rootPlacement := range rootPlacements {
		if timedOut(deadline) {
			result.Complete = false
			result.TimedOutAtRoot = true
			break
		}

		candidate, replyCount, complete := evaluateRoot(board, perspective, rootPlacement, deadline)
		result.ReplyEvaluated += replyCount
		if !complete {
			result.Complete = false
			result.TimedOutAtReply = true
			break
		}

		candidates = append(candidates, candidate)
		result.RootEvaluated++
		if result.RootEvaluated == 1 || compareCandidates(candidate, result.Best) < 0 {
			result.Best = candidate
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
) (Candidate, int, bool) {
	afterRoot := ApplyPlacement(board, rootPlacement)
	engine.ApplyEndOfTurnSupportGains(&afterRoot, perspective)

	tally := Tally{}
	AddOutcome(&tally, engine.ResolveJudgment(&afterRoot, perspective.Opponent()))

	opponentPlacements := GeneratePlacements(afterRoot, perspective.Opponent())
	var worst Candidate
	replyEvaluated := 0
	for replyIndex, replyPlacement := range opponentPlacements {
		if timedOut(deadline) {
			return Candidate{}, replyEvaluated, false
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
	}

	return worst, replyEvaluated, true
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
