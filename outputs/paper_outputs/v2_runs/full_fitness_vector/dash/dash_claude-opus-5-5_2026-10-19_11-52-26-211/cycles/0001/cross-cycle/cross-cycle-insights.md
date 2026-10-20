## Recurring wins

- Only one cycle so far, so nothing has recurred yet. The points below are early signals to confirm, not established patterns.
- Changing the eviction order inside the stash is the only lever that has moved hit-rate. The stash holds most of the capacity, so how demoted items are ordered there matters most.
- Reordering lookups so normal buckets (target, then probing) are checked before the stash scan cut get cost. Hit-rate and metadata-size stayed the same. Most hits come from the normal buckets, so the stash scan is the main lookup cost.
- Single, targeted edits to separate parts of `promotion_get` (stash insert position, stash-hit destination, lookup order) have not conflicted with each other or changed metadata-size. That makes them safe to combine.

## Recurring failures

- Placement tweaks that only fire on rare paths do almost nothing. Balanced placement on stash-hit promotion was one example. The cohort showed it was safe but had negligible impact.
- Comparing per-op time to a baseline measured in a different run is unreliable. Every variant showed large latency drops, even on paths its code change never touched, which points to a shift in the run environment. Only credit a latency change when the code gives a clear reason for it.

## Open questions

- How do hits split between the stash and the normal buckets? Demoted items do seem to get reused, yet most hits are served outside the stash. This split has not been measured.
- Promotion placement is still untested: should `get` insert at the back or the front, and should target-hit promotions use spare space in a neighbouring bucket?
- Does balanced placement at initial `put` time trigger often enough to matter, unlike the stash-hit case?
- Are there further refinements to stash eviction order beyond demoting items to the back?
- Re-measure baseline latency in the same run as the candidates before trusting any per-op-time claim.

## Per-policy notes

- Dash (target/probing buckets plus stash): the recommended next parent is back-of-stash demotion combined with target→probing→stash lookup order. These two wins are independent of each other. Within this policy, stash eviction order is the hit-rate lever and lookup order is the latency lever. Stop working on placement along the stash-hit path.
