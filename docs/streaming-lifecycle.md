# Streaming Availability Lifecycle

This document explains the current responsibility of `runen-spatial-streaming`. Exact public signatures and transition tests are owned by the package source, rustdoc, and tests.

## Ownership

The streaming package coordinates content-agnostic availability work: desired/undesired chunk intent, budgeted load/unload request issuance, correlation with backend events, deterministic request/event ordering, reversal, failure reporting, and diagnostics.

The backend owns actual IO and payload/resource creation. The host owns product semantics, payload caches, gameplay/ECS activation, rendering, retry/backoff policy not explicitly modeled by the framework, and application degradation decisions.

## Current lifecycle

The implementation currently represents requested, loaded, active, parked, evicted, and failed conditions in one combined lifecycle enum. Load/unload requests are correlated by request IDs and drained through per-tick budgets. Backend events drive completion/failure transitions.

This combined model is the current baseline, not proof that availability, operation, activation, and failure are one durable state dimension.

## Known contract debt

The current baseline still requires explicit resolution of:

- load failure versus unload failure semantics and whether residency is preserved;
- request-ID exhaustion/non-reuse behavior;
- post-load payload/result pairing;
- queue and record-retention bounds during long-running exploration;
- orthogonal desired, observed availability, in-flight operation, activation, and last-failure state where required;
- deterministic pressure/event behavior under bounded capacity.

Those corrections must remain content-agnostic and must not move backend IO or host activation policy into this package.
