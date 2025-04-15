/// Helper which keeps track of an evaluator score and the action that produced
/// it.
pub struct ScoredAction<T> {
    score: i32,
    action: Option<T>,
}

impl<T> ScoredAction<T>
where
    T: Copy,
{
    pub fn new(score: i32) -> Self {
        Self { score, action: None }
    }

    pub fn has_action(&self) -> bool {
        self.action.is_some()
    }

    pub fn action(&self) -> T {
        self.action.expect("No action found for ScoredAction")
    }

    pub fn score(&self) -> i32 {
        self.score
    }

    /// Returns this ScoredAction, populating `action` as its action if there is
    /// not currently a saved action.
    pub fn with_fallback_action(self, action: T) -> Self {
        if self.action.is_none() {
            Self { action: Some(action), ..self }
        } else {
            self
        }
    }

    /// Insert this action & score if they are greater than the current score.
    pub fn insert_max(&mut self, action: T, score: i32) {
        if !self.has_action() || score > self.score {
            self.score = score;
            self.action = Some(action);
        }
    }

    /// Insert this action & score if they are lower than the current score.
    pub fn insert_min(&mut self, action: T, score: i32) {
        if !self.has_action() || score < self.score {
            self.score = score;
            self.action = Some(action);
        }
    }
}
