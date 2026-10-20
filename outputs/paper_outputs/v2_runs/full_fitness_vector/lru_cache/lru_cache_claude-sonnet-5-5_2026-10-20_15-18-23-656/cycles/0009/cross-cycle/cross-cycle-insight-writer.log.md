# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-20T19:44:53+00:00

## timeline

### 19:45:12 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Segmentation (segmented LRU) is the established source of scan-resistance hit-rate; no need to re-prove it.\n- Layout/mechanics changes (u32 links, lighter hasher, head-touch early return, move-based eviction) gave speed/size on the plain LRU without touching hit-rate. Hit-rate comes from policy, speed/size from mechanics; the axes are independent.\n- Removing the hash avalanche fold on the segmented variant was a real put-speed and size gain, with no hit-rate loss.\n- Ghost history used as a scan filter on insertion position (cold keys go to the next-to-evict slot unless remembered) is the one structural hit-rate gain since segmentation. It cost put latency, especially the tail.\n- Ghost overhead trimming (smaller table, fewer/single hashes, gated lookup) cut the put tail while hit-rate held.\n- Protected-share increases gave small hit-rate gains, but each step gained less.\n- A capped second-chance eviction walk cut get and put latency, including put p99, at a hit-rate cost.\n\n## Recurring failures\n\n- Shifting work between operations (lazy demotion, reference-bit get path) only moves the promotion/demotion relink cost. It does not remove it.\n- Skipping relinks only near the head did not pay off: too few hits land there, so the added branch cost more than it saved.\n- Ghost-hit promotion (to protected, or to probation MRU) hurt hit-rate. The filter works by keeping suspicious keys cheap to evict, not by rewarding remembered ones.\n- Micro-optimizations (key clone removal, node-slot reuse, further hasher tweaks, ghost shrinking beyond one halving) are invisible. Map remove/insert dominates put.\n- Demotion-position tweaks and `put`-update semantics changes were neutral or slightly negative.\n- Bundling several changes made attribution impossible. Authors could not read diffs, so attribution is inferred.\n- \"Winner\" status since about cycle 0005 often reflects noise tolerance, not improvement. Small per-op-time deltas appear even on untouched paths and are unattributable.\n\n## Open questions\n\n- What is the per-op-time noise floor? A repeat-run measure is still missing, and it blocks crediting any latency claim.\n- Does the ghost-gated segmented design hold on non-scan workloads? This check has been pending for many cycles.\n- Can map operations per put be cut structurally (entry-style combined lookup/insert, cheaper map) to move the median put?\n- Is the remaining hit-rate gain in admission policy or cache sizing, since insertion position and segment share look exhausted?\n- Heap-allocated ghost memory is invisible to the per-node size metric. A heap-inclusive memory measure is needed.\n- Can the eviction-walk cap become adaptive, keeping the put p99 gain without the hit-rate loss?\n- Does u32 links give a speed win on a segmented policy? Only size has been shown.\n\n## Per-policy notes\n\n- Plain LRU: u32 links and a lighter hasher were pure wins in cycle 0002. This is saturated.\n- Segmented LRU: the cost is promotion/demotion relinking. Fold-free hash plus reference bit is the baseline. The protected share and demotion position are exhausted. The cap on the second-chance walk is a latency/hit-rate dial.\n- Segmented LRU with ghost-gated insertion (the cycle-0008 baseline, smaller ghost table, single-hash gated eviction path) is the current parent. The ghost table has headroom, but shrinking it further buys nothing measurable. Next levers are structural: fewer map ops per put, or an admission policy.\n"
}
```

### 19:45:15 chat

I wrote `cross-cycle-insights.md` to the cycle 0009 `cross-cycle` output directory. It has the four fixed sections in order, built only from the ledger digest.

The file's main conclusions:
- **Recurring wins:**
  - Segmentation for scan resistance is established.
  - Mechanics changes gave speed and size without touching hit-rate.
  - Removing the hash fold helped.
  - Ghost-gated insertion position is the one structural hit-rate gain since segmentation.
- **Recurring failures:**
  - Moving work between operations only relocates it.
  - Ghost-hit promotion hurts hit-rate.
  - Micro-optimizations are invisible.
  - Many recent "Winners" are just noise tolerance.
- **Open questions:** the per-op-time noise floor, non-scan workloads, fewer map operations per put, admission policy, a heap-inclusive memory measure, and an adaptive eviction-walk cap.
- **Per-policy notes:** the plain LRU is saturated. The segmented LRU's cost is promotion and demotion relinking. The ghost-gated segmented variant from cycle 0008 is the current parent.


---

- finished: 2026-10-20T19:45:16+00:00
- duration: 22.73s
- status: ok
- input_tokens: 4
- output_tokens: 1768
- cache_read_tokens: 41278
- cache_write_tokens: 13197
- cost_usd: 0.075502
