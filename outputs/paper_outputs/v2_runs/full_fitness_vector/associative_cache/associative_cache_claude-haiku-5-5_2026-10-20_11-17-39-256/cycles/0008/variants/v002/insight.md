# Variant v002 Insight

## Helped

- per-op-time: Swapping the bucket index for a Fibonacci multiplicative hash changes which keys share a bucket on a parent whose victim search is already exact-minimum, and the get and put medians both dropped, so the bucket spread is a real (if modest) timing lever on this parent.

## Hurt

- hit-rate: Re-bucketing the keys shifts which ones compete for the same ways, and the single recorded hit-rate on workload-a came in slightly below the parent; the drop is small enough that it may be run-to-run noise, but it is the only movement against the frequency parent.

## Takeaway

The hash change alone recovers part of the timing gain that cycle 0007 claimed (get p50 fell, but not down to the ~51 ns that was reported there), which suggests the earlier speedup was partly real and partly noise. Bucket placement is worth roughly a few nanoseconds per op on this parent and does not, by itself, change hit-rate in a meaningful way. Confirming the timing claim needs the three-run spread the hypothesis asked for, which is not in this variant's eval output.
