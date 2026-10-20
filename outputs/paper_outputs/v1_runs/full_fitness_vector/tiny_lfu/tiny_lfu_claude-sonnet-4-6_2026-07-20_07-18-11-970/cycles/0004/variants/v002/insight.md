# Variant v002 Insight

## Helped

- `hit-rate`: Even at 1-in-8 frequency, the best-of-2 victim probe rescued enough frequency-blind evictions to marginally improve hit rate over the plain-LRU-head parent.
- `per-op-time`: The amortised conditional (`evict_counter & 7 == 0`) kept the put p50 at parity with get p50, eliminating the elevated put latency the parent exhibited without adding measurable overhead on the common (7-in-8) plain-head path.

## Hurt

- `metadata-size`: Adding `evict_counter: u32` to `TinyLfuCache` grew the struct from 152 to 160 bytes, adding 8 bytes of per-instance overhead — though the per-entry `Node` size is unaffected so the fitness summary was unchanged.

## Takeaway

Amortising best-of-2 victim selection to every eighth eviction preserves the parent's per-op-time profile while still landing a small positive hit-rate signal, confirming the benefit is real rather than an artifact of sampling noise. The gain is proportionally much smaller than what full best-of-2 on every eviction would deliver, which implies that the corrections matter most on the evictions this variant's 7-in-8 plain-LRU path skips — making a finer amortisation ratio (e.g. every 2nd or 4th) a natural next step to explore.
