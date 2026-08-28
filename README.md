# RunenSpatial

RunenSpatial is Dornglut's host-neutral Rust framework for large-world spatial identity and coordinate mechanics, spatial demand planning, and content-agnostic chunk availability control.

It is intended to be usable independently of Runenwerk and engine integrations. Product generation, simulation, ECS/gameplay activation, rendering, persistence, networking, filesystem/network IO, and async-runtime policy remain outside this repository.

## Current capabilities

The current workspace provides:

- namespaced world, chunk, region, hierarchy, clipmap, and ring identities/mapping primitives;
- checked large-world coordinate, hierarchy, partition, frame, and overflow behavior;
- deterministic spatial hashing primitives;
- a deterministic single-focus demand baseline with hysteresis;
- a budgeted content-agnostic streaming controller with request correlation and backend-event handling;
- an optional experimental Godot adapter and a repository demo.

## Maturity

RunenSpatial is public, unpublished, and pre-release. The current API is not a stable persistence or wire-format promise.

Known architectural work remains around retained foundation contracts, adapter/demo disposition, multi-source demand, streaming lifecycle decomposition, request identity, bounded long-running state, standalone conformance, and the later Runenwerk consumer cutover.

## Validation

```text
cargo validate
```

See [TESTING.md](TESTING.md) for the mechanical validation contract.

## Repository authority

- [Architecture](ARCHITECTURE.md)
- [Roadmap](ROADMAP.md)
- [Documentation architecture](docs/documentation-architecture.md)
- [Spatial model](docs/spatial-model.md)
- [Spatial demand](docs/spatial-demand.md)
- [Streaming lifecycle](docs/streaming-lifecycle.md)
- [Godot integration](docs/godot-integration.md)
- [Transfer provenance](docs/provenance/repository-transfer.md)
- [Security policy](SECURITY.md)
- [Licensing](LICENSING.md)

## Licensing

Current repository code is licensed under [GPL-3.0-only](LICENSE). A separate commercial license may be available from copyright holder(s) with sufficient rights; see [LICENSING.md](LICENSING.md).
