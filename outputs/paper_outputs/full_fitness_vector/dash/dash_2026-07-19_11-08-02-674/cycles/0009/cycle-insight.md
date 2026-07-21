# Cycle 0009 Insight

## Cohort outcome

This cycle probed three targeted changes: CLOCK_BIT sparsification at freq≥3 on the clock base (v001), max-frequency promotion-candidate selection replacing MRU-position (v002), and position-0 eviction bundled with the same freq≥3 sparsification (v003). All three variants were named winners by elimination rather than dominance — v001 recovered a genuine hit-rate lift at the cost of elevated put-path tail latency, while v002 and v003 both regressed on hit-rate with v003 also regressing on latency.

## Patterns

- **Workload-a's hot set is concentrated at freq 1–2, making freq≥3 a destructive clock threshold.** Both v001 and v003 applied the same sparsification boundary; v001 survived because LFU-minimum eviction still gave cold victims to expel, while v003's additional position-0 switch removed that fallback entirely, compounding the regression. The principle is confirmed: freq≥3 leaves too much of the working set permanently cold for this trace.
- **Sparsifying CLOCK_BIT creates a latency tension in hot buckets.** v001 successfully restored hit-rate discrimination, but the same boundary that makes CLOCK_BIT sparse globally makes it *dense* within the hottest buckets — items there accumulate three hits quickly, causing `clock_evict_prepare` to rotate more protected entries before finding a cold victim. The threshold trades uniform saturation for hot-bucket saturation, shifting but not eliminating the rotation overhead.
- **MRU-position promotion is already a reliable frequency proxy on Zipfian workloads.** v002 confirmed that substituting a max-lfu_counter scan for the MRU-back position erodes hit-rate marginally. On a stationary Zipfian trace, the most recently accessed item at the back of the bucket is consistently the item with the highest access evidence; cumulative frequency occasionally diverges for cooling-but-historically-hot items, and those divergences are the wrong direction to promote.
- **Position-0 eviction is not a safe drop-in for LFU-minimum when CLOCK_BIT is sparse.** v003's double change — O(1) position-0 eviction plus freq≥3 threshold — produced the worst outcome: the sparser threshold evicted more freq 1–2 items as cold, the position-0 rule offered no frequency-based recovery, and promotion churn under increased miss pressure inflated both hit-rate and tail-latency metrics simultaneously.

## Open questions

- Does freq≥2 as the CLOCK_BIT threshold recover the hit-rate gain of v001 while reducing hot-bucket rotation depth enough to contain the latency regression?
- Can a consecutive-rotation cap (abort `clock_evict_prepare` after k cleared bits and fall back to position-0) bound the tail latency without losing the discriminating power the sparser threshold provides?
- Is position-0 eviction worth retesting in isolation — keeping CLOCK_BIT at freq≥1 or freq≥2 — to assess the O(n) scan's contribution to latency independently of the threshold choice?

## Continuation

The immediate next experiment is freq≥2 as the sole change: apply it to the clock base exactly as v001 applied freq≥3, with no position-0 substitution. v001's insight and v003's double-regression together bracket the answer — the working threshold almost certainly lies at freq≥2 — and isolating the variable will confirm whether the hit-rate gain survives at a lower rotation depth. If the latency regression persists at freq≥2, the rotation-cap idea becomes the natural follow-on. Max-frequency promotion (v002's direction) should be deprioritized; MRU-position is effective on this trace. Position-0 eviction should not be bundled with threshold changes again; test it independently only after the threshold question is settled.
