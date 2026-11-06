# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-10-29T10:17:33+00:00

## timeline

### 10:17:50 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- Scan-resistant insertion is the only hit-rate lever that has paid off clearly. Examples are LIP quarter-depth insertion and a CLOCK reference bit cleared on insert. The gains are largest on wide buckets and were mostly captured in cycle 0001. Later refinements (BIP, GCLOCK levels, ghost filters) only trade small amounts between workloads.\n- Rewrites that keep the layout and the policy logic preserve hit rate exactly (fused rotation, narrowed fields). The current LIP/BIP lineage is a stable base to build on.\n- Bounding worst-case work on the put path, with a capped sweep or a dense absence scan, reliably improves put p99. In every case so far it has also cost something at p50.\n- Shrinking `Item` is the only change that has moved metadata-size (cycle 0004).\n\n## Recurring failures\n\n- Adding work to the put/miss path. This includes mid-bucket inserts, extra structures, hashing, atomics and dynamic calls. Each one has raised put latency. The quarter-depth `Vec::insert` memmove remains the main cost, and new work sits on top of it.\n- Speeding up get by changing scan order, reordering on hit, or using a separate key Vec. None of these moved get p50, and some made it worse through locality loss or extra cache-line writes. Lookup cost follows the number of full-key compares across the wide bucket.\n- Reuse/ghost filters as a replacement for BIP's period. Three designs have failed on the hm_0 vs web_0 tradeoff: a fixed 16-entry ring that was too short, and a hashed 1-bit set that saturated and was reset all at once. Neither beat blind 1/32 admission on both workloads.\n- Treating flat metadata-size as proof a change is free. The metric is blind to per-bucket side state such as key mirrors, ghost rings and bit sets.\n- Trusting per-op-time without a same-session parent baseline. In cycle 0004, a variant whose get path was unchanged from its parent still got slower, so environment drift can hide any real effect.\n\n## Open questions\n\n- The inline fingerprint/tag pre-filter for get p50 has been recommended for three cycles and never tested. Does it cut full-key compares while keeping the hit write a single store?\n- Does fused evict+insert rotation, with victim dispatch decided at construction and a `ptr::copy` shift, actually cut put latency when benchmarked in the same session as its parent?\n- Does the 24-byte `Item` speed up the scan once the global atomic tick is removed (replaced by a per-bucket tick, or no write at all)?\n- Is the quarter-depth insertion approximable without a memmove, for example with segments or a CLOCK-like scheme?\n- Does the v003 frequency-window victim selection from cycle 0001 actually run in the hit-rate harness? This is still unconfirmed.\n- Is any reuse filter worth another attempt? It would need multiple bits per key, gradual aging, a cheap hash and sizing to the bucket. Or should insertion depth / BIP period simply scale with bucket width?\n\n## Per-policy notes\n\n- LIP (quarter-depth insert): strong on scan-heavy workloads (web_0) and wide buckets; weaker gain at the small width. The memmove dominates put cost. A reversed scan gives no measurable get benefit.\n- BIP (LIP + periodic MRU admission): gives a slight hm_0 gain on small buckets but loses some web_0 scan resistance. Removing the string-compare dispatch was not visible in timings. The ghost-ring and hashed-ghost replacements for its period both did worse overall.\n- CLOCK / GCLOCK: the cheapest hit path (a single store). Clear-on-insert gives scan resistance. Capping the sweep at 16 slots fixes the put tail but adds work at put p50. Extra GCLOCK levels add a small, consistent hit-rate gain. Any reordering on hit (swap-halfway) loses its latency advantage.\n- ClassicLRU: the baseline. A per-op `name()` string compare is a pure overhead source and should be decided at construction. The frequency-aware tail-window victim produced results bit-identical to the baseline, so it is unvalidated.\n"
}
```

### 10:17:58 thinking



### 10:17:58 chat

I wrote a fresh `cross-cycle-insights.md` from the four-cycle ledger, using the four required sections in order.

- **Recurring wins:** Inserting new items low in the bucket so one-touch items are evicted first is the only hit-rate change that clearly paid off. Nearly all of that gain came in cycle 0001. Rewrites that keep the data layout and policy logic the same have matched the parent's hit rate exactly. Capping worst-case put work improves put p99 but has always cost some put p50. Shrinking `Item` is the only change that has moved metadata-size.
- **Recurring failures:**
  - Any extra work on the put path has raised put latency.
  - Get p50 hasn't moved with changes to scan order, reordering on hit, or a separate key array. It follows the number of full-key compares.
  - All three reuse/ghost filter designs failed to beat BIP's simple periodic admission on both workloads.
  - Flat metadata-size doesn't mean a change is free, because the metric can't see per-bucket side data.
  - Per-op-time results need a baseline run in the same session; cycle 0004's numbers looked shifted by the benchmark environment.
- **Open questions:**
  - The inline fingerprint/tag pre-filter has been recommended for three cycles and still hasn't been tried.
  - The fused evict-and-insert rotation and the 24-byte `Item` still need a fair benchmark, without the global atomic clock.
  - Can quarter-depth insertion avoid the memmove?
  - Does the hit-rate harness actually run the frequency-window eviction code from cycle 0001?
  - Is another reuse filter worth trying at all?
- **Per-policy notes:** Short notes for LIP, BIP, CLOCK/GCLOCK and ClassicLRU.

Files are in /home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0004/cross-cycle:
- cross-cycle-insights.md


---

- finished: 2026-10-29T10:17:58+00:00
- duration: 25.56s
- status: ok
- input_tokens: 4
- output_tokens: 2258
- cache_read_tokens: 38262
- cache_write_tokens: 9905
- cost_usd: 0.132068
