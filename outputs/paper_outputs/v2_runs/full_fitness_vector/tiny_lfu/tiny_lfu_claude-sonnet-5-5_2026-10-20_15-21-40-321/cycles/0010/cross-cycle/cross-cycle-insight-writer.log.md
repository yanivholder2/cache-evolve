# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-20T16:05:10+00:00

## timeline

### 16:05:24 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Cutting sketch hashing cost is the one speed lever that has paid off repeatedly: a single shared hash feeding all sketch rows (cycles 0001, 0005, 0009). Cycle 0010's isolation test showed this sharing, not the flat table, produced the large cycle-0009 latency gain.\n- Gains show most in put tail and median when hashing or index derivation is removed from the hot path.\n- Shared-hash reuse costs only a tiny hit-rate amount (row correlation) compared with the speed it buys.\n- Strict greater-than admission with the LRU head as victim, never-decaying counters and a small window have held up as well suited to workload-a.\n- Compact sketch layouts (u8/nibble counters, one row) reliably cut sketch heap, though hit-rate and speed effects vary.\n\n## Recurring failures\n\n- Hit-rate has been flat or slightly down in every cohort of ten cycles. Failed levers: aging/halving (twice), window size (small, larger, adaptive), doorkeeper, wider or narrower sketch, conservative update, victim sampling, tie-admitting, segmented main LRU. Treat workload-a hit-rate as a plateau for this architecture.\n- Adding state or hot-path work (doorkeeper bitset, victim-estimate cache, epoch bookkeeping, extra fields) raises tail latency and grows the struct.\n- Footprint cuts do not turn into speed: compact u32-link Node, halved sketch, field removal all gave equal or worse latency. The working set is small, so locality gains are absent, and conversion/unpacking overhead is paid per op. Alignment can absorb Node field savings.\n- Baseline-only speedups often fail to stack on stronger lineages (single-hash derivation in 0006, flat table in 0010). Always check transfer.\n- Blind micro-optimizations (inline/branch hints, pre-sizing, relink skipping) were flat or worse.\n- Small latency deltas are frequently noise; several \"gains\" came from untouched code. Winner status often only reflects the selection rule.\n- Several overlays showed unexplained size growth, suggesting stray sizing edits; attributions in many insights were inferred without source diffs.\n\n## Open questions\n\n- Does the single-hash sketch derivation, applied alone on the best hit-rate incumbent (no table swap), reproduce the cycle-0009 speedup? Earlier stacking attempts failed.\n- Can sketch hashing get cheaper still (fewer rows, cheaper mixer) without hit-rate loss?\n- Where does time actually go in get/put on the incumbents? No profile exists.\n- Is workload-a at its hit-rate ceiling, or does a genuinely new mechanism (not sketch, window, victim or tie tweaks) exist?\n- How much latency variance is noise? Repeat runs are needed before crediting small deltas.\n\n## Per-policy notes\n\n- Baseline TinyLFU (window + sketch + main LRU, strict > admission): the strongest hit-rate configuration found; most speed wins here came from the hashing path, and they transfer unevenly to derived lineages.\n- Sketch: accuracy, width, increment cost, early-exit and aging are exhausted. Only hash sharing mattered for speed. Sketch heap dominates metadata size, and the headline number tracks Node size, so sketch savings are invisible there.\n- Window: sizing (small, larger, adaptive) is a closed plateau; no hit-rate gain.\n- Main region: segmented probation/protected LRU did not raise hit-rate; its apparent gains were not reproduced by the compact Node alone.\n- Storage: the flat open-addressing table plus slab helped on baseline but hurt on the compact-Node lineage; the pointer map is cheap at this cache size.\n- Node layout: compact 24-byte, u32-link layouts shrink size but do not speed things up; little room left without crossing alignment boundaries.\n"
}
```

### 16:05:28 chat

I wrote `cross-cycle-insights.md` to the cycle 0010 cross-cycle output dir. It has the four fixed sections in order, built only from the ten-cycle ledger digest.

- **Recurring wins:** The one speed lever that keeps paying off is sharing a single hash across the sketch rows. Cycle 0010 showed this, not the flat table, produced the large cycle-0009 latency gain. Compact sketch layouts also reliably cut sketch heap.
- **Recurring failures:** Hit-rate on workload-a has stayed flat or slightly down in every cohort. Aging, window changes, the doorkeeper, sketch width, conservative update, victim sampling and tie-admitting have all failed to move it. The file also records that footprint cuts never became speed. Baseline-only speedups often failed to stack on stronger lineages, and small latency deltas are often noise.
- **Open questions:** Does single-hash derivation alone reproduce the cycle-0009 speedup on the best hit-rate incumbent? Can sketch hashing get cheaper still? Where does time go in get and put, given there is no profile? Is workload-a at its hit-rate ceiling?
- **Per-policy notes:** The notes are grouped by component: baseline TinyLFU, sketch, window, main region, storage and Node layout. They give the status of each.

I used no numbers beyond qualitative terms.


---

- finished: 2026-10-20T16:05:29+00:00
- duration: 18.50s
- status: ok
- input_tokens: 4
- output_tokens: 1920
- cache_read_tokens: 42028
- cache_write_tokens: 14031
- cost_usd: 0.080508
