use std::marker::PhantomData;
use std::{fmt, iter};

use bit_set::BitSet;
use serde::{Deserialize, Serialize};

use crate::battle::card_id::{CardId, CardIdType};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CardSet<T> {
    // - BitSet<usize> does around 2% better in our benchmarks than BitSet<u32>
    // - FixedBitSet generally seems to perform the same or worse.
    // - BTreeSet is around 8% slower than BitSet
    set: BitSet<usize>,

    #[serde(skip)]
    _marker: PhantomData<T>,
}

impl<T: CardIdType> fmt::Debug for CardSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CardSet({:?})", self.set)
    }
}

impl<T: CardIdType> Default for CardSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T: CardIdType> IntoIterator for &'a CardSet<T> {
    type IntoIter = iter::Map<bit_set::Iter<'a, usize>, fn(usize) -> T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.set.into_iter().map(|i| T::from_card_id(CardId(i)))
    }
}

impl<T: CardIdType> CardSet<T> {
    pub fn new() -> Self {
        Self { set: BitSet::default(), _marker: PhantomData }
    }

    /// Returns a set containing the given card ID.
    pub fn of(id: T) -> Self {
        let mut set = Self::new();
        set.insert(id);
        set
    }

    /// Returns a set containing the given card ID if it is not `None`, or an
    /// empty set otherwise.
    pub fn of_maybe(id: Option<T>) -> Self {
        if let Some(id) = id { Self::of(id) } else { Self::new() }
    }

    pub fn len(&self) -> usize {
        self.set.len()
    }

    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    pub fn clear(&mut self) {
        self.set.clear();
    }

    pub fn contains(&self, card_id: T) -> bool {
        self.set.contains(card_id.card_id().0)
    }

    pub fn insert(&mut self, card_id: T) -> bool {
        self.set.insert(card_id.card_id().0)
    }

    pub fn remove(&mut self, card_id: T) -> bool {
        self.set.remove(card_id.card_id().0)
    }

    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        self.set.iter().map(|i| T::from_card_id(CardId(i)))
    }

    pub fn difference_with(&mut self, other: &Self) {
        self.set.difference_with(&other.set);
    }
}
