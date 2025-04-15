/// Operation mode for child scoring.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SelectionMode {
    /// Balance between trying new children and re-visiting existing children.
    Exploration,
    /// Select the best overall child without giving any weight to exploration.
    Best,
}

/// Trait for selecting which child node of the Monte Carlo search tree to
/// explore. The child which returns the highest score is selected. Inputs are
/// the number of visits to the current parent, number of visits to this child,
/// known reward value for this child, and [SelectionMode].
pub trait ChildScoreAlgorithm: Send {
    fn score(
        &self,
        parent_visits: f64,
        child_visits: f64,
        child_reward: f64,
        selection_mode: SelectionMode,
    ) -> f64;
}
