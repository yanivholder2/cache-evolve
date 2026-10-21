# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-20T20:05:54+00:00

## timeline

### 20:06:06 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- The second-hit admission filter is the only real policy win. It showed up in cycle 0001, was bit-packed into spare bits at baseline item size in cycle 0002, and was independently re-derived in cycle 0004. Each time it gained hit-rate and kept latency flat or better.\n- Packing policy state into spare bits keeps metadata flat. Footprint is no longer a constraint.\n- Removing work the policy never reads (dead per-get stats and timestamp writes) helped once, in cycle 0004. The gains from further deletion were too small to measure.\n- A lossy doorkeeper cut put-side work sharply (cycle 0006). That is a real lead, but it cost hit-rate.\n\n## Recurring failures\n\n- Hot-path edits to the incumbent have not paid off in five straight cycles (0003 to 0006). These were guards, shortcuts, atomics, early returns, free-slot fast paths, promotion skips, hoisting, tag-first scans and reader-writer locks. Each added branches or work and cost latency. The benchmark looks uncontended and the buckets mostly full, so there is little to exploit.\n- Insert-position and cold-demotion tweaks overlap with the admission filter instead of adding to it. They trade hit-rate against latency.\n- Richer admission or recency signals (counters, ref bits) added no hit-rate beyond the filter. Hit-rate looks saturated on this trace.\n- \"Winner\" status is weak evidence. Flat hit-rate and metadata qualify a variant even when latency regresses.\n- Latency moves often appear on paths the change did not touch. The noise floor and code-layout effects have never been measured, so most latency claims are unreliable.\n- Variant authors have mostly reasoned from fitness vectors without reading source, so the mechanisms offered are inferred.\n\n## Open questions\n\n- What is the run-to-run noise floor for get and put latency on the unchanged incumbent? This has been asked for several cycles and is still unanswered.\n- Where does get and put time actually go? Nothing has profiled it.\n- Does a different trace (scans, frequency skew) change any conclusion? Every result so far comes from the recency-skewed LIRS trace.\n- Can a larger or aged doorkeeper keep the put-side savings without losing hit-rate?\n- How much of the cycle 0004 re-derivation's gain came from the filter and how much from removing dead bookkeeping? Does it match the cycle 0002 bit-packed incumbent?\n\n## Per-policy notes\n\n- DASH segment promotion (the only policy exercised so far): the bit-packed second-hit filter with dead bookkeeping removed is the base to beat.\n- Insert position is exhausted as a lever. Uniform MRU, cold and tiered inserts gave small or noisy effects.\n- Admission is saturated beyond the single second-hit gate. A counter or ref bit adds nothing.\n- Eviction-order changes (CLOCK-style second chance) gave negligible hit-rate gain and a latency cost.\n- Lookup, locking and probing mechanics are not the bottleneck on this workload.\n"
}
```

### 20:06:10 chat

I wrote `cross-cycle-insights.md` to the output dir, with the four required sections in order. It is built only from the six ledger digests, and I didn't read the previous version.

- **Recurring wins:** The bit-packed second-hit filter is the only policy win that has held up, and dropping work the policy never reads helped once. A lossy doorkeeper cut put-side work sharply in cycle 0006, but it cost hit-rate.
- **Recurring failures:** Hot-path edits to the incumbent have failed five cycles in a row (0003 to 0006), and insert-position and cold-demotion tweaks overlap with the filter instead of adding to it. Winner status is weak evidence, and the latency noise floor has never been measured.
- **Open questions:** These are the latency noise floor, where get and put time actually go, and whether a different trace changes the conclusions. They also include whether a larger or aged doorkeeper can keep its put-side savings, and how much of cycle 0004's gain came from the filter versus removed bookkeeping.
- **Per-policy notes:** Every cycle exercised the same DASH segment promotion policy, so the notes cover only that one. The cleaned-up bit-packed second-hit filter is the base to beat. Insert position and admission signals are exhausted as levers, and lookup, locking and probing are not the bottleneck on this workload.

I kept every claim qualitative and invented no numbers.


---

- finished: 2026-10-20T20:06:10+00:00
- duration: 16.48s
- status: ok
- input_tokens: 4
- output_tokens: 1633
- cache_read_tokens: 38145
- cache_write_tokens: 9854
- cost_usd: 0.060154
