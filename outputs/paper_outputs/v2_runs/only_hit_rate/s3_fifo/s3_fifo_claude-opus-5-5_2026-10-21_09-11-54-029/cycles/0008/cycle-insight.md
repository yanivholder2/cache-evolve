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
