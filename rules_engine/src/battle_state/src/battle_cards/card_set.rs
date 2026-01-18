use std::fmt;
use std::marker::PhantomData;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::battle::card_id::{CardId, CardIdType};
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CardSet<T> {
    bits: u128,
    _marker: PhantomData<T>,
}
/// Iterator for CardSet that yields card IDs in order from lowest to highest.
pub struct CardSetIter<T> {
    bits: u128,
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
impl<T: CardIdType> Iterator for CardSetIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            return None;
        }
        let pos = self.bits.trailing_zeros() as usize;
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
impl<T: CardIdType> Serialize for CardSet<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut vec = Vec::with_capacity(self.len());
        for id in self.iter() {
            vec.push(id.card_id().0 as u64);
        }
        vec.serialize(serializer)
    }
}
impl<'de, T: CardIdType> Deserialize<'de> for CardSet<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw: Vec<u64> = Vec::<u64>::deserialize(deserializer)?;
        let mut set = CardSet::new();
        for v in raw {
            if v >= 128 {
                return Err(serde::de::Error::custom("CardSet only supports card IDs 0-127"));
            }
            set.bits |= 1u128 << v;
        }
        Ok(set)
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

    /// Fast intersection operation: keeps only elements that are present in
    /// both sets.
    #[inline(always)]
    pub fn intersect_with(&mut self, other: &Self) {
        self.bits &= other.bits;
    }

    /// Returns the card ID at the given index (0-based), treating the set as
    /// an ordered collection from lowest to highest bit position.
    ///
    /// Returns `None` if the index is out of bounds.
    ///
    /// # Examples
    /// ```
    /// let mut set = CardSet::new();
    /// set.insert(CardId(1));
    /// set.insert(CardId(5));
    /// set.insert(CardId(10));
    ///
    /// assert_eq!(set.get_at_index(0), Some(CardId(1))); // First element
    /// assert_eq!(set.get_at_index(1), Some(CardId(5))); // Second element
    /// assert_eq!(set.get_at_index(2), Some(CardId(10))); // Third element
    /// assert_eq!(set.get_at_index(3), None); // Out of bounds
    /// ```
    #[inline]
    pub fn get_at_index(&self, index: usize) -> Option<T> {
        let len = self.len();
        if index >= len {
            return None;
        }
        self.get_at_index_impl(index)
    }

    #[inline]
    fn get_at_index_impl(&self, index: usize) -> Option<T> {
        let mut remaining_bits = self.bits;
        let mut current_index = 0;
        let mut bit_position = 0;
        if index >= 8 {
            while current_index + 16 < index && remaining_bits != 0 {
                let chunk = remaining_bits & 0xFFFFFFFF;
                let chunk_count = chunk.count_ones() as usize;
                if current_index + chunk_count <= index {
                    current_index += chunk_count;
                    remaining_bits >>= 32;
                    bit_position += 32;
                } else {
                    break;
                }
            }
        }
        while remaining_bits != 0 {
            let trailing_zeros = remaining_bits.trailing_zeros() as usize;
            let pos = bit_position + trailing_zeros;
            if current_index == index {
                return Some(T::from_card_id(CardId(pos)));
            }
            remaining_bits &= remaining_bits - 1;
            current_index += 1;
        }
        None
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
#[cfg(test)]
mod tests {
    use crate::battle::card_id::{CharacterId, HandCardId};
    use crate::battle_cards::card_set::*;
    fn test_card_id(id: usize) -> CharacterId {
        CharacterId(CardId(id))
    }
    #[test]
    fn test_new_and_default() {
        let set1 = CardSet::<CharacterId>::new();
        let set2 = CardSet::<CharacterId>::default();
        assert!(set1.is_empty());
        assert!(set2.is_empty());
        assert_eq!(set1.len(), 0);
        assert_eq!(set2.len(), 0);
        assert_eq!(set1, set2);
    }
    #[test]
    fn test_of_and_contains() {
        let set = CardSet::of(test_card_id(42));
        assert!(!set.is_empty());
        assert_eq!(set.len(), 1);
        assert!(set.contains(test_card_id(42)));
        assert!(!set.contains(test_card_id(43)));
    }
    #[test]
    fn test_of_maybe() {
        let set1 = CardSet::of_maybe(Some(test_card_id(10)));
        let set2 = CardSet::of_maybe(None::<CharacterId>);
        assert!(set1.contains(test_card_id(10)));
        assert!(set2.is_empty());
    }
    #[test]
    fn test_insert_and_remove() {
        let mut set = CardSet::new();
        assert!(set.insert(test_card_id(1)));
        assert!(set.insert(test_card_id(5)));
        assert!(set.insert(test_card_id(10)));
        assert!(!set.insert(test_card_id(5)));
        assert_eq!(set.len(), 3);
        assert!(set.contains(test_card_id(1)));
        assert!(set.contains(test_card_id(5)));
        assert!(set.contains(test_card_id(10)));
        assert!(set.remove(test_card_id(5)));
        assert!(!set.contains(test_card_id(5)));
        assert_eq!(set.len(), 2);
        assert!(!set.remove(test_card_id(99)));
        assert_eq!(set.len(), 2);
    }
    #[test]
    fn test_get_at_index() {
        let mut set = CardSet::new();
        assert_eq!(set.get_at_index(0), None);
        set.insert(test_card_id(10));
        set.insert(test_card_id(3));
        set.insert(test_card_id(7));
        set.insert(test_card_id(1));
        assert_eq!(set.get_at_index(0), Some(test_card_id(1)));
        assert_eq!(set.get_at_index(1), Some(test_card_id(3)));
        assert_eq!(set.get_at_index(2), Some(test_card_id(7)));
        assert_eq!(set.get_at_index(3), Some(test_card_id(10)));
        assert_eq!(set.get_at_index(4), None);
        assert_eq!(set.get_at_index(100), None);
    }
    #[test]
    fn test_get_at_index_large_set() {
        let mut set = CardSet::new();
        let test_ids = [0, 15, 31, 47, 63, 79, 95, 111, 127];
        for &id in &test_ids {
            set.insert(test_card_id(id));
        }
        for (index, &expected_id) in test_ids.iter().enumerate() {
            assert_eq!(set.get_at_index(index), Some(test_card_id(expected_id)));
        }
        assert_eq!(set.get_at_index(test_ids.len()), None);
    }
    #[test]
    fn test_bitwise_operations() {
        let mut set1 = CardSet::new();
        set1.insert(test_card_id(1));
        set1.insert(test_card_id(3));
        set1.insert(test_card_id(5));
        let mut set2 = CardSet::new();
        set2.insert(test_card_id(3));
        set2.insert(test_card_id(5));
        set2.insert(test_card_id(7));
        let mut union_set = set1.clone();
        union_set.union_with(&set2);
        assert_eq!(union_set.len(), 4);
        assert!(union_set.contains(test_card_id(1)));
        assert!(union_set.contains(test_card_id(3)));
        assert!(union_set.contains(test_card_id(5)));
        assert!(union_set.contains(test_card_id(7)));
        let mut diff_set = set1.clone();
        diff_set.difference_with(&set2);
        assert_eq!(diff_set.len(), 1);
        assert!(diff_set.contains(test_card_id(1)));
    }
    #[test]
    fn test_iterator() {
        let mut set = CardSet::new();
        set.insert(test_card_id(10));
        set.insert(test_card_id(3));
        set.insert(test_card_id(7));
        set.insert(test_card_id(1));
        let collected: Vec<_> = set.iter().collect();
        let expected = vec![test_card_id(1), test_card_id(3), test_card_id(7), test_card_id(10)];
        assert_eq!(collected, expected);
        let mut iter = set.iter();
        assert_eq!(iter.len(), 4);
        assert_eq!(iter.size_hint(), (4, Some(4)));
        iter.next();
        assert_eq!(iter.len(), 3);
        assert_eq!(iter.size_hint(), (3, Some(3)));
    }
    #[test]
    fn test_into_iterator() {
        let mut set = CardSet::new();
        set.insert(test_card_id(5));
        set.insert(test_card_id(2));
        let collected1: Vec<_> = (&set).into_iter().collect();
        let expected = vec![test_card_id(2), test_card_id(5)];
        assert_eq!(collected1, expected);
        let collected2: Vec<_> = set.into_iter().collect();
        assert_eq!(collected2, expected);
    }
    #[test]
    fn test_from_iterator_and_extend() {
        let ids = vec![test_card_id(1), test_card_id(5), test_card_id(3), test_card_id(1)];
        let set: CardSet<CharacterId> = ids.into_iter().collect();
        assert_eq!(set.len(), 3);
        assert!(set.contains(test_card_id(1)));
        assert!(set.contains(test_card_id(3)));
        assert!(set.contains(test_card_id(5)));
        let mut set2 = CardSet::new();
        set2.extend(vec![test_card_id(7), test_card_id(9)]);
        assert_eq!(set2.len(), 2);
        assert!(set2.contains(test_card_id(7)));
        assert!(set2.contains(test_card_id(9)));
    }
    #[test]
    fn test_clear() {
        let mut set = CardSet::new();
        set.insert(test_card_id(1));
        set.insert(test_card_id(2));
        assert!(!set.is_empty());
        set.clear();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
    }
    #[test]
    fn test_reinterpret_as() {
        let mut char_set = CardSet::<CharacterId>::new();
        char_set.insert(test_card_id(42));
        let hand_set: CardSet<HandCardId> = char_set.reinterpret_as();
        assert_eq!(hand_set.len(), 1);
        assert!(hand_set.contains(HandCardId(CardId(42))));
    }
    #[test]
    fn test_json_serialization_vector_form() {
        let mut set = CardSet::new();
        set.insert(test_card_id(5));
        set.insert(test_card_id(1));
        set.insert(test_card_id(9));
        let json = serde_json::to_string(&set).unwrap();
        assert_eq!(json, "[1,5,9]");
        let de: CardSet<CharacterId> = serde_json::from_str(&json).unwrap();
        assert_eq!(de.len(), 3);
        assert!(de.contains(test_card_id(1)));
        assert!(de.contains(test_card_id(5)));
        assert!(de.contains(test_card_id(9)));
    }
    #[test]
    fn test_consistency_get_at_index_vs_iterator() {
        let mut set = CardSet::new();
        let test_positions = [0, 15, 31, 47, 63, 79, 95, 111, 120, 127];
        for &pos in &test_positions {
            set.insert(test_card_id(pos));
        }
        let iter_result: Vec<_> = set.iter().collect();
        let index_result: Vec<_> = (0..set.len()).map(|i| set.get_at_index(i).unwrap()).collect();
        assert_eq!(iter_result, index_result);
    }
    #[test]
    fn test_edge_cases_boundary_values() {
        let mut set = CardSet::new();
        set.insert(test_card_id(0));
        set.insert(test_card_id(127));
        assert_eq!(set.len(), 2);
        assert_eq!(set.get_at_index(0), Some(test_card_id(0)));
        assert_eq!(set.get_at_index(1), Some(test_card_id(127)));
        assert_eq!(set.get_at_index(0), Some(test_card_id(0)));
        assert_eq!(set.get_at_index(set.len() - 1), Some(test_card_id(127)));
    }
    #[test]
    fn test_get_at_index_performance_edge_cases() {
        let mut sparse_set = CardSet::new();
        let sparse_ids = [1, 32, 64, 96, 127];
        for &id in &sparse_ids {
            sparse_set.insert(test_card_id(id));
        }
        for (index, &expected_id) in sparse_ids.iter().enumerate() {
            assert_eq!(sparse_set.get_at_index(index), Some(test_card_id(expected_id)));
        }
        let mut dense_set = CardSet::new();
        for id in 0..16 {
            dense_set.insert(test_card_id(id));
        }
        for id in 0..16 {
            assert_eq!(dense_set.get_at_index(id), Some(test_card_id(id)));
        }
    }
    #[test]
    fn test_get_at_index_across_bit_boundaries() {
        let mut set = CardSet::new();
        let test_ids = [10, 50, 70, 100, 120];
        for &id in &test_ids {
            set.insert(test_card_id(id));
        }
        for (index, &expected_id) in test_ids.iter().enumerate() {
            assert_eq!(set.get_at_index(index), Some(test_card_id(expected_id)));
        }
    }
    #[test]
    fn test_get_at_index_random_access_pattern() {
        let mut set = CardSet::new();
        let mut expected_order = Vec::new();
        let insert_order = [50, 10, 90, 30, 70, 20, 60, 40, 80, 100];
        for &id in &insert_order {
            set.insert(test_card_id(id));
            expected_order.push(id);
        }
        expected_order.sort_unstable();
        for &index in &[0, 4, 2, 7, 1, 9, 3, 5, 6, 8] {
            assert_eq!(set.get_at_index(index), Some(test_card_id(expected_order[index])));
        }
    }
}
