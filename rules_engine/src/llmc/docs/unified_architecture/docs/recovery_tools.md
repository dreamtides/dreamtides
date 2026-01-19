---
lattice-id: LB3WQN
parent-id: LBUWQN
name: recovery-tools
description: |-
  Doctor enhancements, salvage command, rescue command, and automatic
  self-healing in patrol.
created-at: 2026-01-19T05:00:00.000000Z
updated-at: 2026-01-19T05:00:00.000000Z
---

# Recovery Tools

## `llmc doctor` Enhancements

```rust
pub fn run_doctor(repair: bool, rebuild: bool, yes: bool) -> Result<()> {
    let mut issues: Vec<Issue> = Vec::new();

    // Check 1: Metadata directory exists
    check_metadata_dir(&mut issues)?;

    // Check 2: Config file valid
    check_config(&mut issues)?;

    // Check 3: State file valid
    check_state(&mut issues)?;

    // Check 4: Source repo accessible and clean
    check_source_repo(&mut issues)?;

    // Check 5: All worktrees exist and are valid
    check_worktrees(&mut issues)?;

    // Check 6: All branches exist
    check_branches(&mut issues)?;

    // Check 7: TMUX sessions match state
    check_tmux_sessions(&mut issues)?;

    // Check 8: No orphaned resources
    check_orphaned_resources(&mut issues)?;

    // Check 9: State consistency
    check_state_consistency(&mut issues)?;

    // Report findings
    report_issues(&issues);

    if repair && !issues.is_empty() {
        if !yes {
            confirm_repair(&issues)?;
        }
        repair_issues(&issues)?;
    }

    if rebuild {
        rebuild_state_from_filesystem()?;
    }

    Ok(())
}
```

## `llmc salvage` - New Command

Recovers work from a broken worker state:

```rust
/// Salvages commits from a worker into a patch file or new branch
pub fn run_salvage(worker: &str, output: SalvageOutput) -> Result<()> {
    let config = Config::load(&config::get_config_path())?;
    let source_repo = config.source_repo();
    let worktree_path = config.worktree_dir().join(worker);

    if !worktree_path.exists() {
        bail!("Worker worktree not found: {}", worktree_path.display());
    }

    // Find commits ahead of master
    let commits = git::list_commits_ahead_of(&worktree_path, "master")?;

    if commits.is_empty() {
        println!("No commits to salvage");
        return Ok(());
    }

    println!("Found {} commit(s) to salvage:", commits.len());
    for commit in &commits {
        println!("  {} {}", &commit.sha[..8], commit.subject);
    }

    match output {
        SalvageOutput::Patch(path) => {
            // Create patch file
            let patch = git::format_patch(&worktree_path, "master")?;
            fs::write(&path, patch)?;
            println!("Saved patch to {}", path.display());
        }
        SalvageOutput::Branch(name) => {
            // Create branch in main repo pointing to current HEAD
            let head = git::get_head_commit(&worktree_path)?;
            git::create_branch_at(&source_repo, &name, &head)?;
            println!("Created branch '{}' at {}", name, &head[..8]);
        }
        SalvageOutput::Stdout => {
            // Print patch to stdout
            let patch = git::format_patch(&worktree_path, "master")?;
            println!("{}", patch);
        }
    }

    Ok(())
}
```

## `llmc rescue` - New Command

Rescues a completely broken LLMC installation:

```rust
/// Complete rescue operation - salvages all work and rebuilds
pub fn run_rescue(yes: bool) -> Result<()> {
    println!("LLMC Rescue Mode");
    println!("----------------");
    println!();
    println!("This will:");
    println!("  1. Salvage all uncommitted work from workers");
    println!("  2. Save patches to ~/llmc-rescue/");
    println!("  3. Completely reset LLMC state");
    println!("  4. Preserve your config.toml");
    println!();

    if !yes {
        confirm("Proceed with rescue?")?;
    }

    let rescue_dir = PathBuf::from(std::env::var("HOME")?).join("llmc-rescue");
    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let rescue_subdir = rescue_dir.join(timestamp.to_string());
    fs::create_dir_all(&rescue_subdir)?;

    // Salvage each worker
    // ... implementation details ...

    println!("\nRescue complete!");
    println!("  Patches saved to: {}", rescue_subdir.display());

    Ok(())
}
```

## Automatic Self-Healing in Patrol

```rust
impl Patrol {
    fn self_heal(&self, state: &mut State, config: &Config) -> Vec<String> {
        let mut healed = Vec::new();

        for (name, worker) in state.workers.iter_mut() {
            let worktree = PathBuf::from(&worker.worktree_path);

            // Heal 1: Worktree missing but worker not offline
            if !worktree.exists() && worker.status != WorkerStatus::Offline {
                tracing::warn!("Healing: Worker {} has no worktree", name);
                worker.status = WorkerStatus::Offline;
                healed.push(format!("{}: marked offline (no worktree)", name));
            }

            // Heal 2: Worker idle with stale commits
            if worker.status == WorkerStatus::Idle && worktree.exists() {
                if let Ok(true) = git::has_commits_ahead_of(&worktree, "master") {
                    tracing::warn!("Healing: Idle worker {} has stale commits", name);
                    if git::reset_to_ref(&worktree, "master").is_ok() {
                        healed.push(format!("{}: reset stale commits", name));
                    }
                }
            }

            // Heal 3: Session exists but worker offline
            if worker.status == WorkerStatus::Offline {
                if session::session_exists(&worker.session_id) {
                    tracing::info!("Worker {} has session but marked offline", name);
                    // Don't change state - let hook handle it
                }
            }
        }

        healed
    }
}
```
