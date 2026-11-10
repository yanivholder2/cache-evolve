# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-11-10T08:07:53+00:00

## timeline

### 08:08:06 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Being strict about which keys get into main is the one lever that has helped in both cycles. Every gain came from tightening or keeping the baseline two-reuse graduation rule. Loosening it never gained in steady state.\n- A freq reset on graduation does no harm, and the winning lineage (v001 → v001) carries it. It doesn't offset pollution at admission, though.\n- The O(1) hash-index/slab refactor kept hit-rate identical to the baseline. That makes it a safe base for policy work, but its cost benefit is still unconfirmed.\n\n## Recurring failures\n\n- Changing one policy constant (graduation threshold or small/main split) doesn't win cleanly. Its effect flips sign across traces (hm_0 vs web_0) and across cache sizes, and the movement is close to noise.\n- Static split tuning has failed in both cycles, at 10/90 and at 15% combined with the looser threshold. Queue sizes matter less than which keys get into main.\n- The one-reuse threshold pollutes main with keys accessed exactly twice. Its apparent gain on hm_0 is a cold-start or small-cache effect that disappears at larger sizes and in steady state.\n- Main occupancy doesn't work as an adaptive signal: main fills up and stays full, so any gate based on it falls back to the baseline rule.\n- An unweighted ARC-style ghost-hit split probably drifts toward a larger small queue because the two ghosts differ in size. It also adds a second linear ghost scan on every miss.\n- Per-op-time and metadata-size have gone unmeasured for two cycles running. Every variant has followed the hit-rate policy line, so the requested re-measurement keeps getting skipped.\n\n## Open questions\n\n- How much per-op-time does the O(1) refactor actually save, and what does its index, slab and stamp state cost in metadata-size? This is now the highest-priority gap.\n- Can a signal that keeps changing in steady state (ghost hit rate, small-queue reuse rate) usefully switch the threshold between 1 and 2? Or is the baseline rule simply right for these traces?\n- Where does an adaptive `small_size` settle on each trace? This has never been logged, and a size-weighted ARC step would answer it.\n- How much per-op-time does the extra linear `main_ghost` scan cost, and would an O(1) ghost structure remove that cost?\n\n## Per-policy notes\n\n- S3-FIFO baseline: the two-reuse (`freq > 1`) graduation rule into main appears near-optimal for hm_0/web_0 under steady-state pressure. Proposed changes have mostly ended up back at it.\n- Graduation threshold: a threshold of 1 helps only hm_0 at the small cache size and hurts web_0 and hm_0 at the large size. The threshold and small-queue size interact, so if they're ever tuned, they should be tuned together.\n- Small/main split: static and unweighted adaptive splits both give effects at noise level whose sign flips. Don't use more hypotheses on this unless the ARC step is size-weighted and logs where `small_size` settles.\n- Adaptive gates: gate on a signal that keeps changing in steady state, never on occupancy.\n- Implementation line: the O(1) index/slab refactor doesn't change policy behaviour. The policy work should be moved onto it once per-op-time and metadata-size have been measured.\n"
}
```

### 08:08:12 thinking



### 08:08:12 chat

I wrote `cross-cycle-insights.md` from the two-cycle ledger, using the four required sections in order.

- **Recurring wins:** being strict about which keys get into main is the only lever that has helped in both cycles. The baseline rule (a key must be reused twice before it moves to main) keeps winning. The O(1) index/slab refactor is safe to build on because hit-rate didn't change, but its speed benefit is still unconfirmed.
- **Recurring failures:**
  - Changing one setting at a time (the reuse threshold or the small/main split) gives results that flip between hm_0 and web_0 and between cache sizes.
  - Gates based on how full main is fail because main stays full once the cache warms up.
  - The unweighted ARC-style adaptive split probably drifts toward a larger small queue.
  - Per-op-time and metadata-size have gone unmeasured for two cycles.
- **Open questions:** whether the refactor actually saves time and what its extra state costs; whether a signal that keeps changing once the cache is full could switch the threshold; where an adaptive small-queue size settles on each trace; and what the second ghost scan costs per operation.
- **Per-policy notes:** grouped by the baseline graduation rule, the reuse threshold, the small/main split, adaptive gates, and the refactor.


---

- finished: 2026-11-10T08:08:12+00:00
- duration: 19.02s
- status: ok
- input_tokens: 4
- output_tokens: 1718
- cache_read_tokens: 35323
- cache_write_tokens: 7887
- cost_usd: 0.104848
