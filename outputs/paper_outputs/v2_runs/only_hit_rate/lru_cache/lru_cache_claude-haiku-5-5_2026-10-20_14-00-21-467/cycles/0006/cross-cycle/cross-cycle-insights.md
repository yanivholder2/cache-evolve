## Recurring wins

- Eviction-neutral representation changes (sentinel index, relink skip on MRU hit, lookup collapse, hasher swap, index pre-sizing) have consistently held hit-rate equal to their parent on workload-a at size 2048. The behaviour-preservation property is reliable; no size or per-op gain has been measured for any of them.

## Recurring failures

- Changes that add structure to the recency list have not paid off on hit-rate: the probation/protected split (v001 in 0005, v003 in 0004 with no clear direction) and the second-chance referenced-bit eviction (0005) both lowered or failed to show a gain.
- Second-hit promotion (0006 v002) lowered hit-rate on the same trace.
- Tail insertion of new keys has moved hit-rate in both directions: a gain in 0005 (v003), then losses in both stacked forms in 0006 (v001, v002). It is not a reliable win yet.
- Ghost-based admission filters (0002 v002, 0003 v002) reported no hit-rate movement, and no cycle showed that the filter engaged on the trace. The result may be a non-test rather than a real null.
- Two-of-three-variant failures in 0001 and one in 0004 were infrastructure rejections (session limit, missing insight file). These say nothing about their strategies and should not be counted as failures of the approach.
- Named Winners with no measured gain or with a hit-rate dip (0003 v002, 0005 v002, 0006 v001) recur. Selection is not tracking the measured dimensions.

## Open questions

- Per-op time and metadata size have never been recorded on any lineage across six cycles. Until they are added to the fitness vector, no size or speed claim (including H01's and H03's) can be judged.
- What is the selection criterion for Winners? Several were chosen without a hit-rate gain.
- Does tail insertion help against the current parent, or did the 0005 gain depend on the parent it was measured against? The 0005 and 0006 results cannot be reconciled from the insights.
- Does the H03 label refer to one mechanism? It has been mapped to a per-op check, a pre-sized index, a segmentation change, and a hasher swap. Fix one definition before lineages cite it.
- Does any result generalise beyond workload-a at size 2048? All hit-rate evidence comes from one trace and one size, so scan-resistance claims are untested.
- Lineage for the 0002 Winner's hit-rate gain is ambiguous; the parent it was measured against needs pinning down.

## Per-policy notes

- LRU (all cycles so far): every variant is LRU-based. Strict recency ordering appears to matter, since every change that altered eviction order either lowered hit-rate or showed no clear gain. Representation-only changes are safe but unproven as improvements.
- LRU with tail insertion (0005, 0006): the only eviction-order change with any positive reading, but conflicting across cycles. Treat as unresolved.
- LRU with segmentation / second-chance / second-hit promotion (0004, 0005, 0006): no evidence of gain on hit-rate; treat as dead as written unless a parameter change (protected cap, demotion rule) is tested on its own.
- Admission (ghost filter, 0002–0003): engagement on the trace is unverified; no hit-rate effect recorded.
- No non-LRU policy has been evaluated.
