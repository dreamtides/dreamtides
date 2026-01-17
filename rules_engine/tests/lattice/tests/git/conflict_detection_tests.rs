use lattice::git::conflict_detection::{ConflictMarker, find_conflict_lines, has_conflict_markers};

#[test]
fn has_conflict_markers_returns_true_for_full_conflict() {
    let content = r#"
Some normal content here.
<<<<<<< HEAD
Our changes
=======
Their changes
>>>>>>> branch-name
More content after.
"#;

    assert!(
        has_conflict_markers(content),
        "Content with all three conflict markers should be detected"
    );
}

#[test]
fn has_conflict_markers_returns_false_for_clean_content() {
    let content = r#"
# Document Title

This is normal markdown content.
No conflicts here.
"#;

    assert!(!has_conflict_markers(content), "Clean content should not trigger conflict detection");
}

#[test]
fn has_conflict_markers_returns_false_for_partial_markers_ours_only() {
    let content = r#"
Example of conflict marker in documentation:
<<<<<<< HEAD shows our side
But without other markers.
"#;

    assert!(
        !has_conflict_markers(content),
        "Single ours marker should not be detected as conflict"
    );
}

#[test]
fn has_conflict_markers_returns_false_for_partial_markers_theirs_only() {
    let content = r#"
Example:
>>>>>>> shows their side reference
Just documentation.
"#;

    assert!(
        !has_conflict_markers(content),
        "Single theirs marker should not be detected as conflict"
    );
}

#[test]
fn has_conflict_markers_returns_false_for_separator_only() {
    let content = r#"
Table:
=======
Header
"#;

    assert!(!has_conflict_markers(content), "Separator alone should not be detected as conflict");
}

#[test]
fn has_conflict_markers_returns_false_for_two_markers_missing_separator() {
    let content = r#"
<<<<<<< HEAD
Our stuff
>>>>>>> branch
No separator between them.
"#;

    assert!(
        !has_conflict_markers(content),
        "Missing separator should not be detected as full conflict"
    );
}

#[test]
fn has_conflict_markers_returns_false_for_two_markers_missing_theirs() {
    let content = r#"
<<<<<<< HEAD
Our stuff
=======
Their stuff but no closing marker
"#;

    assert!(
        !has_conflict_markers(content),
        "Missing theirs marker should not be detected as full conflict"
    );
}

#[test]
fn has_conflict_markers_returns_true_for_multiple_conflicts() {
    let content = r#"
<<<<<<< HEAD
First conflict ours
=======
First conflict theirs
>>>>>>> branch
Some content
<<<<<<< HEAD
Second conflict ours
=======
Second conflict theirs
>>>>>>> branch
"#;

    assert!(has_conflict_markers(content), "Multiple conflicts should be detected");
}

#[test]
fn has_conflict_markers_handles_empty_content() {
    assert!(!has_conflict_markers(""), "Empty content should not have conflicts");
}

#[test]
fn find_conflict_lines_returns_all_marker_locations() {
    let content = r#"line 1
<<<<<<< HEAD
line 3 ours
=======
line 5 theirs
>>>>>>> branch
line 7
"#;

    let locations = find_conflict_lines(content);

    assert_eq!(locations.len(), 3, "Should find exactly 3 conflict markers");

    assert_eq!(locations[0].line_number, 2, "Ours marker should be on line 2");
    assert_eq!(locations[0].marker, ConflictMarker::Ours, "First marker should be Ours");

    assert_eq!(locations[1].line_number, 4, "Separator should be on line 4");
    assert_eq!(locations[1].marker, ConflictMarker::Separator, "Second marker should be Separator");

    assert_eq!(locations[2].line_number, 6, "Theirs marker should be on line 6");
    assert_eq!(locations[2].marker, ConflictMarker::Theirs, "Third marker should be Theirs");
}

#[test]
fn find_conflict_lines_returns_empty_for_clean_content() {
    let content = "Normal content\nwith no conflicts";

    let locations = find_conflict_lines(content);

    assert!(locations.is_empty(), "Clean content should have no conflict locations");
}

#[test]
fn find_conflict_lines_handles_indented_markers() {
    let content = r#"
  <<<<<<< HEAD
  indented ours
  =======
  indented theirs
  >>>>>>> branch
"#;

    let locations = find_conflict_lines(content);

    assert_eq!(locations.len(), 3, "Should find indented conflict markers");
}

#[test]
fn find_conflict_lines_handles_marker_with_extra_info() {
    let content = "<<<<<<< HEAD:file.txt\n=======\n>>>>>>> abc123:file.txt\n";

    let locations = find_conflict_lines(content);

    assert_eq!(locations.len(), 3, "Should find markers with file info appended");
}
