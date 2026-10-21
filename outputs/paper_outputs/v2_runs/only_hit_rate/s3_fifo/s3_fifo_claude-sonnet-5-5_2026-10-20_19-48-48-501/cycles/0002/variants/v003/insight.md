# Variant v003 Insight

## Helped

- (none): No dimension improved; the only measured dimension, hit-rate, did not beat the parent.

## Hurt

- hit-rate: Replacing `freq -= 1` with `freq /= 2` in `evict_main` shortens the reinsertion life of a freq-3 entry (3→1→0 instead of 3→2→1→0) and drops freq 2 straight to 1. Entries that were still going to be reused get evicted from main sooner, and this costs a little hit-rate against v002.

## Takeaway

Making main-queue aging more aggressive does not help on this workload. Main residents that have earned a high frequency tend to be reused, so the extra passes under linear decrement were worth keeping. The change is small and slightly negative, which suggests the existing second-chance depth was already about right, and tuning it is a weak lever compared with changing admission or promotion.
