# Architecture

RunenSpatial is a standalone, host-neutral Rust framework for spatial identity, addressing, indexing, deterministic demand, and content-agnostic chunk availability control.

## Ownership boundary

RunenSpatial owns reusable spatial mechanics. It does not own world products, SDFs, procgen, simulation, ECS activation, rendering, GPU execution, IO, persistence, application recovery policy, or Godot scene and asset ownership.

One streaming controller governs one host-defined neutral availability class. It is not a universal product or residency manager.

## Dependency direction

The current package direction is `runen-spatial-index -> runen-spatial`, `runen-spatial-demand -> runen-spatial`, and `runen-spatial-streaming -> runen-spatial-demand -> runen-spatial`. Core packages must depend only toward lower-level RunenSpatial packages and repository-contained dependencies. Optional adapters may depend on core packages; core packages must not depend on adapters, Runenwerk, Godot, rendering, or GPU systems.

No compatibility façade or copied source authority is an accepted final state.

## Detailed authority

- [Canonical architecture](docs/architecture.md)
- [Package boundaries](docs/package-boundaries.md)
- [Transfer provenance](docs/provenance/repository-transfer.md)
- [Validation contract](docs/tooling/validation.md)
- [Roadmap](docs/roadmap.md)

Historical repository names are provenance evidence only. Current framework authority belongs to `dornglut/runen-spatial`.
