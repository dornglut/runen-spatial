# Godot Integration

`adapters/godot_world_streaming` is RunenSpatial's retained optional, non-default, non-publishable pre-release Godot translation integration. Maintained consumer evidence justifies keeping this artifact in the repository.

It is not framework authority and does not justify Godot dependencies or engine semantics in the host-neutral packages.

## Ownership boundary

RunenSpatial owns the adapter's translation between Godot-facing values and the host-neutral spatial-demand/availability contracts, plus the Godot-visible request/event bridge required to expose those contracts.

The consuming host owns scenes, providers, payload caches, generation, realization, rendering, physics, persistence, gameplay behavior, CPU/frame scheduling, and other product or engine policy.

## Current role

The adapter converts Godot-facing configuration and positions into the current foundation/demand/streaming APIs through one stable internal demand source, emits framework requests/events through Godot-visible methods and signals, and can rebuild its controller from node configuration.

Request IDs are translated exactly across the Godot boundary. Inbound IDs must be positive signed 64-bit values. A host-neutral request ID that cannot be represented exactly as Godot `i64` is reported through `streaming_error`; it is never saturated or aliased to another request.

The adapter currently supplies fixed pre-release streaming-capacity policy to the host-neutral controller:

- at most 1024 tracked runtime records;
- at most 4 in-flight load requests;
- at most 4 in-flight unload requests.

These are adapter policy, not core defaults or a stable Godot configuration API. Per-tick request budgets remain separately configurable through the existing adapter surface. Provider queues, payload/cache residency, CPU scheduling, realization, and degradation policy remain host-owned.

Other numeric conversion, configuration-mutation, and controller-reset behavior remains pre-release integration contract work. Harden those surfaces independently against the accepted host-neutral contracts rather than duplicating core semantics in the adapter.

## Maturity

Retention is established; API stability is not. No production-support or stable Godot API promise is made yet.
