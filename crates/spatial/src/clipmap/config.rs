use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClipmapConfig {
    pub base_cell_edge_meters: f32,
    pub level_count: u8,
    pub level_scale_factor: u32,
    pub window_dims: [u32; 3],
}

impl Default for ClipmapConfig {
    fn default() -> Self {
        Self {
            base_cell_edge_meters: 32.0,
            level_count: 4,
            level_scale_factor: 2,
            window_dims: [17, 5, 17],
        }
    }
}

impl ClipmapConfig {
    pub fn cell_edge_meters_for_level(&self, level: u8) -> f32 {
        let factor = self.level_scale_factor.max(1) as f32;
        self.base_cell_edge_meters.max(1.0) * factor.powi(level as i32)
    }

    pub fn clamped_window_dims(&self) -> [u32; 3] {
        [
            self.window_dims[0].max(1) | 1,
            self.window_dims[1].max(1) | 1,
            self.window_dims[2].max(1) | 1,
        ]
    }
}
