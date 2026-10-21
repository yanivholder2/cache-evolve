# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-21T09:45:56+00:00

## timeline

### 09:46:16 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (cycles 0001–0010)\n\n## Recurring wins\n\n- **A strict newcomer filter is the main hit-rate lever on workload-a (cycles 1–4).** New keys start with little protection and must earn more with a second hit. Making this filter stricter won in each of the first three cycles, on both the LRU line and the CLOCK line. It peaked at about one slot (one miss) of grace: pure LIP went too far.\n- **Extra protection only works when it is capped and rare (cycles 4, 5, 8).** A small, recent ghost ring that gives returning keys one extra sweep pass (counter 1) helped on CLOCK and, by a smaller margin, on LRU. The same small cap is optimal for the first-hit step on the hit side.\n- **Cost-only changes keep hit rate exactly (cycles 7, 8, 9, 10).** Examples: the fingerprint ghost ring, the item shrink, the lower ceiling, packed per-bucket counters, and the SWAR tag filter. They are safe because the policy paths never read the state they remove or move. Keep metadata in lockstep with `items`, especially across the shift in `remove_at`.\n- **The ghost ring is only a membership test.** A 16-bit fingerprint ring loses nothing on either line, so both lines can share one representation.\n- **Current base:** the CLOCK/GCLOCK base, adopted in cycle 0009 (v002). It uses strict hand-on-newcomer filtering, a 4-entry unfiltered fingerprint ghost ring, a +1 hit step with a ceiling of 3, and a compact item. Cycle 0010's v002 (packed counters) and v003 (tag filter) match its behaviour.\n\n## Recurring failures\n\n- **Big rewards for weak signals (cycles 2, 7).** Promoting a ghost hit straight to MRU, or granting admission from a broad, collision-prone frequency sketch, weakens the newcomer filter and loses hit rate.\n- **Frequency-based protection on top of the filter (cycles 2–4, 7, 8).** Frequency-based victim choice, SLRU-style promotion, a larger first-hit step, and sketch-based admission all added nothing or hurt, because the filter already removes keys that are used only once.\n- **Single-knob tweaks once the design was bracketed (cycles 5–8).** Each of these peaked at its current setting and lost when moved either way: depth (1/8), ghost ring size (4), ghost-ring admission filters (both lines), ghost-hit insert depth on LRU (1/4), newcomer leniency, hit step, and ceiling.\n- **Adaptive duels (cycles 5, 6).** A self-referential duel never chose leniency. A DIP-style duel with fixed leaders still lost, and its lenient leaders cost hits just by running.\n- **Placement levers (cycles 9, 10).** Two-choice or alternate buckets break the slot-legality invariant. A better-mixing bucket hash changed nothing, so the remaining misses are capacity or in-bucket misses, not conflicts.\n- **The evaluation gap, every cycle.** Per-op-time and metadata-size have never been recorded. For six cycles straight (5–10), no variant improved hit rate, and the winners were picked on cost savings that nobody measured. Hit-rate-only cycles on workload-a no longer tell candidates apart.\n\n## Open questions\n\n- Can metadata-size and per-op-time actually be recorded? Every pending decision depends on it: ceiling 2, a 3-entry ring, fingerprint hashing versus memory saved, the tag filter versus direct key compares, and CLOCK+ghost versus LRU+ghost as the main line.\n- Does the aggressive filter plus capped ghosts generalise to a second trace (longer reuse distances) or another cache size? Have any of the small losses since cycle 5 gone beyond noise?\n- Cycle 0010 left known leaks:\n  - v002's side storage is fatter than planned. An enum or conditional layout might fix it.\n  - v003 re-hashes the evicted key on the miss path. Storing fingerprints might fix it.\n  - Can v002 and v003 be combined without breaking each other's lockstep invariants?\n- Is there anything left to compact, such as a narrower counter or an unused hash on large buckets?\n\n## Per-policy notes\n\n- **CLOCK/GCLOCK (main line):**\n  - Leaving the hand on the newcomer closed most of the gap to deep-insertion LRU.\n  - Capped ghost admission (cycle 4) made it the leader.\n  - Every knob has been bracketed: ring size, ring admission, leniency, hit step, ceiling.\n  - Treat it as final for hit rate. Future work here should be cost and representation only.\n- **LRU with deep insertion (reference line):**\n  - The best insert depth is 1/8.\n  - Ghost hits help slightly when inserted at 1/4. Going deeper does nothing, and filtering ring admission hurts.\n  - It shifts elements on every miss, so its latency is suspect but unmeasured. Keep it only as a comparison point.\n- **Ghost rings across the two lines:**\n  - The representation carries over between lines: fingerprints work on both.\n  - Lessons about which keys the ring captures do not carry over, because each line feeds its ring a different set of evicted keys.\n  - On both lines, keep the ring unfiltered.\n- **Baseline ClassicLRU and wall-clock state:** the per-item timestamp is never used for decisions. Removing it is safe, and wall-clock ordering ideas are retired.\n"
}
```

### 09:46:22 thinking



### 09:46:22 chat

I've regenerated `cross-cycle-insights.md` from scratch from the ten-cycle ledger, with the four required sections in the required order.

- **The main finding:** on workload-a, hit rate depends on a strict filter for newly inserted keys. Extra protection beyond that filter only helps when it is small and given rarely, such as the capped 4-entry ghost ring.
- **Hit rate has stalled:** the design has been fully bracketed, and no variant has improved hit rate for six cycles (0005–0010). Every winner since then was picked for cost savings that were never measured.
- **Ruled out:**
  - large rewards for weak signals
  - frequency-based protection on top of the filter
  - adaptive duels
  - placement and bucket-hash changes
  - more single-knob tweaks
- **The biggest open issue:** per-op-time and metadata-size are still not recorded. Until they are, the pending decisions (ceiling 2, a 3-entry ring, CLOCK vs. LRU as the main line, and the cycle-0010 v002 vs. v003 candidates) can't be settled. The file also lists checking the results on a second trace or cache size, and the known leaks in both cycle-0010 winners.
- **Per-policy notes:** CLOCK/GCLOCK is the main line and is done for hit rate, so further work there is cost and representation only. Deep-insertion LRU is kept as a reference line. The fingerprint ghost ring works on both lines, but lessons about which keys it should capture don't transfer between them.


---

- finished: 2026-10-21T09:46:23+00:00
- duration: 26.86s
- status: ok
- input_tokens: 4
- output_tokens: 2537
- cache_read_tokens: 43147
- cache_write_tokens: 16500
- cost_usd: 0.192551
