# Variant v001 Insight

## Helped

## Hurt

- **hit-rate**: Expanding the promotion budget from 3 to 5 swaps slightly regressed hit rate, suggesting that items sitting 4–5 positions from the head are not a meaningful source of missed hits on this workload — and the extra displacement of nearer items by over-promoted ones marginally increases wrong evictions.

## Takeaway

The jump from 1→3 swaps was a clear win (cycle 0005), but pushing to 5 swaps does not continue that trend: the hit-rate ticked slightly downward, indicating that diminishing returns have already set in at 3 swaps for this Zipfian workload. The 3-swap budget appears to be the current optimum; items beyond 3 hops from the head are accessed rarely enough that accelerating their convergence provides no benefit and introduces marginal interference with items that cycle through the top three positions.
