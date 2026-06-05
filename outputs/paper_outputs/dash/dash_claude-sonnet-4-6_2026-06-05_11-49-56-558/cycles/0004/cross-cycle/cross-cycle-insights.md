## Recurring wins

- **Hot-tier topology expansion** drove the largest gains across Cycles 1–2: growing normal buckets at the expense of stash slots consistently lifted hit-rate more than any policy change, because capacity was the binding constraint for workload-a's stable hot set.
- **MRU re-entry (Back-insert on stash hit)** produced genuine additive improvements whenever layered on top of topology wins (Cycles 1–3); it never caused a regression and compounds cleanly with structural changes.
- **Position-based recency (LRU ordering)** has been the dominant eviction signal in every cycle where it was tested head-to-head against frequency-based alternatives; item position reliably encodes future-access probability better than any counter variant tried so far.
- **24/8 normal/stash geometry + MRU re-entry** is the confirmed compound optimum: topology peaked here (Cycle 3) and re-entry adds a small but clean increment on top.

## Recurring failures

- **Frequency-based eviction in all forms** (LFU, aging-LFU, threshold-gated decay, per-hit co-resident decay): tried across Cycles 1–4 in progressively gentler formulations; all underperformed ClassicLRU on workload-a. Aggressive variants drain mildly-warm items; soft variants ramp too slowly in early item life. The access pattern is structurally recency-dominant.
- **Weakening or bypassing the stash**: both stash-free direct insertion and front-door ghost bypass (Cycle 4) degraded hit-rate. The stash functions as a load-bearing admission filter — circumventing it lets cold items displace warm residents before they qualify for the hot tier.
- **Promotion gating via LFU threshold** (Cycle 4): intended to filter scan pollution from the probing tier, but scan was not the actual loss source; the gated delay simply slowed the hot-item pipeline with no compensating benefit.
- **Over-extending the topology** beyond 24/8 (Cycle 3, 28/4): shrinking stash below eight slots starves cold-item re-entry and the added hot-tier pressure costs more than it returns; further topology-only probing is exhausted.

## Open questions

- **Intra-normal-tier eviction ordering**: scan resistance or segmented LRU within the hot tier has not been probed; this is the most plausible unexplored structural axis remaining for workload-a.
- **Second workload trace**: frequency-based eviction and admission-control ideas may unlock gains on a workload with heavier scan or bursty cold arrivals that are structurally blocked on workload-a's recency-dominant pattern.
- **Proportional re-entry position**: a re-entry formulation that inserts items at a position proportional to their access count (rather than full Back-insert) might extract additional value from the position lever.
- **Ghost-set tuning**: ghost-set effectiveness was near-zero at current size and insertion position (Cycle 4); whether a larger or differently-positioned ghost set could produce real gains is unresolved.

## Per-policy notes

- **ClassicLRU**: dominant policy for workload-a in every direct comparison; position encodes reuse probability better than any frequency signal tested. Should remain the eviction baseline.
- **LFU / aging-LFU**: workload-contingent at best (marginal win on frequency-skewed traces in Cycle 1); harmful in moderate-to-aggressive decay formulations. Not viable as a general improvement for workload-a; retire until a different workload motivates revisiting.
- **MRU re-entry (Back-insert)**: safe, positive, and additive with topology; marginal gain shrinks as the hot tier grows because natural dwell time increases. Has been fully compounded into the current best configuration.
- **Ghost re-admission**: mechanism is sound but mismatched to workload-a's access pattern — items do not cycle through the stash boundary frequently enough for bypass tracking to pay off. Set aside unless workload characteristics change.
- **Stash-bypass / LFU-gated promotion**: consistently harmful for workload-a; treat the stash-as-admission-filter as a hard constraint in future hypothesis design.
