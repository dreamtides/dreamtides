use std::collections::HashMap;

use serde::Serialize;
use tauri::{AppHandle, State};

use crate::derived::compute_executor::{ComputationRequest, ComputeExecutorState};
use crate::derived::derived_types::{LookupContext, RowData};
use crate::derived::generation_tracker::RowKey;
use crate::error::error_types::TvError;
use crate::toml::metadata_parser;

/// Request to compute a derived column value.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ComputeDerivedRequest {
    /// The file path of the TOML file.
    pub file_path: String,
    /// The table name within the file.
    pub table_name: String,
    /// The zero-based row index.
    pub row_index: usize,
    /// The name of the derived function to execute.
    pub function_name: String,
    /// The row data to pass to the function.
    pub row_data: HashMap<String, serde_json::Value>,
    /// Whether this row is currently visible on screen.
    pub is_visible: bool,
}

/// Request to compute derived values for multiple rows.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ComputeDerivedBatchRequest {
    /// The list of computation requests.
    pub requests: Vec<ComputeDerivedRequest>,
}

/// Request to update the lookup context for cross-table references.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct UpdateLookupContextRequest {
    /// Map of table name to rows, where each row is indexed by its ID.
    pub tables: HashMap<String, HashMap<String, HashMap<String, serde_json::Value>>>,
}

/// Tauri command to queue a single derived column computation.
///
/// The computation runs asynchronously and the result is sent via
/// the "derived-value-computed" event.
#[tauri::command]
pub fn compute_derived(
    _app_handle: AppHandle,
    executor_state: State<ComputeExecutorState>,
    request: ComputeDerivedRequest,
) -> Result<(), TvError> {
    // Get the current generation for this row
    let generation = executor_state
        .with_executor(|executor| {
            executor
                .generation_tracker()
                .get_generation(&RowKey::new(&request.file_path, &request.table_name, request.row_index))
        })
        .unwrap_or(0);

    let computation_request = ComputationRequest {
        row_key: RowKey::new(&request.file_path, &request.table_name, request.row_index),
        function_name: request.function_name.clone(),
        row_data: request.row_data,
        generation,
        is_visible: request.is_visible,
    };

    executor_state.with_executor(|executor| {
        executor.queue_computation(computation_request);
    });

    tracing::debug!(
        component = "tv.commands.derived",
        file_path = %request.file_path,
        table_name = %request.table_name,
        row_index = request.row_index,
        function_name = %request.function_name,
        is_visible = request.is_visible,
        "Queued derived computation"
    );

    Ok(())
}

/// Tauri command to queue multiple derived column computations.
///
/// Visible rows are prioritized for computation.
#[tauri::command]
pub fn compute_derived_batch(
    _app_handle: AppHandle,
    executor_state: State<ComputeExecutorState>,
    batch: ComputeDerivedBatchRequest,
) -> Result<(), TvError> {
    let requests: Vec<ComputationRequest> = batch
        .requests
        .into_iter()
        .map(|req| {
            let generation = executor_state
                .with_executor(|executor| {
                    executor
                        .generation_tracker()
                        .get_generation(&RowKey::new(&req.file_path, &req.table_name, req.row_index))
                })
                .unwrap_or(0);

            ComputationRequest {
                row_key: RowKey::new(&req.file_path, &req.table_name, req.row_index),
                function_name: req.function_name,
                row_data: req.row_data,
                generation,
                is_visible: req.is_visible,
            }
        })
        .collect();

    let count = requests.len();
    executor_state.with_executor(|executor| {
        executor.queue_batch(requests);
    });

    tracing::debug!(
        component = "tv.commands.derived",
        request_count = count,
        "Queued batch of derived computations"
    );

    Ok(())
}

/// Tauri command to update the lookup context for cross-table references.
///
/// This should be called when tables are loaded or reloaded to provide
/// up-to-date data for functions like CardLookup.
#[tauri::command]
pub fn update_lookup_context(
    executor_state: State<ComputeExecutorState>,
    request: UpdateLookupContextRequest,
) -> Result<(), TvError> {
    let mut context = LookupContext::new();
    let mut table_count = 0;
    let mut row_count = 0;

    for (table_name, rows) in request.tables {
        let table_rows: HashMap<String, RowData> = rows.into_iter().collect();
        row_count += table_rows.len();
        context.add_table(table_name, table_rows);
        table_count += 1;
    }

    executor_state.with_executor(|executor| {
        executor.set_lookup_context(context);
    });

    tracing::debug!(
        component = "tv.commands.derived",
        table_count = table_count,
        row_count = row_count,
        "Updated lookup context"
    );

    Ok(())
}

/// Tauri command to increment the generation counter for a row.
///
/// This should be called when a row is edited to invalidate any
/// in-flight computations for that row.
#[tauri::command]
pub fn increment_row_generation(
    executor_state: State<ComputeExecutorState>,
    file_path: String,
    table_name: String,
    row_index: usize,
) -> Result<u64, TvError> {
    let generation = executor_state
        .with_executor(|executor| {
            executor
                .generation_tracker()
                .increment_generation(RowKey::new(&file_path, &table_name, row_index))
        })
        .ok_or_else(|| TvError::BackendThreadPanic {
            thread_name: "compute-executor".to_string(),
            message: "Executor not initialized".to_string(),
        })?;

    tracing::debug!(
        component = "tv.commands.derived",
        file_path = %file_path,
        table_name = %table_name,
        row_index = row_index,
        generation = generation,
        "Incremented row generation"
    );

    Ok(generation)
}

/// Tauri command to clear the computation queue.
///
/// Useful when switching files or tables to avoid processing stale requests.
#[tauri::command]
pub fn clear_computation_queue(executor_state: State<ComputeExecutorState>) -> Result<(), TvError> {
    executor_state.with_executor(|executor| {
        executor.clear_queue();
    });

    tracing::debug!(
        component = "tv.commands.derived",
        "Cleared computation queue"
    );

    Ok(())
}

/// Tauri command to get the current computation queue length.
#[tauri::command]
pub fn get_computation_queue_length(executor_state: State<ComputeExecutorState>) -> Result<usize, TvError> {
    let length = executor_state.with_executor(|executor| executor.queue_len()).unwrap_or(0);

    Ok(length)
}

/// Frontend-facing derived column configuration.
#[derive(Debug, Clone, Serialize)]
pub struct DerivedColumnInfo {
    /// Display name for the column header.
    pub name: String,
    /// Registered function name to compute values.
    pub function: String,
    /// Column position (0-indexed), if specified.
    pub position: Option<usize>,
    /// Column width in pixels.
    pub width: u32,
    /// Input field names passed to the function.
    pub inputs: Vec<String>,
}

/// Tauri command to get derived column configurations from a file's metadata.
#[tauri::command]
pub fn get_derived_columns_config(file_path: String) -> Result<Vec<DerivedColumnInfo>, TvError> {
    let configs = metadata_parser::parse_derived_columns_from_file(&file_path)?;

    let result: Vec<DerivedColumnInfo> = configs
        .into_iter()
        .map(|c| DerivedColumnInfo {
            name: c.name,
            function: c.function,
            position: c.position,
            width: c.width,
            inputs: c.inputs,
        })
        .collect();

    tracing::debug!(
        component = "tv.commands.derived",
        file_path = %file_path,
        column_count = result.len(),
        "Retrieved derived column configs"
    );

    Ok(result)
}
