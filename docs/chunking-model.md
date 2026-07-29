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

- `StreamingFocus`: one validated namespaced `WorldPosition`.
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

The planner returns checked spatial-math errors for coordinate range failures;
an invalid focus update does not replace the active desired set. Its public
focus-update surface is atomic: `update_focus` returns the computed diff, and
`update_focus_with` commits only after its callback has prepared any additional
fallible work. No independently applicable candidate-update token is exposed.
Multi-source demand is RS4 work and is not part of this current implementation.
