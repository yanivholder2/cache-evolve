# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0009

## Winners

- v003
- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

The cohort tried three different levers: a smaller protected segment (v001), a cheaper hasher for the key map (v002), and a bounded ghost ring of recently evicted keys that promotes returning keys straight to protected (v003). The clearest Winner is the hasher swap (v002), which cut per-operation cost with no policy change. v001 and v003 are listed as Winners, but both insights record a hit-rate or cost trade-off that the Winner label does not capture.

## Patterns

- Hashing on the key map is a real hot-path cost, and it is separate from the admission sketch. Swapping to a zero-state hasher lowered `get` and `put` cost without touching policy, and it also shrank the map struct. This is the same kind of cost that cycle 0004 found in the sketch hash.
- Shrinking the protected segment hurt hit-rate on the scan-heavy trace. The 4/5 share appears to be doing real work, so further reductions in protected share are unlikely to pay off.
- The ghost ring gained scan resistance, but the gain came with added work on the miss path. Its per-miss scan over the ring and its extra hashing both land in `put`, and its heap storage is not counted by the struct-size metric.
- The v001 per-operation movement is not explained by its diff, which leaves the code paths unchanged. Treat the latency delta as unconfirmed until it is re-measured.
- The v003 insight was written from the hypothesis and fitness vectors because its source could not be read. Its code-level claims are inferred and should be checked against the diff before they are credited.
- A fast hasher with a raw finish leaves the low bucket-index bits least mixed. The v002 gain may depend on the key distribution, and it has been shown only on integer keys.

## Open questions

- Does the zero-state hasher keep its speedup and its bucket spread on non-sequential integer keys and on string keys, where the byte path is different?
- Does the protected-share sensitivity seen in v001 hold at other capacities and traces, or is it specific to workload-a at 2048 entries?
- Can the ghost ring's lookup be made cheap enough to keep its scan-resistance gain, for example with a hash-indexed set in place of a linear scan?
- Are the `put` tail movements in v001 and v003 real, or run-to-run noise?

## Continuation

Next cycle, keep the zero-state map hasher as the base and test its distribution on string and non-sequential integer keys before stacking more on it. Pursue the ghost-ring idea only if its lookup is bounded, and report its heap size alongside struct size. Drop further protected-share reductions, since v001 shows the current ratio is not oversized, and re-measure v001's per-operation deltas before crediting them.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
