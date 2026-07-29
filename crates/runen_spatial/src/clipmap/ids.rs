use serde::{Deserialize, Serialize};

use crate::WorldId;
use crate::clipmap::{ClipmapCoord3, ClipmapLevel};

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct ClipmapCellId {
    pub world_id: WorldId,
    pub level: ClipmapLevel,
    pub coord: ClipmapCoord3,
}

impl ClipmapCellId {
    pub fn new(world_id: WorldId, level: ClipmapLevel, coord: ClipmapCoord3) -> Self {
        Self {
            world_id,
            level,
            coord,
        }
    }
}
