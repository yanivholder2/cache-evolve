# Cross-Cycle Insights (through cycle 0007)

## Recurring wins

- **Filter, don't protect.** Every winner has made it harder to get into scarce space (0001: stash-hit admission; 0004: BIP fresh-key insertion; 0006: rarer BIP; 0007: recency-gated ghost). No winner has added protection on top of the existing mechanisms.
- **Protected stash capacity is zero-sum.** From 0005 to 0007 the rule held four cycles running. Any slot taken from pre-credited or evidence-backed ex-residents costs at least as much as it gives back to fresh keys.
- **SLRU/2Q split.** Fresh keys face a harsh bar, while proven ex-residents return cheaply and are protected. This split came through in 0002 (MRU stash re-entry), 0004 (LRU-end probation) and 0005–0007 (ghost-guided MRU).
- **Placement beats counters.** Changing insert position or residency moves hit-rate. Changing promotion counters (pre-credit, frequency duels, ghost credit) does not (0004 null, 0005 win).
- **Recent evidence beats more evidence.** A ghost horizon of about one cache works (0005). A larger window lost (0006), and gating to about half a window won (0007).
- **Overlapping paths: the selective one wins.** Once the ghost existed, BIP's blind MRU path became a net cost (0006, 0007).

## Recurring failures

- **Loosening admission or adding capacity to probation.** MRU landing in normal buckets (0001), wider probation (0005) and set-dueling DIP (0005) all weakened the protected region and lost or broke even.
- **Stacked retention mechanisms that protect the same items.** These were tried as the CLOCK scan on top of MRU (0003), scan-skip for ghost hits (0006) and threshold 3 on top of MRU re-entry (0003). They add nothing because position and frequency protection overlap.
- **Counter-only tweaks.** Ghost pre-credit (0004), the TinyLFU duel against the LRU-end resident (0004) and target→probing frequency gating (0002) did nothing. Front landing already couples frequency to position.
- **Free-slot placement** (0001) rarely fires because the normal buckets are almost always full.
- **Cascade-free routing** (0003, 0007) left hit-rate flat, and in 0007 the result was bit-identical. Either it never fires at this size or it never applied. It can't be scored without a per-op-time dimension.
- **Long ghost windows.** Stale far-reuse hits displace residents with shorter reuse. Misses with reuse distance beyond about one cache can't be recovered through the stash.

## Open questions

- Do gated ghost (0007 v002) and BIP removal (0007 v001) stack? Both gained on their own, and they touch different paths.
- Where is the best ghost recency cutoff? Can the ghost table shrink to the gated horizon and save metadata?
- **Measurement gap (raised every cycle, never fixed):** per-op-time and metadata-size are unmeasured for every winner. Ghost, flag and epoch metadata keep growing.
- **Generalisation gap:** every result comes from one LIRS trace at one cache size. It is unknown whether BIP removal, the ghost horizon and the saturation findings carry over.
- **Overlay verification:** writers have never been able to inspect the overlay diff. Exact-parent matches (0003 v001, 0007 v003) may be changes that never applied. A displacement/cascade counter would help settle this.
- Structural capacity levers, such as the stash-to-normal slot ratio and stash size, are still untested.

## Per-policy notes

- **Normal tier (target/probing, Front landing):** admission matters and placement inside a bucket does not. The target→probing edge is already filtered by Front landing. Neighbour cascade damage is mostly absorbed now that the stash recovers demoted items cheaply.
- **Stash admission threshold:** 2 beats 1, and 3 is near the optimum or redundant once MRU re-entry exists. Don't push it higher.
- **Stash victim choice:** the second-chance scan gave only a noise-level gain. The lever is mostly exhausted.
- **Stash insertion (BIP):** LRU-end insertion of fresh keys was a real lever. Its blind MRU path has been tuned toward "off" now that the ghost exists.
- **Ghost table:** it pays only when a ghost hit changes placement (stash MRU), not the counter, and only for recent evictions (roughly half a cache to one cache). Scan protection for ghost hits is redundant.
- **Overall:** hit-rate gains on this trace have shrunk with each cycle. The stash path is close to tuned out, so the next real signal probably needs new traces or sizes, or new fitness dimensions.
