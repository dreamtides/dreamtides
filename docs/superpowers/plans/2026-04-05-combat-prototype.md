# Dreamtides Combat Prototype Implementation Plan

**Status:** COMPLETE (2026-04-05)

**Goal:** Add positional front/back rank combat to Dreamtides, replacing
spark-comparison scoring with per-column Judgment resolution.

**Architecture:** Battlefield uses `[Option<CharacterId>; 8]` arrays for
front/back ranks. Judgment phase at end of turn resolves each column
independently. Turn order: Dreamwell → Draw → Dawn → Main → Judgment.

**Spec:** `docs/superpowers/specs/2026-04-05-combat-prototype-design.md`

## Completed Tasks (29 total)

01. QA Baseline
02. Add Vanilla Characters (6 cards, costs 1-7)
03. Battlefield Data Model (Battlefield struct with front/back arrays)
04. Remove Spark Bonus, Character Limit → 16
05. Phase Renames (Judgment→Dawn, new Judgment at end of turn)
06. ObjectPosition (rank + position in OnBattlefield)
07. UI Rank Rendering (4 ranks, judgment line, 8 slots each)
08. Debug Tools (AddCardToFrontRank, SkipToJudgment, etc.)
09. QA — Character Placement
10. Judgment Phase Resolution (column combat, uncontested scoring)
11. Character Materialization to Back Rank
12. Repositioning Actions (Front/Back buttons, swap logic)
13. QA — Summoning Sickness
14. AI Simplified Actions (forced front-rank, MoveToBack with limit)
15. Drag and Drop UI
16. QA — Full Game Loop
17. Kindle Effect (highest-spark target)
18. QA — Card Effects with Ranks
19. QA — UX Polish Round 1
20. QA — Extended Play Round 1
21. QA — Board Full States
22. QA — Judgment Edge Cases
23. QA — AI Behavior Round 2
24. QA — UX Polish Round 2
25. QA — Extended Play Round 4
26. QA — Regression
27. QA — Stress Testing
28. Update Battle Rules Documentation
29. QA — Final Regression Pass
