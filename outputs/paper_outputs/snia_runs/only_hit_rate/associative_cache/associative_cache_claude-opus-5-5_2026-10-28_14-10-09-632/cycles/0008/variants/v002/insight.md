# Variant v002 Insight

## Helped

- hit-rate (web_0 @ 131072): The biggest gain was here, not in the target cell. The rationale assumed web_0 would get nothing from the veteran bit. In fact, web_0 has a group of blocks that get re-hit inside main, decay to counter 0, and then come back within about one more pass of the 768-slot main region. The one-shot grace visit lets them outlast fresh promotions that were reused only once, which they used to lose to when the hand reached them first. This also refutes the hypothesis's second failure mode: web_0's veterans are mostly not stale once-hot blocks.
- hit-rate (hm_0 @ 131072): Clears the cycle-0007 bar, but only barely. Favoring decayed main-proven blocks over fresh counter-0 promotions does recover some of hm_0's long-distance reuse. But one extra sweep visit is a small amount of extra residency compared with the main-scale reuse distances hm_0 depends on.
- hit-rate (web_0 @ 16384): Essentially flat. In a 64-slot main region the hand wraps quickly, so one more visit of grace only delays a veteran's eviction by a short time. That rarely reaches the block's next reference.

## Hurt

- hit-rate (hm_0 @ 16384): No change (neither helped nor hurt). In the small main region, the extra grace visit was not enough to change outcomes, so the bit-length tier coarsening that v001 introduced is still the cost here.

## Takeaway

A one-shot "hit while in main" bit is a safe tiebreak among counter-0 main residents. It never costs a cell. Where it helps, it helps web_0 at large size more than hm_0, so web_0 benefits from remembering main-proven reuse, not just from the faster decay in v001. hm_0 @ 131072 moved only marginally, so how main orders its counter-0 victims is, like v001's recency gating, at most a small part of hm_0's remaining gap. That points back to admission or history outside main (probation, ghost/near-miss) rather than more ranking inside main. If grace is extended further, scaling it with main-region size is the obvious lever, since one visit is worth much more in a 768-slot ring than in a 64-slot one.
