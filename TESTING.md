# RunenSpatial Validation

The canonical repository validation command is:

```text
cargo validate
```

It is implemented by the private `xtask` package and is the single maintained local/CI entry point.

## What validation covers

The command checks:

1. repository authority, package/manifest inventory, licensing metadata, contained path dependencies, file-size/symlink policy, retired surfaces, transfer provenance, and exact shared-workflow shape;
2. relative Markdown links;
3. locked Cargo metadata;
4. formatting;
5. workspace tests, including the architectural runtime dependency-topology guard and public-API package integration/cross-layer conformance coverage, while excluding the optional Godot adapter from the core test/MSRV pass;
6. all-target workspace checking and denied-warning Clippy;
7. denied-warning workspace rustdoc;
8. the declared Rust 1.93.0 core test baseline;
9. `git diff --check` and a clean repository state after validation.

The optional Godot adapter participates in workspace check/Clippy/rustdoc but does not define the core MSRV.

## CI

`.github/workflows/validation.yml` is intentionally a thin, read-only caller of Dornglut's immutable shared Rust validation workflow. It must delegate to `cargo validate` rather than reproduce a second command list.

Pull-request acceptance requires validation at the exact reviewed head. Accepted-main validation is the post-merge repository evidence.
