# Godot Integration

This documents the optional `adapters/godot_world_streaming` crate, which depends directly on `runen-spatial`, `runen-spatial-demand`, and `runen-spatial-streaming`.

The Godot adapter is adapter-only. It translates Godot-facing configuration and
positions, ticks the core controller, and emits Godot-friendly lifecycle signals.

## Signals

Use precise lifecycle names:

- `chunk_load_requested(request_id, x, y, z)`
- `chunk_provider_started(request_id, x, y, z)`
- `chunk_provider_completed(request_id, x, y, z)`
- `chunk_provider_failed(request_id, x, y, z)`
- `chunk_resident`
- `chunk_unload_requested(request_id, x, y, z)`
- `chunk_unloaded`

Do not emit `chunk_ready` from the core adapter. "Ready" can mean content loaded,
visualized, physics-ready, gameplay-ready, or save-ready. Those meanings belong
to the host application or domain-specific systems.

## In Scope

- Convert Godot `Vector3` values to finite global `f64` meters and construct a
  checked `WorldPosition` in the controller namespace.
- Build `GridPartitionConfig` and a complete demand transaction from
  Godot-facing fields.
- Tick `WorldStreamingController`.
- Translate core events to Godot signals.
- Accept provider-started, provider-completed, and provider-failed callbacks
  from host code.
- Treat provider work as non-cancellable. Desired-state reversals are resolved
  by provider completion or failure events.
- Report checked position or streaming failures through `streaming_error`.

Coordinate signal and callback components are signed 64-bit values, matching
the stable core coordinate contract.

## Out of Scope

- Godot node/scene ownership for chunks.
- MeshInstance creation.
- Asset catalog lookup.
- Blender/glTF loading.
- Save formats.
- SDF payload loading.
- ECS spawning.
- Renderer resources.
