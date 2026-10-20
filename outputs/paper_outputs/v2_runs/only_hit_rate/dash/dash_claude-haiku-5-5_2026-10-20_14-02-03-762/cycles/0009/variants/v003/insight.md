# Variant v003 Insight

## Helped

- hit-rate: Per the hypothesis, the change moves 128 slots of capacity from the stash back to the probing tier while keeping total capacity fixed. On LIRS workload-a at size 2048, hit-rate rose over the parent v002, which is the direction the hypothesis predicted for reversing the cycle 0008 shrink of probing.

## Hurt

## Takeaway

Giving the probing tier 128 slots at the expense of the stash improved hit-rate on this trace, which suggests probing residency, not stash capacity, is the binding resource at size 2048. This is one trace at one size, and the gain is small. I could not read the overlay sources in this pass (the shell and search tools were unavailable and the manifest was not at the expected path), so I have not confirmed that the constants changed exactly as the hypothesis states or that the stash-full retry and derived sizes were kept consistent. Treat this as a reading of the fitness result against the stated change, not of the implementation.
