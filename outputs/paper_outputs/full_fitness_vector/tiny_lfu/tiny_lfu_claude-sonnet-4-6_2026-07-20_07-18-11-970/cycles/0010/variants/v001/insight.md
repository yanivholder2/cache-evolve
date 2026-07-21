# Variant v001 Insight

## Helped

- per-op-time: Removing `promote_to_protected` from the probationary-key update branch in `put` eliminated the cross-chain linked-list relinking, cutting put p50 in half and put p99 by a third; get p99 also dropped, likely because write-path demotion cascades no longer race with the read hot path over the same node pointers.

## Hurt

_(no dimension regressed)_

## Takeaway

Restricting SLRU promotion to the `get` path eliminated the costliest operation on the put write path — the cross-chain migration from probationary to protected — without touching hit rate at all, confirming that write events carry no additional signal beyond what the frequency sketch already records. For this workload, the admission gate's frequency comparison is sufficient to retain hot keys; allocating a protected slot on every write update wastes latency budget without improving eviction decisions.
