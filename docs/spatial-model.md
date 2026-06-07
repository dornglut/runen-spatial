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
- `SpatialHashSeed`
- `SpatialHashValue`

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

## Spatial Hash

`spatial::hash` owns payload-neutral deterministic integer hashing for spatial
keys. It can hash ordered signed integer coordinates such as 2D or 3D world
cells and map a hash into a bucket index.

The hash API does not own:

- generation rules;
- wall thresholds;
- tile descriptors;
- biome, material, or asset policy;
- residency or streaming lifecycle;
- SDF, ECS, renderer, save, or provider behavior.

Generation systems may use the hash as an input, but any meaning assigned to
the hash stays in the host or a future generation crate.
