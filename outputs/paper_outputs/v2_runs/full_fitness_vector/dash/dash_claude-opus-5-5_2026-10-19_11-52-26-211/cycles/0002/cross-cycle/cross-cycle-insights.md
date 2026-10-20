## Recurring wins

- Treating the stash as a probationary queue ordered by proven reuse is the main hit-rate lever. It worked twice: demotees went to the stash MRU end (cycle 1) and fresh puts went to the stash LRU end (cycle 2). The two gains added up.
- Checking the target, then probing, then the stash is a free get-path gain with a clear mechanism, because most hits come from normal buckets. It is settled, so don't revisit it.
- Single-site changes along the promotion/insert path compose cleanly. Stash insert position, lookup order and promotion destination are independent, and combining them reproduced the expected cache state.
- No variant so far has moved metadata-size. Placement and order changes come free on that dimension.

## Recurring failures

- Tweaks to placement when a stash item is promoted to the target. Balanced placement (cycle 1) rarely triggered. MRU placement (cycle 2) lowered hit-rate slightly and lengthened the get tail. In tiny target buckets, hot placement on the way up evicts residents that already proved themselves, and it skips the climb inside the bucket. Placing promotions at the Front works as a useful admission filter.
- "Winner" labels have included neutral and negative variants. Judge results by real metric movement, not by the label.
- Latency deltas compared against the baseline have repeatedly shown up on code paths the diff never touched, so they look like environment or run noise. Only credit per-op-time changes that have a code-level mechanism, and measure the baseline in the same run.

## Open questions

- Has fresh-put-at-stash-LRU been confirmed on top of the combined parent (lookup order + Back-demotion)? It is expected to carry over, but that hasn't been verified.
- Higher hit-rate pushes more traffic onto the slowest hit path: stash lookup → promote_to_target → possible cascade. Can that path be made cheaper to win back get p50?
- Would stricter admission into probing (for example, requiring a second target hit) protect the hottest tier?
- How do hits actually split between the stash and the target/probing buckets? This is still unmeasured.
- Would balanced placement at initial put, or on target-hit promotion into a neighbour's spare space, trigger often enough to matter?
- Front insertion seems to add a small put-p50 cost from shifting the bucket. Is that real or noise?

## Per-policy notes

- Stash (most of the capacity): its eviction order dominates hit-rate. It rewards scan resistance: demotees go to the MRU end, new arrivals to the LRU end. It is also the costliest place to look up, so check it last.
- Target buckets: they are tiny, so placement there is very sensitive. Keep incoming promotions at the Front/LRU end so residents with hits stay safe.
- Probing buckets: they serve a large share of hits. Climbing within the bucket keeps them selective, and stricter admission is the next lever to try.
- Get path vs. put path: hit-rate gains tend to cost some get-path latency through the extra stash→target promotions. Front inserts may add a small put-path cost.
