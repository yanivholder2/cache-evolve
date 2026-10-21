# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0008

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0008/hypotheses/H03.md

## Prior cycles digest

### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Each variant tested one lever that had not been tested before. v001 halved main credit on each sweep rotation instead of decrementing it, parented on cap 15. v002 lowered the small-to-main graduation threshold to a single hit, with no frequency reset. v003 recorded main-queue eviction victims in the shared ghost. v002 and v003 were parented on the indexed cap-7 baseline. v003 won, but only by a very small hit-rate margin. v002 was effectively flat and v001 regressed slightly.

## Patterns

- The credit-decay lever has now been tested and it loses. Faster aging hurts keys with moderate reuse, the same keys whose added laps produced the cap 3→7 gain. Neither cap-raising nor decay helps, so linear decrement at cap 7 looks close to optimal on workload-a. The cap-15 plateau did not come from stale hot keys crowding main.
- Graduation from small is also flat. The ghost already recovers keys that were hit once in small about as well as direct promotion does. Like re-admission, small-to-main admission now looks settled, with evidence from both directions across cycles.
- The only gain came from widening the ghost's evidence without making re-admission more generous. That is consistent with the five-cycle theme: reward proven reuse, and keep the entry point at front-of-main with freq 0. The gain is small because few main victims come back within the ghost's window once their credit has fully drained.
- Every remaining hit-rate lever on this trace now gives near-noise effects. Workload-a at this size appears close to saturated for this policy family.
- Per-op-time and metadata-size are still unmeasured, now for five cycles. The Winner adds work on every main eviction (a ghost insert, plus more frequent compaction). v001's only plausible benefit, a bounded sweep length, cannot be judged.

## Open questions

- Does a separate ghost budget for main victims grow v003's gain, or does it only show how much the shared budget dilutes small-queue rejects?
- Do the accumulated small wins (5% split, cap 7, ghosting main victims) hold on other traces and cache sizes, or are they fitted to workload-a at one size?
- What is the cost of the Winner's extra ghost traffic in per-op-time and metadata-size?

## Continuation

Use v003 (indexed, cap 7, main victims ghosted) as the baseline. Drop credit decay, cap changes, graduation-threshold changes and all re-admission credit or probation variants, since each has been tested and is flat or negative. Hit-rate headroom on workload-a looks nearly exhausted. Spend the next cycle on evaluation coverage instead: per-op-time, metadata-size, and at least one more trace or cache size. The only policy lever left worth a probe is a split or separate ghost budget for main victims.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
