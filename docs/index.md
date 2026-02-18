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

- [toml_card_format.md](toml_card_format/toml_card_format.md): Complete field
  reference for TOML card definitions — regular card and dreamwell card schemas,
  rules-text directive syntax (all pattern types with descriptions), variables
  field format and value types, phrase table entries, modal card conventions,
  multi-paragraph abilities, the prompts field, file locations, generated
  artifacts, staleness checking, and the tv editor app. Read when authoring or
  modifying card data in TOML files, adding new directive patterns, or
  understanding the card data pipeline from TOML to runtime.

- [rlf_localization.md](rlf_localization/rlf_localization.md): Complete guide to
  the RLF localization system — the rlf! macro syntax (constant and
  parameterized phrases, plural matching, variant blocks, :from inheritance,
  transforms), the Phrase type API and composition patterns, how serializers
  connect to RLF to produce display text, parser-side RLF syntax resolution,
  locale overrides, and the rlf_fmt/rlf_lint tools. Read when adding or
  modifying UI strings, working with card text display, or understanding how
  rules text is formatted for rendering.
