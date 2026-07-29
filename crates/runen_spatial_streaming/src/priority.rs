use runen_spatial::{ChunkCoord3, SpatialMathError};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct ChunkPriority {
    pub rank: u32,
    pub distance_squared: i128,
}

impl ChunkPriority {
    pub fn new(rank: u32, distance_squared: i128) -> Self {
        Self {
            rank,
            distance_squared,
        }
    }
}

pub(crate) fn distance_squared(a: ChunkCoord3, b: ChunkCoord3) -> Result<i128, SpatialMathError> {
    let dx = i128::from(a.x).checked_sub(i128::from(b.x)).ok_or(
        SpatialMathError::ArithmeticOverflow {
            operation: "chunk distance delta",
        },
    )?;
    let dy = i128::from(a.y).checked_sub(i128::from(b.y)).ok_or(
        SpatialMathError::ArithmeticOverflow {
            operation: "chunk distance delta",
        },
    )?;
    let dz = i128::from(a.z).checked_sub(i128::from(b.z)).ok_or(
        SpatialMathError::ArithmeticOverflow {
            operation: "chunk distance delta",
        },
    )?;
    let x_squared = dx
        .checked_mul(dx)
        .ok_or(SpatialMathError::ArithmeticOverflow {
            operation: "chunk distance square",
        })?;
    let y_squared = dy
        .checked_mul(dy)
        .ok_or(SpatialMathError::ArithmeticOverflow {
            operation: "chunk distance square",
        })?;
    let z_squared = dz
        .checked_mul(dz)
        .ok_or(SpatialMathError::ArithmeticOverflow {
            operation: "chunk distance square",
        })?;
    x_squared
        .checked_add(y_squared)
        .and_then(|sum| sum.checked_add(z_squared))
        .ok_or(SpatialMathError::ArithmeticOverflow {
            operation: "chunk distance sum",
        })
}
