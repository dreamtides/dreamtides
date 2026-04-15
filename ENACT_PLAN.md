**Skill reference:** `~/.llms/skills/enact/SKILL.md`

# Dreamcaller Draft Feel Evaluation

## Original request

```text
The goal of notes/tides_v2.md and notes/tides_v2.md was to design a system where you can pick a dreamcaller and then you see draft picks that feel thematically relevant -- each draft feels cohesive and yet it is not the same each time.

We have no fully implemented this system in rules_engine/tabula/rendered-cards.toml rules_engine/tabula/dreamcallers.toml and scripts/quest_prototype

What I would like now is for you to write a subagent orchestration plan where we run subagents that generate sample draft traces and analyzes whether we are actually meeting this goal in practice. The subagents should evaluate if they feel like they are making meaningful choices between competing options, whether they are seeing cards that fit their dreamcaller, and whether the experience is fun overall.
```

## Relevant documents

- [notes/tides_v2.md](/Users/dthurn/dreamtides/notes/tides_v2.md)
- [notes/tides_v2_orchestration_plan.md](/Users/dthurn/dreamtides/notes/tides_v2_orchestration_plan.md)
- [notes/pack_quest_simulation_results.md](/Users/dthurn/dreamtides/notes/pack_quest_simulation_results.md)
- [rules_engine/tabula/rendered-cards.toml](/Users/dthurn/dreamtides/rules_engine/tabula/rendered-cards.toml)
- [rules_engine/tabula/dreamcallers.toml](/Users/dthurn/dreamtides/rules_engine/tabula/dreamcallers.toml)
- [scripts/quest_prototype/PLAN.md](/Users/dthurn/dreamtides/scripts/quest_prototype/PLAN.md)
- [scripts/quest_prototype/package.json](/Users/dthurn/dreamtides/scripts/quest_prototype/package.json)
- [docs/quest_prototype/qa_tooling.md](/Users/dthurn/dreamtides/docs/quest_prototype/qa_tooling.md)
- [scripts/quest_prototype/src/data/quest-content.ts](/Users/dthurn/dreamtides/scripts/quest_prototype/src/data/quest-content.ts)
- [scripts/quest_prototype/src/data/dreamcaller-selection.ts](/Users/dthurn/dreamtides/scripts/quest_prototype/src/data/dreamcaller-selection.ts)
- [scripts/quest_prototype/src/screens/quest-start-bootstrap.ts](/Users/dthurn/dreamtides/scripts/quest_prototype/src/screens/quest-start-bootstrap.ts)
- [scripts/quest_prototype/src/draft/draft-engine.ts](/Users/dthurn/dreamtides/scripts/quest_prototype/src/draft/draft-engine.ts)
- [scripts/quest_prototype/src/draft/draft-engine.test.ts](/Users/dthurn/dreamtides/scripts/quest_prototype/src/draft/draft-engine.test.ts)
- [scripts/quest_prototype/src/logging.ts](/Users/dthurn/dreamtides/scripts/quest_prototype/src/logging.ts)

## Execution context

- Project directory: `/Users/dthurn/dreamtides`
- Run directory: `/tmp/enact/tides-draft-feel/`
- Working app: `/Users/dthurn/dreamtides/scripts/quest_prototype`
- Design intent source: `notes/tides_v2.md`
- Live authored content: `rules_engine/tabula/rendered-cards.toml` and `rules_engine/tabula/dreamcallers.toml`
- Runtime implementation under test: `scripts/quest_prototype/src/data/quest-content.ts`, `scripts/quest_prototype/src/draft/draft-engine.ts`, and quest start flow in `scripts/quest_prototype/src/screens/quest-start-bootstrap.ts`
- Standard setup commands:
  - `cd /Users/dthurn/dreamtides/scripts/quest_prototype`
  - `npm install`
  - `npm run setup-assets`
- Preferred bulk trace path: direct TypeScript module scripts using `node --experimental-strip-types` or `npx tsx`, not browser automation
- Preferred manual feel-check path: `agent-browser` against `http://localhost:5173` after `npm run dev`

This evaluation run should extend the standard Enact run directory with these
artifact folders:

- `/tmp/enact/tides-draft-feel/traces/broad/`
- `/tmp/enact/tides-draft-feel/traces/deep/`
- `/tmp/enact/tides-draft-feel/judgments/theme-fit/`
- `/tmp/enact/tides-draft-feel/judgments/choice-fun/`
- `/tmp/enact/tides-draft-feel/summaries/`

## Context

This run is an evaluation run, not a feature implementation run. The goal is to produce evidence about whether the current hidden-tides quest prototype delivers the intended dreamcaller drafting experience in practice.

The specific product question is:

- After picking a Dreamcaller, do draft offers consistently contain cards that feel relevant to that Dreamcaller?
- Across repeated runs with the same Dreamcaller, do the offers still vary enough that the draft does not feel scripted?
- Within a single offer, are players making meaningful tradeoffs between plausible options rather than taking one obvious on-theme card over filler?
- Does the overall drafting experience feel fun, or does it feel repetitive, incoherent, or low-agency?

The evaluation must separate generation from judgment. Agents that generate traces should not grade those same traces. Judges should work from trace artifacts, not by rerunning the generator ad hoc.

The evaluation should use two evidence channels:

1. Bulk structured traces generated from the actual module code paths that build Dreamcaller packages and draft offers.
2. Small-number browser feel checks in the live prototype UI to ensure the on-screen experience matches the trace-level conclusions.

The evaluation should produce these durable artifacts:

- A trace corpus with reproducible inputs and outputs
- Per-trace judgment sheets scoring dreamcaller fit, choice tension, and fun
- Per-dreamcaller comparison summaries across repeated runs
- A final synthesis that says whether the system is meeting the goal, where it fails, and what kind of fixes would likely help

## Scope assessment

Medium complexity.

Reasoning:

- The code surface is fairly narrow and already implemented.
- The difficult part is not coding; it is designing a credible evaluation harness and keeping judgment independent from generation.
- This does not require a surveyor pass over the whole repo. The project-level phase should use 2 focused researchers, then proceed to planning and task generation.
- Task-level work should rely on parallel worker subagents in two lanes: trace generation workers and independent judgment workers.

Recommended project-level subagents:

- 2 researchers
- 1 synthesizer
- 1 planner
- 1 QA scenario generator

Recommended task-level pattern:

- Parallel trace generators own disjoint scenario shards
- Parallel judges own disjoint trace shards they did not generate
- One synthesis worker aggregates per-dreamcaller and corpus-level conclusions

## Evaluation design

Use a staged matrix rather than a full exhaustive sweep with human-style judging everywhere.

### Phase 1: Broad coverage

Generate one baseline trace for every currently valid Dreamcaller in `rules_engine/tabula/dreamcallers.toml` using a single `cohesive` drafter persona.

For each trace:

- Resolve the Dreamcaller package through the real `resolveDreamcallerPackage()` path
- Initialize drafting through the real `initializeDraftState()` path
- Simulate the first 3 draft site visits, each up to `SITE_PICKS`
- Record every revealed offer, the chosen card, the unchosen competitors, the chosen card's tide overlap with the resolved package, and the remaining pool summary after each pick

Purpose:

- Catch obviously off-theme Dreamcallers
- Catch offers with no real tension
- Identify which Dreamcallers deserve deeper replayability analysis

### Phase 2: Deep comparison

Select 6 Dreamcallers for deeper study:

- 4 chosen to maximize archetype spread
- 1 broad-pass success case that already looks strong
- 1 broad-pass failure or borderline case that appears incoherent, repetitive, or low-agency

For each selected Dreamcaller, generate 4 additional traces:

- 2 seeds with a `cohesive` persona
- 2 seeds with a `flex` persona that still prefers package-adjacent cards but will take plausible sidegrades and pivots when the offer supports them

This produces 24 deep-comparison traces and lets judges compare same-Dreamcaller runs for both coherence and replay variety.

### Phase 3: Blind judgment

Split judging into two lanes:

- `theme-fit` judges see the Dreamcaller text and package summary
- `choice-fun` judges see the Dreamcaller text plus trace offers and picks, but should not rely on raw package tide IDs as the primary basis for scoring

Each judgment sheet must score:

- `dreamcaller_fit` on a 1-5 scale
- `meaningful_choice` on a 1-5 scale
- `fun` on a 1-5 scale
- `variety_within_run` on a 1-5 scale

Each judgment sheet must also include:

- 3 concrete moments from the trace that support the score
- a one-paragraph summary of what the run felt like to draft
- a verdict of `pass`, `borderline`, or `fail`

### Phase 4: Synthesis

The final synthesis should answer the user’s actual question directly:

- Are drafts thematically relevant to the chosen Dreamcaller?
- Are repeated drafts cohesive without collapsing into sameness?
- Are picks interesting enough to feel like real decisions?
- Is the overall experience fun?

It should also identify failure modes in concrete terms, such as:

- too many obviously dead off-theme cards
- same-package offers but one card is always clearly correct
- good thematic fit but too little cross-run variation
- variation created mainly by off-theme noise instead of meaningful subpackage tension

## Run name

`tides-draft-feel`

## QA strategy

Manual QA is about validating the evaluation process, not shipping product code.

After each task, QA should confirm that the produced artifact is both usable and grounded in the real prototype.

### Generator QA

For every trace-generation task:

- Verify the worker used the real package resolution and draft engine paths, not a hand-rolled mock
- Verify each trace records Dreamcaller id, seed or RNG stream, selected package tides, per-offer cards, chosen card, and stopping condition
- Verify the trace is reproducible: rerun one sample trace from the same shard and confirm the same offer sequence is produced
- Verify at least one trace from the shard is readable by a human without reopening source code

Successful generator QA means:

- trace files are complete
- traces are reproducible
- the artifact format is sufficient for an independent judge to score without rerunning the app

### Judge QA

For every judgment task:

- Verify the judge scored all required dimensions
- Verify the judge cited concrete offer-level evidence
- Verify the judge did not evaluate traces it generated
- Verify the writeup distinguishes thematic fit from fun and from choice tension

Successful judge QA means:

- judgment sheets are evidence-based
- the rubric is used consistently
- generation and judgment remain separated

### Browser feel-check QA

Run a small live-prototype spot check after the first broad-pass tranche and again before final synthesis.

Commands:

- `cd /Users/dthurn/dreamtides/scripts/quest_prototype`
- `npm run dev`
- `agent-browser open http://localhost:5173`
- `agent-browser wait --load networkidle`
- `agent-browser snapshot -i`

Use the browser only to validate that the live UI still presents the same kind of drafting situation the traces imply. Do not try to recreate exact random seeds in the browser unless the harness has already added a deterministic seed path without changing product behavior.

Successful browser QA means:

- the app starts on Dreamcaller selection
- draft sites still present 4 unique cards when possible
- the observed offers feel consistent with the trace-level conclusions
- `window.__errors` is empty

## Verification

Before declaring the evaluation complete:

1. In `scripts/quest_prototype`, run:
   - `npm install`
   - `npm run setup-assets`
   - `npm run typecheck`
   - `npm test`
   - `npm run build`
2. Confirm the run directory contains:
   - a broad-pass trace corpus covering every valid Dreamcaller
   - a deep-pass trace corpus covering 6 selected Dreamcallers across both personas
   - independent judgment files for every trace shard
   - a final synthesis document with pass/borderline/fail conclusions
3. Confirm the final synthesis includes both strengths and failure cases, not just averages
4. Confirm at least two browser feel-check sessions were completed and summarized

## Resolved assumptions

- This run should evaluate the current implementation first. It should not start by modifying `rendered-cards.toml`, `dreamcallers.toml`, or prototype logic unless the evaluation itself uncovers an obvious tooling blocker.
- Bulk trace generation should happen outside product code when possible. Prefer wrapper scripts in `/tmp/enact/tides-draft-feel/` that import the real TypeScript modules and stub `Math.random` from a fixed stream, rather than changing the app to add seeded RNG just for this evaluation.
- “Fun” is treated as a qualitative judgment backed by trace evidence and spot-check browser play, not as a numeric gameplay metric alone.
- The evaluation should optimize for credible evidence over exhaustive coverage. Broad pass covers all Dreamcallers lightly; deep pass studies a smaller set comparatively.
- If the broad pass shows a clearly broken Dreamcaller package, include it in deep pass even if it duplicates an archetype already represented.
- A trace should cover the first 3 draft site visits or stop earlier if the draft engine produces no valid offer.
- The orchestrator should keep generation and judgment workers disjoint by shard ownership.
- The final answer to the user should be an evaluation report, not a code patch, unless the user explicitly asks for fixes after reviewing the evidence.
