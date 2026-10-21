## Recurring wins

- **Filter admission instead of adding protection.** From 0001 to 0008, nearly every winner made it harder to get into scarce space: first the normal tier (stash-hit threshold), then protected stash space (BIP, rarer BIP, no BIP, recency-gated ghost). Variants that widened access or added protection lost or came out flat.
- **Fresh keys face a harsh bar; proven ex-residents are protected.** This SLRU/2Q-style split ran through every winner from 0002 on: MRU stash re-entry for demoted items, LRU-end (probation) insertion for fresh keys, and MRU placement only when there is evidence of reuse.
- **Placement beats counters.** Moving a key changed hit-rate; crediting its counter did not. Ghosts failed as counter pre-credit (0004) and won once a ghost hit changed placement (0005).
- **Recent evidence beats more evidence.** A ghost window of about one cache helped, a 4× window hurt, and a recency gate at about half the window helped. The useful reuse horizon is shorter than one cache's worth of evictions.
- **When one selective admission path exists, cut the blind ones.** Making BIP rarer, then removing it, paid off each time after the ghost existed. Removal stacked with the recency gate, and it also simplifies the design.

## Recurring failures

- **Taking capacity from the protected ex-resident region.** Wider probation, set-dueling DIP and over-wide ghost windows all lost. Protected stash capacity is close to zero-sum.
- **Stacking retention mechanisms that protect the same items.** The CLOCK scan on top of MRU placement, scan-skip for ghost hits, and a threshold of 3 on top of MRU re-entry added nothing.
- **Counter or frequency gates where position already decides.** The frequency bar on target→probing, the TinyLFU-style duel, and ghost counter pre-credit were all flat, because Front landing couples frequency to position.
- **Free-slot placement and normal-tier MRU landing.** Normal buckets are almost always full, and MRU landing lowers the bar for entering probing.
- **Micro-tuning in-stash parameters.** Threshold, victim choice, gate cutoff and graded mid-stash placement now each move hit-rate by about noise.
- **Per-op-time hypotheses.** Cascade-free routing was tried twice and showed no hit-rate signal, once bit-identical to its parent. Per-op-time has no fitness dimension, so these hypotheses cannot be scored.
- **Process failure.** The continuations asked for more evaluation coverage every cycle, and it never happened. Every result comes from one LIRS trace at one cache size, scored on hit-rate only, and writers cannot inspect overlay diffs.

## Open questions

- Do the gains (BIP and its removal, gated ghost) hold on other traces and cache sizes, especially workloads with useful reuse beyond the ghost horizon?
- Can the ghost table shrink to the gated horizon without losing hit-rate? This is the main metadata lever.
- Do metadata-size and per-op-time (ghost table, flags) justify the small recent gains? Neither has ever been measured.
- Does the cascade/displacement path fire at all at this size? Did some null variants (0003 v001, 0007 v003) actually reach the evaluated build?
- Are structural levers (the stash-to-normal slot ratio, or who reaches the stash at all) the remaining source of hit-rate now that in-stash levers look exhausted?

## Per-policy notes

- **Stash→target admission:** a threshold of 2 to 3 is near optimal. The edge is cheap to filter because the stash is larger than the normal tier.
- **Target→probing edge:** admission gating does nothing here, because Front landing already filters it. Leave it alone.
- **Normal tier:** the bucket is the unit that fills, and evictions cascade into neighbours. That damage stopped mattering for hit-rate once the stash recovers demoted items cheaply.
- **Stash (victim cache):** MRU re-entry for ex-residents, LRU-end insertion for fresh keys, MRU only on a recent ghost hit, no BIP. The insert-position knobs are tuned out on this trace.
- **Ghost history:** it works only when a hit changes placement, the window is at most about one cache, and the hit is recency-gated. Graded placement and tighter cutoffs are on a plateau.
- **BIP/DIP:** it was useful as a bridge before the ghost existed, and its optimum is now "off". Adaptive DIP lost, but its duel signal was noisy.
