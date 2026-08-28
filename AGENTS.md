# RunenSpatial Agent Guide

## Authority

Read [documentation architecture](docs/documentation-architecture.md) before editing durable documentation or changing ownership boundaries.

Follow the organization-wide authority, work, repository, validation, GitHub, and licensing rules in `dornglut/engineering`. Live work authority comes from the current accepted GitHub issue and repository state, not copied status prose in this repository.

For a change, inspect the canonical owner for the concern plus the affected source and tests before editing.

## Repository rules

- Keep RunenSpatial host-neutral. Do not introduce Runenwerk, Godot, ECS, renderer, GPU, IO, async-runtime, product, gameplay, persistence, or network policy into the core framework packages.
- Prefer valid-by-construction or explicitly checked public contracts. Do not silently clamp, saturate, wrap, or repair invalid spatial identity/configuration state unless that behavior is itself the documented contract.
- Do not add compatibility aliases, forwarding modules/crates, duplicate authorities, source mirrors, external path dependencies, submodules, or branch dependencies as migration mechanisms.
- Do not retain a package boundary merely because it already exists. A durable package needs independent ownership, dependency/versioning value, and a proven consumer or contract.
- Do not add speculative abstractions for possible future geometry, products, engines, or consumers.
- Keep durable docs free of current branch, pull request, exact-head, workflow-run, current-child, or temporary blocker state.
- Historical repository names and exact revisions belong only in explicit provenance/history material.

## Validation

The repository-owned validation entry point is:

```text
cargo validate
```

Run it from a checked-out candidate before publication when the change depends on compilation, tests, linting, or generated Cargo state. Pull requests must also pass validation at the exact reviewed head through the repository workflow.

Do not replace repository validation with ad-hoc command lists in CI or durable docs.
