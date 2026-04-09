use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ai_agents::agent_search;
use ai_data::game_ai::GameAI;
use ai_strategic::dataset::{PolicyCandidateRow, PolicyTrainingRow, ValueTrainingRow};
use ai_strategic::{feature_extraction, search_v3};
use ai_uct::uct_config::UctConfig;
use ai_uct::uct_search_v4;
use battle_mutations::actions::apply_battle_action;
use battle_queries::legal_action_queries::legal_actions;
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
use tabula_data::tabula::{Tabula, TabulaSource};
use tabula_generated::card_lists::DreamwellCardIdList;
use uuid::Uuid;

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
enum DeckChoice {
    Vanilla,
    StartingFive,
    Benchmark1,
    Core11,
}

impl DeckChoice {
    fn dreamwell_list(self) -> DreamwellCardIdList {
        match self {
            DeckChoice::Core11 => DreamwellCardIdList::DreamwellBasic5,
            _ => DreamwellCardIdList::TestDreamwellNoAbilities,
        }
    }

    fn tabula_source(self) -> TabulaSource {
        match self {
            DeckChoice::Core11 => TabulaSource::Production,
            _ => TabulaSource::Test,
        }
    }

    fn to_test_deck_name(self) -> TestDeckName {
        match self {
            DeckChoice::Vanilla => TestDeckName::Vanilla,
            DeckChoice::StartingFive => TestDeckName::StartingFive,
            DeckChoice::Benchmark1 => TestDeckName::Benchmark1,
            DeckChoice::Core11 => TestDeckName::Core11,
        }
    }
}

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "12")]
    matches: usize,

    #[arg(long, default_value = "core11")]
    deck: DeckChoice,

    #[arg(long, default_value = "50")]
    teacher_iterations: u32,

    #[arg(long, default_value = "3141592653")]
    seed: u64,

    #[arg(long)]
    output_dir: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let output_dir = args.output_dir.unwrap_or_else(default_output_dir);
    fs::create_dir_all(&output_dir)?;
    let policy_path = output_dir.join("policy.jsonl");
    let value_path = output_dir.join("value.jsonl");
    fs::write(&policy_path, "")?;
    fs::write(&value_path, "")?;

    let resources = MatchResources {
        deck_name: args.deck.to_test_deck_name(),
        dreamwell_list: args.deck.dreamwell_list(),
        tabula: load_tabula(args.deck.tabula_source()),
    };

    for match_index in 0..args.matches {
        collect_match(
            args.seed.wrapping_add(match_index as u64),
            args.teacher_iterations,
            &resources,
            &policy_path,
            &value_path,
        )?;
    }

    println!("Collected StrategicV3 dataset at {}", output_dir.display());
    Ok(())
}

struct MatchResources {
    deck_name: TestDeckName,
    dreamwell_list: DreamwellCardIdList,
    tabula: Arc<Tabula>,
}

fn collect_match(
    seed: u64,
    teacher_iterations: u32,
    resources: &MatchResources,
    policy_path: &Path,
    value_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut battle = new_test_battle::create_and_start(
        BattleId(Uuid::new_v4()),
        resources.tabula.clone(),
        seed,
        Dreamwell::from_card_list(&resources.tabula, resources.dreamwell_list.clone()),
        CreateBattlePlayer {
            player_type: PlayerType::Agent(GameAI::MonteCarloV4(teacher_iterations)),
            deck_name: resources.deck_name,
        },
        CreateBattlePlayer {
            player_type: PlayerType::Agent(GameAI::MonteCarloV4(teacher_iterations)),
            deck_name: resources.deck_name,
        },
        RequestContext {
            logging_options: LoggingOptions {
                log_ai_decisions: false,
                ..LoggingOptions::default()
            },
        },
        PlayerName::One,
        None,
        None,
    );

    let teacher_ai = GameAI::MonteCarloV4(teacher_iterations);
    let teacher_config = UctConfig {
        max_iterations_per_action: teacher_iterations * 1000,
        max_total_actions_multiplier: 6,
        iteration_multiplier_override: None,
        single_threaded: false,
    };
    let mut policy_rows = Vec::new();
    let mut value_rows = Vec::new();

    while !matches!(battle.status, BattleStatus::GameOver { .. }) {
        let Some(player) = legal_actions::next_to_act(&battle) else {
            break;
        };
        if !search_v3::is_tactical_state(&battle) {
            value_rows.push(ValueTrainingRow {
                outcome: 0.0,
                player: format!("{:?}", PlayerName::One),
                seed,
                state_features: feature_extraction::extract_state_features(
                    &battle,
                    PlayerName::One,
                ),
                turn_id: battle.turn.turn_id.0,
            });
            value_rows.push(ValueTrainingRow {
                outcome: 0.0,
                player: format!("{:?}", PlayerName::Two),
                seed,
                state_features: feature_extraction::extract_state_features(
                    &battle,
                    PlayerName::Two,
                ),
                turn_id: battle.turn.turn_id.0,
            });
        }

        let legal = legal_actions::compute(&battle, player);
        let action = if legal.len() == 1 {
            legal.all()[0]
        } else if !search_v3::is_tactical_state(&battle) {
            let summary = uct_search_v4::search_summary(&battle, player, &teacher_config);
            let state_features = feature_extraction::extract_state_features(&battle, player);
            policy_rows.push(PolicyTrainingRow {
                candidates: summary
                    .action_results
                    .iter()
                    .map(|result| PolicyCandidateRow {
                        action: format!("{:?}", result.action),
                        action_short: result.action.battle_action_string(),
                        action_features: feature_extraction::extract_action_features(
                            &battle,
                            player,
                            result.action,
                        ),
                        avg_reward: result.avg_reward,
                        chosen: result.action == summary.action,
                        visit_count: result.visit_count,
                    })
                    .collect(),
                chosen_action_short: summary.action.battle_action_string(),
                legal_action_count: summary.num_actions,
                player: format!("{:?}", player),
                seed,
                state_features,
                turn_id: battle.turn.turn_id.0,
            });
            summary.action
        } else {
            agent_search::select_action_unchecked(&battle, player, &teacher_ai, None)
        };

        apply_battle_action::execute(&mut battle, player, action);
    }

    let winner = match battle.status {
        BattleStatus::GameOver { winner } => winner,
        _ => None,
    };

    for row in &mut value_rows {
        let player = if row.player == "One" { PlayerName::One } else { PlayerName::Two };
        row.outcome = match winner {
            Some(winner) if winner == player => 1.0,
            Some(_) => -1.0,
            None => 0.0,
        };
    }

    append_jsonl(policy_path, &policy_rows)?;
    append_jsonl(value_path, &value_rows)?;
    Ok(())
}

fn append_jsonl<T: serde::Serialize>(
    path: &Path,
    rows: &[T],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    for row in rows {
        writeln!(file, "{}", serde_json::to_string(row)?)?;
    }
    Ok(())
}

fn default_output_dir() -> PathBuf {
    logging::get_developer_mode_project_directory()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("strategic_v3_dataset")
}

fn load_tabula(source: TabulaSource) -> Arc<Tabula> {
    let streaming_assets_path = logging::get_developer_mode_streaming_assets_path();
    let tabula_dir = Path::new(&streaming_assets_path).join("Tabula");
    Arc::new(
        Tabula::load(source, &tabula_dir)
            .unwrap_or_else(|errors| panic!("Failed to load tabula: {errors:?}")),
    )
}
