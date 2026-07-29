use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpatialPoint3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl SpatialPoint3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn from_array(values: [f32; 3]) -> Self {
        Self {
            x: values[0],
            y: values[1],
            z: values[2],
        }
    }

    pub fn to_array(self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpatialAabb3 {
    pub min: SpatialPoint3,
    pub max: SpatialPoint3,
}

impl SpatialAabb3 {
    pub fn new(min: SpatialPoint3, max: SpatialPoint3) -> Self {
        Self { min, max }
    }

    pub fn from_arrays(min: [f32; 3], max: [f32; 3]) -> Self {
        Self {
            min: SpatialPoint3::from_array(min),
            max: SpatialPoint3::from_array(max),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.min.x.is_finite()
            && self.min.y.is_finite()
            && self.min.z.is_finite()
            && self.max.x.is_finite()
            && self.max.y.is_finite()
            && self.max.z.is_finite()
            && self.min.x <= self.max.x
            && self.min.y <= self.max.y
            && self.min.z <= self.max.z
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }
}
