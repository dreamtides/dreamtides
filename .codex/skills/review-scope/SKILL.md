---
name: review-scope
description: Explain how Dreamtides scoped review chooses changed files, maps impact domains, and decides which `just review` steps run or are skipped. Use when asked about REVIEW_SCOPE behavior, defaults, env overrides, or why a run was forced full.
---

# Review Scope

Use this skill to explain the review scope system implemented in `rules_engine/scripts/review_scope.py` and consumed by `rules_engine/scripts/review_perf_runner.py`.

## What It Does

`just review` (perf-runner path) builds a scope decision before running steps:

1. Resolve changed files.
2. Classify impact into `core`, `parser`, `tv`, or `full`.
3. Select steps to run/skip.
4. Emit a `scope_plan` telemetry event and console summary.

## Changed File Resolution

Priority order:

1. `REVIEW_SCOPE_CHANGED_FILES` explicit file list.
2. `REVIEW_SCOPE_BASE_REF` + `REVIEW_SCOPE_HEAD_REF` git diff.
3. CI mode (`CI` truthy): merge-base against `origin/master`.
4. Local mode:
   - Default `REVIEW_SCOPE_LOCAL_STRATEGY=head-if-dirty`: staged + unstaged + untracked changes only.
   - Optional `REVIEW_SCOPE_LOCAL_STRATEGY=merge-base-union`: branch diff since merge-base plus staged/unstaged/untracked.

If local mode is clean under default strategy, scope forces full review.

## Scope Modes

- `REVIEW_SCOPE_MODE=enforce` (default): apply skips when safe.
- `REVIEW_SCOPE_MODE=dry-run`: compute and log scope, but execute full step list.
- `REVIEW_SCOPE_MODE=off`: bypass planner and run full step list.

`REVIEW_SCOPE_FORCE_FULL` always forces full review.

## Forced Full Conditions

Scope fails closed and runs full review when:

- A global full trigger path changes (see `review_scope_config.json`).
- Any changed path is unmapped by domain rules and crate mapping.
- Planner encounters an error.
- No changed files are found in local default mode.

## Domain Mapping

- Parser domain: parser crates and parser path prefixes.
- TV domain: tv crates and tv path prefixes.
- Core domain: everything else that is mapped and not forced full.
- Full domain: any force-full condition.

Crate impact is metadata-driven (`cargo metadata`) and expanded through reverse dependencies.

## Step Selection

Configured sets in `review_scope_config.json`:

- `always_run_steps`
- `parser_steps`
- `tv_steps`

In `enforce` mode with no force-full:

- always-run steps execute
- parser steps execute only if parser domain impacted
- tv steps execute only if tv domain impacted

## Useful Commands

```bash
just review-scope-plan
just review-scope-validate
```

Use `just review-scope-plan` to inspect decision output and source of changed files.
