# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-20T20:00:19+00:00

## timeline

### 20:00:27 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- The second-hit admission filter is the only real policy win. It was found in cycle 0001, made metadata-free by bit-packing in cycle 0002, and re-derived independently in cycle 0004. It keeps hit-rate gains, baseline item size and a latency benefit.\n- Subtractive changes helped once. Removing bookkeeping the policy never reads (cycle 0004 v003) improved latency and hit-rate. The dead stores left after that were too few to measure (cycle 0005).\n- Bit-packing works. Spare bits hold flags, counters and ref bits, so metadata footprint is not a constraint.\n\n## Recurring failures\n\n- Insert-position variants (uniform MRU, cold first admission, cold demotion) overlap with admission rather than stacking. They trade hit-rate against latency and look exhausted.\n- Richer admission and recency signals (2-bit counter, ref bit, skip-if-hot guard) added almost no hit-rate, because the second-hit filter already removes most one-touch items. Each also cost latency.\n- Blind hot-path edits to the incumbent (lookup atomics, early returns, free-slot fast paths, promotion skip, hoisting hash work out of the lock) have failed in four cycles in a row. They added branches or work and slightly worsened latency.\n- \"Winner\" status is weak evidence. Hit-rate gains are within plausible trace noise, and latency regressions often appear on paths the change did not touch. That points to layout effects or measurement noise, and the noise floor has never been measured.\n\n## Open questions\n\n- What is the run-to-run noise floor for get and put latency on the unchanged incumbent? Repeat baseline runs to find out.\n- Where do get and put time actually go (lock acquisition, bucket scan, memory access)? Nothing has profiled this yet.\n- Does a different trace (scans, frequency skew) show differences the LIRS trace hides? All hit-rate conclusions come from one recency-skewed trace.\n- How does the cycle 0004 v003 minimal filter compare with the cycle 0002 v001 incumbent? How much of its gain is dead-bookkeeping removal and how much is the filter?\n- Do structural changes to probing, bucket layout or locking help? Try them only if profiling points there.\n\n## Per-policy notes\n\n- DASH segment promotion and admission policy (the only policy explored so far):\n  - Base: the minimal bit-packed second-hit filter with dead bookkeeping removed, at baseline item size.\n  - Position levers, admission signals and eviction-order levers all look saturated on the LIRS trace.\n  - Future gains need either profiling-driven structural work or evaluation on other traces.\n"
}
```

### 20:00:33 chat

I wrote `cross-cycle-insights.md` to the cycle 0005 cross-cycle output dir. It has the four fixed sections in order and is built only from the five-cycle ledger digest.

The main points it records:
- **Recurring wins:** The second-hit admission filter is the only real policy win. Bit-packing keeps its benefit at baseline item size. Removing bookkeeping the policy never reads helped once, and little of it is left to remove.
- **Recurring failures:**
  - Position variants overlap with admission instead of stacking.
  - Richer admission signals added almost no hit-rate and cost latency.
  - Blind hot-path edits have failed four cycles in a row.
  - "Winner" status is weak evidence, since the latency noise floor has never been measured.
- **Open questions:**
  - Measure the noise floor by repeating baseline runs.
  - Profile where get and put time goes.
  - Try a trace other than the recency-skewed LIRS one.
  - Compare the cycle 0004 minimal filter with the cycle 0002 incumbent.
  - Try structural changes to probing, bucket layout or locking only if profiling points there.
- **Per-policy notes:** Only the DASH segment promotion and admission policy has been explored. The base is the minimal bit-packed second-hit filter, and the position, admission and eviction-order levers look saturated on the LIRS trace.

I used only qualitative terms and did not read the previous insights file.


---

- finished: 2026-10-20T20:00:34+00:00
- duration: 14.89s
- status: ok
- input_tokens: 4
- output_tokens: 1508
- cache_read_tokens: 37304
- cache_write_tokens: 8870
- cost_usd: 0.0548
