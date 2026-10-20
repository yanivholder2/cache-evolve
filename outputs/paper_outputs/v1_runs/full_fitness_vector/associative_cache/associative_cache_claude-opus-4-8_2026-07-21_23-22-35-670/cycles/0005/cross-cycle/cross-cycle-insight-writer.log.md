# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0005\cross-cycle
- allow_edits: True
- started: 2026-07-21T21:29:07+00:00

## timeline

### 21:29:14 thinking



### 21:29:15 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 21:29:15 tool: Glob

```json
{
  "pattern": "**/cycle-000*/**"
}
```

### 21:29:21 thinking



### 21:29:21 tool: Bash

```json
{
  "command": "pwd && ls",
  "description": "Show working directory and contents"
}
```

### 21:29:25 thinking



### 21:29:42 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\associative_cache_2026-07-21_23-22-35-670\\cycles\\0005\\cross-cycle\\cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Trimming dead metadata off the eval path is cheap and monotonic: fields\n  carried only by off-path policies peel off repeatedly with zero hit-rate\n  cost, driving `Item` down to a stable 16-byte floor.\n- Keeping state off the measured `Item` (in the unmeasured bucket struct)\n  makes scan-resistance essentially free on the size axis — hit-rate and\n  metadata stopped being antagonistic once state moved off-`Item`.\n- Stateless probationary-front admission lifts hit-rate at the floor with\n  no per-item state and clean tails; the front slot is the settled optimum\n  on the admission-geometry axis.\n- The index-preserving swap reliably collapses the quadratic p99 tails that\n  key-matched rebuilds inflate, with hit-rate held bit-for-bit — the proven\n  fix for any stateful reuse-vector path.\n- Genuine segment separation (2Q/SLRU-style protected segment) is the one\n  lever that broke the long-standing hit-rate ceiling, and it does so while\n  holding the metadata floor.\n\n## Recurring failures\n\n- Frequency/LFU-based victim choice loses to recency-front every time it is\n  tried; permanent binary reuse-protection fits this trace better.\n- Weakening protection — second-chance/CLOCK aging, reuse-bit promotion\n  gates — sheds immunity too fast or is redundant with position, collapsing\n  back toward plain LRU.\n- Re-timing promotion on a single short bucket (single-step, gradual) does\n  not buy hit-rate: hot keys are evicted before they ratchet out, and small\n  buckets lack the slots gradual promotion needs.\n- Widening the probation window (midpoint insertion) is strictly worse than\n  front admission — never-reused scan items linger above the eviction slot.\n- Stacking explicit reuse protection onto front-admission adds nothing: the\n  two are the same lever under move-to-back-on-hit.\n\n## Open questions\n\n- Can the SLRU/segmented hit-rate win shed its per-access rotate/rebalance\n  latency (lazy demotion, cheaper boundary bookkeeping)?\n- What protected-segment cap is optimal for short buckets, and does the\n  gain survive at the smallest bucket sizes?\n- How much headroom remains above the two-segment ceiling, or is\n  segmentation near the structural limit for this trace?\n- The recurring upward p99/tail drift under flat p50 on metadata cuts —\n  structural or persistent measurement noise?\n\n## Per-policy notes\n\n- LRU / front-admission: front slot is the admission optimum; move-to-back-\n  on-hit makes the frontmost slot always probationary, so added reuse bits\n  are redundant with position.\n- Reuse-bit / protected (stateful): permanent binary protection is the\n  right fit; keep state off `Item` and relocate it via index-preserving\n  swaps to avoid quadratic tails.\n- SLRU / 2Q segmentation: the only ceiling-breaking policy; trades latency\n  (dual-segment rotate/rebalance) rather than metadata for hit-rate.\n- LFU / frequency: consistently a worse victim signal than recency on\n  saturated buckets — drop.\n- Overlay hygiene (all policies): single-file variants repeatedly forfeit\n  parent wins (e.g. the 16-byte `Item`) by omission; enforce full\n  parent-layout inheritance.\n"
}
```

### 21:29:47 chat

Wrote `cross-cycle-insights.md` as a fresh projection of the five-cycle ledger, organized into the four fixed sections:

- **Recurring wins** — metadata trimming to the 16-byte floor, keeping state off `Item`, stateless front-admission, the index-preserving swap for tails, and SLRU segmentation as the ceiling-breaker.
- **Recurring failures** — LFU victim choice, protection-weakening (second-chance/gating), single-list promotion re-timing, midpoint probation, and redundant protection stacking.
- **Open questions** — reclaiming SLRU latency, optimal protected-segment cap, remaining headroom above the two-segment ceiling, and the persistent tail-drift signal.
- **Per-policy notes** — grouped observations for LRU/front, reuse-bit/protected, SLRU/2Q, LFU, plus the recurring overlay-hygiene hazard.

Signals are combined across cycles rather than restated per-cycle, with no invented numbers.


---

- finished: 2026-07-21T21:29:48+00:00
- duration: 40.68s
- status: ok
- input_tokens: 7
- output_tokens: 2168
- cache_read_tokens: 96401
- cache_write_tokens: 9619
- cost_usd: 0.166595
