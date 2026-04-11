use std::cell::RefCell;
use std::io::{self, Write};
use std::panic::{self, AssertUnwindSafe};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{fs, process};

use ai_agents::agent_search;
use backtrace::Backtrace;
use battle_mutations::actions::apply_battle_action;
use battle_queries::legal_action_queries::legal_actions;
use battle_state::battle::battle_rules_config::BalanceMode;
use battle_state::battle::battle_state::{LoggingOptions, RequestContext};
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle_cards::dreamwell_data::Dreamwell;
use battle_state::battle_player::battle_player_state::{
    CreateBattlePlayer, PlayerType, TestDeckName,
};
use clap::{Parser, ValueEnum};
use core_data::identifiers::BattleId;
use core_data::types::PlayerName;
use game_creation::new_test_battle;
use serde_json::from_str;
use tabula_data::tabula::{Tabula, TabulaSource};
use tabula_generated::card_lists::DreamwellCardIdList;
use tracing::{debug, subscriber};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Layer};
use uuid::Uuid;

thread_local! {
    static PANIC_INFO: RefCell<Option<(String, String, String)>> = const { RefCell::new(None) };
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
enum Verbosity {
    None,
    OneLine,
    Actions,
    Verbose,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
enum DeckChoice {
    Vanilla,
    StartingFive,
    Benchmark1,
    Core11,
}

impl DeckChoice {
    fn to_test_deck_name(self) -> TestDeckName {
        match self {
            DeckChoice::Vanilla => TestDeckName::Vanilla,
            DeckChoice::StartingFive => TestDeckName::StartingFive,
            DeckChoice::Benchmark1 => TestDeckName::Benchmark1,
            DeckChoice::Core11 => TestDeckName::Core11,
        }
    }

    fn tabula_source(self) -> TabulaSource {
        match self {
            DeckChoice::Core11 => TabulaSource::Production,
            _ => TabulaSource::Test,
        }
    }

    fn dreamwell_list(self) -> DreamwellCardIdList {
        match self {
            DeckChoice::Core11 => DreamwellCardIdList::DreamwellBasic5,
            _ => DreamwellCardIdList::TestDreamwellNoAbilities,
        }
    }
}

fn parse_balance_mode(s: &str) -> BalanceMode {
    match s {
        "none" => BalanceMode::None,
        "extra-card" => BalanceMode::ExtraCard,
        "bonus-energy" => BalanceMode::BonusEnergy,
        "bonus-energy-no-draw" => BalanceMode::BonusEnergyNoDraw,
        "bonus-points" => BalanceMode::BonusPoints,
        "no-sickness" => BalanceMode::NoSickness,
        "coin" => BalanceMode::Coin,
        _ => panic!(
            "Unknown balance mode: {s}. Expected: none, extra-card, bonus-energy, bonus-energy-no-draw, bonus-points, no-sickness, coin"
        ),
    }
}

fn load_tabula(source: TabulaSource) -> Arc<Tabula> {
    let streaming_assets_path = logging::get_developer_mode_streaming_assets_path();
    let tabula_dir = Path::new(&streaming_assets_path).join("Tabula");
    let tabula = Tabula::load(source, &tabula_dir)
        .unwrap_or_else(|errors| panic!("Failed to load tabula: {errors:?}"));
    Arc::new(tabula)
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

    #[arg(long, help = "Run continuously until a crash is found")]
    stress: bool,

    #[arg(
        long,
        value_enum,
        default_value = "starting-five",
        help = "Deck to use for both players"
    )]
    deck: DeckChoice,

    #[arg(
        long,
        default_value = "none",
        help = "Balance mode: none, extra-card, bonus-energy, bonus-energy-no-draw, bonus-points, no-sickness, coin"
    )]
    balance: String,
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

struct MatchResources {
    deck_name: TestDeckName,
    dreamwell_list: DreamwellCardIdList,
    tabula: Arc<Tabula>,
    balance_mode: BalanceMode,
}

#[derive(Debug)]
enum MatchOutcome {
    Winner(PlayerName, usize, std::time::Duration),
    Draw(usize, std::time::Duration),
}

struct CrashInfo {
    seed: u64,
    match_index: usize,
    panic_message: String,
    backtrace: String,
}

fn catch_panic<F, T>(function: F) -> Result<T, (String, String)>
where
    F: FnOnce() -> T,
{
    PANIC_INFO.with(|info| {
        *info.borrow_mut() = None;
    });

    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(|panic_info| {
        let location_str = match panic_info.location() {
            Some(location) => format!("{}:{}", location.file(), location.line()),
            None => "unknown location".to_string(),
        };

        let panic_msg = format!("{panic_info}");
        let backtrace = Backtrace::new();
        let backtrace_str = format!("{backtrace:?}");

        PANIC_INFO.with(|info| {
            *info.borrow_mut() = Some((location_str, panic_msg, backtrace_str));
        });
    }));

    let result = panic::catch_unwind(AssertUnwindSafe(function));

    panic::set_hook(prev_hook);

    match result {
        Ok(value) => Ok(value),
        Err(panic_error) => {
            let panic_msg = match panic_error.downcast_ref::<&'static str>() {
                Some(s) => s.to_string(),
                None => match panic_error.downcast_ref::<String>() {
                    Some(s) => s.clone(),
                    None => "Unknown panic".to_string(),
                },
            };

            let (message, backtrace) = PANIC_INFO.with(|info| {
                if let Some((location, info, backtrace)) = &*info.borrow() {
                    (format!("{panic_msg} at {location}\n\nDetails:\n{info}"), backtrace.clone())
                } else {
                    (format!("{panic_msg}\n\nNo backtrace available"), String::new())
                }
            });

            Err((message, backtrace))
        }
    }
}

fn run_match(
    ai_one: &str,
    ai_two: &str,
    seed: u64,
    verbosity: Verbosity,
    swap_positions: bool,
    resources: &MatchResources,
) -> Result<(MatchOutcome, MatchActionStats), (String, String)> {
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

    let ai_one_str = ai_one.to_string();
    let ai_two_str = ai_two.to_string();
    let deck_name = resources.deck_name;
    let dreamwell_list = resources.dreamwell_list.clone();
    let tabula = resources.tabula.clone();
    let balance_mode = resources.balance_mode;

    catch_panic(move || {
        let log_directory =
            logging::get_developer_mode_project_directory().ok().map(|p| p.join("matchup_logs"));
        if let Some(log_dir) = &log_directory {
            let _ = fs::create_dir_all(log_dir);
        }
        let battle_id = BattleId(Uuid::new_v4());
        let mut battle = new_test_battle::create_and_start(
            battle_id,
            tabula.clone(),
            seed,
            Dreamwell::from_card_list(&tabula, dreamwell_list),
            CreateBattlePlayer { player_type: battle_ai_one, deck_name },
            CreateBattlePlayer { player_type: battle_ai_two, deck_name },
            RequestContext {
                logging_options: LoggingOptions {
                    log_directory,
                    log_ai_decisions: true,
                    ..LoggingOptions::default()
                },
            },
            PlayerName::One,
            None,
            None,
            balance_mode,
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
                        (PlayerName::One, false) | (PlayerName::Two, true) => &ai_one_str,
                        (PlayerName::Two, false) | (PlayerName::One, true) => &ai_two_str,
                    };

                    let legal_actions = legal_actions::compute(&battle, player);
                    let action_start = Instant::now();
                    let action = if legal_actions.len() == 1 {
                        legal_actions.all()[0]
                    } else {
                        agent_search::select_action_unchecked(&battle, player, &player_ai, None)
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
            BattleStatus::GameOver { winner: None } => {
                (MatchOutcome::Draw(turn_count, elapsed), stats)
            }
            BattleStatus::GameOver { winner: Some(winner) } => {
                (MatchOutcome::Winner(winner, turn_count, elapsed), stats)
            }
            _ => panic!("Game ended without a winner"),
        }
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if !args.stress && args.matches == 0 {
        return Err("Number of matches must be greater than 0".into());
    }

    let resources = MatchResources {
        deck_name: args.deck.to_test_deck_name(),
        dreamwell_list: args.deck.dreamwell_list(),
        tabula: load_tabula(args.deck.tabula_source()),
        balance_mode: parse_balance_mode(&args.balance),
    };

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

    if args.stress {
        println!(
            "Stress testing with {} vs {} (deck: {:?})",
            args.player_one_ai, args.player_two_ai, args.deck,
        );

        let mut match_index: usize = 0;
        loop {
            let swap_positions = match_index % 2 == 1;
            let match_seed = args.seed.wrapping_add(match_index as u64);

            let result = run_match(
                &args.player_one_ai,
                &args.player_two_ai,
                match_seed,
                Verbosity::None,
                swap_positions,
                &resources,
            );

            match result {
                Ok((outcome, _stats)) => {
                    match_index += 1;
                    match &outcome {
                        MatchOutcome::Winner(_, _, _) => {}
                        MatchOutcome::Draw(_, _) => {}
                    }
                    if match_index.is_multiple_of(10) {
                        println!("Completed {match_index} matches without crash...");
                    }
                }
                Err((panic_message, backtrace)) => {
                    let crash =
                        CrashInfo { seed: match_seed, match_index, panic_message, backtrace };
                    println!("\n========== CRASH FOUND ==========");
                    println!("Match index: {}", crash.match_index);
                    println!("Seed: {}", crash.seed);
                    println!("Swap positions: {swap_positions}");
                    println!("Deck: {:?}", args.deck);
                    println!("\nPanic message:\n{}", crash.panic_message);
                    println!("\nBacktrace:\n{}", crash.backtrace);
                    println!("\nReproduce with:");
                    println!(
                        "  just matchup '{}' '{}' --seed {} --deck {:?} -v actions",
                        args.player_one_ai, args.player_two_ai, crash.seed, args.deck,
                    );
                    println!("==================================");
                    process::exit(1);
                }
            }
        }
    }

    if args.matches > 1 {
        println!(
            "Running {} matches between {} and {}",
            args.matches, args.player_one_ai, args.player_two_ai
        );
    }

    for match_index in 0..args.matches {
        let swap_positions = match_index % 2 == 1;
        let match_seed = args.seed.wrapping_add(match_index as u64);

        let match_verbosity =
            if args.verbosity == Verbosity::Verbose { Verbosity::Actions } else { args.verbosity };

        if args.matches > 1 {
            print!("Match {}/{}: ", match_index + 1, args.matches);
            io::stdout().flush().unwrap();
        }

        let result = run_match(
            &args.player_one_ai,
            &args.player_two_ai,
            match_seed,
            match_verbosity,
            swap_positions,
            &resources,
        );

        let (outcome, stats) = match result {
            Ok(r) => r,
            Err((panic_message, backtrace)) => {
                println!("\n========== CRASH ==========");
                println!("Seed: {match_seed}");
                println!("Panic: {panic_message}");
                if !backtrace.is_empty() {
                    println!("Backtrace:\n{backtrace}");
                }
                println!("============================");
                continue;
            }
        };

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
