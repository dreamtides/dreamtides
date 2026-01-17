use tracing::debug;

/// Git conflict marker indicating "our" side of the conflict.
const CONFLICT_MARKER_OURS: &str = "<<<<<<<";
/// Git conflict marker indicating the separator between sides.
const CONFLICT_MARKER_SEPARATOR: &str = "=======";
/// Git conflict marker indicating "their" side of the conflict.
const CONFLICT_MARKER_THEIRS: &str = ">>>>>>>";

/// Type of conflict marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictMarker {
    /// The `<<<<<<<` marker indicating "our" side.
    Ours,
    /// The `=======` separator between sides.
    Separator,
    /// The `>>>>>>>` marker indicating "their" side.
    Theirs,
}

/// Location of a conflict marker in content.
#[derive(Debug, Clone)]
pub struct ConflictLocation {
    /// 1-indexed line number where the marker appears.
    pub line_number: usize,
    /// Type of conflict marker.
    pub marker: ConflictMarker,
}

/// Detects if content contains unresolved git merge conflict markers.
///
/// Returns `true` if all three conflict markers (`<<<<<<<`, `=======`,
/// `>>>>>>>`) are present in the content, indicating an unresolved merge
/// conflict.
///
/// This function checks for the presence of all three markers because partial
/// markers might legitimately appear in documentation or code examples.
pub fn has_conflict_markers(content: &str) -> bool {
    let has_ours = content.contains(CONFLICT_MARKER_OURS);
    let has_separator = content.contains(CONFLICT_MARKER_SEPARATOR);
    let has_theirs = content.contains(CONFLICT_MARKER_THEIRS);

    let has_conflict = has_ours && has_separator && has_theirs;

    if has_conflict {
        debug!(has_ours, has_separator, has_theirs, "Detected git conflict markers in content");
    }

    has_conflict
}

/// Returns details about conflict markers found in content.
///
/// Useful for generating user-facing error messages that indicate where
/// conflicts are located.
pub fn find_conflict_lines(content: &str) -> Vec<ConflictLocation> {
    let mut locations = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let line_number = line_num + 1; // 1-indexed for user display
        let trimmed = line.trim_start();

        if trimmed.starts_with(CONFLICT_MARKER_OURS) {
            locations.push(ConflictLocation { line_number, marker: ConflictMarker::Ours });
        } else if trimmed.starts_with(CONFLICT_MARKER_THEIRS) {
            locations.push(ConflictLocation { line_number, marker: ConflictMarker::Theirs });
        } else if trimmed == CONFLICT_MARKER_SEPARATOR {
            locations.push(ConflictLocation { line_number, marker: ConflictMarker::Separator });
        }
    }

    locations
}
