use std::time::Duration;

use llmc::tmux::sender::{LARGE_MESSAGE_THRESHOLD, TmuxSender};

#[test]
fn test_default_sender() {
    let sender = TmuxSender::new();
    assert_eq!(sender.debounce_base_ms, 500);
    assert_eq!(sender.debounce_per_kb_ms, 100);
    assert_eq!(sender.max_debounce_ms, 2000);
    assert_eq!(sender.enter_retry_count, 3);
    assert_eq!(sender.enter_retry_delay_ms, 200);
}

#[test]
fn test_calculate_delay() {
    let sender = TmuxSender::new();
    assert_eq!(sender.calculate_delay(0), Duration::from_millis(500));
    assert_eq!(sender.calculate_delay(512), Duration::from_millis(500));
    assert_eq!(sender.calculate_delay(1024), Duration::from_millis(600));
    assert_eq!(sender.calculate_delay(2048), Duration::from_millis(700));
    assert_eq!(sender.calculate_delay(10240), Duration::from_millis(1500));
    assert_eq!(sender.calculate_delay(102400), Duration::from_millis(2000));
}

#[test]
fn test_large_message_threshold() {
    let _sender = TmuxSender::new();
    assert!(999 < LARGE_MESSAGE_THRESHOLD);
    assert!(1024 >= LARGE_MESSAGE_THRESHOLD);
}
