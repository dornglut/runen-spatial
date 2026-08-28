# Documentation Architecture

RunenSpatial uses one durable owner per concern. A document may link to another concern but must not restate its authority.

## Ownership map

| Concern | Canonical owner |
| --- | --- |
| Purpose, boundary, maturity, navigation | `README.md` |
| Package/dependency/current ownership structure | `ARCHITECTURE.md` |
| Durable future sequencing | `ROADMAP.md` |
| Mechanical repository validation | `TESTING.md` |
| Agent/executor repository rules | `AGENTS.md` |
| Security reporting/support scope | `SECURITY.md` |
| Current licensing representation | `LICENSE` and `LICENSING.md` |
| Coordinate/frame/addressing concepts | `docs/spatial-model.md` |
| Demand concepts and current demand baseline | `docs/spatial-demand.md` |
| Availability-controller concepts and current lifecycle | `docs/streaming-lifecycle.md` |
| Grid/clipmap/ring composition guidance | `docs/grid-composition.md` |
| Retained Godot translation integration boundary | `docs/godot-integration.md` |
| Repository-transfer history | `docs/provenance/repository-transfer.md` |
| Runtime API and behavior | Rust public API, rustdoc, and tests in the owning package |
| Detailed active work/acceptance | GitHub issues and pull requests |
| Organization governance/standards | `dornglut/engineering` |

## Authority rules

- Rust code, public rustdoc, and tests own implemented API behavior. Conceptual docs explain contracts; they do not maintain copied symbol inventories or pseudo-API declarations.
- `ROADMAP.md` describes future sequence, not current branch/PR/CI state.
- `README.md` is orientation and navigation, not a second architecture or roadmap.
- `AGENTS.md` contains repository-local executor rules only. It does not copy live pickup state or organization governance.
- Historical exact revisions and former repository names may appear in provenance/history material, not as current authority.
- Git history owns superseded document text. Deleted authorities are not retained as forwarding stubs.
- A new durable document requires a distinct concern that is not already owned by an existing artifact.
