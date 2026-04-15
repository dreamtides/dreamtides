# Dreamcaller Draft Experience Black-Box Orchestration Plan

## Goal

Evaluate whether the implemented Dreamcaller draft system is achieving the
intended player experience:

- the chosen Dreamcaller should make the draft feel cohesive
- draft offers should present meaningful choices between competing options
- repeated runs on the same Dreamcaller should stay recognizable without
  feeling identical
- the draft should be fun in practice

This plan is intentionally **black-box**.

- evaluation agents must not read implementation files
- evaluation agents must not inspect tides, package membership, or debug data
- evaluation agents must not use the web UI
- evaluation agents must interact only through a CLI runner that exposes the
  same information a player would see: Dreamcaller names and text, card names,
  card text, visible offer contents, picks, and end-of-run summaries

## Core Rule

Separate the work into two worlds.

### World 1: Infrastructure

These agents are allowed to touch code, because they exist only to create the
black-box surface.

They may:

- build a temporary CLI harness around the quest prototype
- generate raw traces
- store transcripts

They may not:

- evaluate whether the draft is good
- write conclusions about cohesion or fun

### World 2: Evaluation

These agents are not allowed to read implementation files or internal data.

They may see only:

- CLI prompt/response transcripts
- Dreamcaller public text
- card public text
- pick sequences
- final drafted decklists
- simple run metadata such as pick count and run id

They may not see:

- tides
- package composition
- weighting logic
- pool construction
- hidden debug info
- source code

## Required First Step

The current prototype does not expose a player-facing CLI draft runner out of
the box. The orchestration therefore starts by creating a **temporary runner**
under `/tmp`, not in the repo.

The runner should:

- launch a draft run from the existing quest prototype logic
- print only player-visible information
- allow Dreamcaller selection by index
- print each offer in plain text
- accept picks by index
- write a full transcript to disk
- write a machine-readable JSON trace to disk
- support seeded runs for repeatability

The runner must not print or export any hidden implementation metadata.

## CLI Runner Contract

Every evaluation agent should interact with a single command with this shape:

```bash
node /tmp/draft-experience-eval/tools/draft_cli.mjs --seed 12345
```

The CLI should behave like this:

1. Print the offered Dreamcallers:
   - name
   - awakening
   - rendered text
2. Accept a Dreamcaller choice.
3. For each draft pick, print:
   - pick number
   - current deck summary
   - four offered cards
   - for each card: name, cost, type, subtype if any, rarity, fast if visible,
     spark if visible, and rendered text
4. Accept a pick choice.
5. Continue until the configured stopping point.
6. Print a final deck summary.
7. Save both:
   - a human-readable transcript
   - a JSON trace with only public information

Recommended stop point:

- first 15 picks, or 3 draft sites, whichever maps more cleanly onto current
  prototype behavior

## Artifacts

All run artifacts should live under:

```text
/tmp/draft-experience-eval/
  tools/
    draft_cli.mjs
  cohort.md
  transcripts/
    <dreamcaller-slug>/
      run-01.txt
      run-01.json
      run-02.txt
      run-02.json
  reviews/
    <dreamcaller-slug>/
      run-01-review-a.md
      run-01-review-b.md
  summaries/
    cohort_summary.md
    variance_summary.md
    final_report.md
    red_team.md
```

## Dreamcaller Cohort

Use a six-Dreamcaller cohort, but choose it from **public Dreamcaller text
only**.

The selector should read only the CLI-exposed Dreamcaller list and choose six
that appear to promise clearly different draft experiences.

Selection criteria:

- avoid six variants of the same apparent strategy
- prefer Dreamcallers whose text implies distinct incentives
- include both narrow-looking and broad-looking Dreamcallers
- include at least two Dreamcallers that seem risky or ambiguous from their text

The selector should not know whether the underlying implementation supports
those promises well.

Output:

- `/tmp/draft-experience-eval/cohort.md`

The file should record:

- chosen Dreamcallers
- one-paragraph rationale for each, written only from public text
- which Dreamcallers seem most likely to produce cohesive drafts
- which seem most likely to fail

## Sampling Plan

### Primary Pass

- 6 Dreamcallers
- 5 runs per Dreamcaller
- total: 30 black-box runs

### Review Depth

Each run should receive:

- 2 independent transcript reviews

This gives:

- 60 independent reviews

### Suspicion Pass

After the first pass, identify the 2 Dreamcallers with the weakest results and
run:

- 3 additional runs each

This adds:

- 6 more runs
- 12 more independent reviews

## Agent Topology

### 1. Harness Builder

Type: infrastructure

Responsibilities:

- create the temporary CLI runner in `/tmp/draft-experience-eval/tools/`
- ensure it exposes only public information
- verify it can run seeded drafts and save transcripts

Output:

- `draft_cli.mjs`
- `runner_readme.md`

### 2. Cohort Selector

Type: evaluation

Responsibilities:

- inspect Dreamcaller public text from the CLI
- choose the six-Dreamcaller cohort

Output:

- `cohort.md`

### 3. Run Producer Agents

Type: infrastructure

Responsibilities:

- own one Dreamcaller each
- generate the required seeded runs through the CLI
- save transcript and JSON output
- never write evaluative commentary

Output per run:

- `run-XX.txt`
- `run-XX.json`

### 4. First-Pass Review Agents

Type: evaluation

Responsibilities:

- review transcript text only
- never inspect JSON if it contains anything beyond public information
- judge the run as though it were a real player session log

Each run gets two separate reviewers.

Output per review:

- `run-XX-review-a.md`
- `run-XX-review-b.md`

### 5. Variance Analyst

Type: evaluation

Responsibilities:

- compare only public run artifacts
- assess whether repeated runs on the same Dreamcaller feel too similar

Output:

- `variance_summary.md`

### 6. Synthesis Agent

Type: evaluation

Responsibilities:

- combine the paired reviews and variance analysis
- decide whether each Dreamcaller is working
- identify recurring failure modes

Output:

- `final_report.md`

### 7. Red-Team Agent

Type: evaluation

Responsibilities:

- challenge the synthesis
- look for false confidence
- ask whether “cohesive” is being overstated when runs are actually repetitive
- ask whether “meaningful choices” is being overstated when picks are obvious

Output:

- `red_team.md`

## Reviewer Rubric

Each transcript reviewer should answer the same questions on a 1-5 scale.

### 1. Dreamcaller Fit

Does the draft actually feel like it is supporting the chosen Dreamcaller’s
promise, based only on what the Dreamcaller text says?

### 2. Choice Tension

How often do offers contain at least two cards that look meaningfully
defensible for different reasons?

### 3. Novelty

Compared with other runs for the same Dreamcaller, does this run seem likely to
develop in a distinct direction?

### 4. Fun

Would a player likely enjoy drafting this run?

### 5. Trust

Does the system seem to understand what kind of deck the player is trying to
build, or does it feel random/generic?

Each review must also include:

- the best offer in the run
- the weakest offer in the run
- one pick that felt genuinely hard
- one pick that felt automatic
- one sentence describing the apparent deck identity by the end of the run

## What Reviewers Are Allowed To Use

Allowed:

- Dreamcaller text
- card text
- card names
- card costs and visible stats
- offer order
- pick history
- final decklist

Not allowed:

- internal terms from implementation docs
- hidden tags
- source code
- devtools
- debug screens
- any “this probably maps to package X” reasoning

If a reviewer starts inferring internals, that review is invalid and should be
redone.

## Transcript Format Requirements

To make blind review easy, every transcript should have this structure:

```text
RUN <id>
SEED <seed>
DREAMCALLER
  [1] Name - Awakening N
  Text...

PICK 1
CURRENT DECK
  <empty>
OFFER
  [1] Card name | Cost | Type | Rarity
      Text...
  [2] ...
  [3] ...
  [4] ...
CHOICE
  [picked index]
RATIONALE
  <optional player rationale if using an active review run>
```

For bulk runs, the rationale can be omitted at generation time and added by the
reviewer instead.

## How Picks Should Be Generated

Use two different pick modes.

### Mode A: Naturalist Picks

The runner agent chooses the card that seems best for the apparent deck based
only on public information.

Purpose:

- simulate a player earnestly trying to build a coherent deck

### Mode B: Counterfactual Notes

For selected runs, the review agent should also note:

- what the second-best pick was
- why it was tempting

Purpose:

- measure whether the offer contained a real choice rather than one obvious
  answer

## Evaluation Outputs

### Per-Run Review

Each review file should end with:

- `fit: N/5`
- `choice_tension: N/5`
- `novelty: N/5`
- `fun: N/5`
- `trust: N/5`
- `verdict: strong | mixed | weak`

### Per-Dreamcaller Summary

For each Dreamcaller, the synthesis should answer:

- Do runs feel cohesive?
- Do runs present meaningful choices?
- Do repeated runs diverge enough?
- Is the experience fun enough to justify replaying?

### Final Report

The final report should rank failure modes, not just Dreamcallers.

It should explicitly decide whether the biggest problem is:

- weak thematic relevance
- low choice tension
- low replay variance
- low fun despite apparent relevance

## Orchestrator Sequence

1. Spawn the Harness Builder.
2. Verify the runner exposes only public information.
3. Spawn the Cohort Selector.
4. Spawn 6 Run Producer Agents in parallel, one per Dreamcaller.
5. After the first 30 runs are complete, spawn paired First-Pass Review Agents.
6. Spawn the Variance Analyst.
7. Identify the 2 weakest Dreamcallers.
8. Spawn 2 more Run Producer Agents for the suspicion pass.
9. Spawn paired reviewers for those additional runs.
10. Spawn the Synthesis Agent.
11. Spawn the Red-Team Agent.
12. Merge `final_report.md` and `red_team.md` into the final verdict.

## Success Criteria

The system is meeting the goal if most reviewed Dreamcallers show all of the
following:

- reviewers consistently describe the runs as fitting the Dreamcaller’s public
  promise
- offers regularly create real tension between at least two choices
- repeated runs do not collapse into near-identical drafts
- fun scores are solid, not merely acceptable

## Failure Criteria

The system is failing the goal if one or more of these dominate:

- the Dreamcaller promise is clear, but the offers do not support it
- offers often contain one obvious pick and three irrelevant ones
- repeated runs for the same Dreamcaller feel too similar
- reviewers describe the experience as coherent but boring
- reviewers cannot tell what kind of deck the system is trying to help them
  draft
