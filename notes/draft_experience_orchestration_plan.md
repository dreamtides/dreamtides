# Dreamcaller Draft Experience Orchestration Plan

## Goal

Evaluate whether the implemented Dreamcaller draft system is achieving the
original design target from `notes/tides_v2.md` and `notes/tides_v3.md`:

- picks should feel thematically relevant to the chosen Dreamcaller
- the player should face meaningful choices between competing options
- repeated runs with the same Dreamcaller should feel cohesive without feeling
  identical
- the experience should be fun in practice, not only structurally valid

This is an evaluation plan, not a redesign plan. The purpose is to generate
evidence from sample draft traces, then decide whether the current system is
already hitting the target or where it is failing.

## Current Implementation Facts That Matter

- Dreamcaller package resolution is fixed at quest start in
  `scripts/quest_prototype/src/data/quest-content.ts`.
- The optional subset is currently chosen deterministically by
  `chooseBestCandidate()`. Repeated runs on the same Dreamcaller do **not**
  vary the resolved package unless the code changes.
- Draft variance therefore comes from sampling the fixed multiset in
  `scripts/quest_prototype/src/draft/draft-engine.ts`, not from re-rolling
  different optional subsets.
- The prototype already emits usable JSONL trace data through
  `scripts/quest_prototype/src/logging.ts` and the Vite log sink in
  `scripts/quest_prototype/vite.config.ts`.
- There is no standalone draft-simulation CLI today. Browser-driven runs are
  the supported path. If the orchestrator wants large-volume scripted traces,
  it should create a temporary throwaway harness under `/tmp`, not add repo
  code just to support the evaluation.

## Evaluation Questions

Every trace review should answer these four questions:

1. Are the offers thematically relevant to the selected Dreamcaller?
2. Do offers regularly contain at least two plausible picks that compete for
   different reasons?
3. Across repeated runs on the same Dreamcaller, is there enough variation in
   the offered cards and resulting deck texture to avoid sameness?
4. Is the draft experience actually fun, or does it feel scripted, obvious, or
   generic?

## Evidence Model

Use two lanes in parallel.

### Lane A: Quantitative Trace Analysis

Purpose: measure package relevance, pick tension proxies, and run-to-run
variance at scale.

Inputs:

- `rules_engine/tabula/rendered-cards.toml`
- `rules_engine/tabula/dreamcallers.toml`
- `scripts/quest_prototype/src/data/quest-content.ts`
- `scripts/quest_prototype/src/draft/draft-engine.ts`
- JSONL traces from prototype runs

Outputs:

- cohesion metrics per Dreamcaller
- variance metrics across repeated runs
- flagged traces for qualitative review

### Lane B: Qualitative Experience Review

Purpose: answer the subjective questions the logs cannot answer by themselves:
whether choices feel meaningful and whether the run is fun.

Inputs:

- browser-visible draft offers
- downloaded or captured JSONL logs
- the chosen Dreamcaller's tide profile and card/tide mapping

Outputs:

- pick-by-pick reviewer notes
- per-run ratings
- concrete examples of good and bad offers

## Cohort Selection

The first agent should select a **6-Dreamcaller cohort** that spans distinct
draft identities. Do not just pick six favorites. Cover at least:

- one materialize shell
- one event/control shell
- one discard or void shell
- one warrior or figment pressure shell
- one abandon shell
- one judgment or tall-board shell

Suggested starting pool:

- `Orla, Last Hearthkeeper`
- `Selise, Echo Broker`
- `Nyra, Chain Broker`
- `Veya, Figment Regent`
- `Ivera, Debt Arbiter`
- `Naeva, Solitary Arbiter`

The cohort selector may swap names if another Dreamcaller is a clearer
representative of the same bucket, but it must preserve the six coverage roles.

## Sampling Plan

Run both lanes.

### Lane A Sample Size

- 6 Dreamcallers
- 5 traces per Dreamcaller
- evaluate the first 3 draft sites per trace
- total: 30 traces, roughly 450 picks if each site completes 5 picks

This is enough to test same-Dreamcaller variance even though package
resolution is deterministic.

### Lane B Sample Size

- 1 browser-reviewed trace per Dreamcaller in the 6-Dreamcaller cohort
- plus 1 extra browser-reviewed trace for the 2 Dreamcallers that look most
  suspicious after Lane A
- total: 8 qualitative traces

This keeps the subjective pass small enough to finish while still revisiting
the likely failures.

## Agent Topology

### 1. Cohort Selector

Responsibilities:

- choose the 6-Dreamcaller cohort
- write a short rationale for each choice
- record each Dreamcaller’s mandatory and optional tides

Output:

- `/tmp/draft-experience-eval/cohort.md`

### 2. Briefing Agent

Responsibilities:

- build a one-page brief for each selected Dreamcaller
- summarize what cards should feel “on plan” based on tide overlap
- note any special risks, such as overly generic support tides

Output:

- `/tmp/draft-experience-eval/briefs/<dreamcaller-id>.md`

### 3. Trace Runner Agents

Responsibilities:

- own one Dreamcaller each
- generate the assigned traces
- save raw logs and a compact manifest per run

Preferred run method:

- use the prototype in browser mode via `agent-browser`

Allowed scaling fallback:

- create a temporary TS harness under `/tmp` that imports the prototype draft
  modules and writes JSONL-like trace records

Outputs:

- `/tmp/draft-experience-eval/traces/<dreamcaller-id>/run-XX.jsonl`
- `/tmp/draft-experience-eval/traces/<dreamcaller-id>/run-XX.md`

Each `run-XX.md` must state:

- selected Dreamcaller
- whether the run was browser-played or harness-generated
- how many draft sites were completed
- any blockers or ambiguity

### 4. Metrics Agents

Responsibilities:

- parse raw traces
- join offered and picked card numbers back to card data and Dreamcaller tides
- compute cohesion and variance metrics

Outputs:

- `/tmp/draft-experience-eval/metrics/<dreamcaller-id>.md`
- `/tmp/draft-experience-eval/metrics/summary.csv`

### 5. Qualitative Review Agents

Responsibilities:

- review the browser traces only
- score each run against the rubric below
- cite specific offers and picks, not vague impressions

Outputs:

- `/tmp/draft-experience-eval/reviews/<dreamcaller-id>-run-XX.md`

### 6. Synthesis Agent

Responsibilities:

- combine metrics and subjective reviews
- decide whether the system is meeting the goal
- identify the most likely causes when it fails

Output:

- `/tmp/draft-experience-eval/final_report.md`

### 7. Red-Team Agent

Responsibilities:

- read only the final metrics and flagged traces
- look for overclaiming, weak evidence, or “technically cohesive but boring”
  failure modes

Output:

- `/tmp/draft-experience-eval/red_team.md`

## Trace Generation Procedure

### Prototype Setup

Run once at the start:

```bash
cd /Users/dthurn/dreamtides/scripts/quest_prototype
npm install
npm run dev
```

For browser-driven agents:

```bash
agent-browser open http://localhost:5173
agent-browser wait --load networkidle
```

Trace sources:

- `scripts/quest_prototype/logs/quest-log.jsonl`
- downloaded `quest-log-*.jsonl` files from the prototype HUD

### Browser Run Rules

- Play until 3 draft sites are completed, unless the run ends or becomes
  blocked earlier.
- Use the debug screen when needed to confirm selected tides or pool state, but
  judge the normal player-facing experience separately.
- Record why each pick was chosen, especially when two or more options looked
  viable.
- If the target Dreamcaller is not immediately available on the selection
  screen, refresh until it appears. Log refresh count in the run notes.

### Harness Run Rules

- Do not modify the repo to add a new runner.
- Any helper scripts must live under `/tmp/draft-experience-eval/tools/`.
- The harness should mirror current runtime behavior, including deterministic
  package resolution and weighted 4-unique-card offers.
- The harness does not answer the fun question by itself. It exists only to
  expand sample size for cohesion and variance metrics.

## Metrics

Each Metrics Agent should compute these metrics per Dreamcaller.

### Relevance

- `offer_package_hit_rate`: percent of offers with at least 2 cards that share
  any selected package tide
- `offer_strong_hit_rate`: percent of offers with at least 1 card sharing 2 or
  more selected package tides
- `pick_package_hit_rate`: percent of picks that share any selected package tide
- `pick_strong_hit_rate`: percent of picks that share 2 or more selected tides
- `generic_filler_rate`: percent of offers dominated by generic utility cards
  with weak package overlap

### Choice Tension

- `multi_live_pick_rate`: percent of offers with at least 2 plausible on-plan
  options
- `forced_pick_rate`: percent of offers where only 1 option looks remotely
  attractive
- `role_competition_rate`: percent of offers where the live options pull the
  drafter in different useful directions rather than duplicating the same role

These are proxies. The qualitative lane decides whether the choices actually
felt meaningful.

### Variance

- offer overlap across the 5 traces for the same Dreamcaller
- picked-card overlap across the 5 traces for the same Dreamcaller
- deck-shape variance by cost curve and card type
- repeated “same first 8 picks” failures

### Failure Flags

Flag a Dreamcaller if any of these happen repeatedly:

- too many offers with 0 or 1 on-plan cards
- picks repeatedly collapse into obvious single-choice decisions
- repeated traces converge on nearly identical early decks
- the drafter must take off-plan generic filler to stay functional

## Qualitative Review Rubric

Every browser-reviewed run should score these from 1 to 5.

- `dreamcaller_fit`: did the cards shown feel like they belonged to the chosen
  Dreamcaller?
- `choice_tension`: were there real tradeoffs between competing options?
- `novelty`: did the run feel distinct from other runs on the same Dreamcaller?
- `fun`: was the draft enjoyable, surprising, and worth replaying?

Each review must also include:

- the best offer in the run
- the weakest offer in the run
- one example of a meaningful choice
- one example of an offer that felt flat, generic, or misleading

## Orchestrator Sequence

1. Start the dev server and confirm logging is working.
2. Spawn the Cohort Selector.
3. Spawn the Briefing Agent after cohort selection finishes.
4. Spawn 6 Trace Runner Agents in parallel, one per Dreamcaller.
5. After the first trace batch completes, spawn 6 Metrics Agents in parallel.
6. Use the first metrics pass to identify the 2 most suspicious Dreamcallers.
7. Spawn 2 additional browser Trace Runner Agents for those Dreamcallers.
8. Spawn Qualitative Review Agents for all browser traces.
9. Spawn the Synthesis Agent.
10. Spawn the Red-Team Agent.
11. Merge `final_report.md` and `red_team.md` into the final verdict.

## What Counts As Success

The system is probably meeting the goal if most sampled Dreamcallers show all
of the following:

- most offers contain multiple on-plan cards
- the drafter regularly chooses between at least two reasonable directions
- repeated runs on the same Dreamcaller diverge in noticeable ways without
  losing identity
- subjective reviewers describe the draft as cohesive and fun more often than
  not

## What Counts As Failure

The system is not meeting the goal if one or more of these dominate:

- cohesion exists only in metadata, not in the cards a reviewer actually sees
- most offers have an obvious correct pick
- repeated runs on the same Dreamcaller feel nearly identical
- the “fun” score is low even when cohesion metrics look acceptable

## Expected Final Deliverable

The final report should not stop at “pass” or “fail.” It should answer:

- which Dreamcallers are working well
- which Dreamcallers are weak
- whether the main problem is relevance, choice tension, or variance
- whether the failure looks data-driven, algorithmic, or purely experiential
- the top 3 follow-up changes that would most likely improve the system
