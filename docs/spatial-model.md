# Spatial Model

`runen-spatial` owns deterministic spatial vocabulary:

- `WorldId`
- `WorldPosition`
- `FrameLocalPosition`
- `WorldFrame`
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

Stable chunk, region, and clipmap coordinates use signed `i64` components.
Chunk and region mapping use floor division so negative world and chunk
coordinates map consistently across chunk boundaries.

`WorldPosition` contains a `WorldId` and finite global `f64` meters.
`FrameLocalPosition` contains finite `f32` meters and can only be created or
interpreted through explicit checked conversion with a translation-only
`WorldFrame`. Hosts select another frame when they rebase.

Global-to-local-to-global conversion is intentionally bounded by the `f32`
rounding of the local value. Tests use an axis tolerance of
`f32::EPSILON * max(abs(local_axis), 1)` after conversion rather than claiming
bit-exact recovery for values not representable in `f32`.

`GridPartitionConfig` maps validated global positions to chunk coordinates and
chunk coordinates to region coordinates. Its edge and dimensions are checked;
invalid configuration is rejected rather than clamped.

`HierarchicalGridConfig` maps parent and first-child chunk coordinates without
owning LOD policy: level zero is finest and increasing levels are coarser.

`ClipmapConfig` and `RingBufferConfig` describe validated reusable coordinate
windows and physical slot mapping. They do not decide residency or own backing
storage. Configuration serialization is value-only and unversioned; position
and frame types define no persisted or wire schema.

## Bounds DTO

`SpatialAabb3` is intentionally minimal. It exists so `spatial_index` can be
standalone without depending on Runenwerk `geometry` or `glam`.

It supports:

- construction from points or arrays;
- validity checks;
- AABB intersection checks.

It does not replace a full geometry library.

## Spatial Hash

`runen_spatial::hash` owns payload-neutral deterministic integer hashing for spatial
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
