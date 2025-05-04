use core_data::types::PlayerName;
use ui_components::component::Component;

#[derive(Clone)]
pub struct SetOpponentAgentPanel {
    pub user_player: PlayerName,
}

impl Component for SetOpponentAgentPanel {
    fn render(self) -> Option<impl Component> {
        todo!("")
    }
}
