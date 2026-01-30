use std::path::PathBuf;

use tabula_data::card_list_row::{CardListType, build_card_list_row};
use tabula_data::toml_loader::CardListRowRaw;

fn test_file() -> PathBuf {
    PathBuf::from("test.toml")
}

fn raw_base_card_list() -> CardListRowRaw {
    CardListRowRaw {
        list_name: "Core 11".to_string(),
        list_type: "BaseCardId".to_string(),
        card_id: "36c2a4e1-3212-4933-979a-73f109f9b256".to_string(),
        copies: 8,
    }
}

fn raw_dreamwell_card_list() -> CardListRowRaw {
    CardListRowRaw {
        list_name: "Dreamwell Basic 5".to_string(),
        list_type: "DreamwellCardId".to_string(),
        card_id: "bc143c3c-f149-4506-813f-1aa8dd54e370".to_string(),
        copies: 1,
    }
}

#[test]
fn build_base_card_list_row_succeeds() {
    let raw = raw_base_card_list();
    let result = build_card_list_row(&raw, &test_file());

    assert!(result.is_ok());
    let row = result.unwrap();
    assert_eq!(row.list_name, "Core 11");
    assert_eq!(row.list_type, CardListType::BaseCardId);
    assert_eq!(row.copies, 8);
    assert!(!row.card_id.is_nil());
}

#[test]
fn build_dreamwell_card_list_row_succeeds() {
    let raw = raw_dreamwell_card_list();
    let result = build_card_list_row(&raw, &test_file());

    assert!(result.is_ok());
    let row = result.unwrap();
    assert_eq!(row.list_name, "Dreamwell Basic 5");
    assert_eq!(row.list_type, CardListType::DreamwellCardId);
    assert_eq!(row.copies, 1);
    assert!(!row.card_id.is_nil());
}

#[test]
fn build_card_list_row_invalid_uuid_fails() {
    let mut raw = raw_base_card_list();
    raw.card_id = "not-a-uuid".to_string();

    let result = build_card_list_row(&raw, &test_file());
    assert!(result.is_err());
}

#[test]
fn build_card_list_row_invalid_list_type_fails() {
    let mut raw = raw_base_card_list();
    raw.list_type = "InvalidType".to_string();

    let result = build_card_list_row(&raw, &test_file());
    assert!(result.is_err());
}

#[test]
fn build_card_list_row_preserves_copies() {
    let mut raw = raw_base_card_list();
    raw.copies = 42;

    let result = build_card_list_row(&raw, &test_file());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().copies, 42);
}
