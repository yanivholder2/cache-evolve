# Cycle 0008 Insight

## Cohort outcome

All three variants built on cycle-0007 v002, which adds the rescue. v001 added a 1-bit CLOCK reference for picking victims in normal buckets. v002 gave ghost credit from the far zone only when the target had a free slot. v003 sent gated stash promotions into free probing slots when the target was full. The Winner is v001. Its gains were marginal, possibly noise, and came only at 128K on both traces. Its target cell, `web_0` 16K, stayed flat once again. v002 moved gains around without lifting the mean: `web_0` went slightly up and `hm_0` 128K slipped. v003 left `web_0` 128K exactly where the parent had it and lost a little on `hm_0` 128K.

## Patterns

- `web_0` 16K survived a victim-choice lever too. In saturated 2-slot buckets, a reference bit almost always picks the same victim LRU would. Move-to-MRU on a hit already encodes recency, and items enter and leave too fast to build up a second signal. Changing victim selection is not enough unless it brings information that LRU order does not already carry.
- `hm_0` 128K is the cohort's sensitive cell. Both losing variants hurt it, each in a different way. v002 denied credit to long-tail returns that had been displacing residents profitably. v003 used up probing headroom early. The lineage rule that "lost-slot items must not outrank residents" does not hold universally: on `hm_0` 128K, displacing residents sometimes pays.
- Levers that only use free slots are exhausted. Once the rescue exists, sending other items into free slots just changes the order in which the same headroom is spent (v003). Conditioning on occupancy reduces to a crude credit drop, because the bypass already handles empty targets (v002).
- The `web_0` 128K giveback is still unexplained. Stash→full-target promotions are now ruled out as its source, as are foreign retries (c7) and ghost reach.
- Per-op-time and metadata-size are still unmeasured. That is now eight cycles in a row.

## Open questions

- If items are not moved to MRU on a hit, so the reference bit becomes the only recency signal, does victim choice at 16K finally diverge from LRU?
- Could ghost credit or victim choice depend on the resident that would be displaced (its recency or hit count) instead of on whether a slot is free?
- Does reuse that survives across stash stays, or frequency derived from the ghost, provide a victim signal that LRU order lacks?

## Continuation

Build on v001. Drop free-slot routing and occupancy-conditioned ghost credit. For `web_0` 16K, try either removing move-to-MRU on hits in normal buckets or a victim signal that carries history across stash stays. Base admission and credit decisions on how valuable the resident being displaced is, since `hm_0` 128K shows that displacement can pay. Measure per-op-time and metadata-size.
