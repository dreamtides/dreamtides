use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use atomic_write_file::AtomicWriteFile;
use core_data::identifiers::UserId;
use core_data::initialization_error::{ErrorCode, InitializationError};
use serde_json::error::Category;
use tracing::debug;
use {serde_json, serde_path_to_error};

use crate::save_file::SaveFile;

/// Returns the path to the save file for the given user.
pub fn save_path(dir: &Path, user_id: UserId) -> PathBuf {
    dir.join(format!("save-{}.json", user_id.0))
}

/// Reads a save file from the given directory.
pub fn read_save_from_dir(
    dir: &Path,
    user_id: UserId,
) -> Result<Option<SaveFile>, Vec<InitializationError>> {
    let file_path = save_path(dir, user_id);
    if !file_path.exists() {
        return Ok(None);
    }
    read_and_parse_save(&file_path)
}

/// Writes a save file to the given directory.
pub fn write_save_to_dir(dir: &Path, save: &SaveFile) -> Result<(), Vec<InitializationError>> {
    fs::create_dir_all(dir).map_err(|e| {
        vec![InitializationError::with_details(
            ErrorCode::IOError,
            "Failed to create save directory",
            e.to_string(),
        )]
    })?;
    let file_path = save_path(dir, save.id());
    debug!(?file_path, "Writing save file");
    let buf = serialize_save(save).map_err(|e| vec![*e])?;
    validate_serialized_json(&buf).map_err(|e| vec![*e])?;
    atomic_write(&file_path, &buf).map_err(|e| vec![*e])?;
    Ok(())
}

fn parse_with_details(
    data: &[u8],
    length_hint: Option<usize>,
) -> Result<SaveFile, Box<InitializationError>> {
    match serde_json::from_slice::<SaveFile>(data) {
        Ok(save) => Ok(save),
        Err(e) => Err(Box::new(build_parse_error(data, &e, length_hint))),
    }
}

fn build_parse_error(
    data: &[u8],
    error: &serde_json::Error,
    length_hint: Option<usize>,
) -> InitializationError {
    let line = error.line();
    let column = error.column();
    let text = String::from_utf8_lossy(data);
    let lines: Vec<&str> = text.lines().collect();
    let mut snippet = String::new();
    if line > 0 {
        let start = line.saturating_sub(5);
        let end = (line + 1).min(lines.len());
        for (idx, l) in lines[start..end].iter().enumerate() {
            let actual_line = start + idx + 1;
            let truncated = if l.len() > 240 { &l[..240] } else { l };
            snippet.push_str(&format!("{actual_line:>6} | {truncated}\n"));
            if actual_line == line {
                let caret_pos = column.min(240);
                let mut marker = String::new();
                for _ in 0..caret_pos {
                    marker.push(' ');
                }
                marker.push('^');
                snippet.push_str(&format!("       | {marker}\n"));
            }
        }
    }
    let eof_extra = if matches!(error.classify(), Category::Eof) {
        eof_context(data, length_hint)
    } else {
        String::new()
    };
    let details = format!(
        "{error} (line {} , column {})\n{snippet}{eof_extra}",
        if line == 0 { 1 } else { line },
        column + 1
    );
    InitializationError::with_details(ErrorCode::JsonError, "Failed to parse save file", details)
}

fn eof_context(data: &[u8], length_hint: Option<usize>) -> String {
    let reported = data.len();
    let mut tail_bytes = String::new();
    for b in data.iter().rev().take(32).rev() {
        let c = if b.is_ascii_graphic() || *b == b' ' { *b as char } else { '.' };
        tail_bytes.push(c);
    }
    let last_non_ws = data.iter().rposition(|b| !b.is_ascii_whitespace()).map(|i| data[i]);
    let last_char =
        last_non_ws.map(|b| (b as char).to_string()).unwrap_or_else(|| "<none>".to_string());
    let hint = length_hint.map(|l| format!(" length_hint={l}")).unwrap_or_default();
    format!(
        "EOF context: bytes_read={reported}{hint} last_non_ws={last_char} tail='{tail_bytes}'\n"
    )
}

fn read_and_parse_save(file_path: &Path) -> Result<Option<SaveFile>, Vec<InitializationError>> {
    let mut file = File::open(file_path).map_err(|e| {
        vec![InitializationError::with_details(
            ErrorCode::IOError,
            "Failed to open save file",
            e.to_string(),
        )]
    })?;
    let mut data = Vec::new();
    file.read_to_end(&mut data).map_err(|e| {
        vec![InitializationError::with_details(
            ErrorCode::IOError,
            "Failed to read save file",
            e.to_string(),
        )]
    })?;
    match parse_with_details(&data, Some(data.len())) {
        Ok(save) => Ok(Some(save)),
        Err(err) => Err(vec![*err]),
    }
}

fn validate_serialized_json(data: &[u8]) -> Result<(), Box<InitializationError>> {
    let last = data.iter().rposition(|b| !b.is_ascii_whitespace()).map(|i| data[i]);
    if let Some(b) = last
        && b != b'}'
        && b != b']'
    {
        return Err(Box::new(InitializationError::with_details(
            ErrorCode::JsonError,
            "Serialized JSON appears incomplete",
            format!("last_non_ws_byte=0x{b:02x}"),
        )));
    }
    Ok(())
}

fn atomic_write(final_path: &Path, data: &[u8]) -> Result<(), Box<InitializationError>> {
    let mut f = AtomicWriteFile::options().open(final_path).map_err(|e| {
        Box::new(InitializationError::with_details(
            ErrorCode::IOError,
            "Failed to open atomic save file",
            e.to_string(),
        ))
    })?;
    f.write_all(data).map_err(|e| {
        Box::new(InitializationError::with_details(
            ErrorCode::IOError,
            "Failed to write atomic save file",
            e.to_string(),
        ))
    })?;
    f.commit().map_err(|e| {
        Box::new(InitializationError::with_details(
            ErrorCode::IOError,
            "Failed to commit atomic save file",
            e.to_string(),
        ))
    })?;
    Ok(())
}

fn serialize_save(save: &SaveFile) -> Result<Vec<u8>, Box<InitializationError>> {
    let mut buf = Vec::new();
    let mut json = serde_json::Serializer::pretty(&mut buf);
    match serde_path_to_error::serialize(save, &mut json) {
        Ok(_) => Ok(buf),
        Err(e) => {
            let path = e.path().to_string();
            let underlying = e.inner();
            let dbg = format!("{save:?}");
            let key_hint = extract_non_string_map_key_hint(&dbg);
            let details = format!(
                "serde_json serialization error at path={path}: {underlying}; key_hint={}",
                key_hint.unwrap_or("none").replace('\n', "\\n"),
            );
            Err(Box::new(InitializationError::with_details(
                ErrorCode::JsonError,
                "Failed to serialize save file",
                details,
            )))
        }
    }
}

fn extract_non_string_map_key_hint(text: &str) -> Option<&str> {
    let needle = "{";
    if let Some(pos) = text.find(needle) {
        return text[pos..]
            .split(',')
            .find(|seg| !seg.trim_start().starts_with('"'))
            .map(|s| s.trim());
    }
    None
}
