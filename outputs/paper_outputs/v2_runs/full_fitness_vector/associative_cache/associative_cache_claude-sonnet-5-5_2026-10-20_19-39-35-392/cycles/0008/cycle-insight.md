# Cycle 0008 Insight

## Cohort outcome

Variants tried a one-bit "seen once" gate on transpose promotion (v001), a fused single-pass put scan with swap insertion (v002), and an adaptive len/4 vs len/2 insertion point driven by a global hit/miss counter, built on baseline (v003). All three were Winners, but only v003 shows a real gain: better hit rate and clearly cheaper puts, at a slightly higher median get.

## Patterns

- The Winner-versus-evidence mismatch continues, a sixth straight cycle. v001 lost on hit rate and on both get and put latency. v002 got slower on both paths, and its hit-rate gain came from an unintended change in behavior, not from design.
- A second reuse filter on top of transpose promotion does not pay. This matches cycle 0007: transpose is already a gradual frequency filter, and extra gating only delays hot entries and adds per-comparison cost.
- Micro-fusing the put path is not a safe latency win. It slowed put and get, probably through code layout or extra per-slot work, and it quietly changed eviction and placement. Placement in the bucket seems to matter more than scan speed.
- v003 again shows that mid-bucket swap insertion plus transpose promotion beats LRU, now reproduced from baseline a third time. Its adaptive counter fit in existing padding, so metadata did not grow. Its insight cannot separate the adaptive switch from the fixed mid-insertion mechanics.
- Get latency still has no real gain. Every per-hit addition (a flag, a counter) taxed the hot path.
- Only v001 and v003 touched metadata, and neither grew it.

## Open questions

- How much of v003's gain comes from the adaptive switch, and how much from swap insertion at a fixed position? A fixed-position control is missing.
- Does v002's unintended placement change, such as first-empty-slot fill or overwriting the last victim, help hit rate, and can it be adopted deliberately?
- Can adaptivity be driven from the put path alone, to keep the get hot path untouched?
- Do any results hold on a second workload, and are the Winner statuses beyond noise?

## Continuation

Drop reuse-flag gating on promotion and put-scan fusion; both are exhausted. Pursue adaptive insertion from v003 as the lead, and add a fixed-position control to isolate it. Prefer signals updated on the put-miss path, since per-get work costs median get time. Also test v002's empty-slot fill and victim placement as a deliberate policy. Keep discounting Winners whose own insight shows a loss, and add a second workload and a noise check.
