---
lattice-id: LBVWQN
name: please-investigate-following-lattice-iss
description: Please investigate following Lattice issue See rulesenginedocslatticelatticedesignmd appendices context code is ruleseng
parent-id: LCEWQN
task-type: feature
priority: 1
created-at: 2026-01-21T14:18:31.333118Z
updated-at: 2026-01-21T22:31:38.498268Z
closed-at: 2026-01-21T16:41:33.230854Z
---

Please investigate the following Lattice issue. See
@rules_engine/docs/lattice/lattice_design.md and appendices for context, code is
in @rules_engine/src/lattice/. When complete, please run `just fmt` and `just
review` to validate your changes, then create a git commit with a short
description of your work. Please update lattice_design.md if needed to match
these changes.

Please design and implement the --commit flag to `lat create`. This should cause
a git commit to be created with the new lattice file after creation completes.
The commit message should contain the full text of the lattice document, with
the first line formatting like "Create feature request L34567" (or bug report
etc). This must ALSO work with `lat create --interactive`, in which case it
should create the commit with the full text *after* $EDTOR returns.
