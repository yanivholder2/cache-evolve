# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-10-20T15:39:00+00:00

## timeline

### 15:39:08 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Changes with a clear mechanism are the only ones that pay off reliably. Examples: removing redundant linear scans (Cycle 3) and hash-index membership (Cycle 4). Both cut put/get cost and leave the hit/miss sequence unchanged.\n- Constant-only tweaks to the small queue gave a modest gain early on (10%, then 5%). Returns are diminishing and the curve is flattening.\n- Keeping the small queue as a strict one-hit-wonder filter, with baseline promotion, protects hit rate.\n\n## Recurring failures\n\n- Anything that loosens main admission lowers hit rate. This has happened four cycles running. Examples: a lower promotion threshold, a larger small queue, recycling main evictions into the shared ghost, a freq head start for ghost re-entry, and a longer ghost history.\n- Ghost-side tuning of main-evicted keys (recycling, head start, size) has repeatedly regressed or shown no gain. Cycle 2's apparent gain is now doubtful.\n- Rotation caps and other bounds on already-saturated counters trade hit rate for nothing.\n- Latency is a noisy selection signal. Variants with no hot-path mechanism often \"won\" on latency shifts alone, so regressions in hit rate got promoted. Credit latency only when a mechanism explains it.\n- Refactors that delete tests, and index structures whose memory cost is hidden from the per-entry metadata figure, create hidden debt.\n- Infrastructure failures (such as a Maven 429) can silently hide an idea. Retry the idea rather than treating it as rejected.\n\n## Open questions\n\n- Do the small-ratio, ghost and index results hold across other traces, cache sizes and repeated runs? All evidence so far comes from one trace at one size.\n- What is the index's true whole-cache memory cost, and is it acceptable?\n- Can the put tail be cut by making eviction and ghost trimming cheaper?\n- Is any special handling of main-evicted keys worthwhile? It would need an O(1) tagged ghost and evidence of faster reuse.\n- Can the tests deleted in the scan refactor be restored without losing its gains?\n- Is 5% really better than 10%, or is the difference within noise?\n\n## Per-policy notes\n\n- S3-FIFO (the only policy tried so far):\n  - Base: scan consolidation plus hash-index membership. The small ratio is 10%, or 5% if confirmed on other traces.\n  - Baseline promotion threshold and baseline ghost sizing look near-optimal on workload-a.\n  - Stop sweeping the small ratio, the ghost size and the promotion threshold.\n  - The next gains are likely in put-tail cost (eviction and ghost trimming), validated on multiple traces and judged on hit rate.\n"
}
```

### 15:39:12 chat

I wrote `cross-cycle-insights.md` to the output dir, projected from the four-cycle ledger digest, with the four fixed sections in order. I did not read any previous version.

Main takeaways in it:
- **Wins:** Only changes with a clear mechanism have paid off: scan consolidation and the hash-index membership lookup. Both cut put/get cost without changing which requests hit.
- **Failures:** Loosening main admission has lowered hit rate in all four cycles. Ghost tuning for main-evicted keys has regressed or shown no gain. Latency alone is a noisy signal that has promoted regressions.
- **Open questions:** Whether the results hold on other traces and sizes, what the index really costs in memory, how to cut the put tail, and whether the 5% small-queue gain is real.
- **Policy notes:** S3-FIFO is the only policy tried. The notes say to stop sweeping the small ratio, ghost size and promotion threshold, and to look for the next gains in eviction and ghost-trimming cost.

I did not invent any numbers; everything is stated qualitatively.


---

- finished: 2026-10-20T15:39:12+00:00
- duration: 12.44s
- status: ok
- input_tokens: 4
- output_tokens: 1432
- cache_read_tokens: 36762
- cache_write_tokens: 8377
- cost_usd: 0.051961
