# Cycle 0005 Insight

## Cohort outcome

Two variants branched from cycle 0004's v003 (`u16`/2× sketch). v001 merged in the fused increment, and v002 removed the double count on get-miss followed by put. v003 branched from cycle 0004's v001 and added a ~1% plain-LRU admission window, with the region flag packed into a link bit. v002 gave the only hit-rate gain, and it was small. v001 merged cleanly but carried over none of the latency gain. v003 lowered hit rate and raised per-op cost. All three are listed as Winners, but only v002 moved a headline axis forward.

## Patterns

- The fused-increment latency gain did not carry over to the `u16` sketch. Hit rate was identical bit for bit, so the merge is behaviourally sound. That makes cycle 0004/v001's speedup look like either a 32-bit-counter effect or bench drift. Sketch micro-optimisation is used up, just as sketch accuracy was last cycle.
- How accesses are counted barely matters. The +2 bias on a miss applied to residents and newcomers alike, so it mostly cancelled in the duel. Fixing it only changes close duels. Together with cycle 0003's tie-rule result, this suggests the duel's inputs are now about as fair as they can get.
- A tiny window ahead of the duel hurts on workload-a. At ~1% it is too small to hold recency bursts. It also moves the duel from the arriving key to an older window head, which reorders admission badly. Every hit then pays for list selection and masked links, and every full miss re-hashes stored keys. This matches cycle 0002's aging result: the workload rewards stable frequency, and recency-oriented structure doesn't pay for itself.
- Packing the region flag into a link bit costs nothing per entry. If structural changes come back later, per-node bytes are no longer the obstacle.
- Per-op readings are still unreliable. v002 slowed down with no plausible cause in the code. Only v003's regression has a clear mechanism behind it.

## Open questions

- Would a larger window, around 5–20%, with cached per-node hashes or estimates so the overflow duel doesn't re-hash, behave differently? Or is any window wrong for this trace?
- With sketch and admission inputs both used up, is the remaining cost in `HashMap` lookup, `key_hash`, and relinking on every hit, rather than in the sketch?
- Is workload-a's hit rate now close to its ceiling for this design family?

## Continuation

Use v002 (single count per access) as the base. Optionally fold in v001's merged increment: it is orthogonal and leaves hit rate unchanged. Re-measure its latency side by side against cycle 0004's v003 before treating the regression as real. Drop sketch micro-optimisation, access-accounting tweaks, and tiny (~1%) windows. If you test a window again, make it substantially larger and cache each node's hash so the duel does no re-hashing. Otherwise, move the effort to the costs outside the sketch: map lookup, hashing, and relinking on every hit.
