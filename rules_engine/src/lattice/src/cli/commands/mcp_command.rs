use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use chrono::Utc;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tracing::info;

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::document::document_writer::{self, WriteOptions};
use crate::document::field_validation;
use crate::document::frontmatter_schema::{DEFAULT_PRIORITY, Frontmatter, TaskType};
use crate::error::error_types::LatticeError;
use crate::git::client_config;
use crate::id::id_generator::INITIAL_COUNTER;
use crate::id::lattice_id::LatticeId;
use crate::index::document_types::InsertDocument;
use crate::index::{client_counters, directory_roots, document_queries, label_queries};
/// MCP error codes.
#[expect(dead_code, reason = "Defined for completeness per MCP spec")]
mod error_codes {
    pub const INVALID_DIRECTORY: &str = "INVALID_DIRECTORY";
    pub const INVALID_TASK_TYPE: &str = "INVALID_TASK_TYPE";
    pub const INVALID_PRIORITY: &str = "INVALID_PRIORITY";
    pub const NAME_TOO_LONG: &str = "NAME_TOO_LONG";
    pub const DESCRIPTION_TOO_LONG: &str = "DESCRIPTION_TOO_LONG";
    pub const FILE_EXISTS: &str = "FILE_EXISTS";
    pub const MISSING_ROOT: &str = "MISSING_ROOT";
    pub const ID_COLLISION: &str = "ID_COLLISION";
    pub const INVALID_ID: &str = "INVALID_ID";
}

/// JSON-RPC error codes.
#[expect(dead_code, reason = "Defined for completeness per JSON-RPC spec")]
mod rpc_error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
}

/// Articles to skip when generating filenames from descriptions.
const SKIP_WORDS: &[&str] = &["the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for"];
/// Maximum length for auto-generated filenames (excluding extension and ID
/// suffix).
const MAX_FILENAME_LENGTH: usize = 40;

/// Executes the `lat mcp` command.
///
/// Reads a single JSON-RPC request from stdin, executes the requested tool,
/// and writes the JSON-RPC response to stdout.
pub fn execute(context: CommandContext) -> LatticeResult<()> {
    info!("Starting MCP command");

    // Read request from stdin
    let request = read_request()?;

    // Process the request
    let response = process_request(&context, request);

    // Write response to stdout
    write_response(&response)?;

    info!("MCP command completed");
    Ok(())
}

/// JSON-RPC request envelope.
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: serde_json::Value,
    method: String,
    #[serde(default)]
    params: serde_json::Value,
}

/// JSON-RPC response envelope.
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

/// JSON-RPC error object.
#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<McpErrorData>,
}

/// Extended error data for MCP errors.
#[derive(Debug, Serialize)]
struct McpErrorData {
    code: String,
    suggestion: Option<String>,
}

/// Parameters for the `lattice_create_task` tool.
#[derive(Debug, Deserialize)]
struct CreateTaskParams {
    /// Parent directory path (e.g., "api/").
    directory: String,
    /// Human-readable task title.
    description: String,
    /// One of: bug, feature, task, chore.
    task_type: String,
    /// Markdown body content.
    body: String,
    /// Priority 0-4 (default: 2).
    #[serde(default)]
    priority: Option<u8>,
    /// List of labels.
    #[serde(default)]
    labels: Vec<String>,
    /// List of blocking task IDs.
    #[serde(default)]
    blocked_by: Vec<String>,
    /// List of parent task IDs for provenance.
    #[serde(default)]
    discovered_from: Vec<String>,
}

/// Parameters for the `lattice_create_document` tool.
#[derive(Debug, Deserialize)]
struct CreateDocumentParams {
    /// Parent directory path (e.g., "api/").
    directory: String,
    /// Document name (becomes filename, max 64 chars).
    name: String,
    /// Human-readable description (max 1024 chars).
    description: String,
    /// Markdown body content.
    body: String,
    /// List of labels.
    #[serde(default)]
    labels: Vec<String>,
}

/// Result returned by document creation tools.
#[derive(Debug, Serialize)]
struct CreateResult {
    lattice_id: String,
    path: String,
    name: String,
}

/// Reads a JSON-RPC request from stdin.
fn read_request() -> LatticeResult<JsonRpcRequest> {
    let stdin = io::stdin();
    let mut input = String::new();

    for line in stdin.lock().lines() {
        let line = line.map_err(|e| LatticeError::ReadError {
            path: PathBuf::from("<stdin>"),
            reason: e.to_string(),
        })?;
        input.push_str(&line);
        input.push('\n');
    }

    serde_json::from_str(&input).map_err(|e| LatticeError::InvalidArgument {
        message: format!("Invalid JSON-RPC request: {e}"),
    })
}

/// Writes a JSON-RPC response to stdout.
fn write_response(response: &JsonRpcResponse) -> LatticeResult<()> {
    let output = serde_json::to_string(response).map_err(|e| LatticeError::InvalidArgument {
        message: format!("Failed to serialize response: {e}"),
    })?;

    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{output}").map_err(|e| LatticeError::WriteError {
        path: PathBuf::from("<stdout>"),
        reason: e.to_string(),
    })?;

    Ok(())
}

/// Processes a JSON-RPC request and returns the response.
fn process_request(context: &CommandContext, request: JsonRpcRequest) -> JsonRpcResponse {
    // Validate JSON-RPC version
    if request.jsonrpc != "2.0" {
        return error_response(
            request.id,
            rpc_error_codes::INVALID_REQUEST,
            "Invalid JSON-RPC version",
            None,
        );
    }

    // Dispatch to the appropriate tool
    let result = match request.method.as_str() {
        "tools/call" => handle_tools_call(context, &request.params),
        _ => Err(mcp_error(
            rpc_error_codes::METHOD_NOT_FOUND,
            format!("Unknown method: {}", request.method),
            None,
        )),
    };

    match result {
        Ok(value) => success_response(request.id, value),
        Err(err) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(err),
        },
    }
}

/// Handles a tools/call request.
fn handle_tools_call(
    context: &CommandContext,
    params: &serde_json::Value,
) -> Result<serde_json::Value, JsonRpcError> {
    let tool_name = params.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
        mcp_error(rpc_error_codes::INVALID_PARAMS, "Missing tool name".to_string(), None)
    })?;

    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);

    info!(tool = tool_name, "Executing MCP tool");

    match tool_name {
        "lattice_create_task" => create_task(context, arguments),
        "lattice_create_document" => create_document(context, arguments),
        _ => Err(mcp_error(
            rpc_error_codes::METHOD_NOT_FOUND,
            format!("Unknown tool: {tool_name}"),
            None,
        )),
    }
}

/// Creates a task document.
fn create_task(
    context: &CommandContext,
    arguments: serde_json::Value,
) -> Result<serde_json::Value, JsonRpcError> {
    let params: CreateTaskParams = serde_json::from_value(arguments).map_err(|e| {
        mcp_error(rpc_error_codes::INVALID_PARAMS, format!("Invalid parameters: {e}"), None)
    })?;

    // Validate task type
    let task_type = parse_task_type(&params.task_type)?;

    // Validate priority
    let priority = params.priority.unwrap_or(DEFAULT_PRIORITY);
    if priority > 4 {
        return Err(mcp_error_with_data(
            rpc_error_codes::INVALID_PARAMS,
            format!("Priority must be 0-4, got {priority}"),
            error_codes::INVALID_PRIORITY,
            Some("Use a priority value between 0 (highest) and 4 (lowest)".to_string()),
        ));
    }

    // Validate description
    field_validation::validate_description_only(&params.description).map_err(|e| {
        mcp_error_with_data(
            rpc_error_codes::INVALID_PARAMS,
            e.to_string(),
            error_codes::DESCRIPTION_TOO_LONG,
            Some("Keep description under 1024 characters".to_string()),
        )
    })?;

    // Validate directory exists
    let base_dir = context.repo_root.join(params.directory.trim_end_matches('/'));
    if !base_dir.exists() {
        return Err(mcp_error_with_data(
            rpc_error_codes::INVALID_PARAMS,
            format!("Directory '{}' does not exist", params.directory),
            error_codes::INVALID_DIRECTORY,
            Some("Create the directory or use an existing path".to_string()),
        ));
    }

    // Generate new ID first (needed for filename)
    let new_id = generate_new_id(context).map_err(lattice_to_mcp_error)?;

    // Generate filename with ID suffix
    let base_filename = generate_filename_from_description(&params.description);
    let filename_with_id = format!("{}_{}", base_filename, new_id);

    // Resolve file path
    let target_dir = base_dir.join("tasks");
    let file_path =
        find_available_path(&context.conn, &context.repo_root, &target_dir, &filename_with_id)
            .map_err(lattice_to_mcp_error)?;

    // Derive name from path
    let name = field_validation::derive_name_from_path(&file_path).ok_or_else(|| {
        mcp_error(
            rpc_error_codes::INTERNAL_ERROR,
            format!("Cannot derive name from path: {}", file_path.display()),
            None,
        )
    })?;

    // Find parent ID
    let parent_id = find_parent_id(context, &file_path).map_err(lattice_to_mcp_error)?;

    // Parse blocked_by IDs
    let blocked_by = parse_id_list(&params.blocked_by)?;

    // Parse discovered_from IDs
    let discovered_from = parse_id_list(&params.discovered_from)?;

    // Build frontmatter
    let now = Utc::now();
    let frontmatter = Frontmatter {
        lattice_id: new_id.clone(),
        name: name.clone(),
        description: params.description.clone(),
        parent_id,
        task_type: Some(task_type),
        priority: Some(priority),
        labels: params.labels.clone(),
        blocking: Vec::new(),
        blocked_by,
        discovered_from,
        created_at: Some(now),
        updated_at: Some(now),
        closed_at: None,
        skill: false,
    };

    // Write document
    document_writer::write_new(
        &frontmatter,
        &params.body,
        &file_path,
        &WriteOptions::with_parents(),
    )
    .map_err(lattice_to_mcp_error)?;

    // Insert into index
    insert_into_index(context, &frontmatter, &file_path, &params.body)
        .map_err(lattice_to_mcp_error)?;

    // Build response
    let relative_path = file_path
        .strip_prefix(&context.repo_root)
        .unwrap_or(&file_path)
        .to_string_lossy()
        .to_string();

    let result = CreateResult { lattice_id: new_id.to_string(), path: relative_path, name };

    info!(id = %new_id, "Task created via MCP");

    serde_json::to_value(result).map_err(|e| {
        mcp_error(rpc_error_codes::INTERNAL_ERROR, format!("Failed to serialize result: {e}"), None)
    })
}

/// Creates a knowledge base document.
fn create_document(
    context: &CommandContext,
    arguments: serde_json::Value,
) -> Result<serde_json::Value, JsonRpcError> {
    let params: CreateDocumentParams = serde_json::from_value(arguments).map_err(|e| {
        mcp_error(rpc_error_codes::INVALID_PARAMS, format!("Invalid parameters: {e}"), None)
    })?;

    // Validate name length
    if params.name.len() > 64 {
        return Err(mcp_error_with_data(
            rpc_error_codes::INVALID_PARAMS,
            format!("Name exceeds 64 characters: {} chars", params.name.len()),
            error_codes::NAME_TOO_LONG,
            Some("Keep document name under 64 characters".to_string()),
        ));
    }

    // Validate description
    field_validation::validate_description_only(&params.description).map_err(|e| {
        mcp_error_with_data(
            rpc_error_codes::INVALID_PARAMS,
            e.to_string(),
            error_codes::DESCRIPTION_TOO_LONG,
            Some("Keep description under 1024 characters".to_string()),
        )
    })?;

    // Validate directory exists
    let base_dir = context.repo_root.join(params.directory.trim_end_matches('/'));
    if !base_dir.exists() {
        return Err(mcp_error_with_data(
            rpc_error_codes::INVALID_PARAMS,
            format!("Directory '{}' does not exist", params.directory),
            error_codes::INVALID_DIRECTORY,
            Some("Create the directory or use an existing path".to_string()),
        ));
    }

    // Normalize name for filename (convert to snake_case)
    let normalized_name = params.name.replace('-', "_").to_lowercase();

    // Resolve file path (no ID suffix for KB documents)
    let target_dir = base_dir.join("docs");
    let file_path =
        find_available_path(&context.conn, &context.repo_root, &target_dir, &normalized_name)
            .map_err(lattice_to_mcp_error)?;

    // Check if file already exists with exact name
    let exact_path = target_dir.join(format!("{normalized_name}.md"));
    if exact_path.exists() {
        return Err(mcp_error_with_data(
            rpc_error_codes::INVALID_PARAMS,
            format!("A file with the generated name already exists: {}", exact_path.display()),
            error_codes::FILE_EXISTS,
            Some("Choose a different document name".to_string()),
        ));
    }

    // Derive name from path (converts underscores to hyphens)
    let name = field_validation::derive_name_from_path(&file_path).ok_or_else(|| {
        mcp_error(
            rpc_error_codes::INTERNAL_ERROR,
            format!("Cannot derive name from path: {}", file_path.display()),
            None,
        )
    })?;

    field_validation::validate_name_only(&name).map_err(|e| {
        mcp_error_with_data(
            rpc_error_codes::INVALID_PARAMS,
            e.to_string(),
            error_codes::NAME_TOO_LONG,
            Some("Keep document name under 64 characters".to_string()),
        )
    })?;

    // Generate new ID
    let new_id = generate_new_id(context).map_err(lattice_to_mcp_error)?;

    // Find parent ID
    let parent_id = find_parent_id(context, &file_path).map_err(lattice_to_mcp_error)?;

    // Build frontmatter
    let now = Utc::now();
    let frontmatter = Frontmatter {
        lattice_id: new_id.clone(),
        name: name.clone(),
        description: params.description.clone(),
        parent_id,
        task_type: None,
        priority: None,
        labels: params.labels.clone(),
        blocking: Vec::new(),
        blocked_by: Vec::new(),
        discovered_from: Vec::new(),
        created_at: Some(now),
        updated_at: Some(now),
        closed_at: None,
        skill: false,
    };

    // Write document
    document_writer::write_new(
        &frontmatter,
        &params.body,
        &file_path,
        &WriteOptions::with_parents(),
    )
    .map_err(lattice_to_mcp_error)?;

    // Insert into index
    insert_into_index(context, &frontmatter, &file_path, &params.body)
        .map_err(lattice_to_mcp_error)?;

    // Build response
    let relative_path = file_path
        .strip_prefix(&context.repo_root)
        .unwrap_or(&file_path)
        .to_string_lossy()
        .to_string();

    let result = CreateResult { lattice_id: new_id.to_string(), path: relative_path, name };

    info!(id = %new_id, "Document created via MCP");

    serde_json::to_value(result).map_err(|e| {
        mcp_error(rpc_error_codes::INTERNAL_ERROR, format!("Failed to serialize result: {e}"), None)
    })
}

/// Parses a task type string into a TaskType enum.
fn parse_task_type(s: &str) -> Result<TaskType, JsonRpcError> {
    match s.to_lowercase().as_str() {
        "bug" => Ok(TaskType::Bug),
        "feature" => Ok(TaskType::Feature),
        "task" => Ok(TaskType::Task),
        "chore" => Ok(TaskType::Chore),
        _ => Err(mcp_error_with_data(
            rpc_error_codes::INVALID_PARAMS,
            format!("Invalid task type: {s}"),
            error_codes::INVALID_TASK_TYPE,
            Some("Task type must be one of: bug, feature, task, chore".to_string()),
        )),
    }
}

/// Parses a list of ID strings into LatticeId values.
fn parse_id_list(ids: &[String]) -> Result<Vec<LatticeId>, JsonRpcError> {
    ids.iter()
        .map(|s| {
            s.parse::<LatticeId>().map_err(|_| {
                mcp_error_with_data(
                    rpc_error_codes::INVALID_PARAMS,
                    format!("Invalid Lattice ID: {s}"),
                    error_codes::INVALID_ID,
                    Some(
                        "Lattice IDs start with 'L' followed by alphanumeric characters"
                            .to_string(),
                    ),
                )
            })
        })
        .collect()
}

/// Generates a filename from a description.
///
/// Extracts significant words, converts to lowercase with underscores,
/// caps at ~40 characters.
fn generate_filename_from_description(description: &str) -> String {
    let words: Vec<&str> = description
        .split_whitespace()
        .filter(|word| {
            let lower = word.to_lowercase();
            !SKIP_WORDS.contains(&lower.as_str())
        })
        .collect();

    let mut filename = String::new();
    for word in words {
        let cleaned: String =
            word.chars().filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-').collect();

        if cleaned.is_empty() {
            continue;
        }

        if !filename.is_empty() {
            filename.push('_');
        }
        filename.push_str(&cleaned.to_lowercase());

        if filename.len() >= MAX_FILENAME_LENGTH {
            break;
        }
    }

    if filename.is_empty() {
        filename = "untitled".to_string();
    }

    filename.truncate(MAX_FILENAME_LENGTH);
    filename
}

/// Finds an available path, appending numeric suffix on collision.
fn find_available_path(
    conn: &Connection,
    repo_root: &Path,
    target_dir: &Path,
    base_filename: &str,
) -> LatticeResult<PathBuf> {
    let mut candidate = target_dir.join(format!("{base_filename}.md"));
    let mut suffix = 2;

    while path_exists_in_index_or_filesystem(conn, repo_root, &candidate)? {
        candidate = target_dir.join(format!("{base_filename}_{suffix}.md"));
        suffix += 1;

        if suffix > 1000 {
            return Err(LatticeError::OperationNotAllowed {
                reason: format!(
                    "Too many collisions for filename {} in {}",
                    base_filename,
                    target_dir.display()
                ),
            });
        }
    }

    Ok(candidate)
}

/// Checks if a path exists either in the index or on the filesystem.
fn path_exists_in_index_or_filesystem(
    conn: &Connection,
    repo_root: &Path,
    path: &Path,
) -> LatticeResult<bool> {
    if path.exists() {
        return Ok(true);
    }

    let relative = path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().to_string();

    document_queries::exists_at_path(conn, &relative)
}

/// Generates a new unique Lattice ID.
fn generate_new_id(context: &CommandContext) -> LatticeResult<LatticeId> {
    let client_id = client_config::get_or_create_client_id(
        context.client_id_store.as_ref(),
        &context.repo_root,
    )?;

    loop {
        let counter = client_counters::get_and_increment(&context.conn, &client_id)?;
        let effective_counter = counter + INITIAL_COUNTER;
        let id = LatticeId::from_parts(effective_counter, &client_id);

        if !document_queries::exists(&context.conn, id.as_str())? {
            info!(id = %id, "Generated new Lattice ID");
            return Ok(id);
        }

        info!(id = %id, "ID collision detected, generating new ID");
    }
}

/// Finds the parent-id for a document based on directory root lookup.
fn find_parent_id(context: &CommandContext, file_path: &Path) -> LatticeResult<Option<LatticeId>> {
    let relative_path = file_path.strip_prefix(&context.repo_root).unwrap_or(file_path);

    let parent_dir = relative_path.parent().map(|p| p.to_string_lossy().to_string());

    let Some(dir_path) = parent_dir else {
        return Ok(None);
    };

    if dir_path.is_empty() {
        return Ok(None);
    }

    let lookup_dir = if dir_path.ends_with("/tasks") || dir_path.ends_with("/docs") {
        Path::new(&dir_path).parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default()
    } else {
        dir_path
    };

    if lookup_dir.is_empty() {
        return Ok(None);
    }

    if let Some(root_id) = directory_roots::get_root_id(&context.conn, &lookup_dir)? {
        let id = root_id
            .parse::<LatticeId>()
            .map_err(|_| LatticeError::MalformedId { value: root_id.clone() })?;
        return Ok(Some(id));
    }

    Ok(None)
}

/// Inserts the new document into the index.
fn insert_into_index(
    context: &CommandContext,
    frontmatter: &Frontmatter,
    file_path: &Path,
    body: &str,
) -> LatticeResult<()> {
    let relative_path = file_path
        .strip_prefix(&context.repo_root)
        .unwrap_or(file_path)
        .to_string_lossy()
        .to_string();

    let doc = InsertDocument::new(
        frontmatter.lattice_id.to_string(),
        frontmatter.parent_id.as_ref().map(LatticeId::to_string),
        relative_path,
        frontmatter.name.clone(),
        frontmatter.description.clone(),
        frontmatter.task_type,
        frontmatter.priority,
        frontmatter.created_at,
        frontmatter.updated_at,
        None,
        compute_hash(body),
        body.len() as i64,
    );

    document_queries::insert(&context.conn, &doc)?;

    for label in &frontmatter.labels {
        label_queries::add(&context.conn, frontmatter.lattice_id.as_str(), label)?;
    }

    info!(id = frontmatter.lattice_id.as_str(), "Document added to index");
    Ok(())
}

/// Computes SHA-256 hash of content for change detection.
fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Creates a success JSON-RPC response.
fn success_response(id: serde_json::Value, result: serde_json::Value) -> JsonRpcResponse {
    JsonRpcResponse { jsonrpc: "2.0".to_string(), id, result: Some(result), error: None }
}

/// Creates an error JSON-RPC response.
fn error_response(
    id: serde_json::Value,
    code: i32,
    message: &str,
    data: Option<McpErrorData>,
) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: None,
        error: Some(JsonRpcError { code, message: message.to_string(), data }),
    }
}

/// Creates an MCP error.
fn mcp_error(code: i32, message: String, data: Option<McpErrorData>) -> JsonRpcError {
    JsonRpcError { code, message, data }
}

/// Creates an MCP error with extended data.
fn mcp_error_with_data(
    code: i32,
    message: String,
    error_code: &str,
    suggestion: Option<String>,
) -> JsonRpcError {
    JsonRpcError {
        code,
        message,
        data: Some(McpErrorData { code: error_code.to_string(), suggestion }),
    }
}

/// Converts a LatticeError to a JSON-RPC error.
fn lattice_to_mcp_error(err: LatticeError) -> JsonRpcError {
    let (code, error_code) = match &err {
        LatticeError::InvalidArgument { .. } => {
            (rpc_error_codes::INVALID_PARAMS, error_codes::INVALID_DIRECTORY)
        }
        LatticeError::DocumentNotFound { .. } => {
            (rpc_error_codes::INVALID_PARAMS, error_codes::MISSING_ROOT)
        }
        LatticeError::PathAlreadyExists { .. } => {
            (rpc_error_codes::INVALID_PARAMS, error_codes::FILE_EXISTS)
        }
        _ => (rpc_error_codes::INTERNAL_ERROR, "INTERNAL_ERROR"),
    };

    mcp_error_with_data(code, err.to_string(), error_code, None)
}
