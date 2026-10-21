# Cycle 0004 Insight

## Cohort outcome

All three variants were built on cycle-0003 v002 (second-chance stash scan) and each tried a lever not tested before. v001 added non-resident ghost history with a pre-credit. v002 added a TinyLFU-style frequency duel at the stash→target edge. v003 changed where fresh keys land in the stash, using bimodal (BIP) insertion. Only v003 moved hit-rate; v001 and v002 were flat within noise. This breaks the plateau declared in cycle 0003.

## Patterns

- Insert position of *fresh* stash keys is a real lever, unlike normal-tier landing (0001) and stash victim choice (0003). It decides which item the scan reaches first. Placing fresh keys at the LRU end splits each stash bucket into a probation slot that one-touch keys churn through and a protected victim cache for ex-residents. The scan also stops aging credited items.
- Changing the promotion counter does nothing for keys that never get a hit while resident. Both nulls failed for this reason. Ghost pre-credit only lowers a bar that keys with long reuse distances never reach, and the per-bucket ghost window was far too short to cover them anyway.
- Comparative admission has nothing to work with while Front landing makes the normal-tier eviction candidate almost always a fresh arrival with counter 0. In this design, normal-tier frequency and position are coupled, so any frequency check reads what position already tells it.
- The winning pattern is still "fresh keys face a harsh bar, proven ex-residents are protected", the same SLRU/2Q split as in 0002. This cycle applied it with insert position instead of thresholds.
- This cycle ignored the 0003 continuation. It was hit-rate only, on the same single LIRS trace and cache size, with no per-op-time or metadata-size measurements.

## Open questions

- BIP's cost is that keys with moderate reuse distance must hit before the next insert into their bucket. Would a tuned BIP period or a probation area wider than one slot do better?
- Would a ghost window covering roughly cache-size evictions help if a ghost hit changed placement or residency (e.g. admitting straight to target or MRU) instead of the counter?
- Does BIP's scan resistance hold on other traces and sizes, or does it overfit LIRS loops?
- Are per-op-time and metadata-size still unmeasured for every winner, including 0003 v003's cascade-free routing?

## Continuation

Build on v003 (BIP fresh-key insertion). Probe its parameters, such as the BIP period and the probation depth. If ghosts are revisited, they need a much larger window and a ghost hit must change placement, not the counter. Drop counter-only pre-credit and frequency duels against the LRU-end resident, unless victim selection becomes frequency-based. Next cycle should finally measure per-op-time and metadata-size and run at least one more trace or cache size.
