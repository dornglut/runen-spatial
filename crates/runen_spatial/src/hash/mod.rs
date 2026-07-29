//! Payload-neutral deterministic spatial hashing.
//!
//! This module is intentionally rule-free. It maps integer spatial coordinates
//! and caller-provided seeds to stable hash values, but it does not own
//! generation, residency, tile, biome, material, SDF, or provider policy.

use serde::{Deserialize, Serialize};

/// Seed used by deterministic spatial hash functions.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SpatialHashSeed(u64);

impl SpatialHashSeed {
    /// Creates a new hash seed.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw seed value.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }

    /// Returns a new seed mixed with one signed integer value.
    #[must_use]
    pub const fn mix_i64(self, value: i64) -> Self {
        Self(mix_i64(self.0, value))
    }
}

impl From<u64> for SpatialHashSeed {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

/// Stable spatial hash value.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SpatialHashValue(u64);

impl SpatialHashValue {
    /// Creates a hash value from raw bits.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw hash bits.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }

    /// Maps the hash into a bucket index.
    ///
    /// Returns `None` for zero buckets instead of panicking.
    #[must_use]
    pub const fn bucket_index(self, bucket_count: u64) -> Option<u64> {
        if bucket_count == 0 {
            None
        } else {
            Some(self.0 % bucket_count)
        }
    }
}

impl From<u64> for SpatialHashValue {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

/// Mixes one signed integer into raw hash state.
///
/// Signed values are treated by their two's-complement bit pattern so negative
/// coordinates remain deterministic and distinct from positive coordinates.
#[must_use]
pub const fn mix_i64(seed: u64, value: i64) -> u64 {
    mix_u64(seed, value as u64)
}

/// Mixes one unsigned integer into raw hash state.
#[must_use]
pub const fn mix_u64(seed: u64, value: u64) -> u64 {
    let mut h = seed ^ value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    h ^= h >> 30;
    h = h.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    h ^= h >> 27;
    h = h.wrapping_mul(0x94d0_49bb_1331_11eb);
    h ^ (h >> 31)
}

/// Finalizes raw hash state.
#[must_use]
pub const fn finalize_u64(value: u64) -> SpatialHashValue {
    SpatialHashValue(mix_u64(0x243f_6a88_85a3_08d3, value))
}

/// Hashes an ordered sequence of signed integers.
#[must_use]
pub fn hash_i64s(
    seed: impl Into<SpatialHashSeed>,
    values: impl IntoIterator<Item = i64>,
) -> SpatialHashValue {
    let mut state = seed.into().value();
    for value in values {
        state = mix_i64(state, value);
    }
    finalize_u64(state)
}

/// Hashes a 2D integer cell.
#[must_use]
pub fn hash_cell2(seed: impl Into<SpatialHashSeed>, x: i64, z: i64) -> SpatialHashValue {
    hash_i64s(seed, [x, z])
}

/// Hashes a 3D integer cell.
#[must_use]
pub fn hash_cell3(seed: impl Into<SpatialHashSeed>, x: i64, y: i64, z: i64) -> SpatialHashValue {
    hash_i64s(seed, [x, y, z])
}
