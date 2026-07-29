use crate::{
    ChunkLoadOrder, ChunkSet, ChunkSetDiff, ChunkStreamingConfig, ChunkStreamingMode,
    StreamingFocus,
};
use runen_spatial::{ChunkCoord3, GridPartitionConfig, SpatialMathError};

pub struct ChunkStreamer {
    partition: GridPartitionConfig,
    config: ChunkStreamingConfig,
    active: ChunkSet,
}

#[derive(Debug, Clone)]
pub struct ChunkFocusUpdate {
    center: ChunkCoord3,
    next: ChunkSet,
    diff: ChunkSetDiff,
}

impl ChunkFocusUpdate {
    pub fn center(&self) -> ChunkCoord3 {
        self.center
    }

    pub fn diff(&self) -> &ChunkSetDiff {
        &self.diff
    }
}

impl ChunkStreamer {
    pub fn new(partition: GridPartitionConfig, config: ChunkStreamingConfig) -> Self {
        Self {
            partition,
            config: config.clamped(),
            active: ChunkSet::default(),
        }
    }
    pub fn partition(&self) -> &GridPartitionConfig {
        &self.partition
    }
    pub fn config(&self) -> ChunkStreamingConfig {
        self.config
    }
    pub fn active_chunks(&self) -> &ChunkSet {
        &self.active
    }
    pub fn active_chunk_count(&self) -> usize {
        self.active.len()
    }
    pub fn set_config(&mut self, config: ChunkStreamingConfig) {
        self.config = config.clamped();
    }
    pub fn clear(&mut self) {
        self.active.clear();
    }

    pub fn center_chunk_for_focus(
        &self,
        focus: StreamingFocus,
    ) -> Result<ChunkCoord3, SpatialMathError> {
        self.partition
            .chunk_coord_from_world_position(focus.position())
    }

    pub fn update_focus(
        &mut self,
        focus: StreamingFocus,
    ) -> Result<ChunkSetDiff, SpatialMathError> {
        let update = self.preview_focus_update(focus)?;
        Ok(self.apply_focus_update(update))
    }

    pub fn preview_focus_update(
        &self,
        focus: StreamingFocus,
    ) -> Result<ChunkFocusUpdate, SpatialMathError> {
        let center = self.center_chunk_for_focus(focus)?;
        let desired = self.build_chunk_set(
            center,
            self.config.load_radius_chunks,
            self.config.vertical_load_radius_chunks,
        )?;
        let retained = self.build_chunk_set(
            center,
            self.config.unload_radius_chunks,
            self.config.vertical_unload_radius_chunks,
        )?;
        let mut next = desired.clone();
        for chunk in self.active.iter() {
            if retained.contains(chunk) {
                next.insert(*chunk);
            }
        }
        let mut diff = diff_chunk_sets(&self.active, &next);
        sort_chunks(center, &mut diff.entered, self.config.load_order)?;
        sort_chunks(center, &mut diff.exited, self.config.load_order)?;
        Ok(ChunkFocusUpdate { center, next, diff })
    }

    pub fn apply_focus_update(&mut self, update: ChunkFocusUpdate) -> ChunkSetDiff {
        self.active = update.next;
        update.diff
    }

    pub fn desired_chunks_for_focus(
        &self,
        focus: StreamingFocus,
    ) -> Result<ChunkSet, SpatialMathError> {
        self.build_chunk_set(
            self.center_chunk_for_focus(focus)?,
            self.config.load_radius_chunks,
            self.config.vertical_load_radius_chunks,
        )
    }

    fn build_chunk_set(
        &self,
        center: ChunkCoord3,
        horizontal_radius: i32,
        vertical_radius: i32,
    ) -> Result<ChunkSet, SpatialMathError> {
        let horizontal = i64::from(horizontal_radius);
        let vertical = i64::from(vertical_radius);
        let x = checked_range(center.x, horizontal)?;
        let y = checked_range(center.y, vertical)?;
        let z = checked_range(center.z, horizontal)?;
        let mut set = ChunkSet::default();
        match self.config.mode {
            ChunkStreamingMode::PlanarXZ | ChunkStreamingMode::Volume3D => {
                for x in x.0..=x.1 {
                    for y in y.0..=y.1 {
                        for z in z.0..=z.1 {
                            set.insert(ChunkCoord3 { x, y, z });
                        }
                    }
                }
            }
        }
        Ok(set)
    }
}

fn checked_range(center: i64, radius: i64) -> Result<(i64, i64), SpatialMathError> {
    Ok((
        center
            .checked_sub(radius)
            .ok_or(SpatialMathError::ArithmeticOverflow {
                operation: "streaming chunk range",
            })?,
        center
            .checked_add(radius)
            .ok_or(SpatialMathError::ArithmeticOverflow {
                operation: "streaming chunk range",
            })?,
    ))
}

fn diff_chunk_sets(previous: &ChunkSet, next: &ChunkSet) -> ChunkSetDiff {
    let entered = next
        .iter()
        .filter(|chunk| !previous.contains(chunk))
        .copied()
        .collect();
    let exited = previous
        .iter()
        .filter(|chunk| !next.contains(chunk))
        .copied()
        .collect();
    ChunkSetDiff { entered, exited }
}

fn sort_chunks(
    center: ChunkCoord3,
    chunks: &mut [ChunkCoord3],
    order: ChunkLoadOrder,
) -> Result<(), SpatialMathError> {
    let mut ranked = chunks
        .iter()
        .copied()
        .map(|chunk| Ok((checked_distance_squared(chunk, center)?, chunk)))
        .collect::<Result<Vec<_>, SpatialMathError>>()?;
    if ranked.len() > u32::MAX as usize {
        return Err(SpatialMathError::ArithmeticOverflow {
            operation: "streaming priority rank",
        });
    }
    ranked.sort_by(
        |(left_distance, left_coord), (right_distance, right_coord)| {
            let distance_order = match order {
                ChunkLoadOrder::NearestFirst => left_distance.cmp(right_distance),
                ChunkLoadOrder::FarthestFirst => right_distance.cmp(left_distance),
            };
            distance_order.then_with(|| left_coord.cmp(right_coord))
        },
    );
    for (slot, (_, coord)) in chunks.iter_mut().zip(ranked) {
        *slot = coord;
    }
    Ok(())
}

fn checked_distance_squared(a: ChunkCoord3, b: ChunkCoord3) -> Result<i128, SpatialMathError> {
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
