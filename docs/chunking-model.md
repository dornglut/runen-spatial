# Spatial Demand Model

`runen-spatial-demand` computes desired chunk residency around a focus.

It answers:

```text
Given this partition, focus, and policy, which chunk coordinates should be
desired, retained, entered, or exited?
```

It does not answer:

```text
How is a chunk loaded?
What payload is loaded?
When is a visual ready?
How is a chunk saved?
How does ECS spawn content?
```

## Public Concepts

- `StreamingFocus`: focus position in meters.
- `ChunkStreamingConfig`: load/unload radii and mode knobs.
- `ChunkStreamingMode`: planar XZ or full 3D volume.
- `ChunkLoadOrder`: nearest-first or farthest-first deterministic ordering.
- `ChunkSet`: set of chunk coordinates.
- `ChunkSetDiff`: entered/exited chunk coordinates.
- `ChunkStreamer`: stateful desired-residency planner.

## Hysteresis

Load radius and unload radius are separate. Chunks inside the unload radius can
be retained even after leaving the load radius, preventing unnecessary churn when
the focus moves near chunk boundaries.

Lifecycle transitions belong in `runen-spatial-streaming`, not here.
