# Testing and Validation

`cargo validate` is the single maintained local and CI validation authority for RunenSpatial.

It verifies:

- repository policy, required files, manifest inventory, package metadata, publication state, and dependency containment;
- the exact read-only shared-workflow caller and immutable workflow revision;
- Markdown links, transfer provenance, authored-file size, symlink and gitlink policy, and current Dornglut authority;
- locked metadata, formatting, core tests, full workspace checks, denied-warning Clippy, denied-warning rustdoc, and Rust 1.93.0 core compatibility;
- diff hygiene and a clean tracked repository state after validation.

## Required command

```text
cargo validate
```

Run focused commands while editing, but do not substitute them for the complete validation authority before review or merge. GitHub Actions invokes the same repository-owned command through the immutable Dornglut shared Rust workflow.

The optional Godot adapter is checked on stable Rust. The declared Rust 1.93.0 core compatibility command excludes the adapter so an optional platform dependency cannot silently raise the core MSRV.

See [the detailed validation contract](docs/tooling/validation.md).
