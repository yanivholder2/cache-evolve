# Cycle 0009 Insight

## Cohort outcome

All three variants were built on cycle-0008 v001 (recency-gated ghost, no BIP). Following that cycle's continuation, each one moved off in-stash placement. v001 added an admission doorkeeper that stops fresh keys from evicting evidence-backed residents. v002 halved the ghost table to fit the gated horizon. v003 moved one normal bucket's worth of slots into the stash. v002 won by holding hit-rate within noise (slightly up) while cutting metadata. v001 and v003 both came out flat to slightly negative.

## Patterns

- No hit-rate lever moved this cycle. The admission gate, the stash insert position, and now the tier ratio have all hit a plateau on this trace. The parent's design already does what the doorkeeper and the larger stash were meant to add. Fresh keys land at LRU, and the second-chance scan already keeps evidence-backed residents safe.
- The win was about efficiency, not hit-rate. The recency gate, not the table size, sets how much ghost history is useful. About half a cache's worth of fingerprints holds all the evidence that counts, and the extra direct-mapped collisions did not measurably push out recent entries. This continues the 0007–0008 finding that recency matters more than volume.
- Capacity is roughly fungible between tiers at the current split. A normal slot and a stash slot retain about the same value. So "a stash slot is worth more than a normal slot" is not supported, and nudging the ratio is a weak lever.
- The winner's claimed metadata saving is still unconfirmed. The fitness vector reports only hit-rate, so a variant won on a dimension nobody measured. For the seventh cycle running, writers could not inspect the overlay source, and every mechanism claim comes from the hypothesis text. Evaluation is still one LIRS trace at one cache size.

## Open questions

- Where does ghost shrinkage begin to lose hit-rate (a quarter, an eighth)? Does a set-associative ghost push that point further out?
- Did v003's normal-bucket indexing for a non-power-of-two count spread keys evenly, or did uneven spreading hide a small stash gain?
- Would any of this cycle's flat results change on a second trace or a different cache size, where useful reuse falls outside the gated horizon?

## Continuation

Build on v002 (halved ghost). If you shrink the ghost again, do it only once (for example to a quarter) to find where collisions start to cost hits. Drop the fresh-key doorkeeper and further stash/normal ratio shifts, since both are flat on this trace. Hit-rate micro-tuning on this one trace and size is exhausted. The next cycle should prioritize getting metadata-size and per-op-time reported, and should evaluate on a second trace or cache size, before proposing new mechanisms.
