use serde::{Deserialize, Serialize};

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct RingSlot3 {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}
