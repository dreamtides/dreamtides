use std::fmt;
use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::battle::card_id::{CardId, CardIdType};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CardSet<T> {
    bits: u128,

    #[serde(skip)]
    _marker: PhantomData<T>,
}

impl<T: CardIdType> fmt::Debug for CardSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CardSet(0b{:0128b})", self.bits)
    }
}

impl<T: CardIdType> Default for CardSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Iterator for CardSet that yields card IDs in order from lowest to highest.
pub struct CardSetIter<T> {
    bits: u128,
    _marker: PhantomData<T>,
}

impl<T: CardIdType> Iterator for CardSetIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            return None;
        }

        // Find the position of the lowest set bit using trailing_zeros
        let pos = self.bits.trailing_zeros() as usize;

        // Clear the lowest set bit using bit manipulation trick
        self.bits &= self.bits - 1;

        Some(T::from_card_id(CardId(pos)))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.bits.count_ones() as usize;
        (count, Some(count))
    }
}

impl<T: CardIdType> ExactSizeIterator for CardSetIter<T> {
    #[inline]
    fn len(&self) -> usize {
        self.bits.count_ones() as usize
    }
}

impl<T: CardIdType> IntoIterator for &CardSet<T> {
    type IntoIter = CardSetIter<T>;
    type Item = T;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        CardSetIter { bits: self.bits, _marker: PhantomData }
    }
}

impl<T: CardIdType> IntoIterator for CardSet<T> {
    type IntoIter = CardSetIter<T>;
    type Item = T;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        CardSetIter { bits: self.bits, _marker: PhantomData }
    }
}

impl<T: CardIdType> FromIterator<T> for CardSet<T> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut set = Self::new();
        set.extend(iter);
        set
    }
}

impl<T: CardIdType> Extend<T> for CardSet<T> {
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            let pos = item.card_id().0;
            debug_assert!(pos < 128, "CardSet only supports card IDs 0-127, got {pos}");
            self.bits |= 1u128 << pos;
        }
    }
}

impl<T: CardIdType> CardSet<T> {
    #[inline(always)]
    pub fn new() -> Self {
        Self { bits: 0, _marker: PhantomData }
    }

    /// Returns a set containing the given card ID.
    #[inline]
    pub fn of(id: T) -> Self {
        let pos = id.card_id().0;
        if pos >= 128 {
            panic!("CardSet only supports card IDs 0-127, got {pos}");
        }
        Self { bits: 1u128 << pos, _marker: PhantomData }
    }

    /// Returns a set containing the given card ID if it is not `None`, or an
    /// empty set otherwise.
    #[inline]
    pub fn of_maybe(id: Option<T>) -> Self {
        if let Some(id) = id { Self::of(id) } else { Self::new() }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.bits.count_ones() as usize
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.bits == 0
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.bits = 0;
    }

    #[inline(always)]
    pub fn contains(&self, card_id: T) -> bool {
        let pos = card_id.card_id().0;
        debug_assert!(pos < 128, "CardSet only supports card IDs 0-127, got {pos}");
        (self.bits & (1u128 << pos)) != 0
    }

    #[inline]
    pub fn insert(&mut self, card_id: T) -> bool {
        let pos = card_id.card_id().0;
        debug_assert!(pos < 128, "CardSet only supports card IDs 0-127, got {pos}");
        let mask = 1u128 << pos;
        let was_present = (self.bits & mask) != 0;
        self.bits |= mask;
        !was_present
    }

    #[inline]
    pub fn remove(&mut self, card_id: T) -> bool {
        let pos = card_id.card_id().0;
        debug_assert!(pos < 128, "CardSet only supports card IDs 0-127, got {pos}");
        let mask = 1u128 << pos;
        let was_present = (self.bits & mask) != 0;
        self.bits &= !mask;
        was_present
    }

    #[inline]
    pub fn iter(&self) -> CardSetIter<T> {
        CardSetIter { bits: self.bits, _marker: PhantomData }
    }

    /// Fast difference operation: removes all elements in `other` from this
    /// set.
    #[inline(always)]
    pub fn difference_with(&mut self, other: &Self) {
        self.bits &= !other.bits;
    }

    /// Fast union operation: adds all elements from `other` to this set.
    #[inline(always)]
    pub fn union_with(&mut self, other: &Self) {
        self.bits |= other.bits;
    }

    /// Fast reinterpret operation: reinterprets this set as containing
    /// different CardId types. This is a zero-cost operation since all
    /// CardId types are newtypes around the same usize.
    ///
    /// # Safety
    /// This is safe because all CardId types (BattleDeckCardId, VoidCardId,
    /// etc.) are newtypes around CardId which wraps usize. The underlying
    /// bit representation is identical.
    #[inline(always)]
    pub fn reinterpret_as<U: CardIdType>(self) -> CardSet<U> {
        CardSet { bits: self.bits, _marker: PhantomData }
    }
}
