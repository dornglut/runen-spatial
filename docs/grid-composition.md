# Grid Composition

`godot_grid` remains in `Crystonix/grid`. It handles per-chunk tile and topology
visual mapping. `spatial_streaming` decides which chunks should be resident and
which lifecycle requests/events are active.

## Composition Flow

1. `godot_world_streaming` emits `chunk_load_requested(request_id, x, y, z)`.
2. Godot host code creates or loads an application-owned chunk root.
3. Host code builds or fetches the chunk's logic grid.
4. Host code calls `godot_grid` to form visual tile descriptors.
5. `godot_grid` returns asset keys, masks, rotations, and visual coordinates.
6. Host code maps asset keys such as `corner_0`, `edge_90`, or `full` to
   reusable Blender-exported mesh assets.
7. Host code instances those meshes under the chunk root at
   `chunk_world_origin + tile_offset`.
8. Host code reports provider completion with the same `request_id`.
9. Unload events cause host code to detach or free the chunk root and report
   provider completion for the unload request.

`spatial_streaming` does not know about tile masks or Blender meshes.

`godot_grid` does not know about streaming budgets or lifecycle state.

The Godot host composes both adapters.
