# Streaming Availability Lifecycle

This document explains the responsibility of `runen-spatial-streaming`. Exact public signatures and transition tests are owned by the package source, rustdoc, and tests.

## Ownership

The streaming package coordinates content-agnostic availability work: projection of current spatial demand onto tracked runtime state, bounded load/unload request issuance, provider correlation, deterministic ordering and reversal, blocking failure state, working-set admission, and operational pressure diagnostics.

The backend owns actual IO and payload/resource creation. The host owns product semantics, payload caches, gameplay/ECS activation, rendering, retry timing/backoff, CPU/frame scheduling, and application degradation policy.

A provider result is paired to framework work by its nonzero request ID, world-qualified `ChunkId`, and load/unload operation. Payload/resource transfer is not part of the RunenSpatial contract.

## Demand and runtime state

`SpatialDemandPlanner` owns the complete bounded desired-intent set and `DemandRank` ordering. Streaming does not materialize a runtime record merely because an absent chunk is desired.

A tracked chunk record exists only while it carries runtime truth that must survive independently of the planner:

- observed availability: `Absent` or `Resident`;
- an issued or provider-started load/unload operation;
- an optional blocking load/unload failure while current availability does not satisfy current intent.

For an already tracked chunk, desired intent and its current `DemandRank` are projected into the record so reversal and unload ordering remain deterministic. They are not a second desired-set authority.

There is no stored queued operation state. Pending work is derived:

- an effective desired chunk with no tracked runtime record is pending load work;
- a resident, undesired, idle chunk with no blocking failure is pending unload work.

An `Absent + Idle + no blocking failure` record is neutral and is removed even when the planner still desires that chunk. The planner retains the intent and may admit it again on a later tick.

Availability changes only on successful provider completion:

- load request/start leaves availability `Absent`;
- successful load completion changes availability to `Resident`;
- unload request/start leaves availability `Resident`;
- successful unload completion changes availability to `Absent`;
- failed load remains `Absent`;
- failed unload remains `Resident`.

A failure is retained only while it blocks convergence to current intent. Intent reversal clears a failure when existing availability already satisfies the new target. Retry is explicit; RunenSpatial does not choose retry timing or backoff. Clearing a blocking load failure does not preserve a waiting record: the neutral absent record is pruned and the planner remains the admission authority.

## Reversal

The provider contract is non-cancelling once a request is issued:

- if load becomes undesired after issuance, allow it to finish; successful completion leaves a resident undesired record that becomes pending unload work;
- if unload becomes desired after issuance, allow it to finish; successful completion emits the unload result, removes the now-neutral absent record, and the retained planner intent becomes pending load work on a later tick;
- provider failure preserves the observed availability and blocks only when the current target remains unmet.

No provider request is cancelled or discarded merely to satisfy working-set pressure.

## Capacity and pressure

`StreamingCapacity` is immutable controller configuration with three independent bounds:

- maximum tracked runtime records;
- maximum in-flight load requests;
- maximum in-flight unload requests.

These limits are separate from `StreamingBudgets`, which bound only how many load and unload requests may be issued per tick. Zero is a valid explicit capacity or budget policy; the core does not silently repair it.

A full tracked working set never evicts resident, active-request, or blocking-failure state. Instead, new load admission is deferred while the desired chunk remains in the planner. When records become neutral and are pruned, the highest-ranked still-effective missing demand can be admitted without source resubmission.

Load and unload concurrency are bounded independently so load saturation cannot consume the capacity needed to retire resident chunks. Unload work is deterministically ordered by retained rank and then `ChunkId`.

`StreamingPressureDiagnostics` reports distinct operational facts only:

- tracked record count and its configured capacity;
- `deferred_loads`, the effective desired chunks that have no runtime record after the tick;
- current in-flight load and unload counts with their independent capacities;
- `remaining_unloads`, the resident undesired idle chunks not blocked by failure after the tick.

These diagnostics are separate from demand-planner pressure. RunenSpatial does not own provider queue depth, task timing, frame budget, payload cache pressure, or application degradation metrics.

## Request identity and events

Request IDs are opaque, nonzero, monotonically generated identities. They are never silently saturated or reused.

A tick selects its capacity- and budget-bounded request batch, then reserves the complete request-ID batch before materializing any new load records or changing operations to issued state. If the remaining ID domain cannot satisfy that complete batch, it issues no new requests and reports `request_id_exhausted`. Successful demand changes from that tick remain committed and visible.

Every retained `WorldStreamingEvent` is correlated to an issued request and therefore carries a non-optional `StreamRequestId`. Request lookup state is only a correlation index; the active operation owns the complete issued request.
