use crate::WorldId;
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SpatialMathError {
    NonFiniteValue { field: &'static str },
    NonPositiveValue { field: &'static str },
    ZeroDimension { axis: u8 },
    EvenWindowDimension { axis: u8 },
    ScaleFactorTooSmall { scale_factor: u32 },
    LevelCountZero,
    LevelOutOfRange { level: u8, level_count: u8 },
    CoordinateOutOfRange { operation: &'static str },
    PrecisionLoss { operation: &'static str },
    ArithmeticOverflow { operation: &'static str },
    WorldMismatch { expected: WorldId, actual: WorldId },
    LocalPositionOutOfRange { axis: u8 },
}

impl fmt::Display for SpatialMathError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "spatial math error: {self:?}")
    }
}

impl std::error::Error for SpatialMathError {}
