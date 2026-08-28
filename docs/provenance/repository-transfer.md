# Repository Transfer Provenance

This document records repository-transfer history. It is not current architecture, roadmap, or execution authority.

## Current repository

The current repository is `dornglut/runen-spatial` and is public. Current architecture and work authority are owned by the root repository documents and live GitHub issues, not by this provenance record.

## Historical source and normalization

The repository was transferred from `aschenrot/spatial_streaming` with visible commit continuity preserved.

At transfer and during early private normalization, the repository was not yet public. References to private normalization below describe that historical phase only.

Key historical revisions include:

- transferred source baseline: `2a87094cb4ca4ed48238b416f4d4121cb5e074a1`;
- first accepted Dornglut architecture revision: `8d5dae4123dd3e67f572f3c0c32aac7362975aaf`;
- governance/validation normalization revision: `db69f176194bdd3730558acbe0c84dfe5d3ddf59`.

The former repository name and owner are provenance evidence only.

## Evidence and limits

The transfer review verified repository identity, visible commit continuity, current source at the time, and the preserved transferred head. It supported private normalization before later public visibility.

That review was not a complete full-history secret scan and did not independently classify every historical Git object, contributor assertion, large binary, or rewritten ancestor. Later public visibility does not retroactively broaden that historical review.

Release/publication readiness must apply the current Dornglut licensing, provenance, dependency/source-origin, and release standards rather than infer readiness from the transfer review.

## Extraction status

Runenwerk has not completed a dependency cutover. Its current workspace still contains internal spatial, spatial-index, chunking, and world-streaming packages rather than depending on this repository.

A future downstream cutover must be separately authorized, migrate one accepted component at a time, preserve Runenwerk-owned semantics in Runenwerk, and delete each duplicate without leaving forwarding crates, source mirrors, external path dependencies, or submodules as the final state.
