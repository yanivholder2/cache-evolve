# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-10-21T09:21:05+00:00

## timeline

### 09:21:18 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Changes that make frequency estimates more accurate pay off. Fast hashing with a flat sketch (cycle 1) helped a little, probably through different collisions. A wider count-min sketch (cycle 3) gave the largest policy gain so far. Lower collision noise is the strongest lever found to date.\n- On workload-a, long-run frequency is worth keeping. Every result points the same way: lifetime counts beat freshness.\n- Small, single-lever changes on a stable lineage (v003 → c2 v001 → c3 v003) produce readable results. Each Winner was a modest change that left the admission contest's semantics alone.\n\n## Recurring failures\n\n- Sketch aging/decay has lost twice (cycle 1 with conservative update, cycle 2 alone with a long period). It throws away frequency history the trace rewards and creates ties that newcomers lose. Both tests ran on the noisy narrow sketch, so the results are confounded.\n- Loosening the contest has failed. Admitting on ties (cycle 3) swapped one cold key for another. Strict `>` with keep-victim-on-tie is now validated.\n- Victim sampling has stalled. It gave a marginal win in cycle 2, then widening it lost in cycle 3: with estimates that never age, the lowest estimate marks the youngest resident rather than the coldest.\n- Restructuring the window and segments has failed. A fixed 1% window changed nothing (cycle 1). An adaptive hill-climber grew the window toward LRU-like behavior, which this LIRS-style trace punishes (cycle 2).\n- Per-op-time and metadata-size have gone unmeasured in all three cycles. No latency claim (fast hashing) or memory cost (wider sketch, extra estimates on miss-puts) has been checked.\n\n## Open questions\n\n- With collision noise reduced, does very-long-period aging finally help, or does lifetime frequency still win on workload-a?\n- Would a doorkeeper, or deduplicating the get-miss-then-put double increment, help estimates tell one-hit wonders apart?\n- How far does sketch width/depth scale before hit-rate plateaus, and what does that cost in memory and latency?\n- What do the accumulated changes (fast hash, sampled victims, wider sketch) actually cost in per-op-time and metadata-size?\n- Do the conclusions hold beyond workload-a? Every finding so far comes from one LIRS-style trace.\n\n## Per-policy notes\n\n- TinyLFU admission (sketch + strict-`>` contest): the core is sound. Gains come from estimate quality, not from changing the contest rule. The bottlenecks are collision overcount and the get/put double count.\n- W-TinyLFU window/SLRU layout: inert at a 1% window and harmful when adapted naively. Keep the window small and fixed unless a damped, warmed-up climber is tested.\n- Count-min sketch: making it wider is the current winning direction. Conservative update has only been tested together with aggressive aging, so it is unresolved on its own.\n- Victim selection (LRU-head sampling): a small benefit at a narrow sample, a loss at a wider one. Treat it as settled.\n"
}
```

### 09:21:25 chat

I wrote `cross-cycle-insights.md` from the three cycles in the ledger, using the four required sections in order. None of the three cycles measured speed or memory use, so every conclusion is about hit-rate only.

- **Recurring wins:** making the frequency counts more accurate keeps paying off. The wider sketch in cycle 3 was the biggest gain so far, and the fast hash in cycle 1 helped a little. On workload-a, keeping long-run frequency beats favouring recent activity.
- **Recurring failures:**
  - **Aging:** it lost twice, but both tests ran on the old, noisier sketch, so those results are not clean.
  - **Admitting newcomers on ties:** lost in cycle 3. Keep the strict `>` rule.
  - **Wider victim sampling:** gave a tiny win in cycle 2, then lost when widened in cycle 3. It looks used up.
  - **Window/segment restructuring:** fixed or adaptive, it didn't help.
- **Open questions:**
  - Does long-period aging help now that the sketch is wider?
  - Would a doorkeeper, or removing the double count on get-miss-then-put, help?
  - How far can the sketch grow before gains level off, and at what cost?
  - What do the accumulated changes actually cost in speed and memory?
  - Do any of these findings hold beyond workload-a, the only trace tested?
- **Per-policy notes:** grouped under TinyLFU admission, the window layout, the count-min sketch and victim selection. Conservative update has only been tested together with aggressive aging, so its effect on its own is still unknown.

Files are in /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0003/cross-cycle:
- cross-cycle-insights.md


---

- finished: 2026-10-21T09:21:25+00:00
- duration: 19.94s
- status: ok
- input_tokens: 4
- output_tokens: 1804
- cache_read_tokens: 35333
- cache_write_tokens: 7861
- cost_usd: 0.106051
