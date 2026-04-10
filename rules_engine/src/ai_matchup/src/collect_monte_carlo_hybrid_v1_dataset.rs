use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ai_agents::agent_search;
use ai_data::game_ai::GameAI;
use ai_uct::hybrid_dataset::PolicyTrainingRow;
use ai_uct::uct_config::UctConfig;
use ai_uct::{hybrid_features, uct_search_v4};
use battle_mutations::actions::apply_battle_action;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::LegalActions;
use battle_state::battle::battle_rules_config::BalanceMode;
use battle_state::battle::battle_state::{BattleState, LoggingOptions, RequestContext};
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

struct MatchResources {
    deck_name: TestDeckName,
    dreamwell_list: DreamwellCardIdList,
    tabula: Arc<Tabula>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let output_dir = args.output_dir.unwrap_or_else(default_output_dir);
    fs::create_dir_all(&output_dir)?;
    let policy_path = output_dir.join("policy.jsonl");
    fs::write(&policy_path, "")?;

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
        )?;
    }

    println!("Collected MonteCarloHybridV1 dataset at {}", output_dir.display());
    Ok(())
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

fn collect_match(
    seed: u64,
    teacher_iterations: u32,
    resources: &MatchResources,
    policy_path: &Path,
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
        BalanceMode::None,
    );

    let teacher_ai = GameAI::MonteCarloV4(teacher_iterations);
    let teacher_config = UctConfig {
        max_iterations_per_action: teacher_iterations * 1000,
        max_total_actions_multiplier: 6,
        iteration_multiplier_override: None,
        single_threaded: false,
    };
    let mut policy_rows = Vec::new();

    while !matches!(battle.status, BattleStatus::GameOver { .. }) {
        let Some(player) = legal_actions::next_to_act(&battle) else {
            break;
        };

        let legal = legal_actions::compute(&battle, player);
        let action = if legal.len() == 1 {
            legal.all()[0]
        } else if is_stable_root(&battle) {
            let summary = uct_search_v4::search_summary(&battle, player, &teacher_config);
            let state_features = hybrid_features::extract_state_features(&battle, player);
            let total_visits =
                summary.action_results.iter().map(|result| result.visit_count).sum::<u32>().max(1);
            policy_rows.extend(summary.action_results.iter().map(|result| PolicyTrainingRow {
                action: format!("{:?}", result.action),
                action_short: result.action.battle_action_string(),
                avg_reward: result.avg_reward,
                chosen: result.action == summary.action,
                legal_action_count: summary.num_actions,
                player: format!("{player:?}"),
                seed,
                state_features: state_features.clone(),
                action_features: hybrid_features::extract_action_features(
                    &battle,
                    player,
                    result.action,
                ),
                turn_id: battle.turn.turn_id.0,
                visit_count: result.visit_count,
                visit_fraction: f64::from(result.visit_count) / f64::from(total_visits),
            }));
            summary.action
        } else {
            agent_search::select_action_unchecked(&battle, player, &teacher_ai, None)
        };

        apply_battle_action::execute(&mut battle, player, action);
    }

    append_jsonl(policy_path, &policy_rows)?;
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
        .join("monte_carlo_hybrid_v1_dataset")
}

fn is_stable_root(battle: &BattleState) -> bool {
    if battle.cards.has_stack()
        || battle.stack_priority.is_some()
        || !battle.prompts.is_empty()
        || battle.turn.positioning_started
        || battle.turn.positioning_character.is_some()
    {
        return false;
    }

    legal_actions::next_to_act(battle).is_some_and(|player| {
        matches!(legal_actions::compute(battle, player), LegalActions::Standard { .. })
    })
}

fn load_tabula(source: TabulaSource) -> Arc<Tabula> {
    let streaming_assets_path = logging::get_developer_mode_streaming_assets_path();
    let tabula_dir = Path::new(&streaming_assets_path).join("Tabula");
    Arc::new(
        Tabula::load(source, &tabula_dir)
            .unwrap_or_else(|errors| panic!("Failed to load tabula: {errors:?}")),
    )
}
