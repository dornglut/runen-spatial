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
    let mut arguments = env::args().skip(1);
    let result = match (arguments.next().as_deref(), arguments.next()) {
        (Some("validate"), None) => validate(),
        _ => Err("usage: cargo validate".to_owned()),
    };

    if let Err(error) = result {
        eprintln!("validation failed: {error}");
        std::process::exit(1);
    }
}

fn validate() -> Result<(), String> {
    let root = repository_root()?;
    validate_repository_contract(&root)?;
    validate_markdown(&root)?;
    run_validation_commands(&root)?;
    prove_clean_repository_state(&root)?;
    println!("repository validation passed");
    Ok(())
}

fn repository_root() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "xtask must live directly below the repository root".to_owned())
}

fn validate_repository_contract(root: &Path) -> Result<(), String> {
    validate_required_and_retired_paths(root)?;
    validate_manifest_inventory(root)?;
    validate_manifest_policy(root)?;
    validate_retired_surfaces(root)?;
    validate_workflow(root)?;
    validate_path_dependencies(root)?;
    validate_repository_files(root)?;
    validate_current_authority(root)?;
    validate_provenance(root)?;
    validate_git_index(root)
}

fn validate_required_and_retired_paths(root: &Path) -> Result<(), String> {
    const REQUIRED: &[&str] = &[
        ".cargo/config.toml",
        ".github/workflows/validation.yml",
        "AGENTS.md",
        "ARCHITECTURE.md",
        "Cargo.lock",
        "Cargo.toml",
        "LICENSE",
        "LICENSING.md",
        "README.md",
        "ROADMAP.md",
        "SECURITY.md",
        "TESTING.md",
        "docs/documentation-architecture.md",
        "docs/provenance/repository-transfer.md",
        "rust-toolchain.toml",
        "xtask/Cargo.toml",
        "xtask/src/main.rs",
    ];

    const FORBIDDEN: &[&str] = &[
        "LICENSE-MIT",
        "LICENSE-APACHE",
        "docs/architecture.md",
        "docs/chunking-model.md",
        "docs/crate-boundaries.md",
        "docs/full-roadmap-goal-prompt.md",
        "docs/investigations/runenspatial-extraction-boundary.md",
        "docs/package-boundaries.md",
        "docs/roadmap.md",
        "docs/runenwerk-integration.md",
        "docs/tooling/validation.md",
        "crates/runen_spatial_index",
        "crates/runen_spatial/src/bounds.rs",
        "crates/world_core_prelude",
        "demos/chunk_streaming_demo",
    ];

    for relative in REQUIRED {
        require_file(root, relative)?;
    }

    for relative in FORBIDDEN {
        if root.join(relative).exists() {
            return Err(format!(
                "retired authority or surface must remain absent: {relative}"
            ));
        }
    }

    Ok(())
}

fn validate_manifest_inventory(root: &Path) -> Result<(), String> {
    let expected = BTreeSet::from([
        "Cargo.toml".to_owned(),
        "adapters/godot_world_streaming/Cargo.toml".to_owned(),
        "crates/runen_spatial/Cargo.toml".to_owned(),
        "crates/runen_spatial_demand/Cargo.toml".to_owned(),
        "crates/runen_spatial_streaming/Cargo.toml".to_owned(),
        "xtask/Cargo.toml".to_owned(),
    ]);

    let actual = walk_files(root)?
        .into_iter()
        .filter(|path| path.file_name() == Some(OsStr::new("Cargo.toml")))
        .map(|path| relative_string(root, &path))
        .collect::<Result<BTreeSet<_>, _>>()?;

    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "Cargo manifest inventory differs\nexpected: {expected:#?}\nactual: {actual:#?}"
        ))
    }
}

fn validate_manifest_policy(root: &Path) -> Result<(), String> {
    let root_manifest = read_text(root, "Cargo.toml")?;
    for required in [
        "\"xtask\"",
        "rust-version = \"1.93.0\"",
        "license = \"GPL-3.0-only\"",
        "repository = \"https://github.com/dornglut/runen-spatial\"",
        "publish = false",
        "unsafe_code = \"forbid\"",
        "all = { level = \"deny\", priority = -1 }",
    ] {
        require_contains("Cargo.toml", &root_manifest, required)?;
    }

    for relative in [
        "crates/runen_spatial/Cargo.toml",
        "crates/runen_spatial_demand/Cargo.toml",
        "crates/runen_spatial_streaming/Cargo.toml",
    ] {
        let manifest = read_text(root, relative)?;
        for required in [
            "rust-version.workspace = true",
            "license.workspace = true",
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
        "license.workspace = true",
        "repository.workspace = true",
        "publish.workspace = true",
        "[lints.rust]\nunsafe_code = \"allow\"",
        "[lints.clippy]\nall = { level = \"deny\", priority = -1 }",
    ] {
        require_contains(adapter_path, &adapter, required)?;
    }
    if adapter.contains("rust-version") {
        return Err(format!(
            "{adapter_path} must not claim the core MSRV until adapter-specific proof exists"
        ));
    }

    let xtask = read_text(root, "xtask/Cargo.toml")?;
    for required in [
        "license.workspace = true",
        "publish = false",
        "[lints]\nworkspace = true",
    ] {
        require_contains("xtask/Cargo.toml", &xtask, required)?;
    }

    let lockfile = read_text(root, "Cargo.lock")?;
    if lockfile.contains("name = \"runen-spatial-index\"") {
        return Err("Cargo.lock contains retired runen-spatial-index package".to_owned());
    }

    Ok(())
}

fn validate_retired_surfaces(root: &Path) -> Result<(), String> {
    const RETIRED_PACKAGES: &[&str] = &["spatial", "spatial_index", "chunking", "world_streaming"];
    let retired_api = [
        ["World", "LocalPosition"].concat(),
        ["Camera", "RelativeFrame"].concat(),
        ["build_camera", "_relative_frame"].concat(),
        ["fixed", "_point_scale"].concat(),
        ["quantization", "_scale"].concat(),
        ["Spatial", "Aabb3"].concat(),
        ["Spatial", "Point3"].concat(),
        ["Chunk", "Streamer"].concat(),
        ["Chunk", "StreamingConfig"].concat(),
        ["Chunk", "StreamingMode"].concat(),
        ["Chunk", "LoadOrder"].concat(),
        ["Streaming", "Focus"].concat(),
        ["Chunk", "SetDiff"].concat(),
        ["Farthest", "First"].concat(),
    ];

    for manifest_path in walk_files(root)?
        .into_iter()
        .filter(|path| path.file_name() == Some(OsStr::new("Cargo.toml")))
    {
        let relative = relative_string(root, &manifest_path)?;
        let manifest = fs::read_to_string(&manifest_path)
            .map_err(|error| format!("failed to read {relative}: {error}"))?;

        for package in RETIRED_PACKAGES {
            let retired_name = format!(r#"name = "{package}""#);
            if manifest
                .lines()
                .map(str::trim_start)
                .any(|line| line == retired_name)
            {
                return Err(format!("retired package name in {relative}: {package}"));
            }
        }

        if manifest.contains("runen-spatial-index") {
            return Err(format!("retired spatial-index package in {relative}"));
        }
        if manifest.contains("world_core_prelude") {
            return Err(format!("retired broad prelude package in {relative}"));
        }
    }

    for source_path in walk_files(root)?
        .into_iter()
        .filter(|path| path.extension() == Some(OsStr::new("rs")))
        .filter(|path| {
            relative_string(root, path).is_ok_and(|relative| relative != "xtask/src/main.rs")
        })
    {
        let relative = relative_string(root, &source_path)?;
        let source = fs::read_to_string(&source_path)
            .map_err(|error| format!("failed to read {relative}: {error}"))?;

        for retired_name in &retired_api {
            if source.contains(retired_name) {
                return Err(format!(
                    "retired foundational API in {relative}: {retired_name}"
                ));
            }
        }

        if source.contains("runen_spatial_index") {
            return Err(format!("retired spatial-index crate import in {relative}"));
        }

        for package in RETIRED_PACKAGES {
            if contains_standalone_crate_path(&source, package) {
                return Err(format!("retired crate path in {relative}: {package}::"));
            }
        }
    }

    Ok(())
}

fn contains_standalone_crate_path(source: &str, package: &str) -> bool {
    let token = [package, "::"].concat();
    let mut search_start = 0;

    while let Some(relative_index) = source[search_start..].find(&token) {
        let index = search_start + relative_index;
        let preceding = source[..index].chars().next_back();
        if preceding.is_none_or(|character| !character.is_ascii_alphanumeric() && character != '_')
        {
            return true;
        }
        search_start = index + token.len();
    }

    false
}

fn validate_workflow(root: &Path) -> Result<(), String> {
    let workflow = read_text(root, WORKFLOW_PATH)?;
    if workflow != EXPECTED_WORKFLOW {
        return Err(format!(
            "{WORKFLOW_PATH} must remain the exact read-only shared-workflow caller"
        ));
    }
    require_contains(WORKFLOW_PATH, &workflow, WORKFLOW_REVISION)
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
                "repository symlink is forbidden: {}",
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
    matches!(
        path.extension().and_then(OsStr::to_str),
        Some("md" | "rs" | "toml" | "yml" | "yaml" | "json" | "gd" | "txt")
    ) || matches!(
        path.file_name().and_then(OsStr::to_str),
        Some("LICENSE" | "LICENSING.md")
    )
}

fn validate_current_authority(root: &Path) -> Result<(), String> {
    const ACTIVE_DOCS: &[&str] = &[
        "README.md",
        "AGENTS.md",
        "ARCHITECTURE.md",
        "ROADMAP.md",
        "TESTING.md",
        "SECURITY.md",
        "docs/documentation-architecture.md",
        "docs/godot-integration.md",
        "docs/grid-composition.md",
        "docs/spatial-demand.md",
        "docs/spatial-model.md",
        "docs/streaming-lifecycle.md",
    ];
    const STALE_AUTHORITY: &[&str] = &[
        "Crystonix/",
        "aschenrot/spatial_streaming",
        "## Current child",
        "Accepted base: `",
        "Current PR:",
        "CI run ",
        "runen-spatial-index",
        "SpatialAabb3",
        "SpatialPoint3",
        "ChunkStreamer",
        "ChunkStreamingConfig",
        "ChunkStreamingMode",
        "ChunkLoadOrder",
        "StreamingFocus",
        "ChunkSetDiff",
        "FarthestFirst",
    ];

    for relative in ACTIVE_DOCS {
        let text = read_text(root, relative)?;
        for forbidden in STALE_AUTHORITY {
            if text.contains(forbidden) {
                return Err(format!(
                    "active documentation contains stale/live authority {forbidden:?}: {relative}"
                ));
            }
        }
    }

    let include_macro = ["include!", "("].concat();
    let include_bytes_macro = ["include_bytes!", "("].concat();
    for path in walk_files(root)?
        .into_iter()
        .filter(|path| path.extension() == Some(OsStr::new("rs")))
        .filter(|path| {
            relative_string(root, path).is_ok_and(|relative| relative != "xtask/src/main.rs")
        })
    {
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
        "public",
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

fn validate_markdown(root: &Path) -> Result<(), String> {
    for file in walk_files(root)?
        .into_iter()
        .filter(|path| path.extension() == Some(OsStr::new("md")))
    {
        let content = fs::read_to_string(&file)
            .map_err(|error| format!("failed to read {}: {error}", file.display()))?;

        if let Some(target) = reference_style_local_targets(&content).first() {
            return Err(format!(
                "repository-local Markdown links must use inline relative syntax in {}: {target}",
                relative_string(root, &file)?
            ));
        }

        for target in markdown_link_targets(&content) {
            let Some(local_target) = local_markdown_target(&target) else {
                continue;
            };
            let parent = file
                .parent()
                .ok_or_else(|| format!("{} has no parent directory", file.display()))?;
            if !parent.join(local_target).exists() {
                return Err(format!(
                    "broken Markdown link in {}: {target}",
                    relative_string(root, &file)?
                ));
            }
        }
    }

    Ok(())
}

fn markdown_link_targets(content: &str) -> Vec<String> {
    markdown_lines_outside_fences(content)
        .flat_map(inline_link_targets)
        .collect()
}

fn reference_style_local_targets(content: &str) -> Vec<String> {
    markdown_lines_outside_fences(content)
        .filter_map(reference_definition_target)
        .filter(|target| local_markdown_target(target).is_some())
        .collect()
}

fn markdown_lines_outside_fences(content: &str) -> impl Iterator<Item = &str> {
    let mut active_fence: Option<&'static str> = None;
    content.lines().filter(move |line| {
        let trimmed = line.trim_start();
        let marker = if trimmed.starts_with("```") {
            Some("```")
        } else if trimmed.starts_with("~~~") {
            Some("~~~")
        } else {
            None
        };

        if let Some(marker) = marker {
            match active_fence {
                None => active_fence = Some(marker),
                Some(active) if active == marker => active_fence = None,
                Some(_) => {}
            }
            return false;
        }

        active_fence.is_none()
    })
}

fn inline_link_targets(line: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let mut cursor = 0;

    while let Some(relative_start) = line[cursor..].find("](") {
        let start = cursor + relative_start + 2;
        let Some(relative_end) = line[start..].find(')') else {
            break;
        };
        let end = start + relative_end;
        if let Some(target) = normalized_markdown_target(&line[start..end]) {
            targets.push(target.to_owned());
        }
        cursor = end + 1;
    }

    targets
}

fn reference_definition_target(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix('[')?;
    if rest.starts_with('^') {
        return None;
    }
    let marker_end = rest.find("]:")?;
    normalized_markdown_target(&rest[marker_end + 2..]).map(str::to_owned)
}

fn normalized_markdown_target(raw: &str) -> Option<&str> {
    let raw = raw.trim();
    let target = if raw.starts_with('<') {
        raw.strip_prefix('<')?.split_once('>')?.0
    } else {
        raw.split_whitespace().next()?
    };
    (!target.is_empty()).then_some(target)
}

fn local_markdown_target(target: &str) -> Option<&str> {
    if target.starts_with('#')
        || target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with("mailto:")
        || target.starts_with("data:")
    {
        return None;
    }

    let path = target.split('#').next().unwrap_or("");
    (!path.is_empty() && !Path::new(path).is_absolute()).then_some(path)
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

fn run(root: &Path, program: &str, arguments: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
        .args(arguments)
        .current_dir(root)
        .stdin(Stdio::null())
        .status()
        .map_err(|error| format!("failed to run {program} {}: {error}", arguments.join(" ")))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("command failed: {program} {}", arguments.join(" ")))
    }
}

fn run_with_env(
    root: &Path,
    program: &str,
    arguments: &[&str],
    key: &str,
    value: &str,
) -> Result<(), String> {
    let status = Command::new(program)
        .args(arguments)
        .env(key, value)
        .current_dir(root)
        .stdin(Stdio::null())
        .status()
        .map_err(|error| format!("failed to run {program} {}: {error}", arguments.join(" ")))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("command failed: {program} {}", arguments.join(" ")))
    }
}

fn run_output(root: &Path, program: &str, arguments: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(arguments)
        .current_dir(root)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("failed to run {program} {}: {error}", arguments.join(" ")))?;
    if !output.status.success() {
        return Err(format!("command failed: {program} {}", arguments.join(" ")));
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
