#![allow(clippy::too_many_lines)]

use std::collections::BTreeSet;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const MAX_AUTHORED_BYTES: u64 = 128 * 1024;
const WORKFLOW_PATH: &str = ".github/workflows/validation.yml";
const WORKFLOW_REVISION: &str = "624cb41adeed21a6461eb838bc7330bd0a5079fd";
const EXPECTED_WORKFLOW: &str = r#"name: RunenSpatial Validation

on:
  pull_request:
    branches:
      - main
  push:
    branches:
      - main
  workflow_dispatch:

permissions:
  contents: read

concurrency:
  group: runen-spatial-validation-${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

jobs:
  validate:
    name: Validate standalone framework
    uses: dornglut/github-workflows/.github/workflows/reusable-rust-cargo-validate.yml@624cb41adeed21a6461eb838bc7330bd0a5079fd
"#;

fn main() {
    let result = match env::args().nth(1).as_deref() {
        Some("validate") => validate(),
        _ => Err("usage: cargo validate".to_owned()),
    };

    if let Err(error) = result {
        eprintln!("validation failed: {error}");
        std::process::exit(1);
    }
}

fn validate() -> Result<(), String> {
    let root = repository_root()?;
    validate_repository_policy(&root)?;
    validate_markdown_links(&root)?;
    run_validation_commands(&root)?;
    prove_clean_repository_state(&root)
}

fn repository_root() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "xtask must live directly below the repository root".to_owned())
}

fn validate_repository_policy(root: &Path) -> Result<(), String> {
    validate_required_files(root)?;
    validate_manifest_inventory(root)?;
    validate_manifest_policy(root)?;
    validate_workflow(root)?;
    validate_path_dependencies(root)?;
    validate_repository_files(root)?;
    validate_current_authority(root)?;
    validate_provenance(root)?;
    validate_git_index(root)
}

fn validate_required_files(root: &Path) -> Result<(), String> {
    const REQUIRED: &[&str] = &[
        ".cargo/config.toml",
        ".github/workflows/validation.yml",
        "AGENTS.md",
        "ARCHITECTURE.md",
        "Cargo.lock",
        "Cargo.toml",
        "LICENSE-APACHE",
        "LICENSE-MIT",
        "README.md",
        "SECURITY.md",
        "TESTING.md",
        "docs/architecture.md",
        "docs/package-boundaries.md",
        "docs/provenance/repository-transfer.md",
        "docs/roadmap.md",
        "docs/tooling/validation.md",
        "rust-toolchain.toml",
        "xtask/Cargo.toml",
        "xtask/src/main.rs",
    ];

    for relative in REQUIRED {
        require_file(root, relative)?;
    }

    for forbidden in [
        "docs/crate-boundaries.md",
        "docs/full-roadmap-goal-prompt.md",
    ] {
        if root.join(forbidden).exists() {
            return Err(format!(
                "superseded or process-only file must be removed: {forbidden}"
            ));
        }
    }

    Ok(())
}

fn validate_manifest_inventory(root: &Path) -> Result<(), String> {
    let expected = BTreeSet::from([
        "Cargo.toml".to_owned(),
        "adapters/godot_world_streaming/Cargo.toml".to_owned(),
        "crates/chunking/Cargo.toml".to_owned(),
        "crates/spatial/Cargo.toml".to_owned(),
        "crates/spatial_index/Cargo.toml".to_owned(),
        "crates/world_core_prelude/Cargo.toml".to_owned(),
        "crates/world_streaming/Cargo.toml".to_owned(),
        "demos/chunk_streaming_demo/Cargo.toml".to_owned(),
        "xtask/Cargo.toml".to_owned(),
    ]);

    let actual = walk_files(root)?
        .into_iter()
        .filter(|path| path.file_name() == Some(OsStr::new("Cargo.toml")))
        .map(|path| relative_string(root, &path))
        .collect::<Result<BTreeSet<_>, _>>()?;

    if actual != expected {
        return Err(format!(
            "Cargo manifest inventory differs\nexpected: {expected:#?}\nactual: {actual:#?}"
        ));
    }

    Ok(())
}

fn validate_manifest_policy(root: &Path) -> Result<(), String> {
    let root_manifest = read_text(root, "Cargo.toml")?;
    for required in [
        "\"xtask\"",
        "rust-version = \"1.93.0\"",
        "repository = \"https://github.com/dornglut/runen-spatial\"",
        "publish = false",
        "unsafe_code = \"forbid\"",
        "all = { level = \"deny\", priority = -1 }",
    ] {
        require_contains("Cargo.toml", &root_manifest, required)?;
    }

    let package_manifests = [
        "crates/spatial/Cargo.toml",
        "crates/spatial_index/Cargo.toml",
        "crates/chunking/Cargo.toml",
        "crates/world_streaming/Cargo.toml",
        "crates/world_core_prelude/Cargo.toml",
        "demos/chunk_streaming_demo/Cargo.toml",
    ];

    for relative in package_manifests {
        let manifest = read_text(root, relative)?;
        for required in [
            "rust-version.workspace = true",
            "repository.workspace = true",
            "publish.workspace = true",
            "[lints]\nworkspace = true",
        ] {
            require_contains(relative, &manifest, required)?;
        }
    }

    let adapter_path = "adapters/godot_world_streaming/Cargo.toml";
    let adapter = read_text(root, adapter_path)?;
    for required in [
        "rust-version.workspace = true",
        "repository.workspace = true",
        "publish.workspace = true",
        "[lints.rust]\nunsafe_code = \"allow\"",
        "[lints.clippy]\nall = { level = \"deny\", priority = -1 }",
    ] {
        require_contains(adapter_path, &adapter, required)?;
    }

    let xtask = read_text(root, "xtask/Cargo.toml")?;
    require_contains("xtask/Cargo.toml", &xtask, "publish = false")?;
    require_contains("xtask/Cargo.toml", &xtask, "[lints]\nworkspace = true")
}

fn validate_workflow(root: &Path) -> Result<(), String> {
    let workflow = read_text(root, WORKFLOW_PATH)?;
    if workflow != EXPECTED_WORKFLOW {
        return Err(format!(
            "{WORKFLOW_PATH} must remain the exact read-only shared-workflow caller"
        ));
    }

    require_contains(WORKFLOW_PATH, &workflow, WORKFLOW_REVISION)?;
    for retired in [
        "b6caad377102ca73794efaf734a65903b8efa829",
        "79405c457b5b99d5cb9957c9bcdc475109e1e3bf",
    ] {
        if workflow.contains(retired) {
            return Err(format!(
                "{WORKFLOW_PATH} uses retired shared revision {retired}"
            ));
        }
    }

    Ok(())
}

fn validate_path_dependencies(root: &Path) -> Result<(), String> {
    let canonical_root = fs::canonicalize(root)
        .map_err(|error| format!("failed to canonicalize repository root: {error}"))?;

    for manifest_path in walk_files(root)?
        .into_iter()
        .filter(|path| path.file_name() == Some(OsStr::new("Cargo.toml")))
    {
        let manifest = fs::read_to_string(&manifest_path)
            .map_err(|error| format!("failed to read {}: {error}", manifest_path.display()))?;
        let parent = manifest_path
            .parent()
            .ok_or_else(|| format!("manifest has no parent: {}", manifest_path.display()))?;

        for line in manifest.lines() {
            let Some(path_value) = toml_inline_path(line) else {
                continue;
            };
            let dependency = parent.join(path_value);
            let canonical_dependency = fs::canonicalize(&dependency).map_err(|error| {
                format!(
                    "path dependency from {} does not resolve: {} ({error})",
                    manifest_path.display(),
                    dependency.display()
                )
            })?;
            if !canonical_dependency.starts_with(&canonical_root) {
                return Err(format!(
                    "external path dependency is forbidden: {} -> {}",
                    manifest_path.display(),
                    canonical_dependency.display()
                ));
            }
        }
    }

    Ok(())
}

fn toml_inline_path(line: &str) -> Option<&str> {
    let marker = "path = \"";
    let start = line.find(marker)? + marker.len();
    let end = line[start..].find('"')? + start;
    Some(&line[start..end])
}

fn validate_repository_files(root: &Path) -> Result<(), String> {
    for path in walk_files(root)? {
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("failed to inspect {}: {error}", path.display()))?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "tracked repository symlink is forbidden: {}",
                path.display()
            ));
        }
        if is_authored_text(&path) && metadata.len() > MAX_AUTHORED_BYTES {
            return Err(format!(
                "authored text file exceeds 128 KiB: {} ({} bytes)",
                path.display(),
                metadata.len()
            ));
        }
    }

    Ok(())
}

fn is_authored_text(path: &Path) -> bool {
    if path.file_name() == Some(OsStr::new("Cargo.lock")) {
        return false;
    }
    if matches!(
        path.file_name().and_then(OsStr::to_str),
        Some(
            "README.md"
                | "AGENTS.md"
                | "ARCHITECTURE.md"
                | "TESTING.md"
                | "SECURITY.md"
                | "LICENSE-MIT"
                | "LICENSE-APACHE"
        )
    ) {
        return true;
    }
    matches!(
        path.extension().and_then(OsStr::to_str),
        Some("md" | "rs" | "toml" | "yml" | "yaml" | "json" | "gd" | "txt")
    )
}

fn validate_current_authority(root: &Path) -> Result<(), String> {
    const ACTIVE_DOCS: &[&str] = &[
        "README.md",
        "AGENTS.md",
        "ARCHITECTURE.md",
        "TESTING.md",
        "docs/architecture.md",
        "docs/chunking-model.md",
        "docs/godot-integration.md",
        "docs/grid-composition.md",
        "docs/package-boundaries.md",
        "docs/roadmap.md",
        "docs/runenwerk-integration.md",
        "docs/spatial-model.md",
        "docs/streaming-lifecycle.md",
        "docs/tooling/validation.md",
    ];

    for relative in ACTIVE_DOCS {
        let text = read_text(root, relative)?;
        for forbidden in ["Crystonix/", "spatial_streaming"] {
            if text.contains(forbidden) {
                return Err(format!(
                    "active documentation contains stale authority `{forbidden}`: {relative}"
                ));
            }
        }
    }

    let demo_manifest = read_text(root, "demos/chunk_streaming_demo/Cargo.toml")?;
    if demo_manifest.contains("../../../") || demo_manifest.contains("tile_topology") {
        return Err("demo must not depend on an external sibling checkout".to_owned());
    }

    let include_macro = ["include!", "("].concat();
    let include_bytes_macro = ["include_bytes!", "("].concat();
    let xtask_source = root.join("xtask/src/main.rs");

    for path in walk_files(root)?
        .into_iter()
        .filter(|path| path.extension() == Some(OsStr::new("rs")))
    {
        if path == xtask_source {
            continue;
        }
        let text = fs::read_to_string(&path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
        if text.contains(&include_macro) || text.contains(&include_bytes_macro) {
            return Err(format!(
                "source include authority requires explicit review: {}",
                path.display()
            ));
        }
    }

    Ok(())
}

fn validate_provenance(root: &Path) -> Result<(), String> {
    let provenance = read_text(root, "docs/provenance/repository-transfer.md")?;
    for required in [
        "aschenrot/spatial_streaming",
        "dornglut/runen-spatial",
        "2a87094cb4ca4ed48238b416f4d4121cb5e074a1",
        "8d5dae4123dd3e67f572f3c0c32aac7362975aaf",
        "private normalization",
        "not a complete full-history secret scan",
    ] {
        require_contains(
            "docs/provenance/repository-transfer.md",
            &provenance,
            required,
        )?;
    }
    Ok(())
}

fn validate_git_index(root: &Path) -> Result<(), String> {
    let output = run_output(root, "git", &["ls-files", "--stage"])?;
    for line in output.lines() {
        if line.starts_with("160000 ") {
            return Err(format!("gitlink/submodule is forbidden: {line}"));
        }
    }
    Ok(())
}

fn validate_markdown_links(root: &Path) -> Result<(), String> {
    for path in walk_files(root)?
        .into_iter()
        .filter(|path| path.extension() == Some(OsStr::new("md")))
    {
        let text = fs::read_to_string(&path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
        for target in markdown_targets(&text) {
            if target.is_empty()
                || target.starts_with('#')
                || target.starts_with("http://")
                || target.starts_with("https://")
                || target.starts_with("mailto:")
            {
                continue;
            }
            let target = target.split('#').next().unwrap_or_default();
            let target = target.split('?').next().unwrap_or_default();
            if target.is_empty() {
                continue;
            }
            let base = path.parent().unwrap_or(root);
            let resolved = base.join(target);
            if !resolved.exists() {
                return Err(format!(
                    "broken Markdown link in {}: {target}",
                    path.display()
                ));
            }
        }
    }
    Ok(())
}

fn markdown_targets(text: &str) -> Vec<&str> {
    let mut targets = Vec::new();
    let mut remaining = text;
    while let Some(start) = remaining.find("](") {
        remaining = &remaining[start + 2..];
        let Some(end) = remaining.find(')') else {
            break;
        };
        targets.push(remaining[..end].trim_matches(['<', '>']));
        remaining = &remaining[end + 1..];
    }
    targets
}

fn run_validation_commands(root: &Path) -> Result<(), String> {
    run(
        root,
        "cargo",
        &["metadata", "--format-version", "1", "--locked", "--no-deps"],
    )?;
    run(root, "cargo", &["fmt", "--all", "--", "--check"])?;
    run(
        root,
        "cargo",
        &[
            "test",
            "--workspace",
            "--exclude",
            "godot_world_streaming",
            "--locked",
        ],
    )?;
    run(
        root,
        "cargo",
        &["check", "--workspace", "--all-targets", "--locked"],
    )?;
    run(
        root,
        "cargo",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--locked",
            "--",
            "-D",
            "warnings",
        ],
    )?;
    run_with_env(
        root,
        "cargo",
        &["doc", "--workspace", "--no-deps", "--locked"],
        "RUSTDOCFLAGS",
        "-D warnings",
    )?;
    run(
        root,
        "cargo",
        &[
            "+1.93.0",
            "test",
            "--workspace",
            "--exclude",
            "godot_world_streaming",
            "--locked",
        ],
    )?;
    run(root, "git", &["diff", "--check"])
}

fn prove_clean_repository_state(root: &Path) -> Result<(), String> {
    let status = run_output(root, "git", &["status", "--short"])?;
    if status.trim().is_empty() {
        Ok(())
    } else {
        Err(format!(
            "validation changed the tracked repository:\n{status}"
        ))
    }
}

fn run(root: &Path, program: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
        .args(args)
        .current_dir(root)
        .stdin(Stdio::null())
        .status()
        .map_err(|error| format!("failed to run {program} {}: {error}", args.join(" ")))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("command failed: {program} {}", args.join(" ")))
    }
}

fn run_with_env(
    root: &Path,
    program: &str,
    args: &[&str],
    key: &str,
    value: &str,
) -> Result<(), String> {
    let status = Command::new(program)
        .args(args)
        .env(key, value)
        .current_dir(root)
        .stdin(Stdio::null())
        .status()
        .map_err(|error| format!("failed to run {program} {}: {error}", args.join(" ")))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("command failed: {program} {}", args.join(" ")))
    }
}

fn run_output(root: &Path, program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(root)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("failed to run {program} {}: {error}", args.join(" ")))?;
    if !output.status.success() {
        return Err(format!("command failed: {program} {}", args.join(" ")));
    }
    String::from_utf8(output.stdout)
        .map_err(|error| format!("{program} output was not UTF-8: {error}"))
}

fn walk_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    walk_directory(root, root, &mut files)?;
    files.sort();
    Ok(files)
}

fn walk_directory(root: &Path, directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(directory)
        .map_err(|error| format!("failed to read {}: {error}", directory.display()))?
    {
        let entry = entry.map_err(|error| format!("failed to read directory entry: {error}"))?;
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .map_err(|error| format!("failed to relativize {}: {error}", path.display()))?;
        if matches!(
            relative
                .components()
                .next()
                .and_then(|component| component.as_os_str().to_str()),
            Some(".git" | "target")
        ) {
            continue;
        }
        let file_type = entry
            .file_type()
            .map_err(|error| format!("failed to inspect {}: {error}", path.display()))?;
        if file_type.is_dir() {
            walk_directory(root, &path, files)?;
        } else {
            files.push(path);
        }
    }
    Ok(())
}

fn relative_string(root: &Path, path: &Path) -> Result<String, String> {
    path.strip_prefix(root)
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
        .map_err(|error| format!("failed to relativize {}: {error}", path.display()))
}

fn require_file(root: &Path, relative: &str) -> Result<(), String> {
    if root.join(relative).is_file() {
        Ok(())
    } else {
        Err(format!("required file is missing: {relative}"))
    }
}

fn read_text(root: &Path, relative: &str) -> Result<String, String> {
    fs::read_to_string(root.join(relative))
        .map_err(|error| format!("failed to read {relative}: {error}"))
}

fn require_contains(path: &str, text: &str, required: &str) -> Result<(), String> {
    if text.contains(required) {
        Ok(())
    } else {
        Err(format!("{path} is missing required text: {required}"))
    }
}
