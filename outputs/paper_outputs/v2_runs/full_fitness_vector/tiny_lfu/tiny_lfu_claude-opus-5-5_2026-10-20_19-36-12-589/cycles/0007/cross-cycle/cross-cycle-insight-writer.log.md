# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-20T20:27:29+00:00

## timeline

### 20:27:54 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Changes to layout and metadata that leave behaviour unchanged are the most dependable gains. Examples: compact `u32` links (c2), `u16` counters at twice the width for the same bytes (c4), packing the region flag into a link bit (c5) and CLOCK ref bits in place of links (c6). Each one cut per-entry or sketch bytes without lowering hit rate.\n- Making the sketch more accurate is the only lever that has raised hit rate. Conservative update (c3), wider `u16` counters (c4) and counting each access once (c5) each added a small gain. The gains shrink each cycle, and this lever now looks used up.\n- Latency gains with a real cause come from removing per-op hashing or sketch work. Examples: dropping the per-row SipHash (c1) and fusing the conservative-update increment into one register-held pass (c4, confirmed again in c6). Cheap hashing and the fused increment are now the standard building blocks.\n- The strict single-victim `>` duel at the door has held up against every alternative tried. Any change in behaviour should be judged against it.\n- Refactors on the CLOCK base that preserve behaviour keep hit rate identical bit for bit, so it is safe to build on.\n\n## Recurring failures\n\n- Sketch aging or decay lost hit rate both times it was evaluated (c2 with 4-bit counters, c4 with full-width counters and conservative update; c1's attempt never ran because of an infra failure). Workload-a rewards stable long-term frequency, and halving wipes out the margin that keeps scans out.\n- Admission windows lowered hit rate at all three sizes tried: the c1 window+SLRU, about 1% in c5 and 10% in c6. Capacity given to unfiltered arrivals goes to one-hit keys.\n- Using more candidates to pick the victim hurt twice: LRU-tail sampling (c2) and min-of-two CLOCK candidates (c7). Candidate estimates are nearly equal, so taking the minimum loosens admission, and new admits end up evicting each other.\n- Changing the tie rule (`>=`, c3) and spreading sketch counters across cache-line blocks (c3) both hurt hit rate. Correlated collisions inflate estimates, and the sketch already fits in cache.\n- Adding structure or work to the hot path costs latency straight away: SLRU relinking, region tags, re-hashing on overflow, extra struct fields that push the cache past two cache lines.\n- Optimising rare paths does nothing in steady state. That covers CLOCK sweep word-scanning and the fill-phase map probe (c7). A custom hash table that stores only indices also lost to SwissTable (c6).\n- Per-op benchmark readings, p99 most of all, drift between cycles. Variants with no cause in the code have often looked faster or slower. A latency change only counts if a mechanism in the code explains it and it repeats when benchmarked side by side against the parent.\n\n## Open questions\n\n- Does the CLOCK base's steady-state `put` cost come from map work on a full miss (a get probe, then remove and insert, each re-hashing)? Would reusing the precomputed hash through a raw-entry API cut it?\n- Is workload-a's hit rate at its ceiling for this TinyLFU-family design? Recency structure, sketch accuracy, duel inputs and victim choice all look used up.\n- Would an index storing hash tags or home slots beat std `HashMap`, or is the map simply not a bottleneck?\n- The unkeyed fast hasher was never checked against adversarial or collision-prone keys.\n- Before targeting any path, how often does steady state actually run it?\n\n## Per-policy notes\n\n- **LRU + TinyLFU duel (c1–c5 base):** sampling victims from the LRU tail gave nothing. Exact recency order turned out to carry no information the duel uses. SLRU and windows on top of it cost latency and did not raise hit rate.\n- **CLOCK + TinyLFU (c6 onward, current base):** hit rate matches LRU and per-entry metadata is smaller. Sweeps are rare because newcomers enter with a clear bit, so sweep-tail tuning is low-yield. Don't change which victim the hand picks. Keep the fused increment and keep struct fields to a minimum.\n- **W-TinyLFU (window + main):** a net loss on workload-a at about 1% and at 10%, even with packed region bits. Treat it as closed unless the workload changes.\n- **CountMin sketch:** keep conservative update, saturating `u16` counters, rows that stay independent, one count per access and a fused single-pass increment. Don't use aging, blocked layouts or further micro-optimisation.\n"
}
```

### 20:28:02 chat

I've written `cross-cycle-insights.md` from the cycle 1–7 ledger, using the four required sections in order and no numbers.

- **Recurring wins:** Changes to memory layout and metadata that leave behaviour unchanged have been the most reliable gains: compact links, `u16` counters and CLOCK reference bits. Hit rate only rose through small improvements in sketch accuracy, and that is close to used up. Latency gains with a real cause came from cheaper hashing and the fused sketch update. The strict duel between the newcomer and a single victim beat every alternative tried.
- **Recurring failures:**
  - Sketch aging lost hit rate both times it was evaluated.
  - Admission windows lost hit rate at every size tried.
  - Choosing the victim from several candidates hurt twice.
  - Changing the tie rule and spreading sketch counters across cache-line blocks both hurt.
  - Optimising paths that steady state rarely runs gained nothing.
  - Latency readings drift between runs, so a gain only counts if the code explains it and it repeats side by side against the parent.
- **Open questions:** The main threads are:
  - Does the CLOCK base's `put` cost come from the map work on a miss? If so, would reusing the already-computed hash across those calls help?
  - Is workload-a's hit rate already at its ceiling for this design family?
  - Would a custom map that stores hash tags beat the standard one?
  - The fast hasher has never been tested against adversarial keys.
- **Per-policy notes:** I grouped the notes under LRU with the TinyLFU duel, CLOCK (the current base), windowed TinyLFU (treated as closed for this workload) and the CountMin sketch settings to keep.


---

- finished: 2026-10-20T20:28:02+00:00
- duration: 33.74s
- status: ok
- input_tokens: 4
- output_tokens: 2279
- cache_read_tokens: 39300
- cache_write_tokens: 12336
- cost_usd: 0.152926
