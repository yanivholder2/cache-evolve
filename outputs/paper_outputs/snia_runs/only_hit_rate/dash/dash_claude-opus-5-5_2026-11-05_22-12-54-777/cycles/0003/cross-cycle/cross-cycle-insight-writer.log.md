# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-11-05T22:33:37+00:00

## timeline

### 22:33:51 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Making one isolated change per variant gives results you can read. Multi-lever bundles (cycle 0002 v001) hid which part caused a regression. Single-lever tests in cycles 0001 and 0003 gave clear causes.\n- The 24/8 normal/stash topology is a solid base. It improved every cell in cycle 0001, the frequency filter stacked on top of it cleanly in cycle 0002, and shrinking the stash further did not help.\n- A frequency-aware LRU-tail victim (8-slot window, stash hits counted) is the one lever that moved the small cache size. Since cycle 0002, every attempt to cheapen or bypass it has lost hit-rate.\n- Changes that act inside a bucket (victim choice) are what move the small size. Topology changes only help at the large size.\n- hm_0 is the most responsive trace. Gains, and losses, show up there first and largest.\n\n## Recurring failures\n\n- MRU-end promotion has hurt web_0 in two cycles: bundled in cycle 0002 and on its own in cycle 0003. Once the tail scan exists, it also moves new arrivals out of the scan window, so the scan evicts established residents instead. Front landing is part of how the filter works.\n- Insert-position tweaks (MRU landing, counter-gated demotion) are redundant with, or harmful to, the frequency-based victim choice. The insert-position lever is spent.\n- Sweeping the normal/stash ratio beyond 3/4 starves promotion on hm_0, because the stash is the only way into the protected tier. The ratio has peaked.\n- Topology-only changes at the small size do not help.\n- Per-op-time and metadata-size have gone unmeasured for three cycles, even though it was requested every time. So cost-reduction variants (narrower window, lower cap) can only show hit-rate losses and can never show what they save.\n\n## Open questions\n\n- What do the 8-slot scan and halve-on-cap ageing actually cost in per-op-time? This needs a real measurement before any further trade of cost against hit-rate.\n- In cycle 0003, which change caused the small-size loss on hm_0: the narrower window or the lower counter cap? Testing each on its own (window 4 with cap 15, window 8 with cap 7) would separate them.\n- Would a window wider than 8, or a different ageing rule, help hm_0 more?\n- hm_0's long-gap reuse is still unserved. Would more stash capacity or a longer stash wait (without changing the split), or a different admission or stash-promotion rule, reach it?\n- Would a counter-gated stash→target landing keep MRU's small hm_0 gain without hurting web_0?\n- The incumbent's hit-rate stalled in cycle 0003. Is the current design near a plateau?\n\n## Per-policy notes\n\n- Baseline (normal/stash with target/probing tiers): the protected tier gave little protection. Promoted items were evicted quickly or crowded out.\n- Topology (24/8): best at the large size. It has peaked at a 3/4 split, and 28/4 regressed on hm_0.\n- Frequency-aware tail victim (incumbent since cycle 0002): improved all four cells, with the biggest gain at the small size. Narrowing the window or capping counters harder cost hit-rate, most of it on hm_0 at the small size.\n- MRU promotion and landing: helps hm_0 slightly but consistently hurts web_0. It conflicts with the tail-scan filter.\n- Gated demotion: neutral, adds nothing over the scan.\n"
}
```

### 22:33:58 thinking



### 22:33:58 chat

I've written `cross-cycle-insights.md` for cycles 0001–0003, with the four required sections in order.

**Recurring wins**
- The 24/8 normal/stash split is a solid base.
- Choosing the eviction victim from the last 8 items by how often they've been used is the one change that improved the small cache size.
- Testing one change per variant makes the results readable.
- hm_0 is the trace that reacts most to changes.

**Recurring failures**
- Promoting items to the most-recent end hurt web_0 in two cycles, the second time when tested alone. It works against the frequency-based victim choice.
- Changing where items are inserted has stopped paying off.
- Splits beyond 3/4 starve promotion on hm_0.
- Topology changes alone don't help at the small size.
- Per-op-time hasn't been measured in any of the three cycles, so changes meant to cut cost can't be judged.

**Open questions**
- What the victim scan actually costs per operation.
- Which of the two cycle 0003 changes caused the small-size loss on hm_0: the narrower window or the lower counter cap.
- Whether a wider window or a different counter-ageing rule would help more.
- How to serve hm_0's reuse after long gaps.
- Whether the design is near a plateau, since hit-rate stalled in cycle 0003.

**Per-policy notes** cover the baseline, the topology change, the frequency-aware victim choice (the current best), recent-end promotion, and the demotion change.

The file uses qualitative terms only; no numbers were added beyond the ones in the ledger.


---

- finished: 2026-11-05T22:33:58+00:00
- duration: 21.49s
- status: ok
- input_tokens: 4
- output_tokens: 1850
- cache_read_tokens: 37016
- cache_write_tokens: 8365
- cost_usd: 0.111339
