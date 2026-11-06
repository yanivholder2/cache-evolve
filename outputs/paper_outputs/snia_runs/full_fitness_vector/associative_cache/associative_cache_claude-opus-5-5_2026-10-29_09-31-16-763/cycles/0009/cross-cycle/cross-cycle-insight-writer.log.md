# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-29T11:15:52+00:00

## timeline

### 11:16:13 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0009)\n\n## Recurring wins\n\n- **Scan-resistant insertion (LIP quarter-depth + BIP).** This came out of cycle 1 and is still the best policy after many attempts to beat it. Most of its gain is on wide buckets.\n- **Removing whole units of work beats re-implementing them.** Wins of this kind: the byte-tag pre-filter cut full-key compares (c5), the fused evict+insert rotation removed two memmoves (c6), lazy promotion skipped near-MRU hit moves (c9), and a single multiplicative hash replaced redundant SipHash (c9). Each one improved speed on more than one op or percentile.\n- **Dense, same-stream side data.** Byte tags work where a `u64` key mirror failed. Density and locality decide whether a side array helps or hurts.\n- **Look above the bucket.** Cycles 5–8 tuned in-bucket loops. The cost that turned out to matter in c9 was in the hashing and indexing path, which every op shares.\n- **Mechanical rewrites are semantically safe.** Every layout or speed rewrite that was meant to be hit-rate-neutral matched all cells exactly. The risk is entirely in per-op time.\n- **Shrinking `Item` is the only lever that moves metadata-size** (c4, 24-byte item).\n- **Width-keyed static policy split.** It costs no metadata, and narrow cells stay bit-identical while wide buckets get their own setting.\n\n## Recurring failures\n\n- **Reuse/adaptation signals, five tries so far:** BIP blind period, a 16-key ghost ring, a hashed ghost bit set, adaptive depth from hit position, and a 1-bit second chance. None solved the hm_0-vs-web_0 tradeoff. Small windows miss slow reuse, dense filters saturate, and bits without their own decay go stale.\n- **\"Clever\" replacements for simple inline loops:** `rotate_left(1)`, a variable-length `ptr::copy` on the hit path, and a SWAR tag scan. All were slower at p50 than the loops the compiler already handles well.\n- **Reordering on hits or changing scan order:** reverse scan, swap-halfway, and self-organization did nothing for get p50 or made it worse.\n- **Adding state or writes to skip cheap work:** a miss memo and writes on the get path. Both dirtied cache lines and grew the stride.\n- **Extra work on the put-miss path:** mid-bucket insert, unbounded sweeps, per-miss hashing, a dynamic victim check, and a global atomic. Each one raised put latency.\n- **Expecting a speed win from removing the atomic clock.** It is neutral on the tag/fused base. Keep the removal for correctness only.\n- **BIP period tuning on wide buckets.** Going 1/32 → 1/64 → 1/128 trades web_0 against hm_0 roughly one-for-one, so there is no sweet spot. This probe is closed.\n- **Trusting single-run per-op numbers.** Drift showed up in c4, c6, c7 and c8, including slowdowns in variants that only removed work.\n\n## Open questions\n\n- Do c9's single multiplicative hash and lazy promotion stack? Check with repeated same-session runs against both parents.\n- Can the lazy-promotion window widen to `n/4` before it reaches the insertion region and starts changing eviction order?\n- Where does the remaining cost sit: the full-key compare, the fused rotation's quarter-shift plus tag mirror, or bucket indexing (`%` vs mask)? Profile this before forming new hypotheses.\n- Does stride cost anything on its own? This needs a pad-only control (80 → 88 bytes with no behaviour change).\n- Can tags be extended to the CLOCK path?\n- Is any reuse signal still worth a try? Only one with a per-item probationary bit, symmetric grow/decay, and placement outside the admission region.\n\n## Per-policy notes\n\n- **ClassicLRU / LIP / BIP (main line):** The current base is quarter-depth LIP with width-keyed BIP (1/32 narrow, 1/64 wide), byte tags, fused rotation, a clock-free 24-byte `Item`, an 80-byte stride, and now one multiplicative hash plus top-eighth lazy promotion. Keep the policy frozen apart from the promotion window.\n- **CLOCK / GCLOCK:** Clearing the reference bit on insert gives scan resistance comparable to LIP. A 2-bit GCLOCK adds a little hit rate. Bound the sweep (16-slot cap) to keep the put tail down. The hit-path write must stay a single byte store, and any reordering on hits loses CLOCK's advantage. This policy has not been revisited since c3 and has no tag support.\n- **Frequency-aware victim window (c1 v003):** Its result was bit-identical to the baseline, and it was never confirmed that the harness ran its path. Unresolved and deprioritized.\n- **Ghost / second-chance variants:** Each improves one workload and costs the other, wide-bucket web_0 or small hm_0. Don't build another without independent decay.\n- **Metric caveat (all policies):** metadata-size measures only the `Item` layout. Flat readings do not mean per-bucket state (tags, ghosts, counters) is free.\n"
}
```

### 11:16:23 chat

I've rewritten `cross-cycle-insights.md` from scratch from the ledger for cycles 0001–0009, using the four sections in the required order. It doesn't use the previous version of the file or invent any numbers.

- **Recurring wins:** Quarter-depth insertion with a 1-in-N MRU admission rate (LIP + BIP) is still the best policy since cycle 1. The changes that sped things up all removed whole pieces of work instead of rewriting them: byte tags, the fused evict-and-insert rotation, skipping moves for hits near the top of the bucket, and the single multiplicative hash. Mechanical rewrites have never changed hit rate. Shrinking `Item` is the only thing that has moved metadata-size.
- **Recurring failures:**
  - Five reuse or adaptation signals, none of which solved the trade-off between the two workloads (hm_0 and web_0).
  - "Clever" loop replacements (`rotate_left`, a variable-length `ptr::copy` on hits, a SWAR tag scan).
  - Reordering on hits, or changing scan order.
  - Adding state or writes to skip work that was already cheap.
  - More work on the put path when the key isn't found.
  - Expecting a speed win from removing the atomic clock.
  - Tuning the BIP rate on wide buckets.
  - Trusting per-op timings from a single run.
- **Open questions:** whether cycle 9's two speed wins add up when combined; how wide the hit-skip window can get before it changes eviction order (one-eighth now, a quarter next); where the remaining cost sits; whether a bigger bucket costs anything on its own (needs a padding-only test); and whether tags would work on the CLOCK path.
- **Per-policy notes:** the current LIP/BIP base and its full recipe; CLOCK/GCLOCK, which hasn't been looked at since cycle 3; the frequency-aware victim choice, still unconfirmed because it's unclear the test harness ever ran it; the ghost and second-chance designs; and a warning that metadata-size can't see per-bucket state.


---

- finished: 2026-10-29T11:16:23+00:00
- duration: 31.43s
- status: ok
- input_tokens: 4
- output_tokens: 2642
- cache_read_tokens: 43897
- cache_write_tokens: 15908
- cost_usd: 0.188899
