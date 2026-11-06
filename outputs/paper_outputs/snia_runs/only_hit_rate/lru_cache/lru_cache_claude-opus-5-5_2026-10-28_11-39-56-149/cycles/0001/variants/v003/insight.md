# Variant v003 Insight

## Helped

- hit-rate (hm_0, both sizes): hm_0's frequency skew is what the TinyLFU gate rewards. When the cache is full, a long-tail block can no longer push out the LRU victim unless the sketch has seen it more often, so the hot set stays resident at both 16K and 128K. The gain shows up at both sizes, not mainly at the larger one as the hypothesis predicted.

## Hurt

- hit-rate (web_0, mainly the small size): the admission rule is strict-greater, and the JNI path calls `get` before `put`, so a first-seen candidate (sketch count of about 1) ties with any victim that was touched once and gets rejected. Blocks that come back soon after their first access, which is the recency reuse plain LRU caught, now miss on their second access too. A small cache depends most on that short-range reuse, so the 16K web_0 point drops hard. At 128K the drop is small.
- hit-rate (web_0): a rejected `put` leaves the victim at `head` without promoting it, so the same LRU entry screens every following candidate. A moderately warm victim can freeze the cache's contents until the sketch ages at `10 * capacity`. The 1/128 tie-break only applies when the candidate's count is at least 6, so it does nothing for the cold-start ties that cause this.
- metadata-size / per-op-time: v003's fitness vector has no values for these. The sketch (`capacity` u64 words, plus the `rng` and sketch fields in `LruCache`) and the extra `DefaultHasher` pass on every `get` and full-cache `put` almost certainly cost something, but this evaluation can't confirm it.

## Takeaway

Adding a TinyLFU-style admission filter in front of plain LRU, with no admission window, changes which traces the cache serves well rather than raising hit-rate overall. It helps the frequency-skewed trace (hm_0) and hurts the recency-driven one (web_0), most at the small cache size, so the overall hit-rate summary dips slightly. The failure comes from candidates that tie at a count of 1 and are rejected, and from a single unpromoted victim acting as gatekeeper. The next step is therefore a small admission-free recency window (W-TinyLFU style) or admitting on ties at low counts, not a better-tuned sketch.
