use serde::{Deserialize, Deserializer, Serialize};

use crate::SpatialMathError;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize)]
pub struct RingBufferConfig {
    dims: [u32; 3],
}

#[derive(Deserialize)]
struct RawRingBufferConfig {
    dims: [u32; 3],
}

impl<'de> Deserialize<'de> for RingBufferConfig {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::try_new(RawRingBufferConfig::deserialize(deserializer)?.dims)
            .map_err(serde::de::Error::custom)
    }
}

impl Default for RingBufferConfig {
    fn default() -> Self {
        Self::try_new([17, 5, 17]).expect("default ring configuration is valid")
    }
}

impl RingBufferConfig {
    pub fn try_new(dims: [u32; 3]) -> Result<Self, SpatialMathError> {
        for (axis, dimension) in dims.iter().enumerate() {
            if *dimension == 0 {
                return Err(SpatialMathError::ZeroDimension { axis: axis as u8 });
            }
        }
        Ok(Self { dims })
    }

    pub fn dims(&self) -> [u32; 3] {
        self.dims
    }
}
