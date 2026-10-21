# Variant v002 Insight

## Helped

- None. hit-rate is the only reported dimension, and it did not improve.

## Hurt

- hit-rate: The hypothesis made the victim scan non-mutating and moved aging to a periodic global right-shift every capacity/2 evictions. The parent's per-scan decrement gave fast, local aging in the places the evictor actually looks, and the coarse global halving did not replace it. Counters now stay high longer between decay passes, and a minimum-counter fallback picks the victim when no zero is found. That weakens the evictor's ability to separate idle entries from recently hot ones, and hit-rate on workload-a fell below the parent.

## Takeaway

On this trace, the scan-position bias the hypothesis wanted to remove was not the problem. Decrementing counters at scan time acted as a useful idleness signal, and replacing it with eviction-count-driven halving (a TinyLFU-style reset applied to residents) lost more than it gained. Any further change to counter aging should keep some local, scan-time pressure instead of relying only on a global timer. This is a single trace and size, and I did not inspect the overlay source, so this rests on the hypothesis text and the two fitness vectors.
