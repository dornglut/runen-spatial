# RunenSpatial Extraction-Boundary Investigation

## Purpose

This investigation records the transferred repository state, separates
implemented capabilities from target architecture, and binds the decisions
required before semantic implementation or Runenwerk cutover.

Issue authority:

- parent: #1
- decision phase: #2
- transferred baseline: `2a87094cb4ca4ed48238b416f4d4121cb5e074a1`

## Repository state

The repository was transferred from `aschenrot/spatial_streaming` to
`dornglut/runen-spatial` with commit history preserved. It remains private while
governance, naming, correctness, conformance, and release policy are normalized.

The transferred workspace contains:

| Current package | Current role | Investigation status |
| --- | --- | --- |
| `spatial` | identities, positions, frames, grids, hierarchy, clipmaps, rings, bounds, neutral spatial hashes | reusable foundation; hierarchy and large-world contracts need correction |
| `spatial_index` | generic mutable and hash-based spatial lookup | API exists; production behavior and independent package value need audit |
| `chunking` | single-focus desired/retained chunk-set calculation | useful deterministic baseline; terminology and multi-source demand are incomplete |
| `world_streaming` | budgeted load/unload lifecycle and backend event correlation | strongest extracted component; availability, operation, and failure modeling remain incomplete |
| `world_core_prelude` | broad re-export façade | accepted for removal; obscures ownership and preserves stale extraction naming |
| `godot_world_streaming` | optional Godot adapter | remains experimental and non-publishable pending ownership audit |
| `chunk_streaming_demo` | example consumer | retain as evidence, but remove external project authority assumptions |

No post-transfer issue or pull-request history existed before #1 and #2. The
transferred source head remained unchanged.

### Provenance limitation

Connector inspection verified repository identity, visible commit continuity,
current source, and the preserved transferred head. It did not perform a local
scan of every historical Git object for secrets, unrelated binary material, or
rewritten ancestry.

Before public visibility or publication, repository-owned validation must add:

- full-history secret and large-object review appropriate to Dornglut policy;
- license and contributor provenance checks;
- dependency and source-origin checks;
- explicit recording of any accepted historical exceptions.

The transfer is therefore suitable for private normalization, but is not by
itself public-release provenance evidence.

## What is currently implemented

The repository currently provides:

- opaque world/space identities;
- chunk and region coordinates and identities;
- world and frame-local positions;
- camera-relative frame construction;
- grid partitioning and negative-coordinate floor division;
- hierarchy, clipmap-window, ring-slot, bounds, and spatial-hash primitives;
- mutable and hash-based spatial-index APIs;
- one-focus planar and axis-aligned three-dimensional chunk demand;
- load/unload hysteresis and deterministic entered/exited diffs;
- persistent queued load/unload records;
- per-update request budgets;
- request identity and pending-request tracking;
- backend started/completed/failed events;
- mismatched and stale request rejection;
- deterministic load/unload reversal behavior;
- explicit failed-chunk retry;
- inherited resident-failure reporting;
- an optional Godot adapter and demo.

These capabilities establish a deterministic baseline. They do not yet
constitute a production-complete large-world, multi-source, product-aware
streaming framework.

The inherited resident-failure API is not accepted as target architecture. It
mixes post-load payload health with load/unload transition control and must be
removed or translated into explicit host-owned invalidation during the lifecycle
correction.

## Stale or incorrect repository claims

The transferred documentation must not be treated as current authority where it
claims:

- ownership by `Crystonix/*` repositories;
- a completed Runenwerk compatibility-wrapper cutover;
- a Godot World Lab milestone sequence as the RunenSpatial repository roadmap;
- `world_core_prelude` as the ordinary public entry point;
- current package topology as permanently accepted;
- hierarchy or infinite-world support as complete.

Runenwerk still contains duplicate spatial, demand, and streaming source and
still owns a mixed engine chunk lifecycle. RunenSpatial is therefore a candidate
standalone authority, not yet the accepted Runenwerk runtime dependency.

## Source-authority decision

The transferred repository is the preferred reusable source authority because
its streaming lifecycle is newer and more coherent than the older
Runenwerk-local copy.

The final cutover must nevertheless occur component by component and prove that
no copied source or forwarding façade remains.

Historical source identity is provenance evidence only. Once the standalone
architecture is accepted, current framework authority belongs to
`dornglut/runen-spatial`.

## Ownership decision

RunenSpatial owns reusable spatial mechanics:

- spatial namespace identity;
- durable chunk and region addressing;
- global/local frame mathematics;
- grid and hierarchy mathematics;
- clipmap and ring address mappings;
- neutral spatial hashing;
- generic spatial indexes;
- deterministic spatial-demand composition;
- content-agnostic availability lifecycle for one host-defined availability
  class per controller instance;
- request correlation, budgets, transition outcomes, and diagnostics.

RunenSpatial does not own:

- signed-field mathematics or field products;
- world edits, invalidation, product formation, procgen, simulation, networking,
  or save formats;
- ECS activation, gameplay readiness, or application recovery policy;
- render providers, render-cache residency, image formation, shaders, or GPU
  resources;
- filesystem/network IO, async runtime ownership, task handles, or payload
  caches;
- Godot scenes, nodes, meshes, materials, or assets;
- a universal product or availability-class registry.

Consumers translate their own meaning into neutral spatial demand and backend
work. RunenSpatial must not acquire consumer-purpose enums such as collision,
rendering, navigation, or replication.

## Package-topology decision

The transfer phase preserves the current source topology. The target package
names are provisionally:

```text
runen-spatial
runen-spatial-index
runen-spatial-demand
runen-spatial-streaming
```

The dependency direction is:

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

This topology is accepted as the implementation-planning baseline, not yet as an
irreversible release topology.

Before publication:

- `runen-spatial-index` must prove independent consumer value and production
  behavior;
- package versioning and release cadence must be documented;
- package boundaries must not exist only to preserve the transferred directory
  structure.

`world_core_prelude` has no durable ownership boundary and is accepted for
deletion. Consumers must import the narrow owning package directly. No permanent
compatibility re-export package will replace it.

## Identity and large-world decision

### Namespace identity

`WorldId` remains an ergonomic opaque spatial-namespace identifier. It does not
imply ownership of a Runenwerk world model. Documentation must define it as one
independent coordinate namespace.

### Stable addresses

The target chunk and region coordinate components are signed 64-bit integers.
Stable identities combine the namespace identifier with the address.

Reasoning:

- stable addresses must not depend on floating-point origin placement;
- negative coordinates are first-class;
- 64-bit coordinates provide a practical large-world range without claiming
  mathematical infinity;
- overflow remains possible and must be checked rather than wrapped.

The current `i32` coordinate implementation remains unchanged until the
dedicated spatial-contract implementation issue.

### Position tiers

- stable world addressing uses integer chunk/region coordinates;
- global metric positions use finite `f64` values and explicit namespace/frame
  context;
- simulation and rendering may use frame-local finite `f32` positions;
- conversion from global to local requires an explicit frame;
- local positions are not persistence, replay, wire, or cross-frame identity.

### Rebasing

RunenSpatial owns frame and conversion mathematics. The host owns when and why
to rebase, which entities are moved, and how simulation, networking,
persistence, or rendering observe the change.

### Overflow and serialization

All scale, parent/child, neighborhood, and partition operations that can overflow
must return checked outcomes. No wrapping or saturating coordinate arithmetic may
silently alter identity.

Serialization support records values, not permanent schema compatibility. A
stable persisted/wire format requires a separately accepted versioning contract.

## Grid and hierarchy decision

The hierarchy convention is:

```text
level 0 = finest
higher level = coarser
```

For scale factor `s`:

```text
parent level = level + 1
parent coordinate = floor_div(child coordinate, s)
child level = level - 1
first child coordinate = parent coordinate * s
```

Required constraints:

- scale factor is at least 2 for a real hierarchy;
- level count is nonzero;
- parent/child arithmetic is checked;
- floor division is used for negative coordinates;
- tests prove that every child lies inside its parent and that child ranges
  cover the parent exactly.

Clipmap and ring support must be described as address and mapping primitives.
The repository does not yet own complete clipmap streaming, ring residency,
generation reuse, GPU synchronization, or cache management.

## Spatial-demand decision

The current single-focus planner becomes the baseline for a multi-source demand
model.

A source publishes a complete current snapshot of neutral chunk demand. Source
replacement or removal is explicit; omitted source state must not remain
indefinitely by accident.

Each demand entry contains only neutral mechanics:

- chunk identity;
- source identity;
- deterministic priority;
- retention requirement;
- pinned/unpinned state where justified.

Effective demand is merged deterministically:

- a chunk is desired when at least one active source demands it;
- the best priority is selected by a documented total ordering;
- retention uses the strongest active requirement;
- deterministic source identity breaks otherwise equal ties;
- priorities are refreshed for all queued desired chunks, not only newly
  entered chunks;
- source removal releases only demand owned by that source;
- identical source snapshots produce identical effective demand.

The framework does not interpret why a host wants a chunk. Product and
application purpose remain outside the core contract.

## Streaming-lifecycle decision

### Controller scope

One controller instance manages one host-defined, content-agnostic availability
class over chunk IDs. For example, a host may use one controller for base source
data.

Independent classes use independent controllers and preserve their meaning in
the host. RunenSpatial does not define a stable product-class registry or one
universal availability fact for all products attached to a chunk.

### Orthogonal state

The durable state model separates four facts:

1. desired state;
2. observed availability for this controller;
3. active transition operation;
4. last load/unload transition failure.

Target conceptual shape:

```text
desired:
    false | true

availability:
    Absent | Resident

operation:
    Idle
    LoadQueued
    LoadIssued
    Loading
    UnloadQueued
    UnloadIssued
    Unloading

last transition failure:
    None
    LoadFailed
    UnloadFailed
```

`Resident` means only that this controller's stream backend completed the
availability transition. It does not mean a Runenwerk product is current,
collision-certified, ECS-active, visually ready, or saved.

Post-load content corruption, product invalidation, or payload health is not a
stream transition failure. The host may explicitly remove or reload availability
through public lifecycle operations, but the core does not retain a
`ResidentPayloadFailed` state.

Backend terminology becomes:

- `StreamRequest` for controller output;
- `StreamBackendEvent` for started/completed/failed input;
- no mandatory backend trait, async runtime, task type, or IO mechanism in core.

Required behavior:

- load failure leaves availability absent;
- unload failure preserves resident availability;
- request/chunk mismatches cannot mutate state;
- duplicate, stale, malformed, and unknown events have structured outcomes;
- request IDs use checked allocation and explicit exhaustion;
- queued work persists and eventually drains when budgets remain nonzero;
- demand reversal remains deterministic;
- retry is explicit and policy-free;
- cancellation and timeout remain out of scope until a complete backend
  ownership contract is accepted;
- no controller record is interpreted as universal availability across host
  product classes.

## Spatial-index decision gate

`spatial_index` is not yet accepted as production-complete merely because an API
exists.

Before final package acceptance, audit and prove:

- insertion, update, removal, duplicate-key, and empty-index invariants;
- deterministic query-result ordering or explicit unordered semantics;
- negative and extreme coordinates;
- adversarial bucket distributions;
- memory growth and cleanup;
- stated query complexity;
- concurrency claims, if any;
- serialization claims, if any;
- representative benchmarks;
- at least one independent consumer.

The audit may retain, narrow, merge, or defer the package.

## Godot adapter decision gate

The adapter remains:

```text
optional
non-default
experimental
publish = false
```

Permanent ownership requires evidence about:

- API neutrality across more than one project;
- node and scene cleanup;
- editor reload and shutdown;
- request correlation during rapid movement;
- supported Godot version;
- error propagation and duplicate-signal behavior;
- whether World Lab-specific policy has leaked into the adapter.

Do not create `runen-spatial-godot` until multiple real consumers justify it.

## Governance and release decision

The repository will adopt the Dornglut Rust-framework pattern:

- `README.md` for the human entry point;
- `ARCHITECTURE.md` for root ownership and dependency direction;
- `AGENTS.md` for agent workflow and prohibitions;
- `TESTING.md` for one validation authority;
- `docs/roadmap.md` for durable sequence only;
- parent/child issues for accepted live work;
- pull requests and exact-head CI for implementation evidence.

The core target is Rust edition 2024 with Rust `1.93.0` as the initial framework
MSRV, subject to validation against all retained dependencies. Optional adapters
may require a separately documented newer toolchain but must not silently raise
the core MSRV.

All packages remain `publish = false` until public package names, package
boundaries, documentation, provenance, conformance, and compatibility policy are
accepted.

Tracked authored text files should remain at or below 128 KiB. Generated or lock
files require explicit repository-policy treatment rather than accidental
exceptions. Build products, packaged addons, and large generated artifacts are
not source authority.

The target validation authority is `cargo validate`, implemented through a
repository-owned `xtask` and invoked by the immutable Dornglut shared
Rust-validation workflow. Until that exists, focused commands do not constitute
final merge evidence.

## Runenwerk cutover prerequisites

Do not create the Runenwerk cutover parent until the required standalone
component is accepted and validated.

Cutover order:

1. identities and coordinate mathematics;
2. spatial indexes, if retained;
3. spatial demand;
4. streaming lifecycle;
5. decomposition of the mixed engine chunk lifecycle.

Each cutover child must:

- add one exact accepted dependency;
- migrate every real consumer in scope;
- preserve Runenwerk-owned world/product/application meaning;
- delete the corresponding internal duplicate in the same accepted slice;
- prove no forwarding crate, source include, branch dependency, submodule, or
  copied implementation remains.

## Existing-document disposition

| Path | Decision |
| --- | --- |
| `README.md` | revise during this planning change to identify RunenSpatial and correct current status |
| `docs/roadmap.md` | replace with the durable RunenSpatial sequence; remove Godot-project milestone execution state |
| `docs/crate-boundaries.md` | supersede with `docs/package-boundaries.md`; delete in governance normalization after links are migrated |
| `docs/streaming-lifecycle.md` | retain as current implementation reference; revise with accepted lifecycle model during lifecycle implementation |
| `docs/full-roadmap-goal-prompt.md` | delete during governance normalization; process prompt is not durable architecture |
| `world_core_prelude` | delete during the accepted package/API cleanup slice |
| `godot_world_streaming` | retain temporarily as optional experimental adapter |
| `chunk_streaming_demo` | retain and later convert into standalone public-API conformance evidence |

## Next authorized implementation

After this planning change is independently reviewed and merged, create one child
issue limited to behavior-preserving governance normalization:

- add root governance and testing authorities;
- add `cargo validate` and shared CI;
- set explicit publication and repository metadata;
- complete provenance and history-policy checks;
- remove stale process-only documents and migrated duplicate docs;
- preserve all runtime behavior and public API names.

Semantic changes, package renames, Runenwerk migration, and duplicate-source
deletion remain unauthorized until their own accepted issues.
