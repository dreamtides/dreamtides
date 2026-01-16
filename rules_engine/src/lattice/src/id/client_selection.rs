use std::collections::HashSet;

use crate::id::base32_encoding;

/// Client ID length thresholds based on number of known clients.
///
/// | Known Clients | ID Length | Possible Values |
/// |---------------|-----------|-----------------|
/// | 0-16          | 3         | 32768           |
/// | 17-64         | 4         | 1048576         |
/// | 65-256        | 5         | 33554432        |
/// | 257+          | 6         | 1073741824      |
const LENGTH_THRESHOLDS: [(usize, usize); 4] = [(0, 3), (17, 4), (65, 5), (257, 6)];

/// Determines the required client ID length based on known client count.
pub fn required_length(known_clients: usize) -> usize {
    for &(threshold, length) in LENGTH_THRESHOLDS.iter().rev() {
        if known_clients >= threshold {
            return length;
        }
    }
    LENGTH_THRESHOLDS[0].1
}

/// Generates a new random client ID that doesn't collide with existing IDs.
///
/// The length scales with the number of known clients to balance collision
/// probability against ID brevity.
///
/// # Arguments
///
/// * `existing_client_ids` - Set of client IDs already in use
/// * `rng` - Random number generator for reproducible testing
///
/// # Returns
///
/// A new unique client ID string.
pub fn generate_client_id<R: rand::Rng>(
    existing_client_ids: &HashSet<String>,
    rng: &mut R,
) -> String {
    let length = required_length(existing_client_ids.len());
    let max_value = 32u64.pow(length as u32);

    loop {
        let value = rng.random_range(0..max_value);
        let client_id = base32_encoding::encode_u64(value, length);

        if !existing_client_ids.contains(&client_id) {
            tracing::info!(
                client_id = %client_id,
                length = length,
                existing_count = existing_client_ids.len(),
                "Generated new client ID"
            );
            return client_id;
        }

        tracing::debug!(
            client_id = %client_id,
            "Client ID collision, retrying"
        );
    }
}

/// Generates a client ID using the default random number generator.
pub fn generate_client_id_random(existing_client_ids: &HashSet<String>) -> String {
    generate_client_id(existing_client_ids, &mut rand::rng())
}

/// Validates that a client ID has the expected format.
pub fn is_valid_client_id(client_id: &str) -> bool {
    let len = client_id.len();
    (3..=6).contains(&len) && base32_encoding::is_valid_base32(client_id)
}

/// Extracts the set of unique client IDs from a collection of Lattice IDs.
///
/// This is used during counter recovery to identify all clients that have
/// contributed to the repository. Since we don't know the client ID lengths
/// without external information, this uses a heuristic: we try common lengths
/// (3, 4, 5, 6) and check for consistency.
///
/// # Arguments
///
/// * `ids` - Collection of Lattice ID strings (with L prefix)
/// * `assumed_length` - The client ID length to assume
pub fn extract_client_ids<'a, I>(ids: I, assumed_length: usize) -> HashSet<String>
where
    I: IntoIterator<Item = &'a str>,
{
    ids.into_iter()
        .filter_map(|id| {
            let body = id.strip_prefix('L').or_else(|| id.strip_prefix('l'))?;
            if body.len() >= assumed_length + 2 {
                Some(body[body.len() - assumed_length..].to_uppercase())
            } else {
                None
            }
        })
        .collect()
}
