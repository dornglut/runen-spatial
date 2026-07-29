# Spatial Demand Model

`runen-spatial-demand` computes deterministic effective desired chunk coverage
from complete, host-defined source snapshots. It has no loading or payload
responsibility.

It answers:

```text
Given this partition and these complete source snapshots, which chunks are
effectively demanded, in which stable service order, and what changed?
```

It does not answer how a chunk is loaded, what payload it contains, when it is
visually ready, how it is saved, or how a host creates ECS content.

## Public Concepts

- `DemandSourceId` and `DemandSourcePriority` identify and order sources.
- `DemandFocus` owns a validated `WorldPosition`, desired and retain radii on
  horizontal and vertical axes, and a deterministic distance order.
- `DemandSourceSnapshot` is a complete replacement snapshot containing an
  optional focus and explicit pins.
- `DemandTransaction` applies a canonical, duplicate-free batch of source
  replacements and removals.
- `DemandLimits` bounds sources, source contributions, total contributions, and
  effective chunks.
- `EffectiveDemandSnapshot` contains `DemandedChunk` entries with
  `DemandRank`, winning `DemandClass`, and winning source identity.
- `SpatialDemandDelta` reports entered, updated, and exited effective entries.

## Composition and Retention

Every focus covers a checked axis-aligned box: horizontal radii apply to X/Z
and vertical radii apply to Y. A retain radius must be at least its matching
desired radius. Retention is source-local: a source may retain only its own
previous non-pinned focus contribution. Pins are explicit source contributions
and remain until their source replaces or removes them.

For each chunk, class precedence is `Pinned`, then `Desired`, then `Retained`.
Within a class, the larger source priority wins; ties use source-local ordinal,
source ID, and chunk identity. This produces a total deterministic service
order. Capacity selection keeps every pin, then the highest-ranked remaining
entries; an over-capacity pin set is rejected.

Source updates and limit replacement are atomic. Candidate computation,
including checked bounds and capacity checks, completes before the planner
commits source state or effective output. The callback variants commit only
after the caller has prepared its corresponding work.

`runen-spatial-streaming` consumes the effective delta and owns lifecycle
transitions. It does not duplicate demand geometry or source-priority policy.
