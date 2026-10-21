# Cross-Cycle Insights

## Recurring wins

- **Controlling who gets into the scarce normal tier matters more than where items are placed.** Stricter stash→target admission won in cycle 1 and still gained something at a higher threshold in cycle 2. It costs little because the stash is larger than the normal tier, so unproven keys can wait there.
- **Treat fresh keys and proven ex-residents differently (SLRU/2Q-like).** Fresh keys face a high bar to enter. Items demoted from the normal tier get protection (MRU stash re-entry) and a pre-credit so they can come back cheaply. Every variant that gained in cycle 2 used this split.
- **Retention changes and admission changes complement each other.** The pre-credit only pays off if the demoted item stays in the stash long enough to get its return hit. That depends on both where it is placed and how big the credit is relative to the threshold.
- **Build each cycle on the previous winner.** The lineage is cycle-1 v003 (two stash hits before promotion), then cycle-2 v003 (MRU re-entry for demoted items), and each step improved hit-rate.

## Recurring failures

- **Changes that loosen the bar for moving up hurt hit-rate.** MRU landing in the target bucket (cycle 1) also lowered the bar for promotion to probing, because the "is hottest" check reads bucket position. That cost hit-rate.
- **"Free slot" placement ideas barely work.** In steady state the normal buckets are almost always full together, so spilling into a free probing slot rarely happens.
- **A frequency gate on the target→probing edge did nothing.** Front landing already filters that edge, and its hit counter accumulates and is pre-credited to foreign residents, so the gate filters almost nothing. Churn from this edge seems small anyway.
- **Pushing one admission threshold higher and higher gives less each time.** The stash threshold looks close to its best value at 3. Don't go higher.
- **More normal-tier moves cause collateral damage.** Each normal-tier eviction starts a demotion cascade into a neighbouring bucket, so changes that add moves hurt other buckets as well.

## Open questions

- Do MRU stash re-entry and the threshold of 3 (with threshold-relative pre-credit) stack when combined? Or do protected ex-residents crowd out one-touch keys that are waiting for their third hit?
- Would routing evictions differently (where demoted or cascaded items go) beat any further admission tuning?
- Other stash retention controls: the depth at which demoted items land, and explicit protection for pre-credited items.
- Could MRU landing in the target bucket work if a separate hit count kept the bar for entering probing where it is?
- How well does this generalize? Every result so far is hit-rate only, on one LIRS trace at one cache size. Per-op-time and metadata-size have never been measured for any winner. Scan-heavy and larger-working-set behaviour, especially for MRU re-entry, is untested.

## Per-policy notes

- **Tiered stash / target / probing bucketed policy (the only lineage so far):**
  - The normal tier is the bottleneck. The stash works best as a large holding area plus a filter, and it has started to act as a victim cache for demoted items.
  - Item position and promotion thresholds are coupled through the "is hottest" check, so any change to placement also changes the effective promotion thresholds.
  - Most of the remaining hit-rate gains on this trace seem to be in stash retention and in eviction/cascade routing, not in the target→probing edge.
