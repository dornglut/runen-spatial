use spatial::{
    SpatialHashSeed, finalize_spatial_hash, mix_spatial_hash_i64, spatial_hash_cell2,
    spatial_hash_cell3, spatial_hash_i64s,
};

#[test]
fn hash_vectors_are_stable() {
    assert_eq!(
        spatial_hash_cell2(SpatialHashSeed::new(1_337), 10, -4).value(),
        17_815_453_288_321_405_075
    );
    assert_eq!(
        spatial_hash_cell3(SpatialHashSeed::new(1_337), 10, 2, -4).value(),
        13_881_992_441_126_044_876
    );
    assert_eq!(
        spatial_hash_i64s(SpatialHashSeed::new(42), [-1, 0, 1, i64::MIN, i64::MAX]).value(),
        9_124_666_103_991_712_819
    );
}

#[test]
fn coordinate_order_is_significant() {
    let seed = SpatialHashSeed::new(99);

    assert_ne!(
        spatial_hash_cell2(seed, 4, 7),
        spatial_hash_cell2(seed, 7, 4)
    );
    assert_ne!(
        spatial_hash_cell3(seed, 4, 0, 7),
        spatial_hash_cell3(seed, 4, 7, 0)
    );
}

#[test]
fn negative_coordinates_are_distinct() {
    let seed = SpatialHashSeed::new(99);

    assert_ne!(
        spatial_hash_cell2(seed, -1, 2),
        spatial_hash_cell2(seed, 1, 2)
    );
    assert_ne!(
        mix_spatial_hash_i64(seed.value(), -1),
        mix_spatial_hash_i64(seed.value(), 1)
    );
}

#[test]
fn bucket_index_handles_zero_buckets() {
    let value = spatial_hash_cell2(SpatialHashSeed::new(3), -8, 12);

    assert_eq!(value.bucket_index(0), None);
    assert_eq!(value.bucket_index(100), Some(value.value() % 100));
}

#[test]
fn finalize_is_deterministic() {
    assert_eq!(
        finalize_spatial_hash(123).value(),
        finalize_spatial_hash(123).value()
    );
    assert_ne!(finalize_spatial_hash(123), finalize_spatial_hash(124));
}
