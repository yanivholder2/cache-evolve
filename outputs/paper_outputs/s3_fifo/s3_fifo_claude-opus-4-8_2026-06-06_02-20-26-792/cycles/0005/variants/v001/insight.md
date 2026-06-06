# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Replacing the parent's `pop_back`+`push_front` survivor rotation with a tail-to-head in-place decrement loop now decrements `freq` on *every* survivor between the tail and the first `freq == 0` entry, burning down the second-chance counters of many genuinely hot keys in a single eviction and stripping their protection, so more reused keys get evicted on subsequent calls.

## Takeaway

Preserving `get()`'s recency order through eviction backfired: the parent's `push_front` only decremented the few tail entries it actually popped, whereas walking inward and decrementing each survivor along the way over-erodes the `freq` signal across the queue. The recency-vs-frequency tension matters more than head-MRU purity here — the tail was not reliably the coldest residents (`evict_small` and `put` also feed the head), so the assumption that the scan reaches only cold keys does not hold, and the move-to-front discipline from v003 is better left undisturbed by eviction.
