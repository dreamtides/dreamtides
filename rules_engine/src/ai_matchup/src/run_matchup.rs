use std::io::{self, Write};
use std::time::{Duration, Instant};

use ai_agents::agent_search;
use battle_mutations::actions::apply_battle_action;
use battle_queries::legal_action_queries::legal_actions;
use battle_state::battle::battle_state::{LoggingOptions, RequestContext};
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

    #[arg(
        long,
        default_value = "1",
        help = "Number of matches to run, alternating player position"
    )]
    matches: usize,
}

struct MatchResult {
    player_one_wins: usize,
    player_two_wins: usize,
    ai_one_wins: usize,
    ai_two_wins: usize,
    total_turns: usize,
    total_elapsed: Duration,
    timed_out: usize,
    ai_one_timing: AgentTimingStats,
    ai_two_timing: AgentTimingStats,
}

#[derive(Clone, Copy, Debug, Default)]
struct AgentTimingStats {
    total: Duration,
    max: Duration,
    count: usize,
}

impl AgentTimingStats {
    fn record(&mut self, elapsed: Duration) {
        self.total += elapsed;
        self.count += 1;
        if elapsed > self.max {
            self.max = elapsed;
        }
    }

    fn avg(&self) -> Option<Duration> {
        if self.count > 0 { Some(self.total / self.count as u32) } else { None }
    }
}

struct MatchActionStats {
    ai_one: AgentTimingStats,
    ai_two: AgentTimingStats,
}

#[derive(Debug)]
enum MatchOutcome {
    Winner(PlayerName, usize, std::time::Duration),
    Draw(usize, std::time::Duration),
}

fn run_match(
    ai_one: &str,
    ai_two: &str,
    seed: u64,
    verbosity: Verbosity,
    swap_positions: bool,
) -> (MatchOutcome, MatchActionStats) {
    let ai_one_parsed = from_str(ai_one).unwrap();
    let ai_two_parsed = from_str(ai_two).unwrap();

    let filter_string = match verbosity {
        Verbosity::None => "warn",
        Verbosity::OneLine => "warn",
        Verbosity::Actions => "warn",
        Verbosity::Verbose => "debug",
    };

    let filter = EnvFilter::new(filter_string);
    let subscriber =
        tracing_subscriber::registry().with(tracing_subscriber::fmt::layer().with_filter(filter));

    let (battle_ai_one, battle_ai_two) = if swap_positions {
        (PlayerType::Agent(ai_two_parsed), PlayerType::Agent(ai_one_parsed))
    } else {
        (PlayerType::Agent(ai_one_parsed), PlayerType::Agent(ai_two_parsed))
    };

    match verbosity {
        Verbosity::None => {}
        Verbosity::OneLine | Verbosity::Actions | Verbosity::Verbose => {
            if swap_positions {
                println!(
                    "Running matchup between {ai_one} (P2) and {ai_two} (P1) with seed {seed}"
                );
            } else {
                println!(
                    "Running matchup between {ai_one} (P1) and {ai_two} (P2) with seed {seed}"
                );
            }
        }
    }

    let battle_id = BattleId(Uuid::new_v4());
    let mut battle = new_test_battle::create_and_start(
        battle_id,
        seed,
        battle_ai_one,
        battle_ai_two,
        RequestContext { logging_options: LoggingOptions::default() },
    );

    let start_time = Instant::now();
    let mut turn_count = 0;
    let mut ai_one_stats = AgentTimingStats::default();
    let mut ai_two_stats = AgentTimingStats::default();

    subscriber::with_default(subscriber, || {
        while !matches!(battle.status, BattleStatus::GameOver { .. }) {
            let turn = battle.turn.turn_id;
            turn_count = turn.0 as usize;

            if let Some(player) = legal_actions::next_to_act(&battle) {
                let player_ai = match (player, swap_positions) {
                    (PlayerName::One, false) | (PlayerName::Two, true) => ai_one_parsed,
                    (PlayerName::Two, false) | (PlayerName::One, true) => ai_two_parsed,
                };

                let player_ai_json = match (player, swap_positions) {
                    (PlayerName::One, false) | (PlayerName::Two, true) => ai_one,
                    (PlayerName::Two, false) | (PlayerName::One, true) => ai_two,
                };

                let legal_actions = legal_actions::compute(&battle, player);
                let action_start = Instant::now();
                let action = if legal_actions.len() == 1 {
                    legal_actions.all()[0]
                } else {
                    agent_search::select_action_unchecked(&battle, player, &player_ai)
                };
                let action_time = action_start.elapsed();
                match (player, swap_positions) {
                    (PlayerName::One, false) | (PlayerName::Two, true) => {
                        ai_one_stats.record(action_time);
                    }
                    (PlayerName::Two, false) | (PlayerName::One, true) => {
                        ai_two_stats.record(action_time);
                    }
                }
                match verbosity {
                    Verbosity::None => {}
                    Verbosity::OneLine => {
                        print!("\r\x1B[2K");
                        print!("AI {player_ai_json} takes action: {action:?} in turn {turn}");
                        io::stdout().flush().unwrap();
                    }
                    Verbosity::Actions | Verbosity::Verbose => {
                        println!("AI {player_ai_json} takes action: {action:?} in turn {turn}");
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

    if verbosity == Verbosity::OneLine {
        println!();
    }

    let elapsed = start_time.elapsed();
    let stats = MatchActionStats { ai_one: ai_one_stats, ai_two: ai_two_stats };
    match battle.status {
        BattleStatus::GameOver { winner: None } => (MatchOutcome::Draw(turn_count, elapsed), stats),
        BattleStatus::GameOver { winner: Some(winner) } => {
            (MatchOutcome::Winner(winner, turn_count, elapsed), stats)
        }
        _ => panic!("Game ended without a winner"),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.matches == 0 {
        return Err("Number of matches must be greater than 0".into());
    }

    let mut results = MatchResult {
        player_one_wins: 0,
        player_two_wins: 0,
        ai_one_wins: 0,
        ai_two_wins: 0,
        total_turns: 0,
        total_elapsed: Duration::default(),
        timed_out: 0,
        ai_one_timing: AgentTimingStats::default(),
        ai_two_timing: AgentTimingStats::default(),
    };

    if args.matches > 1 {
        println!(
            "Running {} matches between {} and {}",
            args.matches, args.player_one_ai, args.player_two_ai
        );
    }

    for match_index in 0..args.matches {
        let swap_positions = match_index % 2 == 1;

        let match_verbosity =
            if args.verbosity == Verbosity::Verbose { Verbosity::Actions } else { args.verbosity };

        if args.matches > 1 {
            print!("Match {}/{}: ", match_index + 1, args.matches);
            io::stdout().flush().unwrap();
        }

        let (outcome, stats) = run_match(
            &args.player_one_ai,
            &args.player_two_ai,
            args.seed,
            match_verbosity,
            swap_positions,
        );

        results.ai_one_timing.total += stats.ai_one.total;
        results.ai_one_timing.count += stats.ai_one.count;
        if stats.ai_one.max > results.ai_one_timing.max {
            results.ai_one_timing.max = stats.ai_one.max;
        }
        results.ai_two_timing.total += stats.ai_two.total;
        results.ai_two_timing.count += stats.ai_two.count;
        if stats.ai_two.max > results.ai_two_timing.max {
            results.ai_two_timing.max = stats.ai_two.max;
        }
        match outcome {
            MatchOutcome::Winner(winner, turns, elapsed) => {
                match (winner, swap_positions) {
                    (PlayerName::One, false) => {
                        results.player_one_wins += 1;
                        results.ai_one_wins += 1;
                    }
                    (PlayerName::Two, false) => {
                        results.player_two_wins += 1;
                        results.ai_two_wins += 1;
                    }
                    (PlayerName::One, true) => {
                        results.player_one_wins += 1;
                        results.ai_two_wins += 1;
                    }
                    (PlayerName::Two, true) => {
                        results.player_two_wins += 1;
                        results.ai_one_wins += 1;
                    }
                }

                results.total_turns += turns;
                results.total_elapsed += elapsed;
                let winner_ai = if (winner == PlayerName::One && !swap_positions)
                    || (winner == PlayerName::Two && swap_positions)
                {
                    &args.player_one_ai
                } else {
                    &args.player_two_ai
                };

                if args.matches > 1 {
                    println!("Winner: {winner_ai}, Turns: {turns}, Time: {elapsed:.2?}");
                } else {
                    println!("\nGame over after {turns} turns in {elapsed:.2?}!");
                    println!("Winner: AI {winner_ai}");
                }
                print_match_summary(
                    &results,
                    args.matches,
                    &args.player_one_ai,
                    &args.player_two_ai,
                );
            }
            MatchOutcome::Draw(turns, elapsed) => {
                results.timed_out += 1;
                results.total_turns += turns;
                results.total_elapsed += elapsed;

                if args.matches > 1 {
                    println!("Draw after {turns} turns, Time: {elapsed:.2?}");
                } else {
                    println!("\nMatch ended in a draw after {turns} turns in {elapsed:.2?}!");
                }
                print_match_summary(
                    &results,
                    args.matches,
                    &args.player_one_ai,
                    &args.player_two_ai,
                );
            }
        }
    }

    if args.matches > 1 {
        println!("\n===== Match Results =====");
        println!("Total matches: {}", args.matches);
        println!(
            "Completed matches: {} ({:.1}%)",
            args.matches - results.timed_out,
            ((args.matches - results.timed_out) as f64 / args.matches as f64) * 100.0
        );
        println!(
            "Timed-out matches: {} ({:.1}%)",
            results.timed_out,
            (results.timed_out as f64 / args.matches as f64) * 100.0
        );

        if args.matches > results.timed_out {
            println!(
                "Average turns per completed match: {:.1}",
                results.total_turns as f64 / (args.matches - results.timed_out) as f64
            );
            println!("Average time per match: {:.2?}", results.total_elapsed / args.matches as u32);

            println!("By player position:");
            println!(
                "  Player One wins: {} ({:.1}%)",
                results.player_one_wins,
                (results.player_one_wins as f64 / (args.matches - results.timed_out) as f64)
                    * 100.0
            );
            println!(
                "  Player Two wins: {} ({:.1}%)",
                results.player_two_wins,
                (results.player_two_wins as f64 / (args.matches - results.timed_out) as f64)
                    * 100.0
            );

            println!("By AI:");
            println!(
                "  {} wins: {} ({:.1}%)",
                args.player_one_ai,
                results.ai_one_wins,
                (results.ai_one_wins as f64 / (args.matches - results.timed_out) as f64) * 100.0
            );
            println!(
                "  {} wins: {} ({:.1}%)",
                args.player_two_ai,
                results.ai_two_wins,
                (results.ai_two_wins as f64 / (args.matches - results.timed_out) as f64) * 100.0
            );

            if results.ai_one_timing.count > 0 {
                println!(
                    "  {} avg action time: {:.2?}, max: {:.2?}",
                    args.player_one_ai,
                    results.ai_one_timing.avg().unwrap(),
                    results.ai_one_timing.max
                );
            }
            if results.ai_two_timing.count > 0 {
                println!(
                    "  {} avg action time: {:.2?}, max: {:.2?}",
                    args.player_two_ai,
                    results.ai_two_timing.avg().unwrap(),
                    results.ai_two_timing.max
                );
            }
        }
    }

    Ok(())
}

fn print_match_summary(
    results: &MatchResult,
    total_matches: usize,
    player_one_ai: &str,
    player_two_ai: &str,
) {
    let completed = total_matches - results.timed_out;
    if completed > 0 {
        println!(
            "Progress: P1:{} P2:{} {}:{} {}:{}",
            results.player_one_wins,
            results.player_two_wins,
            player_one_ai,
            results.ai_one_wins,
            player_two_ai,
            results.ai_two_wins,
        );
    }
}
