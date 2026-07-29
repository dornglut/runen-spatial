# Runenwerk Integration

Runenwerk has not completed a dependency cutover to RunenSpatial. The repositories currently contain duplicate spatial, demand, and streaming authority, and Runenwerk still owns a mixed engine chunk lifecycle. Any later cutover consumes the explicit `runen-spatial` package family directly.

This document defines the future integration boundary only. Live execution is owned by repository issues.

## Preconditions

Do not create the Runenwerk cutover parent until the standalone component being consumed has:

- an accepted public contract;
- repository-owned validation and exact-head CI;
- documented ownership and exclusions;
- conformance evidence;
- no hidden dependency on Runenwerk or a sibling checkout.

## Cutover order

1. Spatial identities and coordinate mathematics.
2. Spatial indexes, if the index package survives its audit.
3. Spatial demand.
4. Streaming lifecycle.
5. Decomposition of Runenwerk's mixed engine lifecycle.

Each cutover slice must add the accepted dependency, migrate every real consumer in scope, preserve Runenwerk-owned meaning in Runenwerk, and delete the corresponding duplicate source in the same accepted change.

Forwarding crates, copied modules, source includes, branch dependencies, external paths, and submodules are not accepted as the final state.

## Ownership retained by Runenwerk

Runenwerk continues to own:

- world edits and dirty regions;
- product demand and build generations;
- SDF and other world-product payloads;
- procgen, simulation, networking, and persistence;
- collision certification and visual fallback policy;
- ECS and gameplay activation;
- retries, degradation, and application recovery;
- renderer preparation and host backend selection.

RunenSpatial owns only neutral spatial mechanics and one host-defined availability class per streaming controller.

## Validation

Every cutover requires standalone RunenSpatial validation, exact dependency identity, full Runenwerk validation, duplicate-authority guards, and accepted-main push evidence in both repositories where applicable.
