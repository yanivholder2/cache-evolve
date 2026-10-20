# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-10-20T15:52:43+00:00

## timeline

### 15:52:55 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Cutting repeated hashing cost (sketch hashing early on, then the map hasher) was the only speed lever that paid off clearly. Gains showed in the put tail, with hit-rate unchanged.\n- Changes that left admission (strict greater-than, LRU-head victim, conservative sketch) untouched kept hit-rate flat. That makes them safe vehicles for speed work.\n- Sketch simplification on the baseline lineage (single hash, small counters) helped speed and footprint early. It did not stack on later lineages.\n\n## Recurring failures\n\n- Hit-rate levers have all failed on workload-a: aging (including long periods), small and smaller windows, wider and narrower sketches, doorkeeper, conservative update, victim sampling, tie-admitting, and segmented main LRU. Hit-rate looks plateaued for this design.\n- Footprint cuts (u32 links, compact Node, halved sketch, field removal) rarely turned into speed. The working set is small, so locality gains are absent and conversion or unpacking overhead is paid on every op. Padding also absorbed some Node savings.\n- Added state or hot-path work (doorkeeper bitset, extra fields, inline hints) raised tail latency and grew the struct.\n- Blind hot-path micro-tweaks (pre-sizing, inlining, branch hints, early-exit) were flat or worse. Allocation and call overhead are not the bottleneck.\n- Small latency deltas are often noise. Several \"gains\" came from untouched code paths, and the Winner label mostly reflects the selection rule.\n- The metadata headline tracks Node size only, so sketch heap changes are invisible there.\n- Many insights were written without a source diff, so attributions are partly inferred.\n\n## Open questions\n\n- Where do get and put actually spend time (map lookup, relinking, sketch estimate)? No profile exists.\n- Is any hit-rate lever left on workload-a, or is it at its ceiling? A materially different mechanism, such as an adaptive window, is untested.\n- Are the incumbents at the floor for this architecture, so that further gains need a different data structure or map?\n- Does an earlier speed gain (cycle 0007's segmented variant, cycle 0004's hasher) survive repeat runs?\n- Is there any trace with drift or larger frequency gaps that would justify aging or revisit the window?\n\n## Per-policy notes\n\n- TinyLFU (sketch plus window plus LRU main) is the only policy in the ledger. Its sketch is saturated for accuracy, and its admission rule is well matched to workload-a.\n- Window: a small window is fine, and shrinking it further hurt put and hit-rate slightly. Do not tune it as a standalone lever.\n- Sketch: low-precision, single-row variants trade hit-rate for footprint. Width, update rule and increment cost are exhausted.\n- Segmented main LRU adds no hit-rate over sketch admission. Its apparent speed gain was not reproduced by the compact Node alone.\n- Next cycles should obtain a profile first, name the cost each hypothesis targets, and use repeat runs. Avoid building on cycle 0008 v002 without auditing its diff, since it shows unexplained size growth.\n"
}
```

### 15:53:03 chat

I wrote `cross-cycle-insights.md` in the output dir, with the four required sections in order. It's regenerated from the eight-cycle ledger digest alone, so nothing in it comes from source code or per-cycle artifacts. It uses no specific numbers.

The main conclusions it records:
- **Wins:** Cutting repeated hashing cost is the only speed lever that clearly paid off. Changes that leave admission alone keep hit-rate flat.
- **Failures:** Every hit-rate lever tried on workload-a has failed. Footprint cuts rarely turned into speed, and small latency deltas are often noise.
- **Open questions:** There is no profile of where get and put spend time. It's unclear whether any hit-rate lever remains. It's also unclear whether the incumbents are already at the floor for this design.
- **Per-policy notes:** TinyLFU is the only policy in the ledger. The notes cover its window, sketch and segmented main LRU, and say to profile first and use repeat runs. They also warn against building on cycle 0008 v002 without auditing its diff.


---

- finished: 2026-10-20T15:53:03+00:00
- duration: 19.97s
- status: ok
- input_tokens: 4
- output_tokens: 1568
- cache_read_tokens: 40334
- cache_write_tokens: 12092
- cost_usd: 0.068894
