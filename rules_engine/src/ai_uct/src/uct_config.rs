#[derive(Debug, Clone)]
pub struct UctConfig {
    pub max_iterations: usize,
}

impl Default for UctConfig {
    fn default() -> Self {
        Self { max_iterations: 10_000 }
    }
}
