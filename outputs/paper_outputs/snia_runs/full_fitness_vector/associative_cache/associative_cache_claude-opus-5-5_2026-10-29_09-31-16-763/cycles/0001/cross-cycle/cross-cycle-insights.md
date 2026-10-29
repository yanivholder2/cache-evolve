# Cross-Cycle Insights

_Projection of 1 completed cycle (0001). Patterns below are early signals, not yet confirmed across cycles._

## Recurring wins

- Scan-resistant admission raises hit rate. Two separate levers both worked in cycle 0001: inserting new items below the MRU position, and inserting them with a cleared reference bit. Either way, one-touch items get evicted first unless they are reused.
- The gain is largest with wide buckets, where each bucket acts most like a full LRU.
- Reusing existing per-item fields (e.g. `lfu_counter`) or changing only the insert position leaves metadata-size unchanged.

## Recurring failures

- Any added work on the miss/eviction path made put latency worse, especially the tail. This held for a mid-bucket insert, a per-item sweep and a windowed scan with aging. Assuming that "a memmove is a memmove" was wrong.
- Making the hit path cheaper barely changed get p50, because the cost is mostly the linear key lookup across the bucket.
- Comparing the policy name as a string on every op (`name() == "ClassicLRU"`) adds put-path overhead unrelated to the policy. Decide the policy once, at construction.

## Open questions

- Does the hit-rate harness actually run the overlaid bucket `evict_item` / ClassicLRU paths? A hit rate bit-identical to the baseline suggests it may not. Check this before judging any victim-selection idea.
- Do below-MRU insertion and a cleared reference bit stack, or do they give the same scan protection twice?
- Should insertion depth scale with bucket width? The gain was weak at the smaller bucket size.
- Can get latency be cut by speeding up the lookup itself (tag/fingerprint array, per-bucket key index) without growing metadata-size too much?

## Per-policy notes

- ClassicLRU: every change so far was made inside the bucket. Insertion position and hit recording paid off on hit rate. Frequency-aware victim selection is unproven, because it may never have run in the harness. Put latency gets worse with any extra eviction-path work, and get latency is dominated by the key scan.
