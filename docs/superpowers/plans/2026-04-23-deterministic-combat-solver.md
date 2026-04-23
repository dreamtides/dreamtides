# Deterministic Combat Solver Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a standalone Go CLI prototype that solves Core 11
public-information combat positioning over the next two Judgment phases.

**Architecture:** The prototype lives under `prototypes/combat_solver` as an
isolated Go module. It loads Core 11 character data from existing TOML, models
the staggered battlefield with pure transition functions, and runs deterministic
two-ply minimax with a 100ms budget and terminal/JSON output.

**Tech Stack:** Go CLI, `github.com/pelletier/go-toml/v2`, standard-library
JSON/flag/testing packages, existing Dreamtides TOML data.

______________________________________________________________________

## File Structure

| Path                                                       | Purpose                                              |
| ---------------------------------------------------------- | ---------------------------------------------------- |
| `prototypes/combat_solver/go.mod`                          | Isolated Go module and TOML parser dependency        |
| `prototypes/combat_solver/cmd/combat-solver/main.go`       | CLI flags, input selection, solve invocation, output |
| `prototypes/combat_solver/internal/model/model.go`         | Player, card, character, board definitions           |
| `prototypes/combat_solver/internal/model/slots.go`         | 4-front/5-back slot helpers and support topology     |
| `prototypes/combat_solver/internal/cards/loader.go`        | Core 11 TOML loading and support-effect tagging      |
| `prototypes/combat_solver/internal/engine/rules.go`        | Effective spark, end-of-turn gains, Judgment         |
| `prototypes/combat_solver/internal/solver/placements.go`   | Legal placement enumeration                          |
| `prototypes/combat_solver/internal/solver/score.go`        | Lexicographic objective                              |
| `prototypes/combat_solver/internal/solver/solve.go`        | Two-ply minimax, budget tracking, pruning            |
| `prototypes/combat_solver/internal/statejson/state.go`     | JSON board input and result output                   |
| `prototypes/combat_solver/internal/scenarios/scenarios.go` | Representative generated boards                      |
| `prototypes/combat_solver/internal/render/ascii.go`        | Terminal board rendering                             |

## Task 1: Toolchain Preflight And Go Module Scaffold

**Files:**

- Create: `prototypes/combat_solver/go.mod`

- Create: `prototypes/combat_solver/cmd/combat-solver/main.go`

- [ ] **Step 1: Verify or install Go**

Run:

```bash
go version
```

If the command is missing on macOS, run:

```bash
brew install go
go version
```

Expected: `go version` prints the installed toolchain version.

- [ ] **Step 2: Initialize the isolated module**

Run:

```bash
mkdir -p prototypes/combat_solver/cmd/combat-solver
cd prototypes/combat_solver
go mod init dreamtides/prototypes/combat_solver
go get github.com/pelletier/go-toml/v2
```

Expected: `go.mod` and `go.sum` exist.

- [ ] **Step 3: Add the scaffold CLI**

Create `cmd/combat-solver/main.go` with package `main`, a `--version` flag, and
this behavior:

- `go run ./cmd/combat-solver --version` prints `combat-solver prototype`

- running without flags prints `combat-solver: implementation not wired yet` to
  stderr and exits with code `2`

- [ ] **Step 4: Verify and commit**

Run:

```bash
cd prototypes/combat_solver
go test ./...
go run ./cmd/combat-solver --version
git add .
git commit -m "Add combat solver Go scaffold" -m \
  "Create an isolated Go module for the deterministic combat solver prototype with a minimal CLI entry point."
```

## Task 2: Board Model And Support Topology

**Files:**

- Create: `internal/model/model.go`

- Create: `internal/model/slots.go`

- Create: `internal/model/slots_test.go`

- [ ] **Step 1: Define model types**

In `model.go`, define:

```go
type Player int
const (PlayerOne Player = iota; PlayerTwo)
func (p Player) Opponent() Player

type SupportEffect int
const (SupportNone SupportEffect = iota; SupportNocturneStrummer; SupportRuneboundChampion)

type Card struct { ID, Name, RenderedText string; BaseSpark int; SupportEffect SupportEffect }
type Character struct { ID, CardID, Name string; Owner Player; StoredSpark int; CanReposition bool }
type Board struct { Active Player; Slots [2][9]string; Characters map[string]Character; Cards map[string]Card }
func (b Board) Clone() Board
func (b Board) CharacterAt(player Player, slot int) (Character, bool)
```

- [ ] **Step 2: Define slot helpers**

In `slots.go`, define:

```go
const (FrontSlots = 4; BackSlots = 5; TotalSlots = 9)
func FrontSlot(index int) int
func BackSlot(index int) int
func IsFront(slot int) bool
func IsBack(slot int) bool
func SupportedFrontSlots(backSlot int) []int
func SupportingBackSlots(frontSlot int) []int
```

Expected topology:

- `SupportedFrontSlots(0..4)` returns `[0]`, `[0,1]`, `[1,2]`, `[2,3]`, `[3]`

- `SupportingBackSlots(0..3)` returns `[4,5]`, `[5,6]`, `[6,7]`, `[7,8]`

- [ ] **Step 3: Add topology tests**

In `slots_test.go`, add `TestSupportedFrontSlots` and `TestSupportingBackSlots`
covering every slot listed above.

- [ ] **Step 4: Verify and commit**

Run:

```bash
cd prototypes/combat_solver
go test ./internal/model
git add internal/model
git commit -m "Add combat solver board model" -m \
  "Define player, character, card, board, and staggered support topology primitives for the Go combat solver."
```

## Task 3: Runtime Core 11 Card Loader

**Files:**

- Create: `internal/cards/loader.go`

- Create: `internal/cards/loader_test.go`

- [ ] **Step 1: Implement TOML loader**

In `loader.go`, implement:

```go
func LoadCore11Characters(cardListsPath string, renderedCardsPath string) (map[string]model.Card, error)
```

Requirements:

- Decode `[[card-lists]]` entries from `card-lists.toml`

- Keep only `list-name = "Core 11"` and `list-type = "BaseCardId"`

- Decode `[[cards]]` entries from `rendered-cards.toml`

- Keep only Core 11 entries where `card-type = "Character"`

- Parse integer `spark`; tolerate empty-string spark on ignored event cards

- Tag `Nocturne Strummer` or rendered text `Supported characters gain +2 spark.`
  as `SupportNocturneStrummer`

- Tag `Runebound Champion` or rendered text containing
  `each supporting character gains +1 spark` as `SupportRuneboundChampion`

- [ ] **Step 2: Add loader tests**

In `loader_test.go`, create temp TOML fixtures with one Core 11 character and
one Core 11 event. Assert the loader returns only the character, preserves base
spark, and tags `Nocturne Strummer` correctly.

- [ ] **Step 3: Verify against fixtures and repo TOML**

Run:

```bash
cd prototypes/combat_solver
go test ./internal/cards
go test ./...
git add internal/cards go.mod go.sum
git commit -m "Load Core 11 character cards for combat solver" -m \
  "Read Core 11 entries from Tabula TOML, filter to characters, and tag the prototype support abilities used by the solver."
```

## Task 4: Pure Combat Transition Rules

**Files:**

- Create: `internal/engine/rules.go`

- Create: `internal/engine/rules_test.go`

- [ ] **Step 1: Implement rules API**

In `rules.go`, define:

```go
type DissolvedCharacter struct { ID, Name string; Spark int }
type Outcome struct { Points [2]int; Dissolved [2][]DissolvedCharacter }

func EffectiveSpark(board model.Board, player model.Player, slot int) int
func ApplyEndOfTurnSupportGains(board *model.Board, player model.Player)
func ResolveJudgment(board *model.Board, attacker model.Player) Outcome
func SurvivingEffectiveSparks(board model.Board, player model.Player) []int
```

Rules:

- `EffectiveSpark` returns stored spark plus `+2` per supporting back-rank
  `Nocturne Strummer` only for front-rank characters

- `ApplyEndOfTurnSupportGains` scans front-rank `Runebound Champion` characters
  and increments each occupied supporting back slot's stored spark by `1`

- `ResolveJudgment` resolves lanes `F0-F3`; lower effective spark dissolves,
  ties dissolve both, and unblocked attackers score effective spark

- Dissolved characters are removed from `Slots` and `Characters`

- [ ] **Step 2: Add transition tests**

In `rules_test.go`, add:

- `TestNocturneStrummerAddsEffectiveSpark`

- `TestRuneboundChampionEndOfTurnGain`

- `TestJudgmentDissolvesLowerSparkAndScoresUnblocked`

- `TestJudgmentTieDissolvesBoth`

- [ ] **Step 3: Verify and commit**

Run:

```bash
cd prototypes/combat_solver
go test ./internal/engine
git add internal/engine
git commit -m "Add combat solver transition rules" -m \
  "Implement effective support spark, Runebound end-of-turn gains, and lane Judgment resolution as pure Go transitions."
```

## Task 5: Legal Placement Generation

**Files:**

- Create: `internal/solver/placements.go`

- Create: `internal/solver/placements_test.go`

- [ ] **Step 1: Implement placement API**

In `placements.go`, define:

```go
type Placement struct { Player model.Player; Slots [9]string }
func GeneratePlacements(board model.Board, player model.Player) []Placement
func ApplyPlacement(board model.Board, placement Placement) model.Board
func (p Placement) Key() string
```

Requirements:

- Generate placements for only the requested player's alive characters

- Preserve unmovable characters in their current slots

- Assign each movable character to one free slot on that player's battlefield

- Include the current slot naturally as one legal destination

- Return placements in deterministic order by character name, then character ID,
  then slot index

- `ApplyPlacement` must not mutate the input board or the opponent slots

- [ ] **Step 2: Add placement tests**

In `placements_test.go`, add:

- `TestGeneratePlacementsKeepsImmovableCharactersFixed`

- `TestGeneratePlacementsCountsPartialBoardChoices`

- `TestApplyPlacementMovesOnlyChosenPlayer`

- `TestPlacementKeyIsStable`

- [ ] **Step 3: Verify and commit**

Run:

```bash
cd prototypes/combat_solver
go test ./internal/solver -run Placement
git add internal/solver/placements.go internal/solver/placements_test.go
git commit -m "Generate legal combat solver placements" -m \
  "Enumerate deterministic per-player board assignments while preserving immovable characters and partial-board constraints."
```

## Task 6: Lexicographic Score And Two-Ply Minimax

**Files:**

- Create: `internal/solver/score.go`

- Create: `internal/solver/solve.go`

- Create: `internal/solver/score_test.go`

- Create: `internal/solver/solve_test.go`

- [ ] **Step 1: Implement score API**

In `score.go`, define:

```go
type Score struct {
    OwnSurvivors []int
    OpponentDissolved []int
    OwnPoints int
    OpponentPoints int
    OpponentPointsTerm int
}
type Tally struct { Points [2]int; Dissolved [2][]engine.DissolvedCharacter }
func FinalScore(board model.Board, perspective model.Player, tally Tally) Score
func CompareScore(left Score, right Score) int
func AddOutcome(tally *Tally, outcome engine.Outcome)
```

Comparison order:

1. final allied survivor effective sparks, descending
2. opponent dissolved sparks, descending
3. own points
4. negative opponent points

- [ ] **Step 2: Implement minimax API**

In `solve.go`, define:

```go
type Options struct { Budget time.Duration; MaxRanked int }
type Candidate struct { Placement Placement; Reply Placement; Score Score }
type Result struct {
    Complete bool
    Elapsed string
    RootEvaluated int
    ReplyEvaluated int
    Best Candidate
    Ranked []Candidate
    TimedOutAtRoot bool
    TimedOutAtReply bool
}
func Solve(board model.Board, options Options) Result
```

Search sequence per root placement:

1. apply current-player placement
2. apply current-player end-of-turn support gains
3. resolve opponent Judgment
4. enumerate opponent placements
5. for each reply, apply opponent placement and end-of-turn gains
6. resolve current-player Judgment
7. score final board from current-player perspective
8. keep the opponent reply with the lowest score

- [ ] **Step 3: Add score and solver tests**

In `score_test.go`, add `TestScoreCompareIsLexicographic`.

In `solve_test.go`, add:

- `TestSolveChoosesPlacementThatPreservesCharacter`

- `TestSolveUsesWorstCaseOpponentReply`

- `TestSolveReturnsRankedCandidates`

- [ ] **Step 4: Verify and commit**

Run:

```bash
cd prototypes/combat_solver
go test ./internal/solver
go test ./...
git add internal/solver
git commit -m "Add two-ply combat positioning minimax" -m \
  "Score final boards lexicographically and solve current-player placements against worst-case opponent reposition replies."
```

## Task 7: JSON Input, Generated Scenarios, And ASCII Rendering

**Files:**

- Create: `internal/statejson/state.go`

- Create: `internal/scenarios/scenarios.go`

- Create: `internal/render/ascii.go`

- Create: `internal/scenarios/scenarios_test.go`

- [ ] **Step 1: Implement JSON input/output**

In `statejson/state.go`, define:

```go
type File struct {
    Active string
    PlayerOne [9]*CharacterInput
    PlayerTwo [9]*CharacterInput
}
type CharacterInput struct {
    ID string
    CardID string
    Name string
    StoredSpark *int
    CanReposition bool
}
func Load(path string, cards map[string]model.Card) (model.Board, error)
func WriteResult(path string, value any) error
```

Accepted active-player strings: `player_one`, `p1`, `player_two`, `p2`; default
to player one if omitted.

- [ ] **Step 2: Implement generated scenarios**

In `scenarios.go`, define:

```go
func All() map[string]model.Board
```

Return at least these scenario names:

- `support`
- `high-spark`
- `sparse`
- `full-stress`
- `mobility-mixed`

Generated scenarios can use fixture card IDs such as `strummer`, `champion`,
`ring`, `direwolf`, `witness`, and `colossus`; each board must include a
complete `Cards` map for those fixture IDs.

- [ ] **Step 3: Implement ASCII rendering**

In `render/ascii.go`, define:

```go
func Board(board model.Board) string
```

Output must include each player, front row `F0-F3`, back row `B0-B4`, character
names, and stored spark values.

- [ ] **Step 4: Add tests and commit**

In `scenarios_test.go`, add `TestGeneratedScenariosAreNamedAndNonEmpty`.

Run:

```bash
cd prototypes/combat_solver
go test ./internal/scenarios ./internal/render
go test ./...
git add internal/statejson internal/scenarios internal/render
git commit -m "Add combat solver inputs and terminal rendering" -m \
  "Support JSON board input, representative generated scenarios, and ASCII board diagrams for the combat solver CLI."
```

## Task 8: Wire The CLI End To End

**Files:**

- Modify: `cmd/combat-solver/main.go`

- [ ] **Step 1: Implement CLI flags**

Replace the scaffold CLI with flags:

```text
--scenario support
--input ""
--json-out ""
--budget 100ms
--rank 5
--card-lists ../../rules_engine/tabula/card-lists.toml
--rendered-cards ../../rules_engine/tabula/rendered-cards.toml
```

Behavior:

- load Core 11 cards through `cards.LoadCore11Characters`

- use JSON input when `--input` is set

- otherwise select a generated scenario by name

- merge loaded Core 11 cards over generated fixture cards when using a scenario

- print initial ASCII board

- run `solver.Solve`

- print complete flag, elapsed time, evaluated counts, best score, best
  placement, and worst reply

- write JSON result when `--json-out` is set

- [ ] **Step 2: Verify CLI output**

Run:

```bash
cd prototypes/combat_solver
go run ./cmd/combat-solver --scenario support --budget 100ms --rank 3
go run ./cmd/combat-solver --scenario sparse --json-out /tmp/combat-solver-result.json
test -s /tmp/combat-solver-result.json
git add cmd/combat-solver/main.go
git commit -m "Wire combat solver CLI" -m \
  "Load Core 11 cards, select generated or JSON board input, run the minimax solver, and print terminal plus optional JSON output."
```

Expected: output includes `Initial board`, `Complete:`, `Best score:`,
`Best placement:`, and `Worst reply:`.

## Task 9: Budget Pruning, Completeness Reporting, And Stress Validation

**Files:**

- Modify: `internal/solver/solve.go`

- Create: `internal/solver/pruning_test.go`

- [ ] **Step 1: Add timeout and pruning tests**

In `pruning_test.go`, add:

- `TestSolveReportsIncompleteWhenBudgetExpires` using `full-stress` and a
  `time.Nanosecond` budget

- `TestFullStressReturnsBestCandidateWithHundredMillisecondBudget`

- `TestRootReplySearchStopsWhenOpponentCanForceWorseThanBest`

- [ ] **Step 2: Add alpha-style reply cutoff**

In `evaluateRoot`, accept `bestRoot *Score`. While evaluating opponent replies,
return early when the current worst reply is already less than or equal to
`bestRoot`, because the opponent can force a result that cannot improve the
current best root.

- [ ] **Step 3: Add optimistic upper-bound root skip**

Before enumerating opponent replies for a root, compute a safe optimistic score:
all current allied survivors remain alive, all remaining opponent characters are
treated as dissolved, own points stay at current tally or better, and opponent
points do not improve. Skip the root if this upper bound cannot beat the current
best root.

- [ ] **Step 4: Verify and commit**

Run:

```bash
cd prototypes/combat_solver
go test ./internal/solver
go run ./cmd/combat-solver --scenario full-stress --budget 100ms --rank 3
git add internal/solver
git commit -m "Add combat solver budget pruning" -m \
  "Report incomplete searches clearly and prune minimax branches that cannot improve the current best root placement."
```

Expected: the stress command exits successfully. It may print `Complete: false`,
but it must print elapsed time, evaluated root count, and a best fully evaluated
placement when at least one root finished.

## Task 10: Final Validation And Documentation Notes

**Files:**

- Create: `prototypes/combat_solver/README.md`

- Modify:
  `docs/superpowers/specs/2026-04-23-deterministic-combat-solver-design.md` only
  if implementation discovers a required design correction

- [ ] **Step 1: Add README**

Create `README.md` documenting:

- generated scenario command:
  `go run ./cmd/combat-solver --scenario support --budget 100ms --rank 5`

- JSON board command:
  `go run ./cmd/combat-solver --input board.json --json-out result.json`

- modeled scope: in-play characters, no hands, no card plays, no Dreamwell, no
  draw, no stack responses, no events

- turn sequence: current reposition, opponent Judgment, opponent worst-case
  reposition, current next Judgment

- [ ] **Step 2: Run final validation**

Run:

```bash
cd prototypes/combat_solver
go test ./...
go run ./cmd/combat-solver --scenario support --budget 100ms --rank 5
go run ./cmd/combat-solver --scenario sparse --json-out /tmp/combat-solver-result.json
cd ../..
just fmt
just review
```

Expected: Go tests and CLI checks pass. If `just review` still fails on the
known unrelated Rust style-validator item-order violations, record the exact
failure in the final implementation handoff and do not modify those Rust files
unless the task owner explicitly expands scope.

- [ ] **Step 3: Commit**

```bash
git add prototypes/combat_solver docs/superpowers/specs/2026-04-23-deterministic-combat-solver-design.md
git commit -m "Document combat solver prototype usage" -m \
  "Add usage notes for the standalone Go combat solver and record any implementation-driven spec corrections."
```
