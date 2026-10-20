# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-10-20T20:14:25+00:00

## timeline

### 20:14:35 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Admission filtering that keeps one-hit keys out of the hot region is the only lever that has repeatedly paid off (second-hit filter, then doorkeeper variants). It also saves put-side insert and eviction work.\n- Packing filter state into spare bits keeps item size at baseline, so footprint is not a constraint.\n- Subtractive re-derivation from baseline helped once (dropping unread stats), by removing work the policy never reads.\n- A doorkeeper held to one cache line and aged often enough to stay sparse gave the best trade-off so far. It matched a large filter's hit-rate at lower put cost.\n\n## Recurring failures\n\n- Insert-position tweaks, cold demotion and cold first admission overlap with the admission filter. They do not stack, and they trade hit-rate against latency.\n- Richer signals (multi-bit counters, ref bits, ghost entries) added hot-path work for negligible hit-rate gain. The filter already catches most one-touch items.\n- Blind hot-path edits to the incumbent (lookup atomics, early returns, free-slot fast paths, promotion skips, hoisting, tag-first scan, RW-lock upgrade) made latency slightly worse across many cycles. Probing, locking and hashing are not where time goes, and the benchmark looks uncontended.\n- Further dead-store deletion stopped helping once the main dead bookkeeping was gone.\n- Small unaged or lossy filters cut latency but lose hit-rate.\n- Warm-up admission into free slots gave nothing, because the cache is almost always in the full-segment regime.\n- \"Winner\" status is weak evidence, since flat hit-rate and metadata qualify a variant even when latency regresses. Many latency regressions appeared on untouched paths, which points to layout or measurement noise.\n\n## Open questions\n\n- What is the run-to-run noise floor for get and put latency on the unchanged incumbent? This has been asked for five cycles and is still unmeasured, and every small latency claim depends on it.\n- Where do get and put time actually go? Nobody has profiled this.\n- Do the conclusions hold on a second trace with scans or frequency skew? All results so far come from the recency-skewed LIRS trace.\n- Is the one-line doorkeeper's small hit-rate and get gain real or noise?\n- Would a different aging threshold or probe count, or an aged fingerprint filter, push the hit-rate and put-cost trade-off further?\n\n## Per-policy notes\n\n- DASH segment promotion and admission: policy levers on insert position and eviction order look saturated on the LIRS trace. Admission is the remaining live lever.\n- Second-hit filter (bit-packed, baseline item size): the long-standing base. It was later replaced as the incumbent by the doorkeeper family.\n- Doorkeeper ring or table: a size and aging trade-off. Too small or unaged loses hit-rate, too large costs put latency, and one cache line with frequent aging is the current best (adopted as incumbent). Next steps are tuning aging and probe count only, with no edits to get.\n- Ghost or history entries: a one-slot ghost almost never matches a reusable key, so it was not worth its cost.\n"
}
```

### 20:14:39 chat

I wrote `cross-cycle-insights.md` in the cycle 0008 cross-cycle output directory, with the four sections in the required order. It is built only from the ledger digest, with no numbers added.

The main points it carries forward:
- **Recurring wins:** Admission filtering is the only lever that has kept paying off, and the one-cache-line aged doorkeeper from cycle 0008 is the current best trade-off.
- **Recurring failures:** Blind hot-path edits, richer signals, and insert-position tweaks have not paid off. "Winner" status is also weak evidence, because flat hit-rate and metadata qualify a variant even when latency regresses.
- **Open questions:** The latency noise floor has been asked about for five cycles and is still unmeasured, so small latency claims can't be trusted yet. Profiling and a second trace are also still undone.
- **Per-policy notes:** These cover the DASH segment promotion and admission levers, the second-hit filter, the doorkeeper family, and the ghost entries.


---

- finished: 2026-10-20T20:14:39+00:00
- duration: 14.40s
- status: ok
- input_tokens: 4
- output_tokens: 1529
- cache_read_tokens: 39750
- cache_write_tokens: 11484
- cost_usd: 0.065955
