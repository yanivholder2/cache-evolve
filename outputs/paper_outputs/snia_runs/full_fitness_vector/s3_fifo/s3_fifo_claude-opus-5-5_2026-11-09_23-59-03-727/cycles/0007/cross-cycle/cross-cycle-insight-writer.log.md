# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-11-10T04:34:27+00:00

## timeline

### 04:34:44 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **One change per variant.** Bundled changes in cycles 1–2 hid their causes and moved gains back and forth between hm_0 and web_0. Single-change variants (cycles 3 onward) produced the only real wins and made causes clear.\n- **Judge credit by reuse inside main.** Reuse-gated main-victim ghosting and resetting freq/reused at graduation each helped on its own (cycle 3) and added up when combined (cycle 4). That combination is the only change so far that improved every trace/size point.\n- **Spend ghost budget selectively, not generously.** Filters that free ghost slots lengthen the useful history at no cost. Selective filtering has beaten raising the bound every time.\n- **More tenure for strongly reused keys.** Raising MAX_FREQ (cycle 5) helped hm_0 at the small size. Faster aging (cycle 7) hurt, which points the same way from the other side.\n- **Timing gains come from structural changes that cut real work.** Slab + hash (cycle 2) and Fx hashing (cycle 3) are the only timing gains whose diffs explain them, and both kept hit rates bit-identical.\n\n## Recurring failures\n\n- **Resizing the small queue.** Fixed 10% (cycles 1 and 5) and 1/3 (cycle 6) both regressed. The 1/4 share is a local optimum.\n- **Adaptive small/main split (ARC-style).** It regressed in cycle 1 and has not been retried.\n- **Longer or larger ghost history.** Bound = capacity (cycle 4) let stale reuse displace live residents. Resizing or splitting ghost regions (cycles 1–2) only moved the hm_0/web_0 trade-off around.\n- **Changing ghost→main admission in either direction.** Looser admission (re-admit with credit, cycle 6) and stricter admission (send back to small, cycle 7) both hurt web_0 at the large size.\n- **Extra probation passes inside small.** Reinsertion of freq-1 keys (cycle 6) and the looser `freq >= 1` graduation threshold (cycle 2) duplicated the ghost path or let unreused keys into main.\n- **Faster freq aging in main.** Halving on each sweep evicted high-freq keys whose hits are far apart.\n- **Hash-table layout tweaks.** A merged index (cycle 4), 2x over-allocation (cycle 5) and a ghost ring with a liveness probe on every insert (cycle 7) all made the `put` tail worse. Memory footprint and probes per miss matter more than amortized cleanup.\n- **Unbounded eviction work per miss.** Multi-pop loops and eviction cascades reliably inflate `put` p99.\n- **Timing claims from policy-only diffs.** Every cycle showed timing drift its diff couldn't explain, so it is noise. Give no timing credit to policy-only diffs.\n\n## Open questions\n\n- Is web_0's large-size gap out of reach for the S3-FIFO family? Probation length, small share, ghost bound and both directions of ghost admission have all failed.\n- What moves hm_0 at the small size? Candidates: protecting keys at the freq cap, a size-dependent small/main split or MAX_FREQ, or levers that act on every main eviction rather than only on ghost hits.\n- Can the miss path drop a hash probe, for example with fingerprints, a denser ghost index, or ghost liveness tracked without a separate `ghost_index` lookup?\n- Should metadata reporting add heap bytes per resident key? Per-entry size has stayed flat while ghost and index heap grew unseen.\n- The benchmark environment is still not pinned. Same-machine baselines are needed before trusting small timing differences.\n\n## Per-policy notes\n\n- **S3-FIFO (sole policy so far):**\n  - The current best parent is cycle-5 v002: Fx slab + reuse-gated ghosting + reset at graduation + MAX_FREQ 7, with a 1/4 small share, a `freq > 1` threshold and the ghost bound at `main_size`. Cycles 6 and 7 produced nothing better.\n  - hm_0 responds to how long main retains keys. Gains at the large size look saturated, and what's left is a small-cache problem.\n  - web_0 at the large size is the stubborn cell. Every probation and ghost-handoff lever tested so far has regressed it.\n  - Per-entry metadata has stayed flat (24 B) in every cycle. New flag bits fit into padding or tag space.\n"
}
```

### 04:34:52 thinking



### 04:34:52 chat

I've written `cross-cycle-insights.md` in the cycle 0007 cross-cycle output dir. It uses the four required sections and draws only on the seven cycle digests.

- **Recurring wins:** making one change per variant is what finally worked. The best results came from giving a key credit only for reuse while it sits in main: reuse-gated ghosting plus a reset at graduation, the only change that improved every trace and size. Using ghost slots selectively beat making the ghost bigger, and giving heavily reused keys more time in main (MAX_FREQ) helped. Only structural changes have earned timing credit: slab + hash, then Fx hashing.
- **Recurring failures:** these keep losing:
  - resizing the small queue in either direction
  - the adaptive small/main split
  - a longer or bigger ghost
  - changing how ghost hits get back into main, stricter or looser
  - extra probation rounds inside the small queue
  - aging frequencies faster
  - hash-table layout tweaks
  - eviction work per miss that has no upper bound
  - taking timing results from policy-only diffs at face value
- **Open questions:**
  - whether web_0's gap at the large cache size can be closed at all with this policy family
  - what would move hm_0 at the small cache size
  - removing a hash probe from the miss path
  - adding heap bytes per resident key to the memory metric, which currently doesn't show ghost or index growth
  - pinning the benchmark environment so timing can be trusted
- **Per-policy notes:** S3-FIFO is the only policy tested so far. The best parent is still cycle-5 v002, since neither cycle 6 nor cycle 7 beat it. The notes cover how hm_0 and web_0 each behave, and that per-entry metadata has stayed flat throughout.


---

- finished: 2026-11-10T04:34:52+00:00
- duration: 24.85s
- status: ok
- input_tokens: 4
- output_tokens: 2320
- cache_read_tokens: 40989
- cache_write_tokens: 12701
- cost_usd: 0.157038
