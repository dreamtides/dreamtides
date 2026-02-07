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
    CodeOrder,
    MissingDocCommentImport,
    InlineTestModule,
    TestsDirectoryInSrc,
    WorkspaceDependencyNotUsed,
    TestFileNamingConvention,
    SuperOrSelfImport,
    CodeSpacing,
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
            ViolationKind::CodeOrder => {
                "items not in correct order (should be: private constants, private statics, thread_local, public type aliases, public constants, public traits, public structs/enums, public functions, then private items)"
            }
            ViolationKind::MissingDocCommentImport => {
                "doc comment references type not imported in use statements"
            }
            ViolationKind::InlineTestModule => {
                "inline test module not permitted (place tests in the /tests/ directory instead)"
            }
            ViolationKind::TestsDirectoryInSrc => {
                "tests directory not permitted under src/ (place tests in rules_engine/tests/ instead)"
            }
            ViolationKind::WorkspaceDependencyNotUsed => {
                "dependency must use 'workspace = true' (versions should be specified in root Cargo.toml)"
            }
            ViolationKind::TestFileNamingConvention => {
                "test file must end in _tests.rs (not _test.rs)"
            }
            ViolationKind::SuperOrSelfImport => {
                "use super:: or self:: import not permitted (use crate:: instead)"
            }
            ViolationKind::CodeSpacing => {
                "incorrect spacing between code elements (should have exactly one blank line between items, except consecutive constants)"
            }
        }
    }
}
