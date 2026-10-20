# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-20T15:50:51+00:00

## timeline

### 15:51:02 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Behavior-preserving rewrites that remove a mechanism-identified cost keep paying off, with hit rate unchanged. Examples: scan consolidation, hash index, O(1) ring ghost, tagged index, split ghost map, cheap hasher.\n- The lever has moved step by step. Linear scans came first, then ghost maintenance, then hash-map traffic (probe count per `put`, hashing cost per `get`). Each fix exposed the next bottleneck.\n- Cutting the number of map operations per `put` (the tagged index) was the largest single `put` gain. A cheaper hasher mainly helps `get`.\n- A smaller small queue (10%, then 5%) gave small, directionally consistent hit-rate gains. They are the only hit-rate gains seen, and they flatten out.\n\n## Recurring failures\n\n- Anything that loosens main admission or eviction has lost hit rate or gained nothing. This includes a lower promotion threshold, a larger small queue, ghost recycling, a freq head start, a larger ghost bound, a rotation cap, and the recent-main second chance. The small queue's one-hit-wonder filtering is what matters.\n- Micro-optimizations (frequency store, deque pre-sizing, CLOCK-style main eviction rewrite) land within noise. Main-queue eviction is not the cost.\n- Latency shifts without a mechanism (`get` moving in variants that never touch it, big drops from constant-only edits) recur every cycle. They are noise, and they let neutral or regressing variants win selection.\n- Speed wins carry hidden costs. Whole-cache footprint grows with each structure, and the per-entry metadata metric ignores heap. Bundled changes (pre-sizing with hasher) make regressions hard to attribute.\n- Infrastructure failures (a Maven 429) can hide an idea without testing it.\n\n## Open questions\n\n- Do the gains hold across other traces, cache sizes, and repeated runs? All evidence is workload-a at one size.\n- What is the real heap footprint of the tagged index plus the separate ghost map?\n- Do the split ghost map and the cheap hasher combine cleanly? Was the `put` tail regression caused by pre-sizing or by weak hash mixing?\n- What remains in the `put` tail after the map work? Candidates are eviction and the small-to-main move.\n- Is there any hit-rate idea that does not loosen main admission? Is a main-evicted-key special case worth it as an O(1) tagged ghost?\n- Were the tests deleted by the scan refactor restored?\n\n## Per-policy notes\n\n- S3-FIFO (the only policy so far):\n  - The current best structure is a 5% small queue with baseline promotion (freq threshold and ghost sizing at baseline), a ring or split ghost, and a tagged hash index.\n  - The baseline ghost size and promotion threshold look near a local optimum on workload-a.\n  - Hit rate has been flat across cycles. Progress has been in `put` and `get` cost.\n  - Judge on hit rate and on changes with a clear mechanism, not on raw latency.\n"
}
```

### 15:51:07 chat

I wrote `cross-cycle-insights.md` to the output dir, with the four fixed sections in order. I didn't read the previous version. I built the file from the cycle 1–7 ledger digest alone, so every point in it is qualitative.

- **Recurring wins:** Behavior-preserving speed rewrites that target a specific cost keep paying off with hit rate unchanged. The bottleneck has moved from linear scans to ghost upkeep to hash-map traffic. The 10% and then 5% small-queue ratios gave the only hit-rate gains, and those are flattening out.
- **Recurring failures:** Anything that loosens main admission has lost hit rate or gained nothing. Micro-optimizations land within noise. Latency shifts with no mechanism show up every cycle and let neutral variants win selection. Speed wins also carry footprint costs the metadata metric doesn't show.
- **Open questions:**
  - Do the gains hold on other traces, cache sizes and repeated runs?
  - What is the real heap footprint of the tagged index plus the separate ghost map?
  - Did pre-sizing or weak hash mixing cause the `put` tail regression?
  - What is left in the `put` tail?
  - Is there any hit-rate idea that doesn't loosen main admission?
- **Per-policy notes:** S3-FIFO is the only policy so far. The best structure is a 5% small queue, baseline promotion and ghost sizing, a ring or split ghost, and a tagged hash index.


---

- finished: 2026-10-20T15:51:07+00:00
- duration: 15.70s
- status: ok
- input_tokens: 4
- output_tokens: 1668
- cache_read_tokens: 39330
- cache_write_tokens: 11047
- cost_usd: 0.065515
