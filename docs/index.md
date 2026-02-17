# Dreamtides Documentation Index

- [project_overview.md](project_overview/project_overview.md): Technical
  architecture overview covering the full project structure, crate layers, card
  data pipeline, battle execution, display/animation system, client
  architecture, AI, testing, and build tooling. Read when onboarding or needing
  a broad understanding of any system.

- [parser_pipeline.md](parser_pipeline/parser_pipeline.md): End-to-end guide to
  the card ability parser pipeline — how TOML rules-text is lexed, variable-
  resolved, parsed into ability ASTs, and serialized back to rich display text.
  Includes a step-by-step checklist for adding new keywords/effects, the five
  effect sub-parser modules, predicate targeting, RLF phrase integration, tabula
  generate lifecycle, and common pitfalls. Read when adding or modifying card
  abilities, keywords, effects, or parser grammar.
