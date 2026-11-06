# Cycle 0007 Insight

## Cohort outcome

Two variants targeted latency on the cycle-0004 v002 base without changing policy. v001 split eviction and promotion into cold functions and forced the hot helpers inline. v003 removed the victim before inserting the new key on stock hashbrown. v002 turned the cycle-0006 v001 reprieve on at 131K with a capacity/2 ghost. The Winners are v001 and v002. v002 is the clean gain: web_0 at 131K improved more than hm_0 dipped, and 16K was unchanged. v001 won only on the get tail, and its get p50 got worse.

## Patterns

- Two unrelated changes both "fixed" cycle 4's get-p99 rise. One was a pure layout split (v001), the other a change in eviction order (v003). Both also made get p50 worse. When different causes produce the same effect, noise or broad layout sensitivity is a more likely explanation than any one mechanism. Neither claimed cause has been established.
- Get p50 still resists every lever. Forced inlining made it worse, and v003 changed no get-path instruction and it still rose. Under a p50 summary, trading the median for the tail has now lost in two variants.
- Put cost tracks hash count, now for the fifth cycle. v003's extra SipHash on every evicting put undid the single-lookup saving.
- The ghost window works as a reuse-distance filter. Cycle 6 showed capacity/8 is too short for web_0 at 131K. capacity/2 catches most of web_0's useful re-references and lets in only a few of hm_0's loop keys. The best width probably lies between the two. With the zero-seeded Fx ghost, hit-rate deltas can now be trusted.
- `metadata-size` does not count the heap ghost table. At 131K the ghost is a large hidden memory cost.
- Two of the three variant insights were written without access to the overlay source. Their explanations assume the hypothesis was implemented as written.

## Open questions

- Is capacity/4 (or stricter reprieves at large sizes) a better point on the web_0-gain/hm_0-leak trade-off than capacity/2?
- Does v001's `#[cold]` split without the `#[inline(always)]` hints keep the tail fix without the get-p50 loss?
- Is any of this cycle's get-p50 or get-p99 movement real? None of it has been confirmed by repeated paired runs.

## Continuation

Use cycle-0007 v002 (capacity/2 ghost at 131K) as the new base. Tune the large-size ghost width toward capacity/4 or tighten large-size reprieves. Consider counting the ghost table in metadata. Drop forced `#[inline(always)]` and drop remove-before-insert using a second hash. Retry an eviction-order change only through a hash-preserving path (`HashTable`/raw entry). Stop chasing get p99 or p50 until paired repeated runs show the effect is larger than noise.
