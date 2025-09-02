use std::fmt::{self, Debug, Display};
use std::ops::Deref;

use core_data::display_types::StringWrapper;
use serde::de::{self, Deserializer, Visitor};
use serde::{Deserialize, Serialize, Serializer};

/// A wrapper around a value that can be constructed from or serialized to a
/// string.
#[derive(Clone, Copy)]
pub struct TabulaValue<T>(pub T);

impl<T> Deref for TabulaValue<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: StringWrapper> Serialize for TabulaValue<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string_value())
    }
}

struct TabulaVisitor<T>(std::marker::PhantomData<T>);

impl<'de, T: StringWrapper> Visitor<'de> for TabulaVisitor<T> {
    type Value = TabulaValue<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "string")
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        T::from_string_value(v).map(TabulaValue).map_err(|e| E::custom(e))
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
        T::from_string_value(&v).map(TabulaValue).map_err(|e| E::custom(e))
    }
}

impl<'de, T: StringWrapper> Deserialize<'de> for TabulaValue<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_string(TabulaVisitor(std::marker::PhantomData))
    }
}

impl<T: StringWrapper> Debug for TabulaValue<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_string_value())
    }
}

impl<T: StringWrapper> Display for TabulaValue<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_string_value())
    }
}

impl<T> From<T> for TabulaValue<T> {
    fn from(value: T) -> Self {
        TabulaValue(value)
    }
}

impl<T> AsRef<T> for TabulaValue<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}
