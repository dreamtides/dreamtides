use std::fmt;
use std::marker::PhantomData;

use serde::de::{DeserializeOwned, Deserializer, SeqAccess, Visitor};
use serde::ser::{SerializeSeq, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::{Deserializer as JsonDeserializer, Error as JsonError, Value};
use serde_path_to_error as serde_path;
use serde_path_to_error::Error as PathError;

pub fn from_vec<I, T>(items: Vec<T>) -> Table<I, T> {
    Table(items, PhantomData)
}

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
                let mut index: usize = 0;
                while let Some(v) = seq.next_element::<Value>()? {
                    let s = v.to_string();
                    let mut de = JsonDeserializer::from_str(&s);
                    match serde_path::deserialize(&mut de) {
                        Ok(item) => items.push(item),
                        Err(err) => log_tabula_row_error(index, &v, &s, &err),
                    }
                    index += 1;
                }
                Ok(Table(items, PhantomData))
            }
        }

        deserializer.deserialize_seq(TabulaTableVisitor { marker: PhantomData })
    }
}

impl<I, T> Table<I, T> {
    pub fn as_slice(&self) -> &[T] {
        &self.0
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.0.iter_mut()
    }

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

#[expect(clippy::print_stderr)]
fn log_tabula_row_error(index: usize, v: &Value, s: &str, err: &PathError<JsonError>) {
    let header = "\x1b[31;1mTabula deserialization error\x1b[0m";
    let path_line = format!("  \x1b[1mPath\x1b[0m: {}", err.path());
    let error_line = format!("  \x1b[1mError\x1b[0m: {}", err.inner());
    let context_line =
        format!("  \x1b[1mContext\x1b[0m: {}", error_context_snippet(s, err.inner()));
    let row_line = match serde_json::to_string_pretty(v) {
        Ok(pretty) => format!("  \x1b[1mRow JSON\x1b[0m:\n    {}", indent_multiline(&pretty, 4)),
        Err(_) => format!("  \x1b[1mRow\x1b[0m: {v:?}"),
    };
    match v {
        Value::Object(map) => match map.get("id") {
            Some(id) => eprintln!(
                "\n{header}\n  \x1b[1mRow\x1b[0m: {index}\n  \x1b[1mId\x1b[0m: {id}\n{path_line}\n{error_line}\n{context_line}\n{row_line}\n"
            ),
            None => eprintln!(
                "\n{header}\n  \x1b[1mRow\x1b[0m: {index}\n{path_line}\n{error_line}\n{context_line}\n{row_line}\n"
            ),
        },
        _ => eprintln!(
            "\n{header}\n  \x1b[1mRow\x1b[0m: {index}\n{path_line}\n{error_line}\n{context_line}\n{row_line}\n"
        ),
    }
}

fn error_context_snippet(s: &str, json_error: &JsonError) -> String {
    let msg = json_error.to_string();
    match parse_error_column(&msg) {
        Some(col) if col > 0 => {
            let idx = col.saturating_sub(1);
            let start = idx.saturating_sub(30);
            let end = (idx + 30).min(s.len());
            let sb = prev_char_boundary(s, start);
            let eb = next_char_boundary(s, end);
            let ib = prev_char_boundary(s, idx);
            let ch = s.get(ib..eb).and_then(|t| t.chars().next()).unwrap_or('?');
            let ch_len = ch.len_utf8();
            let pre = s.get(sb..ib).unwrap_or("");
            let post = s.get((ib + ch_len).min(s.len())..eb).unwrap_or("");
            format!(
                "context around column {col}: \x1b[2m...{pre}\x1b[0m\x1b[31;1m[{ch}]\x1b[0m\x1b[2m{post}...\x1b[0m"
            )
        }
        _ => String::from("context: <unavailable>"),
    }
}

fn indent_multiline(s: &str, spaces: usize) -> String {
    let pad = " ".repeat(spaces);
    s.lines().map(|line| format!("{pad}{line}")).collect::<Vec<_>>().join("\n")
}

fn parse_error_column(msg: &str) -> Option<usize> {
    match msg.rfind("column ") {
        Some(i) => {
            let j = i + 7;
            let digits: String = msg[j..].chars().take_while(|c| c.is_ascii_digit()).collect();
            digits.parse().ok()
        }
        None => None,
    }
}

fn prev_char_boundary(s: &str, mut idx: usize) -> usize {
    if idx > s.len() {
        idx = s.len();
    }
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

fn next_char_boundary(s: &str, mut idx: usize) -> usize {
    if idx > s.len() {
        idx = s.len();
    }
    while idx < s.len() && !s.is_char_boundary(idx) {
        idx += 1;
    }
    idx
}
