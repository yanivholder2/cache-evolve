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
