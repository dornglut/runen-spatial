# Streaming Availability Lifecycle

This document explains the responsibility of `runen-spatial-streaming`. Exact public signatures and transition tests are owned by the package source, rustdoc, and tests.

## Ownership

The streaming package coordinates content-agnostic availability work: desired/undesired chunk intent, budgeted load/unload request issuance, correlation with backend events, deterministic request/event ordering, reversal, blocking failure state, and diagnostics.

The backend owns actual IO and payload/resource creation. The host owns product semantics, payload caches, gameplay/ECS activation, rendering, retry timing/backoff, and application degradation policy.

A provider result is paired to framework work by its nonzero request ID, world-qualified `ChunkId`, and load/unload operation. Payload/resource transfer is not part of the RunenSpatial contract.

## Runtime state

Each tracked chunk separates four concerns:

- desired intent plus its current `DemandRank`;
- observed availability: `Absent` or `Resident`;
- current operation: idle, queued, issued, or provider-started load/unload work;
- an optional blocking load/unload failure while current availability does not satisfy current intent.

Issued operations own the complete issued request. Request lookup state is only a correlation index and does not duplicate request identity/kind in parallel record fields.

Availability changes only on successful provider completion:

- load queue/request/start leaves availability `Absent`;
- successful load completion changes availability to `Resident`;
- unload queue/request/start leaves availability `Resident`;
- successful unload completion changes availability to `Absent`;
- failed load remains `Absent`;
- failed unload remains `Resident`.

A failure is retained only while it blocks convergence to current intent. Intent reversal clears a failure when existing availability already satisfies the new target. Retry is explicit; RunenSpatial does not choose retry timing or backoff.

## Reversal

The provider contract is non-cancelling once a request is issued:

- if load becomes undesired after issuance, allow it to finish and queue unload after successful completion;
- if unload becomes desired after issuance, allow it to finish and queue load after successful completion;
- reverse unissued queued work directly without provider churn.

Neutral records are removed once they are undesired, absent, idle, have no blocking failure, and own no pending request. Runtime records therefore represent live framework state rather than exploration history.

## Request identity

Request IDs are opaque, nonzero, monotonically generated identities. They are never silently saturated or reused.

A tick reserves the complete request-ID batch before changing queued operations into issued operations. If the remaining ID domain cannot satisfy that complete batch, it issues no new requests and reports `request_id_exhausted` in the tick output. Successful demand changes from that tick remain committed and visible.

## Remaining contract work

Long-running state is not yet fully bounded when records are legitimately retained by residency or stalled providers. The next lifecycle step must define explicit tracked-record/pending-operation capacity, deterministic admission/backpressure behavior, and diagnostics without moving backend IO, payload ownership, or host retry/degradation policy into this package.
