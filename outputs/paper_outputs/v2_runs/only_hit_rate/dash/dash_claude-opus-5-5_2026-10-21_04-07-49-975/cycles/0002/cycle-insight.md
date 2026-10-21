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
