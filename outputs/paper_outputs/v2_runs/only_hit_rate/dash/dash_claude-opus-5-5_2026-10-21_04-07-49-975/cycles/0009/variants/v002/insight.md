# Variant v002 Insight

> Note: I could not open the overlay source in this session. Directory listing tools were unavailable, and the source and manifest paths I guessed did not exist. The mechanism below comes from H02's stated change: halve the ghost table's entry count, narrow the epoch tag if possible, and keep the epoch length and the age ≤ 4 cutoff the same. I did not check the diff itself.

## Helped

- hit-rate: Halving the direct-mapped ghost table did not cost hit-rate; it moved slightly upward, which is within noise. The extra overwrites a smaller table causes seem to land mostly on entries older than the 4-epoch gate. Those entries could not have changed placement anyway. So the useful ghost evidence was already contained in about half a window of fingerprints.

## Hurt

## Takeaway

The recency gate, not the table size, sets how much ghost history matters. Once ghost hits are limited to age ≤ 4 epochs, about half a cache's worth of fingerprints holds all the evidence that counts. Collisions in the smaller direct-mapped table did not measurably push out recent entries, so a set-associative ghost is not needed at this size. The claimed metadata-size saving is not confirmed by this evaluation: the fitness vector reports only hit-rate, from one LIRS trace at one cache size. The next step is to measure the footprint directly, or to try shrinking the ghost further (e.g. to a quarter) to find where collision loss starts.
