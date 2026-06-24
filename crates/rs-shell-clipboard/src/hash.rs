use std::hash::{Hash, Hasher};

/// Compute a content hash for clipboard deduplication.
///
/// Uses `std::hash::DefaultHasher` (SipHash-2-4, 64-bit). This is not
/// cryptographic but provides extremely low collision probability for
/// clipboard-scale dedup (≤ ~10⁻¹² for 10k entries).
pub fn content_hash(content: &[u8]) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_content_same_hash() {
        assert_eq!(content_hash(b"hello"), content_hash(b"hello"));
    }

    #[test]
    fn different_content_different_hash() {
        assert_ne!(content_hash(b"hello"), content_hash(b"world"));
    }

    #[test]
    fn empty_content_hashes() {
        // Empty should not panic
        let h = content_hash(b"");
        assert_eq!(h.len(), 16);
    }
}
