use battle_data::debug_snapshots::debug_battle_data::DebugBattleData;

/// Unwraps an Option value, returning the Some contained within it, or panics.
///
/// This is a macro version of the `expect` method which records more debug
/// information on panic.
///
/// Arguments:
/// - `$option`: The Option value to unwrap.
/// - `$battle`: The battle data.
/// - `$message`: A closure that returns a string message to display on panic.
///
/// Example:
/// ```rust
/// let option = Some(42);
/// let value = expect!(option, battle, || "test message");
/// assert_eq!(value, 42);
/// ```
#[macro_export]
macro_rules! expect {
    ($option:expr, $battle:expr, $message:expr) => {
        match $option {
            Some(v) => v,
            None => {
                let snapshot = $battle.debug_snapshot();
                let message = $crate::panic_message(snapshot, $message());
                panic!("{}", message)
            }
        }
    };
}

/// Asserts that a condition is true, or panics.
///
/// This is a macro version of the `assert` method which records more debug
/// information on panic.
///
/// Arguments:
/// - `$condition`: The condition to assert.
/// - `$battle`: The battle data.
/// - `$message`: A closure that returns a string message to display on panic.
///
/// Example:
/// ```rust
/// assert_with!(2 == 2, battle, || "should not fail");
/// ```
#[macro_export]
macro_rules! assert_with {
    ($condition:expr, $battle:expr, $message:expr) => {
        if !$condition {
            let snapshot = $battle.debug_snapshot();
            let message = $crate::panic_message(snapshot, $message());
            panic!("{}", message)
        }
    };
}

pub fn panic_message(snapshot: DebugBattleData, message: impl AsRef<str>) -> String {
    format!("{}, battle: {:?}", message.as_ref(), snapshot.id)
}

#[cfg(test)]
mod tests {
    use battle_data::battle::battle_data::BattleData;
    use battle_data::battle::battle_status::BattleStatus;
    use battle_data::battle::battle_turn_step::BattleTurnStep;
    use battle_data::battle::request_context::RequestContext;
    use battle_data::battle::turn_data::TurnData;
    use battle_data::battle_cards::all_cards::AllCards;
    use battle_data::battle_player::player_data::PlayerData;
    use core_data::identifiers::BattleId;
    use core_data::numerics::{Energy, Points, Spark, TurnId};
    use core_data::types::PlayerName;
    use rand_xoshiro::rand_core::SeedableRng;
    use rand_xoshiro::Xoshiro256PlusPlus;
    use uuid::Uuid;

    #[test]
    fn test_expect_some() {
        let option = Some(42);
        let battle = get_test_battle();

        let value = crate::expect!(option, battle, || "test message");
        assert_eq!(value, 42);
    }

    #[test]
    #[should_panic(expected = "test message, battle:")]
    fn test_expect_none() {
        let option: Option<i32> = None;
        let battle = get_test_battle();

        let _value = crate::expect!(option, battle, || "test message");
    }

    #[test]
    fn test_formatting() {
        let option = Some(42);
        let battle = get_test_battle();
        let value = crate::expect!(option, battle, || format!("test message {}", 42));
        assert_eq!(value, 42);
    }

    #[test]
    fn test_assert_with_true() {
        let battle = get_test_battle();
        crate::assert_with!(2 == 2, battle, || "should not fail");
    }

    #[test]
    #[should_panic(expected = "assertion failed, battle:")]
    fn test_assert_with_false() {
        let battle = get_test_battle();
        crate::assert_with!(2 == 3, battle, || "assertion failed");
    }

    fn get_test_battle() -> BattleData {
        BattleData {
            id: BattleId(Uuid::new_v4()),
            request_context: RequestContext::UserRequest,
            user: PlayerData {
                name: PlayerName::User,
                ai: None,
                points: Points(0),
                current_energy: Energy(2),
                produced_energy: Energy(2),
                spark_bonus: Spark(0),
            },
            enemy: PlayerData {
                name: PlayerName::Enemy,
                ai: None,
                points: Points(0),
                current_energy: Energy(2),
                produced_energy: Energy(2),
                spark_bonus: Spark(0),
            },
            cards: AllCards::default(),
            status: BattleStatus::Playing,
            turn: TurnData { active_player: PlayerName::User, turn_id: TurnId(1) },
            step: BattleTurnStep::Main,
            rng: Xoshiro256PlusPlus::seed_from_u64(12345),
            animations: None,
            prompt: None,
        }
    }
}
