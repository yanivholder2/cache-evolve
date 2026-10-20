## Recurring wins

- Treating the stash as a scan-resistant probationary queue, ordered by proven reuse, is the only hit-rate lever that has paid off. Demotees go to the stash's MRU end (cycle 1) and fresh puts go to its LRU/Front end (cycle 2). The two gains stack, and this pair is the current best parent.
- Changes to independent parts of the get/put path (stash insert position, lookup order, stash-hit destination) combine cleanly. The cache state you get after combining them is what you would predict.
- The only latency gains you can trust come from skipping work: probing the tier that serves most hits first, or avoiding the remove→promote→demote→reinsert cascade. Credit a latency gain only when the code change explains it.
- Metadata-size has not moved in any variant so far. Placement and ordering changes are free on that dimension.

## Recurring failures

- Changing how items get promoted out of the stash, in either direction, has always cost hit-rate. MRU placement on stash→target (cycle 2) failed. So did a second-hit gate on stash→target and an ownership gate on target→probing (cycle 3). Default LRU-ordered Front promotion looks like a local optimum. The buckets are too small for any extra filter to beat plain recency.
- Placement tweaks on rare corner paths (such as balanced placement on stash hits) are safe but rarely trigger, so the change barely shows up.
- A fixed lookup order doesn't carry over from one lineage to another. Target-first won when stash hits were rare (cycle 1). It lost once fresh-at-Front placement routed first reuse through the stash (cycle 3). The best order depends on placement policy.
- Latency comparisons across runs are unreliable. Big shifts showed up on paths a diff never touched (most of all in cycle 1), and most put-path moves look like noise.
- Every get-tail reduction so far came from promoting less often, and each one gave back hit-rate. Cutting how often promotion happens is not a free latency lever.

## Open questions

- Can a promotion cascade be made cheaper without changing which items get promoted? Options include in-place swaps instead of remove+reinsert, or a cap on how deep a cascade goes.
- Can bucket scans be made cheaper, for example with per-slot fingerprints, at an acceptable metadata-size cost?
- Structural levers are still untested: stash capacity and associativity, and the eviction order inside target buckets. Would bigger stash buckets let admission filters work where 2-slot buckets couldn't?
- How do hits split between the target, probing and stash tiers under the current parent? It has never been measured directly, yet it decides lookup order and where cascade cost matters.
- Would balanced or spare-neighbour placement at initial put or target-hit time trigger often enough to matter?
- Would an adaptive or per-key lookup order beat a fixed one, or would the extra branches cost more than they save?

## Per-policy notes

- Stash (dominant capacity tier): its internal eviction order is the main hit-rate lever. Fresh entries go to Front (LRU end) and reused demotees go to Back (MRU end). As hit-rate rises, more traffic lands on the slow stash-hit→promote path, which slightly erodes get p50.
- Target buckets: these are tiny, so one MRU-placed promotion evicts residents that were already hit. Front placement on promotion works as an implicit admission filter and should stay. Eviction order inside target buckets hasn't been tested.
- Probing buckets: these hold the hottest items. Gating promotion into them (native-ownership gate) lost real reuse. The climb within each bucket already keeps probing selective.
- Lookup order: this is a latency-only knob with no effect on hit-rate. Pick it based on the measured hit split under the current placement policy, not from earlier cycles' results.
