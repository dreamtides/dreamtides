use crate::error::error_types::LatticeError;

/// Decoding table: maps ASCII byte to 5-bit value, 255 = invalid.
const DECODE_TABLE: [u8; 256] = {
    let mut table = [255u8; 256];
    let mut i = 0;
    while i < 32 {
        table[BASE32_ALPHABET[i] as usize] = i as u8;
        i += 1;
    }
    table
};
/// RFC 4648 Base32 alphabet (uppercase only, no padding).
pub const BASE32_ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

/// Encodes an integer as a Base32 string with optional minimum length padding.
///
/// Uses RFC 4648 alphabet: `A-Z`, `2-7`.
///
/// # Examples
///
/// ```text
/// encode_u64(0, 1)   -> "A"
/// encode_u64(31, 1)  -> "7"
/// encode_u64(32, 1)  -> "BA"
/// encode_u64(50, 2)  -> "BS"
/// encode_u64(675, 2) -> "VD"
/// ```
pub fn encode_u64(mut value: u64, min_length: usize) -> String {
    if value == 0 {
        return "A".repeat(min_length.max(1));
    }

    let mut result = Vec::new();
    while value > 0 {
        let digit = (value % 32) as usize;
        result.push(BASE32_ALPHABET[digit]);
        value /= 32;
    }
    result.reverse();

    let mut encoded = String::from_utf8(result)
        .unwrap_or_else(|_| panic!("Base32 alphabet should always produce valid UTF-8"));

    if encoded.len() < min_length {
        let padding = "A".repeat(min_length - encoded.len());
        encoded = format!("{padding}{encoded}");
    }

    encoded
}

/// Decodes a Base32 string to an integer.
///
/// Returns `LatticeError::MalformedId` for invalid characters or overflow.
pub fn decode_u64(encoded: &str) -> Result<u64, LatticeError> {
    if encoded.is_empty() {
        return Err(LatticeError::MalformedId { value: encoded.to_string() });
    }

    let mut result: u64 = 0;
    for byte in encoded.bytes() {
        let digit = DECODE_TABLE[byte as usize];
        if digit == 255 {
            return Err(LatticeError::MalformedId { value: encoded.to_string() });
        }
        result = result
            .checked_mul(32)
            .and_then(|r| r.checked_add(digit as u64))
            .ok_or_else(|| LatticeError::MalformedId { value: encoded.to_string() })?;
    }

    Ok(result)
}

/// Validates that a string contains only valid Base32 characters.
pub fn is_valid_base32(s: &str) -> bool {
    s.bytes().all(|b| DECODE_TABLE[b as usize] != 255)
}

/// Returns the number of Base32 characters needed to encode a given value.
pub fn encoded_length(value: u64) -> usize {
    if value == 0 {
        return 1;
    }

    let mut len = 0;
    let mut v = value;
    while v > 0 {
        len += 1;
        v /= 32;
    }
    len
}
