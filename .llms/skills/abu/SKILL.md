---
name: abu
description: Use when working with ABU (Agent-Browser for Unity), controlling Unity Editor, taking snapshots, clicking UI elements, or modifying the scene walker. Triggers on abu, unity automation, snapshot, scene walker, abu.py.
---

# ABU (Agent-Browser for Unity)

Read the documentation before making changes:

- **Main reference**: [docs/abu/abu.md](../../../docs/abu/abu.md) — architecture, wire protocol, snapshot format, CLI usage, C# components, conventions, and common pitfalls.
- **Development guide**: [docs/abu/abu_development.md](../../../docs/abu/abu_development.md) — how to modify the scene walker, add new UI features, and interactively test changes.

# Acceptance Criteria

- Please *manually validate* all work on ABU against the running Unity editor. It is
  NOT sufficient to rely only on unit tests to demonstrate correctness.
- Please use the `writing-docs` skill and update ABU documentation after making changes.
  Document your new feature and also aggressively fix incorrect information you find.