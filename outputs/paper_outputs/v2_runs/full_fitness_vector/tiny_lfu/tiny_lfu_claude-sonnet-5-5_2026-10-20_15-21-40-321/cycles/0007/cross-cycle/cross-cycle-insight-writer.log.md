# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-20T15:48:29+00:00

## timeline

### 15:48:40 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Footprint and locality reductions (compact Node, link layout, packed segment bit) are the only lever that has moved headline dimensions. Padding and alignment set a floor, so further gains need a layout change that crosses an alignment boundary.\n- Cheaper hashing is the clearest speed lever, both in the sketch (cycles 1-2) and in the map hasher (cycle 4). Each win left hit-rate flat.\n- Changes that make an existing component cheaper without altering admission semantics keep hit-rate flat and sometimes help speed. Changes that add mechanism rarely do.\n\n## Recurring failures\n\n- Hit-rate levers have all failed on workload-a. These were aging (including a long period), a small window, a doorkeeper, a wider sketch, conservative update, a wider victim sample, tie-admitting comparison and a segmented main LRU. Frequencies are stable, often tied, and the baseline strict-greater admission already fits the trace.\n- Sketch-internal tweaks are exhausted: width, counter size, hash derivation, early-exit and conservative update. The single-hash speedup was lineage-specific and did not stack on stronger parents.\n- Adding state or hot-path work (doorkeeper bitset, extra fields, segment lookup) raises tail latency and grows the struct.\n- Micro-optimizations such as pre-sizing, forced inlining, branch hints and removing a cached Node field gave nothing. Allocation and call overhead are not bottlenecks.\n- Small per-op latency deltas have repeatedly proved to be noise, including improvements with no plausible code cause. Many \"wins\" are unverified.\n- The metadata headline tracks Node size only, so sketch heap changes are invisible there. Many insights were also written without a source diff, so attributions are inferred.\n\n## Open questions\n\n- Where do `get` and `put` actually spend time? No profile exists, and the non-sketch hot path (map lookup, relinking, bounds checks) is untested.\n- Does the compact Node alone reproduce the speed gain on the best incumbent, without segmentation, and does it survive repeat runs?\n- Is any hit-rate lever left on workload-a, for example window sizing, a different admission decision, or a drifting trace that would justify decay?\n- Do the best pieces stack (cheap map hasher with MRU skip, compact links, compact sketch) without losing hit-rate?\n- How much of the observed latency variance is noise, and what repeat-run protocol is needed?\n\n## Per-policy notes\n\n- TinyLFU (sketch admission with LRU main): the baseline design is near a local optimum for hit-rate on workload-a. The remaining headroom is per-op cost and footprint.\n- Best speed and size lineage: compact u32-link layout, then cheap map hasher with MRU short-circuit. Merge compact Node packing onto it before trying anything else.\n- Sketch: a single hash with u8 counters and conservative update is the established cheap form. Nibble packing trades hit-rate for footprint. Width and accuracy changes are saturated.\n- Window and segmented LRU variants: no hit-rate gain so far. Any retry needs a rationale tied to a specific trace property.\n"
}
```

### 15:48:44 chat

I wrote `cross-cycle-insights.md` in the cycle's `cross-cycle` output dir. It has the four fixed sections in order, built only from the seven-cycle ledger digest.

- **Recurring wins:** Footprint and locality cuts (compact Node, link layout) and cheaper hashing are the levers that have paid off. Both left hit-rate flat.
- **Recurring failures:** Every hit-rate lever tried on workload-a has failed, and sketch-internal tweaks are exhausted. Small latency deltas keep turning out to be noise.
- **Open questions:** There is still no profile of where `get` and `put` spend time. It is also unclear whether any hit-rate lever remains, and whether the best pieces stack.
- **Per-policy notes:** TinyLFU is near a local optimum for hit-rate on workload-a. The best speed and size lineage is the compact layout plus the cheap map hasher.

I didn't read the previous insights file, and I used no numbers beyond what the digest states qualitatively.


---

- finished: 2026-10-20T15:48:44+00:00
- duration: 15.66s
- status: ok
- input_tokens: 4
- output_tokens: 1540
- cache_read_tokens: 39477
- cache_write_tokens: 11223
- cost_usd: 0.064966
