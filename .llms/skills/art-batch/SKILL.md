---
name: art-batch
description: Orchestrate batch art-to-card matching across ~442 candidate images for 90 unassigned cards. Runs all images against the card pool, writing results to art-assigned.toml.
---

# Art Batch Matching Orchestrator

You are an orchestrator that matches art to cards using a **rolling parallelism** model.
You maintain a target number of concurrent subagents (default 25), launching replacements
immediately as each agent completes — never waiting for an entire batch to finish.

The candidate images are pre-filtered to `/tmp/art-narrow/candidates/` (~442 images).
The card pool is the 90 unassigned cards in `cards_anonymized.txt`.

**All subagents MUST be launched with `model: "sonnet"`. Or, in Codex, use a supported GPT model such as `gpt-5.4-mini` or `gpt-5.4`; `reasoning_effort: "medium"` is acceptable for routine matching.**

## Setup

1. Build the image list:
   ```bash
   python3 .llms/skills/art-batch/build-image-list.py
   ```

2. Create the results directory and clear stale inflight state:
   ```bash
   mkdir -p /tmp/art-batch-results
   rm -f /tmp/art-batch-inflight.txt
   ```

## Rolling Launch

### Initial Fill

Get the first N image IDs (where N is the parallelism target, default 25):
```bash
python3 .llms/skills/art-batch/next-batch.py 25
```
This prints up to 25 image IDs (one per line), or `DONE` if none remain.

Launch **one background Agent per image ID** in a single message (all in parallel).
Use `model: "sonnet"` and `run_in_background: true` for each. Or, in Codex, use
`spawn_agent` with a supported GPT model. Give each agent the Image Agent Prompt
below with its IMAGE_ID filled in.

### Continuous Refill

After the initial fill, **wait for notifications**. Do NOT poll, sleep-loop, or check
on agents. You will be automatically notified when each agent completes.

When notified that one or more agents have completed:

1. Print one short line per completed agent (name, tide, cost — or SKIP reason).
2. Immediately request replacement image IDs for however many agents just finished:
   ```bash
   python3 .llms/skills/art-batch/next-batch.py <COUNT>
   ```
   `next-batch.py` tracks in-flight IDs automatically — it will never return an ID
   that is already launched. It outputs one of:
   - **Image IDs** (one per line): launch a new background agent for each.
   - **`WAITING`**: all remaining images are in-flight; no new agents to launch. Just
     wait for the next notification.
   - **`DONE`**: every image is either done or skipped. Let remaining in-flight agents
     drain.
3. If this returns image IDs, launch new background agents for each in a single message.
4. Continue waiting for the next notification.

### Drain & Finish

Once `next-batch.py` returns `DONE`, stop launching new agents. Continue receiving
notifications for the remaining in-flight agents, printing one line each, until all
have completed. Then proceed to the Join step.

### Image Agent Prompt

```
You are matching one piece of art to a card from the Dreamtides card pool. Run with
ultrathink.

Your image ID is {IMAGE_ID}.

Read the art-match skill at `.llms/skills/art-match/SKILL.md` and follow it completely. Or,
in Codex, read enough of that skill to execute the workflow accurately, and pass the actual
local image as a `local_image` input item when available.

## Overused Name Words

During Phase 4 (naming), check your proposed name before finalizing:
```bash
python3 .llms/skills/art-batch/overused-words.py "Proposed Card Name"
```
If it prints FAIL, one or more words are overused (3+ prior uses) — pick a different name
and check again. Only finalize a name that prints PASS.

## Final Step: Write Result

After selecting a winner and assigning a name/subtype (Phase 4 of the skill), find the
real card matching your selected anonymized rules text:

```bash
grep -n 'rendered-text' rules_engine/tabula/rendered-cards.toml | grep '<EXACT RULES TEXT>'
```

Use the line number to read the surrounding [[cards]] block and extract the real card's
id, tide, tide-cost, energy-cost, card-type, rarity, is-fast, and spark.

Then write this TOML block to `/tmp/art-batch-results/{IMAGE_ID}.toml`.

**The file MUST contain exactly these fields and no others. Do NOT add extra fields
like `narrative`. The `rendered-text` field is REQUIRED — it must contain the exact
rules text from rendered-cards.toml.**

```toml
[[cards]]
name = "<YOUR new card name from Phase 4>"
id = "<real card UUID>"
tide = "<tide from real card>"
tide-cost = <from real card>
rendered-text = "<rules text from real card>"
energy-cost = <from real card>
card-type = "<from real card>"
subtype = "<YOUR subtype from Phase 4>"
rarity = "<from real card>"
is-fast = <from real card>
spark = <from real card, or "" for events>
image-number = {IMAGE_ID}
art-owned = false
card-number = 0
```

After writing, validate the file:
```bash
python3 .llms/skills/art-batch/validate-result.py /tmp/art-batch-results/{IMAGE_ID}.toml
```
If it prints FAIL, fix the file before finishing.

If the art is landscape or abstract, do NOT write a result file. Instead, record the
skip by appending the image ID to `/tmp/art-batch-skips.txt`:
```bash
echo "{IMAGE_ID}" >> /tmp/art-batch-skips.txt
```
Then print "SKIP: [reason]" and stop.

Print ONLY the card name, tide, and cost as your final output.
```

## Join Results

After all agents have drained:

```bash
python3 .llms/skills/art-batch/join-results.py
```

This appends all per-image result files from `/tmp/art-batch-results/` into
`rules_engine/tabula/art-assigned.toml`.

Print final stats:

```
=== Batch Complete ===
Total results joined: [count]
Skips: [count]
```

## Gravity Well Protection

Two layers prevent any single rules text from being overrepresented:

1. **Soft gate (check-match-count.py):** After selecting a winner, the subagent runs
   `check-match-count.py` with the exact rules text. WARN at 2 matches forces
   reconsideration; FAIL at 3+ forces picking a different card.

2. **Hard cap (pool-filter.py in art-match):** Cards matched 3+ times are excluded from
   pool output entirely. The pool also annotates cards with ⚠2× or higher prefixes so
   the agent can see popularity at browse time.

Both scripts use `match_counts.py` to count assignments across **all** sources
(art-assigned.toml + /tmp/art-batch-results/*.toml) with proper TOML parsing.

## Context Preservation — CRITICAL

You will process ~442 images. Your context window may overflow if you are not disciplined.

1. **Never print more than one line per completed agent.** No commentary, no analysis.
2. **Never print summaries, progress updates, or status messages between refills.**
3. **Do not accumulate results.** Subagents write to individual files.
4. **Do not restate or reflect on previous completions.**
5. **Do not poll or sleep-loop for agent completion.** Wait for notifications.
6. **Immediately launch replacements** after each completion notification. No pauses.
7. **Batch your refill launches.** If multiple agents complete in quick succession before
   you respond, request all replacement IDs at once and launch them in a single message.
