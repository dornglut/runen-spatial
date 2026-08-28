# Godot Integration

`adapters/godot_world_streaming` is RunenSpatial's retained optional, non-default, non-publishable pre-release Godot translation integration. Maintained consumer evidence justifies keeping this artifact in the repository.

It is not framework authority and does not justify Godot dependencies or engine semantics in the host-neutral packages.

## Ownership boundary

RunenSpatial owns the adapter's translation between Godot-facing values and the host-neutral spatial-demand/availability contracts, plus the Godot-visible request/event bridge required to expose those contracts.

The consuming host owns scenes, providers, payload caches, generation, realization, rendering, physics, persistence, gameplay behavior, and other product or engine policy.

## Current role

The adapter converts Godot-facing configuration and positions into the current foundation/demand/streaming APIs, emits framework requests/events through Godot-visible methods and signals, and can rebuild its controller from node configuration.

Its current numeric conversion, request-ID representation, configuration-mutation, and controller-reset behavior remain pre-release integration contracts. They should be hardened only after the corresponding host-neutral demand and lifecycle contracts stabilize, avoiding duplicate semantic churn.

## Maturity

Retention is established; API stability is not. No production-support or stable Godot API promise is made yet.
