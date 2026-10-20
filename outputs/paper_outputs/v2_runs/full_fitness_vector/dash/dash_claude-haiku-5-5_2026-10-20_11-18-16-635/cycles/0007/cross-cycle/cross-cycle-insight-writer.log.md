# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-20T11:59:52+00:00

## timeline

### 12:00:01 thinking



### 12:00:07 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-cycle insights (cycles 0001–0007)\n\n## Recurring wins\n\n- Frequency-aware victim selection by a saturating access counter, with a full scan for the lowest-counter resident and no decay, is the only hit-rate lever that has shown a gain in more than one cycle (0002 v003, 0004 v002, 0007 v003 partial). Its gain disappears when the search is sampled or restricted to stash victims (0005).\n- The deferred stash-to-target promotion has kept its hit-rate gain across cycles (0003 v002, 0004 v001). Its cost sits in first-hit bookkeeping on stash hits, and cheaper bookkeeping recovered most of the get-tail regression (0004).\n- Layout and replacement are separable. Repacking records changed segment or metadata size without moving eviction order, and eviction changes did not change size (0004, 0006, 0007).\n\n## Recurring failures\n\n- Insert-position and admission gating keeps losing hit rate. MRU promotion displaced hot residents (0001), gating front-insert on bucket fullness lost hit rate (0002 v001), blocking first-touch admission lost hit rate (0003 v003), and ghost-ring re-admission lost a small amount (0006 v003). Protection works better at eviction than at insert.\n- Shortening or cheapening the victim search does not make puts cheaper. Early exit (0007 v001), two-choice sampling (0005 v001), and incremental minimum caching (0006 v001) all failed to lower put latency. The put cost seems to come from bookkeeping on the eviction branch, not from scan length.\n- Decaying counters lowers hit rate (0007 v002). The never-decaying counter appears to be doing the retention work.\n- Stash occupancy checks and fast paths add latency rather than removing it (0002 v002), which suggests stash buckets are rarely empty on these traces.\n- Recency tie-breaking among equal counters changes no hit rate and adds cost (0003 v001).\n- Packed records add decode work on the get and promotion paths, so a smaller segment alone is not a win (0006 v002).\n- Per-op latency claims keep outrunning their mechanisms. Get and put movements in both directions have been attributed without source confirmation in nearly every cycle, so none should be credited until re-measured.\n\n## Open questions\n\n- Does full-scan frequency-aware eviction keep its hit-rate gain on a second, non-workload-a trace? This check was first requested in cycle 0004 and has been deferred since.\n- How do the stash-dominant and scan-heavy traces that earlier cycles asked for behave? No cycle has measured either, so no directional claim generalises yet.\n- Does the counter-plus-packed-record combination (0007 v003) keep its hit-rate gain on another trace, and is the counter stored in spare bits or in the packed slot?\n- What is the source of the put-side cost, and does it grow with bucket size?\n- Does the ghost ring's bookkeeping show up in metadata size once its bytes are counted?\n- What is the Winner basis for variants that record hit-rate or per-op regressions with no offsetting gain (0001 v001, 0002 v001–v002, 0003 v001–v002, 0005 all, 0006 all, 0007 v001–v002)? It is not visible in the insights.\n\n## Per-policy notes\n\n- **Victim selection (frequency counter):** Full-scan lowest-counter, non-decaying, is the working base. Sampled, stash-only, incrementally cached, and decayed variants have all lost ground, and early exit changed nothing. Only workload-a has shown the gain.\n- **Insert / promotion (MRU, fullness gating, first-touch admission):** Every variant that restricted or repositioned inserts lost hit rate or gained nothing. Treat this family as closed unless paired with a guard against displacing hot target residents.\n- **Stash-to-target deferred promotion:** Keeps its hit-rate gain. Stacking the counter on stash victims hurt hit rate and both tails, so the two should be kept separate until re-measured.\n- **Ghost-ring re-admission:** Lost hit rate and added put-path cost with no offsetting gain. Drop unless a later variant shows a gain and reports its metadata cost.\n- **Layout (not a policy):** Packing changes footprint without moving eviction. Pursue only if the packed form removes decode cost from the get and promotion paths.\n"
}
```

### 12:00:10 chat

I wrote `cross-cycle-insights.md` to the cycle 0007 cross-cycle output directory. It has the four required sections in order, built only from the inlined ledger digest. I did not read any previous version of the file.

The main signals it records:

- **Recurring win:** Full-scan lowest-counter victim selection with no decay is the only hit-rate lever that has shown a gain in more than one cycle. It has only been measured on workload-a.
- **Recurring win:** The deferred stash-to-target promotion has kept its hit-rate gain, and cheaper first-hit bookkeeping recovered most of its get-tail regression.
- **Recurring failure:** Insert-position and admission changes keep losing hit rate, so protection works better at eviction than at insert.
- **Recurring failure:** Sampling, early exit, incremental caching, and decay all failed to make puts cheaper or kept hit rate from holding.
- **Open question:** The second-trace check for the counter-based eviction has been deferred since cycle 0004. Neither the stash-dominant nor the scan-heavy trace has been measured.
- **Open question:** Many variants are labelled Winners despite recorded regressions, and the digest does not show the Winner basis.


---

- finished: 2026-10-20T12:00:11+00:00
- duration: 18.32s
- status: ok
- input_tokens: 4
- output_tokens: 3483
- cache_read_tokens: 40016
- cache_write_tokens: 13636
- cost_usd: 0.004869
