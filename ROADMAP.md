# RunenSpatial Roadmap

The roadmap owns durable sequencing only. GitHub issues and pull requests own detailed acceptance criteria and live execution state.

## 1. Harden retained foundation contracts

Make retained foundational public values and operations intentional and internally consistent. Resolve opaque-ID representation/default semantics, redundant spatial-hash vocabulary, inconsistent level typing, silent narrowing/saturation, and constructors or `Result` surfaces whose advertised fallibility does not match their implementation.

Do not add generic geometry/index abstractions without a proven consumer.

## 2. Prove standalone conformance

Exercise the retained public packages through host-neutral public APIs without internal source access. Bind deterministic replay, bounded resource behavior, failure propagation, and package independence before publication is considered.

## 3. Cut over Runenwerk separately

Authorize downstream Runenwerk migration only after standalone conformance. Migrate in dependency order and delete duplicate Runenwerk authority component-by-component. Do not use permanent wrappers, forwarding crates, source mirrors, branch dependencies, or submodules.
