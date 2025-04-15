use std::hash::Hash;

#[derive(Debug, PartialEq, Eq)]
pub enum GameStatus<TPlayer: Eq> {
    /// Game is still ongoing, it is TPlayer's turn.
    InProgress { current_turn: TPlayer },
    /// Game has ended, TPlayer has won.
    Completed { winner: TPlayer },
}

/// A generic game state used by an AI algorithm.
///
/// Keeping the AI search algorithm implementation generic when possible is
/// useful for testing. We use a much simpler game with a known-optimal
/// strategy (the game of Nim) to sanity-check that the AI implementations are
/// doing broadly correct things.
pub trait GameStateNode {
    /// A game action to transition the game to a new state.
    type Action: Eq + Copy + Hash;

    /// A player in the game.
    type PlayerName: Eq + Copy;

    /// Create a copy of this search node to be mutated by selection algorithms.
    /// A basic implementation of this would be to simply call `.clone()`, but
    /// sometimes parts of the game state are only for display and are not
    /// relevant for selection algorithms.
    fn make_copy(&self) -> Self;

    /// Returns the status for the game, either the player whose turn it is or
    /// the player who won.
    fn status(&self) -> GameStatus<Self::PlayerName>;

    /// Returns the player whose turn it currently is, or an error if the game
    /// has ended.
    fn current_turn(&self) -> Self::PlayerName {
        match self.status() {
            GameStatus::InProgress { current_turn } => current_turn,
            GameStatus::Completed { .. } => panic!("Error: Game is over"),
        }
    }

    /// Returns an iterator over actions that the provided `player` can legally
    /// take in the current game state.
    ///
    /// Should an error if no actions are available.
    fn legal_actions<'a>(
        &'a self,
        player: Self::PlayerName,
    ) -> Box<dyn Iterator<Item = Self::Action> + 'a>;

    /// Apply the result of a given action to this game state.
    fn execute_action(&mut self, player: Self::PlayerName, action: Self::Action);
}
