# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:27:19+00:00

## timeline

### 04:27:28 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Structural separation of one-touch entries from re-referenced ones (a probationary/protected split inside each bucket) is the one lever that has paid off clearly, and it has held across cycles 2-5.\n- A small probation share (about slots/16, roughly 4 slots in a wide bucket) beats larger shares. Independent routes converged on it.\n- A lean positional split, with no counters, ghost state or adaptive targets, captures nearly all of the gain over plain LRU. It was rebuilt on the baseline twice and won both times.\n- Wider buckets gave a small, real hit-rate gain (cycle 1) and are the base geometry.\n\n## Recurring failures\n\n- Weakening hit-path promotion in plain LRU buckets (transposition, midpoint, half-promotion) lost hit rate.\n- Delaying promotion (two-hit rules) hurt, because small probation segments evict entries before they get a second hit. First-hit promotion looks hard to beat.\n- Ghost history (rings, ghost-hit direct promotion, adaptive targets) added nothing over the split. A ghost hit is weak reuse evidence here.\n- Eviction-side tweaks do not help: small-window frequency victim choice was flat, and recency-stamp overrides of probation-first eviction lost hit rate.\n- Placement that moves keys outside their hashed bucket fails the slot-legality invariant.\n- Shrinking probation below the slots/16 size slightly hurt, so size tuning is exhausted.\n- Per-op-time and metadata-size have been missing from the fitness vector in every cycle. All claimed wins are hit-rate only.\n- Several variants are compared with the baseline rather than their parent, so their gains are not increments.\n\n## Open questions\n\n- Do the hit-rate gains survive once per-op-time and metadata-size are measured? In particular, what do reordering and demotion on hits cost, and does the lean split really match baseline footprint?\n- Is there any hit-rate headroom left in protected-segment handling or promotion, or is the design at its ceiling on this trace?\n- Is the tuning specific to the LIRS-style trace? Robustness on other workloads is untested.\n- Could a stricter ghost rule (ghost hit plus a second signal) rescue ghost history? This is low priority.\n- Is an invariant relaxation worth considering to allow multi-bucket placement?\n\n## Per-policy notes\n\n- Plain move-to-back LRU: the baseline. It is hard to beat via promotion tweaks, and its misses are mostly policy misses, not hash imbalance.\n- Segmented LRU (probation/protected) with a small positional probation: the current best. Use the lean version as the base.\n- Segmented LRU with extras (ghost rings, per-entry counters, adaptive targets, stamps): no gain over the lean split, with unpriced state cost. Candidates for removal.\n- Frequency-aware victim choice: flat in a small window.\n- Two-choice placement: rejected at the invariant gate.\n"
}
```

### 04:27:31 chat

I wrote `cross-cycle-insights.md` in the cross-cycle output directory. It has the four required sections, in the required order, built only from the five-cycle ledger.

- **Recurring wins:** A probation/protected split in each bucket is the lever that has paid off, and a small probation share (about slots/16) works best. A lean positional version with no extra state captures nearly all the gain. Wider buckets gave a small gain and are the base geometry.
- **Recurring failures:**
  - Weakened or delayed promotion lost hit rate.
  - Ghost history added nothing over the split.
  - Eviction-side tweaks did not help.
  - Placement outside the hashed bucket fails the slot-legality invariant.
  - Shrinking probation below slots/16 slightly hurt.
  - Per-op-time and metadata-size have been missing in every cycle, so all wins are hit-rate only.
- **Open questions:** Do the gains hold once speed and size are measured? Is there any headroom left, or is the design at its ceiling? Does the tuning carry over to other traces?
- **Per-policy notes:** These cover plain LRU, the lean segmented LRU (the current best), segmented LRU with extra state, frequency-aware victim choice and two-choice placement.

I used qualitative terms only and did not invent any numbers.


---

- finished: 2026-10-21T04:27:32+00:00
- duration: 13.34s
- status: ok
- input_tokens: 4
- output_tokens: 1570
- cache_read_tokens: 37171
- cache_write_tokens: 8848
- cost_usd: 0.055307
