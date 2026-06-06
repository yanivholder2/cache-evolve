# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-06-05T20:24:35+00:00

## timeline

### 20:24:42 thinking



### 20:24:42 tool: Bash

```json
{
  "command": "pwd && ls",
  "description": "Show working directory and contents"
}
```

### 20:24:47 thinking



### 20:25:07 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Protecting the warm working set is the dominant winning family: every\n  early gain came from giving the promotion-protected normal tier more room\n  or shielding it from transient keys. Reallocating budget into that tier\n  carried cycles 0001–0002 and remains the best-so-far baseline.\n- Dwell time is what makes the stash load-bearing. Levers that let a cold\n  key survive long enough to catch a second reference paid off (relocation\n  hop, bounded multi-hop cascade on a clean baseline); the value is giving\n  new keys time to prove themselves, not balanced occupancy.\n- Two-speed promotion is a genuinely distinct positive: landing the\n  stash→target hop slow while keeping target→probing fast reserves the\n  protected tier for twice-seen keys and blocks one-hit spikes.\n- Demotion-path insert position is independent of the hit-path position:\n  re-landing probing-evicted keys at MRU protects the oscillating working set\n  where hit-path tweaks had flattened.\n- Orthogonality beats depth for stacking. The only seam that stacked on the\n  winner was the ghost gate for dropped-then-recurring keys — a population\n  promotion never recaptures. Reapplying an inert idea to a *hotter* victim\n  pool (breadth-of-homes on the probing tier vs. cold stash) also worked.\n\n## Recurring failures\n\n- Removing or withholding stash admission regresses repeatedly: gutting the\n  admission step (cycle 0004) and TinyLFU-style doorkeeper filtering (0010)\n  both forfeit hits because the contended tail recurs too fast to filter.\n- Promotion-gate widening is exhausted in every shape — rank-based quarter\n  and half gates, density-independent touch counters, len-2 loosening — all\n  blocked by the same structural small-bucket occupancy ceiling, not by rule\n  shape.\n- Front-landing cold keys (stash insert-position differentiation) regresses,\n  shown twice (0004, 0005): it makes new keys the immediate eviction victim\n  and denies the dwell time the stash exists to provide.\n- Bucket geometry trades are dead: widening buckets while halving count\n  regressed (0003); the reverse width-for-count trade was exactly neutral\n  (0009). Under conserved budget, addressability and depth are interchangeable.\n- Independently discovered seams overlap rather than compound. Stacking the\n  single hop, deepening the cascade, victim quality, breadth, and combining\n  the two 0009 winners all washed out — they compete for the same thin pool\n  of boundary keys the protected tier already reclaims.\n\n## Open questions\n\n- Has single-workload search fully saturated? Nearly every probe for the last\n  several cycles resolves to overlap or emptiness because the served\n  populations are thin on this one trace.\n- Are the recurring non-additivity and occupancy ceilings properties of this\n  single high-skew trace, or general truths? Only a second, less-skewed trace\n  can disambiguate — this validation has been deferred since cycle 0003 and is\n  now the highest-value next step.\n- Are there other inert stash ideas worth re-pointing at the warm\n  probing/protected tiers under the \"spare warm victims, not cold ones\"\n  principle?\n\n## Per-policy notes\n\n- DASH (the only policy exercised): the protected normal tier wants\n  recency/promote-on-hit, not LFU — forcing LFU eviction regressed (0002) by\n  retaining stale-but-frequent keys over the live set.\n- Bucket count is a value to preserve, not a free associativity knob; hot-key\n  collisions are governed by total capacity, not its partition.\n- The stash behaves as a dwell-time buffer, not probation; its admission\n  filter is structurally load-bearing and must not be removed or delayed.\n- Promotion-side mechanics (gate width, promotion position, touch signals)\n  are occupancy-saturated on this trace; eviction/demotion dynamics and\n  orthogonal warm-population seams have been the more productive frontier.\n"
}
```

### 20:25:13 chat

Wrote `cross-cycle-insights.md` as a fresh projection of the 10-cycle ledger, using the four fixed sections. Key cross-cycle threads captured:

- **Recurring wins**: protected-tier capacity reallocation (the best-so-far baseline), stash dwell-time, two-speed promotion, demotion-MRU, and the orthogonal ghost gate — the one seam that actually stacked.
- **Recurring failures**: removing/withholding stash admission, promotion-gate widening (occupancy-ceiling-bound in every shape), front-landing cold keys, geometry trades, and seam-combination overlap.
- **Open questions**: whether single-workload search has saturated, and the long-deferred second-trace validation.
- **Per-policy notes**: grouped under DASH — recency over LFU for the protected tier, preserve bucket count, stash as load-bearing dwell-time buffer, promotion-side saturation vs. eviction/demotion frontier.


---

- finished: 2026-06-05T20:25:14+00:00
- duration: 38.42s
- status: ok
- input_tokens: 5
- output_tokens: 2014
- cache_read_tokens: 69597
- cache_write_tokens: 12659
- cost_usd: 0.105778
