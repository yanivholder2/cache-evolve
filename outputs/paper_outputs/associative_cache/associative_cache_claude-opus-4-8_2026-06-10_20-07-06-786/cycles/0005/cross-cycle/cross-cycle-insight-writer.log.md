# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-06-10T20:46:40+00:00

## timeline

### 20:46:44 thinking



### 20:46:44 tool: Bash

```json
{
  "command": "pwd && ls",
  "description": "Show working directory and contents"
}
```

### 20:46:48 thinking



### 20:47:02 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Explicit segmentation with a promote-on-first/second-hit rule is the\n  durable breakthrough: demanding proof of reuse before granting protection\n  beat both pure-LFU victim selection and single-position LRU admission, and\n  every later baseline has been built on it.\n- Tuning *which* probationary entry dies is high-leverage at zero metadata\n  cost: reversing probation to oldest-first (FIFO) gives a fresh key a full\n  lap to earn promotion and lifted hit-rate on its own.\n- The earliest, cleanest gains came from intra-bucket victim/admission logic\n  (frequency-first eviction over pure-LRU), leaving routing, capacity\n  invariants, and the policy enum untouched.\n\n## Recurring failures\n\n- Anything that *erodes durable, earned protection* regresses on this\n  collision-bound trace: counter aging, front-admission composition,\n  second-hit promotion thresholds, capped/demoting protected segments. Once a\n  key proves hot it wants to stay protected (\"once-hot immortality\").\n- \"Recently evicted\" is not a reliable proxy for \"hot\": ghost-ring fast\n  re-entry into the protected segment let weakly-warranted keys seize space\n  they did not earn and regressed.\n- Approximating recency away (CLOCK one-bit reference) underperforms full\n  positional ranking in small buckets.\n- Orthogonal levers that each helped alone do not necessarily compose —\n  frequency-first eviction plus probationary front admission actively\n  conflict, making fresh keys the immediate victim.\n- Geometry coarsening (wider buckets) gave the smallest gain and hit\n  diminishing returns early.\n\n## Open questions\n\n- Is hit-rate now at its ceiling on this trace? Admission, promotion,\n  protected-eviction signal, and re-entry have all been ruled out.\n- Does the one genuinely untested hit-rate axis — looser/larger protected\n  sizing and bucket-geometry interaction — move the needle, since *enlarging*\n  protection (rather than tightening or re-routing it) has never been tried?\n- Cost-side wins (reorder-free tiebreak, no-op reorder elision) are real but\n  invisible to a hit-rate-only fitness and always tie. Is a per-op-time\n  fitness axis warranted before spending more cohort slots on them?\n\n## Per-policy notes\n\n- Pure-LRU victim selection: superseded early by frequency-first eviction.\n- Frequency/LFU signals: useful as a promotion flag, but as a victim or\n  re-entry signal they let `==1` newcomers dominate and kill rising keys;\n  banked counts do not separate hot from rising better than protected-LRU\n  recency.\n- Segmented-LRU (FIFO probation + promote-on-first-hit): the strongest scheme\n  found and the standing baseline; behavioral logic appears saturated for\n  hit-rate. The protected segment is load-bearing and never saturates, so its\n  cap-and-demote branch is effectively dead code on this trace.\n- CLOCK / one-bit recency: too coarse for small buckets; loses warm keys.\n"
}
```

### 20:47:08 chat

Wrote `cross-cycle-insights.md` as a fresh projection of the five-cycle ledger digest. Key synthesis across cycles:

- **Recurring wins**: segmentation with proof-of-reuse promotion is the durable breakthrough; probation order (oldest-first/FIFO) is a free high-leverage lever; early intra-bucket frequency-first eviction.
- **Recurring failures**: every protection-eroding scheme (aging, thresholds, caps, ghost rings, front-admission composition) regresses on this collision-bound trace; coarse recency approximation and geometry coarsening also lose.
- **Open questions**: whether hit-rate is at ceiling, whether *larger* protected sizing helps, and whether a per-op-time fitness axis is needed to reward the cost-side ties.
- **Per-policy notes**: grouped observations for pure-LRU, frequency/LFU, segmented-LRU baseline, and CLOCK.


---

- finished: 2026-06-10T20:47:09+00:00
- duration: 28.83s
- status: ok
- input_tokens: 5
- output_tokens: 1712
- cache_read_tokens: 60328
- cache_write_tokens: 7730
- cost_usd: 0.076579
