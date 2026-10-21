# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0010

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0010/hypotheses/H03.md

## Prior cycles digest

### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

All three variants branched from cycle 0006's v002 (fingerprint-ring ghost). Two were cost-side changes that left the policy unchanged: v001 added an identity hasher on the ghost map, and v003 narrowed fingerprints to 32 bits. One changed the policy: v002 replaced main's FIFO reinsertion with a SIEVE-style hand that keeps survivors in place. v002 won with a hit-rate gain beyond its stated noise band. v001 matched its parent exactly and v003 stayed within noise, both as predicted.

## Patterns

- Main-queue eviction *order* was the one lever left untouched, and it is the first real hit-rate gain in several cycles. The "saturated" verdict from cycles 0005–0006 applied only to the partition, credit, graduation and re-admission knobs, not to the policy family. The likely mechanism ties back to the re-admission theme: freq-0 ghost re-entries stop being pushed toward the tail by reinserted survivors, so they get a full sweep to prove reuse. That mechanism is inferred and was not logged.
- SIEVE changes what the retention knobs mean. A credit is now an extra hand lap, not a trip back to the head. So the cap and decay results from cycles 0004–0005 were measured under FIFO reinsertion and may not carry over.
- Both cost-side variants were confirmed safe for the policy (the identity hasher is bit-for-bit identical, and 32-bit fingerprint collisions are negligible at this size). They are also independent and compose cleanly. Neither can be preferred yet, because per-op-time and metadata-size are still unmeasured, now for seven cycles. The Winner adds its own unmeasured costs: per-entry link fields, and sweep steps that chase pointers instead of popping contiguous deque entries.

## Open questions

- Under SIEVE, do lower caps, or clearing credit on pass instead of decrementing it, change the result? Cap and decay were settled only under FIFO reinsertion.
- Does the SIEVE gain hold on other traces and cache sizes, or is it specific to workload-a's reuse pattern?
- Does pointer-chasing in the sweep cancel out the per-op-time savings from the identity hasher and the narrow fingerprints?

## Continuation

Use v002 (SIEVE hand in main, all else as cycle 0006) as the baseline. Fold in the identity hasher and 32-bit fingerprints together as a single cost-only change, since both are proven policy-neutral. Spend the policy budget re-tuning retention under SIEVE (cap value, decrement vs clear-on-pass). Do not reopen partition, graduation, re-admission or ghost-capacity levers. Measuring per-op-time, metadata-size and a second trace or size is still the prerequisite for judging every cost-side change.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants branched from cycle 0007's v002 (SIEVE hand in main). v001 cleared a survivor's credit on each hand pass instead of decrementing it, which is the classic SIEVE visited bit. v002 inserted main newcomers just behind the hand instead of at the head. v003 combined 32-bit fingerprints with an identity-hashed ghost map, with no policy change. v001 won with a small hit-rate gain, just outside its stated noise band. v002 regressed. v003 matched its parent, as intended.

## Patterns

- Under SIEVE, less retention is better. Multi-lap credit (cap 7) won under FIFO reinsertion but loses to a single visited lap once survivors stay in place. Earlier cap and decay verdicts do not carry over to SIEVE. The FIFO-era rule held that stronger retention pays; under SIEVE, both of this cycle's results point the other way.
- Both policy results agree. v001 helped by evicting stale keys sooner. v002 hurt by protecting newcomers longer. Cycle 0007's inferred mechanism ("re-entries get a full sweep to prove reuse") is refuted as stated. SIEVE's quick demotion of newcomers near the hand does useful filtering, and the SIEVE gain comes from survivors staying in place, not from giving newcomers long residency.
- The representation changes are now confirmed policy-neutral on the SIEVE lineage. The full-state reference comparison passes, and the extra false-ghost-hit risk is negligible at this size. They still have no measured benefit, because per-op-time and metadata-size remain unevaluated for the eighth cycle. v001 also probably lowers worst-case sweep length, which is unmeasured as well.

## Open questions

- Does a small main-only retention cap (2–3 laps) beat clear-on-pass, or is a single visited bit optimal?
- With clear-on-pass, can small-queue graduation use its own counter, separate from the main retention signal?
- Would ghost re-admissions do better with even less protection (inserted nearer the hand, or treated differently from graduates)?
- Do the SIEVE and visited-bit gains hold on other traces and cache sizes?

## Continuation

Use v001 (SIEVE with clear-on-pass) as the baseline, and fold in v003's 32-bit fingerprints and identity-hashed ghost map as a cost-only rebase. Drop newcomer-protection ideas such as behind-hand insertion. Probe the opposite direction instead: small main-only caps (2–3), less protection for ghost re-admissions than for graduates, and a graduation counter separate from main retention. Do not reopen partition, re-admission credit or ghost-capacity levers. Measuring per-op-time, metadata-size and a second trace or size is still the prerequisite for judging cost changes and for confirming that the gains generalise.


### Cycle 0009

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
