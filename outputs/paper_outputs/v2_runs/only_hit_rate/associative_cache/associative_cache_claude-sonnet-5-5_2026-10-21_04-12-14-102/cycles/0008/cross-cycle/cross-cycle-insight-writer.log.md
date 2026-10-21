# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:36:46+00:00

## timeline

### 04:36:56 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Structural separation of one-touch from re-referenced entries (a probationary/protected split in each bucket) is the one large, repeatable gain. It was confirmed from several independent routes (cycles 2-8).\n- A lean positional split with no extra per-entry state (no ghost, counters or adaptive targets) captures nearly all of that gain. It has been rebuilt on the baseline several times and each time beat plain LRU.\n- A small probation segment (about slots/16) beats larger ones. The size trend turned in cycle 5, so the current size is at or near the optimum.\n- Wider associativity stacks on the split, but with sharply diminishing increments: a clear gain at first, then marginal, then marginal again. Proportional probation sizing held up better than an absolute size.\n- Inserting new keys just above the probation boundary keeps scan protection without starving them of residency.\n\n## Recurring failures\n\n- Weakening hit-path promotion in plain LRU buckets (transposition, midpoint, half-promotion) lost hit rate.\n- Delaying promotion (two-hit rule) or victim-end insertion (LIP-style) lost hit rate. The trace has short reuse distances, so new keys need residency to earn a first re-reference.\n- Ghost history and direct ghost-hit promotion added nothing over the split. A ghost hit is weak reuse evidence and can displace entries that had earned protection.\n- Small-window frequency victim choice and recency-stamp overrides of probation-first eviction were flat or negative. The segment boundary already encodes the reuse evidence.\n- Demotion placement is a neutral lever (exact ties).\n- Multi-bucket or two-choice placement fails the slot-legality invariant. It is incompatible with the gate as written.\n- Many \"wins\" are ties, or are measured against the baseline instead of the current parent. They are not real increments.\n\n## Open questions\n\n- Per-op-time and metadata-size have been missing from the fitness vector in all 8 cycles. Is this fixable? No cost-side claim has ever been confirmed: the wide-bucket scan cost, the rotate refactors and the skip-move rule are all unjudged.\n- Do the exact-tie results reflect real neutrality, or edits that never changed behaviour? Authors could not read the overlay code.\n- Is the hit-rate ceiling of this design on the trace reached? Would a different policy family (such as frequency-aware admission) be needed?\n- Do the results hold on traces other than the LIRS-style one?\n- Is a mid-probation insertion point better than either extreme?\n\n## Per-policy notes\n\n- Plain move-to-back LRU: the baseline. Every promotion-weakening tweak lost to it.\n- Segmented LRU (probation/protected, first-hit promotion, small probation of about slots/16, boundary insertion): the current incumbent. Its hit-rate levers (size, promotion, demotion, insertion, ghost state, associativity) are nearly exhausted.\n- Ghost-ring or adaptive 2Q variants: unnecessary state, with no gain over the lean split.\n- Frequency-based victim choice: no effect at small windows.\n- Wide buckets (2x-4x associativity): a small hit-rate gain with unpriced scan cost. Accept this only if per-op-time shows the cost is tolerable.\n- Direction: stop tuning hit-rate knobs. First measure speed and size, then try a different policy family or other workloads.\n"
}
```

### 04:37:00 chat

I wrote `cross-cycle-insights.md` in the cycle 0008 cross-cycle output directory, with the four sections in the required order. I built it only from the ledger digest in the prompt, and it names no specific numbers.

The file's main conclusions:
- **Recurring wins:** The one large, repeatable gain is a probation/protected split in each bucket. A lean version with no extra per-entry state gets nearly all of it, and a small probation segment (about slots/16) is at or near the optimum. Wider buckets stack on top, but each doubling adds less.
- **Recurring failures:** Weaker promotion, delayed promotion, victim-end insertion, ghost history and small-window frequency victim choice all lost hit rate or did nothing. Two-choice placement fails the slot-legality invariant. Several "wins" were exact ties or were measured against the baseline instead of the current parent.
- **Open questions:** Per-op-time and metadata-size have been missing from every cycle's fitness vector, so no cost claim has been confirmed. The file also asks whether the exact ties are real neutrality, whether the design has hit its ceiling on this trace, and whether results hold on other traces.
- **Per-policy notes:** These are grouped by plain LRU, segmented LRU, ghost/adaptive variants, frequency victim choice and wide buckets. The closing direction is to stop tuning hit-rate knobs, measure speed and size first, then try a different policy family or other workloads.


---

- finished: 2026-10-21T04:37:01+00:00
- duration: 15.23s
- status: ok
- input_tokens: 4
- output_tokens: 1763
- cache_read_tokens: 39447
- cache_write_tokens: 11270
- cost_usd: 0.06738
