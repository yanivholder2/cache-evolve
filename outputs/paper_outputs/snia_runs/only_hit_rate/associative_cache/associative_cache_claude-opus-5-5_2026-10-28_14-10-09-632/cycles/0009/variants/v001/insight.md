# Variant v001 Insight

## Helped

- hit-rate (hm_0 @ 16384): The hypothesis's main claim holds, and by more than expected. With `halving = false` for the 64-slot main region in 128-slot buckets, a slot's `steps` is now `recent + counter` rather than `recent + bit_length(counter)`. Counters 4–7 used to fall into a single halving tier. Now each value is its own tier, so the sweep can tell a moderately reused block from a heavily reused one. The gain over v003 is about twice what cycle 0008 v001's linear decay gained over its own parent. So the two changes do more than add up in small buckets. One plausible reason: v003's filter already keeps burst-only blocks out of main, so the finer tiers spend their resolution on blocks that really are reused, not on noise from burst promotions.
- hit-rate (web_0 @ 16384): This was v003's weak cell, where its 4-slot correlated-reference window caught almost nothing. It now clears the refutation bar. This is the cell where linear decay was meant to make up for the narrow window, and it did, though the gain is smaller than on hm_0.

## Hurt

- (none) Both 131072 cells are bit-identical to v003. This confirms that the `max_size - small_len >= HALVING_MIN_MAIN` gate leaves 1024-slot buckets (main = 768) on the original halving rule, so v003's large-bucket filtering is untouched.

## Takeaway

The decay rule should depend on the size of the main region. A 64-slot main gains from linear decrement's eight fine tiers, while the 768-slot main keeps halving's 4-rotation bound with no loss. The refutation clause worried that the correlated-reference filter and linear decay would interact badly in small buckets. That did not happen: the interaction on hm_0 was mildly positive. Small buckets are now the cells that respond to tuning. The obvious next step is v003's suggestion to widen the 4-slot correlated window in 128-slot buckets, now on top of linear decay.
