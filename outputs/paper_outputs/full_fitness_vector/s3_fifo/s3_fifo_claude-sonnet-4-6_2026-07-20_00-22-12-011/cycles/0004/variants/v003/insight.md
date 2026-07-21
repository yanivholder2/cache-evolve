# Variant v003 Insight

## Helped

(none)

## Hurt

- hit-rate: Decrementing frequency by 2 strips one protection round from freq=2 and freq=3 entries in `evict_main()`, causing moderately-warm entries to be displaced from main in fewer passes and allowing cold entrants to push them out before they can accumulate enough reaccesses to survive.
- per-op-time: Put p50 and p99 both worsened despite the shorter per-entry scan: faster frequency drainage causes more entries to cycle through the main→ghost→re-admission path, increasing the number of `evict_main()` invocations triggered by ghost hits in `put()` and offsetting any per-scan cost savings.

## Takeaway

Step=2 frequency decrement shortens the worst-case scan length per entry, but on this Zipfian workload the mechanism backfires: warm entries (freq=2, freq=3) lose a protection round they needed, churning through main into ghost more rapidly. The resulting increase in ghost-hit-driven `evict_main()` calls raises per-put overhead rather than lowering it, and the accelerated displacement of warm entries clips hit rate. Reducing eviction scan length by draining frequency faster is not equivalent to reducing total eviction work — the two effects oppose each other here.
