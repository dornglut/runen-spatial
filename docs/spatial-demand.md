# Spatial Demand

This document explains the current demand responsibility and implemented contract. Exact public signatures belong to `runen-spatial-demand` rustdoc and tests. Future sequencing belongs in [ROADMAP.md](../ROADMAP.md).

## Ownership

Spatial demand answers which world-qualified chunks a host wants considered and in what deterministic order. It owns only host-neutral source composition, validated box geometry, source-local hysteresis, explicit pins, effective rank, bounded pressure, and deterministic deltas.

It does not own payload/product identity, IO, async execution, load completion, cache residency, gameplay activation, rendering, replication, retry/degradation policy, host-assigned source priority, or engine-specific visibility geometry.

## Planner contract

One planner is bound to one `WorldId`, one grid partition, and immutable demand limits. A source is identified by an opaque source ID and is completely replaced or removed. A source snapshot contains an optional focus and/or explicit world-qualified pins.

Focus geometry is the proven symmetric X/Z plus independent Y axis-aligned box: horizontal and vertical desired/retain radii are unsigned and validated so retain radii cannot be smaller than desired radii. There is no geometry-mode selector.

Hysteresis is source-local. Only chunks previously contributed by a source may survive as retained chunks for that source. Removing a source removes its retained history.

## Ordering and pressure

Effective classes order pinned, desired, then retained. Within a class, equal sources interleave by source-local nearest-first ordinal, then source ID and chunk coordinate. Source ID is only a deterministic tie-breaker; the framework does not own host source-priority policy.

The planner bounds source count, contributions per source, total contributions, and effective chunks. Desired-box cardinality is checked before materialization. Hard-limit or world/coordinate failures reject the whole source-change batch without mutation.

`DemandLimits::default()` is only an opt-in convenience profile; planner and streaming constructors still require an explicit limits value, and callers may provide stricter bounds.

When valid combined demand exceeds effective capacity, the highest-ranked non-pinned candidates are selected deterministically. Pins are never silently suppressed; unique pinned demand beyond effective capacity is an error. Suppressed candidates can re-enter on later successful source changes without a separate suppression state API.

## Streaming composition

Effective demand is expressed as `ChunkId` plus compact deterministic rank/class information. Deltas report entered, updated/reranked, and exited chunks. The availability controller consumes those ranks directly, so still-demanded queued work can be reprioritized without unload/reload churn while issued requests retain their correlation identity.
