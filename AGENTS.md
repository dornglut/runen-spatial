# RunenSpatial Agent Guide

Start with `README.md`, `ARCHITECTURE.md`, `TESTING.md`, `docs/architecture.md`, `docs/package-boundaries.md`, and `docs/roadmap.md`.

## Repository mission

RunenSpatial owns host-neutral spatial mechanics. It must not acquire world-product, SDF, procgen, ECS, render, GPU, IO, persistence, application-policy, or Godot-scene responsibilities.

## Required workflow

1. Read the current architecture, package-boundary, validation, provenance, and roadmap authorities.
2. Work only under an accepted repository issue with explicit scope and exclusions.
3. Keep one owner per concept and preserve dependency direction.
4. Do not introduce compatibility façades, forwarding crates, copied source, external path dependencies, or duplicate authority.
5. Keep package additions, splits, and publication behind a proven ownership and consumer boundary.
6. Run `cargo validate` before declaring a change ready.
7. Use exact-head GitHub Actions as merge evidence.
8. Update durable docs only when architecture, ownership, sequence, or provenance changes; do not store live branch, PR, SHA, or workflow inventories in them.

## Cold-start pickup

1. Read the repository authorities listed above.
2. Open parent issue #1.
3. Follow its Current child.
4. Read that child’s latest status comment.
5. Inspect its associated pull request and exact-head CI.
6. Fetch and verify origin/main before branching.
7. Never infer live execution state from durable roadmap prose.

## Public API rules

- Prefer explicit domain types and validated construction.
- Keep stable spatial identities separate from frame-local coordinates.
- Keep demand, availability, operations, failures, products, render caches, and GPU realization as separate facts.
- Keep host purpose opaque; do not add collision, rendering, navigation, replication, or editor policy to core contracts.
- Avoid macro or derive magic that hides ownership or invariants.

## Validation and files

- `cargo validate` is the maintained local and CI authority.
- Tracked authored text files must remain at or below 128 KiB.
- Generated artifacts and build products are not source authority.
- The repository must remain independently buildable without sibling checkouts.
