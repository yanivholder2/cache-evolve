# Variant v003 Insight

## Helped

- hit-rate: On web_0, both capacities improved a little, and the gain was larger at the small capacity. This fits the parent insight's diagnosis. Widening the resident-HIR queue from 1% to the size-scaled share (10% at 16384, 5% at 131072) gives new blocks a longer probation window, so more near-window reuse hits in Q instead of missing.

## Hurt

- hit-rate: On hm_0, both capacities slipped slightly. Each slot given to Q comes out of `lir_cap`, which shrinks the protected LIR set that held hm_0's long-reuse-distance blocks. Raising the non-resident bound from `2 * capacity` to `3 * capacity` did not recover the loss: the extra history can promote a returning block to LIR, but the LIR set it joins is now smaller.

## Takeaway

Moving the HIR share from 1% to 5–10% mostly traded hm_0 hit-rate for web_0 hit-rate in near-equal amounts. The net effect on the summary is negligible. It did not close web_0's gap while keeping hm_0's lead, as H03 predicted, which suggests LIRS hit-rate on these traces is fairly flat across this range of HIR shares. The HIR share is a trace-dependent dial, not a lever with a clear best setting. The two parameter changes were made together, so this run cannot say how much of the shift came from the HIR share and how much from the 3× history bound. This variant's source overlay and other eval artifacts could not be listed with the tools available here. This reading therefore rests on H03's stated changes and the two fitness vectors, not on a direct code diff.
