use std::time::Duration;

use llmc::overseer_mode::overseer_loop::FailureSpiralTracker;

#[test]
fn first_failure_should_not_trigger_failure_spiral() {
    // Even if the daemon fails immediately (within cooldown), the first failure
    // should NOT be detected as a failure spiral. We need to attempt remediation
    // at least once before we can say "remediation isn't working".
    let cooldown = Duration::from_secs(60);
    let tracker = FailureSpiralTracker::new(cooldown);

    // Daemon ran for 0 seconds (immediate failure)
    let daemon_runtime = Duration::from_secs(0);

    assert!(
        !tracker.should_detect_spiral(daemon_runtime),
        "First failure should not be detected as failure spiral, \
         even if daemon failed immediately. Must attempt remediation first."
    );
}

#[test]
fn second_failure_within_cooldown_triggers_spiral_after_remediation() {
    // After remediation has been attempted and the daemon fails again within
    // cooldown, THEN we should detect a failure spiral.
    let cooldown = Duration::from_secs(60);
    let mut tracker = FailureSpiralTracker::new(cooldown);

    // Mark that we attempted remediation
    tracker.record_remediation_attempt();

    // Daemon failed within cooldown
    let daemon_runtime = Duration::from_secs(10);

    assert!(
        tracker.should_detect_spiral(daemon_runtime),
        "Second failure within cooldown after remediation should be a spiral"
    );
}

#[test]
fn second_failure_beyond_cooldown_does_not_trigger_spiral() {
    // Even after remediation, if the daemon runs longer than cooldown before
    // failing, that's not a failure spiral - it had a healthy period.
    let cooldown = Duration::from_secs(60);
    let mut tracker = FailureSpiralTracker::new(cooldown);

    tracker.record_remediation_attempt();

    // Daemon ran for 120 seconds (beyond cooldown)
    let daemon_runtime = Duration::from_secs(120);

    assert!(
        !tracker.should_detect_spiral(daemon_runtime),
        "Failure after healthy runtime (beyond cooldown) is not a spiral"
    );
}

#[test]
fn successful_long_run_resets_spiral_tracking() {
    // If the daemon runs successfully past the cooldown, subsequent quick
    // failures should reset the spiral tracking - we should try remediation
    // again before declaring a spiral.
    let cooldown = Duration::from_secs(60);
    let mut tracker = FailureSpiralTracker::new(cooldown);

    // First remediation
    tracker.record_remediation_attempt();

    // Daemon runs beyond cooldown (successful)
    let long_runtime = Duration::from_secs(120);
    tracker.record_daemon_stopped(long_runtime);

    // Now daemon fails quickly again
    let quick_runtime = Duration::from_secs(5);

    assert!(
        !tracker.should_detect_spiral(quick_runtime),
        "After a healthy run past cooldown, tracker should reset. \
         Next quick failure needs remediation attempt before being a spiral."
    );
}

#[test]
fn stall_skips_remediation_but_does_not_affect_spiral_tracking() {
    // Stalls skip remediation (per design), but this should NOT count as
    // "remediation attempted" for spiral detection purposes.
    let cooldown = Duration::from_secs(60);
    let mut tracker = FailureSpiralTracker::new(cooldown);

    // Record that a stall occurred (no remediation)
    tracker.record_stall_handled();

    // Next quick failure
    let daemon_runtime = Duration::from_secs(5);

    assert!(
        !tracker.should_detect_spiral(daemon_runtime),
        "Stall handling does not count as remediation attempt"
    );
}

#[test]
fn multiple_remediations_still_detect_spiral_on_quick_failure() {
    // After multiple remediation attempts, quick failures still trigger spiral
    let cooldown = Duration::from_secs(60);
    let mut tracker = FailureSpiralTracker::new(cooldown);

    tracker.record_remediation_attempt();
    tracker.record_remediation_attempt();
    tracker.record_remediation_attempt();

    let daemon_runtime = Duration::from_secs(10);

    assert!(
        tracker.should_detect_spiral(daemon_runtime),
        "Multiple remediations followed by quick failure is still a spiral"
    );
}
