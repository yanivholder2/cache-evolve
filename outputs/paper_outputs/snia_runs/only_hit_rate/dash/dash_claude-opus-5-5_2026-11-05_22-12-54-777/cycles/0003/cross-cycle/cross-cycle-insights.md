## Recurring wins

- Making one isolated change per variant gives results you can read. Multi-lever bundles (cycle 0002 v001) hid which part caused a regression. Single-lever tests in cycles 0001 and 0003 gave clear causes.
- The 24/8 normal/stash topology is a solid base. It improved every cell in cycle 0001, the frequency filter stacked on top of it cleanly in cycle 0002, and shrinking the stash further did not help.
- A frequency-aware LRU-tail victim (8-slot window, stash hits counted) is the one lever that moved the small cache size. Since cycle 0002, every attempt to cheapen or bypass it has lost hit-rate.
- Changes that act inside a bucket (victim choice) are what move the small size. Topology changes only help at the large size.
- hm_0 is the most responsive trace. Gains, and losses, show up there first and largest.

## Recurring failures

- MRU-end promotion has hurt web_0 in two cycles: bundled in cycle 0002 and on its own in cycle 0003. Once the tail scan exists, it also moves new arrivals out of the scan window, so the scan evicts established residents instead. Front landing is part of how the filter works.
- Insert-position tweaks (MRU landing, counter-gated demotion) are redundant with, or harmful to, the frequency-based victim choice. The insert-position lever is spent.
- Sweeping the normal/stash ratio beyond 3/4 starves promotion on hm_0, because the stash is the only way into the protected tier. The ratio has peaked.
- Topology-only changes at the small size do not help.
- Per-op-time and metadata-size have gone unmeasured for three cycles, even though it was requested every time. So cost-reduction variants (narrower window, lower cap) can only show hit-rate losses and can never show what they save.

## Open questions

- What do the 8-slot scan and halve-on-cap ageing actually cost in per-op-time? This needs a real measurement before any further trade of cost against hit-rate.
- In cycle 0003, which change caused the small-size loss on hm_0: the narrower window or the lower counter cap? Testing each on its own (window 4 with cap 15, window 8 with cap 7) would separate them.
- Would a window wider than 8, or a different ageing rule, help hm_0 more?
- hm_0's long-gap reuse is still unserved. Would more stash capacity or a longer stash wait (without changing the split), or a different admission or stash-promotion rule, reach it?
- Would a counter-gated stash→target landing keep MRU's small hm_0 gain without hurting web_0?
- The incumbent's hit-rate stalled in cycle 0003. Is the current design near a plateau?

## Per-policy notes

- Baseline (normal/stash with target/probing tiers): the protected tier gave little protection. Promoted items were evicted quickly or crowded out.
- Topology (24/8): best at the large size. It has peaked at a 3/4 split, and 28/4 regressed on hm_0.
- Frequency-aware tail victim (incumbent since cycle 0002): improved all four cells, with the biggest gain at the small size. Narrowing the window or capping counters harder cost hit-rate, most of it on hm_0 at the small size.
- MRU promotion and landing: helps hm_0 slightly but consistently hurts web_0. It conflicts with the tail-scan filter.
- Gated demotion: neutral, adds nothing over the scan.
