# RunenSpatial

RunenSpatial is a host-neutral Rust framework for spatial identities,
addressing, indexing, demand calculation, and content-agnostic chunk
availability control.

The repository was transferred to Dornglut with commit history preserved. It is
private while its governance, package identity, provenance, large-world
contracts, demand model, lifecycle semantics, conformance, and release policy
are normalized.

## Ownership boundary

RunenSpatial may own:

- spatial namespace, chunk, and region identities;
- global and frame-local coordinate mathematics;
- grid, hierarchy, clipmap, ring-mapping, bounds, and neutral spatial-hash
  primitives;
- generic spatial indexes;
- deterministic spatial demand;
- abstract load/unload lifecycle for one host-defined availability class per
  controller, including request correlation, budgets, transition outcomes, and
  diagnostics.

It does not own world edits, SDF or other product payloads, procgen, simulation,
ECS activation, replication, save formats, renderer caches, image formation,
GPU resources, IO, async runtime ownership, application recovery policy, a
universal product registry, or Godot scene and asset ownership.

See [the canonical architecture](docs/architecture.md).

## Current transferred workspace

The current source topology remains unchanged during the decision phase:

```text
crates/spatial
crates/spatial_index
crates/chunking
crates/world_streaming
crates/world_core_prelude
adapters/godot_world_streaming
demos/chunk_streaming_demo
```

The target Runen-family names and final package set are not yet implemented. The
provisional mapping and package decision gates are documented in
[package boundaries](docs/package-boundaries.md).

`world_core_prelude` is accepted for later deletion; it is not the target
ordinary entry point.

The Godot adapter remains optional, non-default, experimental, and
non-publishable pending an ownership and lifecycle audit.

## Current maturity

The transferred code provides a deterministic single-focus demand and
streaming-lifecycle baseline with spatial addressing and index primitives.

Implemented evidence includes:

- negative-coordinate partitioning;
- planar and axis-aligned three-dimensional desired chunk sets;
- load/unload hysteresis;
- persistent budgeted request queues;
- request and chunk correlation;
- deterministic load/unload reversal behavior;
- explicit retry and stale-event rejection;
- an optional Godot adapter and demo.

Known limitations include:

- contradictory hierarchy level/parent semantics;
- incomplete large-world, overflow, rebasing, persistence, and precision
  contracts;
- single-source demand and stale queued-priority risk;
- one combined lifecycle with a universal `Failed` state;
- inherited resident-payload failure reporting that crosses the target
  transition boundary;
- saturating request-ID allocation;
- unaudited production characteristics for the spatial index and Godot adapter;
- no completed full-history release-provenance audit;
- no completed Runenwerk consumer cutover.

RunenSpatial must therefore not yet be described as a production-complete
infinite-world or multi-consumer streaming framework.

## Planning authority

- [Parent outcome: establish standalone RunenSpatial authority](https://github.com/dornglut/runen-spatial/issues/1)
- [Current decision phase: investigate the extraction boundary](https://github.com/dornglut/runen-spatial/issues/2)
- [Extraction-boundary investigation](docs/investigations/runenspatial-extraction-boundary.md)
- [Architecture](docs/architecture.md)
- [Package boundaries](docs/package-boundaries.md)
- [Durable roadmap](docs/roadmap.md)

GitHub issues own accepted live work. Pull requests and exact-head CI own
implementation evidence. Durable documents do not track active branch heads,
pull-request inventories, or workflow runs.

## Validation

The transferred baseline currently supports focused Cargo commands such as:

```text
cargo fmt --all --check
cargo test --workspace
```

The accepted target is one repository-owned `cargo validate` command invoked by
Dornglut's immutable shared Rust-validation workflow. That authority will be
added in a separate behavior-preserving governance issue after the decision
phase is accepted.

Optional adapter and demo validation remain separate until their permanent
ownership and toolchain contracts are decided.

## Runenwerk status

Runenwerk has not completed a dependency cutover to this repository. It still
contains duplicate internal spatial, demand, and streaming source and a mixed
engine chunk lifecycle.

A future Runenwerk cutover will be separately authorized and will proceed
component by component. Each accepted slice must migrate real consumers and
delete the corresponding duplicate authority without leaving forwarding crates
as the final state.
