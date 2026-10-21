# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0009

## Winners

- v003

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants branched from cycle 0005's v003 (indexed, cap 7, main victims ghosted). v001 gave main victims their own ghost budget on top of the full small-reject budget. v002 kept the policy fixed and replaced the cloned-key ghost with a fixed-size fingerprint ring. v003 adapted the small/main split ARC-style, driven by which source each ghost hit came from. v002 won with a within-noise hit-rate uptick, probably from the ring's slightly wider effective window. v001 and v003 both regressed.

## Patterns

- Adding ghost capacity is not free evidence. Every ghost hit bypasses the small filter and forces a main eviction. The shared, capped budget was protecting main residents from churn, and was not diluting useful evidence. This is the same lesson the re-admission experiments taught, seen from the capacity side: widen evidence a little and it helps, open the bypass further and it hurts.
- Ghost-hit source is a biased control signal on this trace. Small rejects dominate the shared ghost and main victims rarely return, so an adaptive split most likely drifts back toward the larger small queues that the static sweep already rejected. The adaptation also acts too late, because the hit it reacts to has already been re-admitted. This drift is inferred; it was not logged.
- The Winner is the first cost-side change, and it held the policy exactly (verified against a ring-based reference). Its claimed benefits are in per-op-time and metadata-size, which were again not measured. That is now six cycles. Its hit-rate win is incidental. Its main new cost is hashing twice on every miss.
- Every policy lever on workload-a at this size is now flat or negative. The policy family looks saturated here.

## Open questions

- Does the fingerprint ring actually cut per-op-time and metadata-size compared with the cloned-key ghost, and does the double hash on misses cancel out the savings?
- Would a smaller combined ghost budget, or a source-weighted (|B1|/|B2|-style) adaptive signal, behave differently? Both are low priority given saturation.
- Do the accumulated choices (5% split, cap 7, ghosted main victims, ring ghost) hold on other traces and cache sizes?

## Continuation

Use v002 (fingerprint-ring ghost, policy unchanged) as the baseline. An identity hasher on the ghost map is a reasonable small follow-up. Drop split ghost budgets, adaptive splits driven by ghost source, and anything that enlarges the ghost or makes the main bypass more generous. Do not propose more hit-rate tweaks on workload-a. The next cycle should make per-op-time and metadata-size measurable and add at least one more trace or cache size. Without those, no remaining hypothesis can be judged.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
