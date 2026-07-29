use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RingBufferConfig {
    pub dims: [u32; 3],
}

impl Default for RingBufferConfig {
    fn default() -> Self {
        Self { dims: [17, 5, 17] }
    }
}
