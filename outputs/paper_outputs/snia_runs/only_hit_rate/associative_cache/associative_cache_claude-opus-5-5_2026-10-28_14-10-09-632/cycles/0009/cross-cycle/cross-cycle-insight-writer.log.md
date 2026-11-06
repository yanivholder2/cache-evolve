# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-28T18:11:14+00:00

## timeline

### 18:11:35 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0009)\n\n## Recurring wins\n\n- **Tighter admission to the protected/main region pays off almost every time.** Changes that worked: cold insertion (0001), the probation FIFO in front of GCLOCK (0004), the probation-depth floor (0005), reset-to-0 on promotion (0005–0006), and the correlated-reference promotion filter (0008). The only promotion signal that has held up is a real, non-burst hit while the item is in probation.\n- **The GCLOCK + probation lineage has led since 0004.** It is the first design to help both traces without a trade-off, and the only one that has moved the large-bucket cells.\n- **Reset-to-0 on promotion keeps this lineage safe.** It kept higher ceilings, recency decay and grace bits from trapping stale web_0 blocks. Main always has cold victims, so there is always room.\n- **Improving how main ranks its residents keeps helping. Making main bigger or easier to enter does not.** Ceiling 7 (0006), recent-bit + halving (0007), veteran grace (0008) and size-gated linear decay (0008–0009) each helped without regressions.\n- **Size-gated rules beat one rule for all sizes.** Small buckets want fine linear counter tiers and full main capacity. Large buckets want halving decay and benefit from filtering and grace. Most levers act at only one size, so size them in absolute slots per region.\n- **Changes on separate paths stack.** Linear decay gained more once the filter was in place. Changes on the same path, such as grace and the filter both targeting counter-0 promotions, only partly stack. Combining them still costs nothing.\n\n## Recurring failures\n\n- **Fixed-fraction probation in 128-slot buckets is too short for web_0's late second touches.** This has held in every lineage. A floor fixed part of it, and going deeper only took slots from main.\n- **Probation depth is used up as a lever.** Going beyond floor 64 in small buckets or beyond a quarter in large buckets regressed in 0006 and again in 0007. It is a local optimum.\n- **Ghost/warm admission failed in both lineages, whether the ghost was undersized or main-sized (0003, 0005).** It brings back hm_0's near-misses but re-admits web_0's single-return blocks, so this is a trace split, not a sizing problem.\n- **Non-zero promotion counters and carried credit hurt (0005).** The item's position behind the hand already gives it a rotation of grace, so extra credit only adds churn.\n- **Admission gates keyed on \"main is hot\" never fire (0007).** Reset-to-0 means main is never saturated with hot blocks.\n- **The LRU/BIP line stopped making progress after 0002.** PSEL feedback was miscalibrated or undersized (0003–0004), stacking the floor on BIP was zero-sum, and graded promotion through a single slot cut off the reuse stream. Every fixed rule traded hm_0 against web_0.\n- **Short per-bucket memories sized below the reuse distance they target lose their signal before it is useful (0003).**\n- **Widening the correlated-reference window to an absolute 16 slots in small buckets filters hm_0's real reuse (0009).**\n- **Per-op-time and metadata-size have never been measured in nine cycles, because the harness's fitness vector does not include them.** Sweep-cost changes (ceiling 7, linear decay in small buckets) remain unseen.\n\n## Open questions\n\n- **What limits hm_0 at the large size?** Recency, victim ordering, promotion filtering and counter resolution each moved it only a little. The likely cause is long-distance reuse that probation never sees, which needs a new lever outside main ranking.\n- **Does the veteran grace bit stack with linear decay?** The expectation is that the large-size gain survives and the small-size gain may vanish.\n- **Is there a small-bucket correlated window strictly between the current one and a quarter of the ring?** At most one more test is warranted.\n- **Can near-miss history be used for something other than admission?** One example is a tie-break among probation evictions.\n- **What still limits web_0 at the large size, now that bursts are filtered?**\n- **Does any cost (sweep length, scans) offset the hit-rate gains?** This can only be answered once the harness measures per-op-time.\n\n## Per-policy notes\n\n- **LRU + LIP/BIP (0001–0004, retired):** Cold insertion won on its own. Every addition after that (BIP period, PSEL, floor, graded promotion) traded the traces against each other. hm_0 wants a large protected region and web_0 wants deep probation, and with fixed bucket space you cannot give both.\n- **CLOCK / GCLOCK alone (0001–0003):** Coarser recency cost no hit-rate. A 2-bit GCLOCK improved every cell, but ghost admission on top of it polluted the cache.\n- **GCLOCK + probation FIFO (0004–present, main lineage).** The current parent is cycle 0009 v001. It has ceiling 7, a recent bit, halving in large main regions and linear decay in small ones, a 64-slot probation floor with a quarter fraction in large buckets, the correlated-reference filter, and reset-to-0. Main needs fewer slots than an LRU protected segment, but small buckets still depend on main's capacity and fine tiers. Retired here: depth sweeps, ghost admission, non-zero promotion credit, higher ceilings, hot-main gates.\n- **Traces:** hm_0 has long-distance reuse and is sensitive to main capacity and protection. web_0 is scan- and burst-heavy and responds to filtering promotions and clearing stale blocks. It also gains from main-side history more than expected.\n"
}
```

### 18:11:42 chat

I wrote `cross-cycle-insights.md` from the ledger digest for cycles 0001–0009. It has the four required sections in order, and it is written fresh from the digest, without reading any earlier version.

**Recurring wins:**
- Making it harder to get into main has paid off almost every time. Cold insertion, the probation buffer, the depth floor, resetting counters to 0 on promotion, and the burst filter all worked.
- The GCLOCK + probation line has led since cycle 0004.
- Ranking inside main kept improving results; making main bigger or easier to enter did not.
- Rules set separately for small and large buckets beat one rule for both sizes.

**Recurring failures:**
- Probation depth is used up as a lever.
- Ghost/warm admission failed in both lineages.
- Non-zero promotion credit hurt.
- Gates that only fire when main is "hot" never trigger.
- The LRU/BIP line kept trading the two traces against each other.
- Small per-bucket memories forgot items before they gave a useful signal.
- An absolute 16-slot burst window filtered out hm_0's real reuse.
- Per-op-time and metadata-size have never been measured in nine cycles, because the test harness doesn't record them.

**Open questions:**
- What still limits hm_0 in large buckets?
- Does the veteran grace bit stack with linear decay?
- Is one middle burst-window width in small buckets worth one more test?
- Can near-miss history be used for something other than admission?
- What still limits web_0 in large buckets?
- Do the cost increases offset the hit-rate gains? That can't be answered until per-op-time is measured.

**Per-policy notes** cover the retired LRU line, CLOCK/GCLOCK on its own, the current main line (parent: cycle 0009 v001) with its list of retired ideas, and how hm_0 and web_0 behave differently.


---

- finished: 2026-10-28T18:11:43+00:00
- duration: 28.95s
- status: ok
- input_tokens: 4
- output_tokens: 2690
- cache_read_tokens: 42969
- cache_write_tokens: 16354
- cost_usd: 0.193242
