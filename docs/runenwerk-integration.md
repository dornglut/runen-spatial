# Runenwerk Integration

Runenwerk integration is staged through compatibility wrappers after the
standalone core, adapter, and demo are proven.

## Migration Status

1. `domain/spatial` remains the Runenwerk dependency name and re-exports
   `spatial_streaming/crates/spatial`.
2. `domain/spatial_index` remains the Runenwerk dependency name and re-exports
   `spatial_streaming/crates/spatial_index`.
3. `domain/chunking` remains the Runenwerk dependency name and re-exports
   `spatial_streaming/crates/chunking`.
4. The wrapper packages are versioned separately from the extracted packages so
   Cargo can represent both packages in one lockfile during the transition.
5. `godot_chunking_addon` still compiles against the wrappers. Replacing it
   with `godot_world_streaming` remains a later app-integration decision.

## Remaining Cleanup

Remove duplicated Runenwerk source modules only after the wrapper migration is
accepted and downstream consumers no longer need local fallback source history.

## Reference-Only Sources

The `world_streaming` implementation may read Runenwerk engine lifecycle code
for context, but must not mechanically extract from it. These concepts stay in
Runenwerk:

- dirty reasons;
- chunk revisions and build generations;
- world build queues;
- SDF payload formation;
- render cache invalidation;
- ECS resources and systems;
- networking replication state.

## Validation Before Migration

- `cargo test` in `spatial_streaming`.
- Runenwerk checks for current spatial/chunking consumers.
- Adapter checks only after the Godot adapter exists.
- No migration step should widen `spatial_streaming` to absorb `world_ops`,
  `world_sdf`, `procgen`, `product`, engine, editor, or apps.
