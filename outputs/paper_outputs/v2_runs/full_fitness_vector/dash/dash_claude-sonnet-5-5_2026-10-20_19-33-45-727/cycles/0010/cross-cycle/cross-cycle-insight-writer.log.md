# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-20T20:21:25+00:00

## timeline

### 20:21:37 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- A second-hit admission filter is the one real, repeatedly confirmed win (cycles 0001, 0002, 0004). It keeps one-touch scan items out of hot tiers, and its flag fits in spare bits, so the item struct stays at baseline size.\n- A small, sparse, aged doorkeeper beats an in-item flag on put cost, because it skips insert and eviction work on one-hit keys. Footprint and aging drive this: one cache line, aged often (0006 to 0008).\n- Metadata-free designs hold up: new state packed into spare bits (counters, ref bits, flags) never grew the item.\n- Removing bookkeeping the policy never reads helped once (0004). Later deletions (0005, 0007) gave nothing, so that vein is exhausted.\n\n## Recurring failures\n\n- Hot-path edits to get/put (lookup atomics, early returns, free-slot fast paths, promotion skips, hoisting, tag-first scans, reader-writer locks, resident-key shortcuts) regressed or did not improve latency in seven or more consecutive cycles. Time is evidently not spent there, and the edits add branches or work.\n- Richer admission or recency signals (multi-bit counters, ref bits, ghost fingerprints) added almost no hit-rate once the filter was in place, and cost latency.\n- Stacking position levers on admission levers overlaps and does not add. Insert position alone is exhausted.\n- Lossy, unaged filters (a small ring, a direct-mapped fingerprint table) cut latency but lose hit-rate. Oversized filters recover hit-rate but cost put latency.\n- Doorkeeper tuning (aging threshold, probe count) and warm-up admission are weak levers with small trade-offs.\n- Frequency-counter eviction with aging added per-hit and per-eviction work and gained no hit-rate.\n- \"Winner\" status is weak evidence. Flat metadata plus near-flat hit-rate qualify a variant even when latency regresses.\n- Regressions often appear on untouched paths, which points to layout or measurement noise. Many mechanism explanations are inferred, because authors could not read the source.\n- Empty overlays (0010 v002) are implementation failures and say nothing about the idea.\n\n## Open questions\n\n- What is the run-to-run noise floor for get and put latency on the unchanged incumbent? It has been asked for seven cycles and is still unmeasured. It blocks crediting any latency result.\n- Where do get and put time actually go (locking, memory access, scan, allocation)? Nobody has profiled it.\n- Do the conclusions hold on a second trace with scans or frequency skew? Only the LIRS trace has been used, and frequency-aware or eviction levers may matter elsewhere.\n- Is the one-bit second-chance flag's latency drop (0010 v003) real? Does it stack with the aged doorkeeper?\n- Are the small hit-rate gains (aged doorkeeper, second-chance flag) real or trace noise?\n- Incremental doorkeeper reset remains untested.\n\n## Per-policy notes\n\n- DASH segment with promotion and insert-position policies (0001, 0002): small gains, with position levers overlapping.\n- Second-hit filter, bit-packed (0002, 0004): the base that works. Hit-rate is saturated beyond it on this trace.\n- Doorkeeper-style admission (0006 to 0009): the one-cache-line aged two-probe version (0008 v001) is the current incumbent for policy work. Further tuning is not worthwhile.\n- Eviction (0010): the first eviction-side cycle. A one-bit second-chance flag looks cheap and mildly positive, but is unverified. Frequency counters are not worth it.\n- Locking and bucket layout (0004 to 0006): the benchmark looks uncontended and buckets mostly full, so reader-writer locks, tags and insert fast paths have nothing to exploit.\n"
}
```

### 20:21:42 chat

I wrote `cross-cycle-insights.md` to the output dir, with the four fixed sections in order. I built it only from the ledger digest.

The main points it carries forward:

- **Recurring wins:**
  - **Second-hit filter:** it is the one repeatedly confirmed gain, and it fits in spare bits so the item size stays at baseline.
  - **Aged doorkeeper:** a small, sparse one is cheaper on puts than an in-item flag.
- **Recurring failures:**
  - **Hot-path edits:** these regressed or didn't improve latency in seven or more consecutive cycles.
  - **Extra signals and stacking:** richer signals added little once the filter was in place, and stacking position levers on admission levers didn't add up.
  - **"Winner" status:** it is weak evidence, because flat metadata and near-flat hit-rate qualify a variant even when latency gets worse.
- **Open questions:**
  - **Noise floor:** the run-to-run noise floor for get and put latency has been asked for seven cycles and is still unmeasured.
  - **Profiling:** nobody has profiled where get and put time goes.
  - **Second trace:** only the LIRS trace has been used.
  - **Second-chance flag:** its latency drop in cycle 0010 is unverified.
- **Per-policy notes:** these cover the DASH segment, the second-hit filter, the doorkeeper (the one-cache-line aged version from cycle 0008 is the incumbent), eviction, and locking and bucket layout.

I used qualitative terms throughout and no numbers.


---

- finished: 2026-10-20T20:21:42+00:00
- duration: 16.98s
- status: ok
- input_tokens: 4
- output_tokens: 1971
- cache_read_tokens: 41394
- cache_write_tokens: 13380
- cost_usd: 0.078288
