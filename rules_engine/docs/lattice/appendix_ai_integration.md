# Appendix: AI Integration

This appendix documents AI agent integration. See
[Lattice Design](lattice_design.md#skill-integration) for Lattice's general
approach to AI compatibility.

## Design Goals

AI agents benefit from using Lattice commands rather than direct file reads:

- **`lat show`** provides composed template content, dependencies, and related
  documents in a single call
- **`lat ready`** surfaces actionable work without manual filtering
- **`lat overview`** provides curated context for session orientation
- **View tracking** improves `lat overview` recommendations over time

Direct file reads miss template composition and don't contribute to view
tracking. The hooks below encourage or enforce command usage.

## Installation

```bash
lat setup claude          # Install hooks and configuration
lat setup claude --check  # Verify installation status
lat setup claude --remove # Uninstall
lat setup claude --project # Project-only (vs global)
```

## Hook Architecture

## Enforcing `lat show` via PreToolUse

The recommended approach blocks reads to Lattice documents and instructs
Claude to use `lat show` instead.

**Hook configuration** (`.claude/settings.json`):

Hooks should be configured as follows:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Read",
        "command": [".claude/hooks/lattice-read-guard.py"]
      }
    ]
  }
}
```

The `lattice-read-guard.py` script should be produced by Lattice to enforce this
behavior.