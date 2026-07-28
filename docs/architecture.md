# RunenSpatial Architecture

## Mission

RunenSpatial is a standalone, host-neutral Rust framework for spatial identity,
addressing, indexing, demand calculation, and content-agnostic chunk
availability control.

It provides reusable mechanics. It does not define what a world contains, how
products are generated, how a renderer forms an image, or how a host performs
IO.

## Architectural rule

One concept has one owner.

```text
RunenSpatial
    where spatial cells are
    which spatial cells are desired
    whether one host-defined availability class is absent or resident at a cell

Runenwerk
    what world data means
    which products are required
    how products are generated, invalidated, certified, activated, saved, and replicated

RunenRender
    which prepared render products and acceleration caches are resident
    how visibility, interaction, lighting, transport, reconstruction, and image output work

RunenGPU
    how logical GPU resources and work are realized, executed, completed, and retired
```

No universal residency manager spans these layers.

## Owned concepts

RunenSpatial owns:

- `WorldId` as an opaque spatial-namespace identifier;
- chunk and region coordinates and identities;
- global and frame-local positions;
- explicit spatial frames and conversion mathematics;
- grid partitioning;
- hierarchical address relationships;
- clipmap and ring address mappings;
- minimal spatial bounds and neutral spatial hashes;
- generic spatial-index contracts;
- deterministic multi-source spatial demand;
- content-agnostic load/unload lifecycle for one host-defined availability class
  per controller instance;
- request identities, budgets, correlation, transition outcomes, and diagnostics.

## Excluded concepts

RunenSpatial must not own:

- signed-distance functions or sparse field products;
- world edits, dirty regions, product generations, build queues, procgen,
  simulation, replication, or save formats;
- collision certification, navigation readiness, visual fallback, gameplay
  locks, or ECS activation;
- renderer providers, page tables, render caches, materials, lighting, image
  history, or presentation;
- GPU adapters, devices, queues, resources, shaders, pipelines, command
  submission, or completion;
- async executors, threads, channels, filesystem/network IO, task handles,
  payload caches, retry backoff, or application recovery policy;
- Godot scenes, nodes, assets, meshes, materials, or project-specific signals.

## Package dependency direction

The provisional target packages are:

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

`runen-spatial` is the foundational dependency and must not depend on the other
packages.

`runen-spatial-index` is retained only if its audit proves an independent
reusable contract and consumer value.

`runen-spatial-demand` calculates desired spatial coverage. It performs no IO
and owns no actual availability state.

`runen-spatial-streaming` reconciles effective demand with one neutral,
host-defined availability class. It does not know product types or perform
backend work.

No broad prelude package is part of the target architecture.

## Address model

### Spatial namespaces

A `WorldId` identifies one independent spatial coordinate namespace. It does not
confer authority over a Runenwerk world model or application state.

Every stable chunk and region identity includes the namespace identifier.

### Stable coordinates

Target stable chunk and region coordinates use signed 64-bit components.

Stable coordinates:

- support negative values;
- are independent of camera or simulation origins;
- use checked arithmetic;
- are suitable as runtime map keys;
- do not imply a stable serialized or wire schema without a separately accepted
  version contract.

The framework describes this as a large-world address model, not mathematical
infinity.

### Position tiers

```text
stable chunk/region address
    signed integer identity

global metric position
    finite f64 with explicit namespace/frame context

frame-local position
    finite f32 relative to an explicit frame
```

Frame-local positions are not stable identities. Hosts decide when to rebase and
how systems observe rebasing; RunenSpatial owns only frame definitions and
checked conversions.

## Hierarchy model

Hierarchy uses the convention:

```text
level 0 = finest
larger level = coarser
```

For scale factor `s >= 2`:

```text
parent(level, coord)
    = (level + 1, floor_div(coord, s))

children(level, coord)
    = level - 1 cells covering
      [coord * s, coord * s + (s - 1)] on every axis
```

All arithmetic is checked. Tests must prove containment, complete child
coverage, negative-coordinate behavior, and level-bound handling.

Clipmaps and rings are address mappings only. Complete residency,
reuse-generation, synchronization, cache, and GPU systems belong to their
consumers.

## Demand model

A demand source publishes its current desired spatial coverage. The framework
combines active source snapshots into one deterministic effective demand set.

Neutral demand facts may include:

- chunk identity;
- source identity;
- priority;
- retention requirement;
- explicit pinning.

The framework must not interpret consumer purpose. Terms such as collision,
rendering, navigation, replication, editor preview, or offline generation remain
host-owned.

### Deterministic composition

- any active source may make a chunk desired;
- priority has a documented total ordering;
- equal priorities use stable source and chunk identities as tie breakers;
- retention uses the strongest active requirement;
- replacing or removing one source affects only that source's contribution;
- queued priorities refresh whenever effective demand changes, even if
  membership does not;
- identical snapshots produce identical effective output.

## Streaming model

Streaming reconciles effective spatial demand with one host-defined availability
class. It emits requests; a host backend performs work and reports events.

```text
RunenSpatial Streaming
    emits StreamRequest

host/backend
    performs load or unload

host/backend
    reports StreamBackendEvent

RunenSpatial Streaming
    updates deterministic state and emits diagnostics/events
```

Core does not require a backend trait or runtime.

### Controller scope

One controller instance governs one neutral availability class over chunk IDs.
For example, a host may use one controller for base world-source data. The
controller does not become a registry for collision, visual, navigation, SDF,
or other product classes.

When a host needs independent availability classes, it uses independent
controller instances and keeps their meaning outside RunenSpatial. A stable
cross-class identity or universal product registry is not part of the accepted
architecture and would require a separately accepted ownership decision.

### Orthogonal state

Each chunk stream record separates:

```text
desired
availability
operation
last transition failure
```

Target concepts:

```text
availability
    Absent
    Resident

operation
    Idle
    LoadQueued
    LoadIssued
    Loading
    UnloadQueued
    UnloadIssued
    Unloading

last transition failure
    LoadFailed
    UnloadFailed
```

`Resident` means only that this controller's backend completed its opaque
availability transition. It does not mean a Runenwerk product is current,
certified, active, rendered, replicated, or saved.

Failure facts describe load and unload transition outcomes only. Product health,
content validity, corruption, and post-load invalidation remain host-owned. A
host may explicitly remove or reload availability through public lifecycle
operations, but RunenSpatial must not invent a `ResidentPayloadFailed` product
state.

### Required invariants

- one active transition request per chunk and controller;
- request identity and chunk identity must match;
- unknown, stale, duplicate, malformed, and mismatched events cannot silently
  mutate state;
- load failure leaves availability absent;
- unload failure leaves availability resident;
- queued work survives unchanged demand and drains under nonzero budgets;
- request ID exhaustion is explicit;
- demand reversal during active work is deterministic;
- retries are explicit and policy-free;
- cancellation and timeout are excluded until complete backend contracts exist;
- no record is interpreted as universal availability across host product classes.

## Consumer integration

### Runenwerk

Runenwerk translates players, editors, simulation, networking, and product
requirements into neutral RunenSpatial demand. It owns backend selection, world
products, generation, invalidation, retries, recovery, activation, and
persistence.

Its mixed engine chunk lifecycle must be decomposed rather than copied into
RunenSpatial:

| Dimension | Owner |
| --- | --- |
| spatial demand | RunenSpatial Demand |
| one neutral availability class | RunenSpatial Streaming |
| dirty regions and invalidation | Runenwerk `world_ops` |
| product build and generation | owning Runenwerk product domain |
| gameplay/runtime activation | Runenwerk engine |
| render-cache residency | RunenRender |
| GPU realization | RunenGPU |

### Godot World Lab

Godot World Lab may use an adapter to convert positions/configuration and report
backend events. It owns nodes, scenes, assets, visual realization, experimental
generation, caches, and debug UI.

The current adapter remains experimental until reuse and lifecycle behavior are
audited.

### RunenRender and RunenGPU

RunenRender may consume prepared spatial summaries or adapters, but does not own
authoritative world availability.

RunenGPU has no dependency on RunenSpatial. Generic GPU realization and execution
remain independent of world meaning.

## Extraction and cutover rule

RunenSpatial becomes authoritative only through a clean consumer cutover.

For every migrated component:

1. accept and validate the standalone contract;
2. add the exact dependency in Runenwerk;
3. migrate all real consumers in scope;
4. preserve Runenwerk-specific meaning in Runenwerk-owned adapters;
5. delete the corresponding internal duplicate in the same accepted slice;
6. prove no forwarding crate, copied source, source include, branch dependency,
   or submodule remains.

Historical source is provenance evidence, not continuing authority.

## Architecture stop conditions

A new package, stable serialization format, backend trait, concurrency contract,
compatibility layer, availability-class registry, host-purpose enum, or
cross-repository abstraction requires a separately accepted issue when it
introduces a new ownership or compatibility boundary.
