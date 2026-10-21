# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-10-20T19:40:28+00:00

## timeline

### 19:40:40 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Policy changes drive hit-rate; mechanics changes (u32 links, hasher, key move, head-touch early return) drive speed and size. The two axes stay largely independent, and mechanics changes preserve eviction order.\n- Segmentation gives scan resistance. This is established; do not re-prove it.\n- Ghost history helps as an insertion-position filter: cold keys go to the LRU end of probation unless the ghost table remembers them. This was the last real hit-rate gain.\n- Ghost overhead can be trimmed without losing the gain: a smaller table and fewer redundant hashes cut put tail latency while hit-rate held.\n- Dropping the hash avalanche fold was a real put-speed win on the segmented policy. It came only after the fold was shown to be the cost.\n- Capping the second-chance eviction walk cut put p99, and the cost was hit-rate. This is a usable latency/hit-rate dial.\n\n## Recurring failures\n\n- Shifting promotion/demotion work between operations (lazy demotion, reference bit instead of relink on get) only moves the cost. The tail lands on put or eviction.\n- Cheap-skip tricks for relinking (head-adjacent skip) fail when few hits land in the skipped case. The added branch costs more than it saves.\n- Layout and bit-packing changes stop giving speed once the policy is segmented. They give size only, and sometimes cost latency.\n- Promoting ghost hits straight to protected hurt hit-rate. Returning evictees are not hot enough to displace protected entries.\n- Constant-tuning (protected share), demotion-position tweaks, put-update semantics, and key-clone or hasher micro-optimizations are saturated or inside noise. Stop spending cycles on them.\n- Winner status is weak evidence. Many small per-op-time deltas appeared on paths the change could not touch, so they are noise, and some \"Winners\" are just noise-tolerant.\n- Bundling several changes (fold with layout, hasher with touch path) makes attribution impossible, and authors cannot read diffs.\n\n## Open questions\n\n- What is the per-op-time noise floor? Repeat-run measurement is still missing, and it has been pending for many cycles.\n- Does the hit-rate gain from segmentation and ghost history hold on non-scan workloads? This is the longest-pending check.\n- Can the number of map operations per put be cut to move the median put, since hash and ghost trims only reached the tail?\n- How far can the ghost table shrink or be sampled before hit-rate degrades?\n- Can the eviction cap be made adaptive, or a cheaper rule found for which referenced nodes to spare, so put p99 stays low without the hit-rate loss?\n- Does a ghost-hit insertion at probation MRU (a middle path) beat both extremes?\n- Do u32 links give a speed benefit on the segmented policy at all, or only size?\n- Heap-allocated ghost memory is invisible to the per-node size metric, so a heap-inclusive memory measure is needed.\n\n## Per-policy notes\n\n- Plain LRU: u32 sentinel links, a lighter hasher and a key move were pure wins in early cycles, with hit-rate unchanged.\n- Segmented LRU (probation/protected): it gives scan-resistance hit-rate at the cost of promotion/demotion relinking on the hot path. Protected-share increases give diminishing hit-rate gains. Demoted nodes are best kept warm, not at the LRU end.\n- Segmented LRU with a second-chance reference bit: a small scan-resistance gain and a cheaper get, with work shifted to eviction. A capped walk trades hit-rate for put p99.\n- Ghost-gated segmented LRU is the current baseline: a halved ghost table, a single-hash eviction path, and a ghost lookup gated on a full cache. Its remaining cost is put-path overhead and off-node memory.\n"
}
```

### 19:40:44 chat

I wrote `cross-cycle-insights.md` to the cycle 0008 cross-cycle output directory, using the four fixed sections in order.

- **Recurring wins:** hit-rate comes from policy changes (segmentation, ghost-gated insertion), and speed and size come from mechanics changes. Dropping the hash fold and trimming ghost overhead also paid off.
- **Recurring failures:**
  - Moving promotion or demotion work between operations only shifts the cost.
  - Constant-tuning and micro-optimizations are saturated or inside noise.
  - Small per-op-time deltas can't be attributed to the changes.
- **Open questions:**
  - What is the per-op-time noise floor?
  - Do the hit-rate gains hold on non-scan workloads?
  - Can the number of map operations per put be cut?
  - How far can the ghost table shrink?
  - Can the eviction cap be made adaptive?
  - Is there a heap-inclusive memory measure that makes ghost cost visible?
- **Per-policy notes:** these cover plain LRU, segmented LRU, segmented LRU with the second-chance bit, and the current ghost-gated baseline.

I kept it qualitative and used no numbers beyond the ledger, as the constraints require.


---

- finished: 2026-10-20T19:40:44+00:00
- duration: 16.33s
- status: ok
- input_tokens: 4
- output_tokens: 1859
- cache_read_tokens: 40440
- cache_write_tokens: 12420
- cost_usd: 0.073137
