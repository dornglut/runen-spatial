# Host Composition

RunenSpatial owns neutral spatial demand and availability transitions. A host owns world content, generation, assets, scene realization, and product readiness.

## Composition flow

1. A host publishes neutral spatial demand.
2. RunenSpatial emits a load or unload request with request and chunk identity.
3. The host starts the corresponding backend operation.
4. The host reports the started event.
5. The host loads, generates, or releases its opaque availability payload.
6. The host reports completion or failure with the same request and chunk identity.
7. RunenSpatial updates deterministic transition state and emits diagnostics/events.
8. Product generation, visual realization, physics, gameplay activation, persistence, and retry policy remain separate host concerns.

## Grid and Godot consumers

A Godot World Lab or grid-oriented consumer may combine RunenSpatial with its own topology, generation, mesh, material, and node systems. Those consumer systems do not become RunenSpatial authority.

The optional Godot adapter translates values and events only. It does not own world nodes, tile descriptors, assets, generation rules, caches, or readiness policy.

## Boundary rule

RunenSpatial decides neutral desired coverage and availability transitions. The host decides what an available chunk means and how it is realized.
