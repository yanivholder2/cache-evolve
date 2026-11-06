# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-11-05T22:25:49+00:00

## timeline

### 22:26:04 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- Strengthening the protected (normal/target) tier keeps paying off. Both Winners so far address the baseline's weak protection of promoted items, one through topology and one through eviction choice.\n- Making single, isolated changes on top of the previous Winner has produced clean, attributable gains. Both Winners were one-lever changes with no regression in any cell.\n- The 24 normal / 8 stash (about 3/4) split is a stable foundation. Later changes built on it without having to retune it.\n- Changes that act inside a bucket (choosing the frequency-aware victim at the LRU tail, deciding where a promoted item lands) are what move the small cache size. Topology changes mostly move the large size.\n- The frequency-aware tail victim held up across both cycles. It helped at both sizes on its own, and it added cleanly on top of the topology change.\n\n## Recurring failures\n\n- Topology-only changes at the small cache size: two cycles in a row they did little there, because the cache is too tight for the split alone to help.\n- Pushing the normal/stash ratio past 3/4. With admit-everything-to-stash, the stash is the only way into the protected tier, so shrinking it starves promotion on hm_0 before scan filtering on web_0 gets worse. The ratio has peaked.\n- MRU-end promotion, together with the trigger and demotion changes that came with it, regressed web_0 in both cycles (at the large size in cycle 1, a small cost in cycle 2). It helps hm_0 but does not generalise across traces.\n- Bundling several levers into one variant (cycle-2 v001) made the regressions impossible to attribute.\n- Cost has never been measured. Per-op-time and metadata-size have gone unmeasured for two cycles, even though the Winners add work to the eviction path.\n\n## Open questions\n\n- What do the eviction-window scan and the bucket-wide counter halving cost in per-op-time? Would a narrower window or a lower cap keep the hit-rate gain?\n- Does MRU promotion alone still add anything on top of the frequency filter, or does the filter already capture the small-size gain on hm_0?\n- Which piece of the MRU bundle causes the web_0 loss: mid-stash demotion or the hit-bit probing guard?\n- Could admission or stash-promotion changes (a longer stash wait, promotion on first re-reference) serve hm_0's long-gap reuse without growing the stash?\n\n## Per-policy notes\n\n- Baseline (normal/stash with target/probing tiers): the protected tier gives almost no protection by default. Promoted items are either evicted quickly when they land at the Front, or crowded out when the normal tier is small.\n- Topology variants: they help most at the large size and have peaked at about a 3/4 normal share. Treat the split as settled.\n- Frequency-aware LRU-tail victim (with stash hits counted): the current lineage Winner. It improved all trace/size cells and gained most at the small size. Its eviction-path cost has not been measured.\n- MRU-end promotion variants: they change how readily the probing trigger fires and how demoted items are treated, and they show interaction effects with other levers. They are consistently good on hm_0 and fragile on web_0.\n- Traces: hm_0 is the most responsive trace in both directions, giving the largest gains and the largest regressions. web_0 is the one that catches over-aggressive promotion.\n"
}
```

### 22:26:11 thinking



### 22:26:11 chat

I've written `cross-cycle-insights.md` from the ledger digest for cycles 0001 and 0002. It has the four required sections in order, and it uses no numbers beyond the slot split the ledger itself names.

- **Recurring wins:** In both cycles, the winning change made the protected tier actually protect promoted items. Each Winner changed one thing on top of the previous Winner and hurt no trace/size cell. The 24 normal / 8 stash split now looks settled. Changes inside a bucket, such as the frequency-aware choice of which item to evict at the LRU tail, are what help the small cache size.
- **Recurring failures:**
  - Changing only the normal/stash split does nothing at the small size.
  - Shrinking the stash past 3/4 normal stops items reaching the protected tier on hm_0.
  - Promoting items to the MRU end (most recently used) keeps costing hit-rate on web_0.
  - Bundling several changes into one variant hides which one caused a regression.
  - Per-op-time and metadata-size have still not been measured in either cycle.
- **Open questions:**
  - What the Winner's eviction scan and counter halving cost in per-op-time.
  - Whether MRU promotion on its own still adds anything on top of the frequency filter.
  - Which part of the MRU bundle causes the web_0 loss.
  - Whether changes to admission or stash promotion could help hm_0's long-gap reuse.
- **Per-policy notes:** These are grouped by the baseline, topology variants, the frequency-aware victim (the current Winner), and MRU promotion, plus a note on how each trace behaves. hm_0 swings the most in both directions; web_0 is where over-aggressive promotion shows up.


---

- finished: 2026-11-05T22:26:11+00:00
- duration: 22.59s
- status: ok
- input_tokens: 4
- output_tokens: 1848
- cache_read_tokens: 36054
- cache_write_tokens: 7356
- cost_usd: 0.103035
