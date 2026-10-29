# Variant v003 Insight

## Helped

- hit-rate (hm_0 @ 16K): Raising probation to about 1/3 of a 16K cache gives a new key more time to get its second hit before eviction. On hm_0 more keys reached protected, which supports the hypothesis that probation at 20% is too small at this size for this trace.

## Hurt

- hit-rate (web_0 @ 16K): The same split slightly lowered web_0 at 16K. Taking space from protected cost web_0 more in lost reuse of hot keys than it gained from the longer probation window. So "probation is undersized at 16K" holds for hm_0 but not for web_0.
- per-op-time: Get p50 and both put percentiles rose clearly, while get p99 fell sharply. `get`, `put`, `on_hit`, `unlink`, `push_back` and the node layout are byte-for-byte identical to the parent. The only code change is a branch in `try_new` that runs once. The one real way this change could affect timing is in the benchmark itself, if it runs at a capacity of 32K or less. With a smaller protected cap, more hits land on probation entries. More of those promotions then overflow protected and trigger a second unlink/push_back to demote an entry. That extra work is small and cannot explain p50 and p99 moving in opposite directions by this much. Most of this movement is probably run-to-run noise, the same noise suspected in v002's get-p99 rise.

## Takeaway

A static 1/3-probation split at 16K or less mostly helps hm_0 and slightly hurts web_0, so the best probation share at the small size depends on the trace. No single fixed fraction captures it. As predicted, the 131K results and node size are unchanged, so the net hit-rate change comes entirely from the 16K size. Since the hot path is unchanged, the large per-op-time swing in both directions shows the timing benchmark is too noisy to trust between variants without repeated runs.
