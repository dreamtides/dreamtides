use std::collections::HashSet;

use lattice::id::lattice_id::LatticeId;

#[test]
fn parse_valid_ids() {
    assert!(LatticeId::parse("LABCDEF").is_ok());
    assert!(LatticeId::parse("LVDDTX").is_ok());
    assert!(LatticeId::parse("LBSDTX").is_ok());
    assert!(LatticeId::parse("LAAAAA").is_ok());
    assert!(LatticeId::parse("L77777").is_ok());
    assert!(LatticeId::parse("LABCDEFGHIJ").is_ok());
}

#[test]
fn parse_normalizes_case() {
    assert_eq!(LatticeId::parse("labcdef").unwrap().as_str(), "LABCDEF");
    assert_eq!(LatticeId::parse("LvDdTx").unwrap().as_str(), "LVDDTX");
}

#[test]
fn parse_trims_whitespace() {
    assert_eq!(LatticeId::parse("  LVDDTX  ").unwrap().as_str(), "LVDDTX");
}

#[test]
fn reject_too_short() {
    assert!(LatticeId::parse("LABCD").is_err());
    assert!(LatticeId::parse("L").is_err());
    assert!(LatticeId::parse("").is_err());
}

#[test]
fn reject_missing_prefix() {
    assert!(LatticeId::parse("ABCDEF").is_err());
    assert!(LatticeId::parse("K3DTXL").is_err());
}

#[test]
fn reject_invalid_chars() {
    assert!(LatticeId::parse("L01ABC").is_err()); // 0 and 1 not in Base32
    assert!(LatticeId::parse("L89ABC").is_err()); // 8 and 9 not in Base32
    assert!(LatticeId::parse("LAB!CD").is_err());
    assert!(LatticeId::parse("LAB CD").is_err());
}

#[test]
fn from_parts() {
    let id = LatticeId::from_parts(50, "DTX");
    assert_eq!(id.as_str(), "LBSDTX");

    let id = LatticeId::from_parts(675, "DTX");
    assert_eq!(id.as_str(), "LVDDTX");

    let id = LatticeId::from_parts(0, "AAA");
    assert_eq!(id.as_str(), "LAAAAA");
}

#[test]
fn counter_extraction() {
    let id = LatticeId::from_parts(675, "DTX");
    assert_eq!(id.counter_assuming_client_len(3).unwrap(), 675);

    let id = LatticeId::from_parts(50, "WXYZ");
    assert_eq!(id.counter_assuming_client_len(4).unwrap(), 50);
}

#[test]
fn client_id_extraction() {
    let id = LatticeId::from_parts(675, "DTX");
    assert_eq!(id.client_id_assuming_len(3).unwrap(), "DTX");

    let id = LatticeId::from_parts(50, "WXYZ");
    assert_eq!(id.client_id_assuming_len(4).unwrap(), "WXYZ");
}

#[test]
fn display_and_debug() {
    let id = LatticeId::from_parts(675, "DTX");
    assert_eq!(format!("{}", id), "LVDDTX");
    assert_eq!(format!("{:?}", id), "LatticeId(LVDDTX)");
}

#[test]
fn equality_and_hash() {
    let id1 = LatticeId::parse("LVDDTX").unwrap();
    let id2 = LatticeId::parse("lvddtx").unwrap();
    let id3 = LatticeId::parse("LBSDTX").unwrap();

    assert_eq!(id1, id2);
    assert_ne!(id1, id3);

    let mut set = HashSet::new();
    set.insert(id1.clone());
    assert!(set.contains(&id2));
    assert!(!set.contains(&id3));
}

#[test]
fn serde_roundtrip() {
    let id = LatticeId::from_parts(675, "DTX");
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, "\"LVDDTX\"");

    let parsed: LatticeId = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, id);
}
