# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Per H01, the change moves a further 128 slots from the stash to the probing tier at fixed total capacity, on top of v003's split. Hit-rate on LIRS workload-a at size 2048 fell below the parent v003, so the extra probing residency did not pay for the stash capacity it cost. The stash-hit path (re-reference promotion gate and MRU landing) was left untouched, so the loss most plausibly comes from the stash shrinking, not from any change to hit handling. This is an inference from the hypothesis; I could not read the overlay sources to confirm the constants and derived sizes were changed as stated.

## Takeaway

The v003 gain from moving 128 slots into probing does not extend to 256 slots: the probing benefit saturates somewhere between those two points, and beyond it the stash capacity lost is the bigger cost. This is one trace at one size, so the result only locates the optimum on this workload. The split should stay near v003's 128-slot shift unless a finer sweep between 128 and 256 shows a different peak.
