# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-10-20T10:16:34+00:00

## timeline

### 10:16:41 thinking



### 10:16:45 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Constant-factor work that leaves eviction order alone is the most reliable source of gains: skipping relinks at the MRU tail, hashing each key once, and trimming pointer work on repeated accesses to the newest key all cut per-op cost without moving hit rate.\n- Metadata reduction is decoupled from policy. Packing protected state into existing node fields shrank overhead without changing hit rate, so bookkeeping can be trimmed independently of the eviction logic.\n- Counter-index fidelity to the public sketch path matters for admission. Restoring the public per-row seeded indexing recovered hit rate, which suggests the admission gate is sensitive to exact sketch behaviour.\n\n## Recurring failures\n\n- Recency rotation of rejected admission victims lost hit rate and added per-op cost. The frequency gate already does the protective work, so recency adds cost without improving admission.\n- Capping the protected segment without demotion lost hit rate. The segmentation benefit appears to depend on the protected set turning over, so caps are the wrong lever.\n- Shortening frequency aging lost hit rate. Stale counts do not appear to be what limits admission.\n- A one-hit doorkeeper in front of the sketch barely moved hit rate and mostly added state and per-put work. One-hit pollution does not appear to be the bottleneck on this workload.\n- Changes that look like speedups or regressions are often attributed to helpers or accessors without verified source inspection. Treat attributions as hypotheses until confirmed.\n\n## Open questions\n\n- Does the marginal hit-rate gain from protected/probation segmentation hold on any trace other than lirs workload-a? Is its latency drop real or measurement variance?\n- Can the seeded per-row sketch indexing be made cheaper while keeping identical counter indices, so the fidelity gain survives without the speed loss?\n- Are the precomputed-hash helpers index-equivalent to the public sketch methods? Resolving this decides whether the earlier hit-rate drop was a bug or a real trade-off.\n- How much of the eviction-order-preserving savings depends on the share of accesses that hit the newest key?\n- Is the doorkeeper's memory cost fully reported, and do any of these effects appear on a trace where one-hit keys dominate?\n- Do any results survive repeat runs and a second trace? All evidence so far comes from one trace.\n\n## Per-policy notes\n\n- TinyLFU-style admission (frequency sketch and gate): hit rate is sensitive to sketch fidelity and is not limited by stale counts or one-hit pollution on the tested trace. Preserve indexing semantics; avoid shortening aging; avoid adding a doorkeeper unless a one-hit-heavy trace shows an effect.\n- Rejected-victim handling: recency rotation of rejected victims is a dead direction.\n- Segmented (probation/protected) resident list: the only source of hit-rate movement seen so far, but small, metadata-costly, and dependent on unbounded turnover of the protected set. Re-test on another trace before building on it.\n- MRU-tail and newest-key paths: safe targets for constant-factor speedups that keep eviction order unchanged.\n"
}
```

### 10:16:48 chat

I wrote `cross-cycle-insights.md` in the cycle 0003 cross-cycle output directory. It has the four required sections in order, built from the three cycles in the ledger digest.

- **Recurring wins:** constant-factor changes that keep eviction order, decoupling metadata reduction from hit rate, and matching the public sketch indexing for admission fidelity.
- **Recurring failures:** recency rotation of rejected victims, capping the protected segment, shortening frequency aging, and the one-hit doorkeeper. I also flagged that several speedup and regression attributions were never verified against source.
- **Open questions:** whether segmentation's gain and latency drop hold on other traces or are variance, whether the seeded indexing can be made cheaper, whether the hash helpers are index-equivalent to the public path, and whether any result survives repeat runs or a second trace.
- **Per-policy notes:** grouped under admission, rejected-victim handling, segmented resident list, and MRU-tail/newest-key paths.

The digest has no numbers I could cite, so the file describes effects qualitatively. I did not read any previous version of the file. The file has not been committed.


---

- finished: 2026-10-20T10:16:49+00:00
- duration: 14.70s
- status: ok
- input_tokens: 4
- output_tokens: 2695
- cache_read_tokens: 35691
- cache_write_tokens: 8551
- cost_usd: 0.003415
