# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-06-05T20:07:04+00:00

## timeline

### 20:07:10 thinking



### 20:07:10 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 20:07:10 tool: Bash

```json
{
  "command": "pwd && ls",
  "description": "Show working directory and contents"
}
```

### 20:07:27 thinking



### 20:07:33 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Reallocating fixed budget toward the promotion-protected normal tier was the\n  earliest and largest win (cycles 0001–0002); shrinking the over-provisioned\n  stash repeatedly lifted hit-rate until the split saturated around the\n  normal-dominant ratio.\n- The stash's load-bearing value is *dwell time* — giving new/cold keys room to\n  prove a second reference. Levers that extend dwell (relocation hop, bounded\n  multi-hop cascade) yield clean positives where promotion-side levers do not.\n- Demotion/eviction-path insert position is a genuinely independent lever from\n  the hit-path position: landing demoted or probing-evicted keys at MRU\n  protected the oscillating working set and gained after hit-path tuning flattened.\n- Two-speed promotion (slow stash→target, fast target→probing) reserves the\n  protected tier for twice-seen keys and gains where uniform promotion stalled.\n- The strongest late principle: *which population a lever spares matters more\n  than its shape*. Orthogonal, warm-population seams stack on the winner —\n  a ghost gate for dropped-then-recurring keys (LIRS short-reuse) and a\n  breadth-of-homes hop on the warm probing tier both netted positive, while the\n  same breadth idea was inert on cold stash overflow.\n\n## Recurring failures\n\n- Removing or gutting stash admission regresses sharply (cycles 0004, 0005) —\n  the admission filter is protective, not dead probation; front-landing cold\n  keys makes them instant eviction victims and is closed in both directions.\n- Promotion-gate widening is occupancy-bottlenecked and now exhausted across\n  every signal shape — quarter-gate, half-gate, density-independent touch\n  counter, and len-2 loosening all returned slivers or slight negatives. The\n  ceiling is the trace's small (len-1/len-2) target buckets, not the rule.\n- Stacking independently-discovered seams overlaps rather than compounds when\n  they chase the same boundary-key pool already reclaimed by promotion (single\n  hop onto the winner, power-of-two admission, deeper/wider cascade on the\n  winner all washed out or inverted).\n- Geometry trades under conserved budget are neutral-to-negative: widening\n  buckets while halving count regressed (collisions), and the symmetric\n  width-for-count trade was exactly flat — addressability and depth are\n  interchangeable at fixed capacity.\n- Changing how the protected tier *ranks* residents (LFU eviction) regressed —\n  recency/promote-on-hit already fits the skew; LFU retained stale-frequent keys.\n\n## Open questions\n\n- Do the two latest orthogonal positives (ghost gate, probing-tier breadth)\n  stack with each other, or overlap once both ride the winner?\n- Can the \"spare warm victims, not cold ones\" principle revive other inert\n  stash ideas when re-pointed at the warm probing/protected tiers?\n- Is non-additivity and the occupancy ceiling a property of this single\n  high-skew trace, or general? A second, less-skewed trace has been overdue for\n  validation since cycle 0004 and is now the highest-value next step.\n\n## Per-policy notes\n\n- Recency / promote-on-hit on the protected (normal) tier is the consistent\n  best fit for this skewed trace; promotion-position and two-speed promotion\n  refine it marginally.\n- LFU eviction on the protected tier is confirmed dead here.\n- Stash (cold admission): recency-with-dwell is essential; MRU default for\n  cold keys holds, front-insertion regresses. Relocation/cascade helps on a\n  clean baseline but inverts once the protected tier is present (it evicts\n  already-warmed residents).\n- Ghost-list assist (LIRS-style) on dropped-then-recurring keys is the one\n  population promotion never recaptures and the freshest orthogonal win.\n"
}
```

### 20:07:42 chat

Wrote `cross-cycle-insights.md` — a fresh projection of all nine ledger cycles into the four fixed sections.

Key synthesis across the run:
- **Wins** that held up: budget shift to the protected tier (early/largest), stash dwell-time levers, MRU demotion-path landing, two-speed promotion, and the late "spare warm populations, not cold ones" principle that finally produced orthogonal stacking gains (ghost gate, probing-tier breadth).
- **Failures** that kept recurring: gutting stash admission, promotion-gate widening (occupancy-capped regardless of signal shape), non-additive seam stacking, neutral geometry trades, and LFU on the protected tier.
- **Open threads**: whether the two newest orthogonal seams stack, reapplying the warm-victim principle, and the long-overdue second-trace validation.
- **Per-policy notes** grouped by protected-tier, stash, and ghost-list behavior.


---

- finished: 2026-06-05T20:07:43+00:00
- duration: 38.60s
- status: ok
- input_tokens: 5
- output_tokens: 2189
- cache_read_tokens: 67991
- cache_write_tokens: 11988
- cost_usd: 0.104784
