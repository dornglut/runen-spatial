use serde::{Deserialize, Serialize};

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct ClipmapCoord3 {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct ClipmapLevel(pub u8);
