# Spatial Model

`spatial` owns deterministic spatial vocabulary:

- `WorldId`
- `WorldPosition`
- `WorldLocalPosition`
- `WorldFrame`
- `CameraRelativeFrame`
- `ChunkCoord3`
- `RegionCoord3`
- `ChunkId`
- `RegionId`
- `GridPartitionConfig`
- `HierarchicalGridConfig`
- `ClipmapConfig`
- `ClipmapWindow`
- `RingBufferConfig`
- `SpatialPoint3`
- `SpatialAabb3`

## Coordinate Rules

Chunk and region mapping use floor division so negative world and chunk
coordinates map consistently across chunk boundaries.

`GridPartitionConfig` maps world-local meter positions to chunk coordinates and
chunk coordinates to region coordinates.

`HierarchicalGridConfig` maps parent and first-child chunk coordinates without
owning LOD policy.

`ClipmapConfig` and `RingBufferConfig` describe reusable coordinate windows and
physical slot mapping. They do not decide residency or own backing storage.

## Bounds DTO

`SpatialAabb3` is intentionally minimal. It exists so `spatial_index` can be
standalone without depending on Runenwerk `geometry` or `glam`.

It supports:

- construction from points or arrays;
- validity checks;
- AABB intersection checks.

It does not replace a full geometry library.
