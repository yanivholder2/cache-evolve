# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-10-20T15:55:25+00:00

## timeline

### 15:55:35 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Behavior-preserving rewrites with a clear mechanism are the reliable gains: removing linear scans, hash-index membership, ring-buffer ghost, and the single location-tagged index with a split ghost map. Each cut a specific cost and left hit rate identical.\n- The `put` tail was traced step by step. Linear scans came first, then ghost maintenance, then hash-map probe count. Fixing the real bottleneck moved it. Fixing something adjacent did not.\n- Shrinking the small queue helped hit rate with diminishing returns (10% → 5% → 3%). It is the only hit-rate lever that has paid off.\n- A cheap hasher helps `get` on the tagged index.\n\n## Recurring failures\n\n- Anything that loosens main admission lowered hit rate or gave nothing: freq>=1 promotion, a larger small queue, ghost recycling, a freq head start, a larger ghost bound, and the recent-main second chance. The small queue's filtering role matters more than the loosened variants assumed.\n- Main-queue eviction rewrites and rotation caps gave nothing. Eviction-policy tweaks have little room on this trace.\n- Micro-optimizations (frequency store, deque sizing) and hasher/finalizer/pre-size tuning ended in tradeoffs with no free win. That direction is exhausted.\n- Latency shifts with no mechanism recur, including `get` moves in variants that never touched `get`. Winners picked on such shifts are probably noise. Latency alone is an unreliable selection signal.\n- Bundled changes (pre-size plus finalizer) made it impossible to attribute effects.\n\n## Open questions\n\n- Do the gains hold across other traces, cache sizes, and repeated runs? Every result so far is one trace at one size.\n- Is the 3% small ratio safe on workloads with more mid-reuse objects?\n- What is the real heap footprint of the tagged index plus split ghost map? The metadata metric counts only inline struct bytes.\n- Can hit rate improve through promotion or ghost policy changes that do not loosen main admission? One option is an O(1) tagged variant of the ghost.\n- What remains in the `put` tail, and can the `get` p99 cost of the tagged index be reduced?\n- Were the tests removed in the scan refactor ever restored?\n\n## Per-policy notes\n\n- S3-FIFO (the only policy tried): the small queue is a filter for one-hit-wonders. Keep the baseline promotion threshold, keep main admission strict, and keep the ghost near its baseline size.\n- The current base is the tagged index, split ghost map, ring ghost, and one Fx-style hasher with moderate slack. Stop work on hashers and ratios.\n- Next cycles should seek hit-rate ideas, validate on multiple traces with repeated runs, and credit only mechanism-backed latency changes.\n"
}
```

### 15:55:41 chat

I wrote `cross-cycle-insights.md` in the output dir, with the four sections in the required order. I didn't read the previous version and didn't add numbers.

- **Recurring wins:**
  - Behavior-preserving rewrites with a clear mechanism keep paying off. Examples are scan removal, the hash index, the ring ghost, and the tagged index with a split ghost map.
  - A smaller small queue is the only hit-rate lever that has paid off, with diminishing returns.
- **Recurring failures:**
  - Anything that loosens main admission lowered hit rate or gave nothing.
  - Eviction rewrites, micro-optimizations, and hasher tuning have run out of room.
  - Latency shifts with no mechanism are noise, so latency alone is an unreliable selection signal.
- **Open questions:**
  - Do the gains hold on other traces, sizes, and repeated runs?
  - Is 3% safe on workloads with more mid-reuse objects?
  - What is the real heap footprint of the tagged index plus split ghost map?
  - Can hit rate improve without loosening main admission?
  - What remains in the `put` tail?
  - Were the tests deleted in the scan refactor ever restored?
- **Per-policy notes:** S3-FIFO is the only policy tried. The notes give the current base and what to focus on next.


---

- finished: 2026-10-20T15:55:41+00:00
- duration: 16.50s
- status: ok
- input_tokens: 4
- output_tokens: 1540
- cache_read_tokens: 40165
- cache_write_tokens: 11803
- cost_usd: 0.067426
