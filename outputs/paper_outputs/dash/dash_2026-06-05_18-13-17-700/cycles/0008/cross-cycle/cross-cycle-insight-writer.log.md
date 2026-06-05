# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-06-05T19:49:39+00:00

## timeline

### 19:49:44 thinking



### 19:49:45 tool: Bash

```json
{
  "command": "pwd && ls",
  "description": "Identify working/output directory"
}
```

### 19:49:58 thinking



### 19:50:02 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Reallocating fixed budget toward the promotion-protected normal tier was\n  the dominant early win: it climbed hit-rate from the baseline through past\n  even and into normal-dominant before saturating.\n- Protecting the warm working set is the through-line behind every gain —\n  whether by capacity, insertion position, promotion gating, or demotion\n  landing — transient/cold keys must be kept from displacing resident hot keys.\n- Stash dwell-time is the most durable late-stage seam: giving cold keys time\n  to prove a second reference (two-speed promotion, relocation hop, bounded\n  multi-hop cascade) yielded the only fresh positives once geometry saturated.\n- Reactive rescue of genuine collision victims beats proactive balancing;\n  preserving primary-bucket locality consistently outperforms even fill.\n\n## Recurring failures\n\n- Removing or gutting stash admission regresses sharply — the admission step\n  is load-bearing warm-set protection, not dead probation.\n- Front-inserting cold keys at the stash eviction edge regressed twice; cold\n  keys need dwell time, so denying residency is consistently wrong.\n- Promotion-gate widening of every shape (quarter, half, len-2 loosening,\n  density-independent touch counters) is capped by a structural small-bucket\n  occupancy ceiling, not by rule shape — repeatedly flat or negative.\n- Trading bucket count for bucket width regressed: addressability beats depth.\n- Independently-discovered seams overlap rather than compound — stacking late\n  slivers (relocation onto protected tier; depth/victim/breadth on the winner)\n  comes back flat because they chase the same boundary-key pool.\n\n## Open questions\n\n- Is every hit-rate lever ultimately gated by the same hashing skew /\n  small-bucket occupancy, or is stash dwell-time structurally different?\n- With combination shown non-additive repeatedly, does any pair of discovered\n  seams stack on this trace, or do all compete for one thin boundary pool?\n- The cohort has searched a single high-skew trace across all cycles —\n  validating against a second, less-skewed trace is long overdue and now the\n  highest-value next step before further single-workload variants.\n\n## Per-policy notes\n\n- Recency / promote-on-hit (protected normal tier): the workhorse; suits the\n  skewed trace and underpins every banked win. Promotion mechanics (position,\n  two-speed) gave only slivers once capacity saturated.\n- LFU on the protected tier: a dead lever — retained stale-but-frequent keys\n  over the live working set and regressed.\n- Stash / admission probation: LRU-style dwell-time is the value; relocation\n  and bounded cuckoo cascades extend it, but depth inverts once the protected\n  tier is present (it evicts already-warmed residents).\n"
}
```

### 19:50:07 chat

Wrote `cross-cycle-insights.md` as a fresh projection of the 8-cycle ledger. Summary of what the four sections capture:

- **Recurring wins**: protected-tier capacity reallocation (the dominant early gain), warm-set protection as the through-line, stash dwell-time as the durable late seam, and reactive rescue beating proactive balancing.
- **Recurring failures**: removing/gutting stash admission, front-inserting cold keys, promotion-gate widening (all shapes hit the small-bucket ceiling), bucket-width-for-count trades, and non-additive seam stacking.
- **Open questions**: whether everything is gated by hashing skew, whether any seams stack, and the long-overdue second-trace validation.
- **Per-policy notes**: grouped by recency/promote-on-hit (workhorse), LFU (dead), and stash/dwell-time (value source, but depth inverts atop the protected tier).


---

- finished: 2026-06-05T19:50:07+00:00
- duration: 27.93s
- status: ok
- input_tokens: 5
- output_tokens: 1627
- cache_read_tokens: 65848
- cache_write_tokens: 10388
- cost_usd: 0.088964
