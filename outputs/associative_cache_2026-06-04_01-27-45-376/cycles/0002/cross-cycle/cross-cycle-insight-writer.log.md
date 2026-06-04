# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-7)
- workspace: /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_01-27-45-376/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-06-03T22:54:33+00:00

## timeline

### 22:54:37 thinking



### 22:54:56 tool: Write

```json
{
  "file_path": "cross-cycle-insights.md",
  "content": "# Cross-cycle insights\n\n## Recurring wins\n\n- Eviction-side discrimination over promotion-side gating: leaving LRU\n  ordering intact and only biasing victim selection has been the only\n  approach to crack the Pareto front so far.\n- Reusing the existing `lfu_counter` field as a *binary*\n  touched/never-touched signal is effectively free in metadata and\n  composes with the bucket trait's `&[Item]` eviction signature\n  without churn.\n- Sacrificing never-touched slots before genuinely re-used ones is the\n  single idiom that has reliably beaten baseline; it remains the\n  policy to beat.\n\n## Recurring failures\n\n- Withholding or shortening promotion inside the ~16-slot bucket\n  consistently regresses hit-rate — there is no headroom for a hot\n  newcomer to survive its vulnerable first-hit window without a full\n  move-to-back.\n- Replacing exact LRU ordering with a coarser recency proxy (e.g.,\n  CLOCK's two-level bit) loses information the bucket was already\n  preserving cheaply.\n- Pushing `lfu_counter` past the binary touched/untouched distinction\n  — whether as tiered filters or smooth counter-rank eviction —\n  regresses once every slot is touched, because raw lifetime counts\n  start overriding the fresher recency encoded in bucket position.\n\n## Open questions\n\n- Would a cheap aging or decay step (decrement/halve on eviction\n  sweep) make multi-bit counter tiering finally outperform the binary\n  filter by keeping the counter aligned with *recent* hits?\n- Is there a detectable proxy for the \"all-touched\" regime (bucket-\n  level touched-count, absence of any zero slot) that could trigger a\n  fallback to pure LRU exactly when counter-based ranking turns\n  harmful?\n- Would loosening the bucket trait's `&[Item]` eviction signature\n  unlock a side-channel reference bit on the bucket itself, opening\n  per-op-time wins that the current signature blocks?\n- Are there orthogonal axes — insertion placement, bucket-level\n  admission, victim-side recency reconstruction — that could compose\n  with the untouched-first filter without re-introducing the failure\n  modes above?\n- Does a \"must survive two hits\" variant of the filter help in any\n  workload regime, or does it always reintroduce the killed-hot-\n  newcomer failure mode?\n\n## Per-policy notes\n\n- ClassicLRU: the entire cohort history has revolved around this\n  policy. The blanket move-to-back on hit is the primary lever, and\n  the bucket size (~16) is the binding constraint on every promotion-\n  side experiment. Eviction-side tweaks layered on top of intact LRU\n  ordering are the productive direction; promotion-side tweaks and\n  ordering replacements are not.\n- CLOCK (explored as a promotion-path replacement): coarsening the\n  recency signal lost more than it gained inside the narrow bucket;\n  would likely need a trait-signature change to host its reference\n  bit natively before it is worth revisiting.\n- LFU-style counter ranking: only viable as a zero/non-zero gate\n  today; full LFU semantics need decay/aging before they can coexist\n  with LRU ordering without fighting it.\n"
}
```

### 22:55:01 chat

Wrote `cross-cycle-insights.md` projecting the two-cycle ledger into the four fixed sections. Key throughlines captured: eviction-side discrimination with a binary `lfu_counter` gate is the only winning idiom so far; promotion-side gating and counter-rank eviction (beyond binary) consistently regress; the bucket-trait `&[Item]` signature and the "all-touched" regime remain the dominant open constraints.


---

- finished: 2026-06-03T22:55:02+00:00
- duration: 28.40s
- status: ok
- input_tokens: 6
- output_tokens: 1387
- cache_read_tokens: 36666
- cache_write_tokens: 6833
- cost_usd: 0.097835
