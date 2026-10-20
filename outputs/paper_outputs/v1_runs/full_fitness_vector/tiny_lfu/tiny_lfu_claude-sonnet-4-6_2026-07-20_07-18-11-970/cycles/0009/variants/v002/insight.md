# Variant v002 Insight

## Helped

- hit-rate: The protected segment gave genuinely hot keys a structural refuge from eviction pressure, yielding a marginal improvement over the parent — though the gain is nearly negligible because the existing admission gate already performed most of that filtering via frequency comparison.

## Hurt

- per-op-time: Adding the protected segment doubled put p50 and p99 latency because every `put` on a full cache must now traverse dual linked lists (probationary and potentially protected) and may trigger a demotion write-back through `promote_to_protected`, whereas the parent's put path only walked the single probationary list.

## Takeaway

Layering a 20%-capacity protected LRU segment on top of an already-tuned amortised admission gate produced almost no hit-rate gain: the frequency-based gate already shields hot keys effectively enough that a structural protection zone adds little on this workload, while the dual-list bookkeeping meaningfully taxes the put path. The `prot_flags` bitset successfully kept `Node<u64,u64>` at 48 bytes as intended, but the `TinyLfuCache` struct itself grew by 72 bytes for the five new fields, confirming that SLRU overhead in this design is in control-plane metadata and per-operation branching rather than per-node memory.
