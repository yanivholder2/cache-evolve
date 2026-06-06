# Variant v002 Insight

## Helped

- (none) — the ghost-history re-admission moved no dimension in the intended direction; metadata-size's summary metric (`Node` size) stayed put because ghost data lives in separate `ghost_queue`/`ghost_set` structures, not in `Node`.

## Hurt

- hit-rate: Re-admitting every recently-evicted key straight into the **protected** segment over-promotes. A single post-eviction re-reference is treated as already-hot, so `prot_len` overflows and `demote_protected_lru` pushes genuinely hot protected entries back to probation where they get evicted next — net hit-rate fell *below* the pure-SLRU 0.7241 baseline it was trying to beat.
- per-op-time: Each `put` now runs `ghost_admit` (a `HashMap::remove`) on the new-key path and each eviction runs `ghost_remember` (a `HashMap::insert` plus `VecDeque` push and FIFO-bound trimming), adding hashing and allocation work to the hot put path; the larger `LruCache` struct (extra queue/set/cap fields) also worsens locality, so even the ghost-untouched `get` p50 rose.

## Takeaway

LIRS-style reuse-distance re-admission is real, but routing ghost hits *unconditionally into protected* overshoots on this trace: it lets transient re-references evict truly hot keys, so the policy loses hit-rate instead of gaining it while paying ghost-bookkeeping latency on every put. The reuse-distance signal needs a gentler admission (e.g. re-admit into probation, or gate on repeated ghost hits) rather than instant promotion.
