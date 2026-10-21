# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants changed only the promotion path out of the stash. One changed where an item lands inside its bucket (v001, MRU landing), one changed which bucket it goes to (v002, spill to a free probing slot), and one added an admission filter (v003, two stash hits before promotion). The winner was v003, the only one that made it harder to get into the normal tier. v002 gained very little, and v001 lowered hit-rate.

## Patterns

- What decides the hit-rate here is which items are allowed into the scarce normal tier, not where they are placed. The variant that made admission stricter won. The variant that made it looser (v001, where promoting to MRU lowers the bar for moving to probing) lost.
- Every eviction in the normal tier starts a cross-bucket demotion cascade into a neighbour's target. Changes that cause more normal-tier moves hurt neighbouring buckets as well as their own.
- In steady state the normal buckets are almost always full together. Changes that only matter when a free slot exists (v002) rarely take effect.
- The stash is larger than the normal tier, so it can hold unproven keys while they wait. That is why a filter at the stash→target edge costs little.
- Item position and promotion thresholds are coupled. The "is hottest" check reads bucket position, so changing where an item is placed also changes how many hits it needs for the next promotion.

## Open questions

- Every variant was scored on hit-rate only. Nothing is known yet about metadata-size or per-op-time, including v003's claimed get-latency improvement.
- Is a threshold of two stash hits the best value? Would a frequency or recency filter on the target→probing edge help in the same way?
- Could an SLRU-style MRU landing work if it were combined with a separate hit count, so the bar for entering probing stays where it is?

## Continuation

Build on v003. Try stricter or adaptive admission on the other edges: a different stash-hit threshold, and a hit-count requirement for target→probing. Get v003 evaluated on per-op-time and metadata-size too. Drop "free slot" placement ideas like v002. Only revisit MRU landing if it keeps the current bar for entering probing.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three variants were built on cycle-0001 v003. Two of them tightened admission: v001 put a frequency bar on the target→probing edge, and v002 raised the stash→target threshold from 2 to 3. The third, v003, changed retention instead: items demoted from the normal tier now re-enter the stash at the MRU end. v003 won. v002 also raised hit-rate, but by less, and v001 was flat.

## Patterns

- The cohort moved from admission to retention. Last cycle's lesson, "stricter admission wins", now gives smaller returns. The stash edge still gains a little at 3 and looks close to its optimum. The probing edge gains nothing because Front landing already filters it.
- The demotion pre-credit only pays off if the credited item survives long enough to collect its return hit. Where an item is placed in the stash (v003) and how big the credit is relative to the threshold (v002) both decide whether ex-residents come back cheaply. These two mechanisms work together; they are not competing.
- A useful split is "fresh keys face a high bar, proven ex-residents face a low bar and get protected". This is SLRU/2Q-like behaviour, and both variants that gained used it.
- Gating the target→probing edge did nothing on this trace. Its counter accumulates hits and is pre-credited to foreign residents, so it barely filters. The cascade churn from that edge seems small, so there is little left to win there through admission.
- Every result so far comes from a single LIRS trace at a single cache size and covers hit-rate only.

## Open questions

- Do v003 (MRU stash re-entry) and v002 (threshold 3) stack when combined? Or does protecting ex-residents at MRU crowd out one-touch keys that need a third hit?
- Does MRU re-entry hurt on scan-heavy or larger-working-set workloads?
- Per-op-time and metadata-size are still not measured for any winner.
- Would routing evictions differently (where a demoted or cascaded item goes) do better than any further admission gating?

## Continuation

Build on v003. First test combining it with v002's threshold of 3 and the threshold-relative pre-credit. Then probe other stash retention controls, for example the depth at which demoted items land or protection for pre-credited items. Also try routing evictions to change cascade targets. Drop frequency gating on the target→probing edge, and don't push the stash threshold past 3. Get per-op-time and metadata-size measured, and run on at least one more trace or cache size.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants were built on cycle-0002 v003 (MRU stash re-entry) and each tested a different lever. v001 stacked admission by raising the stash threshold to 3. v002 changed stash victim choice to a CLOCK-like scan that prefers counter-0 slots. v003 changed cascade routing so that evicted foreign residents go straight to the stash. None of them moved hit-rate beyond noise. v002 is the nominal winner, but only by a noise-level margin.

## Patterns

- The design looks saturated on this trace at this size. Once the stash works as an MRU-placed victim cache with a pre-credit, admission, victim choice and routing all end up choosing between victims that can be recovered cheaply. Changing which one is chosen barely matters.
- The pre-credited ex-residents seem to dominate stash traffic. That explains all three null results. A higher threshold has no fresh keys to filter (v001). The scan skips the same items that MRU placement already protects (v002). Rerouting the cascade swaps one victim the stash can bring back for another (v003).
- Position-based protection and frequency-based protection in the stash overlap almost completely. Stacking two retention mechanisms that protect the same items adds nothing.
- The neighbour cascade damage flagged in cycle 0001 no longer matters for hit-rate now that the stash recovers demoted items cheaply.
- v001's result matched its parent exactly. That could mean a real no-effect, but it could also mean the change never reached the evaluated build. None of the variant writers could inspect the overlay source.

## Open questions

- Did v001's threshold change actually reach the evaluated build?
- Does v003's cascade-free routing lower per-op-time, as its rationale predicted? It cost no hit-rate, so it may win on that axis instead.
- Is the plateau a property of this trace and cache size, or of the design? Every result so far comes from one LIRS trace at one size.
- Are stash capacity, or the split of slots between the stash and the normal tier, the remaining hit-rate levers?

## Continuation

Stop running hit-rate-only variants on this single trace: three different levers came out flat, so the design looks saturated there. Next cycle should put per-op-time and metadata-size measurement first (v003's cascade-free routing is the natural candidate for a per-op-time win), check that v001's overlay was applied, and evaluate on at least one more trace or cache size. If hit-rate work continues, try structural capacity changes such as the stash-to-normal slot ratio. Drop further tweaks to stash victim choice and admission thresholds.


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
