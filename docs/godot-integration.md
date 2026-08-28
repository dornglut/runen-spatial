# Godot Integration

`adapters/godot_world_streaming` is RunenSpatial's retained optional, non-default, non-publishable pre-release Godot translation integration. Maintained consumer evidence justifies keeping this artifact in the repository.

It is not framework authority and does not justify Godot dependencies or engine semantics in the host-neutral packages.

## Ownership boundary

RunenSpatial owns the adapter's translation between Godot-facing values and the host-neutral spatial-demand/availability contracts, plus the Godot-visible request/event bridge required to expose those contracts.

The consuming host owns scenes, providers, payload caches, generation, realization, rendering, physics, persistence, gameplay behavior, CPU/frame scheduling, and other product or engine policy.

## Current role

The adapter converts Godot-facing configuration and positions into the current foundation/demand/streaming APIs through one stable internal demand source, emits framework requests/events through Godot-visible methods and signals, and keeps structural configuration distinct from live streaming policy.

Request IDs are translated exactly across the Godot boundary. Inbound IDs must be positive signed 64-bit values. A host-neutral request ID that cannot be represented exactly as Godot `i64` is reported through `streaming_error`; it is never saturated or aliased to another request.

Godot-facing world IDs, partition values, demand radii, and request budgets are checked rather than silently repaired. Invalid mutations leave the previously accepted configuration and controller state unchanged.

### Structural configuration

World ID, chunk edge, and region dimensions define namespace or partition structure. Changing one requires rebuilding the host-neutral controller.

A structural change is accepted only when no runtime records exist. Resident chunks, active provider requests, and blocking failures therefore prevent a rebuild and are preserved. Planner-only demand with no runtime record does not block a rebuild; it may be cleared and republished by the next host focus update.

`reset_streaming_state` follows the same rule: it may rebuild only when runtime records are absent. It does not cancel provider work, unload host payloads, clear caches, or coordinate scene/scheduler state.

### Live policy

Demand radii and per-tick request budgets are live policy rather than structural configuration.

- checked radius changes update the adapter policy in place and take effect on the next focus-source replacement;
- checked request-budget changes update the controller in place;
- zero request budgets are valid explicit policy;
- negative budgets are rejected rather than converted to zero.

These live changes do not rebuild the controller and therefore do not discard request correlation, residency, or failure state.

The adapter supplies fixed pre-release streaming-capacity policy to the host-neutral controller:

- at most 1024 tracked runtime records;
- at most 4 in-flight load requests;
- at most 4 in-flight unload requests.

These are adapter policy, not core defaults or a stable Godot configuration API. Provider queues, payload/cache residency, CPU scheduling, realization, and degradation policy remain host-owned.

No planar/volume mode selector is part of the retained adapter contract. The host-neutral demand footprint is the accepted axis-aligned chunk-volume contract defined by RunenSpatial demand semantics.

## Maturity

Retention and translation/mutation semantics are established; API stability is not. No production-support or stable Godot API promise is made yet.
