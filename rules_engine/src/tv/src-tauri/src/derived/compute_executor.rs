use std::collections::VecDeque;
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

use crate::derived::derived_types::{DerivedResult, LookupContext, RowData};
use crate::derived::function_registry::global_registry;
use crate::derived::generation_tracker::{GenerationTracker, RowKey};
use crate::error::error_types::TvError;

/// A request to compute a derived column value.
#[derive(Debug, Clone)]
pub struct ComputationRequest {
    /// The row key identifying the row to compute.
    pub row_key: RowKey,
    /// The name of the derived function to execute.
    pub function_name: String,
    /// The row data to pass to the function.
    pub row_data: RowData,
    /// The generation of the row when this request was created.
    pub generation: u64,
    /// Whether this row is currently visible (affects priority).
    pub is_visible: bool,
}

/// Payload sent to the frontend when a derived value is computed.
#[derive(Debug, Clone, Serialize)]
pub struct DerivedValueComputedPayload {
    /// The file path of the TOML file.
    pub file_path: String,
    /// The table name within the file.
    pub table_name: String,
    /// The zero-based row index.
    pub row_index: usize,
    /// The name of the derived function that computed this value.
    pub function_name: String,
    /// The computed result.
    pub result: serde_json::Value,
    /// The generation this result was computed for.
    pub generation: u64,
}

/// Tauri-managed state for the compute executor.
pub struct ComputeExecutorState {
    executor: RwLock<Option<ComputeExecutor>>,
}

/// Executes a single derived function computation, catching panics at the
/// task boundary and converting them to `DerivedResult::Error`.
pub fn execute_computation(
    function_name: &str,
    row_data: &RowData,
    context: &LookupContext,
) -> DerivedResult {
    let result = global_registry().with_function(function_name, |function| {
        let panic_result = panic::catch_unwind(AssertUnwindSafe(|| {
            function.compute(row_data, context)
        }));

        match panic_result {
            Ok(result) => result,
            Err(panic_info) => {
                let panic_message = if let Some(s) = panic_info.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic_info.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown panic".to_string()
                };

                tracing::error!(
                    component = "tv.derived.executor",
                    function_name = %function_name,
                    panic_message = %panic_message,
                    "Derived function panicked"
                );

                DerivedResult::Error(format!("Function panicked: {panic_message}"))
            }
        }
    });

    match result {
        Some(r) => r,
        None => {
            tracing::error!(
                component = "tv.derived.executor",
                function_name = %function_name,
                "Derived function not found"
            );
            DerivedResult::Error(format!("Function not found: {function_name}"))
        }
    }
}

/// Manages async computation of derived column values.
///
/// The executor maintains a queue of computation requests and processes them
/// using a tokio thread pool. Results are sent to the frontend via Tauri events.
/// Visible rows are prioritized over offscreen rows.
pub struct ComputeExecutor {
    /// The tokio runtime for async execution.
    runtime: Runtime,
    /// Queue of pending computation requests.
    queue: Arc<Mutex<VecDeque<ComputationRequest>>>,
    /// Generation tracker for staleness detection.
    generation_tracker: Arc<GenerationTracker>,
    /// Lookup context for cross-table references.
    lookup_context: Arc<RwLock<LookupContext>>,
    /// Flag indicating whether the executor is running.
    is_running: Arc<AtomicBool>,
    /// Channel to send shutdown signals.
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl ComputeExecutor {
    /// Creates a new compute executor with the specified number of worker threads.
    ///
    /// If `num_threads` is None, defaults to the number of CPU cores.
    pub fn new(num_threads: Option<usize>) -> Result<Self, TvError> {
        let threads = num_threads.unwrap_or_else(num_cpus::get);
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(threads)
            .thread_name("tv-compute")
            .enable_all()
            .build()
            .map_err(|e| TvError::BackendThreadPanic {
                thread_name: "compute-executor".to_string(),
                message: format!("Failed to create tokio runtime: {e}"),
            })?;

        tracing::info!(
            component = "tv.derived.executor",
            num_threads = threads,
            "Created compute executor"
        );

        Ok(Self {
            runtime,
            queue: Arc::new(Mutex::new(VecDeque::new())),
            generation_tracker: Arc::new(GenerationTracker::new()),
            lookup_context: Arc::new(RwLock::new(LookupContext::new())),
            is_running: Arc::new(AtomicBool::new(false)),
            shutdown_tx: None,
        })
    }

    /// Returns a reference to the generation tracker.
    pub fn generation_tracker(&self) -> &Arc<GenerationTracker> {
        &self.generation_tracker
    }

    /// Updates the lookup context for cross-table references.
    pub fn set_lookup_context(&self, context: LookupContext) {
        if let Ok(mut ctx) = self.lookup_context.write() {
            *ctx = context;
            tracing::debug!(
                component = "tv.derived.executor",
                "Updated lookup context"
            );
        }
    }

    /// Queues a computation request.
    ///
    /// Visible rows are added to the front of the queue for priority processing.
    pub fn queue_computation(&self, request: ComputationRequest) {
        let mut queue = self.queue.lock().expect("Compute queue lock poisoned");

        if request.is_visible {
            // Visible rows get priority - add to front
            queue.push_front(request.clone());
            tracing::debug!(
                component = "tv.derived.executor",
                file_path = %request.row_key.file_path,
                table_name = %request.row_key.table_name,
                row_index = request.row_key.row_index,
                function_name = %request.function_name,
                "Queued visible row computation (priority)"
            );
        } else {
            // Offscreen rows go to the back
            queue.push_back(request.clone());
            tracing::debug!(
                component = "tv.derived.executor",
                file_path = %request.row_key.file_path,
                table_name = %request.row_key.table_name,
                row_index = request.row_key.row_index,
                function_name = %request.function_name,
                "Queued offscreen row computation"
            );
        }
    }

    /// Queues multiple computation requests for a batch of rows.
    ///
    /// Visible rows are added to the front of the queue for priority processing.
    pub fn queue_batch(&self, requests: Vec<ComputationRequest>) {
        let mut queue = self.queue.lock().expect("Compute queue lock poisoned");

        // Separate visible and offscreen requests
        let (visible, offscreen): (Vec<_>, Vec<_>) =
            requests.into_iter().partition(|r| r.is_visible);

        // Add visible rows to front (in reverse order to maintain order)
        for request in visible.into_iter().rev() {
            queue.push_front(request);
        }

        // Add offscreen rows to back
        for request in offscreen {
            queue.push_back(request);
        }

        tracing::debug!(
            component = "tv.derived.executor",
            queue_size = queue.len(),
            "Queued batch of computation requests"
        );
    }

    /// Starts the executor loop that processes queued computations.
    ///
    /// Returns immediately; processing happens on background threads.
    pub fn start(&mut self, app_handle: AppHandle) {
        if self.is_running.swap(true, Ordering::SeqCst) {
            tracing::warn!(
                component = "tv.derived.executor",
                "Executor already running"
            );
            return;
        }

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        let queue = Arc::clone(&self.queue);
        let generation_tracker = Arc::clone(&self.generation_tracker);
        let lookup_context = Arc::clone(&self.lookup_context);
        let is_running = Arc::clone(&self.is_running);

        self.runtime.spawn(async move {
            tracing::info!(
                component = "tv.derived.executor",
                "Executor loop started"
            );

            loop {
                // Check for shutdown
                if shutdown_rx.try_recv().is_ok() {
                    tracing::info!(
                        component = "tv.derived.executor",
                        "Executor received shutdown signal"
                    );
                    break;
                }

                // Get next request from queue
                let request = {
                    let mut q = queue.lock().expect("Compute queue lock poisoned");
                    q.pop_front()
                };

                match request {
                    Some(req) => {
                        // Check if the request is still current
                        if !generation_tracker.is_generation_current(&req.row_key, req.generation) {
                            tracing::debug!(
                                component = "tv.derived.executor",
                                file_path = %req.row_key.file_path,
                                table_name = %req.row_key.table_name,
                                row_index = req.row_key.row_index,
                                "Skipping stale computation request"
                            );
                            continue;
                        }

                        // Execute the computation
                        let context = lookup_context.read().expect("Lookup context lock poisoned");
                        let result =
                            execute_computation(&req.function_name, &req.row_data, &context);
                        drop(context);

                        // Check again if result is still current before emitting
                        if !generation_tracker.is_generation_current(&req.row_key, req.generation) {
                            tracing::debug!(
                                component = "tv.derived.executor",
                                file_path = %req.row_key.file_path,
                                table_name = %req.row_key.table_name,
                                row_index = req.row_key.row_index,
                                "Discarding stale computation result"
                            );
                            continue;
                        }

                        // Emit result to frontend
                        let payload = DerivedValueComputedPayload {
                            file_path: req.row_key.file_path.clone(),
                            table_name: req.row_key.table_name.clone(),
                            row_index: req.row_key.row_index,
                            function_name: req.function_name.clone(),
                            result: result.to_frontend_value(),
                            generation: req.generation,
                        };

                        if let Err(e) = app_handle.emit("derived-value-computed", &payload) {
                            tracing::error!(
                                component = "tv.derived.executor",
                                error = %e,
                                "Failed to emit derived value event"
                            );
                        } else {
                            tracing::debug!(
                                component = "tv.derived.executor",
                                file_path = %req.row_key.file_path,
                                table_name = %req.row_key.table_name,
                                row_index = req.row_key.row_index,
                                function_name = %req.function_name,
                                "Emitted derived value result"
                            );
                        }
                    }
                    None => {
                        // Queue is empty, wait a bit before checking again
                        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                    }
                }
            }

            is_running.store(false, Ordering::SeqCst);
            tracing::info!(
                component = "tv.derived.executor",
                "Executor loop stopped"
            );
        });
    }

    /// Stops the executor loop.
    pub fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.try_send(());
            tracing::info!(
                component = "tv.derived.executor",
                "Shutdown signal sent"
            );
        }
    }

    /// Clears all pending computation requests.
    pub fn clear_queue(&self) {
        let mut queue = self.queue.lock().expect("Compute queue lock poisoned");
        let count = queue.len();
        queue.clear();
        tracing::debug!(
            component = "tv.derived.executor",
            cleared_count = count,
            "Cleared computation queue"
        );
    }

    /// Returns the number of pending computation requests.
    pub fn queue_len(&self) -> usize {
        self.queue.lock().expect("Compute queue lock poisoned").len()
    }

    /// Returns whether the executor loop is currently running.
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }
}

impl Drop for ComputeExecutor {
    fn drop(&mut self) {
        self.stop();
    }
}

impl ComputeExecutorState {
    /// Creates a new uninitialized executor state.
    pub fn new() -> Self {
        Self { executor: RwLock::new(None) }
    }

    /// Initializes the compute executor with the given configuration.
    pub fn initialize(&self, num_threads: Option<usize>) -> Result<(), TvError> {
        let mut guard = self.executor.write().expect("Executor state lock poisoned");
        if guard.is_some() {
            tracing::warn!(
                component = "tv.derived.executor",
                "Executor already initialized"
            );
            return Ok(());
        }

        let executor = ComputeExecutor::new(num_threads)?;
        *guard = Some(executor);
        Ok(())
    }

    /// Executes a callback with access to the executor.
    pub fn with_executor<F, R>(&self, callback: F) -> Option<R>
    where
        F: FnOnce(&ComputeExecutor) -> R,
    {
        let guard = self.executor.read().expect("Executor state lock poisoned");
        guard.as_ref().map(callback)
    }

    /// Executes a callback with mutable access to the executor.
    pub fn with_executor_mut<F, R>(&self, callback: F) -> Option<R>
    where
        F: FnOnce(&mut ComputeExecutor) -> R,
    {
        let mut guard = self.executor.write().expect("Executor state lock poisoned");
        guard.as_mut().map(callback)
    }

    /// Starts the executor if not already running.
    pub fn start(&self, app_handle: AppHandle) {
        let mut guard = self.executor.write().expect("Executor state lock poisoned");
        if let Some(executor) = guard.as_mut() {
            executor.start(app_handle);
        }
    }

    /// Stops the executor if running.
    pub fn stop(&self) {
        let mut guard = self.executor.write().expect("Executor state lock poisoned");
        if let Some(executor) = guard.as_mut() {
            executor.stop();
        }
    }
}

impl Default for ComputeExecutorState {
    fn default() -> Self {
        Self::new()
    }
}

