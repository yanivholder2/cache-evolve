## Recurring wins

- Cutting sketch hashing cost is the one speed lever that has paid off repeatedly: a single shared hash feeding all sketch rows (cycles 0001, 0005, 0009). Cycle 0010's isolation test showed this sharing, not the flat table, produced the large cycle-0009 latency gain.
- Gains show most in put tail and median when hashing or index derivation is removed from the hot path.
- Shared-hash reuse costs only a tiny hit-rate amount (row correlation) compared with the speed it buys.
- Strict greater-than admission with the LRU head as victim, never-decaying counters and a small window have held up as well suited to workload-a.
- Compact sketch layouts (u8/nibble counters, one row) reliably cut sketch heap, though hit-rate and speed effects vary.

## Recurring failures

- Hit-rate has been flat or slightly down in every cohort of ten cycles. Failed levers: aging/halving (twice), window size (small, larger, adaptive), doorkeeper, wider or narrower sketch, conservative update, victim sampling, tie-admitting, segmented main LRU. Treat workload-a hit-rate as a plateau for this architecture.
- Adding state or hot-path work (doorkeeper bitset, victim-estimate cache, epoch bookkeeping, extra fields) raises tail latency and grows the struct.
- Footprint cuts do not turn into speed: compact u32-link Node, halved sketch, field removal all gave equal or worse latency. The working set is small, so locality gains are absent, and conversion/unpacking overhead is paid per op. Alignment can absorb Node field savings.
- Baseline-only speedups often fail to stack on stronger lineages (single-hash derivation in 0006, flat table in 0010). Always check transfer.
- Blind micro-optimizations (inline/branch hints, pre-sizing, relink skipping) were flat or worse.
- Small latency deltas are frequently noise; several "gains" came from untouched code. Winner status often only reflects the selection rule.
- Several overlays showed unexplained size growth, suggesting stray sizing edits; attributions in many insights were inferred without source diffs.

## Open questions

- Does the single-hash sketch derivation, applied alone on the best hit-rate incumbent (no table swap), reproduce the cycle-0009 speedup? Earlier stacking attempts failed.
- Can sketch hashing get cheaper still (fewer rows, cheaper mixer) without hit-rate loss?
- Where does time actually go in get/put on the incumbents? No profile exists.
- Is workload-a at its hit-rate ceiling, or does a genuinely new mechanism (not sketch, window, victim or tie tweaks) exist?
- How much latency variance is noise? Repeat runs are needed before crediting small deltas.

## Per-policy notes

- Baseline TinyLFU (window + sketch + main LRU, strict > admission): the strongest hit-rate configuration found; most speed wins here came from the hashing path, and they transfer unevenly to derived lineages.
- Sketch: accuracy, width, increment cost, early-exit and aging are exhausted. Only hash sharing mattered for speed. Sketch heap dominates metadata size, and the headline number tracks Node size, so sketch savings are invisible there.
- Window: sizing (small, larger, adaptive) is a closed plateau; no hit-rate gain.
- Main region: segmented probation/protected LRU did not raise hit-rate; its apparent gains were not reproduced by the compact Node alone.
- Storage: the flat open-addressing table plus slab helped on baseline but hurt on the compact-Node lineage; the pointer map is cheap at this cache size.
- Node layout: compact 24-byte, u32-link layouts shrink size but do not speed things up; little room left without crossing alignment boundaries.
