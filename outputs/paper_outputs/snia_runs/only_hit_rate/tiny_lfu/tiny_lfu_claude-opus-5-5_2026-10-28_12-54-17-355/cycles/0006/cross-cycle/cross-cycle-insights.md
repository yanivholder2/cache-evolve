# Cross-Cycle Insights (through cycle 0006)

## Recurring wins

- Progress comes from changing the admission contest: who competes, which resident is the victim, how sharp the frequency comparison is. Real wins so far: 1% window (0001), adaptive window (0002), SLRU main (0003), coldest-of-4 sampled probation victim (0005).
- Structural changes add up. Window, then SLRU, then victim sampling each stacked with no regression. Timing and feedback changes almost never stack.
- Changes that make selection by frequency sharper keep paying off on web_0, especially where scan-like one-hit traffic dominates.
- Strict `>` admission, where the victim wins ties, has held up whenever it was challenged (0001 cap interaction, 0004 tie-break). Keep it.
- The real best base is the cycle 0005 lineage: adaptive window, adaptive protected split, coldest-of-4 victim, double counting kept. Cycle 0006's named winner did not beat it.

## Recurring failures

- Window/protected controller redesign has failed in every cycle since 0003: capacity-tied periods, fixed periods, turnover periods, alternating dual climbers, per-event ARC ghost control. Causes: lazy rebalance lag, noisy short samples, integer step truncation, warmup bias, and ghost lists that fill unevenly under the TinyLFU filter.
- Any horizon or period that scales with capacity misbehaves at one end of the size range: aging resets, climber samples, ghost size capacity/2 (which leaked scans on web_0 large).
- Stacking frequency-favouring tweaks gives diminishing or negative returns (single counting, wider victim minimum, re-entry tie bonus on an unaged sketch). Each one lowers the bar or removes the newcomer's edge. web_0 large is where this shows up first.
- Victim-choice tweaks at the probation LRU end do not reach hm_0. Victim samples wider than 4 pick entries that only look cold because the estimates are noisy.
- Counter caps and recency tie-breaks create mass ties, which hurts admission.
- Hypotheses aimed at hm_0 small have failed in every cycle. In each case the predicted mechanism was refuted.
- Metadata-size and per-op-time have gone unmeasured for six cycles, despite being asked for every cycle.

## Open questions

- With SLRU and sampling in place, does sketch aging or decay (halving, doorkeeper reset) now help? The cycle 0003 verdict against halving came before these existed. Two variants in 0006 named the unaged sketch as the root cause.
- If aging holds web_0 large, do single counting (correct accounting) and a re-entry bonus then start to pay off on top of it?
- Would a less noisy victim estimate or an explicit contest margin sharpen admission better than a wider minimum? Options: median of the sample, conservative-update sketch.
- Does hm_0 need an explicit recency or re-entry signal with a short, fixed horizon, or a different landing position and residence time in probation?
- Is any window controller worth reviving? Only if it uses a balanced, batched counterfactual signal and is suppressed until the cache is full.
- What is the real metadata and latency cost of the third list, sampling estimates, ghost sets and full-table resets?

## Per-policy notes

- **W-TinyLFU window**: adaptive beat fixed 1%, mainly at small sizes. The climber is effectively frozen. Retuning its timing is a dead end so far.
- **SLRU main (80/20)**: the only thing that moved web_0 large under the adaptive window. It costs hm_0 slightly, because the short probation stay hurts recency-driven keys. The adaptive protected split was a marginal, near-flat gain.
- **Count-min sketch / aging**: a capped sketch caused regressions. Cap-free halving helped before SLRU but hurt after the adaptive window. Aging that counted reset periods per sketch increment (where a miss counts twice) fired too often on small caches. Unaged double counting quietly acts as the newcomer's only recency weight. Revisit aging on the current base before changing the counting.
- **Victim selection**: coldest-of-4 from the probation tail is the sweet spot. 8 overfits to noise.
- **Ghost / ARC-style**: a poor fit as a window controller. As a re-entry bonus it needs a fixed, short horizon and a bonus that matters beyond ties.
- **Traces**: web_0 is the main source of gains, and web_0 large is the most fragile cell. hm_0 is largely served by recency at large sizes, and hm_0 small is the persistent unsolved cell.
