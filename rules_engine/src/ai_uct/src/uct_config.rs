#[derive(Debug, Clone)]
pub struct UctConfig {
    pub max_iterations: u32,
}

impl Default for UctConfig {
    fn default() -> Self {
        Self { max_iterations: 10_000 }
    }
}
