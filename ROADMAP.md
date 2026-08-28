# RunenSpatial Roadmap

The roadmap owns durable sequencing only. GitHub issues and pull requests own detailed acceptance criteria and live execution state.

## 1. Harden foundation invariants

Make foundational public values valid by construction or explicitly checked. Resolve opaque-ID representation, bounds validity, redundant hash vocabulary, and any remaining silent narrowing/saturation or impossible fallible constructors.

## 2. Prune provisional package and integration surfaces

Audit the spatial-index package against real consumer and complexity requirements. Remove it if no durable independent boundary is proven.

Decide the permanent disposition of the framework-local Godot adapter and demo from maintained-consumer evidence. Engine-specific integration must not survive here solely because it was inherited.

## 3. Establish deterministic multi-source spatial demand

Replace the single-focus planner with bounded, atomic, deterministic multi-source demand: explicit source identity, complete source replacement/removal, source-local hysteresis, pins, total ordering/ranks, pressure limits, and deterministic deltas.

Model the actual axis-aligned chunk-volume contract. Do not preserve a geometry selector when alternative variants execute identical semantics.

## 4. Correct availability lifecycle semantics

Replace the combined lifecycle with orthogonal desired/availability/operation/failure state where that separation is required by observable behavior. Correct request identity/exhaustion, unload-failure residency, payload/result pairing, queue pressure, reversal, and long-running record-retention semantics.

## 5. Prove standalone conformance

Exercise the retained public packages through host-neutral public APIs without internal source access. Bind deterministic replay, bounded resource behavior, failure propagation, and package independence before publication is considered.

## 6. Cut over Runenwerk separately

Authorize downstream Runenwerk migration only after standalone conformance. Migrate in dependency order and delete duplicate Runenwerk authority component-by-component. Do not use permanent wrappers, forwarding crates, source mirrors, branch dependencies, or submodules.
