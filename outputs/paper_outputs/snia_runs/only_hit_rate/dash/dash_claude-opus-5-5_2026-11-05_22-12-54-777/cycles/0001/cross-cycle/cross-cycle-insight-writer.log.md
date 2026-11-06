# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0001/cross-cycle
- allow_edits: True
- started: 2026-11-05T22:19:03+00:00

## timeline

### 22:19:06 thinking



### 22:19:15 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0001/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\nThe ledger has one cycle (0001). Each pattern below rests on that single cohort and is provisional until later cycles confirm it.\n\n## Recurring wins\n\n- Strengthening the protected tier (target/probing) gives the most reliable hit-rate gains. All three cycle-0001 variants attacked this weakness from a different side, and all three found headroom.\n- Rebalancing the topology toward a larger normal tier and a smaller stash (v002, a 3/4 normal split) was the cleanest win. It improved hit-rate on both traces and both sizes, with no regression and no new mechanism.\n- Each lever helps at a different cache size. Promotion placement helps most at the small size, topology at the large size, and the frequency-aware tail victim at both. So the levers look complementary rather than redundant.\n\n## Recurring failures\n\n- Topology-only changes at the small cache size do little, because the cache is too tight for the split alone to protect promoted items.\n- When one lever changes alone, the other levers react. MRU promotion landing (v001) made the probing trigger fire too readily and penalised demoted items on web_0 at the large size. Expect similar interactions when changes are stacked.\n- Variants have measured only hit-rate. No metadata-size or per-op-time data exists yet, so the cost side of every win is unknown.\n\n## Open questions\n\n- Do v002's topology, v001's MRU promotion and v003's frequency-aware victim add up when stacked, or do they compete for the same gain?\n- Does the normal/stash split keep improving past 3/4 (e.g. toward 7/8)? At some point the stash should stop filtering scans; where is that point?\n- What per-op-time cost do the eviction-window scan and the bucket-wide counter halving add?\n- Would a stricter probing-promotion condition, or a softer demotion position, remove the MRU-promotion regression on web_0 at the large size?\n- Should stash hits count toward the frequency filter?\n\n## Per-policy notes\n\n- Baseline two-tier policy (normal/stash, target/probing): its protected tier barely protects anything. Promoted items are either evicted quickly when they land at the front, or crowded out of a small normal tier.\n- v002 (topology 3/4 normal): current Winner and the recommended parent for stacking.\n- v001 (MRU promotion/demotion placement): strong at the small size. It needs a gated probing trigger before it is safe at the large size.\n- v003 (frequency-aware LRU-tail victim): gains at both sizes, larger on hm_0. Its per-op cost has not been measured.\n- Traces: hm_0 responds to changes more readily than web_0. web_0 is where interaction regressions have appeared.\n"
}
```

### 22:19:21 thinking



### 22:19:21 chat

I wrote `cross-cycle-insights.md` in the output directory with the four required sections in order. The ledger has only one cycle so far, so the file says up front that every "recurring" pattern is provisional until later cycles confirm it.

- **Recurring wins:** The changes that pay off all make the protected tier (target/probing) actually protect promoted items. Shifting the split toward a larger normal tier and a smaller stash (v002) was the cleanest win. The three levers seem to help at different cache sizes, so they look complementary rather than redundant.
- **Recurring failures:** Changing only the topology does little at the small cache size. Changing one lever moves the others: MRU promotion made the probing trigger fire too easily and hurt web_0 at the large size. So far only hit-rate has been measured, with nothing on metadata size or per-op time.
- **Open questions:** whether the three levers add up when stacked; whether a split beyond 3/4 normal keeps improving; what the frequency filter costs per operation; whether a stricter probing trigger removes the web_0 regression; and whether stash hits should count toward the frequency filter.
- **Per-policy notes:** one short entry each for the baseline, v001, v002 (the Winner and suggested parent) and v003, plus a note on the two traces: hm_0 responds more readily, and web_0 is where the regression appeared.

All figures are given qualitatively. The only exceptions are the 3/4 and 7/8 split ratios, which come straight from the ledger.


---

- finished: 2026-11-05T22:19:21+00:00
- duration: 18.34s
- status: ok
- input_tokens: 4
- output_tokens: 1639
- cache_read_tokens: 34528
- cache_write_tokens: 6968
- cost_usd: 0.095446
