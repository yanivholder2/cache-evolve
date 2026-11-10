# Cycle 0003 Insight

## Cohort outcome

All three variants were single-change children of cycle-2 v003 (slab + hash). v001 ghosts only main victims that were hit while resident. v002 resets freq on promotion from small to main, alone. v003 swaps SipHash for an Fx-style hasher. All three are Winners, and this is the first cycle where neither trace regressed: the two policy variants each gained slightly on hm_0 and held or gained on web_0, and the hasher swap gave a timing gain that its diff explains, with hit rates bit-identical.

## Patterns

- Isolating one change per variant worked. The cycle-1/2 trade-off between hm_0 and web_0 went away once changes stopped being bundled. Both policy variants filter out "credit without reuse in main", either as ghost slots or as carried-over freq, and both help hm_0 without costing web_0.
- web_0's mechanism is now mostly pinned down. The freq reset is harmless, so cycle-2's web_0 loss came from the looser `freq >= 1` graduation threshold. Unreused ghost re-admissions are at most a minor factor: web_0 at the small size didn't move under v001, and its large-size gain was only about the size of hm_0's.
- Filters that free up ghost budget lengthen the useful history for free. This fits cycle 1's finding that the ghost bound is a hidden coupled variable.
- Timing noise is still a problem. Both policy variants showed per-op-time regressions their diffs can't explain. Only structural changes (v003 now, and cycle-2 v003) earn timing credit.
- Per-entry metadata stayed flat throughout. v003 also shrank the fixed per-cache struct.

## Open questions

- Do the v001 gate and the v002 reset stack additively? They touch different mechanisms, so combining them is the natural next test.
- What explains the rest of web_0's large-size gap? The remaining candidates are small-queue sizing and the ghost bound itself.
- Do the v002 `put`-tail shifts come from more frequent main evictions or ghost compaction, or from noise? Only a fixed-environment re-run can tell.

## Continuation

Merge the three Winners into one parent: Fx hashing + reset-on-promotion + reuse-gated main-victim ghosting, with the `freq > 1` threshold kept. Confirm the policy gains add up before testing anything new. After that, probe what's left of web_0's large-size loss, one variable per variant: ghost bound relative to main size, or small-queue share. For implementation, cut hash probes per miss with the entry API. Keep refusing timing credit for policy-only diffs until the benchmark environment is pinned.
