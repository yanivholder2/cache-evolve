# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-20T20:42:22+00:00

## timeline

### 20:42:45 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Changes that only alter layout or metadata, with no new hot-path work, are the most reliable gains. Examples: fast hashing (c1), compact `u32` links (c2), the region flag packed into a link bit (c5), CLOCK ref bits (c6), and `u8` counters (c9).\n- The only levers that have raised hit rate are those that make cold-key estimates more accurate without coupling the rows: conservative update (c3), wider `u16` counters at the same byte size (c4), and counting each access once (c5). Each gain was small, and the lever is now used up.\n- Behaviour-preserving refactors on the CLOCK base keep hit rate exactly the same every time. That makes the base safe for efficiency work.\n- Counters only need a small range. Narrowing them (`u16`, then `u8`) never changed a duel, because admission is decided by low counts. Shrinking the sketch helps `put` more than `get`, since a full-miss `put` walks the sketch twice.\n- Holding the sketch increment in registers in a single pass is the only `get`-path latency gain with a clear mechanism behind it (c4, c6). Whether it carries over has been inconsistent (it did not in c5).\n\n## Recurring failures\n\n- Aging or decay in any form lowers hit rate on workload-a: 4-bit aging (c2), periodic halving with full-width counters (c4), and the doorkeeper's all-at-once clear (c8). The workload rewards stable, non-decaying frequency.\n- Admission windows of any size hurt: about 1% (c5) and 10% (c6). Any capacity given to unfiltered arrivals goes to keys that are seen once.\n- Changing the duel hurts or does nothing. Ties favouring the newcomer (c3), LRU-tail victim sampling (c2), and taking the minimum of two CLOCK candidates (c7) all loosened admission. Strict `>` against a single victim remains the best rule.\n- Blocked or one-cache-line sketch layouts (c3, c9) break row independence and inflate estimates. Doubling sketch width (c8) added footprint without a measurable hit-rate gain.\n- Adding fixed per-access cost to save on rare paths loses at the median. Failed examples: prefetching, extra hash mixes, recomputed indices, mask math, extra struct fields, word-scan sweeps, fill-phase tweaks and hash-once maps (c6–c9).\n- Reducing hash count does not move latency for `u64` keys (c6, c8). An index table that stores only node indices does worse than SwissTable (c6). Heavy list restructuring such as SLRU (c1) did not pay off.\n- Single-run latency readings, especially p99, are unreliable. Several variants with no code-level cause sped up or slowed down on every op.\n\n## Open questions\n\n- Would packed 4-bit counters, which saturate at 15, keep hit rate exactly the same while cutting metadata further? Or would the nibble masking cost more at the median than the smaller footprint saves?\n- Is `get` p50 at its floor (one SwissTable lookup plus one sketch increment)? It has not moved under changes to relinking, hashing, sketch size or sketch layout.\n- Is workload-a's hit rate at its ceiling for strict-duel TinyLFU? Every admission, window, aging and sketch-accuracy change since c5 has been flat or negative.\n- Does CLOCK's `put` p99 come from the full-miss map path (probe, then remove and insert) rather than from the sweep?\n\n## Per-policy notes\n\n- **TinyLFU admission / CountMin sketch:** Use conservative update, non-decaying saturating counters (now `u8`), independent rows, and count each access once. The sketch fits in L1/L2 at the benchmark sizes, so cache-miss-oriented sketch work has nothing left to win.\n- **LRU (early base):** Exact recency order gave admission no useful information. Restructuring it (SLRU, windows) added cost without adding hit rate.\n- **CLOCK (current base since c6):** Hit rate matches LRU and per-entry metadata is smaller. Long sweeps are rare in steady state, because newcomers enter with a clear bit. Keep the hand and victim choice simple. Keep the cache struct within two cache lines.\n- **W-TinyLFU / windowed variants:** Consistently negative on this workload. Don't revisit them without strong new evidence.\n- **Evaluation practice:** Count how often each path runs before targeting it. Accept a latency claim only if a mechanism explains it and median results repeat side by side against the parent.\n"
}
```

### 20:42:54 chat

I've written `cross-cycle-insights.md` in the cycle 0009 cross-cycle folder. It follows the four fixed sections in order and draws only on the ledger for cycles 1–9.

- **Recurring wins:**
  - Changes that only shrink metadata or layout, with no new work on hot paths, are the most reliable gains. The latest is CLOCK with `u8` counters.
  - Hit rate has moved only when estimates of rarely seen keys got more accurate with the sketch rows kept independent: conservative update, wider counters at the same size, and counting each access once. Each gain was small, and this lever now looks used up.
  - Refactors that don't change behaviour keep hit rate exactly the same on the CLOCK base, so it is safe for efficiency work.
- **Recurring failures:**
  - Aging or decay in any form hurt hit rate, and so did admission windows of every size tried.
  - Every change to the admission duel either hurt or did nothing; strict `>` against a single victim is still best.
  - Packing a key's counters into one block made estimates worse both times it was tried.
  - Adding a fixed cost to every operation to save on rare paths lost at the median, and cutting the number of hashes never moved latency.
  - Single-run latency readings, especially p99, can't be trusted.
- **Open questions:**
  - Would 4-bit counters keep hit rate identical?
  - Has `get` p50 hit its floor?
  - Has workload-a's hit rate hit its ceiling for this design?
  - Does the CLOCK `put` tail come from map work on full misses rather than from the sweep?
- **Per-policy notes:** grouped by the sketch, LRU, CLOCK, windowed variants, and a short note on how to judge latency results.


---

- finished: 2026-10-20T20:42:54+00:00
- duration: 32.48s
- status: ok
- input_tokens: 4
- output_tokens: 2253
- cache_read_tokens: 41239
- cache_write_tokens: 14241
- cost_usd: 0.168228
