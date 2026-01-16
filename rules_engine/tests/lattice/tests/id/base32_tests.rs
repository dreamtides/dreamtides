use lattice::id::base32_encoding;

#[test]
fn encode_zero() {
    assert_eq!(base32_encoding::encode_u64(0, 1), "A");
    assert_eq!(base32_encoding::encode_u64(0, 2), "AA");
    assert_eq!(base32_encoding::encode_u64(0, 3), "AAA");
}

#[test]
fn encode_single_digit() {
    assert_eq!(base32_encoding::encode_u64(0, 1), "A");
    assert_eq!(base32_encoding::encode_u64(1, 1), "B");
    assert_eq!(base32_encoding::encode_u64(25, 1), "Z");
    assert_eq!(base32_encoding::encode_u64(26, 1), "2");
    assert_eq!(base32_encoding::encode_u64(31, 1), "7");
}

#[test]
fn encode_multi_digit() {
    assert_eq!(base32_encoding::encode_u64(32, 1), "BA");
    assert_eq!(base32_encoding::encode_u64(33, 1), "BB");
    assert_eq!(base32_encoding::encode_u64(50, 2), "BS");
    assert_eq!(base32_encoding::encode_u64(675, 2), "VD");
    assert_eq!(base32_encoding::encode_u64(1023, 1), "77");
    assert_eq!(base32_encoding::encode_u64(1024, 1), "BAA");
}

#[test]
fn encode_with_padding() {
    assert_eq!(base32_encoding::encode_u64(1, 2), "AB");
    assert_eq!(base32_encoding::encode_u64(1, 3), "AAB");
    assert_eq!(base32_encoding::encode_u64(32, 3), "ABA");
}

#[test]
fn decode_single_digit() {
    assert_eq!(base32_encoding::decode_u64("A").unwrap(), 0);
    assert_eq!(base32_encoding::decode_u64("B").unwrap(), 1);
    assert_eq!(base32_encoding::decode_u64("Z").unwrap(), 25);
    assert_eq!(base32_encoding::decode_u64("2").unwrap(), 26);
    assert_eq!(base32_encoding::decode_u64("7").unwrap(), 31);
}

#[test]
fn decode_multi_digit() {
    assert_eq!(base32_encoding::decode_u64("BA").unwrap(), 32);
    assert_eq!(base32_encoding::decode_u64("BS").unwrap(), 50);
    assert_eq!(base32_encoding::decode_u64("VD").unwrap(), 675);
    assert_eq!(base32_encoding::decode_u64("AAA").unwrap(), 0);
    assert_eq!(base32_encoding::decode_u64("BAA").unwrap(), 1024);
}

#[test]
fn roundtrip() {
    for value in [0, 1, 31, 32, 50, 675, 1023, 1024, 32767, 1_000_000] {
        let encoded = base32_encoding::encode_u64(value, 1);
        let decoded = base32_encoding::decode_u64(&encoded).unwrap();
        assert_eq!(decoded, value, "roundtrip failed for {value}");
    }
}

#[test]
fn decode_invalid_chars() {
    assert!(base32_encoding::decode_u64("0").is_err());
    assert!(base32_encoding::decode_u64("1").is_err());
    assert!(base32_encoding::decode_u64("8").is_err());
    assert!(base32_encoding::decode_u64("9").is_err());
    assert!(base32_encoding::decode_u64("a").is_err());
    assert!(base32_encoding::decode_u64("!").is_err());
    assert!(base32_encoding::decode_u64("AB0").is_err());
}

#[test]
fn decode_empty() {
    assert!(base32_encoding::decode_u64("").is_err());
}

#[test]
fn validate_base32() {
    assert!(base32_encoding::is_valid_base32("ABCDEFGHIJKLMNOPQRSTUVWXYZ234567"));
    assert!(base32_encoding::is_valid_base32("VDDTX"));
    assert!(base32_encoding::is_valid_base32(""));
    assert!(!base32_encoding::is_valid_base32("0"));
    assert!(!base32_encoding::is_valid_base32("1"));
    assert!(!base32_encoding::is_valid_base32("a"));
    assert!(!base32_encoding::is_valid_base32("AB!"));
}

#[test]
fn test_encoded_length() {
    assert_eq!(base32_encoding::encoded_length(0), 1);
    assert_eq!(base32_encoding::encoded_length(31), 1);
    assert_eq!(base32_encoding::encoded_length(32), 2);
    assert_eq!(base32_encoding::encoded_length(1023), 2);
    assert_eq!(base32_encoding::encoded_length(1024), 3);
}
