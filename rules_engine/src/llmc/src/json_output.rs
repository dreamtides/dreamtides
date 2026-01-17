use serde::Serialize;

const JSON_SCHEMA_DOC: &str = r#"{
  "llmc_json_schema": {
    "version": "1.0",
    "description": "JSON output schema for LLMC commands. Use --json flag with any command to get structured output.",
    "commands": {
      "status": {
        "description": "Show status of all workers",
        "output": {
          "workers": [{
            "name": "string - Worker identifier",
            "status": "string - Current state: idle|working|needs_review|reviewing|rejected|rebasing|error|offline",
            "branch": "string - Git branch name",
            "time_in_state_secs": "number - Seconds since last state change",
            "commit_sha": "string|null - Commit SHA if awaiting review",
            "prompt_cmd": "string|null - Command used to generate prompt",
            "prompt_excerpt": "string|null - First 50 chars of prompt"
          }]
        }
      },
      "start": {
        "description": "Start a worker on a task",
        "output": {
          "worker": "string - Worker name that was started",
          "status": "string - New status (working)",
          "self_review_enabled": "boolean - Whether self-review is enabled",
          "worktree_path": "string - Path to worker's worktree",
          "branch": "string - Git branch name"
        }
      },
      "add": {
        "description": "Add a new worker",
        "output": {
          "worker": "string - Worker name",
          "branch": "string - Git branch name",
          "worktree_path": "string - Path to worktree",
          "model": "string - Model being used"
        }
      },
      "nuke": {
        "description": "Remove worker(s)",
        "output": {
          "workers_removed": ["string - Names of removed workers"]
        }
      },
      "message": {
        "description": "Send message to worker",
        "output": {
          "worker": "string - Worker name",
          "message_sent": "boolean - Whether message was sent"
        }
      },
      "review": {
        "description": "Review a worker's completed work",
        "output": {
          "worker": "string - Worker name",
          "status": "string - Worker status",
          "commit_sha": "string|null - Commit SHA being reviewed",
          "changed_files": ["string - List of changed file paths"]
        }
      },
      "reject": {
        "description": "Reject a worker's work",
        "output": {
          "worker": "string - Worker name",
          "previous_status": "string - Status before rejection",
          "new_status": "string - Status after rejection (rejected)"
        }
      },
      "accept": {
        "description": "Accept a worker's work and merge",
        "output": {
          "worker": "string - Worker name",
          "commit_sha": "string - Final commit SHA",
          "commit_message": "string - Commit message",
          "status": "string - Final status (idle or rebasing)",
          "needs_conflict_resolution": "boolean - Whether conflicts need resolution"
        }
      },
      "rebase": {
        "description": "Rebase a worker's branch",
        "output": {
          "worker": "string - Worker name",
          "success": "boolean - Whether rebase succeeded",
          "conflicts": ["string - List of conflicting files if any"]
        }
      },
      "reset": {
        "description": "Reset a worker to clean state",
        "output": {
          "worker": "string - Worker name",
          "previous_status": "string - Status before reset",
          "new_status": "string - Status after reset (idle)"
        }
      },
      "doctor": {
        "description": "Check system health",
        "output": {
          "healthy": "boolean - Whether system is healthy",
          "issues": [{
            "category": "string - Issue category",
            "description": "string - Issue description",
            "severity": "string - warning|error"
          }],
          "repairs": ["string - Repairs performed (if --repair)"]
        }
      },
      "peek": {
        "description": "Show lines from worker session",
        "output": {
          "worker": "string - Worker name",
          "lines": ["string - Terminal output lines"]
        }
      },
      "pick": {
        "description": "Grab changes from worker",
        "output": {
          "worker": "string - Worker name",
          "success": "boolean - Whether pick succeeded",
          "commit_sha": "string|null - Commit SHA if successful"
        }
      },
      "down": {
        "description": "Stop the LLMC daemon",
        "output": {
          "workers_stopped": ["string - Names of stopped workers"]
        }
      }
    },
    "common_types": {
      "WorkerStatus": "One of: idle, working, needs_review, reviewing, rejected, rebasing, error, offline"
    }
  }
}"#;

#[derive(Debug, Clone, Serialize)]
pub struct StartOutput {
    pub worker: String,
    pub status: String,
    pub self_review_enabled: bool,
    pub worktree_path: String,
    pub branch: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AddOutput {
    pub worker: String,
    pub branch: String,
    pub worktree_path: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NukeOutput {
    pub workers_removed: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageOutput {
    pub worker: String,
    pub message_sent: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReviewOutput {
    pub worker: String,
    pub status: String,
    pub commit_sha: Option<String>,
    pub changed_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RejectOutput {
    pub worker: String,
    pub previous_status: String,
    pub new_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AcceptOutput {
    pub worker: String,
    pub commit_sha: String,
    pub commit_message: String,
    pub status: String,
    pub needs_conflict_resolution: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RebaseOutput {
    pub worker: String,
    pub success: bool,
    pub conflicts: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResetOutput {
    pub worker: String,
    pub previous_status: String,
    pub new_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DoctorOutput {
    pub healthy: bool,
    pub issues: Vec<DoctorIssue>,
    pub repairs: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DoctorIssue {
    pub category: String,
    pub description: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PeekOutput {
    pub worker: String,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PickOutput {
    pub worker: String,
    pub success: bool,
    pub commit_sha: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DownOutput {
    pub workers_stopped: Vec<String>,
}

pub fn print_json<T: Serialize>(output: &T) {
    println!(
        "{}",
        serde_json::to_string_pretty(output)
            .unwrap_or_else(|e| { format!("{{\"error\": \"JSON serialization failed: {e}\"}}") })
    );
}

pub fn print_json_schema() {
    println!("{}", JSON_SCHEMA_DOC);
}
