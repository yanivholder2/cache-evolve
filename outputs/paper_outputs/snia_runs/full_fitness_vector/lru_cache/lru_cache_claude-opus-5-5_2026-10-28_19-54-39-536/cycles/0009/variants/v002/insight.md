# Variant v002 Insight

## Helped

- (none at the summary level. `metadata-size` is unchanged, as intended: the second-chance flag fits in spare bit 30 of `prev`, so `Node<u64,u64>` stays at 24 bytes and the header stays the same size. The 30-bit index limit costs nothing at the traced sizes.)

## Hurt

- hit-rate: The reprieve did not pay off. The summary dipped slightly instead of moving toward the ghost variants' level. Only web_0 at the large size improved; web_0 at the small size fell, and both hm_0 points were flat to slightly down. A demoted key is the coldest member of protected. Reprieving it doubles its probation residence with no new evidence of reuse, and each reprieve pushes out an unflagged newcomer that has not yet had a chance at its first hit. When probation is small (20% of a 16K cache), losing newcomers costs about as much as the reprieved keys win back, or more.
- per-op-time: Get p50 rose even though the get path gained no new branch. `on_hit` only gained an unconditional `&= !SC_BIT` on promotion and an unconditional `|= SC_BIT` on demotion, both to a node line it already writes, and `set_prev` actually lost its redundant mask. The reprieve branch itself lives in `put`, and put p50 did not move. So by the hypothesis's own test the result is ambiguous. Instruction count does not explain the get regression; it is most likely noise, or a codegen or inlining change from the slightly larger inlined `on_hit` in a larger `put` body. Get p99 fell back from the parent's outlier to a normal level. That is consistent with the parent insight's suspicion that its get-p99 spike was noise, and it says the same may apply to this p50 shift.

## Takeaway

A flag-only "second chance for demoted-from-protected keys" is nearly free in space and in put cost, but it does not reproduce the ghost table's hit-rate gain. Past protected membership turns out to be a weak predictor of future reuse here: the ghost works because it reacts to a re-reference after eviction, not to history before eviction. Because get p50 rose with no new get-path branch, this variant cannot settle the footprint-vs-branch question. A re-run, or several runs of the parent and this variant side by side, is needed before the get regression is blamed on the change.
