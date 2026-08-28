use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const ARCHITECTURAL_DEPENDENCIES: &[&str] = &[
    "godot",
    "runen-spatial",
    "runen-spatial-demand",
    "runen-spatial-streaming",
];

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask must live directly below the repository root")
        .to_path_buf()
}

fn architectural_dependencies(relative: &str) -> BTreeSet<String> {
    let manifest = fs::read_to_string(repository_root().join(relative)).unwrap();
    let mut in_dependency_section = false;
    let mut dependencies = BTreeSet::new();

    for raw_line in manifest.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_dependency_section = dependency_section(line);
            continue;
        }
        if !in_dependency_section || line.is_empty() {
            continue;
        }

        let Some((raw_key, value)) = line.split_once('=') else {
            continue;
        };
        let key = raw_key.trim().split('.').next().unwrap_or("");
        for dependency in ARCHITECTURAL_DEPENDENCIES {
            let aliased_package = format!("package = \"{dependency}\"");
            if key == *dependency || value.contains(&aliased_package) {
                dependencies.insert((*dependency).to_owned());
            }
        }
    }

    dependencies
}

fn dependency_section(header: &str) -> bool {
    matches!(
        header,
        "[dependencies]" | "[dev-dependencies]" | "[build-dependencies]"
    ) || header.ends_with(".dependencies]")
        || header.ends_with(".dev-dependencies]")
        || header.ends_with(".build-dependencies]")
}

fn expected(dependencies: &[&str]) -> BTreeSet<String> {
    dependencies
        .iter()
        .map(|value| (*value).to_owned())
        .collect()
}

#[test]
fn runtime_dependency_topology_matches_architecture() {
    for (manifest, dependencies) in [
        ("crates/runen_spatial/Cargo.toml", expected(&[])),
        (
            "crates/runen_spatial_demand/Cargo.toml",
            expected(&["runen-spatial"]),
        ),
        (
            "crates/runen_spatial_streaming/Cargo.toml",
            expected(&["runen-spatial", "runen-spatial-demand"]),
        ),
        (
            "adapters/godot_world_streaming/Cargo.toml",
            expected(&[
                "godot",
                "runen-spatial",
                "runen-spatial-demand",
                "runen-spatial-streaming",
            ]),
        ),
    ] {
        assert_eq!(
            architectural_dependencies(manifest),
            dependencies,
            "architectural dependency topology changed in {manifest}"
        );
    }
}
