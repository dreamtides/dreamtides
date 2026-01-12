use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct GitState {
    pub branch: String,
    pub head: String,
    pub clean: bool,
}
