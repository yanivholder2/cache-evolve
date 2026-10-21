# Cycle 0009 Insight

## Cohort outcome

All three variants branched from cycle 0008's v001 (SIEVE with clear-on-pass). v001 gave freq ≥ 2 survivors a second lap of main retention. v002 reset a graduate's freq to 0 when it was promoted from small. v003 was the cost-only rebase: 32-bit fingerprints, a cheap FxHash/splitmix mixer and an identity-hashed ghost map. v003 won by matching its parent exactly. Both policy probes moved hit-rate slightly down, within or near noise.

## Patterns

- Both policy probes landed on the same answer from opposite sides: graduates entering main with a single carried lap is the sweet spot. Adding a lap (v001) hurt, and so did removing the carried credit (v002). One visited bit, plus credit carried over from small, looks locally optimal under SIEVE on workload-a.
- Small-queue hits are real evidence of reuse, not one-burst noise. Within the 5% window, keys that reach freq > 1 deserve their carried protection. Graduation and main retention share one counter, and separating them gains nothing.
- v001 was not a clean test of S3-FIFO's 2-bit idea. Graduates carry freq ≥ 2 into main, so the second lap went mostly to small-queue history rather than to repeated main hits. That confound weakens, but does not overturn, the "one lap is optimal" reading.
- The cheap representation is now confirmed policy-neutral on the current lineage, with full-state reference parity. Its cost benefits are still unmeasured, for the ninth cycle running, so it wins only as a cheaper base.

## Open questions

- Would a two-lap rule that counts only main hits (freq capped or reset at graduation, with a separate main-hit bit) behave differently? It is low priority, since both neighbouring points regressed.
- Ghost re-admissions are now the only newcomers with no credit. Should they get less protection still (e.g. insertion nearer the hand), or does their freq-0 entry already filter them enough?
- Do the SIEVE and clear-on-pass gains, and the cost savings, hold on other traces and cache sizes?

## Continuation

Use v003 (clear-on-pass SIEVE on 32-bit identity-hashed fingerprints) as the baseline. Drop main retention caps above one lap and graduation-credit resets or splits. Main-side retention and graduate filtering look exhausted on workload-a. The only remaining main-side policy thread is how ghost re-admissions are handled, and it is optional. The higher priority, now overdue, is making per-op-time and metadata-size measurable and adding a second trace or cache size. Without those, no further hypothesis can be judged meaningfully.
