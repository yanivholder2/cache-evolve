# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three variants tried: wider buckets (geometry, v001), cheaper one-step hit promotion (v002), and 2Q-style half-promotion for scan resistance (v003). Only the geometry change (v001) won, with a small hit-rate gain. Both policy-tweak variants slightly lowered hit rate versus plain move-to-back LRU.

## Patterns

- Only hit-rate was reliably measured. Per-op-time and metadata-size were missing from every variant's fitness vector, so the speed and size claims in all three hypotheses went untested.
- Both variants that weakened promotion in short buckets (v002, v003) lost hit rate. Full move-to-back recency seems hard to beat at small associativity on this LIRS trace.
- The geometry gain was marginal, so hash imbalance across sets is a minor source of misses. Most misses look like capacity or policy misses.
- Winning by a small hit-rate margin is unproven, because the likely timing cost of 64-way scans and shifts was never checked.

## Open questions

- Why was per-op-time not reported, and can the evaluation be fixed so speed-targeting hypotheses can be judged?
- Does the wider-bucket gain survive once get/remove/push cost is measured?
- Would scan resistance work with a separate probationary region instead of a shallower promotion within one short bucket?

## Continuation

Build on v001's geometry, but first make sure per-op-time and metadata-size are measured so trade-offs can be judged. Drop promotion-weakening ideas (transposition, midpoint) in plain LRU buckets. Next, try policies that change eviction or admission (e.g. a separate probationary segment, frequency-aware victim choice) rather than hit-path promotion.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Three eviction/admission ideas were tried on the wide-bucket geometry: a probationary segment (v001), frequency-aware victim choice (v002), and two-choice bucket placement (v003). Only the probationary segment won, with a clearly larger hit-rate gain than cycle 0001's geometry change. Frequency-aware victim choice left hit rate unchanged. Two-choice placement was rejected at the invariant gate.

## Patterns

- Admission control (separating one-touch entries from re-referenced ones) helped, while a small frequency window on the LRU end did nothing measurable. The scan-pollution benefit seems to need a structural separation, not a tie-break among a few old entries.
- Placement changes that break the "key lives in its hashed bucket" rule fail the slot-legality invariant, even on the baseline. Any multi-bucket placement scheme conflicts with the gate as written.
- Per-op-time and metadata-size were again unmeasured, so none of the added bookkeeping costs (partition counters, per-entry counters, longer scans) has been weighed against the hit-rate gains. This gap is now two cycles old.
- Policy, not geometry or hash balance, looks like the main limit on hit rate for this trace.

## Open questions

- Is the probation size (about a quarter of the bucket) near the best value, or does a smaller or larger share do better?
- Would a wider frequency window, or an actual probation-to-protected frequency test, add anything on top of the probationary segment?
- Does the probation win hold once per-op-time and metadata-size are measured?

## Continuation

Build on v001's probationary segment: tune probation size and try variants that combine it with frequency or ghost-history information for promotion. Drop small-window frequency victim selection and any placement scheme that moves keys outside their hashed bucket, unless the invariant is deliberately relaxed. Get per-op-time and metadata-size into the fitness vector before judging further trade-offs.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

Three refinements of the probation/protected split were tried: a smaller probation segment (v001), a per-bucket ghost ring that promotes ghost hits straight to protected (v002), and an adaptive 2Q-style probation target with ghost history (v003). Only the smaller probation segment won, with a small hit-rate gain. The ghost-promotion variant did not improve on its parent.

## Patterns

- Tuning the split size gave only a small gain, so most of the benefit comes from having the split at all. This matches cycle 0002.
- Ghost-history ideas did not add value on top of the split. A ghost hit looks like weak reuse evidence on this trace and promoting on it can displace entries that had earned protection.
- v003's insight compares against plain LRU and credits the whole split, not an increment over the current parent. It is hard to read as a new gain, and it did not win.
- Per-op-time and metadata-size were unmeasured for the third cycle running. Every added piece of state (ghost rings, adaptive targets, markers) is still unpriced.

## Open questions

- Where does the smaller-probation trend turn? Is a smaller share still better, or is the optimum near the current one?
- Does a fixed probation size match an adaptive one, making the extra ghost state unnecessary?
- Would a stricter promotion rule (ghost hit plus a second signal) rescue ghost history?
- Do the gains survive once speed and metadata cost are measured?

## Continuation

Build on v001's smaller probation size and probe a few more sizes to find the turning point. Drop ghost-hit direct promotion and further ghost-ring variants unless promotion is made stricter. Fixing the evaluation so per-op-time and metadata-size appear in the fitness vector should come before adding any more bookkeeping.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

Three probation-segment variants were tried: shrinking probation further (v001), a stricter two-hit promotion rule (v002), and a minimal fixed probation share built directly on the baseline geometry (v003). The two Winners were the small-probation variants (v001, v003). The stricter promotion rule lost hit rate.

## Patterns

- Two independent routes both ended at a small probation segment (about 4 slots per bucket) and both won. The trend toward smaller probation has not clearly turned, but the gain from v001 is marginal and looks to be flattening.
- v003 reports a large gain over plain LRU without the cycle 1-3 additions (ghost rings, counters, adaptive state). This suggests the simple split plus small probation captures most of the benefit, and the extra bookkeeping from earlier cycles may be unnecessary.
- Delaying promotion hurt (v002). With a small probation segment, entries cannot wait for a second hit without being evicted. Together with cycle 0003's ghost result, first-hit promotion looks hard to beat here.
- Per-op-time and metadata-size were unmeasured for the fourth cycle running. Every claimed win is hit-rate only, and the cost of segment reordering and demotion on `get` is still unpriced.
- v003's comparison is against the baseline, not the current parent, so its size is not an increment over v001.

## Open questions

- Where is the actual minimum of probation size? Is going below 4 slots (for example 2, or 1) still better, or does scan resistance break?
- Does v003's simple split match the full accumulated stack, and can the older extra state be removed?
- Do the hit-rate gains survive once per-op-time and metadata-size are measured?
- Would a larger probation segment make a two-hit rule viable, or is first-hit promotion simply better?

## Continuation

Probe probation sizes just below the current point (and one above, as a check) to find the turn, and consider simplifying by stripping unneeded earlier state, using v003's lean split as a reference. Drop two-hit or other delayed-promotion rules in a small probation segment. Fix the evaluation so per-op-time and metadata-size appear in the fitness vector before adding any new bookkeeping.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Three probes of the small-probation segmented LRU were tried: a fixed 2-slot probation (v001), a recency-stamp compare that lets stale protected entries be evicted (v002), and a lean positional 4-slot split rebuilt on the baseline with no extra state (v003). Only v003 was a Winner; v001 and v002 each lost a little hit rate against their parents.

## Patterns

- The probation-size trend has turned. Going below the slots/16 sizing slightly hurt, so the current size is at or near the optimum and further shrinking is exhausted.
- Eviction-side tweaks did not help. Letting raw recency override probation-first eviction cost hit rate, because the segment boundary already encodes reuse evidence that one last-access stamp does not capture.
- Together with earlier cycles (ghost history, delayed promotion, frequency windows, size tuning), most simple levers on this split have now been tried and found flat or negative. The simple structural split holds almost all the gain.
- v003 again shows that a lean positional split with no ghost, counter or adaptive state improves on plain LRU. Its comparison is against the baseline, not an increment over the current best.
- Per-op-time and metadata-size were unmeasured for the fifth cycle running. v003's "lean" claim (same footprint and speed as baseline) is still untested, and promotion/demotion reordering on hits is still unpriced.

## Open questions

- Does the lean split really keep baseline metadata size and per-op-time, and do the gains survive once those are measured?
- Is there any remaining hit-rate headroom in promotion or protected-segment handling, or is this design at its ceiling for the trace?
- Does the result hold on traces other than the LIRS-style one, or is the tuning specific to it?

## Continuation

Treat the lean 4-slot positional split (v003, with slots/16 probation sizing as the reference) as the base. Drop further probation-size shrinking and recency- or stamp-based victim overrides. The next cycle should first get per-op-time and metadata-size into the fitness vector; after that, try only mechanisms that add no per-entry state, or check robustness on other workloads.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

Three probes were tried: demoted-entry placement at the probation LRU end (v001), a rotate-based rewrite of promotion/demotion aimed at per-op-time (v002), and doubled associativity combined with the lean 4-slot split on the baseline (v003). v001 and v002 won only by tying their parents on hit-rate. v003 improved clearly over baseline, but that is a gain against the baseline and not over the current best.

## Patterns

- Both parent-derived variants were exactly neutral on hit-rate. Demotion placement is not a lever, which fits the earlier finding that most simple levers on this split are flat. The two Winners are ties, not improvements.
- v002 was a policy-preserving refactor, so hit-rate cannot judge it. Its goal was per-op-time, and that has still not been measured. The two Winners say nothing about speed.
- v003 suggests wider buckets and the small probation segment stack. This is an unpriced claim: longer bucket scans and segment shuffles may cost latency.
- Per-op-time and metadata-size were unmeasured for the sixth cycle running, so no cost-side claim has ever been confirmed or refuted.
- Variant authors could not read the overlay code, so a tie that exact may also mean the edit never changed behaviour.

## Open questions

- Does the rotate rewrite lower per-op-time, and do the wider buckets raise it?
- Is v001's exact tie real neutrality, or a demotion path that rarely fires or an edit with no effect?
- Does the associativity gain over the incumbents hold, or is it only a gain over the baseline?
- Do the results hold on traces other than the LIRS-style one?

## Continuation

Stop probing hit-rate levers inside the segmented split (demotion placement, probation size, promotion rules, ghost state); they are exhausted. Make per-op-time and metadata-size measurable first, since v002 and v003 can't be judged without them. Then take wider associativity from v003 and test it as an increment on the current best parent, or check robustness on other workloads.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Three probes were tried: doubled associativity as an increment on the best lean split with probation scaling at slots/16 (v001), the same doubling with probation held at an absolute 4 slots (v002), and a policy-preserving speed refactor of the rotate-based split (v003). Only v001 was a Winner, with a very small hit-rate gain. v002 lost marginally, and v003 was a hit-rate tie that cannot be judged.

## Patterns

- Wider buckets do stack on the incumbent, but the increment is tiny. The remaining misses look like capacity or recency-ordering misses, not bucket-conflict misses.
- Proportional probation sizing (slots/16) held up better than an absolute 4 slots on wider buckets. Associativity and probation size are only weakly coupled, and the absolute-scan-length reading is not supported.
- This is the seventh cycle with only hit-rate visible. v003's refactor (early-exit scan, minimal-range rotate, reused miss-scan) is unjudged on its goal. The cost of wider scans in v001 is also unpriced.
- Both v002 and v003 authors could not inspect the code. v003's exact tie confirms policy preservation only if the edit actually took effect.
- The hit-rate levers on this split are essentially exhausted: size, promotion, demotion, ghost state and now associativity all give flat or marginal results.

## Open questions

- Does v003 lower per-op-time, and does v001's wider bucket raise it enough to cancel its small hit-rate gain?
- Is the hit-rate ceiling of this design on the trace reached, so that further gains need an admission or eviction policy of a different kind?
- Do the results hold on traces other than the LIRS-style one?

## Continuation

Stop probing hit-rate knobs in the segmented split (associativity, probation size or sizing rule, promotion, demotion). Treat v001 (wide buckets, slots/16 probation) as the hit-rate base only if per-op-time shows its cost is acceptable. Put per-op-time and metadata-size into the fitness vector first, then judge the v003 rotate refactor against it. After that, try either a different policy family or robustness checks on other workloads.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

Three probes were tried: LIP-style insertion at the probation eviction end (v001), a further doubling of associativity to 4x baseline at fixed capacity with slots/16 probation (v002), and a boundary-insertion lean split with a skip-move rule for hits already in the recent quarter (v003). Only v002 was a Winner, with a marginal hit-rate gain. v001 lost hit-rate against its parent, and v003 improved on plain LRU but did not beat the incumbent.

## Patterns

- Insertion position is the one lever that moved things this cycle, in both directions. Inserting at the victim end starved new keys of the residency needed to earn a first re-reference. Inserting just above the probation boundary kept scan protection without that starvation. The trace's short reuse distances punish aggressive scan resistance.
- The associativity increment shrank again. The first doubling gave a clear gain, the second a marginal one, so the remaining misses look capacity- or recency-driven, not conflict-driven.
- v003 repeats the earlier finding that a lean positional split with no extra per-entry state beats plain LRU. Its comparison is against the baseline, not against the current best.
- The hit-rate levers on this split are now almost fully worked: size, promotion, demotion, ghost state, associativity and insertion point all give flat, marginal or negative results.
- For the eighth cycle running, only hit-rate is visible. The cost of wider scans in v002 and of positional checks in v003 is unpriced. Authors could not read the overlay code, so their explanations are inferred.

## Open questions

- Does the cost of 4x-wide bucket scans cancel the marginal v002 gain once per-op-time is measured?
- Is a mid-probation insertion point between MRU and the boundary better than either extreme, or is the parent's choice already near optimal?
- Is further gain only possible from a different policy family, such as frequency-aware admission, or does it need other workloads to show a difference?

## Continuation

Stop probing hit-rate knobs in the segmented split: associativity beyond the current width, LIP or victim-end insertion, probation size, promotion and demotion. Get per-op-time and metadata-size into the fitness vector first, then judge v002's width and v003's skip-move rule against them. After that, try a different policy family or robustness checks on other traces.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

Three probes were tried on the 4x-wide split: mid-probation insertion (v001), skipping the protected-MRU rotate for hits already in the top quarter (v002), and a larger slots/8 probation fraction on wide buckets (v003). v002 was the only Winner, and only as an exact hit-rate tie with its parent. v001 lost slightly, and v003 beat the baseline but was not compared against the incumbent.

## Patterns

- Insertion position is now negative at both extremes. Victim-end insertion (cycle 0008) and mid-probation insertion (this cycle) both cost hits, because new keys need residency to earn a second reference. Insertion point is not a useful knob.
- The Winner is a speed-motivated refactor with identical hit-rate, so it wins by default. Its goal, lower per-op-time, is again unmeasured.
- v003 repeats the pattern of a lean positional split beating the baseline. It does not show that slots/8 beats the incumbent's slots/16, so the probation fraction question is still open.
- Per-op-time and metadata-size are missing for the ninth cycle running. The cost of 4x-wide scans, segment shuffles and skip-move checks has never been priced.
- Authors again could not read the overlay code, so their explanations rest on the hypothesis text and are inferred. An exact tie may also mean the edit had no effect.

## Open questions

- Does the skip-move rule lower per-op-time enough to justify keeping it?
- Does the 4x width, with its marginal hit-rate gain, survive pricing in time?
- Is the hit-rate ceiling for this design on this trace reached, so that further gains need a different policy family?
- Do the results hold on other traces?

## Continuation

Stop probing hit-rate knobs in the segmented split: insertion point (victim-end or mid-probation), probation fraction, associativity, promotion and demotion. Get per-op-time and metadata-size into the fitness vector before another cycle, so v002's skip-move and the wide buckets can be judged. After that, either try a different policy family such as frequency-aware admission, or run robustness checks on other traces.


### Cycle 0010

# Cycle 0010 Insight

## Cohort outcome

Three probes were tried on the wide skip-move split: a larger slots/8 probation fraction (v001), a further doubling of associativity to 8x baseline with proportional probation (v002), and eviction of the first never-reused probation entry using a one-bit reuse flag (v003). Only v002 was a Winner, with a very small hit-rate gain. v001 and v003 lost slightly against their parent.

## Patterns

- Every lever is now flat or negative. The third associativity doubling gave only a marginal gain, and the increments keep shrinking. Remaining misses look capacity- or recency-driven, not conflict-driven.
- slots/8 probation is slightly worse than slots/16 against the incumbent. This closes the open probation-fraction question from cycle 0009: extra probation room takes protected capacity and gains nothing.
- A binary reuse bit is a worse eviction signal than plain probation recency. Demoted entries keep the flag, so fresh insertions are evicted before they can prove themselves. This matches the earlier negative results on insertion position.
- Per-op-time and metadata-size are still unmeasured, for the tenth cycle running. The cost of 8x-wide scans against a marginal hit-rate gain is unpriced.
- Authors again could not read the overlay code, so mechanisms are inferred from the hypothesis text and fitness deltas.

## Open questions

- Does the cost of 8x-wide bucket scans cancel the marginal hit-rate gain once time is measured?
- Would a multi-bit or recency-aware signal succeed where the one-bit flag failed, or is the ceiling for this design on this trace already reached?
- Do any of these results hold on traces other than the LIRS-style one?

## Continuation

Stop probing hit-rate knobs in the segmented split: associativity, probation fraction, insertion point, promotion, demotion, and binary reuse flags. Put per-op-time and metadata-size into the fitness vector before running another cycle, then judge v002's width against them. After that, switch to a different policy family, such as frequency-aware admission, or run robustness checks on other traces.


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
