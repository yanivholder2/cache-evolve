# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-11-10T04:40:10+00:00

## timeline

### 04:40:28 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- One change per variant is what made progress possible (cycles 3–5). Every bundled change in cycles 1–2 just moved the trade-off between hm_0 and web_0 around. Isolated changes pinned down mechanisms and let gains stack.\n- Credit should come only from reuse inside main. Reuse-gated main-victim ghosting, resetting freq/reused at graduation, and keeping the `freq > 1` graduation threshold each helped alone (cycle 3) and added up when merged (cycle 4). This is still the core of the parent.\n- Spending the ghost budget selectively beats making it bigger. Filters that free ghost slots lengthen useful history at no cost.\n- Structural rewrites that leave hit rate bit-identical are the only trustworthy source of timing gains: slab plus hash indexes (cycle 2), then the Fx hasher (cycle 3).\n- Current parent: cycle-5 v002 (Fx slab, both filters, `MAX_FREQ` 7 with decrement-by-one aging, ghost bound = `main_size`, small share 1/4). It has survived three cycles of challengers.\n\n## Recurring failures\n\n- Resizing the small queue fails both ways: fixed ratios (cycle 1), shrinking (cycle 5), growing (cycle 6). The 1/4 share is a local optimum.\n- Changing the ghost bound or container hurts or does nothing. A larger bound lets stale reuse back into main (cycle 4). Separate ghost regions didn't help (cycle 2). A ring container (cycle 7) and fingerprint keys (cycle 8) both raised `put` cost.\n- Changing ghost→main admission fails both ways: looser credit (cycle 6) and stricter re-probation (cycle 7) both regressed web_0 at the large size.\n- Main-tenure knobs have hit their limit from both sides. Aging: halving hurt (cycle 7) and a cap of 15 hurt (cycle 8). Graduation credit: full carry-over and a credit of 1 both hurt. Tenure has to be earned in main.\n- Extra rounds inside small (cycle 6) duplicate what the ghost already does and push out newcomers.\n- Index layout tweaks (merged index, over-allocation) made tails worse. The `put` tail tracks hash computations and sweep steps per miss, not footprint or load factor.\n- Policy-only diffs keep showing timing shifts their diffs can't explain. These are environmental noise and never get timing credit. Eviction work per miss must stay bounded, because the multi-pop loops and cascades tried so far inflated the tail.\n\n## Open questions\n\n- Is a small/main split that depends on cache size, aimed at 16384, the last untested hit-rate lever? If it fails, treat hit rate in this policy family as converged.\n- Is web_0's large-size gap out of reach for this policy family? Probation length, small share, ghost bound and both admission directions have all been ruled out.\n- Would fingerprint ghosts with a pass-through hasher (one hash per miss) finally beat the parent on `put`? More broadly, can a hash probe or a sweep step be removed from the miss path?\n- Should the metadata metric add heap bytes per resident key? Ghost memory and index memory keep growing without the metric showing it.\n- The benchmark environment still isn't fixed, so small per-op-time differences can't be trusted.\n\n## Per-policy notes\n\n- S3-FIFO (the only policy explored so far):\n  - hm_0 responds to how main retains reused keys. web_0 is sensitive to probation and admission, and every change aimed at its large-size cell has failed.\n  - At 131072, hit rate has been flat under every policy change since cycle 5, so main retention there is saturated. All remaining hit-rate room is at 16384, mostly on hm_0.\n  - Per-entry metadata has stayed flat at 24 bytes across all cycles. New bits fit in padding or tag space.\n"
}
```

### 04:40:34 thinking



### 04:40:34 chat

I've written `cross-cycle-insights.md` with the four required sections in order, built only from the eight-cycle ledger digest.

- **Recurring wins:** Making one change per variant is what let gains stack from cycle 3 on. The main policy finding is that keys should earn frequency credit only from reuse while in main. The only timing gains worth trusting came from rewrites that left hit rates unchanged. The parent is still cycle-5 v002, which has outlasted three cycles of challengers.
- **Recurring failures:** Several knobs have now failed in both directions: small-queue size, the ghost→main admission rule, freq aging and the cap, and credit at graduation. Changing the ghost's size or container hurt or did nothing. Layout tweaks to the hash index made the `put` tail worse. Timing shifts from policy-only changes are environmental noise.
- **Open questions:** On hit rate, the last untested lever is a small/main split that depends on cache size, aimed at the 16384 size. If it fails, hit rate for this policy can be treated as converged. Other threads: whether web_0's gap at the large size is reachable at all, fingerprint ghosts with a pass-through hasher, a heap-bytes-per-key metric, and the still-unpinned benchmark environment.
- **Per-policy notes:** S3-FIFO is the only policy explored so far. Hit rate at 131072 is saturated. The room left is at 16384, mostly on hm_0. Per-entry metadata has stayed at 24 bytes throughout.


---

- finished: 2026-11-10T04:40:35+00:00
- duration: 24.41s
- status: ok
- input_tokens: 4
- output_tokens: 1969
- cache_read_tokens: 41917
- cache_write_tokens: 13417
- cost_usd: 0.156026
