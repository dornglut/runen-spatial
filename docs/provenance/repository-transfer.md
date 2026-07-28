# Repository Transfer Provenance

## Current authority

The current repository is `dornglut/runen-spatial`. It is the candidate standalone authority for RunenSpatial and remains private during normalization and architectural correction.

## Historical source

The repository was transferred from `aschenrot/spatial_streaming` with visible commit continuity preserved.

Key revisions:

- transferred source baseline: `2a87094cb4ca4ed48238b416f4d4121cb5e074a1`;
- first accepted Dornglut architecture revision: `8d5dae4123dd3e67f572f3c0c32aac7362975aaf`.

The earlier name and owner are provenance evidence only. They are not current package, documentation, or execution authority.

## Evidence and limits

Connector review verified repository identity, visible commit continuity, current source, and the preserved transferred head. The transfer was appropriate for private normalization.

That review was not a complete full-history secret scan and did not independently classify every historical Git object, contributor assertion, large binary, or rewritten ancestor.

Before public visibility or package publication, release readiness must include:

- a full-history secret and large-object review appropriate to Dornglut policy;
- license and contributor provenance checks;
- dependency and source-origin checks;
- explicit recording of accepted historical exceptions, if any.

## Extraction status

Runenwerk has not completed a dependency cutover. It still contains duplicate internal spatial, demand, and streaming source and a mixed engine lifecycle.

Future cutover work must migrate one accepted component at a time, preserve Runenwerk-owned semantics in Runenwerk, and delete the corresponding duplicate without leaving forwarding crates, source mirrors, external path dependencies, or submodules as the final state.
