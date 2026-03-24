---
name: hs-batch-orchestrator
description: Orchestrate 100 sequential hearthstone-adapt subagent runs across 1000 Hearthstone cards. Each subagent designs one card from a pool of 10 HS cards, writes it to rendered-cards.toml, then the next subagent starts.
---

# Hearthstone Batch Adaptation Orchestrator

You are an orchestrator that runs 100 sequential subagents, each adapting a pool of 10
Hearthstone cards into one Dreamtides card design. Sequential execution is critical — each
subagent must finish and write its card to `rendered-cards.toml` before the next starts, so
later subagents see all prior designs and avoid duplication.

**All subagents MUST be launched with `model: "opus"`.**

## Setup

1. Read the current highest `card-number` in `rules_engine/tabula/rendered-cards.toml`:
   ```bash
   grep 'card-number' rules_engine/tabula/rendered-cards.toml | sort -t= -k2 -n | tail -1
   ```
   Store this as `NEXT_CARD_NUMBER` (highest + 1). Increment by 1 for each subagent.

2. Read the hearthstone-adapt skill file so you can inline its full contents into each
   subagent prompt:
   ```
   .llms/skills/hearthstone-adapt/SKILL.md
   ```
   Store the complete skill text (everything after the frontmatter `---` block) as
   `SKILL_BODY`. You will paste this into every subagent prompt.

3. The input file is `/tmp/hearthstone/hearthstone.md` with 1000 cards. The file has a
   4-line header, then each card occupies exactly 5 lines starting at line 5. Pool boundaries:
   - Pool 1 (cards 1-10): lines 5-54
   - Pool 2 (cards 11-20): lines 55-104
   - Pool N (cards (N-1)*10+1 to N*10): lines (N-1)*50+5 to N*50+4
   - Pool 100 (cards 991-1000): lines 4955-5004

## Execution Loop

For each pool N from 1 to 100:

**Launch ONE foreground Agent** with `model: "opus"` and the prompt below. Wait for it to
complete before launching the next. Do NOT run agents in parallel.

After each agent completes, print a one-line status: `✓ Pool N/100 complete — [card name]`

If an agent fails or produces no valid design, note the failure and continue to the next pool.
Do not retry.

### Subagent Prompt Template

For pool N, compute:
- `START_LINE = (N-1)*50 + 5`
- `END_LINE = N*50 + 4`
- `CARD_NUM = NEXT_CARD_NUMBER + (N-1)`
- `POOL_START = (N-1)*10 + 1`
- `POOL_END = N*10`

Use this exact prompt (with values substituted). `{SKILL_BODY}` is the full text of the
hearthstone-adapt skill you read in Setup step 2 — paste it in verbatim:

````
You are a card designer. Your job is to design ONE outstanding Dreamtides card from a pool
of 10 Hearthstone cards, then write it to rendered-cards.toml. Run with ultrathink.

## Step 1: Read Your Card Pool

Read lines {START_LINE} to {END_LINE} of `/tmp/hearthstone/hearthstone.md` using the Read
tool (use the `offset` and `limit` parameters). These are Hearthstone cards
{POOL_START}-{POOL_END} of 1000.

Also read `/tmp/hearthstone/glossary.md` for Hearthstone keyword definitions.

## Step 2: Design a Card

Follow the complete card design skill instructions below. Your 10 input cards are the ones
you just read in Step 1.

IMPORTANT CONTEXT: This is pool {N} of 100 sequential runs. Cards from pools 1-{N-1} have
already been added to rendered-cards.toml. When you load the card pool in Phase 1, you will
see those recent additions. You MUST avoid duplicating any existing design — check carefully
against ALL cards in the pool, especially recent additions near the bottom of the file.

--- BEGIN SKILL INSTRUCTIONS ---

{SKILL_BODY}

--- END SKILL INSTRUCTIONS ---

## Step 3: Write the Winning Design to rendered-cards.toml

After producing a final design, append it to `rules_engine/tabula/rendered-cards.toml`.

Insert the new [[cards]] entry BEFORE the [metadata] section (find the line `[metadata]` and
insert above it).

Generate a UUID for the id field by running:
```bash
python3 -c "import uuid; print(uuid.uuid4())"
```

Use this exact TOML format:

```toml
[[cards]]
name = "<card name>"
id = "<UUID from python>"
tide = "<tide>"
tide-cost = <integer>
rendered-text = "<rules text, max 100 chars>"
energy-cost = <integer>
card-type = "<Character or Event>"
subtype = "<subtype or empty string for events>"
rarity = "<Common, Uncommon, Rare, or Legendary>"
is-fast = <true or false>
spark = <integer for characters, or "" for events>
image-number = 0
art-owned = false
card-number = {CARD_NUM}
```

Key formatting rules:
- `spark` for events must be `""` (empty string), for characters use a bare integer
- `is-fast` is a bare boolean, not quoted
- `tide-cost` and `energy-cost` are bare integers, not quoted
- `image-number = 0` and `art-owned = false` for all new cards
- Card name must be 25 characters or fewer
- rendered-text must be 100 characters or fewer

## Step 4: Verify

After writing, grep for your card name in rendered-cards.toml to confirm it was written
correctly.

Print ONLY the final card name and its key stats (tide, cost, spark, rules text) as your
final output. Keep output minimal to preserve context for the orchestrator.
````

## Context Preservation

This orchestrator will run for a long time across 100 sequential agents. To preserve context:

1. **Keep your own output minimal.** After each agent, print only the one-line status.
2. **Do not accumulate agent results** in your context. The subagents write to the file;
   you don't need to remember their designs.
3. **Track only:** current pool number, NEXT_CARD_NUMBER, and any failures.
4. **Every 10 pools,** print a brief checkpoint: `--- Checkpoint: {N}/100 pools complete, {failures} failures ---`

## Error Handling

- If a subagent fails to produce a design: log `✗ Pool N/100 failed — [reason]`, continue.
- If the TOML write fails (e.g., duplicate name): the next subagent will see the state and
  can work around it.
- Do not retry failed pools. The goal is coverage, not perfection.

## Completion

After all 100 pools are processed, print a final summary:
```
=== Batch Complete ===
Pools processed: 100
Cards added: [count]
Failures: [count]
Failed pools: [list if any]
```

Then run `just fmt` and `just review` per the acceptance checklist.
