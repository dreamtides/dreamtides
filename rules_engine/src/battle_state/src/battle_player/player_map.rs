use core_data::types::PlayerName;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerMap<T> {
    pub one: T,
    pub two: T,
}

impl<T: Default> Default for PlayerMap<T> {
    fn default() -> Self {
        Self { one: T::default(), two: T::default() }
    }
}

impl<T> PlayerMap<T> {
    #[inline(always)]
    pub fn player(&self, player: PlayerName) -> &T {
        match player {
            PlayerName::One => &self.one,
            PlayerName::Two => &self.two,
        }
    }

    #[inline(always)]
    pub fn player_mut(&mut self, player: PlayerName) -> &mut T {
        match player {
            PlayerName::One => &mut self.one,
            PlayerName::Two => &mut self.two,
        }
    }

    #[inline(always)]
    pub fn insert(&mut self, player: PlayerName, value: T) {
        match player {
            PlayerName::One => self.one = value,
            PlayerName::Two => self.two = value,
        }
    }
}
