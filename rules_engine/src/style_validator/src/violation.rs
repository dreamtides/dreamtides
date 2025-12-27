use std::path::PathBuf;

#[derive(Default, Clone)]
pub struct StyleViolation {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub kind: ViolationKind,
    pub path_str: String,
}

#[derive(Default, Clone, Copy)]
pub enum ViolationKind {
    #[default]
    TooManyQualifiers,
    TypeShouldNotBeQualified,
    EnumVariantTooManyQualifiers,
    DirectFunctionImport,
    CodeInModLibFile,
    PubUseStatement,
    CargoTomlDependencyOrder,
    InlineUseStatement,
}

impl ViolationKind {
    pub fn description(&self) -> &str {
        match self {
            ViolationKind::TooManyQualifiers => {
                "function call has too many qualifiers (should be 0 for same-file or 1 for cross-file)"
            }
            ViolationKind::TypeShouldNotBeQualified => {
                "type name should not be qualified (should have zero qualifiers)"
            }
            ViolationKind::EnumVariantTooManyQualifiers => {
                "enum variant has too many qualifiers (should have exactly one)"
            }
            ViolationKind::DirectFunctionImport => {
                "function imported directly in use statement (should use module::function_name at call site instead)"
            }
            ViolationKind::CodeInModLibFile => {
                "code added to mod.rs or lib.rs file (only mod declarations allowed)"
            }
            ViolationKind::PubUseStatement => {
                "pub use statement not permitted (all imports must come from their original file location)"
            }
            ViolationKind::CargoTomlDependencyOrder => {
                "Cargo.toml dependencies must be alphabetized: internal dependencies first, then external dependencies"
            }
            ViolationKind::InlineUseStatement => {
                "use statement placed inline within function body or other nested context (use statements must be at top of file)"
            }
        }
    }
}
