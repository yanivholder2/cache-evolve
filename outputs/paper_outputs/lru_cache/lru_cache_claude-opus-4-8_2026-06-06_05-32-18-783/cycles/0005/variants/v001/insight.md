# Variant v001 Insight

## Helped

- metadata-size: Housing the entire ghost history (`ghost_queue`, `ghost_set`, `ghost_cap`) in the `LruCache` struct and never in `Node` kept `Node<u64,u64>` in the same 24-byte class as the parent, exactly as the fold intended — `push_front` routes through the existing `slot`/`link`/`set_*_link` helpers and the packed `PROT_MASK` flag, so direct-to-protected admission cost no extra per-node bytes.

## Hurt

- per-op-time: The ghost machinery is paid on every new-key `put` unconditionally — a `ghost_admit` HashMap probe on each insert plus a `ghost_remember` insert, VecDeque trim, and key clone on each eviction — and the cache-level struct grew with the extra HashMap and deque, so both put and get latency rose well above the parent's lean SLRU path.
- hit-rate: The ported gated ghost admission moved recall essentially not at all on workload-a; the protected fast-track it grants fires too rarely (or on keys that would have survived anyway) at this trace and size to change which entries are resident.

## Takeaway

Folding v002's ghost admission onto v001's 24-byte base held the node footprint but failed its core premise: the hit-rate delta the fold was meant to capture was vanishingly small to begin with, so the merge bought a near-doubled per-op latency for no measurable recall gain. The ghost bookkeeping's cost is structural and per-operation while its benefit is workload-contingent and absent here, making this fold a net regression on v001's base rather than the additive win the cross-cycle insight predicted.
