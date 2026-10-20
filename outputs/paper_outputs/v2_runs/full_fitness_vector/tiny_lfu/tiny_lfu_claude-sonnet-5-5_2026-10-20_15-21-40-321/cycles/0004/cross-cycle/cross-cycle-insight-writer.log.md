# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-10-20T15:36:57+00:00

## timeline

### 15:37:16 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Cutting per-op cost in the existing machinery pays off: single hash, cheaper index derivation, cheap map hasher. These keep helping speed and tail latency with hit-rate unchanged.\n- Footprint reductions that leave counters, conservative update and admission untouched (u8 counters, u32 links) keep hit-rate flat. The nibble-packed sketch shrinks the footprint further but costs hit-rate.\n- The map hasher was the largest remaining latency cost, and swapping it recovered the time lost to the compact layout while keeping the size gain.\n- Gains that show in headline dimensions (latency, Node size) are the ones worth pursuing.\n\n## Recurring failures\n\n- Aging or halving, at any period, failed twice. workload-a has stable frequencies, so decay discards useful history.\n- A window, a doorkeeper, a wider sketch, a wider victim sample and tie-admitting `>=` all failed to lift hit-rate. Some lowered it. Sketch accuracy looks saturated, and strict greater-than admission with the LRU head as victim suits this trace.\n- Extra state or hot-path work (bitsets, segment lookups, cached fields) raises tail latency and grows the struct without a matching benefit.\n- Shrinking sketch precision too far (4-bit, one row, tight sizing) trades hit-rate for footprint.\n- Small latency deltas are often noise or code-layout effects. Several unexplained moves have been seen, so single-run deltas are unreliable.\n- The headline metadata number tracks Node size only. Sketch heap changes, both savings and costs, are invisible in it.\n- Several attributions were inferred without a source diff, so they are not confirmed.\n\n## Open questions\n\n- Is any hit-rate lever left on workload-a? Admission, victim choice, tie rules, aging, windows and sketch accuracy are all exhausted, so a different mechanism is needed.\n- Do the compact nibble sketch, the cheap hashing and the cheap map hasher stack without losing hit-rate? How much accuracy does the nibble layout need back, and which part of its loss comes from the cap, the single row or the sizing?\n- Which part of the map-hasher change accounts for its gain: the hasher or the MRU skip? Does it hold under repeat measurement?\n- Is there a trace with drift or large frequency gaps where aging, a window or the u8 cap would matter?\n- How much latency variance is noise, and is repeated measurement needed?\n\n## Per-policy notes\n\n- TinyLFU (the only policy so far): the baseline's design (never-decaying counters, strict-greater admission, LRU-head victim) is hard to beat on hit-rate on workload-a. Progress has come from making it cheaper and smaller.\n- Current best lineage: single-hash conservative sketch, then the compact u32 link layout, then the cheap map hasher with the MRU short-circuit. The nibble sketch and the cached-hash speedup are not yet combined with it.\n"
}
```

### 15:37:26 chat

I wrote `cross-cycle-insights.md` to the output dir, with the four sections in the required order. I didn't re-read it after writing.

Across the four cycles, speed and footprint changes kept paying off and hit-rate ideas kept failing:

- **Recurring wins:** Cheaper hashing and the compact layout paid off. The cheap map hasher recovered the per-op time that cycle 0003's compact layout had cost.
- **Recurring failures:**
  - Aging, windows, doorkeepers, wider sketches, wider victim samples and tie-admitting comparisons all failed to lift hit-rate.
  - Small latency deltas looked unreliable.
  - Several attributions were inferred without a source diff.
- **Open questions:**
  - Whether any hit-rate lever is left on workload-a.
  - Whether the compact sketch and the cheaper hashing stack without losing hit-rate.
  - Which part of the map-hasher change accounts for its gain.
- **Per-policy notes:** TinyLFU is the only policy so far. The baseline's admission design is hard to beat on hit-rate, so progress has come from making it cheaper and smaller.

The file uses only qualitative terms and no numbers.


---

- finished: 2026-10-20T15:37:27+00:00
- duration: 29.25s
- status: ok
- input_tokens: 4
- output_tokens: 1536
- cache_read_tokens: 36877
- cache_write_tokens: 8555
- cost_usd: 0.053734
