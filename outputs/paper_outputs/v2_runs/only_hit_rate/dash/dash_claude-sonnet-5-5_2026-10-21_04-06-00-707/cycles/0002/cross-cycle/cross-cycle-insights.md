## Recurring wins

- Changing where a promoted item lands (MRU placement on hit promotion, cycle 1) is the only lever that has improved hit-rate so far. It beat variants that moved more items.
- Stacking further placement or eligibility tweaks on that winner has not hurt it. Cycle 2 variants matched the parent exactly, so the MRU baseline is stable.

## Recurring failures

- Approaches that add item movement, such as bidirectional swaps, looser promotion eligibility, or extra promotions, give marginal or negative returns. The likely cause is demotion cascades toward stash and eviction.
- The placement-position family (insert, demotion, eligibility) produced no hit-rate change in cycle 2. On workload-a at this cache size it looks exhausted, or its overlays were dead edits.
- Cold-end demotion, justified by the cascade-eviction theory, got no support.

## Open questions

- Do the overlays change the executed code path at all? Equal hit-rate across all cycle 2 variants could mean no-ops.
- Is workload-a or the cache size insensitive to placement? Other traces or sizes might separate the variants.
- Do the variants differ on per-op-time or metadata size? Only hit-rate has been measured, so there are no cost claims and ties cannot be broken.
- Which untested levers matter: eviction victim selection, bucket and probing structure, or admission?

## Per-policy notes

- DashSegment (promotion in `get` and `promotion_get`): MRU insertion on hit promotion is the incumbent, and the lineage branches from cycle/0001/v001. Eligibility, insert-path and demotion-path variants add nothing on top of it on workload-a.
- Add per-op-time and metadata-size to the fitness vector, and test on a second workload, before trusting any further placement result.
