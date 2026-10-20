# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-20T15:44:35+00:00

## timeline

### 15:44:45 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Removing repeated hashing from the sketch path (single hash, cheaper mixing, cached hash) was the clearest early speed gain, mostly on put tail. It held up on the baseline lineage but did not stack on stronger lineages (cycle 0006).\n- Replacing the map hasher (with MRU short-circuit on hit) was the largest later latency win. It recovered the per-op cost of the compact link layout and kept that layout's size gain, with hit-rate unchanged.\n- Compact layouts (u32 links, u8/nibble counters) cut footprint. Without extra hot-path work they leave hit-rate flat. Sketch heap savings are invisible in the headline metadata number, which tracks Node size.\n- Changes that make existing machinery cheaper, rather than adding mechanism, are the only ones that have paid off.\n\n## Recurring failures\n\n- Hit-rate levers on workload-a have all failed: aging (twice, including a 10x longer period), small window, wider victim sample, tie-admitting `>=`, doorkeeper, wider sketch, conservative update. Strict greater-than admission with the LRU head as victim fits this trace (low, often tied, stable frequencies).\n- Adding state or hot-path work (doorkeeper bitset, window, segment lookup, halving counters) raises tail latency and grows the struct without a benefit.\n- Sketch tweaks of every kind (accuracy, width, increment cost, early-exit) are exhausted. Counters rarely saturate, so the early-exit rarely fires.\n- Node field removal did not shrink the struct because alignment absorbed it, and it moved work onto the eviction path.\n- Small per-op deltas, including ones in untouched code, recur as noise. Several apparent gains and losses are not trustworthy without repeat runs.\n- Shrinking sketch precision too far (4-bit, one row, tight sizing) costs hit-rate noticeably.\n- Many insights were written without a source diff, so attributions are inferred.\n\n## Open questions\n\n- Where does `get` spend its time on the incumbent: map lookup, list relinking, sketch estimate, allocation, bounds checks? No profile exists.\n- Are the incumbents at the floor for this design?\n- Is there any hit-rate lever left on workload-a? A structurally different mechanism is needed (segmented layout, larger window), or a trace with drift to justify decay.\n- Which part of the early single-hash gain was real, and why did it not stack on the stronger lineage?\n- Can the compact nibble sketch recover its hit-rate loss cheaply, and does it combine with the cheap map hasher?\n- How much latency variance is noise? Repeat or larger measurements are needed.\n\n## Per-policy notes\n\n- TinyLFU (the only policy so far): the cycle-0004 v001 lineage (cheap map hasher, MRU skip, compact links) and the cycle-0004 v003 incumbent are the current parents. Baseline-rooted variants have shown gains that did not transfer to them.\n- Admission by sketch estimate, strictly greater than the LRU-head victim, has resisted every change. Treat it as fixed unless a new mechanism is proposed.\n- Next hypotheses should target the non-sketch hot path or a structurally different admission design, and name the cost they expect to move.\n"
}
```

### 15:44:49 chat

I wrote `cross-cycle-insights.md` in the cycle 0006 cross-cycle output directory. It has the four required sections in order, and I built it only from the six-cycle ledger digest.

- **Recurring wins:**
  - Removing repeated hashing helped on the baseline lineage, but it did not stack on stronger lineages.
  - The cheap map hasher with the MRU skip was the largest later latency win.
  - Compact layouts cut footprint while leaving hit-rate flat.
  - Only changes that make existing machinery cheaper have paid off.
- **Recurring failures:**
  - Every hit-rate lever tried on workload-a has failed: aging, a small window, a wider victim sample, tie-admitting, the doorkeeper, a wider sketch and conservative update.
  - Added hot-path state raises tail latency and grows the struct.
  - Sketch tweaks of every kind are exhausted.
  - Removing a Node field did not shrink the struct, because alignment absorbed it.
  - Small per-op latency deltas keep turning out to be noise.
- **Open questions:**
  - Where `get` actually spends its time (no profile exists).
  - Whether the incumbents are at the floor for this design.
  - Whether any hit-rate lever is left on workload-a.
  - Why the single-hash gain did not stack on the stronger lineage.
  - How much latency variance is noise.
- **Per-policy notes:**
  - TinyLFU is the only policy so far.
  - The cycle-0004 v001 and v003 lineages are the current parents.
  - The admission rule (sketch estimate strictly greater than the LRU-head victim) is treated as fixed.
  - Next hypotheses should target the non-sketch hot path or a structurally different admission design.

I used no numbers beyond cycle and variant labels.


---

- finished: 2026-10-20T15:44:50+00:00
- duration: 15.34s
- status: ok
- input_tokens: 4
- output_tokens: 1848
- cache_read_tokens: 38655
- cache_write_tokens: 10438
- cost_usd: 0.064742
