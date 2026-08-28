# Spatial Demand

This document explains the current demand responsibility and baseline. Exact public signatures belong to `runen-spatial-demand` rustdoc and tests. Future redesign sequencing belongs in [ROADMAP.md](../ROADMAP.md).

## Ownership

Spatial demand answers which chunk coordinates a host wants considered and in what deterministic order. It may own geometry-to-demand calculation, hysteresis, source composition, rank, and pressure only when those rules are host-neutral.

It does not own payload/product identity, IO, async execution, load completion, cache residency, gameplay activation, rendering, replication, or retry/degradation policy.

## Current baseline

The current package is still a single-focus planner. A validated focus is mapped through the grid partition into an axis-aligned chunk box. Desired and retain radii provide hysteresis, and ordering determines deterministic service order.

The inherited `PlanarXZ` and `Volume3D` modes currently execute the same three-axis box construction; their observable distinction is supplied by vertical radii. They must not be treated as two proven geometry algorithms.

Configuration currently contains legacy clamping behavior and the package exposes legacy single-focus names. Those are current implementation facts, not target contracts.

## Required long-term properties

Any retained demand model must remain deterministic, bounded before large allocation, atomic on fallible updates, world-qualified, and independent of product/application purpose. Multi-source composition must define complete source identity/replacement/removal, source-local hysteresis, pins, total ordering, pressure behavior, and deterministic deltas before becoming the public contract.
