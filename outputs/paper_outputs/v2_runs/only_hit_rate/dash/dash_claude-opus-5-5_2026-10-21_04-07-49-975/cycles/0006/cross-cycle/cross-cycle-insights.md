# Cross-Cycle Insights (through cycle 0006)

## Recurring wins

- **Fresh keys face a harsh bar; proven ex-residents are protected.** Winners in 0001, 0002, 0004, 0005 and 0006 all used this SLRU/2Q-style split, applied first through admission thresholds, then through stash re-entry position, BIP insertion and ghost-guided placement.
- **Placement beats counters.** Where a key lands (MRU stash re-entry in 0002, BIP fresh insertion in 0004, ghost hits mapped to MRU placement in 0005) moved hit-rate. Credit or threshold tweaks on the same keys mostly did not.
- **Protected stash capacity is zero-sum.** Variants that added evidence-gated admission (ghost, 0005) or removed blind admission (rarer BIP, 0006) gained. The protected region for ex-residents should only be entered on evidence.
- **Evidence has to be timely.** Ghost history pays off with a window of about one cache's worth of evictions, and only when a ghost hit changes placement rather than the counter.
- **Selective paths outcompete blind ones.** Once an evidence-based route out of probation existed, the random route (BIP's MRU path) became a net cost.

## Recurring failures

- **Stacking retention mechanisms that protect the same items** (frequency plus position in 0003, scan-skip for ghost hits in 0006): MRU placement already covers those items, so the second mechanism adds nothing.
- **Counter-only credit or comparative frequency checks** (ghost pre-credit in 0004, the TinyLFU duel in 0004, target→probing gating in 0002): keys that never hit while resident don't benefit, and in the normal tier frequency is coupled to position, so these checks read nothing new.
- **Anything that takes protected capacity away from ex-residents** (wider probation in 0005, set-dueling DIP in 0005, the 4× ghost window in 0006) lost hit-rate or broke even.
- **Free-slot placement** (0001): normal buckets are almost always full, so it rarely fires.
- **Further stash victim-choice, routing and admission-threshold tweaks** were flat in 0003. Pushing the stash threshold past roughly 3 shows diminishing returns.
- **Long-reuse misses beyond about one cache** cannot be recovered by stash placement. Larger ghost windows let in weak evidence that displaces better residents.

## Open questions

- **Per-op-time and metadata-size have never been measured** after six cycles, while ghost and flag metadata keep accumulating. Cascade-free routing (0003) is still an untested per-op-time candidate.
- **Generality:** every result comes from one LIRS trace at one cache size. Do the BIP and ghost gains, and the saturation finding, hold elsewhere?
- **BIP endpoint:** does an even larger period help, or removing BIP entirely so the ghost is the sole exit from probation?
- **Sharper ghost evidence:** would a window smaller than one cache, or a recency-gated window, help?
- **Structural levers** (the stash-to-normal slot ratio, stash capacity) have not been explored.
- **Build fidelity:** writers have not been able to inspect overlay source. The 0003 v001 result was identical to its parent, so it is still unconfirmed that changes reach the evaluated build.

## Per-policy notes

- **Stash (victim cache):** the main lever across all cycles. It works best as a split, with a probation region for fresh keys at the LRU end and a protected MRU region for pre-credited ex-residents and ghost-evidenced keys. Victim choice inside the stash barely matters once placement is right. Gains have been shrinking since 0004, and the stash path looks close to tuned out on this trace.
- **Normal tier (target/probing):** admission into it was the first lever (two stash hits needed, 0001). Front landing already filters the target→probing edge. Normal-tier evictions cascade into neighbouring buckets, but cheap stash recovery made that cascade irrelevant to hit-rate.
- **BIP/DIP insertion:** BIP fresh-key insertion is a real win, and rarer blind MRU admission is better once ghosts exist. DIP set-dueling cost hit-rate on this trace, though its signal was noisy.
- **Ghost history:** it helps only with a window of about one cache and placement-based effects. Counter-based effects or oversized windows hurt or do nothing.
