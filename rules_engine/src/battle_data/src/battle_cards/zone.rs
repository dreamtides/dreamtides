/// Possible game regions where a card can be located.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Zone {
    Banished,
    Battlefield,
    Deck,
    Hand,
    Stack,
    Void,
}
