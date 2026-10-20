# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0006\cross-cycle
- allow_edits: True
- started: 2026-07-22T02:24:58+00:00

## timeline

### 02:25:05 thinking



### 02:25:22 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Trimming dead metadata off the eval path is cheap, monotonic, and\n  repeatable: fields carried only by off-path policies peel off with zero\n  hit-rate change and reclaim the 16-byte `Item` floor. Kept paying off\n  cycle after cycle.\n- Keeping state off the measured `Item` (in the unmeasured bucket struct)\n  makes scan-resistance mechanisms \"free\" on the size axis — the size\n  metric weighs only `Item`, so hit-rate and metadata-size stopped being\n  antagonistic once state moved into the bucket.\n- The index-preserving swap consistently collapses the quadratic p99 tails\n  that key-matched reuse-bit rebuilds create, recovering clean tails with\n  hit-rate held bit-for-bit identical. Proven across the cycles that used it.\n- Genuine segment separation (2Q/SLRU-style) is the one lever that broke\n  the front-admission hit-rate ceiling, and it does so at the metadata\n  floor — trading only latency, not bytes.\n\n## Recurring failures\n\n- Frequency/LFU-based victim choice loses to recency-front every time it is\n  tried: least-frequently-reused is a worse victim signal than plain\n  LRU-front on this trace.\n- Weakening permanent protection fails repeatedly — second-chance/CLOCK\n  aging sheds immunity too fast and collapses back toward plain LRU;\n  permanent binary protection fits this trace.\n- Re-timing promotion on a single short bucket (single-step promotion,\n  reuse-bit gating) buys latency at best and regresses hit-rate at worst —\n  small buckets lack the slots for gradual promotion. Segmentation, not\n  timing, is the lever.\n- Admission geometry has a sharp optimum at the front slot; midpoint/wider\n  probation is strictly worse.\n- Overlay-hygiene debt recurs: single-file variants silently forfeit\n  parent-layout wins (the 16-byte `Item`), producing metadata regressions\n  by omission rather than logic.\n\n## Open questions\n\n- Can the segmented incumbent shed its per-access rotate/rebalance latency\n  (e.g. lazy demotion) without the saturated-bucket hit-rate leak, since\n  the rotation quietly maintains intra-protected eviction order?\n- Where does the optimal protected-segment cap sit for these short buckets\n  — near `max_size` with near-total-only demotion — and does that turn the\n  aggressive-cap regression into a gain?\n- Is two-segment separation the structural hit-rate limit for this trace,\n  or is there headroom above the SLRU ceiling?\n- How can overlay inheritance be enforced so prior-cycle layout wins are\n  never dropped by variants that touch only one file?\n\n## Per-policy notes\n\n- LRU / front-admission: the settled base. Move-to-back-on-hit makes the\n  frontmost slot always probationary, so explicit reuse protection and\n  stateless front-admission are redundant re-encodings of one lever, not\n  compounding ones.\n- Reuse-bit / protection policies: permanent binary protection is the fit;\n  keep state off `Item` and move it with index-preserving swaps. Protection\n  adds value only over a policy that does not already positionally\n  segregate reused items.\n- SLRU / segmented: the current incumbent — only path to break the ceiling,\n  delivered at the 16-byte floor. Frontier is now a hit-rate/latency trade;\n  protected cap is real but sharply mis-tuned when aggressive.\n- Frequency/LFU: dropped — worse victim signal than recency on this trace.\n"
}
```

### 02:25:37 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\associative_cache_2026-07-21_23-22-35-670\\cycles\\0006\\cross-cycle\\cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Trimming dead metadata off the eval path is cheap, monotonic, and\n  repeatable: fields carried only by off-path policies peel off with zero\n  hit-rate change and reclaim the 16-byte `Item` floor. Kept paying off\n  cycle after cycle.\n- Keeping state off the measured `Item` (in the unmeasured bucket struct)\n  makes scan-resistance mechanisms \"free\" on the size axis — the size\n  metric weighs only `Item`, so hit-rate and metadata-size stopped being\n  antagonistic once state moved into the bucket.\n- The index-preserving swap consistently collapses the quadratic p99 tails\n  that key-matched reuse-bit rebuilds create, recovering clean tails with\n  hit-rate held bit-for-bit identical. Proven across the cycles that used it.\n- Genuine segment separation (2Q/SLRU-style) is the one lever that broke\n  the front-admission hit-rate ceiling, and it does so at the metadata\n  floor — trading only latency, not bytes.\n\n## Recurring failures\n\n- Frequency/LFU-based victim choice loses to recency-front every time it is\n  tried: least-frequently-reused is a worse victim signal than plain\n  LRU-front on this trace.\n- Weakening permanent protection fails repeatedly — second-chance/CLOCK\n  aging sheds immunity too fast and collapses back toward plain LRU;\n  permanent binary protection fits this trace.\n- Re-timing promotion on a single short bucket (single-step promotion,\n  reuse-bit gating) buys latency at best and regresses hit-rate at worst —\n  small buckets lack the slots for gradual promotion. Segmentation, not\n  timing, is the lever.\n- Admission geometry has a sharp optimum at the front slot; midpoint/wider\n  probation is strictly worse.\n- Overlay-hygiene debt recurs: single-file variants silently forfeit\n  parent-layout wins (the 16-byte `Item`), producing metadata regressions\n  by omission rather than logic.\n\n## Open questions\n\n- Can the segmented incumbent shed its per-access rotate/rebalance latency\n  (e.g. lazy demotion) without the saturated-bucket hit-rate leak, since\n  the rotation quietly maintains intra-protected eviction order?\n- Where does the optimal protected-segment cap sit for these short buckets\n  — near `max_size` with near-total-only demotion — and does that turn the\n  aggressive-cap regression into a gain?\n- Is two-segment separation the structural hit-rate limit for this trace,\n  or is there headroom above the SLRU ceiling?\n- How can overlay inheritance be enforced so prior-cycle layout wins are\n  never dropped by variants that touch only one file?\n\n## Per-policy notes\n\n- LRU / front-admission: the settled base. Move-to-back-on-hit makes the\n  frontmost slot always probationary, so explicit reuse protection and\n  stateless front-admission are redundant re-encodings of one lever, not\n  compounding ones.\n- Reuse-bit / protection policies: permanent binary protection is the fit;\n  keep state off `Item` and move it with index-preserving swaps. Protection\n  adds value only over a policy that does not already positionally\n  segregate reused items.\n- SLRU / segmented: the current incumbent — only path to break the ceiling,\n  delivered at the 16-byte floor. Frontier is now a hit-rate/latency trade;\n  protected cap is real but sharply mis-tuned when aggressive.\n- Frequency/LFU: dropped — worse victim signal than recency on this trace.\n"
}
```

### 02:25:42 chat

Wrote `cross-cycle-insights.md` as a fresh projection of the 6-cycle ledger, using the four fixed sections:

- **Recurring wins** — dead-metadata trimming (cheap/monotonic), keeping state off the measured `Item`, the index-preserving swap for clean tails, and SLRU segmentation as the one lever that broke the hit-rate ceiling.
- **Recurring failures** — LFU/frequency victim choice, weakening permanent protection, single-list promotion re-timing, non-front admission geometry, and the recurring overlay-hygiene debt.
- **Open questions** — shedding segment-rotation latency without the saturated-bucket leak, optimal protected cap, whether two-segment is the structural limit, and enforcing overlay inheritance.
- **Per-policy notes** — grouped by LRU/front-admission, reuse-bit/protection, SLRU/segmented, and frequency/LFU.


---

- finished: 2026-07-22T02:25:42+00:00
- duration: 43.82s
- status: ok
- input_tokens: 5
- output_tokens: 3267
- cache_read_tokens: 73154
- cache_write_tokens: 12385
- cost_usd: 0.200565
