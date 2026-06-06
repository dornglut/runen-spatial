use spatial::SpatialAabb3;
use spatial_index::{
    AabbQuery, MutableSpatialIndex, SpatialEntry, SpatialHashConfig, SpatialHashIndex, SpatialIndex,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct TestKey(u32);

fn aabb(min: [f32; 3], max: [f32; 3]) -> SpatialAabb3 {
    SpatialAabb3::from_arrays(min, max)
}

#[test]
fn insert_and_query_single_entry() {
    let mut index = SpatialHashIndex::<TestKey>::new(SpatialHashConfig { cell_size: 1.0 }).unwrap();

    index
        .insert(SpatialEntry::new(
            TestKey(1),
            aabb([0.0, 0.0, 0.0], [0.5, 0.5, 0.5]),
        ))
        .unwrap();

    let result = index
        .query_aabb(AabbQuery::new(aabb(
            [-0.25, -0.25, -0.25],
            [0.25, 0.25, 0.25],
        )))
        .unwrap();

    assert_eq!(result.keys, vec![TestKey(1)]);
}

#[test]
fn query_filters_false_candidates_by_actual_bounds() {
    let mut index = SpatialHashIndex::<TestKey>::new(SpatialHashConfig { cell_size: 1.0 }).unwrap();

    index
        .insert(SpatialEntry::new(
            TestKey(1),
            aabb([0.0, 0.0, 0.0], [0.1, 0.1, 0.1]),
        ))
        .unwrap();

    index
        .insert(SpatialEntry::new(
            TestKey(2),
            aabb([0.9, 0.9, 0.9], [1.1, 1.1, 1.1]),
        ))
        .unwrap();

    let result = index
        .query_aabb(AabbQuery::new(aabb([0.0, 0.0, 0.0], [0.2, 0.2, 0.2])))
        .unwrap();

    assert_eq!(result.keys, vec![TestKey(1)]);
}

#[test]
fn update_moves_entry_between_cells() {
    let mut index = SpatialHashIndex::<TestKey>::new(SpatialHashConfig { cell_size: 1.0 }).unwrap();

    index
        .insert(SpatialEntry::new(
            TestKey(1),
            aabb([0.0, 0.0, 0.0], [0.2, 0.2, 0.2]),
        ))
        .unwrap();

    index
        .update(SpatialEntry::new(
            TestKey(1),
            aabb([5.0, 5.0, 5.0], [5.2, 5.2, 5.2]),
        ))
        .unwrap();

    let old_result = index
        .query_aabb(AabbQuery::new(aabb([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0])))
        .unwrap();

    let new_result = index
        .query_aabb(AabbQuery::new(aabb([4.5, 4.5, 4.5], [5.5, 5.5, 5.5])))
        .unwrap();

    assert!(old_result.keys.is_empty());
    assert_eq!(new_result.keys, vec![TestKey(1)]);
}

#[test]
fn remove_erases_entry() {
    let mut index = SpatialHashIndex::<TestKey>::new(SpatialHashConfig { cell_size: 1.0 }).unwrap();

    index
        .insert(SpatialEntry::new(
            TestKey(1),
            aabb([0.0, 0.0, 0.0], [0.2, 0.2, 0.2]),
        ))
        .unwrap();

    assert!(index.remove(TestKey(1)));
    assert!(!index.contains(TestKey(1)));
    assert_eq!(index.len(), 0);
}

#[test]
fn supports_negative_coordinates() {
    let mut index = SpatialHashIndex::<TestKey>::new(SpatialHashConfig { cell_size: 1.0 }).unwrap();

    index
        .insert(SpatialEntry::new(
            TestKey(1),
            aabb([-2.5, -2.5, -2.5], [-1.5, -1.5, -1.5]),
        ))
        .unwrap();

    let result = index
        .query_aabb(AabbQuery::new(aabb([-3.0, -3.0, -3.0], [-1.0, -1.0, -1.0])))
        .unwrap();

    assert_eq!(result.keys, vec![TestKey(1)]);
}
