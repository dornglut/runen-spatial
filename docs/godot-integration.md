# Godot Integration

`adapters/godot_world_streaming` is an optional, non-default, non-publishable experimental consumer of the RunenSpatial packages.

It exists to exercise integration with a Godot node and signal/property surface. It is not framework authority and does not justify Godot dependencies or engine semantics in core packages.

## Current role

The adapter converts Godot-facing configuration and positions into the current foundation/demand/streaming APIs, emits framework requests/events through Godot-visible methods/signals, and can rebuild its controller from node configuration.

Its current behavior includes adapter-specific conversions and state-reset semantics that still require a dedicated ownership/API audit. In particular, public numeric conversion, request-ID representation, configuration mutation, and controller rebuild behavior must not silently weaken the checked framework contracts.

## Maturity

No production-support, stable Godot API, or permanent repository-ownership promise is made. Retention of this adapter requires maintained-consumer evidence and tests that justify keeping an engine-specific integration in the standalone framework repository. Otherwise it should be removed and integration should live with the consuming product/lab.
