# Cycle 0003 Insight

## Cohort outcome

All three variants changed a single policy knob. Two of them branched from cycle 1's S3-FIFO:
- **v001** shrank the ghost window to half of capacity.
- **v002** raised the probation promotion bar to two hits.

The third, **v003**, enlarged LIRS's resident-HIR queue. v001 won with a clear hit-rate gain. v002 was essentially flat, and v003 regressed slightly.

## Patterns

- **The ghost is the main lever on admission.** Cycle 2 lengthened the ghost and lost hit rate. This cycle shortened it and gained. Ghost size is a real tuning knob, and the parent's near-capacity ghost was too loose.
- **Probation strictness and ghost size can't be tuned separately.** With a large ghost, a key that v002 rejects from `small` comes back through the ghost-hit path straight into `main`. Its stricter bar only delays admission by one miss. Any test of "stricter admission" has to change the ghost path too.
- **Probation size doesn't explain why S3-FIFO beats LIRS.** Giving LIRS a bigger probation buffer cost protected LIR slots and didn't help. The gap between the two policies comes from somewhere else, most likely how each one handles ghost and non-resident re-admission.
- **Measurement is still missing.** For the third cycle in a row, only hit rate was reported. v001's smaller ghost probably saves memory, but that is unverified. Cycle 2's planned representation tests (fingerprint ring at the original ghost size, the hasher and narrow-index graft) were not attempted.

## Open questions

- Where is the optimum ghost size? Would going below half of capacity start dropping genuine re-references?
- Would sending ghost hits back into `small` instead of `main` beat simply shrinking the ghost? Does it make a stricter promotion bar meaningful?
- How much `metadata-size` does the smaller ghost actually save?

## Continuation

Use this cycle's v001 as the parent. Keep searching ghost size, going smaller (for example a quarter of capacity). Separately, test sending ghost hits into `small` rather than `main`. Revisit `PROMOTE_FREQ` only together with one of those ghost changes.

Drop the LIRS HIR-ratio tuning: restore the 1% split, or set LIRS aside. Measuring `metadata-size` and `per-op-time` remains the top harness priority. Once a ghost-size optimum is settled, run the representation-only experiments that cycle 2 asked for.
