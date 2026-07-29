use crate::clipmap::ClipmapCoord3;
use crate::ring::{RingBufferConfig, RingSlot3};

pub fn slot_for_coord(
    anchor: ClipmapCoord3,
    coord: ClipmapCoord3,
    config: &RingBufferConfig,
) -> RingSlot3 {
    fn slot(coord: i64, anchor: i64, size: u32) -> u32 {
        let modulus = i64::from(size);
        (coord.rem_euclid(modulus) - anchor.rem_euclid(modulus)).rem_euclid(modulus) as u32
    }
    let dims = config.dims();
    RingSlot3 {
        x: slot(coord.x, anchor.x, dims[0]),
        y: slot(coord.y, anchor.y, dims[1]),
        z: slot(coord.z, anchor.z, dims[2]),
    }
}
