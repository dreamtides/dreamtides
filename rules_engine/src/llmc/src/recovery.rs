use crate::state::WorkerRecord;

/// Reset crash count if appropriate (after 24 hours)
pub fn should_reset_crash_count(worker: &WorkerRecord, now_unix: u64) -> bool {
    if let Some(last_crash) = worker.last_crash_unix {
        let time_since_crash = now_unix.saturating_sub(last_crash);
        time_since_crash >= 24 * 60 * 60
    } else {
        false
    }
}

/// Reset API error count if appropriate (after 24 hours)
pub fn should_reset_api_error_count(worker: &WorkerRecord, now_unix: u64) -> bool {
    if let Some(last_api_error) = worker.last_api_error_unix {
        let time_since_error = now_unix.saturating_sub(last_api_error);
        time_since_error >= 24 * 60 * 60
    } else {
        false
    }
}
