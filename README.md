# RunenSpatial

RunenSpatial is a host-neutral Rust framework for spatial identities, addressing, indexing, deterministic demand, and content-agnostic chunk availability control.

The repository was transferred to Dornglut with commit history preserved. It remains private while package identity, correctness, conformance, and release policy are completed.

## Ownership boundary

RunenSpatial may own:

- spatial namespace, chunk, and region identities;
- global and frame-local coordinate mathematics;
- grid, hierarchy, clipmap, ring-mapping, bounds, and neutral spatial-hash primitives;
- generic spatial indexes;
- deterministic spatial demand;
- abstract load/unload lifecycle for one host-defined availability class per controller, including correlation, budgets, transition outcomes, and diagnostics.

It does not own world edits, SDF or other product payloads, procgen, simulation, ECS activation, replication, save formats, renderer caches, image formation, GPU resources, IO, async runtime ownership, application recovery policy, a universal product registry, or Godot scene and asset ownership.

Start with [Architecture](ARCHITECTURE.md), [Agent Guide](AGENTS.md), and [Testing](TESTING.md).

## Current workspace

The current workspace uses explicit RunenSpatial package identities:

```text
crates/runen_spatial
crates/runen_spatial_index
crates/runen_spatial_demand
crates/runen_spatial_streaming
adapters/godot_world_streaming
demos/chunk_streaming_demo
```

The [package boundaries](docs/package-boundaries.md) record the accepted package map and decision gates. The non-owning `world_core_prelude` package has been removed; consumers import directly from the package that owns each concept.

The Godot adapter remains optional, non-default, experimental, and non-publishable pending an ownership and lifecycle audit.

## Current maturity

The current code provides a deterministic single-focus demand and streaming-lifecycle baseline with checked spatial addressing and index primitives. Stable chunk, region, and clipmap coordinates are signed `i64`; global positions include their `WorldId`; and frame-local positions are explicitly converted through a translation-only `WorldFrame`.

Implemented evidence includes negative-coordinate partitioning, planar and axis-aligned three-dimensional desired chunk sets, load/unload hysteresis, persistent budgeted request queues, request correlation, deterministic reversal behavior, explicit retry, stale-event rejection, and an optional Godot adapter.

Known limitations include single-source demand, stale queued-priority risk, one combined lifecycle with a universal `Failed` state, inherited post-load payload failure reporting, saturating request-ID allocation, unaudited index and adapter characteristics, and no completed Runenwerk cutover.

RunenSpatial is not yet a production-complete infinite-world or multi-consumer streaming framework.

## Authority and roadmap

- [Canonical architecture](docs/architecture.md)
- [Package boundaries](docs/package-boundaries.md)
- [Transfer provenance](docs/provenance/repository-transfer.md)
- [Validation contract](docs/tooling/validation.md)
- [Durable roadmap](docs/roadmap.md)
- [Parent outcome issue](https://github.com/dornglut/runen-spatial/issues/1)

Issues own accepted live work; begin at the [parent outcome issue](https://github.com/dornglut/runen-spatial/issues/1) and follow its current child. Pull requests and exact-head CI own implementation evidence. Durable documents do not track active branch, PR, SHA, or workflow inventories.

## Licensing

The current revision is licensed under the GNU General Public License version 3 only (`GPL-3.0-only`). See [LICENSE](LICENSE).

A separate commercial license may be available from copyright holder(s) with sufficient rights to grant it. Historical licensing, third-party material, and commercial-licensing boundaries are described in [LICENSING.md](LICENSING.md).

## Contributions

Issue reports, design discussion, reviews, and reproducible cases that do not add third-party repository content are welcome under the repository's normal governance.

Until reviewed inbound contribution terms preserve the rights needed for separate commercial licensing, external pull requests that add tracked repository content are not accepted.

## Validation

Run the complete maintained validation surface:

```text
cargo validate
```

The command is invoked unchanged by Dornglut's immutable shared Rust-validation workflow. Focused commands are useful during editing but are not substitute merge evidence.

## Runenwerk status

Runenwerk has not completed a dependency cutover. Future cutover work will be separately authorized and performed component by component, with real consumer migration and immediate duplicate-source deletion.
