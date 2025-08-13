use std::fmt::{self, Debug, Display};

use serde::de::{self, Deserializer, Visitor};
use serde::{Deserialize, Serialize, Serializer};

/// Integer wrapper type for interoperating with JSON.
///
/// This type will fail deserialization if a number in the JSON payload cannot
/// be represented as a u32.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct TabulaInt {
    pub value: u32,
}

impl Debug for TabulaInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Display for TabulaInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Serialize for TabulaInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.value)
    }
}

impl<'de> Deserialize<'de> for TabulaInt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TabulaIntVisitor;

        impl<'de> Visitor<'de> for TabulaIntVisitor {
            type Value = TabulaInt;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a JSON number representable as u32")
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(TabulaInt {
                    value: u32::try_from(v)
                        .map_err(|_| E::custom("number out of range for u32"))?,
                })
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if v < 0 {
                    return Err(E::custom("negative numbers not allowed"));
                }
                Ok(TabulaInt {
                    value: u32::try_from(v as u64)
                        .map_err(|_| E::custom("number out of range for u32"))?,
                })
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if !v.is_finite() {
                    return Err(E::custom("non-finite float"));
                }
                if v < 0.0 {
                    return Err(E::custom("negative numbers not allowed"));
                }
                if v.fract() != 0.0 {
                    return Err(E::custom("expected integer, found float"));
                }
                if v > u32::MAX as f64 {
                    return Err(E::custom("number out of range for u32"));
                }
                Ok(TabulaInt { value: v as u32 })
            }
        }

        deserializer.deserialize_any(TabulaIntVisitor)
    }
}
