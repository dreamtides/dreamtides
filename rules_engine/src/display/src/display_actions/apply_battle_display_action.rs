use action_data::battle_display_action::{BattleDisplayAction, CardBrowserType};
use core_data::display_types::StudioAnimation;
use core_data::identifiers::UserId;
use core_data::numerics::Energy;
use core_data::types::PlayerName;
use display_data::battle_view::DisplayPlayer;
use display_data::command::{Command, CommandSequence, PlayStudioAnimationCommand, StudioType};
use display_data::object_position::Position;
use state_provider::StateProvider;

use crate::core::response_builder::ResponseBuilder;
use crate::display_actions::display_state;

/// Modifies the display state of a battle and returns commands in response to
/// the action.
pub fn execute(
    provider: impl StateProvider + 'static,
    action: BattleDisplayAction,
    player: PlayerName,
    user_id: UserId,
) -> CommandSequence {
    let mut builder = ResponseBuilder::with_state_provider(player, user_id, provider, true);

    match action {
        BattleDisplayAction::BrowseCards(card_browser_type) => {
            browse_cards(card_browser_type, &mut builder);
        }
        BattleDisplayAction::CloseCardBrowser => close_card_browser(&builder),
        BattleDisplayAction::SetSelectedEnergyAdditionalCost(energy) => {
            set_selected_energy_additional_cost(&builder, energy);
        }
        BattleDisplayAction::OpenPanel(address) => {
            display_state::set_current_panel_address(&builder, Some(address));
        }
        BattleDisplayAction::CloseCurrentPanel => {
            display_state::set_current_panel_address(&builder, None);
        }
        BattleDisplayAction::ToggleStackVisibility => {
            toggle_stack_visibility(&builder);
        }
    }

    builder.commands()
}

fn browse_cards(card_browser: CardBrowserType, builder: &mut ResponseBuilder) {
    let source_position = match card_browser {
        CardBrowserType::UserDeck => Position::InDeck(DisplayPlayer::User),
        CardBrowserType::EnemyDeck => Position::InDeck(DisplayPlayer::Enemy),
        CardBrowserType::UserVoid => Position::InVoid(DisplayPlayer::User),
        CardBrowserType::EnemyVoid => Position::InVoid(DisplayPlayer::Enemy),
        CardBrowserType::UserStatus => {
            builder.push(Command::PlayStudioAnimation(PlayStudioAnimationCommand {
                studio_type: StudioType::UserIdentityCard,
                enter_animation: Some(StudioAnimation::new("IDL_ArmsFolded_Casual_Enter")),
                animation: StudioAnimation::new("IDL_ArmsFolded_Casual_Loop"),
                exit_animation: Some(StudioAnimation::new("IDL_ArmsFolded_Casual_Exit")),
            }));
            Position::InPlayerStatus(DisplayPlayer::User)
        }
        CardBrowserType::EnemyStatus => {
            builder.push(Command::PlayStudioAnimation(PlayStudioAnimationCommand {
                studio_type: StudioType::EnemyIdentityCard,
                enter_animation: Some(StudioAnimation::new("IDL_ArmsFolded_Casual_Enter")),
                animation: StudioAnimation::new("IDL_ArmsFolded_Casual_Loop"),
                exit_animation: Some(StudioAnimation::new("IDL_ArmsFolded_Casual_Exit")),
            }));
            Position::InPlayerStatus(DisplayPlayer::Enemy)
        }
    };

    display_state::set_card_browser_source(builder, Some(source_position));
}

fn close_card_browser(builder: &ResponseBuilder) {
    display_state::set_card_browser_source(builder, None);
}

fn set_selected_energy_additional_cost(builder: &ResponseBuilder, energy: Energy) {
    display_state::set_selected_energy_additional_cost(builder, Some(energy));
}

/// Returns whether a card browser is currently active and what source position
/// it's browsing.
pub fn current_browser_source(builder: &ResponseBuilder) -> Option<Position> {
    display_state::get_card_browser_source(builder)
}

/// Toggles the visibility of the stack.
fn toggle_stack_visibility(builder: &ResponseBuilder) {
    let current = display_state::is_stack_hidden(builder);
    display_state::set_stack_hidden(builder, !current);
}
