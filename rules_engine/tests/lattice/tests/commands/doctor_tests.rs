//! Tests for the `lat doctor` command.

use std::process::ExitCode;

use lattice::cli::commands::doctor_command::doctor_types::{
    CheckCategory, CheckResult, CheckStatus, DoctorConfig, DoctorReport, DoctorSummary,
    EXIT_CODE_WARNINGS_ONLY, compute_exit_code,
};
use lattice::cli::maintenance_args::DoctorArgs;
use lattice::error::exit_codes;

fn default_args() -> DoctorArgs {
    DoctorArgs { fix: false, dry_run: false, deep: false, quiet: false }
}

// ============================================================================
// Exit Code Computation Tests
// ============================================================================

#[test]
fn compute_exit_code_returns_success_when_all_passed() {
    let checks = vec![
        CheckResult::passed(CheckCategory::Core, "test", "passed"),
        CheckResult::passed(CheckCategory::Index, "test2", "also passed"),
    ];
    let report = DoctorReport::new(checks);

    assert_eq!(
        compute_exit_code(&report),
        exit_codes::success(),
        "All passed checks should return exit code 0"
    );
}

#[test]
fn compute_exit_code_returns_success_when_only_info() {
    let checks = vec![
        CheckResult::passed(CheckCategory::Core, "test", "passed"),
        CheckResult::info(CheckCategory::Git, "info-check", "just info"),
    ];
    let report = DoctorReport::new(checks);

    assert_eq!(
        compute_exit_code(&report),
        exit_codes::success(),
        "Info-only (no warnings or errors) should return exit code 0"
    );
}

#[test]
fn compute_exit_code_returns_validation_error_when_errors_present() {
    let checks = vec![
        CheckResult::passed(CheckCategory::Core, "test", "passed"),
        CheckResult::error(CheckCategory::Core, "broken", "something broke"),
    ];
    let report = DoctorReport::new(checks);

    assert_eq!(
        compute_exit_code(&report),
        exit_codes::validation_error(),
        "Errors should return exit code 2"
    );
}

#[test]
fn compute_exit_code_returns_warnings_only_code_when_no_errors() {
    let checks = vec![
        CheckResult::passed(CheckCategory::Core, "test", "passed"),
        CheckResult::warning(CheckCategory::Claims, "stale", "stale claim found"),
    ];
    let report = DoctorReport::new(checks);

    assert_eq!(
        compute_exit_code(&report),
        ExitCode::from(EXIT_CODE_WARNINGS_ONLY),
        "Warnings without errors should return exit code 3"
    );
}

#[test]
fn compute_exit_code_prefers_errors_over_warnings() {
    let checks = vec![
        CheckResult::warning(CheckCategory::Claims, "warn1", "warning"),
        CheckResult::error(CheckCategory::Core, "err1", "error"),
        CheckResult::warning(CheckCategory::Config, "warn2", "another warning"),
    ];
    let report = DoctorReport::new(checks);

    assert_eq!(
        compute_exit_code(&report),
        exit_codes::validation_error(),
        "Errors should take precedence over warnings"
    );
}

// ============================================================================
// DoctorConfig From DoctorArgs Tests
// ============================================================================

#[test]
fn doctor_config_from_default_args() {
    let args = default_args();
    let config = DoctorConfig::from(&args);

    assert!(!config.fix);
    assert!(!config.dry_run);
    assert!(!config.deep);
    assert!(!config.quiet);
}

#[test]
fn doctor_config_preserves_all_flags() {
    let args = DoctorArgs { fix: true, dry_run: true, deep: true, quiet: true };
    let config = DoctorConfig::from(&args);

    assert!(config.fix);
    assert!(config.dry_run);
    assert!(config.deep);
    assert!(config.quiet);
}

// ============================================================================
// DoctorSummary Tests
// ============================================================================

#[test]
fn summary_starts_empty() {
    let summary = DoctorSummary::default();
    assert_eq!(summary.passed, 0);
    assert_eq!(summary.info, 0);
    assert_eq!(summary.warnings, 0);
    assert_eq!(summary.failed, 0);
}

#[test]
fn summary_add_increments_correct_counter() {
    let mut summary = DoctorSummary::default();

    summary.add(CheckStatus::Passed);
    assert_eq!(summary.passed, 1);

    summary.add(CheckStatus::Info);
    assert_eq!(summary.info, 1);

    summary.add(CheckStatus::Warning);
    assert_eq!(summary.warnings, 1);

    summary.add(CheckStatus::Error);
    assert_eq!(summary.failed, 1);
}

#[test]
fn summary_has_errors_is_true_when_failed_positive() {
    let mut summary = DoctorSummary::default();
    assert!(!summary.has_errors());

    summary.add(CheckStatus::Error);
    assert!(summary.has_errors());
}

#[test]
fn summary_has_warnings_is_true_when_warnings_positive() {
    let mut summary = DoctorSummary::default();
    assert!(!summary.has_warnings());

    summary.add(CheckStatus::Warning);
    assert!(summary.has_warnings());
}

// ============================================================================
// CheckResult Builder Tests
// ============================================================================

#[test]
fn check_result_passed_has_correct_status() {
    let result = CheckResult::passed(CheckCategory::Core, "test", "message");
    assert_eq!(result.status, CheckStatus::Passed);
    assert_eq!(result.category, CheckCategory::Core);
    assert_eq!(result.name, "test");
    assert_eq!(result.message, "message");
    assert!(result.details.is_empty());
    assert!(!result.fixable);
    assert!(result.fix_command.is_none());
}

#[test]
fn check_result_info_has_correct_status() {
    let result = CheckResult::info(CheckCategory::Git, "test", "info message");
    assert_eq!(result.status, CheckStatus::Info);
}

#[test]
fn check_result_warning_has_correct_status() {
    let result = CheckResult::warning(CheckCategory::Claims, "test", "warning message");
    assert_eq!(result.status, CheckStatus::Warning);
}

#[test]
fn check_result_error_has_correct_status() {
    let result = CheckResult::error(CheckCategory::Index, "test", "error message");
    assert_eq!(result.status, CheckStatus::Error);
}

#[test]
fn check_result_with_details_adds_details() {
    let result = CheckResult::warning(CheckCategory::Claims, "stale", "stale claims")
        .with_details(vec!["LABC01".to_string(), "LDEF02".to_string()]);

    assert_eq!(result.details.len(), 2);
    assert_eq!(result.details[0], "LABC01");
    assert_eq!(result.details[1], "LDEF02");
}

#[test]
fn check_result_with_fix_sets_fixable_and_command() {
    let result = CheckResult::error(CheckCategory::Index, "missing", "index missing")
        .with_fix("lat doctor --fix");

    assert!(result.fixable);
    assert_eq!(result.fix_command, Some("lat doctor --fix".to_string()));
}

// ============================================================================
// CheckStatus Tests
// ============================================================================

#[test]
fn check_status_is_failure_only_for_error() {
    assert!(!CheckStatus::Passed.is_failure());
    assert!(!CheckStatus::Info.is_failure());
    assert!(!CheckStatus::Warning.is_failure());
    assert!(CheckStatus::Error.is_failure());
}

#[test]
fn check_status_is_warning_only_for_warning() {
    assert!(!CheckStatus::Passed.is_warning());
    assert!(!CheckStatus::Info.is_warning());
    assert!(CheckStatus::Warning.is_warning());
    assert!(!CheckStatus::Error.is_warning());
}

#[test]
fn check_status_icons_are_correct() {
    assert_eq!(CheckStatus::Passed.icon(), "✓");
    assert_eq!(CheckStatus::Info.icon(), "ℹ");
    assert_eq!(CheckStatus::Warning.icon(), "⚠");
    assert_eq!(CheckStatus::Error.icon(), "✖");
}

// ============================================================================
// CheckCategory Tests
// ============================================================================

#[test]
fn check_category_display_names() {
    assert_eq!(CheckCategory::Core.display_name(), "CORE SYSTEM");
    assert_eq!(CheckCategory::Index.display_name(), "INDEX INTEGRITY");
    assert_eq!(CheckCategory::Git.display_name(), "GIT INTEGRATION");
    assert_eq!(CheckCategory::Config.display_name(), "CONFIGURATION");
    assert_eq!(CheckCategory::Claims.display_name(), "CLAIMS");
    assert_eq!(CheckCategory::Skills.display_name(), "SKILLS");
}

#[test]
fn check_category_all_returns_all_categories_in_order() {
    let all = CheckCategory::all();
    assert_eq!(all.len(), 6);
    assert_eq!(all[0], CheckCategory::Core);
    assert_eq!(all[1], CheckCategory::Index);
    assert_eq!(all[2], CheckCategory::Git);
    assert_eq!(all[3], CheckCategory::Config);
    assert_eq!(all[4], CheckCategory::Claims);
    assert_eq!(all[5], CheckCategory::Skills);
}

// ============================================================================
// DoctorReport Tests
// ============================================================================

#[test]
fn doctor_report_new_computes_summary() {
    let checks = vec![
        CheckResult::passed(CheckCategory::Core, "p1", "passed"),
        CheckResult::passed(CheckCategory::Core, "p2", "passed"),
        CheckResult::info(CheckCategory::Git, "i1", "info"),
        CheckResult::warning(CheckCategory::Claims, "w1", "warning"),
        CheckResult::error(CheckCategory::Index, "e1", "error"),
    ];
    let report = DoctorReport::new(checks);

    assert_eq!(report.summary.passed, 2);
    assert_eq!(report.summary.info, 1);
    assert_eq!(report.summary.warnings, 1);
    assert_eq!(report.summary.failed, 1);
}

#[test]
fn doctor_report_checks_for_category_filters_correctly() {
    let checks = vec![
        CheckResult::passed(CheckCategory::Core, "c1", "core check 1"),
        CheckResult::passed(CheckCategory::Core, "c2", "core check 2"),
        CheckResult::passed(CheckCategory::Git, "g1", "git check"),
    ];
    let report = DoctorReport::new(checks);

    let core_checks = report.checks_for_category(CheckCategory::Core);
    assert_eq!(core_checks.len(), 2);

    let git_checks = report.checks_for_category(CheckCategory::Git);
    assert_eq!(git_checks.len(), 1);

    let config_checks = report.checks_for_category(CheckCategory::Config);
    assert!(config_checks.is_empty());
}

#[test]
fn doctor_report_issues_returns_only_warnings_and_errors() {
    let checks = vec![
        CheckResult::passed(CheckCategory::Core, "p1", "passed"),
        CheckResult::info(CheckCategory::Git, "i1", "info"),
        CheckResult::warning(CheckCategory::Claims, "w1", "warning"),
        CheckResult::error(CheckCategory::Index, "e1", "error"),
    ];
    let report = DoctorReport::new(checks);

    let issues = report.issues();
    assert_eq!(issues.len(), 2);
    assert!(issues.iter().any(|i| i.status == CheckStatus::Warning));
    assert!(issues.iter().any(|i| i.status == CheckStatus::Error));
}

#[test]
fn doctor_report_includes_version() {
    let report = DoctorReport::new(vec![]);
    assert!(!report.version.is_empty(), "Report should include version string");
}

// ============================================================================
// Core System Check Integration Tests
// ============================================================================

mod core_checks {
    use std::fs;

    use lattice::cli::commands::doctor_command::doctor_checks;
    use lattice::cli::commands::doctor_command::doctor_types::{
        CheckCategory, CheckStatus, DoctorConfig,
    };
    use lattice::test::test_environment::TestEnv;

    fn find_check<'a>(
        results: &'a [lattice::cli::commands::doctor_command::doctor_types::CheckResult],
        category: CheckCategory,
        name: &str,
    ) -> Option<&'a lattice::cli::commands::doctor_command::doctor_types::CheckResult> {
        results.iter().find(|r| r.category == category && r.name == name)
    }

    #[test]
    fn installation_check_passes_with_lattice_directory() {
        let env = TestEnv::new();
        let (_temp, context) = env.into_parts();
        let config = DoctorConfig::default();

        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Core, "Installation")
            .expect("Installation check should be present");
        assert_eq!(
            check.status,
            CheckStatus::Passed,
            "Installation check should pass when .lattice/ exists"
        );
        assert!(check.message.contains(".lattice/"), "Message should mention .lattice/ directory");
    }

    #[test]
    fn installation_check_fails_without_lattice_directory() {
        let env = TestEnv::new();
        let (_temp, context) = env.into_parts();

        fs::remove_dir_all(context.repo_root.join(".lattice")).expect("Remove .lattice dir");

        let config = DoctorConfig::default();
        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Core, "Installation")
            .expect("Installation check should be present");
        assert_eq!(
            check.status,
            CheckStatus::Error,
            "Installation check should fail when .lattice/ is missing"
        );
    }

    #[test]
    fn index_check_passes_and_reports_document_count() {
        let env = TestEnv::new();
        env.create_dir("api/tasks");
        env.create_document("api/api.md", "LAPIXX", "api", "API root");
        let (_temp, context) = env.into_parts();

        lattice::index::document_queries::insert(
            &context.conn,
            &lattice::index::document_types::InsertDocument {
                id: "LAPIXX".to_string(),
                parent_id: None,
                path: "api/api.md".to_string(),
                name: "api".to_string(),
                description: "API root".to_string(),
                task_type: None,
                is_closed: false,
                priority: None,
                created_at: None,
                updated_at: None,
                closed_at: None,
                body_hash: "hash123".to_string(),
                content_length: 100,
                is_root: true,
                in_tasks_dir: false,
                in_docs_dir: false,
                skill: false,
            },
        )
        .expect("Insert document");

        let config = DoctorConfig::default();
        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Core, "Index Database")
            .expect("Index check should be present");
        assert_eq!(check.status, CheckStatus::Passed, "Index check should pass");
        assert!(
            check.message.contains("1 documents"),
            "Message should include document count: {}",
            check.message
        );
    }

    #[test]
    fn index_check_fails_without_index_file() {
        let env = TestEnv::new();
        let (_temp, context) = env.into_parts();

        fs::remove_file(context.repo_root.join(".lattice/index.sqlite"))
            .expect("Remove index file");

        let config = DoctorConfig::default();
        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Core, "Index Database")
            .expect("Index check should be present");
        assert_eq!(
            check.status,
            CheckStatus::Error,
            "Index check should fail when index.sqlite is missing"
        );
        assert!(check.fixable, "Missing index should be fixable");
    }

    #[test]
    fn schema_version_check_passes_with_current_version() {
        let env = TestEnv::new();
        let (_temp, context) = env.into_parts();
        let config = DoctorConfig::default();

        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Core, "Schema Version")
            .expect("Schema version check should be present");
        assert_eq!(
            check.status,
            CheckStatus::Passed,
            "Schema version check should pass with current version"
        );
        assert!(
            check.message.contains("current"),
            "Message should indicate version is current: {}",
            check.message
        );
    }

    #[test]
    fn schema_version_check_warns_on_mismatch() {
        let env = TestEnv::new();
        let (_temp, context) = env.into_parts();

        context
            .conn
            .execute("UPDATE index_metadata SET schema_version = 999 WHERE id = 1", [])
            .expect("Set outdated schema version");

        let config = DoctorConfig::default();
        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Core, "Schema Version")
            .expect("Schema version check should be present");
        assert_eq!(
            check.status,
            CheckStatus::Warning,
            "Schema version check should warn on mismatch"
        );
        assert!(check.fixable, "Schema mismatch should be fixable");
        assert!(
            check.message.contains("999"),
            "Message should include old version: {}",
            check.message
        );
    }

    #[test]
    fn wal_health_check_passes_with_no_wal_files() {
        let env = TestEnv::new();
        let (_temp, context) = env.into_parts();
        let config = DoctorConfig::default();

        let wal_path = context.repo_root.join(".lattice/index.sqlite-wal");
        let shm_path = context.repo_root.join(".lattice/index.sqlite-shm");
        if wal_path.exists() {
            fs::remove_file(&wal_path).ok();
        }
        if shm_path.exists() {
            fs::remove_file(&shm_path).ok();
        }

        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Core, "WAL Health")
            .expect("WAL health check should be present");
        assert_eq!(check.status, CheckStatus::Passed, "WAL health should pass with no WAL files");
        assert!(
            check.message.contains("clean state"),
            "Message should indicate clean state: {}",
            check.message
        );
    }

    #[test]
    fn wal_health_check_detects_orphan_wal_without_shm() {
        let env = TestEnv::new();
        let (_temp, context) = env.into_parts();

        let wal_path = context.repo_root.join(".lattice/index.sqlite-wal");
        let shm_path = context.repo_root.join(".lattice/index.sqlite-shm");

        fs::write(&wal_path, vec![0u8; 4096]).expect("Create WAL file");
        if shm_path.exists() {
            fs::remove_file(&shm_path).expect("Remove SHM file");
        }

        let config = DoctorConfig::default();
        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Core, "WAL Health")
            .expect("WAL health check should be present");
        assert_eq!(check.status, CheckStatus::Error, "WAL health should fail with orphan WAL file");
        assert!(check.fixable, "WAL corruption should be fixable");
        assert!(
            check.details.iter().any(|d| d.contains("without SHM")),
            "Details should mention missing SHM: {:?}",
            check.details
        );
    }

    #[test]
    fn wal_health_check_detects_empty_wal_file() {
        let env = TestEnv::new();
        let (_temp, context) = env.into_parts();

        let wal_path = context.repo_root.join(".lattice/index.sqlite-wal");
        let shm_path = context.repo_root.join(".lattice/index.sqlite-shm");

        fs::write(&wal_path, "").expect("Create empty WAL file");
        fs::write(&shm_path, vec![0u8; 32768]).expect("Create SHM file");

        let config = DoctorConfig::default();
        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Core, "WAL Health")
            .expect("WAL health check should be present");
        assert_eq!(check.status, CheckStatus::Error, "WAL health should fail with empty WAL file");
        assert!(
            check.details.iter().any(|d| d.contains("empty")),
            "Details should mention empty file: {:?}",
            check.details
        );
    }

    #[test]
    fn all_core_checks_are_present() {
        let env = TestEnv::new();
        let (_temp, context) = env.into_parts();
        let config = DoctorConfig::default();

        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let core_checks: Vec<_> =
            results.iter().filter(|r| r.category == CheckCategory::Core).collect();

        assert!(
            core_checks.iter().any(|c| c.name == "Installation"),
            "Should have Installation check"
        );
        assert!(
            core_checks.iter().any(|c| c.name == "Index Database"),
            "Should have Index Database check"
        );
        assert!(
            core_checks.iter().any(|c| c.name == "Schema Version"),
            "Should have Schema Version check"
        );
        assert!(core_checks.iter().any(|c| c.name == "WAL Health"), "Should have WAL Health check");
        assert_eq!(core_checks.len(), 4, "Should have exactly 4 core checks");
    }
}

mod index_checks {
    use lattice::cli::commands::doctor_command::doctor_checks;
    use lattice::cli::commands::doctor_command::doctor_types::{
        CheckCategory, CheckStatus, DoctorConfig,
    };
    use lattice::index::document_queries;
    use lattice::index::document_types::InsertDocument;
    use lattice::test::test_environment::TestEnv;

    fn find_check<'a>(
        results: &'a [lattice::cli::commands::doctor_command::doctor_types::CheckResult],
        category: CheckCategory,
        name: &str,
    ) -> Option<&'a lattice::cli::commands::doctor_command::doctor_types::CheckResult> {
        results.iter().find(|r| r.category == category && r.name == name)
    }

    #[test]
    fn filesystem_sync_passes_when_all_indexed_documents_exist() {
        let env = TestEnv::new();
        env.create_dir("api");
        env.create_document("api/api.md", "LAPIXX", "api", "API root");

        let (_temp, context) = env.into_parts();

        document_queries::insert(&context.conn, &InsertDocument {
            id: "LAPIXX".to_string(),
            parent_id: None,
            path: "api/api.md".to_string(),
            name: "api".to_string(),
            description: "API root".to_string(),
            task_type: None,
            is_closed: false,
            priority: None,
            created_at: None,
            updated_at: None,
            closed_at: None,
            body_hash: "hash123".to_string(),
            content_length: 100,
            is_root: true,
            in_tasks_dir: false,
            in_docs_dir: false,
            skill: false,
        })
        .expect("Insert document");

        let config = DoctorConfig::default();
        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Index, "Filesystem Sync")
            .expect("Filesystem Sync check should be present");
        assert_eq!(check.status, CheckStatus::Passed, "Should pass when all docs exist on disk");
    }

    #[test]
    fn filesystem_sync_fails_when_indexed_document_missing_from_disk() {
        let env = TestEnv::new();
        let (_temp, context) = env.into_parts();

        document_queries::insert(&context.conn, &InsertDocument {
            id: "LMISSING".to_string(),
            parent_id: None,
            path: "missing/doc.md".to_string(),
            name: "doc".to_string(),
            description: "Missing doc".to_string(),
            task_type: None,
            is_closed: false,
            priority: None,
            created_at: None,
            updated_at: None,
            closed_at: None,
            body_hash: "hash123".to_string(),
            content_length: 100,
            is_root: false,
            in_tasks_dir: false,
            in_docs_dir: false,
            skill: false,
        })
        .expect("Insert document");

        let config = DoctorConfig::default();
        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Index, "Filesystem Sync")
            .expect("Filesystem Sync check should be present");
        assert_eq!(
            check.status,
            CheckStatus::Error,
            "Should fail when indexed doc missing from disk"
        );
        assert!(check.fixable, "Missing file issue should be fixable");
        assert!(
            check.details.iter().any(|d| d.contains("LMISSING")),
            "Details should mention missing doc ID"
        );
    }

    #[test]
    fn coverage_passes_when_all_documents_indexed() {
        let env = TestEnv::new();
        env.create_dir("api");
        env.create_document("api/api.md", "LAPIXX", "api", "API root");
        env.fake_git().track_files(["api/api.md"]);

        let (_temp, context) = env.into_parts();

        document_queries::insert(&context.conn, &InsertDocument {
            id: "LAPIXX".to_string(),
            parent_id: None,
            path: "api/api.md".to_string(),
            name: "api".to_string(),
            description: "API root".to_string(),
            task_type: None,
            is_closed: false,
            priority: None,
            created_at: None,
            updated_at: None,
            closed_at: None,
            body_hash: "hash123".to_string(),
            content_length: 100,
            is_root: true,
            in_tasks_dir: false,
            in_docs_dir: false,
            skill: false,
        })
        .expect("Insert document");

        let config = DoctorConfig::default();
        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Index, "Coverage")
            .expect("Coverage check should be present");
        assert_eq!(check.status, CheckStatus::Passed, "Should pass when all docs are indexed");
    }

    #[test]
    fn coverage_warns_when_document_not_indexed() {
        let env = TestEnv::new();
        env.create_dir("api");
        env.create_document("api/api.md", "LAPIXX", "api", "API root");
        env.fake_git().track_files(["api/api.md"]);

        let (_temp, context) = env.into_parts();

        let config = DoctorConfig::default();
        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Index, "Coverage")
            .expect("Coverage check should be present");
        assert_eq!(check.status, CheckStatus::Warning, "Should warn when doc not indexed");
        assert!(check.fixable, "Unindexed document issue should be fixable");
        assert!(
            check.details.iter().any(|d| d.contains("LAPIXX")),
            "Details should mention unindexed doc ID"
        );
    }

    #[test]
    fn duplicate_ids_passes_when_no_duplicates() {
        let env = TestEnv::new();
        let (_temp, context) = env.into_parts();

        document_queries::insert(&context.conn, &InsertDocument {
            id: "LONE01".to_string(),
            parent_id: None,
            path: "doc1.md".to_string(),
            name: "doc1".to_string(),
            description: "Doc 1".to_string(),
            task_type: None,
            is_closed: false,
            priority: None,
            created_at: None,
            updated_at: None,
            closed_at: None,
            body_hash: "hash1".to_string(),
            content_length: 100,
            is_root: false,
            in_tasks_dir: false,
            in_docs_dir: false,
            skill: false,
        })
        .expect("Insert document");

        let config = DoctorConfig::default();
        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Index, "No Duplicates")
            .expect("No Duplicates check should be present");
        assert_eq!(check.status, CheckStatus::Passed, "Should pass with no duplicates");
    }

    #[test]
    fn closed_state_passes_when_consistent() {
        let env = TestEnv::new();
        env.create_dir("api/tasks/.closed");
        env.create_document("api/tasks/.closed/done.md", "LDONE1", "done", "Done task");

        let (_temp, context) = env.into_parts();

        document_queries::insert(&context.conn, &InsertDocument {
            id: "LDONE1".to_string(),
            parent_id: None,
            path: "api/tasks/.closed/done.md".to_string(),
            name: "done".to_string(),
            description: "Done task".to_string(),
            task_type: Some(lattice::document::frontmatter_schema::TaskType::Task),
            is_closed: true,
            priority: None,
            created_at: None,
            updated_at: None,
            closed_at: None,
            body_hash: "hash1".to_string(),
            content_length: 100,
            is_root: false,
            in_tasks_dir: true,
            in_docs_dir: false,
            skill: false,
        })
        .expect("Insert document");

        let config = DoctorConfig::default();
        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Index, "Closed State")
            .expect("Closed State check should be present");
        assert_eq!(check.status, CheckStatus::Passed, "Should pass when is_closed matches path");
    }

    #[test]
    fn closed_state_warns_when_flag_inconsistent_with_path() {
        let env = TestEnv::new();
        env.create_dir("api/tasks");
        env.create_document("api/tasks/open.md", "LOPEN1", "open", "Open task");

        let (_temp, context) = env.into_parts();

        document_queries::insert(&context.conn, &InsertDocument {
            id: "LOPEN1".to_string(),
            parent_id: None,
            path: "api/tasks/open.md".to_string(),
            name: "open".to_string(),
            description: "Open task".to_string(),
            task_type: Some(lattice::document::frontmatter_schema::TaskType::Task),
            is_closed: true,
            priority: None,
            created_at: None,
            updated_at: None,
            closed_at: None,
            body_hash: "hash1".to_string(),
            content_length: 100,
            is_root: false,
            in_tasks_dir: true,
            in_docs_dir: false,
            skill: false,
        })
        .expect("Insert document");

        let config = DoctorConfig::default();
        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Index, "Closed State")
            .expect("Closed State check should be present");
        assert_eq!(
            check.status,
            CheckStatus::Warning,
            "Should warn when is_closed=true but path not in .closed/"
        );
        assert!(check.fixable, "Closed state mismatch should be fixable");
    }

    #[test]
    fn root_state_passes_when_consistent() {
        let env = TestEnv::new();
        env.create_dir("api");
        env.create_document("api/api.md", "LAPIXX", "api", "API root");

        let (_temp, context) = env.into_parts();

        document_queries::insert(&context.conn, &InsertDocument {
            id: "LAPIXX".to_string(),
            parent_id: None,
            path: "api/api.md".to_string(),
            name: "api".to_string(),
            description: "API root".to_string(),
            task_type: None,
            is_closed: false,
            priority: None,
            created_at: None,
            updated_at: None,
            closed_at: None,
            body_hash: "hash123".to_string(),
            content_length: 100,
            is_root: true,
            in_tasks_dir: false,
            in_docs_dir: false,
            skill: false,
        })
        .expect("Insert document");

        let config = DoctorConfig::default();
        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Index, "Root State")
            .expect("Root State check should be present");
        assert_eq!(check.status, CheckStatus::Passed, "Should pass when is_root matches path");
    }

    #[test]
    fn root_state_warns_when_flag_inconsistent() {
        let env = TestEnv::new();
        env.create_dir("api");
        env.create_document("api/other.md", "LOTHER", "other", "Not a root");

        let (_temp, context) = env.into_parts();

        document_queries::insert(&context.conn, &InsertDocument {
            id: "LOTHER".to_string(),
            parent_id: None,
            path: "api/other.md".to_string(),
            name: "other".to_string(),
            description: "Not a root".to_string(),
            task_type: None,
            is_closed: false,
            priority: None,
            created_at: None,
            updated_at: None,
            closed_at: None,
            body_hash: "hash123".to_string(),
            content_length: 100,
            is_root: true,
            in_tasks_dir: false,
            in_docs_dir: false,
            skill: false,
        })
        .expect("Insert document");

        let config = DoctorConfig::default();
        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Index, "Root State")
            .expect("Root State check should be present");
        assert_eq!(
            check.status,
            CheckStatus::Warning,
            "Should warn when is_root=true but path doesn't match directory"
        );
        assert!(check.fixable, "Root state mismatch should be fixable");
    }

    #[test]
    fn parent_consistency_passes_when_all_parents_exist() {
        let env = TestEnv::new();
        env.create_dir("api/docs");
        env.create_document("api/api.md", "LPARENT", "api", "Parent");
        env.create_document("api/docs/child.md", "LCHILD1", "child", "Child");

        let (_temp, context) = env.into_parts();

        document_queries::insert(&context.conn, &InsertDocument {
            id: "LPARENT".to_string(),
            parent_id: None,
            path: "api/api.md".to_string(),
            name: "api".to_string(),
            description: "Parent".to_string(),
            task_type: None,
            is_closed: false,
            priority: None,
            created_at: None,
            updated_at: None,
            closed_at: None,
            body_hash: "hash1".to_string(),
            content_length: 100,
            is_root: true,
            in_tasks_dir: false,
            in_docs_dir: false,
            skill: false,
        })
        .expect("Insert parent");

        document_queries::insert(&context.conn, &InsertDocument {
            id: "LCHILD1".to_string(),
            parent_id: Some("LPARENT".to_string()),
            path: "api/docs/child.md".to_string(),
            name: "child".to_string(),
            description: "Child".to_string(),
            task_type: None,
            is_closed: false,
            priority: None,
            created_at: None,
            updated_at: None,
            closed_at: None,
            body_hash: "hash2".to_string(),
            content_length: 100,
            is_root: false,
            in_tasks_dir: false,
            in_docs_dir: true,
            skill: false,
        })
        .expect("Insert child");

        let config = DoctorConfig::default();
        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Index, "Parent Consistency")
            .expect("Parent Consistency check should be present");
        assert_eq!(check.status, CheckStatus::Passed, "Should pass when all parents exist");
    }

    #[test]
    fn parent_consistency_warns_when_parent_missing() {
        let env = TestEnv::new();
        env.create_dir("api/docs");
        env.create_document("api/docs/orphan.md", "LORPHAN", "orphan", "Orphan");

        let (_temp, context) = env.into_parts();

        document_queries::insert(&context.conn, &InsertDocument {
            id: "LORPHAN".to_string(),
            parent_id: Some("LMISSING".to_string()),
            path: "api/docs/orphan.md".to_string(),
            name: "orphan".to_string(),
            description: "Orphan".to_string(),
            task_type: None,
            is_closed: false,
            priority: None,
            created_at: None,
            updated_at: None,
            closed_at: None,
            body_hash: "hash1".to_string(),
            content_length: 100,
            is_root: false,
            in_tasks_dir: false,
            in_docs_dir: true,
            skill: false,
        })
        .expect("Insert orphan");

        let config = DoctorConfig::default();
        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Index, "Parent Consistency")
            .expect("Parent Consistency check should be present");
        assert_eq!(
            check.status,
            CheckStatus::Warning,
            "Should warn when parent_id references non-existent doc"
        );
        assert!(
            check.details.iter().any(|d| d.contains("LORPHAN") && d.contains("LMISSING")),
            "Details should mention orphan and missing parent IDs"
        );
    }

    #[test]
    fn all_index_checks_are_present() {
        let env = TestEnv::new();
        let (_temp, context) = env.into_parts();
        let config = DoctorConfig::default();

        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let index_checks: Vec<_> =
            results.iter().filter(|r| r.category == CheckCategory::Index).collect();

        assert!(
            index_checks.iter().any(|c| c.name == "Filesystem Sync"),
            "Should have Filesystem Sync check"
        );
        assert!(index_checks.iter().any(|c| c.name == "Coverage"), "Should have Coverage check");
        assert!(
            index_checks.iter().any(|c| c.name == "No Duplicates"),
            "Should have No Duplicates check"
        );
        assert!(
            index_checks.iter().any(|c| c.name == "Closed State"),
            "Should have Closed State check"
        );
        assert!(
            index_checks.iter().any(|c| c.name == "Root State"),
            "Should have Root State check"
        );
        assert!(
            index_checks.iter().any(|c| c.name == "Parent Consistency"),
            "Should have Parent Consistency check"
        );
        assert_eq!(index_checks.len(), 6, "Should have exactly 6 index integrity checks");
    }
}

mod git_checks {
    use lattice::cli::commands::doctor_command::doctor_checks;
    use lattice::cli::commands::doctor_command::doctor_types::{
        CheckCategory, CheckStatus, DoctorConfig,
    };
    use lattice::test::test_environment::TestEnv;

    fn find_check<'a>(
        results: &'a [lattice::cli::commands::doctor_command::doctor_types::CheckResult],
        category: CheckCategory,
        name: &str,
    ) -> Option<&'a lattice::cli::commands::doctor_command::doctor_types::CheckResult> {
        results.iter().find(|r| r.category == category && r.name == name)
    }

    #[test]
    fn repository_check_passes_for_valid_git_repo() {
        let env = TestEnv::new();
        // Add a commit so rev_parse("HEAD") works
        env.fake_git().add_commit("abc123", "Initial commit", vec![]);
        let (_temp, context) = env.into_parts();
        let config = DoctorConfig::default();

        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Git, "Repository")
            .expect("Repository check should be present");
        assert_eq!(
            check.status,
            CheckStatus::Passed,
            "Repository check should pass for valid git repo"
        );
        assert!(
            check.message.contains("Valid"),
            "Message should indicate valid repository: {}",
            check.message
        );
    }

    #[test]
    fn repository_check_fails_when_rev_parse_fails() {
        use lattice::test::fake_git::FailingOperation;

        let env = TestEnv::new();
        // Inject failure for rev_parse to simulate invalid git repository
        env.fake_git().inject_failure(FailingOperation::RevParse, "not a git repository");
        let (_temp, context) = env.into_parts();
        let config = DoctorConfig::default();

        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Git, "Repository")
            .expect("Repository check should be present");
        assert_eq!(
            check.status,
            CheckStatus::Error,
            "Repository check should fail when HEAD cannot be resolved"
        );
    }

    #[test]
    fn configuration_check_passes_with_standard_repo() {
        let env = TestEnv::new();
        env.fake_git().add_commit("abc123", "Initial commit", vec![]);
        let (_temp, context) = env.into_parts();
        let config = DoctorConfig::default();

        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Git, "Configuration")
            .expect("Configuration check should be present");
        assert_eq!(
            check.status,
            CheckStatus::Passed,
            "Configuration check should pass for standard repo"
        );
        assert!(
            check.message.contains("Standard") || check.message.contains("no edge cases"),
            "Message should indicate standard config: {}",
            check.message
        );
    }

    #[test]
    fn working_tree_check_passes_with_clean_state() {
        let env = TestEnv::new();
        env.fake_git().add_commit("abc123", "Initial commit", vec![]);
        let (_temp, context) = env.into_parts();
        let config = DoctorConfig::default();

        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Git, "Working Tree")
            .expect("Working Tree check should be present");
        assert_eq!(
            check.status,
            CheckStatus::Passed,
            "Working Tree check should pass with clean state"
        );
        assert!(
            check.message.contains("Clean") || check.message.contains("no in-progress"),
            "Message should indicate clean state: {}",
            check.message
        );
    }

    #[test]
    fn head_state_check_passes_when_on_branch() {
        let env = TestEnv::new();
        env.fake_git().add_commit("abc123", "Initial commit", vec![]);
        // FakeGit defaults to being on 'main' branch
        let (_temp, context) = env.into_parts();
        let config = DoctorConfig::default();

        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Git, "HEAD State")
            .expect("HEAD State check should be present");
        assert_eq!(
            check.status,
            CheckStatus::Passed,
            "HEAD State check should pass when on a branch"
        );
        assert!(
            check.message.contains("branch"),
            "Message should mention branch: {}",
            check.message
        );
    }

    #[test]
    fn head_state_check_reports_detached_head() {
        let env = TestEnv::new();
        env.fake_git().add_commit("abc123def456", "Initial commit", vec![]);
        env.fake_git().detach_head("abc123def456");
        let (_temp, context) = env.into_parts();
        let config = DoctorConfig::default();

        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Git, "HEAD State")
            .expect("HEAD State check should be present");
        assert_eq!(
            check.status,
            CheckStatus::Info,
            "HEAD State check should be Info for detached HEAD"
        );
        assert!(
            check.message.to_lowercase().contains("detached"),
            "Message should mention detached: {}",
            check.message
        );
    }

    #[test]
    fn working_tree_check_warns_on_in_progress_merge() {
        use std::fs;

        let env = TestEnv::new();
        env.fake_git().add_commit("abc123", "Initial commit", vec![]);

        // Create MERGE_HEAD file to simulate in-progress merge
        let merge_head_path = env.repo_root().join(".git").join("MERGE_HEAD");
        fs::write(&merge_head_path, "deadbeef1234567890").expect("Create MERGE_HEAD");

        let (_temp, context) = env.into_parts();
        let config = DoctorConfig::default();

        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Git, "Working Tree")
            .expect("Working Tree check should be present");
        assert_eq!(
            check.status,
            CheckStatus::Warning,
            "Working Tree check should warn when merge is in progress"
        );
        assert!(
            check.message.to_lowercase().contains("merge"),
            "Message should mention merge: {}",
            check.message
        );
    }

    #[test]
    fn all_git_checks_are_present() {
        let env = TestEnv::new();
        env.fake_git().add_commit("abc123", "Initial commit", vec![]);
        let (_temp, context) = env.into_parts();
        let config = DoctorConfig::default();

        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let git_checks: Vec<_> =
            results.iter().filter(|r| r.category == CheckCategory::Git).collect();

        assert!(git_checks.iter().any(|c| c.name == "Repository"), "Should have Repository check");
        assert!(
            git_checks.iter().any(|c| c.name == "Configuration"),
            "Should have Configuration check"
        );
        assert!(
            git_checks.iter().any(|c| c.name == "Working Tree"),
            "Should have Working Tree check"
        );
        assert!(git_checks.iter().any(|c| c.name == "HEAD State"), "Should have HEAD State check");
        assert_eq!(git_checks.len(), 4, "Should have exactly 4 git integration checks");
    }
}

mod config_checks {
    use lattice::cli::commands::doctor_command::doctor_checks;
    use lattice::cli::commands::doctor_command::doctor_types::{
        CheckCategory, CheckStatus, DoctorConfig,
    };
    use lattice::test::test_environment::TestEnv;

    fn find_check<'a>(
        results: &'a [lattice::cli::commands::doctor_command::doctor_types::CheckResult],
        category: CheckCategory,
        name: &str,
    ) -> Option<&'a lattice::cli::commands::doctor_command::doctor_types::CheckResult> {
        results.iter().find(|r| r.category == category && r.name == name)
    }

    #[test]
    fn user_config_check_is_present() {
        // Note: This test is environment-dependent because it reads the real
        // ~/.lattice.toml file. The check should be Info (file not found) or
        // Passed (valid file found).
        let env = TestEnv::new();
        let (_temp, context) = env.into_parts();
        let config = DoctorConfig::default();

        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Config, "User Config")
            .expect("User Config check should be present");
        assert!(
            check.status == CheckStatus::Info || check.status == CheckStatus::Passed,
            "User Config should be Info (no file) or Passed (valid file): {:?}",
            check.status
        );
    }

    #[test]
    fn repo_config_check_passes_when_file_not_present() {
        let env = TestEnv::new();
        let (_temp, context) = env.into_parts();
        let config = DoctorConfig::default();

        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Config, "Repo Config")
            .expect("Repo Config check should be present");
        assert_eq!(
            check.status,
            CheckStatus::Info,
            "Repo Config should be Info when file not present"
        );
        assert!(
            check.message.contains("defaults") || check.message.contains("No"),
            "Message should indicate using defaults: {}",
            check.message
        );
    }

    #[test]
    fn repo_config_check_passes_when_file_is_valid() {
        let env = TestEnv::new();
        env.write_file(
            ".lattice/config.toml",
            "[format]\nline_width = 100\n[logging]\nlevel = \"warn\"\n",
        );
        let (_temp, context) = env.into_parts();
        let config = DoctorConfig::default();

        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Config, "Repo Config")
            .expect("Repo Config check should be present");
        assert_eq!(check.status, CheckStatus::Passed, "Repo Config should pass for valid TOML");
        assert!(
            check.message.contains("valid"),
            "Message should indicate valid config: {}",
            check.message
        );
    }

    #[test]
    fn repo_config_check_warns_on_parse_error() {
        let env = TestEnv::new();
        env.write_file(".lattice/config.toml", "this is not valid toml {{{{");
        let (_temp, context) = env.into_parts();
        let config = DoctorConfig::default();

        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Config, "Repo Config")
            .expect("Repo Config check should be present");
        assert_eq!(check.status, CheckStatus::Warning, "Repo Config should warn on parse error");
        assert!(
            check.message.contains("parse error") || check.message.contains("error"),
            "Message should indicate parse error: {}",
            check.message
        );
    }

    #[test]
    fn client_id_check_passes_when_assigned() {
        let env = TestEnv::new();
        let (_temp, context) = env.into_parts();

        // Set a client ID for this repository
        context.client_id_store.set(&context.repo_root, "DTX").expect("Set client ID");

        let config = DoctorConfig::default();
        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Config, "Client ID")
            .expect("Client ID check should be present");
        assert_eq!(check.status, CheckStatus::Passed, "Client ID should pass when assigned");
        assert!(
            check.message.contains("Assigned") || check.message.contains("DTX"),
            "Message should show assigned client ID: {}",
            check.message
        );
    }

    #[test]
    fn client_id_check_warns_when_not_assigned() {
        use lattice::git::client_config::FakeClientIdStore;

        let env = TestEnv::new();
        let (_temp, mut context) = env.into_parts();

        // Replace with a FakeClientIdStore that has no client ID set
        context.client_id_store = Box::new(FakeClientIdStore::new(""));
        let config = DoctorConfig::default();

        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Config, "Client ID")
            .expect("Client ID check should be present");
        assert_eq!(check.status, CheckStatus::Warning, "Client ID should warn when not assigned");
        assert!(check.fixable, "Missing client ID should be fixable");
    }

    #[test]
    fn config_values_check_passes_with_defaults() {
        let env = TestEnv::new();
        let (_temp, context) = env.into_parts();
        let config = DoctorConfig::default();

        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Config, "Config Values")
            .expect("Config Values check should be present");
        assert_eq!(check.status, CheckStatus::Passed, "Config Values should pass with defaults");
        assert!(
            check.message.contains("valid") || check.message.contains("within"),
            "Message should indicate valid values: {}",
            check.message
        );
    }

    #[test]
    fn config_values_check_warns_on_invalid_log_level() {
        let env = TestEnv::new();
        env.write_file(".lattice/config.toml", "[logging]\nlevel = \"invalid\"\n");
        let (_temp, context) = env.into_parts();
        let config = DoctorConfig::default();

        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Config, "Config Values")
            .expect("Config Values check should be present");
        assert_eq!(
            check.status,
            CheckStatus::Warning,
            "Config Values should warn on invalid log level"
        );
        assert!(
            check.message.contains("log") || check.message.contains("level"),
            "Message should mention log level: {}",
            check.message
        );
    }

    #[test]
    fn config_values_check_warns_on_negative_weights() {
        let env = TestEnv::new();
        env.write_file(".lattice/config.toml", "[overview]\nview_weight = -0.5\n");
        let (_temp, context) = env.into_parts();
        let config = DoctorConfig::default();

        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Config, "Config Values")
            .expect("Config Values check should be present");
        assert_eq!(
            check.status,
            CheckStatus::Warning,
            "Config Values should warn on negative weights"
        );
        assert!(
            check.message.contains("weight") || check.message.contains("Weight"),
            "Message should mention weights: {}",
            check.message
        );
    }

    #[test]
    fn all_config_checks_are_present() {
        let env = TestEnv::new();
        let (_temp, context) = env.into_parts();
        let config = DoctorConfig::default();

        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let config_checks: Vec<_> =
            results.iter().filter(|r| r.category == CheckCategory::Config).collect();

        assert!(
            config_checks.iter().any(|c| c.name == "User Config"),
            "Should have User Config check"
        );
        assert!(
            config_checks.iter().any(|c| c.name == "Repo Config"),
            "Should have Repo Config check"
        );
        assert!(config_checks.iter().any(|c| c.name == "Client ID"), "Should have Client ID check");
        assert!(
            config_checks.iter().any(|c| c.name == "Config Values"),
            "Should have Config Values check"
        );
        assert_eq!(config_checks.len(), 4, "Should have exactly 4 configuration checks");
    }
}

mod claim_checks {
    use std::path::PathBuf;

    use lattice::claim::claim_operations;
    use lattice::cli::commands::doctor_command::doctor_checks;
    use lattice::cli::commands::doctor_command::doctor_types::{
        CheckCategory, CheckStatus, DoctorConfig,
    };
    use lattice::id::lattice_id::LatticeId;
    use lattice::index::document_queries;
    use lattice::index::document_types::InsertDocument;
    use lattice::test::test_environment::TestEnv;

    fn find_check<'a>(
        results: &'a [lattice::cli::commands::doctor_command::doctor_types::CheckResult],
        category: CheckCategory,
        name: &str,
    ) -> Option<&'a lattice::cli::commands::doctor_command::doctor_types::CheckResult> {
        results.iter().find(|r| r.category == category && r.name == name)
    }

    #[test]
    fn active_claims_check_passes_when_no_claims() {
        let env = TestEnv::new();
        let (_temp, context) = env.into_parts();
        let config = DoctorConfig::default();

        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Claims, "Active Claims")
            .expect("Active Claims check should be present");
        assert_eq!(check.status, CheckStatus::Passed, "Active Claims should pass with no claims");
        assert!(
            check.message.contains("No claims"),
            "Message should indicate no claims: {}",
            check.message
        );
    }

    #[test]
    fn active_claims_check_shows_count_when_claims_exist() {
        let env = TestEnv::new();

        // Create a valid Lattice ID
        let id = LatticeId::from_parts(100, "ABC");
        let id_str = id.as_str();

        env.create_dir("api/tasks");
        env.create_task("api/tasks/task1.md", id_str, "task1", "Task 1", "task", 1);

        let (_temp, context) = env.into_parts();

        // Insert the task into the index
        document_queries::insert(&context.conn, &InsertDocument {
            id: id_str.to_string(),
            parent_id: None,
            path: "api/tasks/task1.md".to_string(),
            name: "task1".to_string(),
            description: "Task 1".to_string(),
            task_type: Some(lattice::document::frontmatter_schema::TaskType::Task),
            is_closed: false,
            priority: Some(1),
            created_at: None,
            updated_at: None,
            closed_at: None,
            body_hash: "hash1".to_string(),
            content_length: 100,
            is_root: false,
            in_tasks_dir: true,
            in_docs_dir: false,
            skill: false,
        })
        .expect("Insert task");

        // Create a claim for the task using the actual worktree path
        let work_path = context.repo_root.clone();
        claim_operations::claim_task(&context.repo_root, &id, &work_path).expect("Claim task");

        let config = DoctorConfig::default();
        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Claims, "Active Claims")
            .expect("Active Claims check should be present");
        assert_eq!(
            check.status,
            CheckStatus::Passed,
            "Active Claims should pass with valid claims"
        );
        assert!(
            check.message.contains("1 active"),
            "Message should show claim count: {}",
            check.message
        );
    }

    #[test]
    fn stale_claims_check_passes_when_no_stale_claims() {
        let env = TestEnv::new();
        let (_temp, context) = env.into_parts();
        let config = DoctorConfig::default();

        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Claims, "Stale Claims")
            .expect("Stale Claims check should be present");
        assert_eq!(check.status, CheckStatus::Passed, "Stale Claims should pass with no claims");
    }

    #[test]
    fn stale_claims_check_warns_when_task_is_closed() {
        let env = TestEnv::new();

        // Create a valid Lattice ID
        let id = LatticeId::from_parts(200, "ABC");
        let id_str = id.as_str();

        env.create_dir("api/tasks/.closed");
        env.create_task("api/tasks/.closed/done.md", id_str, "done", "Done task", "task", 1);

        let (_temp, context) = env.into_parts();

        // Insert the closed task into the index
        document_queries::insert(&context.conn, &InsertDocument {
            id: id_str.to_string(),
            parent_id: None,
            path: "api/tasks/.closed/done.md".to_string(),
            name: "done".to_string(),
            description: "Done task".to_string(),
            task_type: Some(lattice::document::frontmatter_schema::TaskType::Task),
            is_closed: true,
            priority: Some(1),
            created_at: None,
            updated_at: None,
            closed_at: None,
            body_hash: "hash1".to_string(),
            content_length: 100,
            is_root: false,
            in_tasks_dir: true,
            in_docs_dir: false,
            skill: false,
        })
        .expect("Insert closed task");

        // Create a claim for the closed task
        let work_path = context.repo_root.clone();
        claim_operations::claim_task(&context.repo_root, &id, &work_path).expect("Claim task");

        let config = DoctorConfig::default();
        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Claims, "Stale Claims")
            .expect("Stale Claims check should be present");
        assert_eq!(
            check.status,
            CheckStatus::Warning,
            "Stale Claims should warn when claim exists for closed task"
        );
        assert!(check.fixable, "Stale claims should be fixable");
        assert!(
            check.details.iter().any(|d| d.contains(id_str)),
            "Details should mention the stale claim ID: {:?}",
            check.details
        );
    }

    #[test]
    fn missing_tasks_check_passes_when_no_missing_tasks() {
        let env = TestEnv::new();
        let (_temp, context) = env.into_parts();
        let config = DoctorConfig::default();

        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Claims, "Missing Tasks")
            .expect("Missing Tasks check should be present");
        assert_eq!(check.status, CheckStatus::Passed, "Missing Tasks should pass with no claims");
    }

    #[test]
    fn missing_tasks_check_warns_when_task_deleted() {
        let env = TestEnv::new();
        let (_temp, context) = env.into_parts();

        // Create a valid Lattice ID for a task that doesn't exist in the index
        let id = LatticeId::from_parts(300, "ABC");
        let id_str = id.as_str();

        // Create a claim for a task that doesn't exist in the index
        let work_path = context.repo_root.clone();
        claim_operations::claim_task(&context.repo_root, &id, &work_path).expect("Claim task");

        let config = DoctorConfig::default();
        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Claims, "Missing Tasks")
            .expect("Missing Tasks check should be present");
        assert_eq!(
            check.status,
            CheckStatus::Warning,
            "Missing Tasks should warn when claim references non-existent task"
        );
        assert!(check.fixable, "Missing tasks claims should be fixable");
        assert!(
            check.details.iter().any(|d| d.contains(id_str)),
            "Details should mention the missing task ID: {:?}",
            check.details
        );
    }

    #[test]
    fn orphaned_worktrees_check_passes_when_no_orphaned_worktrees() {
        let env = TestEnv::new();
        let (_temp, context) = env.into_parts();
        let config = DoctorConfig::default();

        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Claims, "Orphaned Worktrees")
            .expect("Orphaned Worktrees check should be present");
        assert_eq!(
            check.status,
            CheckStatus::Passed,
            "Orphaned Worktrees should pass with no claims"
        );
    }

    #[test]
    fn orphaned_worktrees_check_warns_when_work_path_missing() {
        let env = TestEnv::new();

        // Create a valid Lattice ID
        let id = LatticeId::from_parts(400, "ABC");
        let id_str = id.as_str();

        env.create_dir("api/tasks");
        env.create_task("api/tasks/task1.md", id_str, "task1", "Task 1", "task", 1);

        let (_temp, context) = env.into_parts();

        // Insert the task into the index
        document_queries::insert(&context.conn, &InsertDocument {
            id: id_str.to_string(),
            parent_id: None,
            path: "api/tasks/task1.md".to_string(),
            name: "task1".to_string(),
            description: "Task 1".to_string(),
            task_type: Some(lattice::document::frontmatter_schema::TaskType::Task),
            is_closed: false,
            priority: Some(1),
            created_at: None,
            updated_at: None,
            closed_at: None,
            body_hash: "hash1".to_string(),
            content_length: 100,
            is_root: false,
            in_tasks_dir: true,
            in_docs_dir: false,
            skill: false,
        })
        .expect("Insert task");

        // Create a claim with a non-existent work path
        let work_path = PathBuf::from("/nonexistent/worktree/path/that/does/not/exist");
        claim_operations::claim_task(&context.repo_root, &id, &work_path).expect("Claim task");

        let config = DoctorConfig::default();
        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let check = find_check(&results, CheckCategory::Claims, "Orphaned Worktrees")
            .expect("Orphaned Worktrees check should be present");
        assert_eq!(
            check.status,
            CheckStatus::Warning,
            "Orphaned Worktrees should warn when work path doesn't exist"
        );
        assert!(check.fixable, "Orphaned worktree claims should be fixable");
        assert!(
            check.details.iter().any(|d| d.contains(id_str)),
            "Details should mention the task ID: {:?}",
            check.details
        );
        assert!(
            check.details.iter().any(|d| d.contains("nonexistent")),
            "Details should mention the missing path: {:?}",
            check.details
        );
    }

    #[test]
    fn all_claim_checks_are_present() {
        let env = TestEnv::new();
        let (_temp, context) = env.into_parts();
        let config = DoctorConfig::default();

        let results = doctor_checks::run_all_checks(&context, &config).expect("Run checks");

        let claim_checks: Vec<_> =
            results.iter().filter(|r| r.category == CheckCategory::Claims).collect();

        assert!(
            claim_checks.iter().any(|c| c.name == "Active Claims"),
            "Should have Active Claims check"
        );
        assert!(
            claim_checks.iter().any(|c| c.name == "Stale Claims"),
            "Should have Stale Claims check"
        );
        assert!(
            claim_checks.iter().any(|c| c.name == "Missing Tasks"),
            "Should have Missing Tasks check"
        );
        assert!(
            claim_checks.iter().any(|c| c.name == "Orphaned Worktrees"),
            "Should have Orphaned Worktrees check"
        );
        assert_eq!(claim_checks.len(), 4, "Should have exactly 4 claims checks");
    }
}
