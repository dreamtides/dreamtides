package main

import (
	"flag"
	"fmt"
	"os"
	"path/filepath"
	"time"

	"dreamtides/prototypes/combat_solver/internal/cards"
	"dreamtides/prototypes/combat_solver/internal/model"
	"dreamtides/prototypes/combat_solver/internal/render"
	"dreamtides/prototypes/combat_solver/internal/scenarios"
	"dreamtides/prototypes/combat_solver/internal/solver"
	"dreamtides/prototypes/combat_solver/internal/statejson"
)

const version = "combat-solver prototype"

func main() {
	showVersion := flag.Bool("version", false, "print version and exit")
	scenarioName := flag.String("scenario", "support", "generated scenario name")
	inputPath := flag.String("input", "", "JSON board input path")
	jsonOut := flag.String("json-out", "", "optional JSON result output path")
	budget := flag.Duration("budget", 100*time.Millisecond, "solve time budget")
	rank := flag.Int("rank", 5, "number of ranked root placements to print")
	cardListsPath := flag.String("card-lists", "../../rules_engine/tabula/card-lists.toml", "path to card-lists.toml")
	renderedCardsPath := flag.String("rendered-cards", "../../rules_engine/tabula/rendered-cards.toml", "path to rendered-cards.toml")
	flag.Parse()

	if *showVersion {
		fmt.Println(version)
		return
	}

	coreCards, err := cards.LoadCore11Characters(
		resolvePath(*cardListsPath),
		resolvePath(*renderedCardsPath),
	)
	if err != nil {
		exitf("load cards: %v", err)
	}

	board := loadBoard(*scenarioName, *inputPath, coreCards)

	fmt.Println("Initial board")
	fmt.Print(render.Board(board))

	result := solver.Solve(board, solver.Options{Budget: *budget, MaxRanked: *rank})
	if len(result.Ranked) > 0 {
		proposed := solver.ApplyPlacement(board, result.Best.Placement)
		fmt.Println("\nProposed board")
		fmt.Print(render.Board(proposed))
	}
	fmt.Printf(
		"\nComplete: %v  Elapsed: %s  Roots: %d  Replies: %d\n",
		result.Complete,
		result.Elapsed,
		result.RootEvaluated,
		result.ReplyEvaluated,
	)
	fmt.Printf("Best score: %+v\n", result.Best.Score)
	fmt.Printf("Best placement: %v\n", result.Best.Placement.Slots)
	fmt.Printf("Worst reply: %v\n", result.Best.Reply.Slots)

	if *jsonOut != "" {
		if err := statejson.WriteResult(resolvePath(*jsonOut), result); err != nil {
			exitf("write result: %v", err)
		}
	}
}

func loadBoard(scenarioName string, inputPath string, coreCards map[string]model.Card) model.Board {
	if inputPath != "" {
		board, err := statejson.Load(resolvePath(inputPath), coreCards)
		if err != nil {
			exitf("load input: %v", err)
		}
		return board
	}

	generated := scenarios.All()
	board, ok := generated[scenarioName]
	if !ok {
		exitf("unknown scenario %q", scenarioName)
	}
	board.Cards = mergeCards(coreCards, board.Cards)
	return board
}

func mergeCards(primary map[string]model.Card, fallback map[string]model.Card) map[string]model.Card {
	result := make(map[string]model.Card, len(primary)+len(fallback))
	for id, card := range fallback {
		result[id] = card
	}
	for id, card := range primary {
		result[id] = card
	}
	return result
}

func resolvePath(path string) string {
	if path == "" || filepath.IsAbs(path) {
		return path
	}
	if _, err := os.Stat(path); err == nil {
		return path
	}
	moduleRelative := filepath.Join("prototypes", "combat_solver", path)
	if _, err := os.Stat(moduleRelative); err == nil {
		return moduleRelative
	}
	return path
}

func exitf(format string, args ...any) {
	fmt.Fprintf(os.Stderr, "combat-solver: "+format+"\n", args...)
	os.Exit(1)
}
