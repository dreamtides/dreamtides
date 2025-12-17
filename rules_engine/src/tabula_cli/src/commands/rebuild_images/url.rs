use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use calamine::{self, Data, Range, Reader, Xlsx};
use reqwest::blocking::Client;
use reqwest::header::{
    ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, HeaderMap, HeaderValue, REFERER, USER_AGENT,
};
use roxmltree::Document;

use super::{FileRecord, read_zip, write_zip};

const MAIN_NS: &str = "http://schemas.openxmlformats.org/spreadsheetml/2006/main";
const REL_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const PACKAGE_REL_NS: &str = "http://schemas.openxmlformats.org/package/2006/relationships";
const WEB_IMAGE_NS: &str =
    "http://schemas.microsoft.com/office/spreadsheetml/2020/richdatawebimage";
const RICH_DATA_NS: &str = "http://schemas.microsoft.com/office/spreadsheetml/2017/richdata";

#[derive(Clone)]
struct WebImage {
    address_rid: String,
    image_path: String,
}

#[derive(Clone)]
struct Relationship {
    id: String,
    target: String,
    type_name: String,
    mode: Option<String>,
}

struct SheetInfo {
    name: String,
    path: String,
}

struct ImageCell {
    sheet: String,
    cell_ref: String,
    identifier: usize,
    formula: String,
    base_col: u32,
    base_row: u32,
    cell_col: u32,
    cell_row: u32,
}

pub fn rebuild_from_urls(source: &Path) -> Result<()> {
    let (records, file_order) = read_zip(source).with_context(|| {
        format!("File {path} is not a valid XLSM archive", path = source.display())
    })?;
    let mut record_map: BTreeMap<String, FileRecord> =
        records.into_iter().map(|record| (record.name.clone(), record)).collect();
    let mut warnings = Vec::new();

    let workbook_rels = require_record(&record_map, "xl/_rels/workbook.xml.rels")?;
    let workbook_xml = require_record(&record_map, "xl/workbook.xml")?;
    let sheets = parse_sheets(workbook_xml, workbook_rels)?;

    let metadata_xml = require_record(&record_map, "xl/metadata.xml")?;
    let rich_values_xml = require_record(&record_map, "xl/richData/rdrichvalue.xml")?;
    let vm_to_identifier = vm_identifier_map(metadata_xml, rich_values_xml)?;

    let web_image_xml = require_record(&record_map, "xl/richData/rdRichValueWebImage.xml")?;
    let web_image_rels_xml =
        require_record(&record_map, "xl/richData/_rels/rdRichValueWebImage.xml.rels")?;
    let mut relationships = parse_relationships(web_image_rels_xml)?;
    let web_images = parse_web_images(web_image_xml, &relationships)?;

    let cells = parse_image_cells(&sheets, &vm_to_identifier, &record_map, &mut warnings)?;
    if cells.is_empty() {
        bail!("No IMAGE() formulas with metadata were found");
    }

    let identifier_urls = compute_urls(source, &cells, &mut warnings)?;
    for idx in 0..web_images.len() {
        if !identifier_urls.contains_key(&idx) {
            warnings.push(format!("Identifier {idx} had no IMAGE() URL; skipped"));
        }
    }

    let downloads = download_images(&identifier_urls, &mut warnings)?;
    if !warnings.is_empty() {
        for warning in &warnings {
            eprintln!("warning: {warning}");
        }
    }

    if downloads.is_empty() {
        bail!("No images were downloaded from IMAGE() formulas");
    }

    apply_downloads(&mut record_map, &web_images, &downloads, &mut warnings)?;
    update_relationship_targets(&mut relationships, &web_images, &identifier_urls);
    write_relationships(&mut record_map, relationships)?;

    write_zip(source, record_map.into_values().collect(), &file_order)
}

fn require_record<'a>(records: &'a BTreeMap<String, FileRecord>, name: &str) -> Result<&'a [u8]> {
    records
        .get(name)
        .map(|record| record.data.as_slice())
        .ok_or_else(|| anyhow::anyhow!("Spreadsheet is missing entry {}", name))
}

fn parse_doc(xml: &[u8]) -> Result<Document> {
    let text = std::str::from_utf8(xml).context("XML data is not valid UTF-8")?;
    Document::parse(text).context("Failed to parse XML")
}

fn parse_sheets(workbook_xml: &[u8], workbook_rels: &[u8]) -> Result<Vec<SheetInfo>> {
    let relationships = parse_relationships(workbook_rels)?;
    let mut rel_map = HashMap::new();
    for rel in relationships {
        rel_map.insert(rel.id, rel.target);
    }

    let doc = parse_doc(workbook_xml)?;
    let mut sheets = Vec::new();
    for node in doc.descendants().filter(|n| n.has_tag_name((MAIN_NS, "sheet"))) {
        let name = node.attribute("name").context("Sheet entry missing name")?;
        let rel_id =
            node.attribute((REL_NS, "id")).context("Sheet entry missing relationship id")?;
        let target = rel_map
            .get(rel_id)
            .with_context(|| format!("Workbook relationship {rel_id} is missing"))?;
        let path = Path::new("xl").join(target);
        sheets.push(SheetInfo { name: name.to_string(), path: path.to_string_lossy().to_string() });
    }
    Ok(sheets)
}

fn parse_relationships(xml: &[u8]) -> Result<Vec<Relationship>> {
    let doc = parse_doc(xml)?;
    let mut relationships = Vec::new();
    for node in doc.descendants().filter(|n| n.has_tag_name((PACKAGE_REL_NS, "Relationship"))) {
        let id = node.attribute("Id").context("Relationship missing Id")?.to_string();
        let target = node.attribute("Target").context("Relationship missing Target")?.to_string();
        let type_name = node.attribute("Type").context("Relationship missing Type")?.to_string();
        relationships.push(Relationship {
            id,
            target,
            type_name,
            mode: node.attribute("TargetMode").map(|s| s.to_string()),
        });
    }
    Ok(relationships)
}

fn parse_web_images(xml: &[u8], relationships: &[Relationship]) -> Result<Vec<WebImage>> {
    let rel_map: HashMap<_, _> = relationships.iter().map(|rel| (rel.id.as_str(), rel)).collect();
    let doc = parse_doc(xml)?;
    let mut images = Vec::new();
    for node in doc.descendants().filter(|n| n.has_tag_name((WEB_IMAGE_NS, "webImageSrd"))) {
        let address = node
            .children()
            .find(|child| child.has_tag_name((WEB_IMAGE_NS, "address")))
            .context("webImageSrd missing address")?;
        let blip = node
            .children()
            .find(|child| child.has_tag_name((WEB_IMAGE_NS, "blip")))
            .context("webImageSrd missing blip")?;
        let address_rid = address.attribute((REL_NS, "id")).context("address missing r:id")?;
        let blip_rid = blip.attribute((REL_NS, "id")).context("blip missing r:id")?;
        let image_target = rel_map
            .get(blip_rid)
            .with_context(|| format!("Missing relationship {blip_rid}"))?
            .target
            .clone();
        images.push(WebImage {
            address_rid: address_rid.to_string(),
            image_path: normalize_media_path(&image_target)?,
        });
    }
    Ok(images)
}

fn normalize_media_path(target: &str) -> Result<String> {
    let path = Path::new("xl/richData").join(target);
    let normalized = path.components().fold(PathBuf::new(), |mut acc, comp| {
        if comp == std::path::Component::ParentDir {
            acc.pop();
        } else {
            acc.push(comp);
        }
        acc
    });
    Ok(normalized.to_string_lossy().to_string())
}

fn vm_identifier_map(metadata_xml: &[u8], rich_values_xml: &[u8]) -> Result<Vec<usize>> {
    let identifiers = parse_rich_value_identifiers(rich_values_xml)?;
    let doc = parse_doc(metadata_xml)?;

    let future = doc
        .descendants()
        .find(|n| n.has_tag_name((MAIN_NS, "futureMetadata")))
        .context("metadata.xml is missing futureMetadata")?;
    let mut future_to_rv = Vec::new();
    for bk in future.children().filter(|n| n.has_tag_name((MAIN_NS, "bk"))) {
        let rvb = bk
            .descendants()
            .find(|n| n.has_tag_name((RICH_DATA_NS, "rvb")))
            .context("futureMetadata entry missing rvb")?;
        let idx = rvb
            .attribute("i")
            .context("rvb missing index")?
            .parse::<usize>()
            .context("rvb index is invalid")?;
        future_to_rv.push(idx);
    }

    let value_meta = doc
        .descendants()
        .find(|n| n.has_tag_name((MAIN_NS, "valueMetadata")))
        .context("metadata.xml is missing valueMetadata")?;
    let mut vm_to_identifier = Vec::new();
    for bk in value_meta.children().filter(|n| n.has_tag_name((MAIN_NS, "bk"))) {
        let rc = bk
            .children()
            .find(|n| n.has_tag_name((MAIN_NS, "rc")))
            .context("valueMetadata entry missing rc")?;
        let future_idx = rc
            .attribute("v")
            .context("valueMetadata rc missing v")?
            .parse::<usize>()
            .context("valueMetadata index is invalid")?;
        let rv_idx = future_to_rv
            .get(future_idx)
            .with_context(|| format!("Future metadata {future_idx} is out of range"))?;
        let identifier = identifiers
            .get(*rv_idx)
            .with_context(|| format!("Rich value {rv_idx} is out of range"))?;
        vm_to_identifier.push(*identifier);
    }

    Ok(vm_to_identifier)
}

fn parse_rich_value_identifiers(xml: &[u8]) -> Result<Vec<usize>> {
    let doc = parse_doc(xml)?;
    let mut identifiers = Vec::new();
    for rv in doc.descendants().filter(|n| n.has_tag_name((RICH_DATA_NS, "rv"))) {
        let first = rv
            .children()
            .find(|child| child.has_tag_name((RICH_DATA_NS, "v")))
            .context("Rich value missing identifier")?;
        let id = first
            .text()
            .context("Rich value identifier missing text")?
            .parse::<usize>()
            .context("Rich value identifier is invalid")?;
        identifiers.push(id);
    }
    Ok(identifiers)
}

fn parse_image_cells(
    sheets: &[SheetInfo],
    vm_to_identifier: &[usize],
    records: &BTreeMap<String, FileRecord>,
    warnings: &mut Vec<String>,
) -> Result<Vec<ImageCell>> {
    let mut cells = Vec::new();
    for sheet in sheets {
        let xml = require_record(records, sheet.path.as_str())?;
        let doc = parse_doc(xml)?;
        let mut shared_formulas = HashMap::new();
        for node in doc.descendants().filter(|n| n.has_tag_name((MAIN_NS, "c"))) {
            if let Some(formula) = node.children().find(|child| child.has_tag_name((MAIN_NS, "f")))
            {
                if let Some(si) = formula.attribute("si") {
                    if let Some(text) = formula.text() {
                        let anchor = node.attribute("r").context("Cell missing reference")?;
                        shared_formulas
                            .insert(si.to_string(), (text.to_string(), anchor.to_string()));
                    }
                }
            }
        }

        for node in doc.descendants().filter(|n| n.has_tag_name((MAIN_NS, "c"))) {
            let cell_ref = match node.attribute("r") {
                Some(r) => r,
                None => continue,
            };
            let vm_index = match node.attribute("vm") {
                Some(value) => {
                    let raw = value.parse::<usize>().context("Value metadata index is invalid")?;
                    match raw.checked_sub(1) {
                        Some(index) => index,
                        None => {
                            warnings.push(format!(
                                "Value metadata index {raw} is invalid for {}!{}",
                                sheet.name, cell_ref
                            ));
                            continue;
                        }
                    }
                }
                None => continue,
            };
            let identifier = match vm_to_identifier.get(vm_index) {
                Some(value) => *value,
                None => {
                    warnings.push(format!(
                        "Value metadata {} is missing for {}!{} ({} entries present)",
                        vm_index + 1,
                        sheet.name,
                        cell_ref,
                        vm_to_identifier.len()
                    ));
                    continue;
                }
            };
            let formula_node =
                match node.children().find(|child| child.has_tag_name((MAIN_NS, "f"))) {
                    Some(f) => f,
                    None => continue,
                };
            let raw_formula = formula_node.text().unwrap_or("").to_string();
            let (formula_text, base_ref) = if raw_formula.trim().is_empty() {
                let si = match formula_node.attribute("si") {
                    Some(value) => value,
                    None => {
                        warnings.push(format!(
                            "Shared formula missing si for {}!{}",
                            sheet.name, cell_ref
                        ));
                        continue;
                    }
                };
                match shared_formulas.get(si) {
                    Some(value) => value.clone(),
                    None => {
                        warnings.push(format!(
                            "Shared formula {si} is missing base text for {}!{}",
                            sheet.name, cell_ref
                        ));
                        continue;
                    }
                }
            } else {
                (raw_formula, cell_ref.to_string())
            };

            if !formula_text.to_uppercase().contains("IMAGE(") {
                continue;
            }

            let (cell_col, cell_row) = match parse_cell_position(cell_ref) {
                Ok(pos) => pos,
                Err(err) => {
                    warnings.push(format!("{} in {}!{}", err, sheet.name, cell_ref));
                    continue;
                }
            };
            let (base_col, base_row) = match parse_cell_position(&base_ref) {
                Ok(pos) => pos,
                Err(err) => {
                    warnings.push(format!(
                        "{} in {}!{} (shared base {})",
                        err, sheet.name, cell_ref, base_ref
                    ));
                    continue;
                }
            };
            cells.push(ImageCell {
                sheet: sheet.name.clone(),
                cell_ref: cell_ref.to_string(),
                identifier,
                formula: formula_text,
                base_col,
                base_row,
                cell_col,
                cell_row,
            });
        }
    }
    Ok(cells)
}

fn compute_urls(
    path: &Path,
    cells: &[ImageCell],
    warnings: &mut Vec<String>,
) -> Result<BTreeMap<usize, String>> {
    let mut workbook: Xlsx<_> = calamine::open_workbook(path)
        .with_context(|| format!("Cannot open spreadsheet at {}", path.display()))?;
    let mut ranges: HashMap<String, Range<Data>> = HashMap::new();
    let mut identifier_urls = BTreeMap::new();

    for cell in cells {
        if !ranges.contains_key(&cell.sheet) {
            let range = match workbook.worksheet_range(&cell.sheet) {
                Ok(value) => value,
                Err(err) => {
                    warnings.push(format!("Failed to read sheet {}: {}", cell.sheet, err));
                    continue;
                }
            };
            ranges.insert(cell.sheet.clone(), range);
        }
        let range = match ranges.get(&cell.sheet) {
            Some(value) => value,
            None => continue,
        };
        let row_offset = cell.cell_row as i32 - cell.base_row as i32;
        let col_offset = cell.cell_col as i32 - cell.base_col as i32;
        let url = match evaluate_formula(&cell.formula, range, row_offset, col_offset) {
            Ok(value) => value,
            Err(err) => {
                warnings.push(format!(
                    "Cannot parse IMAGE() formula in {}!{}: {}",
                    cell.sheet, cell.cell_ref, err
                ));
                continue;
            }
        };
        if let Some(existing) = identifier_urls.get(&cell.identifier) {
            if existing != &url {
                warnings.push(format!(
                    "IMAGE() formulas for identifier {} disagree: '{}' vs '{}'; skipping {}!{}",
                    cell.identifier, existing, url, cell.sheet, cell.cell_ref
                ));
            }
        } else {
            identifier_urls.insert(cell.identifier, url);
        }
    }

    Ok(identifier_urls)
}

fn evaluate_formula(
    formula: &str,
    range: &Range<Data>,
    row_offset: i32,
    col_offset: i32,
) -> Result<String> {
    let argument = first_image_argument(formula)?;
    let parts = parse_expression_parts(&argument)?;
    let mut output = String::new();
    for part in parts {
        match part {
            ExprPart::Literal(text) => output.push_str(&text),
            ExprPart::Reference(reference) => {
                let col = reference.col + if reference.col_abs { 0 } else { col_offset };
                let row = reference.row + if reference.row_abs { 0 } else { row_offset };
                if row < 1 || col < 1 {
                    bail!("IMAGE() formula shifted a reference before A1");
                }
                let value = cell_value(range, row as usize - 1, col as usize - 1)?;
                output.push_str(&value);
            }
        }
    }
    Ok(output)
}

fn first_image_argument(formula: &str) -> Result<String> {
    let upper = formula.to_uppercase();
    let start =
        upper.find("IMAGE(").ok_or_else(|| anyhow::anyhow!("Formula does not contain IMAGE()"))?;
    let mut depth = 0i32;
    let mut in_quotes = false;
    let mut argument = String::new();
    for ch in formula[start + 6..].chars() {
        if ch == '"' {
            in_quotes = !in_quotes;
            argument.push(ch);
            continue;
        }
        if !in_quotes {
            if ch == '(' {
                depth += 1;
            } else if ch == ')' {
                if depth == 0 {
                    break;
                }
                depth -= 1;
            } else if ch == ',' && depth == 0 {
                break;
            }
        }
        argument.push(ch);
    }
    Ok(argument.trim().to_string())
}

enum ExprPart {
    Literal(String),
    Reference(CellReference),
}

struct CellReference {
    col: i32,
    row: i32,
    col_abs: bool,
    row_abs: bool,
}

fn parse_expression_parts(expr: &str) -> Result<Vec<ExprPart>> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut depth = 0i32;

    for ch in expr.chars() {
        if ch == '"' {
            in_quotes = !in_quotes;
            current.push(ch);
            continue;
        }
        if !in_quotes {
            if ch == '(' {
                depth += 1;
            } else if ch == ')' {
                depth -= 1;
            }
            if ch == '&' && depth == 0 {
                if !current.trim().is_empty() {
                    parts.push(parse_part(current.trim())?);
                }
                current.clear();
                continue;
            }
        }
        current.push(ch);
    }
    if !current.trim().is_empty() {
        parts.push(parse_part(current.trim())?);
    }
    Ok(parts)
}

fn parse_part(text: &str) -> Result<ExprPart> {
    if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
        let inner = &text[1..text.len() - 1];
        let literal = inner.replace("\"\"", "\"");
        return Ok(ExprPart::Literal(literal));
    }
    Ok(ExprPart::Reference(parse_reference(text)?))
}

fn parse_reference(text: &str) -> Result<CellReference> {
    let mut chars = text.chars().peekable();
    let mut col_abs = false;
    let mut row_abs = false;
    if matches!(chars.peek(), Some('$')) {
        col_abs = true;
        chars.next();
    }
    let mut col_letters = String::new();
    while let Some(ch) = chars.peek() {
        if ch.is_ascii_alphabetic() {
            col_letters.push(ch.to_ascii_uppercase());
            chars.next();
        } else {
            break;
        }
    }
    if matches!(chars.peek(), Some('$')) {
        row_abs = true;
        chars.next();
    }
    let mut row_digits = String::new();
    while let Some(ch) = chars.peek() {
        if ch.is_ascii_digit() {
            row_digits.push(*ch);
            chars.next();
        } else {
            break;
        }
    }
    if col_letters.is_empty() || row_digits.is_empty() {
        bail!("IMAGE() formula contains an unsupported reference '{}'", text);
    }
    let col = column_index(&col_letters)?;
    let row = row_digits.parse::<i32>().context("Row index is invalid")?;
    Ok(CellReference { col, row, col_abs, row_abs })
}

fn column_index(text: &str) -> Result<i32> {
    let mut value = 0i32;
    for ch in text.chars() {
        if !ch.is_ascii_uppercase() {
            bail!("Invalid column {text}");
        }
        value = value * 26 + (ch as i32 - 'A' as i32) + 1;
    }
    Ok(value)
}

fn cell_value(range: &Range<Data>, row: usize, col: usize) -> Result<String> {
    let cell = range.get((row, col)).ok_or_else(|| anyhow::anyhow!("Cell is out of bounds"))?;
    match cell {
        Data::Empty => bail!("Cell is empty"),
        Data::String(s) => Ok(s.clone()),
        Data::Float(f) => {
            if f.fract() == 0.0 {
                Ok((*f as i64).to_string())
            } else {
                Ok(f.to_string())
            }
        }
        Data::Int(i) => Ok(i.to_string()),
        Data::Bool(b) => Ok(b.to_string()),
        Data::DateTime(dt) => Ok(dt.as_f64().to_string()),
        Data::DateTimeIso(s) => Ok(s.clone()),
        Data::DurationIso(s) => Ok(s.clone()),
        Data::Error(e) => bail!("Cell contains error {e:?}"),
    }
}

fn parse_cell_position(cell_ref: &str) -> Result<(u32, u32)> {
    let reference = parse_reference(cell_ref)?;
    if reference.col < 1 || reference.row < 1 {
        bail!("Cell reference {} is invalid", cell_ref);
    }
    Ok((reference.col as u32, reference.row as u32))
}

fn download_images(
    urls: &BTreeMap<usize, String>,
    warnings: &mut Vec<String>,
) -> Result<BTreeMap<usize, Vec<u8>>> {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
        ),
    );
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("image/avif,image/webp,image/apng,image/*,*/*;q=0.8"),
    );
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
    headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip, deflate, br"));
    headers.insert(REFERER, HeaderValue::from_static("https://www.shutterstock.com/"));
    let client = Client::builder()
        .default_headers(headers)
        .build()
        .context("Failed to build HTTP client")?;
    let mut results = BTreeMap::new();
    for (identifier, url) in urls {
        let response = match client.get(url).send() {
            Ok(value) => value,
            Err(err) => {
                warnings.push(format!("Failed to download image from {url}: {err}"));
                continue;
            }
        };
        let status = response.status();
        if !status.is_success() {
            warnings.push(format!("Failed to download image from {url} (status {status})"));
            continue;
        }
        let data = match response.bytes() {
            Ok(value) => value,
            Err(err) => {
                warnings.push(format!("Failed to read image body from {url}: {err}"));
                continue;
            }
        };
        results.insert(*identifier, data.to_vec());
        thread::sleep(Duration::from_millis(200));
    }
    Ok(results)
}

fn apply_downloads(
    records: &mut BTreeMap<String, FileRecord>,
    web_images: &[WebImage],
    downloads: &BTreeMap<usize, Vec<u8>>,
    warnings: &mut Vec<String>,
) -> Result<()> {
    for (identifier, data) in downloads {
        let image = match web_images.get(*identifier) {
            Some(value) => value,
            None => {
                warnings.push(format!("No web image entry for identifier {identifier}"));
                continue;
            }
        };
        let record = match records.get_mut(&image.image_path) {
            Some(value) => value,
            None => {
                warnings.push(format!(
                    "Spreadsheet is missing image entry {path}",
                    path = image.image_path
                ));
                continue;
            }
        };
        record.data = data.clone();
    }
    Ok(())
}

fn update_relationship_targets(
    relationships: &mut [Relationship],
    web_images: &[WebImage],
    identifier_urls: &BTreeMap<usize, String>,
) {
    let mut address_targets = HashMap::new();
    for (identifier, url) in identifier_urls {
        if let Some(web) = web_images.get(*identifier) {
            address_targets.insert(web.address_rid.as_str(), url.as_str());
        }
    }
    for rel in relationships {
        if rel.type_name.ends_with("/hyperlink") {
            if let Some(url) = address_targets.get(rel.id.as_str()) {
                rel.target = (*url).to_string();
            }
        }
    }
}

fn write_relationships(
    records: &mut BTreeMap<String, FileRecord>,
    relationships: Vec<Relationship>,
) -> Result<()> {
    let xml = render_relationships(&relationships);
    let record =
        records.get_mut("xl/richData/_rels/rdRichValueWebImage.xml.rels").with_context(|| {
            "Spreadsheet is missing xl/richData/_rels/rdRichValueWebImage.xml.rels".to_string()
        })?;
    record.data = xml.into_bytes();
    Ok(())
}

fn render_relationships(relationships: &[Relationship]) -> String {
    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
    xml.push_str(
        r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
    );
    for rel in relationships {
        xml.push_str("<Relationship");
        xml.push_str(&format!(
            r#" Id="{}" Type="{}" Target="{}""#,
            xml_escape(&rel.id),
            xml_escape(&rel.type_name),
            xml_escape(&rel.target)
        ));
        if let Some(mode) = &rel.mode {
            xml.push_str(&format!(r#" TargetMode="{}""#, xml_escape(mode)));
        }
        xml.push_str("/>");
    }
    xml.push_str("</Relationships>");
    xml
}

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
