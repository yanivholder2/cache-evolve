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

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
