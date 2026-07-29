use runen_spatial::{SpatialAabb3, SpatialPoint3};

#[test]
fn spatial_aabb_rejects_non_finite_and_inverted_bounds() {
    assert!(!SpatialAabb3::from_arrays([0.0, 0.0, 0.0], [-1.0, 1.0, 1.0]).is_valid());
    assert!(!SpatialAabb3::from_arrays([0.0, f32::NAN, 0.0], [1.0, 1.0, 1.0]).is_valid());
}

#[test]
fn spatial_aabb_intersection_is_inclusive() {
    let a = SpatialAabb3::from_arrays([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    let b = SpatialAabb3::from_arrays([1.0, 1.0, 1.0], [2.0, 2.0, 2.0]);
    let c = SpatialAabb3::from_arrays([1.1, 1.1, 1.1], [2.0, 2.0, 2.0]);

    assert!(a.intersects(&b));
    assert!(!a.intersects(&c));
}

#[test]
fn spatial_point_round_trips_array_values() {
    let point = SpatialPoint3::from_array([2.0, -3.0, 4.5]);

    assert_eq!(point.to_array(), [2.0, -3.0, 4.5]);
    assert_eq!(SpatialPoint3::new(2.0, -3.0, 4.5), point);
}
