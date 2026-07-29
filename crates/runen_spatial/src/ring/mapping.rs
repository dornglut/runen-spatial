use crate::clipmap::ClipmapCoord3;
use crate::ring::{RingBufferConfig, RingSlot3};

pub fn slot_for_coord(
    anchor: ClipmapCoord3,
    coord: ClipmapCoord3,
    config: &RingBufferConfig,
) -> RingSlot3 {
    fn wrap(delta: i32, size: u32) -> u32 {
        let size_i = size.max(1) as i32;
        let mut v = delta % size_i;
        if v < 0 {
            v += size_i;
        }
        v as u32
    }

    RingSlot3 {
        x: wrap(coord.x - anchor.x, config.dims[0]),
        y: wrap(coord.y - anchor.y, config.dims[1]),
        z: wrap(coord.z - anchor.z, config.dims[2]),
    }
}
