use std::path::{Path, PathBuf};
use std::sync::Arc;

use ai_data::game_ai::GameAI;
use battle_mutations::card_mutations::move_card;
use battle_state::battle::battle_state::{BattleState, RequestContext};
use battle_state::battle::card_id::BattleDeckCardId;
use battle_state::battle_cards::dreamwell_data::Dreamwell;
use battle_state::battle_player::battle_player_state::{
    CreateBattlePlayer, PlayerType, TestDeckName,
};
use battle_state::core::effect_source::EffectSource;
use core_data::identifiers::{BattleId, QuestId, UserId};
use core_data::numerics::Energy;
use core_data::types::PlayerName;
use database::quest_save_file::QuestSaveFile;
use database::save_file::{SaveFile, SaveFileV1};
use database::save_file_io;
use game_creation::new_test_battle;
use rand::RngCore;
use tabula_data::tabula::{Tabula, TabulaSource};
use tabula_generated::card_lists::DreamwellCardIdList;
use uuid::Uuid;

/// User ID used by the Unity client in development mode.
const UNITY_USER_ID: Uuid = uuid::uuid!("d2da9785-f20e-4879-bed5-35b2e1926faf");

/// Generates a test save file with customizable battle parameters.
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut energy: Option<u32> = None;
    let mut card_names: Vec<String> = Vec::new();
    let mut save_dir: Option<PathBuf> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--energy" => {
                i += 1;
                energy = Some(args[i].parse().expect("Invalid energy value"));
            }
            "--card" => {
                i += 1;
                card_names.push(args[i].clone());
            }
            "--save-dir" => {
                i += 1;
                save_dir = Some(PathBuf::from(&args[i]));
            }
            "--help" | "-h" => {
                print_usage();
                return;
            }
            "--list-cards" => {
                list_cards();
                return;
            }
            other => {
                eprintln!("Unknown argument: {other}");
                print_usage();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let save_dir = save_dir.unwrap_or_else(default_save_dir);
    let tabula = load_tabula();
    let mut battle = create_battle(tabula.clone());

    if let Some(e) = energy {
        set_energy(&mut battle, Energy(e));
        eprintln!("Set energy to {e}");
    }

    for name in &card_names {
        move_card_to_hand(&mut battle, &tabula, name);
    }

    write_save(&battle, &save_dir);
    eprintln!("Save file written to {}", save_dir.display());
}

/// Prints usage information.
fn print_usage() {
    eprintln!("Usage: test_save_generator [OPTIONS]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --energy <N>        Set player energy to N");
    eprintln!("  --card <NAME>       Add a card to player's hand (can be repeated)");
    eprintln!("  --save-dir <DIR>    Override save file directory");
    eprintln!("  --list-cards        List all available card names");
    eprintln!("  --help              Show this help message");
}

/// Returns the default macOS save directory.
fn default_save_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Cannot determine home directory")
        .join("Library/Application Support/Dreamtides/Dreamtides")
}

/// Loads the Tabula card database from the local tabula/ directory.
fn load_tabula() -> Arc<Tabula> {
    let tabula_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tabula");
    Arc::new(
        Tabula::load(TabulaSource::Production, &tabula_dir).expect("Failed to load tabula data"),
    )
}

/// Creates a new battle with Core11 decks and default dreamwell.
fn create_battle(tabula: Arc<Tabula>) -> BattleState {
    let user_id = UserId(UNITY_USER_ID);
    let seed = rand::rng().next_u64();
    let dreamwell = Dreamwell::from_card_list(&tabula, DreamwellCardIdList::DreamwellBasic5);
    new_test_battle::create_and_start(
        BattleId(Uuid::new_v4()),
        tabula,
        seed,
        dreamwell,
        CreateBattlePlayer {
            player_type: PlayerType::User(user_id),
            deck_name: TestDeckName::Core11,
        },
        CreateBattlePlayer {
            player_type: PlayerType::Agent(GameAI::MonteCarlo(5)),
            deck_name: TestDeckName::Core11,
        },
        RequestContext::default(),
    )
}

/// Sets both current and produced energy for player one.
fn set_energy(battle: &mut BattleState, amount: Energy) {
    battle.players.one.current_energy = amount;
    battle.players.one.produced_energy = amount;
}

/// Finds a card by name in the player's deck and moves it to their hand.
fn move_card_to_hand(battle: &mut BattleState, tabula: &Tabula, card_name: &str) {
    let target_name_lower = card_name.to_lowercase();

    // Find all deck cards for player one (both shuffled and top-of-deck)
    let deck_cards: Vec<BattleDeckCardId> = battle.cards.all_deck_cards(PlayerName::One).collect();

    for deck_card_id in deck_cards {
        let identity = battle.cards[deck_card_id].identity;
        let definition = battle.card_definitions.get_definition(identity);
        if definition.displayed_name.to_lowercase() == target_name_lower {
            move_card::from_deck_to_hand(
                battle,
                EffectSource::Game { controller: PlayerName::One },
                PlayerName::One,
                deck_card_id,
            );
            eprintln!("Moved '{}' to hand", definition.displayed_name);
            return;
        }
    }

    // Card not in deck, check if it's in tabula at all
    let exists_in_tabula =
        tabula.cards.values().any(|def| def.displayed_name.to_lowercase() == target_name_lower);

    if exists_in_tabula {
        eprintln!(
            "Warning: '{card_name}' exists but is not in the player's deck. \
             It may already be in hand."
        );
    } else {
        let mut closest: Option<(usize, &str)> = None;
        for def in tabula.cards.values() {
            let dist = levenshtein(&target_name_lower, &def.displayed_name.to_lowercase());
            if closest.is_none() || dist < closest.unwrap().0 {
                closest = Some((dist, &def.displayed_name));
            }
        }
        eprintln!("Error: Card '{card_name}' not found in card database.");
        if let Some((_, suggestion)) = closest {
            eprintln!("  Did you mean '{suggestion}'?");
        }
        std::process::exit(1);
    }
}

/// Lists all available card names.
fn list_cards() {
    let tabula = load_tabula();
    let mut names: Vec<&str> =
        tabula.cards.values().map(|def| def.displayed_name.as_str()).collect();
    names.sort();
    for name in names {
        println!("{name}");
    }
}

/// Writes the battle state as a save file.
fn write_save(battle: &BattleState, save_dir: &Path) {
    let user_id = UserId(UNITY_USER_ID);
    let quest_id = QuestId(Uuid::new_v4());
    let save = SaveFile::V1(Box::new(SaveFileV1 {
        id: user_id,
        quest: Some(QuestSaveFile { id: quest_id, battle: Some(battle.clone()) }),
    }));
    save_file_io::write_save_to_dir(save_dir, &save).expect("Failed to write save file");
}

/// Computes the Levenshtein distance between two strings.
fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let n = b_chars.len();
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr = vec![0usize; n + 1];
    for (i, &a_ch) in a_chars.iter().enumerate() {
        curr[0] = i + 1;
        for (j, &b_ch) in b_chars.iter().enumerate() {
            let cost = if a_ch == b_ch { 0 } else { 1 };
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
}
