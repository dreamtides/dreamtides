use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use tracing::{debug, instrument};

use crate::cli::query_args::ListArgs;
use crate::cli::shared_options::{FilterOptions, OutputOptions, SortField, TaskState};
use crate::error::error_types::LatticeError;
use crate::index::document_filter::{DocumentFilter, DocumentState, SortColumn, SortOrder};

/// Builds a [`DocumentFilter`] from command-line arguments.
///
/// Parses and validates all filter options, converting CLI argument types to
/// the domain types used by the query layer.
#[instrument(skip_all, fields(has_state = args.filter.state.is_some()))]
pub fn build_filter(args: &ListArgs) -> Result<DocumentFilter, LatticeError> {
    debug!("Building document filter from CLI arguments");

    let mut filter = create_base_filter(&args.filter)?;
    apply_timestamp_filters(&mut filter, &args.filter)?;
    apply_sort_options(&mut filter, &args.output);
    apply_limit(&mut filter, &args.output);

    debug!(?filter, "Document filter built successfully");
    Ok(filter)
}

/// Creates the base filter with state, priority, type, labels, path, and name
/// filters.
fn create_base_filter(opts: &FilterOptions) -> Result<DocumentFilter, LatticeError> {
    let mut filter = if opts.include_closed || opts.closed_only {
        DocumentFilter::including_closed()
    } else {
        DocumentFilter::new()
    };

    apply_state_filter(&mut filter, opts)?;
    apply_priority_filters(&mut filter, opts)?;

    if let Some(task_type) = opts.r#type {
        filter = filter.with_task_type(task_type);
    }

    if !opts.label.is_empty() {
        filter = filter.with_labels_all(opts.label.clone());
    }

    if !opts.label_any.is_empty() {
        filter = filter.with_labels_any(opts.label_any.clone());
    }

    if let Some(name) = &opts.name_contains {
        filter = filter.with_name_contains(name.clone());
    }

    if let Some(path) = &opts.path {
        filter = filter.with_path_prefix(path.clone());
    }

    if opts.roots_only {
        filter = filter.with_is_root(true);
    }

    if let Some(parent_id) = &opts.discovered_from {
        filter = filter.with_discovered_from(parent_id.clone());
    }

    Ok(filter)
}

/// Applies state filter, handling --state, --closed-only, and their
/// interactions.
fn apply_state_filter(
    filter: &mut DocumentFilter,
    opts: &FilterOptions,
) -> Result<(), LatticeError> {
    if opts.closed_only {
        if let Some(state) = opts.state
            && state != TaskState::Closed
        {
            return Err(LatticeError::ConflictingOptions {
                option1: "--closed-only".to_string(),
                option2: format!("--state {state:?}"),
            });
        }
        filter.state = Some(DocumentState::Closed);
        return Ok(());
    }

    if let Some(state) = opts.state {
        filter.state = Some(map_task_state(state));
    }

    Ok(())
}

/// Maps CLI TaskState to DocumentFilter DocumentState.
fn map_task_state(state: TaskState) -> DocumentState {
    match state {
        TaskState::Open => DocumentState::Open,
        TaskState::Blocked => DocumentState::Blocked,
        TaskState::Closed => DocumentState::Closed,
    }
}

/// Applies priority filters, validating range constraints.
fn apply_priority_filters(
    filter: &mut DocumentFilter,
    opts: &FilterOptions,
) -> Result<(), LatticeError> {
    if let Some(priority) = opts.priority {
        validate_priority(priority)?;
        *filter = filter.clone().with_priority(priority);
    }

    let min = opts.priority_min;
    let max = opts.priority_max;

    if min.is_some() || max.is_some() {
        let min_val = min.unwrap_or(0);
        let max_val = max.unwrap_or(4);

        validate_priority(min_val)?;
        validate_priority(max_val)?;

        if min_val > max_val {
            return Err(LatticeError::InvalidArgument {
                message: format!(
                    "priority-min ({min_val}) cannot be greater than priority-max ({max_val})"
                ),
            });
        }

        *filter = filter.clone().with_priority_range(min_val, max_val);
    }

    Ok(())
}

/// Validates a priority value is in the valid range (0-4).
fn validate_priority(priority: u8) -> Result<(), LatticeError> {
    if priority > 4 {
        return Err(LatticeError::InvalidArgument {
            message: format!("priority must be between 0 and 4, got {priority}"),
        });
    }
    Ok(())
}

/// Applies timestamp filter options.
fn apply_timestamp_filters(
    filter: &mut DocumentFilter,
    opts: &FilterOptions,
) -> Result<(), LatticeError> {
    if let Some(date_str) = &opts.created_after {
        let dt = parse_timestamp(date_str, "created-after")?;
        *filter = filter.clone().with_created_after(dt);
    }

    if let Some(date_str) = &opts.created_before {
        let dt = parse_timestamp(date_str, "created-before")?;
        *filter = filter.clone().with_created_before(dt);
    }

    if let Some(date_str) = &opts.updated_after {
        let dt = parse_timestamp(date_str, "updated-after")?;
        *filter = filter.clone().with_updated_after(dt);
    }

    if let Some(date_str) = &opts.updated_before {
        let dt = parse_timestamp(date_str, "updated-before")?;
        *filter = filter.clone().with_updated_before(dt);
    }

    Ok(())
}

/// Parses a timestamp string, accepting ISO 8601 date or datetime formats.
///
/// Accepts:
/// - `YYYY-MM-DD` - date only, interpreted as midnight UTC
/// - `YYYY-MM-DDTHH:MM:SS` - datetime without timezone, interpreted as UTC
/// - `YYYY-MM-DDTHH:MM:SSZ` - datetime with Z suffix
/// - `YYYY-MM-DDTHH:MM:SS+00:00` - datetime with offset
fn parse_timestamp(s: &str, field_name: &str) -> Result<DateTime<Utc>, LatticeError> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc));
    }

    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Ok(ndt.and_utc());
    }

    if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let dt = date.and_time(NaiveTime::MIN).and_utc();
        return Ok(dt);
    }

    Err(LatticeError::InvalidArgument {
        message: format!(
            "invalid date format for --{field_name}: '{s}'. \
             Expected ISO 8601 format (YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS)"
        ),
    })
}

/// Applies sort options from CLI arguments.
fn apply_sort_options(filter: &mut DocumentFilter, opts: &OutputOptions) {
    if let Some(sort_field) = opts.sort {
        filter.sort_by = map_sort_field(sort_field);
    }

    if opts.reverse {
        filter.sort_order = match filter.sort_order {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        };
    }
}

/// Maps CLI SortField to DocumentFilter SortColumn.
fn map_sort_field(field: SortField) -> SortColumn {
    match field {
        SortField::Priority => SortColumn::Priority,
        SortField::Created => SortColumn::CreatedAt,
        SortField::Updated => SortColumn::UpdatedAt,
        SortField::Name => SortColumn::Name,
    }
}

/// Applies limit option if specified.
fn apply_limit(filter: &mut DocumentFilter, opts: &OutputOptions) {
    if let Some(limit) = opts.limit {
        *filter = filter.clone().limit(limit as u32);
    }
}
