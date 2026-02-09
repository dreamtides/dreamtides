# Codex Skills Mirror

This directory provides Codex equivalents of project-local Claude skills with minimal duplication.

## Strategy

- Symlink directly to `.claude/skills/*` when frontmatter is already Codex-compatible.
- For `feature`, `loop`, and `tv`, keep a local `SKILL.md` wrapper copy that strips Claude-only frontmatter keys:
  - `disable-model-invocation`
  - `user-invocable`
- Reuse supporting docs via symlinks (for example, `tv/ARCHITECTURE.md` and `tv/COMMANDS.md`).
- Codex-native task management skills live directly in this tree:
  - `breaking-down-tasks-codex`
  - `task-ops-codex`

## Global install option (no duplication)

If you want these project skills available in Codex globally, link them into `~/.codex/skills`:

```bash
ln -s /Users/dthurn/Documents/GoogleDrive/dreamtides/.codex/skills/<skill-name> ~/.codex/skills/<skill-name>
```

Restart Codex after adding global links so new skills are picked up.
