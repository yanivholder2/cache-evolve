# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-20T15:40:56+00:00

## timeline

### 15:41:06 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Cutting per-op hashing cost is the most reliable speed lever: single hash, shift/bit-sliced index derivation, cheap map hasher, saturation early-exit. It has paid off in cycles 0001, 0002, 0004 and 0005 on independent lineages, with hit-rate left flat.\n- Changes that make existing machinery cheaper without altering counters, update rule or admission keep hit-rate flat. Changes that add mechanism rarely help.\n- Compact layout (u32 links) cuts Node size. Its latency cost was recovered by the cheap map hasher (0004).\n\n## Recurring failures\n\n- Aging or decay (naive, long-period): workload-a has stable frequencies, so decay discards useful history. Failed in 0001 and 0002.\n- Admission-path additions: window, doorkeeper, wider victim sample, tie-admitting `>=`. None lifted hit-rate. Several raised tail latency or lowered hit-rate. Ties are common and loosening protection hurts.\n- Sketch width/precision tuning: widening gives marginal hit-rate at a large hidden heap cost, and narrowing gives a hair of loss with no speedup. 4-bit counters trade hit-rate for footprint. Sketch accuracy looks saturated.\n- Node field removal: padding/alignment absorbed the saving, and recomputation taxed put tail latency.\n- Extra state on the hot path grows the struct and widens the tail.\n- Small latency deltas (a few percent, or gains on untouched paths) have repeatedly proved to be noise. Do not credit them without repeat runs.\n\n## Open questions\n\n- Does the 0005 v003 shift-derived increment stack on the 0004 v001 lineage (cheap hasher, MRU skip, compact links)? Which part matters: early-exit or hash derivation?\n- Can the nibble sketch plus fast hashing be combined, recovering hit-rate with minimal extra accuracy?\n- Is any hit-rate lever left on workload-a? It would need a genuinely new mechanism, not a tweak to admission, victim choice, sketch size or aging.\n- Is there a drifting trace that would justify decay or a window?\n- How large is the latency noise floor, and is repeated measurement needed?\n- Is there any footprint lever left once Node alignment is the floor?\n\n## Per-policy notes\n\n- TinyLFU (the only policy so far, tested on workload-a): the baseline's strict greater-than admission, LRU-head victim, never-decaying counters and immediate admission are close to a local optimum for hit-rate. Gains have come from per-op cost and footprint.\n- The metadata headline tracks Node size, not sketch heap, so sketch savings are invisible there. Judge sketch changes separately.\n- Several insights were written without source diffs, so attributions (for example hash caching versus mixing) are inferred.\n"
}
```

### 15:41:11 chat

I wrote `cross-cycle-insights.md` to the output dir, built only from the ledger digest for cycles 0001–0005. It has the four fixed sections in order, with short bullets and no invented numbers.

- **Recurring wins:** Cutting per-op hashing cost keeps paying off, and so does making existing machinery cheaper without changing admission. The compact layout's latency cost was recovered by the cheap map hasher.
- **Recurring failures:**
  - Aging and decay.
  - Admission-path additions: window, doorkeeper, wider victim sample, tie-admitting comparison.
  - Sketch width and precision tuning.
  - Node field removal.
  - Extra hot-path state.
  - Crediting small latency deltas, which have repeatedly turned out to be noise.
- **Open questions:** These are whether the shift-derived increment stacks on the cycle-0004 v001 lineage, and whether the nibble sketch combines with fast hashing. Others are whether any hit-rate lever is left on workload-a, whether a drifting trace would justify decay, how large the latency noise floor is, and whether any footprint lever remains.
- **Per-policy notes:** The ledger covers only TinyLFU on workload-a, so this section is about that one policy. It notes that the baseline admission rules look close to a local optimum for hit-rate. It also notes that the metadata headline misses sketch heap savings, and that some attributions are inferred because there were no source diffs.


---

- finished: 2026-10-20T15:41:11+00:00
- duration: 15.15s
- status: ok
- input_tokens: 4
- output_tokens: 1583
- cache_read_tokens: 37768
- cache_write_tokens: 9411
- cost_usd: 0.057805
