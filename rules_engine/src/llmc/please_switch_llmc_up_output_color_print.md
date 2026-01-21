---
lattice-id: LBUWQN
name: please-switch-llmc-up-output-color-print
description: Please switch llmc up output color printed lines using AYU theme eg when task is assigned when it is completed etc
task-type: feature
priority: 1
created-at: 2026-01-21T13:58:01.019598Z
updated-at: 2026-01-21T13:58:46.531251Z
---

Please switch `llmc up` output to color printed lines using the AYU theme, e.g. when a task is assigned, when it is completed, etc:

/// Green for success states (Ayu green)
pub const AYU_SUCCESS: Rgb = Rgb(149, 230, 203);

/// Yellow/amber for warning states (Ayu orange)
pub const AYU_WARNING: Rgb = Rgb(255, 180, 84);

/// Red/coral for error states (Ayu red)
pub const AYU_ERROR: Rgb = Rgb(240, 113, 120);

/// Blue for accent/highlighting (Ayu blue)
pub const AYU_ACCENT: Rgb = Rgb(89, 194, 255);

/// Gray for muted/secondary text (Ayu comment)
pub const AYU_MUTED: Rgb = Rgb(99, 106, 114);

/// Brighter gray for de-emphasized but readable text
pub const AYU_DIM: Rgb = Rgb(140, 145, 152);

/// Purple for special highlighting (Ayu purple)
pub const AYU_SPECIAL: Rgb = Rgb(217, 149, 255);

