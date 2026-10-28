# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-28T15:19:21+00:00

## timeline

### 15:19:45 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **Cold insertion / probation is the base lever.** New items placed below reused ones won in cycle 0001 (LIP into LRU). It won again, and more strongly, once moved onto CLOCK as an S3-FIFO-style probation FIFO in cycle 0004. That is the first design that helped both traces without trading one for the other.\n- **GCLOCK main + probation FIFO is the dominant lineage (0004→0007).** It has improved every cycle since it took over. It filters scans on web_0 and keeps hm_0's reused blocks. It does no element shifting, and it scales better to large buckets than the LRU line did.\n- **Promote only on a hit during probation, and reset to 0 on promotion.** This is the lineage's safety property. It stopped carried-counter and ghost admission from hurting more (0005). It also kept a higher counter ceiling (0006) and recency decay (0007) from trapping stale blocks, and it means main always has cold victims.\n- **Main-side ranking quality beats main size or admission changes.** Gains on the main side came from how main ranks residents: frequency resolution (ceiling 7, cycle 0006) and recency-gated halving decay (cycle 0007). These are the only changes since 0004 that moved large-bucket cells, and they did not cause a web_0/hm_0 trade-off.\n- **A probation-depth floor in small buckets paid off once (0005).** It raised both traces at the small size without regressions, because a frequency-ranked main needs fewer slots than an LRU protected segment.\n- **Exact LRU order is not needed.** Coarser recency (CLOCK) has matched or beaten LRU on hit-rate from cycle 0001 onward.\n\n## Recurring failures\n\n- **Ghost / warm admission (0003, 0005).** It failed in two lineages and at two sizes. With enough coverage it brings back hm_0 near-miss reuse but re-admits web_0 single-return blocks, so the cause is a trace split, not ring size. Retired.\n- **Weakening the admission bar or adding counter mass to main.** Non-zero promotion counters (0005) and ghost credit raised churn in main and decayed established blocks, mostly on web_0. Retired.\n- **Sweeping probation depth past the bracket (0006, 0007).** Depth beyond about half the bucket in small buckets, or beyond a quarter in large buckets, takes capacity from main, and both traces need that capacity. 64 slots in small buckets is a local optimum. Closed.\n- **Per-bucket self-feedback on the LRU/BIP line (0002–0004).** Fixed BIP periods, PSEL with short sample rings, and PSEL with symmetric ±1 votes all traded hm_0 against web_0. The feedback state was undersized or miscalibrated, and rewarded hits that probation would have caught anyway.\n- **Stacking mechanisms that split the same fixed LRU bucket space.** The floor plus BIP (0003) and graded promotion through a single boundary slot (0004) failed this way. On LRU, probation and protected space are zero-sum.\n- **Weakened promotion with MRU insertion (0001)**, and **admission gates keyed on a \"hot main\" (0007).** Under reset-to-0 that state almost never happens: the variant gave results bit-identical to its parent.\n- **Short per-bucket memories sized below reuse distance.** This was the common cause of failure for rings and trackers in cycle 0003.\n- **Per-op-time and metadata size have never been measured (all 7 cycles).** This is a gap in the harness's fitness vector. Every latency and cost claim (mid-vector inserts, sweep bounds, ring scans) is still untested.\n\n## Open questions\n\n- Can small-bucket tier resolution be restored under the recency rule, for example by ranking on recent bit plus raw counter instead of bit length? The goal is to win back hm_0 at the small size without giving up web_0's large-size gain.\n- Main's victims are almost always fresh counter-0 promotions. Would choosing among counter-0 blocks by age or fresh-promotion status beat first-from-hand?\n- The remaining hm_0 large-size gap seems to sit outside main. Can near-miss history be used for something other than admission, such as a tie-break among probation evictions or as an input to the promotion decision?\n- What still limits web_0 at the small size? Depth, admission strength, counter resolution and hot-main gating have each been ruled out.\n- Does adaptive splitting via leader/follower set dueling on `small_len` add anything, now that the fixed depth is at a local optimum?\n- What are the actual per-op-time and metadata costs of the lineage, especially the promotion sweep at ceiling 7?\n- Was cycle 0007 v002 actually evaluated with its change applied? Its bit-identical result is worth a one-time check that the build was not reused.\n\n## Per-policy notes\n\n- **LRU + LIP / BIP (0001–0004).** Strong first gain from cold insertion, then stuck. Each refinement (probation floor, BIP period, PSEL, graded promotion) traded hm_0 against web_0, because probation and protected space compete for one fixed LRU ordering. hm_0 wants a large protected region and web_0 wants deep probation. Treat this line as retired unless a fundamentally different feedback design comes up.\n- **CLOCK / GCLOCK (0001–0003).** Cost no hit-rate compared with LRU, and the 2-bit counter improved every cell (0002), with the gain growing with bucket size. Ghost admission on plain GCLOCK failed (0003). This was the base for the current lineage.\n- **GCLOCK main + probation FIFO (0004–0007, current parent: 0007 v001).** Probation fraction is a quarter in large buckets with a floor of 64 in small buckets. Ceiling is 7, with a recency bit plus halving decay, and reset-to-0 on promotion. Small buckets depend on main's capacity and on how finely main ranks blocks, and lose whenever either one shrinks; coarser bit-length ranking cost hm_0 a little at the small size. Large buckets responded only to main-side ranking changes. Depth, admission strength, ceiling and hot-main gating are used up as levers; next directions are victim choice within counter-0 blocks and the promotion signal.\n- **Trace behavior.** web_0 is scan-heavy with late second touches and single-return blocks, so it is hurt by anything that loosens admission. hm_0 has long-distance reuse and benefits from protected or main capacity and near-miss retention. The levers that work help both traces by tightening admission while improving how main ranks residents.\n"
}
```

### 15:19:52 chat

I wrote a fresh `cross-cycle-insights.md` from the ledger for cycles 0001–0007. It uses the four required sections in the required order.

- **Main approach:** Since cycle 0004, the leading design has been CLOCK with frequency counters (GCLOCK) plus a separate probation queue that new items enter first. An item moves into the main region only if it is hit while in probation, and its counter resets to 0 when it does. Cycle 0007's variant v001 is the current base.
- **What keeps paying off:** Putting new items below reused ones, so a single reference can't push out reused data. Beyond that, the only changes that have improved the large buckets are ones that make main rank its residents better: a higher counter ceiling in cycle 0006 and recency-based decay in cycle 0007.
- **What has stopped paying off:**
  - Probation depth is used up; going deeper takes space that the main region needs.
  - Letting items into main more easily (ghost admission, non-zero starting counters) has failed repeatedly.
  - Gating admission on main being full of hot items changed nothing, because that case almost never happens.
  - The older LRU-based line kept trading one trace against the other and is treated as retired.
- **Gap across every cycle:** No cycle has ever measured per-operation time or metadata size, because the harness doesn't record them. Every cost claim is still untested.
- **Open questions:**
  - How to win back the small-bucket ranking detail lost in cycle 0007.
  - Which empty-counter item main should evict first.
  - Using near-miss history for something other than admission.
  - Whether cycle 0007 v002's bit-identical result means its change never actually ran.


---

- finished: 2026-10-28T15:19:53+00:00
- duration: 31.71s
- status: ok
- input_tokens: 4
- output_tokens: 2929
- cache_read_tokens: 41349
- cache_write_tokens: 13794
- cost_usd: 0.177218
