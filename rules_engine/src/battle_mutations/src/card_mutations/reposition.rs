use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CharacterId;
use battle_state::battle_cards::battlefield::Battlefield;
use core_data::types::PlayerName;

/// Moves a character to a specific front rank position.
///
/// If the target slot is occupied, the occupant swaps to the mover's old slot.
pub fn to_front_rank(
    battle: &mut BattleState,
    player: PlayerName,
    character_id: CharacterId,
    position: u8,
) {
    let bf = battle.cards.battlefield_mut(player);
    let source = find_slot(bf, character_id);
    let target_occupant = bf.front[position as usize];

    remove_at(bf, &source);

    if let Some(occupant) = target_occupant {
        bf.front[position as usize] = None;
        place_at(bf, &source, occupant);
    }

    bf.front[position as usize] = Some(character_id);
    battle.turn.moved_this_turn.push(character_id);
}

/// Moves a character to a specific back rank position.
///
/// If the target slot is occupied, the occupant swaps to the mover's old slot.
pub fn to_back_rank(
    battle: &mut BattleState,
    player: PlayerName,
    character_id: CharacterId,
    position: u8,
) {
    let bf = battle.cards.battlefield_mut(player);
    let source = find_slot(bf, character_id);
    let target_occupant = bf.back[position as usize];

    remove_at(bf, &source);

    if let Some(occupant) = target_occupant {
        bf.back[position as usize] = None;
        place_at(bf, &source, occupant);
    }

    bf.back[position as usize] = Some(character_id);
    battle.turn.moved_this_turn.push(character_id);
}

enum SlotLocation {
    Front(usize),
    Back(usize),
}

fn find_slot(bf: &Battlefield, character_id: CharacterId) -> SlotLocation {
    for (i, slot) in bf.front.iter().enumerate() {
        if *slot == Some(character_id) {
            return SlotLocation::Front(i);
        }
    }
    for (i, slot) in bf.back.iter().enumerate() {
        if *slot == Some(character_id) {
            return SlotLocation::Back(i);
        }
    }
    panic!("Character {character_id:?} not found on battlefield");
}

fn remove_at(bf: &mut Battlefield, slot: &SlotLocation) {
    match slot {
        SlotLocation::Front(i) => bf.front[*i] = None,
        SlotLocation::Back(i) => bf.back[*i] = None,
    }
}

fn place_at(bf: &mut Battlefield, slot: &SlotLocation, character_id: CharacterId) {
    match slot {
        SlotLocation::Front(i) => bf.front[*i] = Some(character_id),
        SlotLocation::Back(i) => bf.back[*i] = Some(character_id),
    }
}
