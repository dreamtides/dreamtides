/// High-level context for what operation the rules engine is currently
/// performing.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum RequestContext {
    UserRequest,
}
