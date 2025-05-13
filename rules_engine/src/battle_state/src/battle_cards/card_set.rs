use std::iter;
use std::marker::PhantomData;

use bit_set::BitSet;

use crate::battle::card_id::{CardId, CardIdType};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CardSet<T> {
    set: BitSet<usize>,
    _marker: PhantomData<T>,
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
}
