use std::cmp::Ordering;
use std::fmt::Write;

use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CharacterId;
use core_data::types::PlayerName;

/// How a character should be positioned during the positioning phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterPlacement {
    StayBack,
    MoveToFrontRank(u8),
}

/// A complete set of positioning decisions for all eligible characters.
#[derive(Debug, Clone)]
pub struct PositionAssignment {
    pub placements: Vec<(CharacterId, CharacterPlacement)>,
}

/// Generates candidate position assignments covering the strategic spectrum.
pub fn generate(battle: &BattleState, player: PlayerName) -> Vec<PositionAssignment> {
    let eligible = eligible_characters(battle, player);
    if eligible.is_empty() {
        return Vec::new();
    }

    let threats = opponent_front_threats(battle, player.opponent());
    let opp_max = opponent_max_spark(battle, player.opponent());
    let mut candidates: Vec<PositionAssignment> = Vec::new();

    // 1. Hold all back
    let hold_all = build_hold_all(&eligible);
    push_unique(&mut candidates, hold_all);

    // 2. Efficient blocking
    if let Some(assignment) = build_efficient_blocking(&eligible, &threats) {
        push_unique(&mut candidates, assignment);
    }

    // 3. Winning blocks — use the smallest character that strictly outclasses each
    //    threat so the blocker survives and gets a free kill.
    if let Some(assignment) = build_winning_blocks(&eligible, &threats) {
        push_unique(&mut candidates, assignment);
    }

    // 4. Winning blocks + attack — after assigning winning blockers, send remaining
    //    high-spark characters to attack empty lanes.
    if let Some(assignment) =
        build_winning_blocks_and_attack(battle, player, &eligible, &threats, opp_max)
    {
        push_unique(&mut candidates, assignment);
    }

    // 5. Chump blocking
    if let Some(assignment) = build_chump_blocking(&eligible, &threats) {
        push_unique(&mut candidates, assignment);
    }

    // 6. Attack-focused
    if let Some(assignment) = build_attack_focused(battle, player, &eligible, opp_max) {
        push_unique(&mut candidates, assignment);
    }

    // 7. Mixed
    if let Some(assignment) = build_mixed(battle, player, &eligible, &threats, opp_max) {
        push_unique(&mut candidates, assignment);
    }

    // 8. Chump-block-all
    if let Some(assignment) = build_chump_block_all(&eligible, &threats) {
        push_unique(&mut candidates, assignment);
    }

    candidates
}

/// Returns the single best assignment by heuristic score.
pub fn best_assignment(battle: &BattleState, player: PlayerName) -> Option<PositionAssignment> {
    let candidates = generate(battle, player);
    if candidates.is_empty() {
        return None;
    }

    let threats = opponent_front_threats(battle, player.opponent());
    let opp_max = opponent_max_spark(battle, player.opponent());

    candidates.into_iter().max_by(|a, b| {
        score(battle, player, a, &threats, opp_max)
            .partial_cmp(&score(battle, player, b, &threats, opp_max))
            .unwrap_or(Ordering::Equal)
    })
}

/// Returns a human-readable description of a position assignment.
pub fn describe(
    battle: &BattleState,
    player: PlayerName,
    assignment: &PositionAssignment,
) -> String {
    let threats = opponent_front_threats(battle, player.opponent());
    let mut parts: Vec<String> = Vec::new();

    let all_back = assignment.placements.iter().all(|(_, p)| *p == CharacterPlacement::StayBack);
    if all_back {
        return "hold-all".to_string();
    }

    for &(char_id, placement) in &assignment.placements {
        if let CharacterPlacement::MoveToFrontRank(col) = placement {
            let our_spark = battle.cards.spark(player, char_id).map_or(0, |s| s.0);
            let threat_at_col = threats.iter().find(|t| t.column == col);
            match threat_at_col {
                Some(threat) if our_spark >= threat.spark => {
                    let mut part = format!("block-{}@col{}", our_spark, col);
                    let _ = write!(part, "(with-{})", threat.spark);
                    parts.push(part);
                }
                Some(threat) => {
                    let mut part = format!("chump-{}@col{}", our_spark, col);
                    let _ = write!(part, "(vs-{})", threat.spark);
                    parts.push(part);
                }
                None => {
                    parts.push(format!("attack-{}@col{}", our_spark, col));
                }
            }
        }
    }

    if parts.is_empty() { "hold-all".to_string() } else { parts.join("+") }
}

struct EligibleCharacter {
    id: CharacterId,
    spark: u32,
}

struct OpponentThreat {
    column: u8,
    spark: u32,
}

fn eligible_characters(battle: &BattleState, player: PlayerName) -> Vec<EligibleCharacter> {
    let bf = battle.cards.battlefield(player);
    let current_turn = battle.turn.turn_id.0;
    bf.back
        .iter()
        .flatten()
        .copied()
        .filter(|character_id| {
            let has_summoning_sickness = battle
                .cards
                .battlefield_state(player)
                .get(character_id)
                .is_some_and(|state| state.played_turn == current_turn);
            !has_summoning_sickness && !battle.turn.moved_this_turn.contains(character_id)
        })
        .map(|id| EligibleCharacter {
            id,
            spark: battle.cards.spark(player, id).map_or(0, |s| s.0),
        })
        .collect()
}

fn opponent_front_threats(battle: &BattleState, opponent: PlayerName) -> Vec<OpponentThreat> {
    battle
        .cards
        .battlefield(opponent)
        .front
        .iter()
        .enumerate()
        .filter_map(|(col, slot)| {
            slot.map(|char_id| OpponentThreat {
                column: col as u8,
                spark: battle.cards.spark(opponent, char_id).map_or(0, |s| s.0),
            })
        })
        .collect()
}

/// Returns the maximum spark among all opponent characters that could
/// potentially block an attacker — this includes both front and back row,
/// since back-row characters can be repositioned to block on the next turn.
fn opponent_max_spark(battle: &BattleState, opponent: PlayerName) -> u32 {
    let bf = battle.cards.battlefield(opponent);
    bf.front
        .iter()
        .chain(bf.back.iter())
        .flatten()
        .map(|&char_id| battle.cards.spark(opponent, char_id).map_or(0, |s| s.0))
        .max()
        .unwrap_or(0)
}

fn find_attack_column(battle: &BattleState, player: PlayerName) -> Option<u8> {
    let opponent_front = &battle.cards.battlefield(player.opponent()).front;
    let own_front = &battle.cards.battlefield(player).front;
    opponent_front
        .iter()
        .zip(own_front.iter())
        .position(|(opp, own)| opp.is_none() && own.is_none())
        .map(|col| col as u8)
}

fn same_placements(a: &PositionAssignment, b: &PositionAssignment) -> bool {
    if a.placements.len() != b.placements.len() {
        return false;
    }
    a.placements.iter().all(|(id, placement)| {
        b.placements.iter().any(|(bid, bp)| *id == *bid && *placement == *bp)
    })
}

fn push_unique(candidates: &mut Vec<PositionAssignment>, assignment: PositionAssignment) {
    if !candidates.iter().any(|c| same_placements(c, &assignment)) {
        candidates.push(assignment);
    }
}

fn build_hold_all(eligible: &[EligibleCharacter]) -> PositionAssignment {
    PositionAssignment {
        placements: eligible.iter().map(|c| (c.id, CharacterPlacement::StayBack)).collect(),
    }
}

fn build_efficient_blocking(
    eligible: &[EligibleCharacter],
    threats: &[OpponentThreat],
) -> Option<PositionAssignment> {
    if threats.is_empty() {
        return None;
    }

    let mut sorted_threats: Vec<&OpponentThreat> = threats.iter().collect();
    sorted_threats.sort_by(|a, b| b.spark.cmp(&a.spark));

    let mut used: Vec<bool> = vec![false; eligible.len()];
    let mut assignments: Vec<(CharacterId, u8)> = Vec::new();

    for threat in &sorted_threats {
        let best_blocker = eligible
            .iter()
            .enumerate()
            .filter(|(i, c)| !used[*i] && c.spark >= threat.spark)
            .min_by_key(|(_, c)| c.spark);
        if let Some((idx, blocker)) = best_blocker {
            used[idx] = true;
            assignments.push((blocker.id, threat.column));
        }
    }

    if assignments.is_empty() {
        return None;
    }

    let placements = eligible
        .iter()
        .map(|c| {
            if let Some((_, col)) = assignments.iter().find(|(id, _)| *id == c.id) {
                (c.id, CharacterPlacement::MoveToFrontRank(*col))
            } else {
                (c.id, CharacterPlacement::StayBack)
            }
        })
        .collect();

    Some(PositionAssignment { placements })
}

/// Assigns blockers that strictly outclass each threat (spark > threat.spark),
/// so the blocker survives combat. Uses the smallest such character per threat.
fn build_winning_blocks(
    eligible: &[EligibleCharacter],
    threats: &[OpponentThreat],
) -> Option<PositionAssignment> {
    if threats.is_empty() {
        return None;
    }

    let mut sorted_threats: Vec<&OpponentThreat> = threats.iter().collect();
    sorted_threats.sort_by(|a, b| b.spark.cmp(&a.spark));

    let mut used: Vec<bool> = vec![false; eligible.len()];
    let mut assignments: Vec<(CharacterId, u8)> = Vec::new();

    for threat in &sorted_threats {
        let winner = eligible
            .iter()
            .enumerate()
            .filter(|(i, c)| !used[*i] && c.spark > threat.spark)
            .min_by_key(|(_, c)| c.spark);
        if let Some((idx, blocker)) = winner {
            used[idx] = true;
            assignments.push((blocker.id, threat.column));
        }
    }

    if assignments.is_empty() {
        return None;
    }

    let placements = eligible
        .iter()
        .map(|c| {
            if let Some((_, col)) = assignments.iter().find(|(id, _)| *id == c.id) {
                (c.id, CharacterPlacement::MoveToFrontRank(*col))
            } else {
                (c.id, CharacterPlacement::StayBack)
            }
        })
        .collect();

    Some(PositionAssignment { placements })
}

/// Assigns winning blockers first, then sends remaining high-spark characters
/// to attack empty lanes.
fn build_winning_blocks_and_attack(
    battle: &BattleState,
    player: PlayerName,
    eligible: &[EligibleCharacter],
    threats: &[OpponentThreat],
    opp_max: u32,
) -> Option<PositionAssignment> {
    if threats.is_empty() {
        return None;
    }

    let own_front = &battle.cards.battlefield(player).front;
    let opponent_front = &battle.cards.battlefield(player.opponent()).front;

    let mut sorted_threats: Vec<&OpponentThreat> = threats.iter().collect();
    sorted_threats.sort_by(|a, b| b.spark.cmp(&a.spark));

    let mut used: Vec<bool> = vec![false; eligible.len()];
    let mut assignments: Vec<(CharacterId, u8)> = Vec::new();
    let mut used_cols: Vec<u8> = Vec::new();

    // First pass: assign winning blockers
    for threat in &sorted_threats {
        let winner = eligible
            .iter()
            .enumerate()
            .filter(|(i, c)| !used[*i] && c.spark > threat.spark)
            .min_by_key(|(_, c)| c.spark);
        if let Some((idx, blocker)) = winner {
            used[idx] = true;
            assignments.push((blocker.id, threat.column));
            used_cols.push(threat.column);
        }
    }

    if assignments.is_empty() {
        return None;
    }

    // Second pass: send remaining high-spark characters to attack
    let mut attackers: Vec<(usize, &EligibleCharacter)> =
        eligible.iter().enumerate().filter(|(i, c)| !used[*i] && c.spark > opp_max).collect();
    attackers.sort_by(|(_, a), (_, b)| b.spark.cmp(&a.spark));

    for (idx, attacker) in attackers {
        if let Some(col) = find_next_attack_column(own_front, opponent_front, &used_cols) {
            used[idx] = true;
            assignments.push((attacker.id, col));
            used_cols.push(col);
        }
    }

    let placements = eligible
        .iter()
        .map(|c| {
            if let Some((_, col)) = assignments.iter().find(|(id, _)| *id == c.id) {
                (c.id, CharacterPlacement::MoveToFrontRank(*col))
            } else {
                (c.id, CharacterPlacement::StayBack)
            }
        })
        .collect();

    Some(PositionAssignment { placements })
}

fn build_chump_blocking(
    eligible: &[EligibleCharacter],
    threats: &[OpponentThreat],
) -> Option<PositionAssignment> {
    if threats.is_empty() {
        return None;
    }

    let mut sorted_threats: Vec<&OpponentThreat> = threats.iter().collect();
    sorted_threats.sort_by(|a, b| b.spark.cmp(&a.spark));

    let mut sorted_eligible: Vec<(usize, &EligibleCharacter)> =
        eligible.iter().enumerate().collect();
    sorted_eligible.sort_by_key(|(_, c)| c.spark);

    let mut used: Vec<bool> = vec![false; eligible.len()];
    let mut assignments: Vec<(CharacterId, u8)> = Vec::new();

    for threat in &sorted_threats {
        let chump = sorted_eligible.iter().find(|(i, c)| !used[*i] && c.spark < threat.spark);
        if let Some(&(idx, blocker)) = chump {
            used[idx] = true;
            assignments.push((blocker.id, threat.column));
        }
    }

    if assignments.is_empty() {
        return None;
    }

    let placements = eligible
        .iter()
        .map(|c| {
            if let Some((_, col)) = assignments.iter().find(|(id, _)| *id == c.id) {
                (c.id, CharacterPlacement::MoveToFrontRank(*col))
            } else {
                (c.id, CharacterPlacement::StayBack)
            }
        })
        .collect();

    Some(PositionAssignment { placements })
}

fn build_attack_focused(
    battle: &BattleState,
    player: PlayerName,
    eligible: &[EligibleCharacter],
    opp_max: u32,
) -> Option<PositionAssignment> {
    let _ = find_attack_column(battle, player)?;
    let own_front = &battle.cards.battlefield(player).front;
    let opponent_front = &battle.cards.battlefield(player.opponent()).front;

    let mut sorted_eligible: Vec<&EligibleCharacter> =
        eligible.iter().filter(|c| c.spark > opp_max).collect();
    sorted_eligible.sort_by(|a, b| b.spark.cmp(&a.spark));

    if sorted_eligible.is_empty() {
        return None;
    }

    let mut assignments: Vec<(CharacterId, u8)> = Vec::new();
    let mut used_cols: Vec<u8> = Vec::new();

    for attacker in &sorted_eligible {
        if let Some(col) = find_next_attack_column(own_front, opponent_front, &used_cols) {
            assignments.push((attacker.id, col));
            used_cols.push(col);
        }
    }

    if assignments.is_empty() {
        return None;
    }

    let placements = eligible
        .iter()
        .map(|c| {
            if let Some((_, col)) = assignments.iter().find(|(id, _)| *id == c.id) {
                (c.id, CharacterPlacement::MoveToFrontRank(*col))
            } else {
                (c.id, CharacterPlacement::StayBack)
            }
        })
        .collect();

    Some(PositionAssignment { placements })
}

fn find_next_attack_column(
    own_front: &[Option<CharacterId>; 4],
    opponent_front: &[Option<CharacterId>; 4],
    used_cols: &[u8],
) -> Option<u8> {
    own_front
        .iter()
        .zip(opponent_front.iter())
        .enumerate()
        .position(|(col, (own, opp))| {
            own.is_none() && opp.is_none() && !used_cols.contains(&(col as u8))
        })
        .map(|col| col as u8)
}

fn build_mixed(
    battle: &BattleState,
    player: PlayerName,
    eligible: &[EligibleCharacter],
    threats: &[OpponentThreat],
    opp_max: u32,
) -> Option<PositionAssignment> {
    if threats.is_empty() {
        return None;
    }

    let highest_threat = threats.iter().max_by_key(|t| t.spark)?;

    let blocker = eligible
        .iter()
        .filter(|c| c.spark >= highest_threat.spark)
        .min_by_key(|c| c.spark)
        .or_else(|| eligible.iter().min_by_key(|c| c.spark));
    let blocker = blocker?;

    let own_front = &battle.cards.battlefield(player).front;
    let opponent_front = &battle.cards.battlefield(player.opponent()).front;
    let used_cols = vec![highest_threat.column];

    let attacker =
        eligible.iter().filter(|c| c.id != blocker.id && c.spark > opp_max).max_by_key(|c| c.spark);

    let attack_assignment = attacker.and_then(|a| {
        find_next_attack_column(own_front, opponent_front, &used_cols).map(|col| (a.id, col))
    });

    let placements = eligible
        .iter()
        .map(|c| {
            if c.id == blocker.id {
                (c.id, CharacterPlacement::MoveToFrontRank(highest_threat.column))
            } else if let Some((atk_id, col)) = &attack_assignment {
                if c.id == *atk_id {
                    (c.id, CharacterPlacement::MoveToFrontRank(*col))
                } else {
                    (c.id, CharacterPlacement::StayBack)
                }
            } else {
                (c.id, CharacterPlacement::StayBack)
            }
        })
        .collect();

    Some(PositionAssignment { placements })
}

fn build_chump_block_all(
    eligible: &[EligibleCharacter],
    threats: &[OpponentThreat],
) -> Option<PositionAssignment> {
    if threats.len() < 2 {
        return None;
    }

    let mut sorted_threats: Vec<&OpponentThreat> = threats.iter().collect();
    sorted_threats.sort_by(|a, b| b.spark.cmp(&a.spark));

    let mut sorted_eligible: Vec<(usize, &EligibleCharacter)> =
        eligible.iter().enumerate().collect();
    sorted_eligible.sort_by_key(|(_, c)| c.spark);

    let mut used: Vec<bool> = vec![false; eligible.len()];
    let mut assignments: Vec<(CharacterId, u8)> = Vec::new();

    for threat in &sorted_threats {
        let blocker = sorted_eligible.iter().find(|(i, _)| !used[*i]);
        if let Some(&(idx, b)) = blocker {
            used[idx] = true;
            assignments.push((b.id, threat.column));
        }
    }

    if assignments.len() < 2 {
        return None;
    }

    let placements = eligible
        .iter()
        .map(|c| {
            if let Some((_, col)) = assignments.iter().find(|(id, _)| *id == c.id) {
                (c.id, CharacterPlacement::MoveToFrontRank(*col))
            } else {
                (c.id, CharacterPlacement::StayBack)
            }
        })
        .collect();

    Some(PositionAssignment { placements })
}

fn score(
    battle: &BattleState,
    player: PlayerName,
    assignment: &PositionAssignment,
    threats: &[OpponentThreat],
    opp_max: u32,
) -> f64 {
    let mut points_prevented: f64 = 0.0;
    let mut spark_lost: f64 = 0.0;
    let mut attack_points: f64 = 0.0;

    for &(char_id, placement) in &assignment.placements {
        if let CharacterPlacement::MoveToFrontRank(col) = placement {
            let our_spark = battle.cards.spark(player, char_id).map_or(0, |s| s.0);
            let threat_at_col = threats.iter().find(|t| t.column == col);
            match threat_at_col {
                Some(threat) => {
                    points_prevented += threat.spark as f64;
                    if our_spark <= threat.spark {
                        spark_lost += our_spark as f64;
                    }
                }
                None => {
                    let discount = if opp_max >= our_spark { 0.25 } else { 0.75 };
                    attack_points += our_spark as f64 * discount;
                }
            }
        }
    }

    points_prevented - spark_lost + attack_points
}
