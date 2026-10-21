## Recurring wins

- Asymmetric SLRU/2Q-style split: fresh keys face a harsh bar, proven ex-residents get a cheap path back and are protected. Every winner since 0001 is a version of this: stash-hit admission filter (0001), MRU stash re-entry for demoted items (0002), BIP fresh-key insertion (0004), ghost-guided MRU placement (0005).
- Placement/position beats counter credit. Insert position of fresh stash keys (0004) and ghost hits that change placement (0005) moved hit-rate. Counter-only pre-credit and frequency tweaks did not.
- The stash works best as a two-part structure: a narrow probation region that one-touch keys churn through, and a protected victim cache for pre-credited ex-residents. Leave the protected region intact and admit fresh keys into it only when there is evidence they return, e.g. a ghost hit within a window about one cache in size.
- Stricter admission at the stash→target edge helped early on (thresholds 2 then 3). Returns flattened by 0002–0003.

## Recurring failures

- Anything that takes protected capacity away from ex-residents: wider probation (0005), set-dueling DIP restoring MRU insertion (0005), and MRU landing in the normal tier that lowers the bar for probing (0001). Stash capacity is close to zero-sum on this trace.
- Frequency gating that reads what position already encodes: a target→probing hit bar (0002), and a TinyLFU duel against an LRU-end resident that is almost always counter-0 because of Front landing (0004).
- Counter-only pre-credit for keys that never hit while resident, e.g. short per-bucket ghosts that credit the counter (0004).
- Mechanisms that only take effect when a slot is free (0001 spill), since normal buckets are almost always full together.
- Stacking retention mechanisms that protect the same items: a stash victim scan, threshold 3 on top of MRU re-entry, or cascade rerouting all came out null (0003).
- Process failure: every cycle so far was scored on hit-rate only, on one LIRS trace at one cache size. Each continuation asked for more coverage and none was done. Overlay application was never verified (0003 v001 matched its parent exactly).

## Open questions

- Per-op-time and metadata-size for the winner chain, which are now urgent because the ghost table adds metadata. A related question is whether cascade-free routing (0003 v003) wins on per-op-time.
- Do BIP and the ghost gains generalise to other traces and cache sizes, or do they overfit LIRS loops?
- Ghost tuning: a larger window, or a ghost hit that also protects the key from the next scan pass. Also tuning `BIP_PERIOD`, the only fresh-key survival knob that takes no protected capacity.
- Structural capacity levers: the stash-to-normal slot ratio and stash size.
- Are the remaining misses mostly true one-touch keys or keys with reuse distances beyond one cache's worth of evictions? This sets the ceiling for further hit-rate work.

## Per-policy notes

- Normal tier (target/probing, Front landing): position and frequency are coupled, and the "is hottest" check reads position. Every eviction there sets off a cross-bucket demotion cascade. Since 0002, cheap stash recovery has made that cascade harmless for hit-rate.
- Stash: larger than the normal tier, and the stash design is where most of the hit-rate gains came from. Current lineage: two-hit admission filter, then MRU re-entry with pre-credit, then a second-chance scan (a noise-level winner), then BIP fresh insertion, then a ghost table about one cache in size with MRU placement on a ghost hit. Victim-choice tweaks were saturated by 0003.
- Target→probing edge: little left to gain through admission. Drop it.
- Adaptive insertion (DIP): it lost, but its miss signal was noisy and its leader groups were small, so this is weak evidence against it rather than proof.
