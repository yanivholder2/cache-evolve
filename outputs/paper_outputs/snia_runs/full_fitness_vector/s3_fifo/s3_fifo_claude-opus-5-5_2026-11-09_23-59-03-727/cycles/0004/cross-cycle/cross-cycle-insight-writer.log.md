# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-11-10T04:16:29+00:00

## timeline

### 04:16:47 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- Making one change per variant pays off. Bundled changes in cycles 1–2 hid each other's effects and kept the hm_0/web_0 trade-off in place. Isolated changes in cycle 3 removed that trade-off, and cycle 4's merge of isolated wins added up cleanly.\n- Judging a key only by its reuse while in main works on both traces. Two filters do this: reuse-gated main-victim ghosting, and resetting freq/reused on graduation. Each won alone in cycle 3, together they won at every trace/size point in cycle 4, and neither added entry size or latency.\n- Spending ghost budget selectively (filtering out unreused victims) lengthens useful history at no cost.\n- Structural changes that leave hit rates bit-identical are the only reliable source of timing gains: the slab plus hash indexes (cycle 2) and the Fx-style hasher (cycle 3). Per-entry metadata stayed flat throughout.\n- hm_0 reliably responds to how main retains keys: more main residency, main-victim history, or reuse-based credit.\n\n## Recurring failures\n\n- Fixed-ratio and region resizing (small-queue %, ghost region splits) only move gains between hm_0 and web_0 without a net win (cycles 1–2).\n- Making the ghost larger hurts. A ghost bound beyond `main_size` lets stale reuse back into main, which pushes out live residents. Cycles 1, 2 and 4 all point to the ghost bound being coupled to other variables, and to the fix being selectivity rather than size.\n- Loosening graduation to `freq >= 1` causes web_0's regression. The freq reset alone is harmless.\n- ARC-style adaptive small/main splitting regressed when the signals were unbalanced and the ghost was tied to a fixed bound (cycle 1). It has not been retried.\n- Policy-only diffs keep showing timing shifts their changes can't explain, in both directions, in every cycle. These are environmental noise and earn no timing credit.\n- Cutting probe count by merging the resident and ghost indexes slowed things down, tails most of all. Higher load factor and wider buckets outweighed the fewer lookups.\n- The metadata-size metric keeps missing real heap growth: ghost key storage, hash maps, stale ghost tuples, unified tables.\n\n## Open questions\n\n- What explains web_0's remaining loss at the large size? The ghost bound and ghost competition have been ruled out. The small-queue share (`capacity / 4`) is the last candidate and is still untested in isolation on the current parent.\n- Can main-retention knobs (`MAX_FREQ`, graduation threshold) add hm_0 gains without the web_0 cost that `freq >= 1` showed?\n- Would a unified index pay off with explicit load-factor headroom or a compact slot (a `u32` slab index plus a ghost flag bit)?\n- Would a balanced adaptive split work on top of the current filtered-ghost parent?\n- Timing needs a same-environment re-baseline. Should metadata reporting add heap bytes per resident key?\n\n## Per-policy notes\n\n- S3-FIFO (the only policy explored so far). The current best lineage is slab + hash, then Fx hasher, then both reuse filters with the `freq > 1` threshold and the ghost bound kept at `main_size` or lower (cycle 4 v001). Build on it.\n- S3-FIFO on hm_0: sensitive to how main retains keys and to main-victim history, and gains the most from the reuse filters.\n- S3-FIFO on web_0: sensitive to admitting freq-0 or loosely qualified keys into main and to squeezing the probation window. It regresses under a looser graduation threshold or a larger ghost, and its weakest point is the large size.\n- S3-FIFO implementation: hashing and slab layout are where timing gains come from. Probe count matters less than table occupancy and bucket width.\n"
}
```

### 04:16:54 thinking



### 04:16:55 chat

I've regenerated `cross-cycle-insights.md` from scratch using the four ledger cycles, with the four required sections in order and no numbers.

- **Recurring wins:** testing one change per variant works. The best result so far is cycle 4 v001, which combines two "reuse in main" filters, and it improved hit rate on every trace and size. Only structural changes that leave hit rates identical have produced real timing gains.
- **Recurring failures:** resizing the queues or ghost regions just moves gains between hm_0 and web_0. A larger ghost actively hurts. Loosening graduation to one hit (`freq >= 1`) is what caused web_0's earlier regression. Merging the two indexes to cut lookups made things slower. Timing shifts from policy-only changes are still benchmark noise, and the metadata metric misses real memory growth.
- **Open questions:** the one untested explanation for web_0's loss at the large size is the small-queue share (`capacity / 4`). Also open: the main-retention settings (`MAX_FREQ`, graduation threshold), a merged index with spare table room, a retry of the adaptive split on the current parent, and re-measuring timing on the same machine.
- **Per-policy notes:** only S3-FIFO has been explored. The notes record what to build on next (cycle 4 v001) and what each trace is sensitive to.


---

- finished: 2026-11-10T04:16:55+00:00
- duration: 25.77s
- status: ok
- input_tokens: 4
- output_tokens: 1923
- cache_read_tokens: 37908
- cache_write_tokens: 9412
- cost_usd: 0.121863
