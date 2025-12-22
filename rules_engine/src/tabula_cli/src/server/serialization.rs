use anyhow::{Context, Result};

use super::model::{
    Change, ChangedRange, HorizontalAlignment, PROTOCOL_VERSION, Request, Response, ResponseStatus,
};

pub fn parse_request(body: &[u8]) -> Result<Request> {
    let text = std::str::from_utf8(body).context("Request body is not valid UTF-8")?;
    let mut request_id = None;
    let mut workbook_path = None;
    let mut workbook_mtime = None;
    let mut workbook_size = None;
    let mut changed_range = None;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line == "TABULA/1" {
            continue;
        }

        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        if parts.len() < 2 {
            continue;
        }

        match parts[0] {
            "REQUEST_ID" => {
                request_id = Some(percent_decode(parts[1])?);
            }
            "WORKBOOK_PATH" => {
                workbook_path = Some(percent_decode(parts[1])?);
            }
            "WORKBOOK_MTIME" => {
                workbook_mtime = Some(parts[1].parse::<i64>().context("Invalid WORKBOOK_MTIME")?);
            }
            "WORKBOOK_SIZE" => {
                workbook_size = Some(parts[1].parse::<u64>().context("Invalid WORKBOOK_SIZE")?);
            }
            "CHANGED_RANGE" => {
                let range_parts: Vec<&str> = parts[1].splitn(2, ' ').collect();
                if range_parts.len() == 2 {
                    changed_range = Some(ChangedRange {
                        sheet: percent_decode(range_parts[0])?,
                        range: percent_decode(range_parts[1])?,
                    });
                }
            }
            _ => {}
        }
    }

    Ok(Request {
        request_id: request_id.context("Missing REQUEST_ID")?,
        workbook_path: workbook_path.context("Missing WORKBOOK_PATH")?,
        workbook_mtime: workbook_mtime.context("Missing WORKBOOK_MTIME")?,
        workbook_size: workbook_size.context("Missing WORKBOOK_SIZE")?,
        changed_range,
    })
}

pub fn serialize_response(response: &Response) -> String {
    let mut lines = vec![PROTOCOL_VERSION.to_string()];

    if let Some(ref request_id) = response.request_id {
        lines.push(format!("REQUEST_ID {}", percent_encode(request_id.as_str())));
    }

    match response.status {
        ResponseStatus::Ok => lines.push("STATUS ok".to_string()),
        ResponseStatus::Error => lines.push("STATUS error".to_string()),
    }

    if let Some(retry_after_ms) = response.retry_after_ms {
        lines.push(format!("RETRY_AFTER_MS {retry_after_ms}"));
    }

    for warning in &response.warnings {
        lines.push(format!("WARNING {}", percent_encode(warning)));
    }

    for change in &response.changes {
        lines.push(serialize_change(change));
    }

    if let Some(ref changeset_id) = response.changeset_id {
        lines.push(format!("CHANGESET_ID {}", percent_encode(changeset_id.as_str())));
    }

    lines.join("\n") + "\n"
}

fn serialize_change(change: &Change) -> String {
    match change {
        Change::SetBold { sheet, cell, bold } => {
            format!(
                "CHANGE set_bold {} {} {}",
                percent_encode(sheet),
                percent_encode(cell),
                if *bold { "1" } else { "0" }
            )
        }
        Change::SetBoldSpans { sheet, cell, bold, spans } => {
            let spans_str = spans
                .iter()
                .map(|s| format!("{}:{}", s.start, s.length))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "CHANGE set_bold_spans {} {} {} {}",
                percent_encode(sheet),
                percent_encode(cell),
                if *bold { "1" } else { "0" },
                spans_str
            )
        }
        Change::SetItalicSpans { sheet, cell, italic, spans } => {
            let spans_str = spans
                .iter()
                .map(|s| format!("{}:{}", s.start, s.length))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "CHANGE set_italic_spans {} {} {} {}",
                percent_encode(sheet),
                percent_encode(cell),
                if *italic { "1" } else { "0" },
                spans_str
            )
        }
        Change::SetUnderlineSpans { sheet, cell, underline, spans } => {
            let spans_str = spans
                .iter()
                .map(|s| format!("{}:{}", s.start, s.length))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "CHANGE set_underline_spans {} {} {} {}",
                percent_encode(sheet),
                percent_encode(cell),
                if *underline { "1" } else { "0" },
                spans_str
            )
        }
        Change::SetFontColorSpans { sheet, cell, rgb, spans } => {
            let spans_str = spans
                .iter()
                .map(|s| format!("{}:{}", s.start, s.length))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "CHANGE set_font_color_spans {} {} {} {}",
                percent_encode(sheet),
                percent_encode(cell),
                rgb,
                spans_str
            )
        }
        Change::SetValue { sheet, cell, value } => {
            format!(
                "CHANGE set_value {} {} {}",
                percent_encode(sheet),
                percent_encode(cell),
                percent_encode(value)
            )
        }
        Change::ClearValue { sheet, cell } => {
            format!("CHANGE clear_value {} {}", percent_encode(sheet), percent_encode(cell))
        }
        Change::SetFontColor { sheet, cell, rgb } => {
            format!(
                "CHANGE set_font_color {} {} {}",
                percent_encode(sheet),
                percent_encode(cell),
                rgb
            )
        }
        Change::SetFontSize { sheet, cell, points } => {
            format!(
                "CHANGE set_font_size {} {} {}",
                percent_encode(sheet),
                percent_encode(cell),
                points
            )
        }
        Change::SetFillColor { sheet, cell, rgb } => {
            format!(
                "CHANGE set_fill_color {} {} {}",
                percent_encode(sheet),
                percent_encode(cell),
                rgb
            )
        }
        Change::SetNumberFormat { sheet, cell, format } => {
            format!(
                "CHANGE set_number_format {} {} {}",
                percent_encode(sheet),
                percent_encode(cell),
                percent_encode(format)
            )
        }
        Change::SetHorizontalAlignment { sheet, cell, alignment } => {
            let alignment_str = match alignment {
                HorizontalAlignment::Left => "left",
                HorizontalAlignment::Center => "center",
                HorizontalAlignment::Right => "right",
            };
            format!(
                "CHANGE set_horizontal_alignment {} {} {}",
                percent_encode(sheet),
                percent_encode(cell),
                alignment_str
            )
        }
        Change::SetItalic { sheet, cell, italic } => {
            format!(
                "CHANGE set_italic {} {} {}",
                percent_encode(sheet),
                percent_encode(cell),
                if *italic { "1" } else { "0" }
            )
        }
        Change::SetUnderline { sheet, cell, underline } => {
            format!(
                "CHANGE set_underline {} {} {}",
                percent_encode(sheet),
                percent_encode(cell),
                if *underline { "1" } else { "0" }
            )
        }
        Change::SetFontNameSpans { sheet, cell, font_name, spans } => {
            let spans_str = spans
                .iter()
                .map(|s| format!("{}:{}", s.start, s.length))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "CHANGE set_font_name_spans {} {} {} {}",
                percent_encode(sheet),
                percent_encode(cell),
                percent_encode(font_name),
                spans_str
            )
        }
        Change::SetFontSizeSpans { sheet, cell, points, spans } => {
            let spans_str = spans
                .iter()
                .map(|s| format!("{}:{}", s.start, s.length))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "CHANGE set_font_size_spans {} {} {} {}",
                percent_encode(sheet),
                percent_encode(cell),
                points,
                spans_str
            )
        }
        Change::SetSubscriptSpans { sheet, cell, subscript, spans } => {
            let spans_str = spans
                .iter()
                .map(|s| format!("{}:{}", s.start, s.length))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "CHANGE set_subscript_spans {} {} {} {}",
                percent_encode(sheet),
                percent_encode(cell),
                if *subscript { "1" } else { "0" },
                spans_str
            )
        }
    }
}

fn percent_encode(s: &str) -> String {
    let mut encoded = String::new();
    for byte in s.bytes() {
        match byte {
            b' ' => encoded.push_str("%20"),
            b'%' => encoded.push_str("%25"),
            b'\n' => encoded.push_str("%0A"),
            b'\r' => encoded.push_str("%0D"),
            _ if byte.is_ascii() && !byte.is_ascii_control() => encoded.push(byte as char),
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

fn percent_decode(s: &str) -> Result<String> {
    let mut decoded = Vec::new();
    let mut bytes = s.bytes();
    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            let hex1 = bytes.next().context("Incomplete percent-encoding")?;
            let hex2 = bytes.next().context("Incomplete percent-encoding")?;
            let hex_str = format!("{}{}", hex1 as char, hex2 as char);
            let decoded_byte = u8::from_str_radix(&hex_str, 16)
                .with_context(|| format!("Invalid hex in percent-encoding: {hex_str}"))?;
            decoded.push(decoded_byte);
        } else {
            decoded.push(byte);
        }
    }
    String::from_utf8(decoded).context("Decoded string is not valid UTF-8")
}

pub fn compute_changeset_id(
    workbook_path: &str,
    workbook_mtime: i64,
    workbook_size: u64,
    changes: &[Change],
) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(workbook_path.as_bytes());
    hasher.update(workbook_mtime.to_le_bytes());
    hasher.update(workbook_size.to_le_bytes());
    for change in changes {
        hasher.update(format!("{change:?}").as_bytes());
    }
    let hash = hasher.finalize();
    format!("{hash:x}")
}
