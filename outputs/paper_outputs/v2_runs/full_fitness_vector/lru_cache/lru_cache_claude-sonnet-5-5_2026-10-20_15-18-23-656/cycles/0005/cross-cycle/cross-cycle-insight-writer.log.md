# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-20T15:48:43+00:00

## timeline

### 15:48:54 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Mechanics changes (u32 links, lighter hasher, head-touch early return, move-based eviction instead of key clone) preserve eviction order and hit-rate. Speed and size gains come from mechanics. Hit-rate comes only from policy. The two axes stay independent.\n- Segmentation gives scan resistance on the scan-heavy trace. This is established and needs no re-proving.\n- Removing work from the hot path (the hash fold, the key clone) paid off. Shuffling work between operations did not.\n- Combining independent wins into one baseline works, but later combinations compose only weakly, with gains near noise.\n\n## Recurring failures\n\n- Moving promotion/demotion relink work between `get`, `put` and eviction (lazy demotion, reference bit) only shifts the cost. One operation or tail gets worse as another improves.\n- Cheap-skip heuristics (head-adjacent relink skip) failed. Few hits land in the skipped zone, so the extra branch costs more than it saves.\n- Link-width and bit-packing changes on the segmented policy gave size savings but no speed. Widening, masking and NIL checks cost latency.\n- Bundling several changes (hash fold with layout, hasher with touch path) blurs attribution, because authors cannot read diffs.\n- Constant tuning and small hasher tweaks fall within the per-op-time noise floor and should not be credited as wins.\n- Infrastructure failures (build rate limit) can wrongly reject an idea. Check a failure's cause before concluding the idea is bad.\n\n## Open questions\n\n- What is the per-op-time noise floor? Which recent speed claims fall below it?\n- Can the eviction cap be made adaptive, or the choice of spared nodes cheaper, so put p99 stays low and hit-rate recovers?\n- Would a smaller protected share improve hit-rate beyond the current split?\n- Can the segment be stored implicitly, by list membership, so no masking is needed?\n- Does the hit-rate behavior hold on workloads other than the scan-heavy trace? This has been pending for many cycles.\n- Does the struct-size shrink seen when the fold was removed hold, and what caused it?\n\n## Per-policy notes\n\n- Plain LRU: u32 links and the fast hasher were pure wins in cycle 0002. This is the clearest case where mechanics gave both speed and size.\n- Segmented LRU: hit-rate gain on scans, paid for by promotion/demotion relinking on the hot path and larger container metadata. A fold-free single-multiply-plus-rotate hash is enough. Protected share is a weak lever, and a larger share slightly hurt hit-rate.\n- Second-chance reference bit (CLOCK-style): get is slightly cheaper and scan resistance rises a little. The cost moves to the eviction walk and the put tail. Capping the walk cuts latency but lowers hit-rate, so the cap is a direct latency/hit-rate dial.\n- Current baseline: segmented, u32 links, fold-free hasher, reference bit, move-based eviction.\n"
}
```

### 15:49:01 chat

I wrote `cross-cycle-insights.md` to the output dir, with the four fixed sections in order. I built it only from the five cycle digests in the prompt.

- **Recurring wins:** Mechanics changes keep hit-rate and eviction order intact. Segmentation's scan resistance is established. Removing hot-path work pays off, and combining independent wins works, though later combinations add little.
- **Recurring failures:** Moving relink work between `get`, `put` and eviction only shifts the cost. Cheap-skip heuristics lost more than they saved. Link-width and bit-packing changes on the segmented policy gave size but not speed. Bundled changes blur attribution, and tiny speed deltas fall within noise.
- **Open questions:**
  - What the per-op-time noise floor is.
  - Whether the eviction cap can be made adaptive.
  - Whether a smaller protected share helps.
  - Whether the segment can be stored implicitly.
  - Whether results hold off the scan-heavy trace, which has been pending for many cycles.
  - Whether the struct-size shrink holds.
- **Per-policy notes:** Separate entries cover plain LRU, segmented LRU and the reference-bit variant. A last entry names the current baseline: segmented, u32 links, fold-free hasher, reference bit, move-based eviction.

I kept the claims qualitative and included no numbers.


---

- finished: 2026-10-20T15:49:02+00:00
- duration: 18.51s
- status: ok
- input_tokens: 4
- output_tokens: 1616
- cache_read_tokens: 37938
- cache_write_tokens: 9637
- cost_usd: 0.059073
