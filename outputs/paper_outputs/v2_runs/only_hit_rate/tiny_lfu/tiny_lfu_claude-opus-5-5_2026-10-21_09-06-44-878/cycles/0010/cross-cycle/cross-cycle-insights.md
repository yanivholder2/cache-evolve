# Cross-Cycle Insights

## Recurring wins

- Lowering the noise in the estimates gave the only large hit-rate gain in ten cycles. That was the wider count-min sketch in cycle 0003. Every other Winner was marginal or won only by tying the parent.
- Honest counts helped a little. Dedup of the get-miss-then-put double increment (0004) brought the sketch in line with true access counts, and it should also reduce per-op work.
- Unaged lifetime frequency is the right signal for workload-a. Every attempt to dilute it lost, and every attempt to sharpen it lost too. The plain access count, weighted equally for every access, already sits at the right balance.
- Strict `>` with keep-victim-on-tie stays. Every change to tie handling or admission was inert or worse.
- The sketch is over-provisioned. Width and conservative update substitute for each other. Half width with u16 counters plus conservative update was neutral (0005), except inside SLRU, where about 8× width looks like the floor (0006). Capping counters at 31 is inert on hit-rate (0009), so 5-bit counters should be free.
- Speed-oriented, semantics-preserving changes reliably tie on hit-rate. Examples are the single fast hash (0001) and the hash-cached fast path with ≤ 1 shortcuts (0005).

## Recurring failures

- **Aging and decay** lost in every form tested: halving at short or long periods, with or without conservative update, on the noisy sketch, on the clean sketch, and on the current SLRU+dedup design (0001, 0002, 0004, 0008). Per-entry idle discounts and idle-based demotion also lost (0008). Variant takeaways keep suggesting aging anyway. Treat that suggestion as refuted.
- **Recency evidence in the admission contest** failed in all three forms tried: ghost tie-breaks, ghost-hit margins and random admission of warm losers (0007–0008). Exact ties almost never happen at warm estimates, and wider margins churn cold probation keys.
- **Victim-selection mechanics** are inert or negative. This covers sampling depth from 3 to 8 and wider (0003, 0009), CLOCK-style rotation, and adding the protected head as a candidate (0010). The minimum estimate in the sample does not move, because old probation keys share near-floor counts.
- **Admission windows** did not pay off. A 1% window was inert on the noisy sketch (0001) and negative on clean SLRU (0006). An adaptive hill-climbing window grew toward LRU and lost (0002).
- **SLRU internals** are over-defended. Changing the demotion choice, the split ratio, promotion gating or idle demotion never moved hit-rate (0006–0010).
- **Tilting the contest toward incumbents** also lost: double-weighting hits starves keys that were rejected (0010).
- **Variants aimed at metadata-size or per-op-time** can only tie or lose, because those dimensions have never been scored in ten cycles.

## Open questions

- Are metadata-size and per-op-time scored yet? This is the binding question. If they are, re-test these four cost leads:
  - 5-bit counters (cap 31)
  - a narrow conservative-update sketch on SLRU, with a recency tie-break between equal estimates
  - the hash-cached fast path
  - pricing the extra per-op work in SLRU, ghost ring, promotion estimates and rotation
- Would a 4-bit cap (15) change comparisons? It has never been tested.
- Do other workloads leave room that workload-a doesn't? On workload-a, TinyLFU+SLRU looks to be at the trace's ceiling.
- Two minor tests are still open: raw-estimate protected-head demotion alone, and halving with a period far beyond 10×capacity. Both are expected to converge to the parent, so they are low priority.

## Per-policy notes

- **TinyLFU sketch:** the gains come from estimate quality (width, then dedup), and that lever is now exhausted. Width, depth and conservative update overlap, and the counters can be narrow. If any decay or reset is ever reintroduced, cycle 0005's ≤ 1 fast-path shortcuts must be removed, because they assume counts never age.
- **Admission contest:** keep strict `>` with probation-only victim sampling of about 8, settled by the min-estimate rule. Tested and rejected: tie-admission, jitter, ghost bonuses and changes to sample depth or placement.
- **W-TinyLFU window:** negative on workload-a, whether fixed or adaptive. Few keys there come back within a short recency trial.
- **SLRU (probation/protected):** gave a marginal gain over sampled victims alone (0005). It largely duplicates the soft protection that min-estimate sampling already provides. Protected keys are stale in recency but not in count, so no rule based only on frequency can reach them.
- **Current baseline lineage:** wide sketch with dedup, then SLRU, then a series of hit-rate-neutral Winners (0006–0010). Hit-rate has been flat for six cycles in a row.
