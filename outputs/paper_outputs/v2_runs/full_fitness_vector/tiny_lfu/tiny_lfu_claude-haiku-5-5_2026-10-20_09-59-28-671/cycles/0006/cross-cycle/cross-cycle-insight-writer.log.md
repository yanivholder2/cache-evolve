# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-20T10:28:26+00:00

## timeline

### 10:28:33 thinking



### 10:28:38 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-cycle insights\n\n## Recurring wins\n\n- Constant-factor work that leaves eviction order and counter indices alone is the most reliable source of gains. Hoisting or reusing hashes, inlining accessors, and splitting shared put-side paths each moved per-op time without moving hit rate.\n- Keeping sketch seeds in a module-level constant table, not per instance, removes the struct growth that per-instance caching caused while keeping the speedup.\n- Packing protected-segment state into existing node fields cut metadata without changing hit rate.\n- Restoring public per-row seeded indexing corrects admission fidelity. The cost is per-row work on both get and put, so it is a correctness win that gives back speed.\n\n## Recurring failures\n\n- Recency rotation of rejected admission victims lost on both hit rate and per-op time. The frequency gate already does the protective work.\n- Changes to admission or aging that do not target a real limiting factor did not raise hit rate: a one-hit-wonder doorkeeper barely moved it, shorter frequency aging lowered it, and a small recency window in front of the gate did not move it while adding list work.\n- Capping the protected segment without demotion lowered hit rate, so bounding segment size is the wrong lever.\n- Index layouts that change which keys collide in the sketch move hit rate, even when they speed up the per-op path.\n- Each attempt to speed up the sketch's per-row indexing has traded away either index fidelity or speed.\n\n## Open questions\n\n- Do the const-table seeds and the hoisted-hash helpers produce exactly the same counter indices as the per-access public derivation for every key, or only matching aggregate hit rate?\n- Can the public per-row indices be produced with less per-row work, so fidelity does not cost the speed gain?\n- Can a cache-line block-grouped counter layout keep its per-op gain while preserving the original collision pattern?\n- Does the protected-capacity branch ever fire on the workload? If not, protected-tail demotion has not really been tested.\n- Do the timing shifts (put-side improvements and regressions, get-side gains) survive repeat runs, or are they noise from single runs?\n- Are the metadata figures complete? Doorkeeper storage and window bookkeeping do not appear in the reported breakdown, and struct growth has gone unexplained more than once.\n- Do any of the hit-rate effects, especially the segmentation gain, hold on a second trace? Every result so far comes from one trace.\n\n## Per-policy notes\n\n- **TinyLFU-style admission with a frequency sketch and segmented resident list (all cycles):** the main levers are the sketch (indexing, seeds, layout, aging), the admission gate (doorkeeper, window), and the protected/probation split.\n  - Segmentation is the only source of hit-rate movement seen so far, and it is small. Its benefit appears to depend on the protected set turning over, so caps hurt.\n  - The sketch's indexing is the main place where hit rate and speed trade off directly.\n  - Admission-side additions (doorkeeper, window) and aging changes have not helped on this workload.\n- **Recency handling of rejected victims:** dropped after a single cycle of clear losses. No other recency-ordering change has been tested in this policy family.\n- **Evidence scope:** every cycle uses the same single trace (workload-a), so no finding here is yet a general policy property.\n"
}
```

### 10:28:40 chat

I wrote `cross-cycle-insights.md` in the output directory (`.../cycles/0006/cross-cycle/`) with the four required sections in order. I did not read any previous version of the file.

The main signals from the ledger, cycles 0001–0006:

- **Wins:** constant-factor changes that keep eviction order and counter indices intact have repeatedly improved per-op time without moving hit rate. Storing sketch seeds in a module-level constant table avoids the struct growth that per-instance caching caused.
- **Failures:** recency rotation of rejected victims, protected-segment caps, shorter frequency aging, the doorkeeper, and the recency window did not pay off. Changes that alter sketch collisions move hit rate.
- **Open questions:** whether the const-table and hoisted-hash indices match the public per-access derivation for every key, whether the protected-capacity branch ever fires on this trace, whether the timing shifts survive repeat runs, and whether any effect holds on a second trace. All evidence so far comes from one trace (workload-a).

The per-policy section covers one family, TinyLFU-style admission with a segmented resident list. The ledger gives no evidence about other eviction policies, and the file says so.


---

- finished: 2026-10-20T10:28:41+00:00
- duration: 15.41s
- status: ok
- input_tokens: 4
- output_tokens: 2779
- cache_read_tokens: 38098
- cache_write_tokens: 11023
- cost_usd: 0.003975
