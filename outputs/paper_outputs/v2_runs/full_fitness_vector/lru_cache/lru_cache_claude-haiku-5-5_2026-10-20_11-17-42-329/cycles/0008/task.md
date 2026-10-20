# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0008

## Winners

- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0008/hypotheses/H03.md

## Prior cycles digest

### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

The cohort tried a bounded credit for rescued protected heads (v001), narrowed arena links and list pointers to `u32` with a sentinel (v002), and feeding the admission sketch only from writes (v003). The two Winners are the representation change (v002) and the credit-based rescue (v001); v003 was not a Winner because its read-path saving came with a hit-rate loss.

## Patterns

- Narrowing the arena links to `u32` with a sentinel is a clean, policy-neutral win on node and struct size. It did not move `get` and may have helped `put` slightly, which is consistent with the cycle 0003 finding that link encoding is not the hot-path cost.
- A two-bit credit in the existing flags byte recovers only a small share of the protection the parent lost when it dropped the list rotation. Protection seems to depend on the head actually being moved, not only on a retained flag.
- Removing the sketch hash from `get` is the first change that cleanly lowers read cost on the admission path. The price is that read-only popularity no longer counts toward admission, and the only measured effect is a small hit-rate loss on one trace.
- Tail-latency movement in both Winners is small and comes from single runs. Credit to the mechanism should wait for a re-measure, especially where the diff does not touch the moved path (v003's `put` change).
- v002 argues that the earlier index-access slowdowns came from accessor structure rather than the casts. That is an inference from one measurement and is not yet isolated.

## Open questions

- Does the credit scheme's protection gap close with a different recency signal, or is list position the only thing that keeps the head safe?
- Does the read-path saving in the sketch change survive on read-heavy workloads, where reads carry most of the frequency signal?
- Is the `put` tail movement in v001 and v003 real, or run-to-run noise?
- Does the `u32` capacity cap in v002 (rejecting very large capacities at construction) matter for any workload the cache is expected to serve?

## Continuation

Next cycle, keep the `u32` sentinel links as the base layout and build on v001's credit only if it can add protection without list surgery on the shared path. Test the read-path sketch change on a read-heavy workload before stacking more on it, and re-measure `put` tails before crediting any mechanism. Drop further link-width experiments and any attempt to recover the lost rotation by moving the head on the hit path.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
