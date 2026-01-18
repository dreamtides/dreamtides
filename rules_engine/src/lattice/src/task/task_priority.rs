use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::error_types::LatticeError;

/// Minimum priority value (highest priority).
pub const MIN_PRIORITY: u8 = 0;

/// Maximum priority value (lowest priority / backlog).
pub const MAX_PRIORITY: u8 = 4;

/// Default priority for new tasks.
pub const DEFAULT_PRIORITY: u8 = 2;

/// Task priority level.
///
/// Priority levels range from P0 (highest/critical) to P4 (lowest/backlog).
/// The `Ord` implementation ensures lower numbers sort first, so P0 < P1 < P2 <
/// P3 < P4 in sorting order.
///
/// # Parsing
///
/// Supports both numeric ("2") and prefixed ("P2", "p2") formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Priority {
    /// Critical: security, data loss, broken builds.
    P0 = 0,
    /// High: major features, important bugs.
    P1 = 1,
    /// Medium: default priority.
    P2 = 2,
    /// Low: polish, optimization.
    P3 = 3,
    /// Backlog: future ideas, excluded from `lat ready`.
    P4 = 4,
}

impl Priority {
    /// All priority levels in order from highest to lowest.
    pub const ALL: [Priority; 5] =
        [Priority::P0, Priority::P1, Priority::P2, Priority::P3, Priority::P4];
    /// The default priority for new tasks.
    pub const DEFAULT: Priority = Priority::P2;

    /// Returns the numeric value of this priority.
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    /// Creates a Priority from a numeric value.
    ///
    /// # Errors
    ///
    /// Returns `LatticeError::InvalidArgument` if the value is outside 0-4.
    pub fn from_u8(value: u8) -> Result<Self, LatticeError> {
        match value {
            0 => Ok(Priority::P0),
            1 => Ok(Priority::P1),
            2 => Ok(Priority::P2),
            3 => Ok(Priority::P3),
            4 => Ok(Priority::P4),
            _ => {
                tracing::debug!(value, "Priority value out of range");
                Err(LatticeError::InvalidArgument {
                    message: format!(
                        "invalid priority '{}': expected 0-4 (P0=critical, P4=backlog)",
                        value
                    ),
                })
            }
        }
    }

    /// Returns true if this is the backlog priority (P4).
    ///
    /// Backlog items are excluded from `lat ready`.
    pub fn is_backlog(&self) -> bool {
        *self == Priority::P4
    }

    /// Returns true if this is a critical priority (P0).
    pub fn is_critical(&self) -> bool {
        *self == Priority::P0
    }
}

impl Default for Priority {
    fn default() -> Self {
        Priority::DEFAULT
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "P{}", self.as_u8())
    }
}

impl FromStr for Priority {
    type Err = LatticeError;

    /// Parses a priority from a string.
    ///
    /// Accepts both numeric ("2") and prefixed ("P2", "p2") formats.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        // Handle prefixed format (P0, P1, p0, etc.)
        let numeric = if s.starts_with('P') || s.starts_with('p') { &s[1..] } else { s };

        match numeric.parse::<u8>() {
            Ok(n) => Priority::from_u8(n),
            Err(_) => {
                tracing::debug!(value = s, "Failed to parse priority");
                Err(LatticeError::InvalidArgument {
                    message: format!("invalid priority '{}': expected 0-4 or P0-P4", s),
                })
            }
        }
    }
}

impl Serialize for Priority {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.as_u8())
    }
}

impl<'de> Deserialize<'de> for Priority {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        Priority::from_u8(value).map_err(serde::de::Error::custom)
    }
}
