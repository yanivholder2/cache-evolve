# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-20T19:36:09+00:00

## timeline

### 19:36:20 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Policy changes drive hit-rate; mechanics changes (u32 links, hasher, touch fast path, key move) drive speed and size. The axes are largely independent, and mechanics changes on the plain LRU were pure wins (cycle 2).\n- Segmentation (scan resistance) is established as a hit-rate win on the scan-heavy trace. It needs no more proof.\n- Removing the hash avalanche fold (single multiply plus rotate) was a real put-speed and size win on the segmented policy.\n- Insertion-position filters help scan resistance: ghost-gated LRU-end insertion for cold keys raised hit-rate. A second-chance reference bit gave a small gain with a cheaper get.\n- Protected-share increases gave small hit-rate gains, with diminishing returns.\n\n## Recurring failures\n\n- Shifting promotion/demotion work between operations (lazy demotion, reference-bit eviction walks) only moves the cost and usually worsens put tails. The relink work itself is the cost.\n- Narrow skips such as the head-adjacent relink skip add branch cost because few hits qualify.\n- Packing or narrowing layout on the segmented policy saves size but not speed. Bundling the hash fold with layout changes obscured attribution.\n- Ghost-hit promotion into protected hurt hit-rate. Returning evictees are not hot enough.\n- Key-clone and constant-only micro-optimizations are invisible or within noise. Many recent \"Winner\" labels reflect noise tolerance, not improvement.\n- Ghost tables cost put (extra hash, random access, side memory), and the per-node size metric hides that memory.\n- Variant authors cannot read diffs, so attribution is inferred.\n\n## Open questions\n\n- What is the per-op-time noise floor? Repeat-run measurement is still pending after many cycles.\n- Does any hit-rate gain hold on non-scan workloads? This has been pending since cycle 1.\n- Can the ghost cost be cut (reuse the map hash, smaller or sampled table, cheaper fingerprint) without losing the hit-rate gain?\n- Does a middle path work: ghost hits at probation MRU, with no protected admission?\n- Can the capped eviction walk be tuned or made adaptive to hold put p99 without losing hit-rate?\n- Can the segment be stored implicitly (list membership) to drop masking costs?\n- Is hit-rate left in segment-boundary policy, or does the remaining gain lie in admission and sizing?\n\n## Per-policy notes\n\n- Plain LRU: u32 links and the lighter hasher gave speed and size wins. This is the cleanest mechanics gain.\n- Segmented LRU: hit-rate is better on scans, but promotion/demotion relinking costs time, especially on put tails. Protected share is a weak lever. Demoted nodes are better kept warm than put at the eviction end.\n- Segmented plus reference bit and capped walk: the cap trades put p99 directly against hit-rate.\n- Segmented plus ghost history (current best baseline: ghost-gated LRU-end insertion): it filters scans through insertion position, with a hit-rate gain and a put and memory cost.\n"
}
```

### 19:36:24 chat

I wrote `cross-cycle-insights.md` to the output dir. It has the four required sections in order and is built only from the seven cycles in the ledger digest.

- **Recurring wins:** Policy changes drive hit-rate, and mechanics changes drive speed and size. Segmentation is established as a scan-workload win. Dropping the hash fold helped put speed and size on the segmented policy. Insertion-position filters, like the ghost-gated LRU-end insertion, raised hit-rate.
- **Recurring failures:** Moving promotion/demotion work between operations only shifts the cost, and head-adjacent relink skips don't pay off. Ghost-to-protected promotion hurt hit-rate, and ghost tables add put and memory cost that the per-node size metric hides. Key-clone and constant-only tweaks are lost in noise, so some recent "Winner" labels likely reflect noise tolerance rather than real gains.
- **Open questions:** These are the per-op-time noise floor, whether the hit-rate gains hold on non-scan workloads, and how to cut the ghost cost. The rest are a middle-path ghost design, an adaptive eviction cap, implicit segment storage, and any hit-rate left in segment-boundary policy.
- **Per-policy notes:** Grouped by plain LRU, segmented LRU, segmented plus reference bit and capped walk, and segmented plus ghost history. The last is the current best baseline.

I kept everything qualitative, with no invented numbers.


---

- finished: 2026-10-20T19:36:24+00:00
- duration: 15.82s
- status: ok
- input_tokens: 4
- output_tokens: 1695
- cache_read_tokens: 38938
- cache_write_tokens: 11939
- cost_usd: 0.069334
