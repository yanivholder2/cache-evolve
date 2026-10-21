# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0007

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants branched from cycle 0003's v003 (5% split, cap 7) and each changed one thing. v001 ported that policy onto the indexed slab/index/lazy-ghost structure without changing behaviour. v002 raised the frequency cap from 7 to 15. v003 sent ghost hits back through the small queue with one banked credit. v002 is the nominal Winner, but its hit-rate uptick is within noise. v001 matched its parent exactly, as intended, and v003 regressed clearly.

## Patterns

- The frequency cap stops paying off at 7. Main-queue evictions mostly take freq-0 ghost re-entries, which drain long before a hot key runs out of credit, so extra credits rarely change which key gets evicted. Main retention is now flat on the cap axis. The Winner's only certain effect is longer eviction sweeps, and that cost is still unmeasured.
- The re-admission question is now settled from both sides. Tail insertion (cycle 0003) and small-queue probation (this cycle) both lost, and extra credit (cycle 0002) also lost. A bare ghost hit is enough evidence of reuse. Front-of-main at freq 0 is the right re-admission point. v003 showed why the probation idea fails: the small queue's window is much shorter than the ghost's reuse horizon, so it rejects the keys the ghost correctly found. Routing re-entries through small also crowds out cold keys that are trying to graduate.
- The structural port requested for three cycles now exists on the current policy, verified step by step against the linear reference. Policy and representation are separate. But per-op-time and metadata-size were again not reported, so nobody can yet say whether the port helps. Four cycles have now passed without those measurements.

## Open questions

- How much per-op-time does the indexed port save, and how much metadata-size does it add (duplicated resident and ghost keys, a stale ghost deque, occasional compaction spikes)?
- Does the cap of 15 cost per-op-time on the linear structure? If it does, the Winner is a net loss.
- With partition, cap and re-admission all flat or settled, is there any hit-rate headroom left on workload-a? Credit aging or decay is the main untested retention lever.

## Continuation

Treat the cap of 15 as a hit-rate tie. Unless per-op-time shows the longer sweeps cost nothing, use cap 7 on v001's indexed structure as the effective baseline. Drop all ghost re-admission variants (credit, tail, small-queue probation) and stop raising the cap. Make the evaluator report per-op-time and metadata-size the first priority, because the remaining open questions depend on them. After that, the only hit-rate lever worth probing is how main-queue credit is aged or spent (e.g., decay), not how much of it there is. Also check whether the gains hold on other traces or cache sizes.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
