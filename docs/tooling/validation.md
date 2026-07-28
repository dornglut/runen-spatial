# Validation Contract

## Authority

`cargo validate` is the only maintained complete validation command for RunenSpatial. Local development and GitHub Actions invoke the same repository-owned `xtask` path.

The GitHub caller is intentionally minimal and read-only. It invokes the immutable shared workflow revision recorded in repository policy; it does not duplicate command lists or inherit secrets.

## Validation order

1. Repository policy and required-file inventory.
2. Cargo manifest inventory, metadata, publication, lint, and path-containment policy.
3. Workflow shape and immutable shared revision.
4. Symlink, gitlink, authored-file-size, source-include, provenance, and current-authority checks.
5. Relative Markdown links.
6. Locked Cargo metadata.
7. Formatting.
8. Core and support-package tests, excluding the optional Godot adapter.
9. Full workspace all-target check.
10. Full workspace all-target Clippy with warnings denied.
11. Workspace rustdoc with warnings denied.
12. Rust 1.93.0 tests excluding the optional Godot adapter.
13. `git diff --check` and clean tracked repository state.

## Core and adapter toolchains

The core framework declares Rust 1.93.0 as its initial MSRV. The optional Godot adapter is checked on the repository stable toolchain and is excluded from the MSRV proof. This prevents an optional platform dependency from silently changing the core compatibility contract.

A future adapter-specific MSRV claim requires direct evidence and an accepted issue.

## File and dependency policy

- All path dependencies resolve inside this repository.
- Sibling checkouts, submodules, gitlinks, and copied source authority are forbidden.
- Tracked authored text files remain at or below 128 KiB.
- `Cargo.lock` is retained as a generated dependency lock and is exempt from the authored-file size limit.
- Build output and generated platform artifacts are not source authority.
- Active documents use current Dornglut and RunenSpatial authority. Historical names appear only in explicit provenance or investigation context.

## CI acceptance

A pull request is mergeable only when the exact reviewed head passes the shared validation workflow. After squash merge, the push run for the accepted main revision must pass before the next roadmap child becomes active.

Focused commands are useful during editing but are not substitute merge evidence.
