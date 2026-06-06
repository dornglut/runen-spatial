use crate::{
    AabbQuery, MutableSpatialIndex, QueryResult, SpatialEntry, SpatialIndex, SpatialIndexError,
    SpatialKey,
};
use spatial::SpatialAabb3;
use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SpatialHashConfig {
    pub cell_size: f32,
}

impl Default for SpatialHashConfig {
    fn default() -> Self {
        Self { cell_size: 1.0 }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
struct CellCoord {
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct IndexedEntry {
    bounds: SpatialAabb3,
    min_cell: CellCoord,
    max_cell: CellCoord,
}

pub struct SpatialHashIndex<K: SpatialKey> {
    cell_size: f32,
    cells: HashMap<CellCoord, BTreeSet<K>>,
    entries: HashMap<K, IndexedEntry>,
}

impl<K: SpatialKey> SpatialHashIndex<K> {
    pub fn new(config: SpatialHashConfig) -> Result<Self, SpatialIndexError> {
        if !config.cell_size.is_finite() || config.cell_size <= 0.0 {
            return Err(SpatialIndexError::InvalidCellSize {
                cell_size: config.cell_size,
            });
        }

        Ok(Self {
            cell_size: config.cell_size,
            cells: HashMap::new(),
            entries: HashMap::new(),
        })
    }

    fn ensure_valid_bounds(bounds: SpatialAabb3) -> Result<(), SpatialIndexError> {
        if !bounds.is_valid() {
            return Err(SpatialIndexError::InvalidBounds);
        }
        Ok(())
    }

    fn axis_to_cell(&self, value: f32) -> i32 {
        (value / self.cell_size).floor() as i32
    }

    fn cell_range_for_bounds(&self, bounds: SpatialAabb3) -> (CellCoord, CellCoord) {
        (
            CellCoord {
                x: self.axis_to_cell(bounds.min.x),
                y: self.axis_to_cell(bounds.min.y),
                z: self.axis_to_cell(bounds.min.z),
            },
            CellCoord {
                x: self.axis_to_cell(bounds.max.x),
                y: self.axis_to_cell(bounds.max.y),
                z: self.axis_to_cell(bounds.max.z),
            },
        )
    }

    fn remove_key_from_cells(&mut self, key: K, min_cell: CellCoord, max_cell: CellCoord) {
        for x in min_cell.x..=max_cell.x {
            for y in min_cell.y..=max_cell.y {
                for z in min_cell.z..=max_cell.z {
                    let coord = CellCoord { x, y, z };
                    let mut should_remove = false;

                    if let Some(occupants) = self.cells.get_mut(&coord) {
                        occupants.remove(&key);
                        should_remove = occupants.is_empty();
                    }

                    if should_remove {
                        self.cells.remove(&coord);
                    }
                }
            }
        }
    }

    fn insert_key_into_cells(&mut self, key: K, min_cell: CellCoord, max_cell: CellCoord) {
        for x in min_cell.x..=max_cell.x {
            for y in min_cell.y..=max_cell.y {
                for z in min_cell.z..=max_cell.z {
                    self.cells
                        .entry(CellCoord { x, y, z })
                        .or_default()
                        .insert(key);
                }
            }
        }
    }

    fn upsert_internal(&mut self, key: K, bounds: SpatialAabb3) {
        let (min_cell, max_cell) = self.cell_range_for_bounds(bounds);

        if let Some(previous) = self.entries.remove(&key) {
            self.remove_key_from_cells(key, previous.min_cell, previous.max_cell);
        }

        self.insert_key_into_cells(key, min_cell, max_cell);
        self.entries.insert(
            key,
            IndexedEntry {
                bounds,
                min_cell,
                max_cell,
            },
        );
    }
}

impl<K: SpatialKey> SpatialIndex<K> for SpatialHashIndex<K> {
    fn contains(&self, key: K) -> bool {
        self.entries.contains_key(&key)
    }

    fn bounds(&self, key: K) -> Option<SpatialAabb3> {
        self.entries.get(&key).map(|entry| entry.bounds)
    }

    fn query_aabb(&self, query: AabbQuery) -> Result<QueryResult<K>, SpatialIndexError> {
        Self::ensure_valid_bounds(query.bounds)?;

        let (min_cell, max_cell) = self.cell_range_for_bounds(query.bounds);
        let mut candidates = BTreeSet::new();

        for x in min_cell.x..=max_cell.x {
            for y in min_cell.y..=max_cell.y {
                for z in min_cell.z..=max_cell.z {
                    if let Some(occupants) = self.cells.get(&CellCoord { x, y, z }) {
                        candidates.extend(occupants.iter().copied());
                    }
                }
            }
        }

        let mut keys = Vec::new();
        for key in candidates {
            let Some(entry) = self.entries.get(&key) else {
                continue;
            };

            if entry.bounds.intersects(&query.bounds) {
                keys.push(key);
            }
        }

        Ok(QueryResult::new(keys))
    }

    fn len(&self) -> usize {
        self.entries.len()
    }
}

impl<K: SpatialKey> MutableSpatialIndex<K> for SpatialHashIndex<K> {
    fn insert(&mut self, entry: SpatialEntry<K>) -> Result<(), SpatialIndexError> {
        Self::ensure_valid_bounds(entry.bounds)?;
        self.upsert_internal(entry.key, entry.bounds);
        Ok(())
    }

    fn update(&mut self, entry: SpatialEntry<K>) -> Result<(), SpatialIndexError> {
        Self::ensure_valid_bounds(entry.bounds)?;
        self.upsert_internal(entry.key, entry.bounds);
        Ok(())
    }

    fn upsert(&mut self, entry: SpatialEntry<K>) -> Result<(), SpatialIndexError> {
        Self::ensure_valid_bounds(entry.bounds)?;
        self.upsert_internal(entry.key, entry.bounds);
        Ok(())
    }

    fn remove(&mut self, key: K) -> bool {
        let Some(previous) = self.entries.remove(&key) else {
            return false;
        };

        self.remove_key_from_cells(key, previous.min_cell, previous.max_cell);
        true
    }

    fn clear(&mut self) {
        self.cells.clear();
        self.entries.clear();
    }
}
