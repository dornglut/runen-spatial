# RunenSpatial Package Boundaries

## Status

This document defines the accepted package-planning baseline. It preserves the transferred source topology while replacing inherited generic names and broad re-exports with explicit Runen-family ownership.

The final published package set remains subject to the spatial-index audit and downstream-consumer evidence.

## Target package map

| Current package | Target package | Owns | Must not own | Decision |
| --- | --- | --- | --- | --- |
| `spatial` | `runen-spatial` | namespace identities, chunk/region addresses, positions, frames, partitions, hierarchy, clipmap/ring mappings, bounds, neutral spatial hashes | demand membership, payload lifecycle, IO, products, ECS, rendering, GPU, host policy | retain as foundational package |
| `spatial_index` | `runen-spatial-index` | generic spatial-entry and query contracts over RunenSpatial identities/bounds | product meaning, collision response, ECS storage, world invalidation, rendering acceleration | provisional; retain only after audit and consumer proof |
| `chunking` | `runen-spatial-demand` | deterministic multi-source desired spatial coverage, priority, retention, pinning, membership diffs | actual payload availability, backend work, IO, product purpose | retain and redesign terminology/API through a dedicated issue |
| `world_streaming` | `runen-spatial-streaming` | demand-to-availability reconciliation, request IDs, budgets, backend events, transition state, failure facts, diagnostics | backend implementation, async runtime, payload types, retries/backoff policy, product readiness | retain and harden through a dedicated issue |
| `world_core_prelude` | none | inherited broad re-exports | any durable authority | delete; no replacement façade |
| `godot_world_streaming` | undecided | optional translation between Godot values/signals and public streaming contracts | core policy, Godot world ownership, visuals, generation, cache policy | retain temporarily as experimental, non-default, non-publishable adapter |
| `chunk_streaming_demo` | example/conformance | public-API demonstration | architecture or dependency authority | retain; remove external project coupling and use as conformance evidence |

## Why the framework is not one package by default

The current dependency graph exposes plausible independent consumer subsets:

```text
runen-spatial

runen-spatial-index
    -> runen-spatial

runen-spatial-demand
    -> runen-spatial

runen-spatial-streaming
    -> runen-spatial
    -> runen-spatial-demand
```

Examples of legitimate narrow consumption:

- a renderer or editor may need stable addresses and frames without demand or streaming;
- a tool may need demand calculation without payload lifecycle;
- a host may need spatial indexing without streaming;
- a world runtime may use the complete stack.

This establishes technical separability. It does not by itself prove independent publication or versioning. Every retained package must also justify:

- an independently understandable public contract;
- at least one meaningful consumer boundary;
- dependency or compile-cost value;
- ownership clarity greater than the maintenance cost;
- an intentional compatibility and release policy.

## Foundational package

`runen-spatial` is the lowest-level public authority.

It may expose:

- `WorldId` as an opaque spatial namespace;
- chunk and region coordinates and IDs;
- global and local positions;
- world/local frames and checked conversion;
- grid partitions and hierarchy addresses;
- clipmap/ring address functions;
- minimal bounds and neutral hash helpers.

It must not contain mutable desired sets, streaming records, backend events, product tags, engine resources, or adapter code.

## Index package decision gate

`runen-spatial-index` remains provisional until it proves:

- complete insert/update/remove invariants;
- explicit duplicate-key behavior;
- deterministic query ordering or documented unordered behavior;
- checked extreme-coordinate behavior;
- acceptable adversarial hash behavior;
- bounded cleanup and memory growth;
- documented complexity claims;
- no unsupported concurrency or persistence claims;
- representative benchmarks;
- a real independent consumer.

Possible accepted outcomes are retain, narrow, merge into `runen-spatial`, or defer from the public release surface.

## Demand package

`runen-spatial-demand` owns desired coverage, not loading.

It accepts host-defined demand sources and produces deterministic effective demand. It may own:

- source IDs;
- source snapshots;
- neutral priorities;
- hysteresis/retention;
- explicit pinning;
- stable merge and diff ordering.

It must not own:

- product-purpose enums;
- IO or task execution;
- actual payload availability;
- cache eviction;
- retry, timeout, or degradation policy.

## Streaming package

`runen-spatial-streaming` owns deterministic payload-availability transitions.

It consumes effective demand and emits abstract load/unload requests. Hosts report backend events.

It may own:

- per-chunk desired, availability, operation, and failure facts;
- request IDs and correlation;
- per-update request budgets;
- deterministic request ordering;
- stale/mismatched event rejection;
- pressure and transition diagnostics.

It must not own:

- a required async runtime or backend trait;
- filesystem/network IO;
- payload types, caches, or persistence;
- application retry/backoff/cancellation policy;
- product formation, certification, activation, or rendering.

## Prelude deletion

`world_core_prelude` is accepted for deletion because it:

- hides the owning package for each concept;
- preserves a misleading world-core identity;
- encourages wildcard imports;
- makes dependency and compatibility growth difficult to review;
- has no independent algorithm, state, platform dependency, or consumer contract.

Consumers will migrate directly to the owning packages. A forwarding crate or replacement wildcard prelude is not the final state.

## Adapter boundary

The Godot adapter remains outside core default members and remains `publish = false`.

It may translate:

- `Vector3` and Godot configuration values;
- controller updates;
- requests and backend events;
- lifecycle diagnostics into Godot-friendly signals.

It must not define reusable demand policy, own nodes beyond adapter behavior, generate world content, allocate visual assets, or become the source of core invariants.

Permanent package ownership is decided only after the adapter audit.

## Package naming and publication

Target Cargo package names use the `runen-` prefix; Rust crate names use underscores.

All packages remain unpublished until:

- package boundaries are accepted;
- package metadata and repository links are normalized;
- `world_core_prelude` is removed;
- provenance and licensing are validated;
- standalone conformance passes;
- compatibility/versioning policy is documented;
- public visibility is explicitly approved.

## Change control

A package may be added, merged, split, or made public only through an accepted issue that proves the ownership and dependency boundary. Historical directory structure is not sufficient justification.