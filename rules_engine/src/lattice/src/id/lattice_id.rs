use std::fmt;
use std::str::FromStr;

use crate::error::error_types::LatticeError;
use crate::id::base32_encoding;

/// Required prefix for all Lattice IDs.
pub const ID_PREFIX: char = 'L';

/// Minimum length for a valid Lattice ID (L + 2 counter + 3 client = 6).
pub const MIN_ID_LENGTH: usize = 6;

/// A validated Lattice document identifier.
///
/// Format: `L` prefix + Base32 document counter (min 2 chars) + Base32 client
/// ID (3-6 chars). Example: `LVDDTX` where `VD` is counter 675 and `DTX` is the
/// client ID.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct LatticeId {
    /// The complete ID string, always uppercase.
    value: String,
}

impl LatticeId {
    /// Creates a new `LatticeId` from validated components.
    ///
    /// This is an internal constructor used by ID generation. For parsing user
    /// input, use `LatticeId::parse` or `FromStr`.
    pub fn from_parts(counter: u64, client_id: &str) -> Self {
        let counter_encoded = base32_encoding::encode_u64(counter, 2);
        let value = format!("{ID_PREFIX}{counter_encoded}{client_id}");
        Self { value }
    }

    /// Parses and validates a Lattice ID string.
    ///
    /// # Errors
    ///
    /// Returns `LatticeError::MalformedId` if:
    /// - Missing `L` prefix
    /// - Fewer than 6 characters
    /// - Contains invalid Base32 characters
    pub fn parse(input: &str) -> Result<Self, LatticeError> {
        let input = input.trim();

        if input.len() < MIN_ID_LENGTH {
            tracing::debug!(id = input, "ID too short, minimum {} characters", MIN_ID_LENGTH);
            return Err(LatticeError::MalformedId { value: input.to_string() });
        }

        let first_char = input.chars().next().unwrap_or('\0');
        if first_char != ID_PREFIX && first_char != ID_PREFIX.to_ascii_lowercase() {
            tracing::debug!(id = input, "ID missing 'L' prefix");
            return Err(LatticeError::MalformedId { value: input.to_string() });
        }

        let body = &input[1..];
        if !base32_encoding::is_valid_base32(&body.to_uppercase()) {
            tracing::debug!(id = input, "ID contains invalid Base32 characters");
            return Err(LatticeError::MalformedId { value: input.to_string() });
        }

        Ok(Self { value: input.to_uppercase() })
    }

    /// Returns the ID string.
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Extracts the document counter from this ID.
    ///
    /// This requires knowing the client ID length, which varies. This method
    /// attempts extraction assuming a 3-character client ID, which is the most
    /// common case for small repositories.
    pub fn counter_assuming_client_len(&self, client_len: usize) -> Result<u64, LatticeError> {
        let body = &self.value[1..];
        if body.len() < client_len + 2 {
            return Err(LatticeError::MalformedId { value: self.value.clone() });
        }
        let counter_end = body.len() - client_len;
        base32_encoding::decode_u64(&body[..counter_end])
    }

    /// Extracts the client ID suffix from this ID.
    pub fn client_id_assuming_len(&self, client_len: usize) -> Result<&str, LatticeError> {
        let body = &self.value[1..];
        if body.len() < client_len + 2 {
            return Err(LatticeError::MalformedId { value: self.value.clone() });
        }
        Ok(&body[body.len() - client_len..])
    }
}

impl fmt::Display for LatticeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Debug for LatticeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LatticeId({})", self.value)
    }
}

impl FromStr for LatticeId {
    type Err = LatticeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        LatticeId::parse(s)
    }
}

impl AsRef<str> for LatticeId {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

impl serde::Serialize for LatticeId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.value)
    }
}

impl<'de> serde::Deserialize<'de> for LatticeId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        LatticeId::parse(&s).map_err(serde::de::Error::custom)
    }
}
