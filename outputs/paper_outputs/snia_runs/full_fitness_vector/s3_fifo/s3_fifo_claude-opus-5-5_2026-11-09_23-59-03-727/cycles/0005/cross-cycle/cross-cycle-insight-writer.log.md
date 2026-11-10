# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-11-10T04:22:06+00:00

## timeline

### 04:22:20 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- One change per variant. Bundled changes in cycles 1–2 kept trading hm_0 against web_0. Once changes were isolated (cycles 3–5), the mechanisms could be told apart and the gains could be stacked.\n- Filters that grant credit only for reuse inside main. Gating main-victim ghosts on reuse and resetting freq/reused at graduation each helped hm_0 without hurting web_0 (cycle 3). Merged, they gained at every trace/size point (cycle 4). This is the most reliable lever found so far.\n- Spend the ghost budget selectively, don't enlarge it. Filters that free ghost slots lengthen useful history for free.\n- Structural changes that keep hit rates bit-identical (slab + hash indexes, Fx hasher) are the only per-op-time gains the diff explains. They are a stable base for later policy work.\n- hm_0 responds to main retention: more main residency, main-victim history, a deeper freq cap. Recent gains sit mostly at the small cache size.\n\n## Recurring failures\n\n- Fixed-ratio and adaptive small/main splits (cycles 1, 5). They shift the hm_0 vs web_0-large trade-off without improving the summary. Shrinking the small queue hurts web_0 at both sizes.\n- Resizing or enlarging the ghost (cycles 1, 2, 4). Splitting ghost regions didn't rescue web_0. A bigger bound lets stale reuse back into main, where it evicts live residents.\n- Looser graduation (`freq >= 1`). This was cycle 2's web_0 regression, and later isolation pinned the loss on it.\n- Hash-table layout tweaks aimed at probe count or load factor. A merged index (denser) and pre-sized indexes (sparser) both made tails worse with hit rates unchanged. Footprint and locality matter more than probe count.\n- Timing credit for policy-only diffs. Unexplained per-op-time drift has shown up every cycle and is environmental noise. `put` p99 drifts slightly upward across variants.\n- Per-entry metadata is flat in every cycle while heap use (ghost keys, index overallocation) grows unseen.\n\n## Open questions\n\n- web_0 large-size gap: now looks like a probation-window problem. Test a longer probation (small share above 25%) or easier graduation applied only inside probation, without bringing back the `freq >= 1` regression.\n- Can main-retention gains at the small size (`MAX_FREQ`, threshold) be made size-aware so they also help at the large size, where main looks saturated?\n- Can the ghost index be shrunk (fingerprints, compact keys) to cut `put` tail cost?\n- Should the metadata metric include heap bytes per resident key, including ghost and index memory?\n- Is a pinned, same-machine benchmark environment achievable, so policy-variant timing becomes interpretable?\n\n## Per-policy notes\n\n- S3-FIFO (the only policy explored so far). Current lineage: Fx slab + two-table index → reuse-gated main-victim ghost + credit reset at graduation (`freq > 1` threshold kept) → `MAX_FREQ` 7. Ghost bound stays at `main_size` or below. Small share stays at 25% or above. Raising `MAX_FREQ` further is unlikely to pay off. Implementation work should only shrink tables, starting with the ghost index.\n- Trace behaviour: hm_0 is driven by how main retains keys and is near-saturated at large size. web_0 is driven by how long probation lasts and is sensitive to graduation strictness and small-queue size.\n"
}
```

### 04:22:28 thinking



### 04:22:28 chat

I rewrote `cross-cycle-insights.md` from scratch using only the five-cycle ledger. It has the four required sections in order, and it uses no numbers that aren't in the ledger.

- **Recurring wins:**
  - Changing one thing per variant is what finally separated the two traces' behaviour.
  - Filters that only give credit for reuse inside the main queue are the most reliable lever found so far. They add up when combined.
  - The ghost history should be used more selectively, not made bigger.
  - The only timing gains that hold up come from structural changes that leave hit rates unchanged.
- **Recurring failures:**
  - Splitting capacity between the small and main queues, whether by a fixed ratio or adaptively, only moves gains from one trace to the other.
  - Enlarging or splitting the ghost history didn't help.
  - Loosening the graduation threshold to one hit caused the cycle-2 web_0 loss.
  - Tuning the hash-table layout made tail latency worse.
  - Policy-only changes still don't earn timing credit, and the per-entry metadata number misses growth in heap memory.
- **Open questions:**
  - Can a longer probation window, or easier graduation applied only during probation, close web_0's gap at the large cache size?
  - Can main-retention settings adapt to cache size so they also help at the large size?
  - Would a more compact ghost index cut `put` tail latency?
  - Should the metadata measure count heap memory?
  - Can the benchmark run on a pinned machine so policy timing becomes readable?
- **Per-policy notes:** S3-FIFO is the only policy tried so far. The notes give its current lineage and the limits to keep: ghost history no larger than the main queue, small queue at 25% or more of capacity. They also give each trace's driver: hm_0 depends on how the main queue keeps keys, and web_0 depends on how long new keys stay in probation.


---

- finished: 2026-11-10T04:22:28+00:00
- duration: 22.64s
- status: ok
- input_tokens: 4
- output_tokens: 1986
- cache_read_tokens: 38926
- cache_write_tokens: 10313
- cost_usd: 0.130635
