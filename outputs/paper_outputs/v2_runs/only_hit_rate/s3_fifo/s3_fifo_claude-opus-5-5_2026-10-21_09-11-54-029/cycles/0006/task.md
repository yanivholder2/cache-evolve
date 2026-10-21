# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0006

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0006/hypotheses/H03.md

## Prior cycles digest

### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants branched from cycle 0002's v003 (5% split) and each changed one thing. v001 inserted ghost-readmitted keys at the main tail (stricter re-admission). v002 is the long-requested behaviour-neutral port onto indexed structures. v003 raised the main frequency cap from 3 to 7. The Winner was v003, by a small hit-rate gain. v002 matched its parent's hit-rate exactly, as intended. v001 regressed.

## Patterns

- After three cycles the lesson is consistent: workload-a rewards protecting keys that have shown reuse. The ghost hit is itself enough evidence of reuse. Giving re-admitted keys extra credit hurt in cycle 0002, and here a near-zero probation window (tail insertion) also hurt. The current front-of-main, freq-0 re-admission looks close to the right point between those two.
- Tail insertion did not test "require a second reuse signal". Its deadline is the next main insertion, not a span of the access stream, and in steady state that comes almost immediately. Any stricter re-admission rule needs a probation window measured in accesses.
- Main-queue retention is a new axis. It gave a real but modest gain: a cap of 3 already captured most of the frequency signal, because cold ghost re-entries usually supply the eviction victims.
- The indexed port finally exists and is verified as exact. Per-op-time and metadata-size were still not measured, so we can't yet tell whether it helps. The Winner's higher cap makes the linear-structure sweeps longer, so the unmeasured cost keeps growing on the incumbent lineage.

## Open questions

- How much per-op-time does the indexed port save, and how much metadata-size does it add, given that its slab, index and duplicated ghost keys use more memory?
- How long do eviction sweeps get under the higher frequency cap, and does a cap above 7 add any hit-rate?
- Does a re-admission probation window based on access count, somewhere between tail and front, beat front insertion?

## Continuation

Use v003's frequency cap of 7 as the policy baseline, and combine it with v002's indexed structure. The policy and structure changes are independent and should stack. Every variant must report per-op-time and metadata-size, which have now gone unmeasured for three cycles. Don't pursue tail insertion or ghost re-admission credits. Probing the frequency cap further, or decoupling the graduation threshold from the cap, is a reasonable next step. Do it only once the cost of the eviction sweep is measured.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
