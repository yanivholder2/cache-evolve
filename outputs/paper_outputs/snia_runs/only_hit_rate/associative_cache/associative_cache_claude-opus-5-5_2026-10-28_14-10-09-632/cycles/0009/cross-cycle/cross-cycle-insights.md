# Cross-Cycle Insights (through cycle 0009)

## Recurring wins

- **Tighter admission to the protected/main region pays off almost every time.** Changes that worked: cold insertion (0001), the probation FIFO in front of GCLOCK (0004), the probation-depth floor (0005), reset-to-0 on promotion (0005–0006), and the correlated-reference promotion filter (0008). The only promotion signal that has held up is a real, non-burst hit while the item is in probation.
- **The GCLOCK + probation lineage has led since 0004.** It is the first design to help both traces without a trade-off, and the only one that has moved the large-bucket cells.
- **Reset-to-0 on promotion keeps this lineage safe.** It kept higher ceilings, recency decay and grace bits from trapping stale web_0 blocks. Main always has cold victims, so there is always room.
- **Improving how main ranks its residents keeps helping. Making main bigger or easier to enter does not.** Ceiling 7 (0006), recent-bit + halving (0007), veteran grace (0008) and size-gated linear decay (0008–0009) each helped without regressions.
- **Size-gated rules beat one rule for all sizes.** Small buckets want fine linear counter tiers and full main capacity. Large buckets want halving decay and benefit from filtering and grace. Most levers act at only one size, so size them in absolute slots per region.
- **Changes on separate paths stack.** Linear decay gained more once the filter was in place. Changes on the same path, such as grace and the filter both targeting counter-0 promotions, only partly stack. Combining them still costs nothing.

## Recurring failures

- **Fixed-fraction probation in 128-slot buckets is too short for web_0's late second touches.** This has held in every lineage. A floor fixed part of it, and going deeper only took slots from main.
- **Probation depth is used up as a lever.** Going beyond floor 64 in small buckets or beyond a quarter in large buckets regressed in 0006 and again in 0007. It is a local optimum.
- **Ghost/warm admission failed in both lineages, whether the ghost was undersized or main-sized (0003, 0005).** It brings back hm_0's near-misses but re-admits web_0's single-return blocks, so this is a trace split, not a sizing problem.
- **Non-zero promotion counters and carried credit hurt (0005).** The item's position behind the hand already gives it a rotation of grace, so extra credit only adds churn.
- **Admission gates keyed on "main is hot" never fire (0007).** Reset-to-0 means main is never saturated with hot blocks.
- **The LRU/BIP line stopped making progress after 0002.** PSEL feedback was miscalibrated or undersized (0003–0004), stacking the floor on BIP was zero-sum, and graded promotion through a single slot cut off the reuse stream. Every fixed rule traded hm_0 against web_0.
- **Short per-bucket memories sized below the reuse distance they target lose their signal before it is useful (0003).**
- **Widening the correlated-reference window to an absolute 16 slots in small buckets filters hm_0's real reuse (0009).**
- **Per-op-time and metadata-size have never been measured in nine cycles, because the harness's fitness vector does not include them.** Sweep-cost changes (ceiling 7, linear decay in small buckets) remain unseen.

## Open questions

- **What limits hm_0 at the large size?** Recency, victim ordering, promotion filtering and counter resolution each moved it only a little. The likely cause is long-distance reuse that probation never sees, which needs a new lever outside main ranking.
- **Does the veteran grace bit stack with linear decay?** The expectation is that the large-size gain survives and the small-size gain may vanish.
- **Is there a small-bucket correlated window strictly between the current one and a quarter of the ring?** At most one more test is warranted.
- **Can near-miss history be used for something other than admission?** One example is a tie-break among probation evictions.
- **What still limits web_0 at the large size, now that bursts are filtered?**
- **Does any cost (sweep length, scans) offset the hit-rate gains?** This can only be answered once the harness measures per-op-time.

## Per-policy notes

- **LRU + LIP/BIP (0001–0004, retired):** Cold insertion won on its own. Every addition after that (BIP period, PSEL, floor, graded promotion) traded the traces against each other. hm_0 wants a large protected region and web_0 wants deep probation, and with fixed bucket space you cannot give both.
- **CLOCK / GCLOCK alone (0001–0003):** Coarser recency cost no hit-rate. A 2-bit GCLOCK improved every cell, but ghost admission on top of it polluted the cache.
- **GCLOCK + probation FIFO (0004–present, main lineage).** The current parent is cycle 0009 v001. It has ceiling 7, a recent bit, halving in large main regions and linear decay in small ones, a 64-slot probation floor with a quarter fraction in large buckets, the correlated-reference filter, and reset-to-0. Main needs fewer slots than an LRU protected segment, but small buckets still depend on main's capacity and fine tiers. Retired here: depth sweeps, ghost admission, non-zero promotion credit, higher ceilings, hot-main gates.
- **Traces:** hm_0 has long-distance reuse and is sensitive to main capacity and protection. web_0 is scan- and burst-heavy and responds to filtering promotions and clearing stale blocks. It also gains from main-side history more than expected.
