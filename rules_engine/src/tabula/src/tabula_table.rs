use std::fmt;
use std::marker::PhantomData;

use serde::de::{DeserializeOwned, Deserializer, SeqAccess, Visitor};
use serde::ser::{SerializeSeq, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A trait for types that have an ID.
pub trait HasId<I> {
    type Id: PartialEq + Copy + fmt::Debug;

    fn id(&self) -> Self::Id;
}

/// A wrapper around a vector of items that implements Serde's `Serialize` and
/// `Deserialize` traits allowing for failure. Values that fail to deserialize
/// are logged and skipped.
#[derive(Clone, Debug)]
pub struct Table<I, T>(pub Vec<T>, PhantomData<I>);

impl<I, T> Serialize for Table<I, T>
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

impl<'de, I, T> Deserialize<'de> for Table<I, T>
where
    T: DeserializeOwned,
{
    #[expect(clippy::print_stderr)]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TabulaTableVisitor<I, T> {
            marker: PhantomData<(I, T)>,
        }

        impl<'de, I, T> Visitor<'de> for TabulaTableVisitor<I, T>
        where
            T: DeserializeOwned,
        {
            type Value = Table<I, T>;

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
                            eprintln!("failed to deserialize table row, skipping: {v:?}");
                        }
                    }
                }
                Ok(Table(items, PhantomData))
            }
        }

        deserializer.deserialize_seq(TabulaTableVisitor { marker: PhantomData })
    }
}

impl<I, T> Table<I, T> {
    /// Returns the item with the given ID. Panics if the item is not found.
    pub fn get(&self, id: T::Id) -> &T
    where
        T: HasId<I>,
    {
        self.0
            .iter()
            .find(|item| item.id() == id)
            .unwrap_or_else(|| panic!("item with id {id:?} not found in table"))
    }

    /// Returns the item with the given ID, if it exists.
    pub fn get_optional(&self, id: T::Id) -> Option<&T>
    where
        T: HasId<I>,
    {
        self.0.iter().find(|item| item.id() == id)
    }
}
