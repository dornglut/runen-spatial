# Spatial Model

This document explains the foundational spatial concepts implemented by `runen-spatial`. Exact public signatures and error variants are owned by the crate's rustdoc and tests.

## Namespace and identity

`WorldId` qualifies stable runtime spatial identities. Chunk, region, hierarchical, and clipmap identities must not silently cross world namespaces.

Chunk, region, and clipmap coordinates use signed 64-bit components. They are practical large-world addresses, not mathematical infinity; operations that can overflow must report failure instead of wrapping or saturating spatial identity.

## Global and local positions

`WorldPosition` is a validated world-qualified global metric position using finite `f64` components.

`FrameLocalPosition` is a validated finite `f32` position meaningful only relative to a `WorldFrame`. `WorldFrame` is translation-only. The host chooses frames/rebases and owns the consequences for rendering, physics, ECS state, or other systems.

RunenSpatial does not own a camera-relative frame, render origin, rebase scheduler, or movement event system.

## Partitioning

`GridPartitionConfig` maps validated global positions to chunk and region coordinates using positive metric scale and nonzero region dimensions. Negative coordinates follow mathematical floor semantics.

Conversions that can lose range, overflow, or cross namespaces are checked. Stable identities are never produced through silent float-to-integer saturation.

## Hierarchy

Hierarchy level zero is finest; larger levels are coarser. A parent is one level coarser and uses mathematical floor division by the positive level scale factor. Child bounds and scale arithmetic are checked.

Hierarchy math is address composition only. It does not imply storage, residency, generation, traversal scheduling, or an octree runtime.

## Clipmaps and rings

Clipmap and ring APIs are mapping primitives. They define validated level/window/slot relationships and checked coordinate mapping, but do not own cache residency, generation reuse, synchronization, GPU resources, or streaming policy.

## Serialization

Serde support is value serialization only. The repository does not currently promise a stable persisted schema, wire protocol, replay format, or cross-version migration contract.
