# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-06-10T21:30:47+00:00

## timeline

### 21:30:51 thinking



### 21:30:52 tool: Bash

```json
{
  "command": "pwd && ls",
  "description": "Show working directory and contents"
}
```

### 21:30:55 thinking



### 21:31:13 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **Segmentation with proof-of-reuse beats single-scalar tuning.** The\n  durable line of progress was: frequency-first victim selection →\n  explicit segmented-LRU (promote-on-second-hit) → FIFO probation order.\n  Demanding evidence of reuse before granting protection, then giving\n  rising keys a full probation lap to earn it, repeatedly lifted hit-rate.\n- **Promote straight to protected-MRU, single hit, then keep forever.**\n  One probationary hit is sufficient proof on this trace; promoted keys\n  belong at the MRU end and benefit from \"once-hot immortality.\" Every\n  scheme respecting this held its gains.\n- **Geometry is a real, independent lever.** After behavioural saturation,\n  a single capacity-conserving bucket-count halving (wider associativity)\n  produced the only fresh hit-rate gain in many cycles.\n- **Behaviour-preserving cost rewrites are always safe to bank.** O(1)\n  eviction scans, reorder-free tiebreaks, rotate-based get/put, and smaller\n  metadata are provably byte-identical and never regress.\n\n## Recurring failures\n\n- **Protection erosion always loses.** Counter aging, second-hit promotion\n  thresholds, protected-segment caps with demotion, CLOCK recency\n  approximation, LFU-min fallback, ghost-ring fast re-entry, and elite\n  sub-tiers all regressed — each made earned protection harder to get or\n  easier to lose on a collision-bound trace.\n- **The two original levers do not compose.** Front/probationary admission\n  stacked onto frequency-first eviction makes a fresh key both oldest and\n  min-frequency, so it is evicted immediately.\n- **Geometry overshoots past one halving.** A second halving (too-wide\n  buckets) and absolute associativity targets (degenerate fixed width) both\n  regressed; the sweet spot is bracketed at exactly one halving.\n- **Looser/larger or re-tiered protected sizing is net-negative** in every\n  form tested, including non-eroding reserved-slot variants.\n- **Routing quality offers no headroom** — SipHash is already uniform on\n  this trace, so stronger finalizers only reshuffle collisions.\n\n## Open questions\n\n- Is hit-rate now fully at its ceiling? Admission, promotion, eviction\n  signal, re-entry, protected sizing/tiering, routing, and bucket width\n  have all been exhausted.\n- Should a per-op-time / metadata-size fitness dimension be added? Cost-side\n  wins keep recurring and keep landing as undifferentiated ties because the\n  hit-rate-only vector cannot see or reward them — deferred for four+ cycles.\n- Do the independent, stackable cost rewrites change anything observable\n  once all adopted, or do they remain collectively invisible to fitness?\n- Is there any untested trace-structure axis left, or should the loop be\n  declared solved and stopped?\n\n## Per-policy notes\n\n- **Segmented-LRU (the established baseline):** the strongest scheme found.\n  FIFO (oldest-first) probation + single-hit promote-to-MRU + immortal\n  protected segment is settled and load-bearing. Do not tighten, cap,\n  re-tier, or re-seat below MRU.\n- **Pure LRU:** the starting point; superseded once frequency/segmentation\n  shielded the working set from one-hit-wonders and scan traffic.\n- **LFU / frequency scalars:** useful only as the bridge to segmentation;\n  as a standalone victim signal or fallback it lets `==1` newcomers dominate\n  and kills freshly-promoted rising keys.\n- **CLOCK / one-bit recency:** coarse reprieve underperforms full positional\n  ranking in small buckets — recency cannot be safely approximated away.\n- **Geometry (orthogonal to policy):** one halving optimal; bracketed above\n  and below. Wider buckets raise per-eviction scan cost, invisible to\n  current fitness.\n"
}
```

### 21:31:19 chat

Wrote `cross-cycle-insights.md` — a fresh projection of the 9-cycle ledger digest into the four fixed sections.

Key synthesis across cycles:
- **Recurring wins:** segmentation + proof-of-reuse, single-hit promote-to-MRU immortality, the one-halving geometry lever, and always-safe cost rewrites.
- **Recurring failures:** every protection-erosion variant, lever non-composition, geometry overshoot, looser/re-tiered protected sizing, and routing tweaks.
- **Open questions:** the ceiling, the long-deferred per-op-time fitness axis, stackable-cost-rewrite visibility, and whether to stop the loop.
- **Per-policy notes:** grouped observations for segmented-LRU, pure LRU, LFU/frequency, CLOCK, and geometry.


---

- finished: 2026-06-10T21:31:19+00:00
- duration: 32.81s
- status: ok
- input_tokens: 5
- output_tokens: 2057
- cache_read_tokens: 68100
- cache_write_tokens: 11968
- cost_usd: 0.102755
