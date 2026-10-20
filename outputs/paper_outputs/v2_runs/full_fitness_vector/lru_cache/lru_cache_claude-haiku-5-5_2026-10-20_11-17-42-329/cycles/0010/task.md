# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0010

## Winners

- v003
- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0010/hypotheses/H03.md

## Prior cycles digest

### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

The cohort tried three levers: extra mixing in the hasher's finish step (v001), promoting returning ghost keys straight into the protected segment (v002), and packing the flags into the top bits of the link field (v003). None of them improved per-operation cost while keeping policy intact. v003 bought a real node-size reduction at a latency cost, and the insights record no clear helped dimension for v001. The Winner labels for v001 and v003 are not supported by their insights' Helped sections, so the fitness vectors should be checked before crediting them.

## Patterns

- Extra finish-step mixing gave no spread benefit on integer keys and added per-lookup cost on both `get` and `put`. Together with the cycle 0006 hasher win, the gain on the key-map path comes from cheaper hashing, not from more mixing.
- Ghost-driven admission into protected lost hit-rate on the workload tested. Recency of re-reference did not pick out keys worth protecting, and the miss path picked up extra cost. The ghost idea has weakened since cycle 0006.
- Masking flag bits out of the link field removed the padding on the node and was policy-neutral, but every index read and write paid the mask. The saving is a size-for-latency trade, not a free change.
- Across the three variants, node or struct growth and per-operation slowdowns appeared without matching hit-rate gains. Changes that touch the shared index helpers have been the most reliable source of per-operation regressions.
- The v002 node growth is not explained by its hypothesis text and has not been checked against the diff. Its per-operation attribution is also inferred.

## Open questions

- Is the v002 node growth a side effect of the diff rather than the hypothesis, and does moving the ghost set out of the node path remove it?
- Can the flag-bit mask be kept off the `get` path, for example by folding it into a single index helper or keeping flags in a parallel array, so that the node-size gain survives?
- Does the ghost's re-reference signal carry value on workloads other than workload-a at 2048?
- Do the fitness vectors actually show a Winner-level gain for v001 and v003, or were they labelled Winners on size alone?

## Continuation

Next cycle, keep the zero-state map hasher as the base and do not add further finish-step mixing. Drop ghost-to-protected promotion; if the ghost idea is pursued again, keep ghost hits probationary and measure the ghost's hit rate before changing admission. Pursue the flag packing only if the mask is removed from the read path, otherwise prefer the byte-sized flags. Re-measure per-operation deltas before crediting any of these changes.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

The cohort tried three levers: confining the packed-flag mask to one inlined index helper with a raw-OR REFERENCED set on hits (v001), collapsing the `put` miss path onto a single key-map lookup (v002), and admitting ghost-ring hits at probation instead of protected (v003). v001 is the only variant whose insight records an improved dimension, so it is the credible Winner. v002 carries a Winner label but its insight records no improvement and a worse `put` tail, so the label is not supported. v003 lost hit rate and is not a Winner.

## Patterns

- Consolidating the masked index reads (v001) recovered per-operation cost on the packed layout without changing policy or metadata size. The mask still runs on every read, so this addresses the call-site cost from cycle 0007 but not the mask itself.
- The single-lookup rewrite of the `put` miss path (v002) did not reduce `put` cost. The forced remove-then-insert ordering appears to absorb the saved hash, so the second key hash is not the dominant `put` cost.
- Ghost-ring admission has now lost hit rate in both directions: to protected in cycle 0007 and to probation here, on workload-a at 2048. The ghost idea has no working admission target on that trace.
- `put` tail movement appears again across variants (v002 and v003 here, and several earlier cycles), while `get` tails moved without a `get` change in v002. This pattern points to measurement noise or a shared layout effect more than to any single diff.
- Two of the three insights rest on hypothesis text and fitness vectors because the source diffs could not be inspected. Their code-level claims are inferred.

## Open questions

- Is v001's gain carried by the raw-OR REFERENCED set on hits, by the helper consolidation, or by both?
- Is v002's `put` tail regression real, given that its `get` tail also moved without a `get` change?
- Does the ghost-ring hit-rate loss hold on workloads other than workload-a at 2048?
- Are the recurring `put` tail movements a run-to-run artifact of the measurement setup?

## Continuation

Next cycle, build on v001's single-helper index access as the packed-flags base, and test whether the raw-OR REFERENCED set on hits is what drives its gain. Drop the ghost ring as an admission lever on workload-a, since neither promotion target held hit rate. Do not pursue the single-lookup `put` rewrite, and re-measure `put` tails before crediting or blaming any further `put`-path change.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

The cohort tried three levers: a larger protected segment (v001), skipping the reference-bit write when it is already set (v002), and replacing the two-segment SLRU with a single CLOCK ring of 2-bit counters (v003). Only v001 records a dimension that improved (hit rate on workload-a at 2048), and it paid for that with `put` tail latency. v002 records no improvement, and v003 gained metadata size at a hit-rate and latency cost. The Winner labels for v002 and v003 are not supported by their insights' Helped sections.

## Patterns

- The protected share is a real hit-rate lever in this SLRU. Growing it moved reused keys out of reach of the probationary scan traffic, but the probationary segment got thin and the `put` miss path did more demotion work. Cycle 0006 found that shrinking the share hurt hit rate, so the current range looks like a trade-off rather than a clear optimum.
- Guarding the reference-bit write on hits (v002) cost more than the store it skipped. The unconditional OR on the packed word is cheaper than the extra load and branch, so the hit path should stay as it is.
- Replacing the segmented structure with CLOCK (v003) removed the protection that kept rescued keys safe, and it also moved the per-slot state into a separate allocation. The metadata saving is smaller once that allocation is counted, and the per-hit increment and the multi-slot sweep both show up as latency.
- The v003 insight's claim that the hit-rate cost was small was an underestimate. The segmented protection appears to matter more than the node-size argument suggested.
- Across the three variants, the `put` tail moved in the variants that touched capacity handling or the miss path (v001 and v003). This matches the recurring `put` tail pattern from earlier cycles and is still not separated from run-to-run noise.
- The v001 and v003 per-operation causes are inferred from the code and the hypotheses, not profiled. Their code-level claims should be checked against the diffs before they are credited.

## Open questions

- Does the protected-share gain on workload-a at 2048 hold at other capacities and traces, and can it be had without the `put` tail cost?
- Is the v001 `put` tail regression real, or is it the same run-to-run artifact seen in earlier cycles?
- Would a CLOCK variant that keeps some form of protection recover the hit rate while keeping the smaller node?
- Is the `refs` allocation cost in v003 large enough to offset the node-size saving across the full footprint?

## Continuation

Next cycle, build on v001's larger protected share as the hit-rate candidate, and test it at a second capacity or trace before stacking more on it. Measure its `put` tail against the baseline before accepting the change. Drop the reference-bit guard from v002 and the plain CLOCK rewrite from v003, since neither kept the protection or the hit path that the parent relied on.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
