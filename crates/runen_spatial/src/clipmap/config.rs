use serde::{Deserialize, Deserializer, Serialize};

use crate::SpatialMathError;
use crate::clipmap::ClipmapLevel;

#[derive(Debug, Copy, Clone, PartialEq, Serialize)]
pub struct ClipmapConfig {
    base_cell_edge_meters: f64,
    level_count: u8,
    level_scale_factor: u32,
    window_dims: [u32; 3],
}

#[derive(Deserialize)]
struct RawClipmapConfig {
    base_cell_edge_meters: f64,
    level_count: u8,
    level_scale_factor: u32,
    window_dims: [u32; 3],
}

impl<'de> Deserialize<'de> for ClipmapConfig {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawClipmapConfig::deserialize(deserializer)?;
        Self::try_new(
            raw.base_cell_edge_meters,
            raw.level_count,
            raw.level_scale_factor,
            raw.window_dims,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl Default for ClipmapConfig {
    fn default() -> Self {
        Self::try_new(32.0, 4, 2, [17, 5, 17]).expect("default clipmap configuration is valid")
    }
}

impl ClipmapConfig {
    pub fn try_new(
        base_cell_edge_meters: f64,
        level_count: u8,
        level_scale_factor: u32,
        window_dims: [u32; 3],
    ) -> Result<Self, SpatialMathError> {
        if !base_cell_edge_meters.is_finite() {
            return Err(SpatialMathError::NonFiniteValue {
                field: "base_cell_edge_meters",
            });
        }
        if base_cell_edge_meters <= 0.0 {
            return Err(SpatialMathError::NonPositiveValue {
                field: "base_cell_edge_meters",
            });
        }
        if level_count == 0 {
            return Err(SpatialMathError::LevelCountZero);
        }
        if level_scale_factor < 2 {
            return Err(SpatialMathError::ScaleFactorTooSmall {
                scale_factor: level_scale_factor,
            });
        }
        for (axis, dimension) in window_dims.iter().enumerate() {
            if *dimension == 0 {
                return Err(SpatialMathError::ZeroDimension { axis: axis as u8 });
            }
            if dimension % 2 == 0 {
                return Err(SpatialMathError::EvenWindowDimension { axis: axis as u8 });
            }
        }
        Ok(Self {
            base_cell_edge_meters,
            level_count,
            level_scale_factor,
            window_dims,
        })
    }

    pub fn base_cell_edge_meters(&self) -> f64 {
        self.base_cell_edge_meters
    }
    pub fn level_count(&self) -> u8 {
        self.level_count
    }
    pub fn level_scale_factor(&self) -> u32 {
        self.level_scale_factor
    }
    pub fn window_dims(&self) -> [u32; 3] {
        self.window_dims
    }

    pub fn validate_level(&self, level: ClipmapLevel) -> Result<(), SpatialMathError> {
        if level.0 >= self.level_count {
            Err(SpatialMathError::LevelOutOfRange {
                level: level.0,
                level_count: self.level_count,
            })
        } else {
            Ok(())
        }
    }

    pub fn cell_edge_meters_for_level(
        &self,
        level: ClipmapLevel,
    ) -> Result<f64, SpatialMathError> {
        self.validate_level(level)?;
        let edge =
            self.base_cell_edge_meters * (self.level_scale_factor as f64).powi(i32::from(level.0));
        if !edge.is_finite() {
            return Err(SpatialMathError::ArithmeticOverflow {
                operation: "clipmap cell edge",
            });
        }
        Ok(edge)
    }
}
