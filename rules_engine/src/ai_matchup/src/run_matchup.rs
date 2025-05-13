use std::io::{self, Write};
use std::time::Instant;

use ai_agents::agent_search;
use battle_mutations::actions::apply_battle_action;
use battle_queries::legal_action_queries::legal_actions;
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle_player::battle_player_state::PlayerType;
use clap::{Parser, ValueEnum};
use core_data::identifiers::BattleId;
use core_data::types::PlayerName;
use game_creation::new_test_battle;
use serde_json::from_str;
use tracing::{debug, subscriber};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Layer};
use uuid::Uuid;

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
enum Verbosity {
    None,
    OneLine,
    Actions,
    Verbose,
}

#[derive(Parser)]
#[command(
    version,
    about = "Run a matchup between two AI agents",
    after_help = "EXAMPLE:\n    run_matchup '{\"uct1MaxIterations\": 1000}' '{\"uct1MaxIterations\": 5000}'"
)]
struct Args {
    #[arg(help = "JSON serialized GameAI for Player One")]
    player_one_ai: String,

    #[arg(help = "JSON serialized GameAI for Player Two")]
    player_two_ai: String,

    #[arg(long, default_value = "3141592653", help = "Random seed for the battle")]
    seed: u64,

    #[arg(long, short, value_enum, default_value = "one-line", help = "Verbosity level")]
    verbosity: Verbosity,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let player_one_ai = from_str(&args.player_one_ai)?;
    let player_two_ai = from_str(&args.player_two_ai)?;

    // Set up the appropriate verbosity filter
    let filter_string = match args.verbosity {
        Verbosity::None => "warn",
        Verbosity::OneLine => "warn",
        Verbosity::Actions => "warn",
        Verbosity::Verbose => "debug",
    };

    let filter = EnvFilter::new(filter_string);
    let subscriber =
        tracing_subscriber::registry().with(tracing_subscriber::fmt::layer().with_filter(filter));

    match args.verbosity {
        Verbosity::None => {
            // Only print the final result
            println!("Running matchup...");
        }
        Verbosity::OneLine | Verbosity::Actions | Verbosity::Verbose => {
            println!("Running matchup between {} and {}", args.player_one_ai, args.player_two_ai);
        }
    }

    let battle_id = BattleId(Uuid::new_v4());
    let mut battle = new_test_battle::create_and_start(
        battle_id,
        args.seed,
        PlayerType::Agent(player_one_ai),
        PlayerType::Agent(player_two_ai),
    );

    let start_time = Instant::now();

    // Run the matchup with the configured subscriber
    subscriber::with_default(subscriber, || {
        while !matches!(battle.status, BattleStatus::GameOver { .. }) {
            let turn = battle.turn.turn_id;
            if let Some(player) = legal_actions::next_to_act(&battle) {
                let player_ai = match player {
                    PlayerName::One => player_one_ai,
                    PlayerName::Two => player_two_ai,
                };

                let player_ai_json = match player {
                    PlayerName::One => &args.player_one_ai,
                    PlayerName::Two => &args.player_two_ai,
                };

                let legal_actions = legal_actions::compute(&battle, player);
                let action = if legal_actions.len() == 1 {
                    legal_actions.all()[0]
                } else {
                    agent_search::select_action_unchecked(&battle, player, &player_ai)
                };

                match args.verbosity {
                    Verbosity::None => {}
                    Verbosity::OneLine => {
                        // Clear the entire line and move cursor to beginning
                        print!("\r\x1B[2K");
                        print!("AI {} takes action: {:?} in turn {}", player_ai_json, action, turn);
                        io::stdout().flush().unwrap();
                    }
                    Verbosity::Actions | Verbosity::Verbose => {
                        println!(
                            "AI {} takes action: {:?} in turn {}",
                            player_ai_json, action, turn
                        );
                    }
                }

                debug!("Player {:?} executing action: {:?}", player, action);
                apply_battle_action::execute(&mut battle, player, action);
                debug!("Action completed");
            } else {
                panic!("No player to act, but game not over.");
            }
        }
    });

    // Print a newline to ensure the final result isn't on the same line as the last
    // action
    if args.verbosity == Verbosity::OneLine {
        println!();
    }

    let elapsed = start_time.elapsed();

    match battle.status {
        BattleStatus::GameOver { winner } => {
            let winner_ai = match winner {
                PlayerName::One => &args.player_one_ai,
                PlayerName::Two => &args.player_two_ai,
            };

            println!("\nGame over after {} turns in {:.2?}!", battle.turn.turn_id, elapsed);
            println!(
                "Winner: AI {} wins {:?} to {:?}",
                winner_ai,
                battle.players.player(winner).points,
                battle.players.player(winner.opponent()).points
            );
        }
        _ => panic!("Game ended without a winner"),
    }

    Ok(())
}
