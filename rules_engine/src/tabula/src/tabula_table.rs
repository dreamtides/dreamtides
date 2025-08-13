use std::fmt;
use std::marker::PhantomData;
use std::ops::Deref;

use serde::de::{DeserializeOwned, Deserializer, SeqAccess, Visitor};
use serde::ser::{SerializeSeq, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A wrapper around a vector of items that implements Serde's `Serialize` and
/// `Deserialize` traits allowing for failure. Values that fail to deserialize
/// are logged and skipped.
#[derive(Clone, Debug)]
pub struct Table<T>(pub Vec<T>);

impl<T> Deref for Table<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Serialize for Table<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for item in &self.0 {
            seq.serialize_element(item)?;
        }
        seq.end()
    }
}

impl<'de, T> Deserialize<'de> for Table<T>
where
    T: DeserializeOwned,
{
    #[expect(clippy::print_stderr)]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TabulaTableVisitor<T> {
            marker: PhantomData<T>,
        }

        impl<'de, T> Visitor<'de> for TabulaTableVisitor<T>
        where
            T: DeserializeOwned,
        {
            type Value = Table<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a JSON array of table rows")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut items: Vec<T> = Vec::new();
                while let Some(v) = seq.next_element::<Value>()? {
                    match T::deserialize(v.clone()) {
                        Ok(item) => items.push(item),
                        Err(_) => {
                            eprintln!("failed to deserialize table row: {v:?}");
                        }
                    }
                }
                Ok(Table(items))
            }
        }

        deserializer.deserialize_seq(TabulaTableVisitor { marker: PhantomData })
    }
}
