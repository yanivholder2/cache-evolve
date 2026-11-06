# Cross-cycle insights (cycles 0001–0010)

## Recurring wins

- **Retention of proven items drives hit rate.** The changes that kept proven or long-gap reusers alive longer raised hit rate every time they were tested: the two-hit promotion gate (0001), MRU demotion into stash with or without the gate (0002, 0003), probing overflow and the probing room gate (0003, 0004). The current base keeps all of them.
- **Hashing cost is a first-order per-op term.** Calling the hasher fewer times did nothing (0007). Making each call cheaper with an fmix64-style hasher improved every percentile (0008). The cheap hasher also shrank costs that came from re-hashing elsewhere.
- **A dense, separate stash presence mask lowers `get` p50.** It is the only change that has moved the median (0007). Upkeep can be made O(1) as long as the read-side word is left alone (0009 v001). A k=2 Bloom mapping in the same u64 sped up both `get` and `put` at no cost (0009 v002), which suggests `put`'s pre-insert lookup goes through the same reject test.
- **The `get` tail is on the stash-hit promotion chain.** This was confirmed by direct intervention. A fused rotate (0006) and a cold, out-of-line promotion chain (0010) both cut `get` p99.
- **Cheap per-item state** (reusing the spare `lfu_counter` field) adds no metadata size.

## Recurring failures

- **Skipping or deferring a cascade moves its cost; it does not remove it.** The cost lands on `put` through fuller targets, or on a later `get` through foreign residents in probing or forced promotions (0004–0007). Unbounded deferral jams stash. A limited deferral helps the large cache but hurts the small one.
- **Deferral gates that read stash-bucket-local state** do not separate harmful deferrals from harmless ones. Both fill level (0006) and the marked-LRU signal (0007) failed. The stash is almost always full at steady state.
- **Common-path levers ruled out for `get` p50:** lookup order with stash last (0003, made things worse), scan direction (0005), skipping the near-MRU reorder (0006), and repeated segment hashing (0007).
- **Weakening the read-side mask** gives back the p50 gain. Shrunken counting masks raise false positives, and in-bucket fingerprint byte scans are slower than one AND (0008).
- **Collision-word O(1) clearing under k=2** is a net loss: collisions are frequent and the rebuild fires on a steady schedule (0010).
- **Stash victim selection is a dead lever.** Narrow (0009) and whole-bucket (0010) windows both left hit rate on the same plateau, and the wide window evicts young entrants.
- **Demotion placement based on reuse within a single normal-tier stay** lost hit rate everywhere (0005).
- **The ungated MRU + overflow line** never got back to baseline latency (0003, 0004).
- **Any extra work or layout change on the stash insert/eviction path shows up first in `put` p99.** This held in nearly every cycle: mask refresh, collide words, cold shared helpers, mid-bucket evictions.

## Open questions

- Does splitting the promotion path keep 0010 v002's `get` p99 win without the `put` p99 loss? The split would keep a cold out-of-line entry for `get` and give `put`'s full-bucket insert/demotion its own inlined shift. Is the `rotate` → memmove call the real culprit compared with an unrolled loop?
- What sets the hit-rate plateau now that victim choice is ruled out? Candidates are the stash vs. normal-tier capacity split, the promotion gate threshold, and the admission and demotion rules.
- Is k=3 worth trying on the mask, or does bucket fullness erase the gain? Is there a cheap O(1) clear that works with k≥2 and does not put a second word on the hot path?
- How much p50 is left in segment-level selection or in the remaining mask false positives?
- Would the cheap hasher change the trade-offs on the old deferral line?
- Tail-only deltas rest on single samples. Unexplained p99 drift on untouched paths (likely noise or codegen/inlining) has recurred often enough that these results need reruns.

## Per-policy notes

- **Dash (the only policy studied so far):** the current lineage is the two-hit gate, MRU demotion, probing overflow, the probing room gate, the fast hasher, and the dense k=2 stash mask (0009 v002), plus the cold `get` promotion path (0010 v002) as a candidate.
- **Dash, hm_0 at the large size** is consistently the most sensitive cell. Retention gains and losses show up there first and largest.
- **Dash, cache size decides whether stash residency helps.** Longer stash residency helps the large cache, where proven residents stay in place, and hurts the small cache, where pending items take up slots.
- **Dash, stash hits are a large share of all hits.** Stash-first lookup is correct, and stash-path costs dominate both `get` percentiles.
- **Dash, code sharing between paths:** `get` promotion and `put` full-bucket insert/demotion share the shift/cascade code, so cost or layout changes on one show up on the other.
