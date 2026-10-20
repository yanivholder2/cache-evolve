# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0008\cross-cycle
- allow_edits: True
- started: 2026-07-22T02:50:27+00:00

## timeline

### 02:50:33 thinking



### 02:50:34 tool: Bash

```json
{
  "command": "pwd && ls",
  "description": "Show working directory and contents"
}
```

### 02:50:37 thinking



### 02:50:59 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\associative_cache_2026-07-21_23-22-35-670\\cycles\\0008\\cross-cycle\\cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Trim dead metadata off the eval path.** Fields carried only by\n  off-eval-path policies are pure dead weight; peeling them one at a time\n  is monotonic and hit-rate-inert, and it reclaims the metadata floor\n  cheaply. This paid off repeatedly and was the first unambiguous move.\n- **Park state off the measured `Item`.** The size metric weighs only\n  `Item`, so scan-resistance state kept in the bucket struct (reuse bits,\n  segment boundaries, caps) is free on the size axis. This exploit broke\n  the early hit-rate/metadata antagonism and has held every time.\n- **Index-preserving relocation of parallel state.** Mirroring a policy's\n  own single-element move onto its parallel reuse vector — instead of\n  key-matched snapshot-and-rebuild each hit — repeatedly collapsed blown-out\n  p99 tails while holding hit-rate bit-for-bit. It is the portable latency\n  win of the run.\n- **Segmentation (SLRU/2Q-style) is the one lever that raised hit-rate.**\n  Reserving a protected segment for twice-hit keys was the only change to\n  cross the front-admission ceiling, and it does so at the metadata floor.\n\n## Recurring failures\n\n- **Re-timing promotion on a single list.** Slow/gradual promotion\n  under-protects hot keys on short buckets; reuse-bit gating is redundant\n  with position. Gradual promotion needs more slots than a small bucket has.\n- **Frequency/LFU victim signals.** LFU tiebreaks and least-frequently-reused\n  eviction lost to plain recency-front every time it was tried.\n- **Second-chance / CLOCK aging.** Self-clearing protection sheds immunity\n  too fast and collapses toward LRU; permanent binary protection fits this\n  trace.\n- **Stacking explicit protection onto front-admission.** Under\n  move-to-back-on-hit the front slot is already the probationary one, so an\n  added reuse bit is behaviorally redundant.\n- **Buying hit-rate through more depth/memory/protection.** Wider\n  associativity (coarsens hash spread), 2Q ghost memory (over-inflates the\n  protected set), and near-total protection caps all regressed — three walls\n  mapped around the segmentation ceiling.\n- **Sub-resolution micro-optimizations.** Removing a single-element rotation\n  below the ~100ns timer is correct but unrewardable by the harness.\n- **Overlay hygiene by omission.** Single-file variants repeatedly reverted\n  the parent's slim `Item` back to the base layout — a win lost by omission,\n  not logic. Recurred across several cycles until parents were anchored\n  explicitly.\n\n## Open questions\n\n- Is two-segment SLRU at diverse small buckets the structural hit-rate\n  ceiling for this trace, or is there headroom no explored lever reaches?\n- Where does the protected cap actually optimize? Both extremes are\n  falsified and mid-range reads nearly flat — treat as settled or search\n  the mid-range once more?\n- With the O(n²) reuse rebuild gone, do remaining tails live in the\n  `insert(0)`/`remove(0)` front shifts, or is the path at its latency floor?\n- Has the binding constraint shifted entirely from hit-rate to\n  harness-resolvable latency?\n\n## Per-policy notes\n\n- **ClassicLRU / recency-front:** the settled baseline. Recency-front victim\n  choice beat every frequency-based alternative; the front slot is the\n  optimum on the admission-geometry axis (midpoint/wider probation regress).\n- **Reuse-protection (binary):** permanent protection is the feature, not a\n  bug, on this trace; refinements must preserve it. Effective only over a\n  policy that does *not* already positionally segregate reused items.\n- **SLRU / segmented:** the incumbent and the only ceiling-breaker; delivers\n  its gain at the 16-byte floor. Cost moved from metadata to per-access\n  rotation latency. Protected-cap knob mapped end-to-end and is nearly flat;\n  per-hit protected rotation is overhead except in the saturated\n  (all-protected) bucket, where it silently maintains eviction order.\n- **2Q / ghost memory:** cross-eviction memory did not lift hit-rate — a\n  re-reference is not proof of long-term heat, and blanket pre-protection\n  erodes reuse-bit discrimination while taxing every put.\n"
}
```

### 02:51:05 chat

Wrote `cross-cycle-insights.md` as a fresh projection of the 8-cycle ledger, using the four fixed sections in order:

- **Recurring wins** — the durable, cross-cycle patterns: trimming dead metadata off the eval path, parking state off the measured `Item`, index-preserving relocation of parallel reuse state, and segmentation as the sole hit-rate lever.
- **Recurring failures** — approaches that repeatedly failed with *why*: single-list promotion re-timing, LFU/frequency victims, second-chance aging, redundant protection stacking, buying hit-rate via more depth/memory/protection, sub-resolution micro-opts, and overlay-hygiene-by-omission.
- **Open questions** — the still-live threads: structural ceiling of two-segment SLRU, the flat protected-cap knob, where remaining tails live, and the hit-rate→latency frontier shift.
- **Per-policy notes** — grouped by ClassicLRU/recency-front, binary reuse-protection, SLRU/segmented, and 2Q/ghost memory.

Signals are qualitative (no invented numbers) and combine observations across cycles rather than restating single-cycle notes.


---

- finished: 2026-07-22T02:51:06+00:00
- duration: 38.64s
- status: ok
- input_tokens: 5
- output_tokens: 2403
- cache_read_tokens: 76991
- cache_write_tokens: 13229
- cost_usd: 0.187851
