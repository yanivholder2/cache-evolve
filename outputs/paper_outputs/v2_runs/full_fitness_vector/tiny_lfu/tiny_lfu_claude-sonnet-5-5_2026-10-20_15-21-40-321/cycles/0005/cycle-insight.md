# Cycle 0005 Insight

## Cohort outcome

Three footprint and speed changes were tried: reverting the 4x wider sketch on the cycle-0004 parent (v001), dropping a cached field from Node on the other cycle-0004 parent (v002), and a single-hash shift-derived sketch increment with a saturation early-exit, rooted at baseline (v003). All three were kept as Winners, but only v003 gave a plausible real gain (cheaper put and get medians, hit-rate level). v001 and v002 gave no real improvement.

## Patterns

- Sketch width is a weak lever in both directions. Narrowing saves hidden heap and costs a hair of hit-rate. The predicted locality speedup did not appear, and latency got slightly worse.
- v001's slowdown, and v002's get "gain" from an untouched hot path, are best read as noise. Together they cast doubt on the parent's earlier per-op improvement. This is the third cycle where small latency deltas looked unreliable.
- Removing a cached field from Node did not shrink it, because padding or alignment absorbed the saving. Moving the hash recompute onto the eviction path then taxed put tail latency. Node size has little room left to move this way.
- Cheaper sketch increment work (one hash, shift-derived indices, saturation early-exit) is the only speed idea this cycle that cleanly left hit-rate alone. It confirms the cycle-0002 finding that sketch cost is a main per-op term, now on an independent baseline lineage.
- Hit-rate stayed flat across the cohort. No variant touched a mechanism that could change it.
- Two insights were written without a source diff, so some attributions are inferred. v003 cannot separate the early-exit from the cheaper hashing.

## Open questions

- Does v003's increment speedup stack on the cycle-0004 v001 parent (cheap map hasher, MRU skip, compact links), or is it only visible against the baseline?
- Which part of v003 matters: the saturation early-exit or the single-hash derivation?
- Does any footprint lever remain once Node alignment is the floor, short of changing counter or key types?
- Is there any hit-rate lever left on workload-a?

## Continuation

Use v003's single-hash, shift-derived increment as the speed component and merge it onto the cycle-0004 v001 lineage; test the early-exit and the hash derivation separately. Drop sketch width tuning, and drop Node field removal unless it is paired with a layout change that crosses an alignment boundary. Repeat latency runs before crediting any small delta. Hit-rate gains still need a genuinely new mechanism.
