# Cycle 0008 Insight

## Cohort outcome

The cycle tried two hasher/pre-sizing follow-ups on the tagged index with split ghost map (a multiply-xorshift hasher in v001, and a tight capacity pre-size plus an avalanche finalizer in v002). It also tried a smaller small-queue ratio, 3% instead of 5% (v003). The only Winner with a clear signal is v003, which gave a tiny hit-rate gain. v001 and v002 hold hit rate flat and trade latency, so their wins are weak and probably noise-assisted.

## Patterns

- Hasher work is exhausted. A cheaper hasher (v001) made everything slightly slower. A stronger finalizer with tight sizing (v002) fixed the `put` tail but gave back much of the `get` gain. Together they show a hash-quality versus hash-cost tradeoff with no free win, and hashing is no longer the main bottleneck.
- Tail and median trade off in v002. Tight sizing and mixing remove cold-insert outliers, while the added finalizer and fuller table raise the median. The two effects were bundled again, so their split is still unknown.
- The small-ratio lever is saturating. The step from 5% to 3% gave a much smaller hit-rate gain than the earlier steps, and the latency shifts in v003 have no mechanism, so they are noise.
- Hit-rate-neutral latency tweaks now produce mixed results. Variants selected as Winners without a mechanism-backed gain are increasingly likely to be noise.
- Evidence is still one trace at one size.

## Open questions

- Does the 3% small ratio hold up on other traces and sizes, or does it hurt workloads with more mid-reuse objects?
- Is a lighter finalizer or modest slack enough to keep the `get` gain and still fix the `put` tail?
- Do the earlier latency gains (tagged index, split ghost map, Fx hasher) survive repeated runs, now that the base is more stable?

## Continuation

Stop hasher, finalizer, and small-ratio sweeps. Pick one hasher config (the Cycle 0007 Fx hasher with moderate slack, or at most a light finalizer) and keep it. Keep 3% only if it holds on other traces. The next cycle should move to hit-rate ideas that change promotion or ghost policy without loosening main admission, and validate on multiple traces with repeated runs, including a heap footprint measurement.
