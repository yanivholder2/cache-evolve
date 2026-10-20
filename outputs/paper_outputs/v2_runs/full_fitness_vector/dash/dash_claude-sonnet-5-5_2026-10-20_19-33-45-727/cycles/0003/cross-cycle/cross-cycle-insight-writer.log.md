# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-10-20T19:51:47+00:00

## timeline

### 19:51:59 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Zero-metadata designs: bit-packing the second-hit admission flag into spare bits kept the hit-rate benefit, restored baseline item size, and improved latency. It is the only clean, repeatable win so far.\n- Scan resistance (keeping one-hit-wonders out of the hot region) is the lever that pays off, whether through insert position or admission. Admission gating beats position tweaks.\n- Latency gains appear only where work is removed (e.g. skipping remove-and-promote on a first hit), not where work is added.\n\n## Recurring failures\n\n- Insert-position tweaks (uniform MRU, cold/hot split, cold first admission, cold demotions): small gains, overlap with admission, and trade hit-rate against latency. This lever looks exhausted.\n- Stacking overlapping levers (position plus admission): they target the same items, so gains do not add.\n- Richer signals on top of the filter (2-bit counter, ref bit, skip-if-hot shortcut): negligible hit-rate gain and a latency loss, because each adds hot-path work. A branch that only sometimes succeeds is not free.\n- \"Winner\" labels on tiny hit-rate gains are often within trace noise and should not be trusted.\n\n## Open questions\n\n- Are the small latency changes real or run-to-run noise? Re-measure before trusting them.\n- Where does get/put time actually go (lookup, locking, bucket scan)? Latency work on those paths is untried.\n- Do the levers matter on other traces (scan-heavy, frequency-skewed), or is saturation specific to the recency-skewed LIRS trace?\n- Is the admission-plus-eviction space truly saturated, or only on this trace?\n\n## Per-policy notes\n\n- DASH (segment promotion/admission): the base to build on is the bit-packed second-hit filter at baseline item size. Spare bits can hold a counter or ref bit, so footprint is not a constraint, but hit-rate is saturated on LIRS. Next effort should go to latency on lookup, locking and bucket-scan paths, or to a different trace.\n"
}
```

### 19:52:02 chat

I wrote `cross-cycle-insights.md` to the cycle 0003 cross-cycle output directory. It uses the four fixed sections in order and is built only from the three-cycle ledger digest.

- **Recurring wins:** The bit-packed second-hit filter is the only clean, repeatable win. Scan resistance through admission gating pays off more than position tweaks. Latency improves only where work is removed.
- **Recurring failures:** Insert-position tweaks look exhausted, and stacking position and admission levers doesn't add up. Counters, ref bits and shortcuts on top of the filter added almost no hit-rate and cost latency. "Winner" labels on tiny gains are probably trace noise.
- **Open questions:**
  - Are the small latency changes real or run-to-run noise?
  - Where does get/put time actually go?
  - Does the saturation seen on the LIRS trace hold on other traces?
- **Per-policy notes:** For DASH, the bit-packed second-hit filter at baseline item size is the base to build on. The next effort should go to latency on the lookup, locking and bucket-scan paths, or to a different trace.

I didn't read the previous insights file.


---

- finished: 2026-10-20T19:52:03+00:00
- duration: 15.72s
- status: ok
- input_tokens: 4
- output_tokens: 1210
- cache_read_tokens: 35777
- cache_write_tokens: 7139
- cost_usd: 0.04459
