# Variant v001 Insight

## Helped

- metadata-size: Unchanged from the parent, as the hypothesis required. The hash-derived 1-in-32 MRU rule needs no per-slot state, so the insertion policy stayed metadata-free. This is a constraint met, not a gain.

## Hurt

- hit-rate: Slightly below the parent. Moving the insertion point from about len/4 to the LRU-most slot, with a rare MRU insert, gave no extra scan resistance on workload-a. A new key sitting in the last slot is evicted by the next insert into that bucket, before a second reference can promote it. This looks like a marginal loss, possibly within noise, and not a collapse.
- per-op-time: Both get and put latency rose, put by far the most. The extra branch on the key hash and the different shift pattern for the LRU-end insert are plausible causes. I could not inspect the source diff, so this is unconfirmed.

## Takeaway

len/4 appears to be a better balance than the LRU-most slot. Fully LRU-end insertion starves new working-set entries of the time they need to prove reuse, and the 1-in-32 MRU escape is too rare to compensate. Pushing the insertion point further toward LRU has diminishing or negative returns here, and the added hash-conditional logic cost latency without improving hit rate.
