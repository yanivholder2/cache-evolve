# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants started from baseline and each changed one thing: the partition split (v001), the graduation rule (v002), and the data structures with policy left unchanged (v003). The Winner was v001, which shrank the small queue to the paper's 10%. v002 lowered the graduation threshold and lost a little hit-rate. v003 kept hit-rate exactly where it was.

## Patterns

- On workload-a, giving more room to the main queue and the ghost queue (which grows with main) helped. Making it easier to enter main did not. The policy rewards keys that have shown repeated reuse, not keys that were reused once.
- The ghost queue already acts as a safety net. A key that misses graduation can still get back into main later, so both a shorter probation window (v001) and an earlier promotion (v002) change less than expected.
- Each variant reported hit-rate only. Neither per-op-time nor metadata-size was measured anywhere in the cohort, so the stated purpose of v003 and the ghost-scan cost risk in v001 are both still untested.
- v002 changed two things together (the threshold and a frequency reset on graduation), so we can't tell which one caused the regression.

## Open questions

- Does the larger ghost queue in v001 hurt per-op-time while the linear scans are still in place?
- Does v003's indexed slab actually lower get/put latency, and what does it cost in metadata-size?
- Was v002's regression caused by the lower threshold, by the frequency reset, or by both together?

## Continuation

Use v001's 10% split as the new policy baseline, and port it onto v003's behaviour-neutral indexed structure so that hit-rate gains and per-op-time gains can stack. Every variant must report all three dimensions. Don't pursue earlier graduation as it was tried here. If you revisit it, test the threshold change alone, with no frequency reset. Partition sweeps around 10% and ghost-queue sizing that is decoupled from main are worth probing next.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three variants branched from cycle 0001's v001 (10% split) and each changed one thing. v001 decoupled the ghost and doubled its size. v002 gave ghost-readmitted keys a freq-1 credit. v003 shrank the small queue to 5%. Only v003 won, and only by a small margin. Both ghost-handling variants lost a little hit-rate.

## Patterns

- Both ghost changes made re-admission more generous, either by keeping a longer history or by giving re-admitted keys a free second lap, and both hurt. On workload-a, one ghost hit is weak evidence of reuse. Main capacity spent on re-admitted keys comes out of keys that have already shown repeated reuse. This matches cycle 0001's finding that the policy rewards proven reuse rather than easy entry.
- The hit-rate gains from the partition sweep are coming from a bigger main queue, not from a bigger ghost queue. v001's isolated ghost test answers cycle 0001's question about where the 10% split's gain came from.
- The partition curve is still rising slightly at 5% but looks close to flat. At this point the ghost, not the small queue, is doing most of the filtering.
- Per-op-time and metadata-size were again not measured, even though cycle 0001's Continuation asked for them. The port onto the indexed structure was also not attempted. Every ghost-growing change, including the Winner, lengthens the linear ghost scan, so its unmeasured cost keeps going up.

## Open questions

- Is a stricter ghost re-admission rule net positive on workload-a, for example requiring repeat ghost hits or inserting at the main tail? And does that rule make a larger ghost useful?
- How much per-op-time does the Winner's larger ghost cost under linear scans?
- Do the small partition gains hold on other traces or other cache sizes?

## Continuation

Use v003's 5% split as the policy baseline. Don't pursue ghost re-admission credits or larger ghost queues on their own. Ghost changes are only worth trying if they make re-admission more selective. The top priority is the action carried over from cycle 0001 and still not done: port the current policy onto the indexed structure, and require every variant to report per-op-time and metadata-size as well as hit-rate. More partition shrinking beyond 5% is likely to give diminishing returns.


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

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
