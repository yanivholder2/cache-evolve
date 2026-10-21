# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-10-21T09:18:36+00:00

## timeline

### 09:18:52 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Strict admission filtering is the only thing that has reliably raised hit rate. S3-FIFO's probation-queue filter won cycle 1, and every later change that loosened it lost hit rate. Across both cycles, deciding *which keys get in* matters more than *how recency is tracked*.\n- The fixed ~10% probation (small-queue) split has held up. Drifting away from it has cost hit rate.\n- Exact LRU ordering is not needed for hit rate on this trace. Coarser recency (CLOCK) matched LRU, so precise ordering can be traded for cheaper metadata without a hit-rate penalty.\n\n## Recurring failures\n\n- **The measurement gap has lasted every cycle.** Only hit rate has ever been reported. Most hypotheses now claim `metadata-size` or `per-op-time` wins (CLOCK, the u32/Fx representation, the fingerprint ghost ring, LIRS's costs), and none of those claims has been checked. More hit-rate-only cycles give little new information.\n- **Loosening admission into `main` keeps losing.** Longer ghost windows, ghost-hit fast paths into `main`, and growing the probation queue all admitted keys without proven resident reuse, and all lowered hit rate.\n- **Treating a ghost hit as a reuse signal is misleading on loops longer than the cache.** A key can come back within the ghost window but not within `main`'s residency. Policies that act on ghost hits (lengthening the window, resizing adaptively) keep getting this wrong.\n- **Bundling a representation change with a policy change confuses attribution.** When both change at once, a hit-rate drop can't be pinned on either.\n\n## Open questions\n\n- Once measured, do S3-FIFO's ghost structures cost more per-key memory than dropping the `prev`/`next` links saves? Is `get` actually faster than LRU?\n- Does a compact fingerprint ghost ring at the parent's original window size reduce memory and per-op time without hurting hit rate? Do its stale-slot and early-decrement issues matter at that size?\n- Are the policy-independent representation tweaks (Fx hasher, `u32` indices, MRU fast path) truly orthogonal, so they can be stacked onto the S3-FIFO winner?\n- Does CLOCK's victim sweep cause tail-latency spikes on `put` when most entries are marked referenced?\n- Can LIRS with a larger HIR fraction and an O(1) non-resident FIFO close the hit-rate gap to S3-FIFO, and at what memory cost?\n\n## Per-policy notes\n\n- **S3-FIFO:** the best hit rate so far, and the right lineage to build on. Use the cycle-1 variant as the parent, not the weaker cycle-2 children. Keep strict admission and the ~10% split. Remaining work: compact the ghost structures at the same window size, and graft on the representation tweaks.\n- **ARC-style adaptivity (applied to S3-FIFO):** the adaptive split drifted toward a larger probation queue, which let in more scan traffic and lost hit rate. Deprioritize it.\n- **CLOCK:** hit rate about equal to LRU. Its only possible value is memory or latency, which has not been measured. Don't do further recency tweaks unless measurement shows a clear win.\n- **LIRS:** beats LRU but trails S3-FIFO. It is heavier: a larger node and a linear walk to enforce the ghost bound. Revisit only after cost can be measured and the bound walk is O(1).\n- **Strict LRU + representation tweaks:** hit rate unchanged, as intended. This is a stackable speed/memory candidate whose benefit has not been measured yet.\n"
}
```

### 09:18:57 chat

I've written `cross-cycle-insights.md` with the four required sections, built only from the cycle 1 and cycle 2 ledger entries.

- **Biggest caveat:** across both cycles, only hit rate has ever been measured. Most hypotheses now claim memory or speed gains, and none of those has been checked. I put this first under "Recurring failures".
- **What has held up:** strictly filtering which keys get into the cache (S3-FIFO's probation queue) is the only thing that has reliably raised hit rate. The fixed ~10% probation split has held up too. Exact LRU ordering doesn't add hit rate on this trace.
- **What keeps failing:**
  - Changes that let keys into `main` without showing reuse while they were cached have all lowered hit rate.
  - Treating a ghost hit as a sign the key will be reused has been misleading on loops longer than the cache.
  - Changing representation and policy in the same variant made it impossible to tell which caused a hit-rate drop.
- **Per-policy notes:** S3-FIFO is the line to build on, using the cycle 1 variant as the parent. ARC-style adaptive sizing should be dropped. CLOCK and LIRS should wait until memory and speed can be measured. The strict-LRU representation tweaks (Fx hashing, `u32` links) look stackable onto S3-FIFO, but their benefit hasn't been measured either.

Files are in `/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0002/cross-cycle`:
- cross-cycle-insights.md


---

- finished: 2026-10-21T09:18:57+00:00
- duration: 21.16s
- status: ok
- input_tokens: 4
- output_tokens: 1910
- cache_read_tokens: 34831
- cache_write_tokens: 7537
- cost_usd: 0.105478
