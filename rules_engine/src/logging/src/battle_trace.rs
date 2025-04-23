/// Macro for adding a tracing event to a battle.
///
/// If tracing is enabled, this will record an event for the current battle
/// turn as well as emitting an event via the 'tracing' crate.
///
/// Arguments:
/// - `$message`: A message describing the event.
/// - `$battle`: The battle data, must be a mutable reference.
/// - The remaining arguments are symbols to include in the trace and can take
///   either of two forms:
///
///   - A list of variable names
///   - A list of expressions with the form `name = expr`
///
///   (Note that these forms cannot be combined)
///
/// Example:
/// ```rust
/// // With simple variables:
/// battle_trace!("Drawing cards", battle, player, count);
///
/// // With expressions:
/// battle_trace!("Resolving card", battle, card_id = card_id, controller = source.controller());
/// ```
#[macro_export]
macro_rules! battle_trace {
    ($message:expr, $battle:expr) => {{
        if $battle.tracing.is_some() {
            $battle.add_tracing_event(battle_data::battle::battle_tracing::BattleTraceEvent {
                m: $message.to_string(),
                vs: String::new(),
                values: std::collections::BTreeMap::new(),
                snapshot: $battle.debug_snapshot(),
            });
        }
    }};
    ($message:expr, $battle:expr, $($key:ident),* $(,)?) => {{
        if $battle.tracing.is_some() {
            let mut values = std::collections::BTreeMap::new();
            let mut values_string = String::new();
            $(
                values.insert(stringify!($key).to_string(), format!("{:?}", $key));
                values_string.push_str(&format!("{}: {:?}, ", stringify!($key), $key));
            )*

            $battle.add_tracing_event(battle_data::battle::battle_tracing::BattleTraceEvent {
                m: $message.to_string(),
                vs: values_string,
                values,
                snapshot: $battle.debug_snapshot(),
            });
        }
    }};
    ($message:expr, $battle:expr, $($key:ident = $value:expr),* $(,)?) => {{
        if $battle.tracing.is_some() {
            let mut values = std::collections::BTreeMap::new();
            let mut values_string = String::new();
            $(
                values.insert(stringify!($key).to_string(), format!("{:?}", $value));
                values_string.push_str(&format!("{}: {:?}, ", stringify!($key), $value));
            )*

            $battle.add_tracing_event(battle_data::battle::battle_tracing::BattleTraceEvent {
                m: $message.to_string(),
                vs: values_string,
                values,
                snapshot: $battle.debug_snapshot(),
            });
        }
    }};
    ($message:expr, $battle:expr, $($simple_key:ident),+ $(,)? $($complex_key:ident = $complex_value:expr),+ $(,)?) => {{
        if $battle.tracing.is_some() {
            let mut values = std::collections::BTreeMap::new();
            let mut values_string = String::new();
            $(
                values.insert(stringify!($simple_key).to_string(), format!("{:?}", $simple_key));
                values_string.push_str(&format!("{}: {:?}, ", stringify!($simple_key), $simple_key));
            )*
            $(
                values.insert(stringify!($complex_key).to_string(), format!("{:?}", $complex_value));
                values_string.push_str(&format!("{}: {:?}, ", stringify!($complex_key), $complex_value));
            )*

            $battle.add_tracing_event(battle_data::battle::battle_tracing::BattleTraceEvent {
                message: $message.to_string(),
                values,
                values_string,
                snapshot: $battle.debug_snapshot(),
            });
        }
    }};
}

#[cfg(test)]
mod tests {
    use battle_data::battle::battle_data::BattleData;
    use battle_data::battle::battle_status::BattleStatus;
    use battle_data::battle::battle_tracing::BattleTracing;
    use battle_data::battle::battle_turn_step::BattleTurnStep;
    use battle_data::battle::request_context::RequestContext;
    use battle_data::battle::turn_data::TurnData;
    use battle_data::battle_cards::all_cards::AllCards;
    use battle_data::battle_player::player_data::{PlayerData, PlayerType};
    use core_data::identifiers::{BattleId, UserId};
    use core_data::numerics::{Energy, Points, Spark, TurnId};
    use core_data::types::PlayerName;
    use rand_xoshiro::rand_core::SeedableRng;
    use rand_xoshiro::Xoshiro256PlusPlus;
    use uuid::Uuid;

    fn create_test_battle() -> BattleData {
        BattleData {
            id: BattleId(Uuid::new_v4()),
            request_context: RequestContext::UserRequest,
            user: PlayerData {
                name: PlayerName::User,
                player_type: PlayerType::User(UserId::default()),
                points: Points(0),
                current_energy: Energy(2),
                produced_energy: Energy(2),
                spark_bonus: Spark(0),
            },
            enemy: PlayerData {
                name: PlayerName::Enemy,
                player_type: PlayerType::User(UserId::default()),
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
            prompt_resume_action: None,
            tracing: Some(BattleTracing::default()),
        }
    }

    #[test]
    fn test_battle_trace_with_no_values() {
        let mut battle = create_test_battle();
        battle_trace!("Something happened", battle);

        let events = &battle.tracing.unwrap().current;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].m, "Something happened");
        assert!(events[0].values.is_empty());
        assert_eq!(events[0].vs, "");
    }

    #[test]
    fn test_battle_trace_with_values() {
        let mut battle = create_test_battle();
        let player = PlayerName::User;
        let count = 2;

        battle_trace!("Drawing cards", battle, player, count);

        let events = &battle.tracing.unwrap().current;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].m, "Drawing cards");
        assert_eq!(events[0].values.get("player").unwrap(), "User");
        assert_eq!(events[0].values.get("count").unwrap(), "2");
        assert!(events[0].vs.contains("player: User"));
        assert!(events[0].vs.contains("count: 2"));
    }

    #[test]
    fn test_battle_trace_multiple_events() {
        let mut battle = create_test_battle();

        battle_trace!("First event", battle);

        let damage = 5;
        let target = "enemy character";
        battle_trace!("Damage dealt", battle, damage, target);

        let events = &battle.tracing.unwrap().current;
        assert_eq!(events.len(), 2);

        assert_eq!(events[0].m, "First event");
        assert!(events[0].values.is_empty());
        assert_eq!(events[0].vs, "");

        assert_eq!(events[1].m, "Damage dealt");
        assert_eq!(events[1].values.get("damage").unwrap(), "5");
        assert_eq!(events[1].values.get("target").unwrap(), "\"enemy character\"");
        assert!(events[1].vs.contains("damage: 5"));
        assert!(events[1].vs.contains("target: \"enemy character\""));
    }

    #[test]
    fn test_values_string_format() {
        let mut battle = create_test_battle();
        let number = 42;
        let text = "sample text";

        battle_trace!("Format test", battle, number, text);

        let events = &battle.tracing.unwrap().current;
        assert_eq!(events.len(), 1);

        let expected_format = "number: 42, text: \"sample text\", ";
        assert_eq!(events[0].vs, expected_format);
    }

    #[test]
    fn test_battle_trace_with_expressions() {
        let mut battle = create_test_battle();
        let player = PlayerName::User;
        let count = 2;

        // Simple usage
        battle_trace!("Simple trace", battle, player, count);

        // Using expressions
        battle_trace!(
            "With expressions",
            battle,
            player_name = format!("{:?}", player),
            doubled_count = count * 2
        );

        let events = &battle.tracing.unwrap().current;
        assert_eq!(events.len(), 2);

        assert_eq!(events[0].m, "Simple trace");
        assert_eq!(events[0].values.get("player").unwrap(), "User");
        assert_eq!(events[0].values.get("count").unwrap(), "2");

        assert_eq!(events[1].m, "With expressions");
        assert_eq!(events[1].values.get("player_name").unwrap(), "\"User\"");
        assert_eq!(events[1].values.get("doubled_count").unwrap(), "4");
    }
}
