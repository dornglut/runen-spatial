# RunenSpatial Roadmap

## Authority

This document records durable sequence and completion gates.

Live work, priority, assignees, branches, pull requests, commit heads, and validation runs belong in GitHub issues, pull requests, and CI.

- live work: parent issue #1 and its current child

No GitHub milestone is required. Parent and child issues form the implementation roadmap.

## Operating principles

1. Preserve one owner per concept.
2. Keep transfer/governance, semantic correction, consumer migration, and source deletion in separate reviewable slices.
3. Create only the next accepted child issue.
4. Do not preserve compatibility façades as the final state.
5. Do not call primitive address math a complete runtime system.
6. Do not call a target capability supported before implementation and proof exist.
7. RunenSpatial remains independent of Runenwerk, SDF, ECS, rendering, GPU, Godot project state, IO, and application policy.
8. Exact-head CI is merge evidence; durable docs are not live execution ledgers.

## RS0 — Decide the extraction boundary

Status: complete. The accepted architecture and package-boundary decisions are
the authority for later slices.

Bind:

- framework ownership and exclusions;
- source authority and provenance;
- package topology and naming;
- large-world identities, coordinate widths, frames, rebasing, overflow, and serialization status;
- hierarchy convention;
- spatial-demand composition;
- streaming availability, operation, failure, and backend-event contracts;
- spatial-index audit gate;
- Godot adapter disposition gate;
- governance, validation, visibility, and publication policy;
- Runenwerk cutover prerequisites.

Exit gate:

- architecture and package decisions are documented;
- current capabilities and target capabilities are separated;
- known defects have explicit proof requirements;
- the next child is behavior-preserving governance normalization only;
- the planning pull request is independently reviewed and merged.

## RS1 — Normalize governance and validation

Status: complete. The governance and validation baseline is accepted.

Behavior-preserving repository work:

- add `ARCHITECTURE.md`, `AGENTS.md`, and `TESTING.md` root authorities;
- add repository-owned `cargo validate` through `xtask`;
- invoke the immutable Dornglut shared Rust-validation workflow;
- set edition, MSRV, repository metadata, and `publish = false` explicitly;
- establish dependency, provenance, file-size, generated-file, Markdown-link, and clean-tree checks;
- remove process-only prompts and superseded duplicate documents;
- preserve all runtime behavior and public API names.

Exit gate:

```text
cargo validate
git diff --check
```

The exact reviewed head passes shared CI.

## RS2 — Normalize package identity

Apply the accepted Cargo identities without changing runtime behavior:

```text
runen-spatial
runen-spatial-index, retained provisionally pending RS6
runen-spatial-demand
runen-spatial-streaming
```

Delete the broad prelude and migrate consumers directly to owning packages.

Exit gate:

- no broad compatibility façade remains;
- examples and adapters use explicit owning packages;
- dependency direction is guarded;
- no behavior change is mixed into the naming/cutover diff.

## RS3 — Correct spatial addressing and hierarchy

Implement the accepted foundational contract:

- signed 64-bit stable chunk and region addresses;
- checked partition, neighborhood, scale, and parent/child arithmetic;
- finite global `f64` and frame-local `f32` positions with explicit frame context;
- documented host-owned rebasing;
- level 0 finest, higher levels coarser;
- negative-coordinate floor division;
- containment and complete-child-coverage proofs;
- explicit serialization/versioning status;
- clipmap and ring claims limited to mapping primitives.

Exit gate:

- property and negative tests cover containment, round trips, negative coordinates, extreme values, overflow, frame conversion, clipmap windows, and ring mappings;
- no silent wrapping or saturation changes spatial identity.

RS3 is not complete until its implementation is independently reviewed,
squash-merged, and validated on accepted main.

## RS4 — Establish multi-source spatial demand

Replace the inherited single-focus `chunking` surface with an accepted neutral demand model:

- source IDs and complete source snapshots;
- explicit source replacement and removal;
- deterministic effective demand merging;
- stable total priority ordering and priority refresh;
- retention/hysteresis;
- explicit pinning where justified;
- bounded pressure behavior;
- deterministic replay.

RunenSpatial does not interpret host purpose such as collision, rendering, navigation, replication, editor preview, or offline generation.

Exit gate:

- multiple sources combine deterministically;
- removing one source preserves other sources' demand;
- queued priorities refresh when effective demand changes;
- pins persist only until explicit release;
- identical snapshots reproduce identical output.

## RS5 — Harden streaming lifecycle

Implement orthogonal state and backend semantics:

- desired state;
- observed availability;
- active operation;
- last failure;
- checked request-ID allocation;
- structured unknown, stale, duplicate, malformed, and mismatched event handling;
- load-failure versus unload-failure behavior;
- persistent budgeted queues and eventual drain;
- deterministic demand reversal;
- pressure diagnostics.

Keep async runtimes, IO, task handles, caches, backoff, timeout, cancellation, payload types, and application recovery outside core.

Exit gate:

- transition-table, property, replay, pressure, and negative tests prove every accepted invariant;
- unload failure preserves resident availability;
- no accepted queued work silently disappears.

## RS6 — Audit spatial indexing

Audit before accepting the final public package boundary:

- insert/update/remove and duplicate-key behavior;
- deterministic or explicitly unordered query results;
- negative/extreme coordinates;
- adversarial distributions;
- memory growth and cleanup;
- complexity and benchmark claims;
- concurrency and serialization claims;
- independent consumer value.

Accepted outcomes:

- retain `runen-spatial-index`;
- narrow it;
- merge it into `runen-spatial`;
- defer it from the public release surface.

## RS7 — Decide the Godot adapter owner

Audit:

- API neutrality;
- node/scene cleanup;
- editor reload and shutdown;
- rapid-movement request correlation;
- Godot-version support;
- error and duplicate-signal behavior;
- World Lab policy leakage;
- independent reuse.

Until accepted otherwise, the adapter remains optional, non-default, experimental, and non-publishable.

Do not create a separate public Godot package without multiple real consumers.

## RS8 — Prove standalone conformance

Required:

- no Runenwerk dependency;
- no Godot dependency in core packages;
- no SDF, ECS, rendering, GPU, IO, or application-policy semantics;
- public-API examples and downstream conformance;
- dependency and source guards;
- provenance and license validation;
- property, negative, deterministic replay, rustdoc, and MSRV validation;
- explicit compatibility, visibility, and publication policy.

Only after the required standalone contract is accepted may a Runenwerk cutover parent be created.

## Runenwerk cutover sequence

Runenwerk cutover is owned by a separate parent issue in `dornglut/runenwerk`.

Dependency order:

1. identities and coordinate mathematics;
2. spatial indexes, if retained;
3. spatial demand;
4. streaming lifecycle;
5. decomposition of the mixed engine chunk lifecycle.

Each child must:

- add one exact accepted dependency;
- migrate every real consumer in scope;
- keep world/product/application meaning in Runenwerk;
- delete the corresponding internal duplicate in the same accepted slice;
- prove no forwarding crate, copied source, source include, branch dependency, or submodule remains.

## Work ownership

### GPT Web

- investigation and architecture;
- issue decomposition and parent reconciliation;
- terminology and documentation authority;
- pull-request and CI review;
- cross-repository sequencing and closure verification.

### Codex

- local source and history audits;
- bounded implementation;
- tests, benchmarks, validation, and migrations;
- branch, commit, push, and draft-pull-request publication;
- exact base/head and clean-worktree evidence.

### Manual administration

Only repository settings or transfer operations not exposed through connected tooling.

## Stop conditions

Stop and create a separate accepted issue before introducing:

- a new public package;
- a stable persisted or wire format;
- a mandatory backend trait or async runtime;
- concurrency guarantees;
- host-purpose enums;
- a compatibility façade;
- a new cross-repository ownership boundary;
- public visibility or crate publication.
