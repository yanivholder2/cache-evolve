# Variant v003 Insight

## Helped

- hit-rate: Replacing the small-queue's in-place `freq` bump with `remove` + `push_front` keeps a short-term-reused key from drifting to the tail and being FIFO-evicted before its second hit lands, so a few of those reuses now resolve as small-queue hits instead of misses.

## Hurt

## Takeaway

Making the thin admission queue recency-aware barely moves hit-rate: at ~4% capacity (`capacity / 24`) the small queue holds so few entries that move-to-front only changes an outcome in the narrow case where a reused key sits at the very tail with a refill arriving in the same tiny window. It also works against itself — delaying a hot key's trip to the tail delays the `freq > 1` promotion in `evict_small` that would have moved it into main — leaving the two effects to roughly cancel. The small queue's ordering is too short a lever to matter here; the binding constraint stays in main and ghost capacity, not intra-small recency.
