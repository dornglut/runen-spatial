# RunenSpatial Roadmap

The roadmap owns durable sequencing only. GitHub issues and pull requests own detailed acceptance criteria and live execution state.

## 1. Prove standalone conformance

Exercise the retained public packages through host-neutral public APIs without internal source access. Bind deterministic replay, bounded resource behavior, failure propagation, and package independence before publication is considered.

## 2. Cut over Runenwerk separately

Authorize downstream Runenwerk migration only after standalone conformance. Migrate in dependency order and delete duplicate Runenwerk authority component-by-component. Do not use permanent wrappers, forwarding crates, source mirrors, branch dependencies, or submodules.
