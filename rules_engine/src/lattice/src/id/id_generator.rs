use crate::id::lattice_id::LatticeId;

/// Starting counter value to ensure all IDs are at least 6 characters.
///
/// Counter starts at 50 (Base32: "BS") so that L + 2-digit counter + 3-digit
/// client ID = 6 characters minimum.
pub const INITIAL_COUNTER: u64 = 50;

/// Document counter state for ID generation.
///
/// Each client maintains its own counter that increments for each new document.
/// The counter can be recovered from existing documents when re-cloning.
#[derive(Debug, Clone)]
pub struct DocumentCounter {
    /// Current counter value.
    counter: u64,
}

/// Generates a new Lattice ID using the given counter and client ID.
///
/// This is the primary ID generation function. It combines:
/// - The L prefix
/// - A Base32-encoded document counter (minimum 2 characters)
/// - The client ID suffix
pub fn generate_id(counter: &mut DocumentCounter, client_id: &str) -> LatticeId {
    let counter_value = counter.next_value();
    let id = LatticeId::from_parts(counter_value, client_id);
    tracing::info!(id = %id, counter = counter_value, client = client_id, "Generated new Lattice ID");
    id
}

impl DocumentCounter {
    /// Creates a new counter starting at the initial value.
    pub fn new() -> Self {
        Self { counter: INITIAL_COUNTER }
    }

    /// Creates a counter starting at a specific value.
    ///
    /// Used when recovering counter state from existing documents.
    pub fn starting_at(value: u64) -> Self {
        Self { counter: value.max(INITIAL_COUNTER) }
    }

    /// Returns the current counter value without incrementing.
    pub fn current(&self) -> u64 {
        self.counter
    }

    /// Increments the counter and returns the previous value.
    ///
    /// The returned value should be used for ID generation.
    pub fn next_value(&mut self) -> u64 {
        let current = self.counter;
        self.counter += 1;
        tracing::debug!(counter = current, "Generated next counter value");
        current
    }

    /// Updates the counter to be at least the given value.
    ///
    /// Used during counter recovery to ensure we never reuse an ID.
    pub fn ensure_at_least(&mut self, value: u64) {
        if value >= self.counter {
            tracing::debug!(
                old_counter = self.counter,
                new_counter = value + 1,
                "Updating counter to avoid collision"
            );
            self.counter = value + 1;
        }
    }
}

impl Default for DocumentCounter {
    fn default() -> Self {
        Self::new()
    }
}
