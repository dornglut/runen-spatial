# RunenSpatial Roadmap

The roadmap owns durable sequencing only. GitHub issues and pull requests own detailed acceptance criteria and live execution state.

## 1. Harden retained foundation contracts

Make retained foundational public values and operations intentional and internally consistent. Resolve opaque-ID representation/default semantics, redundant spatial-hash vocabulary, inconsistent level typing, silent narrowing/saturation, and constructors or `Result` surfaces whose advertised fallibility does not match their implementation.

Do not add generic geometry/index abstractions without a proven consumer.

## 2. Establish deterministic multi-source spatial demand

Replace the single-focus planner with bounded, atomic, deterministic multi-source demand: explicit source identity, complete source replacement/removal, source-local hysteresis, pins, total ordering/ranks, pressure limits, and deterministic deltas.

Model the actual axis-aligned chunk-volume contract. Do not preserve a geometry selector when alternative variants execute identical semantics.

## 3. Correct availability lifecycle semantics

Replace the combined lifecycle with orthogonal desired/availability/operation/failure state where that separation is required by observable behavior. Correct request identity/exhaustion, unload-failure residency, payload/result pairing, queue pressure, reversal, and long-running record-retention semantics.

## 4. Harden retained integration contracts

After the host-neutral demand and lifecycle contracts stabilize, harden the retained `godot_world_streaming` translation boundary against those contracts. Keep Godot-specific scene, provider, cache, realization, and product policy outside the framework packages, and keep the adapter API pre-release until its translation behavior is independently proven.

## 5. Prove standalone conformance

Exercise the retained public packages through host-neutral public APIs without internal source access. Bind deterministic replay, bounded resource behavior, failure propagation, and package independence before publication is considered.

## 6. Cut over Runenwerk separately

Authorize downstream Runenwerk migration only after standalone conformance. Migrate in dependency order and delete duplicate Runenwerk authority component-by-component. Do not use permanent wrappers, forwarding crates, source mirrors, branch dependencies, or submodules.
