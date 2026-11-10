# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-11-10T04:28:05+00:00

## timeline

### 04:28:23 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Changing one thing per variant is what has produced the real gains. Each time changes were bundled (cycles 1–2), the result was a trade-off between hm_0 and web_0. Once changes were isolated (cycle 3 onward), wins appeared where neither trace regressed.\n- Main should only credit reuse that happens inside main. Two filters do this: ghosting a main victim only if it was hit while resident, and resetting freq/reuse when a key graduates from small to main. They each won in cycle 3 and added up cleanly when merged in cycle 4. That merged design is still the core of the parent.\n- Spend the ghost budget selectively, not by making the ghost bigger. Filters that leave stale entries out of the ghost lengthen the useful history at no cost.\n- Structural changes that leave hit rate untouched can earn real timing credit: slab plus hash indexes (cycle 2) and the Fx hasher (cycle 3). These are the only per-op-time gains the diffs themselves explain.\n- Small main-retention knobs pay off mainly on hm_0 at the small cache size. The MAX_FREQ 7 parent from cycle 5 was a slight gain, and nothing since has replaced it.\n\n## Recurring failures\n\n- Resizing the small queue in either direction does not help. Fixed ratios (cycle 1), shrinking to 10% (cycle 5) and growing to 1/3 (cycle 6) all lost. The 1/4 share is a local optimum.\n- Adaptive ARC-style splitting regressed in cycle 1.\n- A larger or re-split ghost does not help. Region splits (cycle 2) and a ghost sized to full capacity (cycle 4) let stale reuse back into main, where it pushes out live residents, mostly on web_0.\n- Loosening graduation or admission keeps costing web_0. Examples are the `freq >= 1` threshold (cycle 2), an extra reinsertion inside small (cycle 6), and giving freq credit to keys re-admitted from the ghost (cycle 6).\n- Index-layout work on probe count or load factor made tails worse. This covers merging the indexes into one map (cycle 4) and pre-sizing indexes to 2x occupancy (cycle 5). Memory footprint and locality matter more than the number of lookups.\n- An eviction loop whose work per miss is unbounded (multi-pop, cycle 6) inflates the `put` p99.\n- Timing shifts in policy-only diffs have been noise in every cycle. The per-entry metadata metric stayed flat throughout and never reflected ghost or index heap growth.\n\n## Open questions\n\n- web_0 at the large size is the last persistent gap. Probation length, small share and ghost bound have all been ruled out. The remaining lead is the rule for admitting keys from the ghost into main. A stricter rule (for example, re-admitting small-demotee ghost hits to small instead of main) is untested.\n- hm_0 at the small size is stuck under changes that only act on ghost hits. Two untested levers act on every main eviction: the main-eviction sweep itself, and the graduation threshold (`freq > 1`).\n- Could cycle-6 v003's large-size hm_0 gain be kept without its web_0 cost, for example by giving ghost-hit credit only when the ghost hit comes soon after eviction?\n- Making the ghost index denser (fingerprints or compact keys) to cut the `put` tail is still untested. So is a compact unified slot (a `u32` index plus a flag bit).\n- Should heap bytes per resident key be added alongside the metadata metric? And is a same-environment baseline needed before any timing credit?\n\n## Per-policy notes\n\n- S3-FIFO is the only policy in the ledger so far. The current parent is: Fx slab+hash, reuse-gated main-victim ghost with bound ≤ `main_size`, reset on graduation, `freq > 1` graduation threshold, small share 1/4, MAX_FREQ 7.\n- On S3-FIFO, hm_0 responds to anything that strengthens main retention or main-victim history, and most of its remaining headroom is at the small cache size. Main retention looks nearly saturated at the large size.\n- On S3-FIFO, web_0 is hurt by any loosening of graduation or ghost→main admission, and its large-size gap has resisted every structural change tried so far.\n- On S3-FIFO, the ghost already serves as the second chance for demoted keys, so extra rounds inside small duplicate it and push out newcomers.\n"
}
```

### 04:28:29 thinking



### 04:28:29 chat

I wrote `cross-cycle-insights.md` from the six-cycle ledger, using the four fixed sections in order. It contains no numbers.

- **Recurring wins:** changing one thing per variant is what produced the real gains, and the two filters that only credit reuse inside main added up cleanly when merged. The ghost works better when kept selective rather than made bigger. Timing credit has only gone to structural changes: slab plus hash indexes, and the Fx hasher.
- **Recurring failures:** resizing the small queue lost in both directions, so the 1/4 share looks like a local optimum. Growing or re-splitting the ghost didn't help, and loosening graduation or ghost-to-main admission kept costing web_0. Index-layout changes made latency tails worse, and so did eviction loops with unbounded work per miss.
- **Open questions:**
  - web_0 at the large size is the last persistent gap; the main untested lead is a stricter rule for re-admitting keys from the ghost into main.
  - hm_0 at the small size might respond to levers that act on every main eviction.
  - Making the ghost index smaller and denser is still untested as a fix for `put` latency tails.
  - The metadata metric doesn't capture ghost or index memory growth, and timing needs a baseline measured on the same machine before it can be trusted.
- **Per-policy notes:** S3-FIFO is the only policy so far. This section records the current parent configuration and how each trace tends to respond to changes.


---

- finished: 2026-11-10T04:28:29+00:00
- duration: 24.69s
- status: ok
- input_tokens: 4
- output_tokens: 2140
- cache_read_tokens: 40001
- cache_write_tokens: 11685
- cost_usd: 0.145014
