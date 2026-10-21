# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-20T20:18:03+00:00

## timeline

### 20:18:13 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Second-hit admission (keep one-touch items out of hot tiers) is the one real policy win. It was confirmed independently in cycles 0001, 0002 and 0004, and it held at flat metadata once the flag was bit-packed.\n- A small admission filter sized to one cache line and aged often enough to stay sparse (cycle 0008) is the best put-side result. It skips insert and eviction work on one-hit keys and holds hit-rate.\n- Removing bookkeeping that the policy never reads helped once (cycle 0004). Later deletion-only variants gave nothing, so that gain is spent.\n- Metadata stays flat in nearly every variant, because spare bits are enough. Footprint is not a constraint.\n\n## Recurring failures\n\n- Hot-path or control-flow edits to the incumbent (lookup atomics, early returns, free-slot fast paths, promotion skips, hoisting, tag-first scans, reader-writer locks, put early-return) regressed latency in about seven cycles in a row. They add branches or work, and the benchmark appears uncontended.\n- Latency regressions often show up on paths the change did not touch. This points to layout effects or noise, and the noise floor has never been measured.\n- Extra admission or recency signals (hit counters, ref bits, ghost fingerprints) gave negligible hit-rate, because the second-hit filter already screens most one-touch items.\n- Insert-position tweaks overlap with admission and do not stack. Position alone looks exhausted.\n- Lossy, unaged or tiny filters buy latency at the cost of hit-rate (cycles 0006 and 0008). Large filters recover hit-rate but cost put latency.\n- Doorkeeper tuning (aging threshold, probe count) is now a weak lever.\n- Winner status is weak evidence. Flat hit-rate and metadata are enough to qualify, even when latency regresses.\n- Variant authors often could not read the source, so mechanisms are inferred from fitness vectors.\n\n## Open questions\n\n- What is the run-to-run noise floor for get and put latency on the unchanged incumbent? It has been asked for six cycles and is still unanswered. Measure it with repeated runs, comparing against the parent in the same run.\n- Where does get and put time actually go? No profiling has been done.\n- Do the conclusions hold on a second trace with scans or frequency skew? All results so far are from the recency-skewed LIRS trace.\n- Would eviction or promotion changes, rather than admission, move hit-rate further? Admission looks saturated.\n- Are the small hit-rate gains of the aged doorkeeper variants real or within trace noise?\n\n## Per-policy notes\n\n- DASH segment, second-hit / doorkeeper admission: this is the active family. The current incumbent is the one-cache-line aged two-probe doorkeeper from cycle 0008. It is tuned out, so stop further tuning.\n- Insert position (uniform MRU, cold/hot split, cold demotions): small gains at best, with a hit-rate and latency trade-off. Do not stack on admission.\n- Eviction order (CLOCK-style second chance): no hit-rate gain and a latency cost.\n- Locking and bucket scan: no sign of a bottleneck. Do not revisit without profiling.\n- Next direction: measure noise and profile first, then try eviction or promotion policy on a second trace.\n"
}
```

### 20:18:17 chat

I wrote `cross-cycle-insights.md` to the output dir, with the four fixed sections in order. It is built only from the ledger digest for cycles 0001–0009. I did not read the previous insights file or any code.

- **Recurring wins:**
  - Second-hit admission.
  - The one-cache-line aged doorkeeper.
  - A single dead-bookkeeping removal that helped once and has since run out.
  - Flat metadata.
- **Recurring failures:**
  - Hot-path and control-flow edits kept regressing latency.
  - Extra admission or recency signals gave almost no hit-rate.
  - Insert-position tweaks overlap with admission and don't stack.
  - Winner status is weak evidence.
- **Open questions:**
  - The latency noise floor is still unmeasured after six cycles of asking.
  - No profiling has been done.
  - Everything so far comes from the single LIRS trace.
  - Eviction and promotion changes are untried.
- **Per-policy notes:** these are grouped by the DASH segment's admission, insert-position, eviction-order and locking/bucket-scan levers. They name the cycle 0008 doorkeeper as the incumbent and point the next cycle at measuring noise and profiling first.


---

- finished: 2026-10-20T20:18:18+00:00
- duration: 14.86s
- status: ok
- input_tokens: 4
- output_tokens: 1670
- cache_read_tokens: 40473
- cache_write_tokens: 12269
- cost_usd: 0.070648
