# Cycle 0005 Insight

## Cohort outcome

All three variants built on the FIFO-probation segmented-LRU baseline and
attacked it from different angles: an LFU-min fallback for full-protection
eviction (v001), a ghost-ring accelerating re-entry of recently-evicted keys
straight into the protected segment (v002), and a cost-side fast path that
elides a no-op reorder on the hottest hit path (v003). v003 won — but only as
a hit-rate-neutral tie; both behavioral changes regressed below the parent,
so the baseline again stands hit-rate-unimproved.

## Patterns

- The two hit-rate attempts failed for the same reason that has recurred all
  run: both weakened the discriminating power of earned protection. The LFU
  fallback let `== 1` newcomers dominate the tally and killed freshly-promoted
  rising keys; the ghost ring let weakly-warranted bounce-back keys seize
  protected space without earning it. Recency at the protected end and
  earn-it-each-time probation both beat their proposed replacements.
- This sharpens the multi-cycle boundary: on this collision-bound trace,
  "recently evicted" is not a reliable proxy for "hot," and banked frequency
  counts do not separate hot from rising keys better than plain protected-LRU
  recency. The admission/promotion/eviction logic is now repeatedly confirmed
  saturated for hit-rate.
- The only non-regressing change was purely cost-side, and it landed as an
  undifferentiated tie because hit-rate is the sole fitness axis. A provably
  behavior-neutral per-op saving is invisible to selection — there is no
  dimension to reward it.

## Open questions

- Is hit-rate now effectively at its ceiling on this trace, given that
  admission, promotion, protected-eviction signal, and re-entry have all been
  ruled out across cycles?
- Without a per-op-time fitness dimension, can any cost-side refinement ever
  be selected for, or will it always tie? Is adding such an axis warranted?
- The one untested hit-rate axis remaining is looser/larger protected sizing
  and bucket geometry — does enlarging protection (rather than tightening or
  re-routing it) move the needle?

## Continuation

Keep the FIFO-probation segmented-LRU as the baseline; this cycle produced no
hit-rate gain. Stop probing protected-eviction re-signaling (LFU fallback) and
accelerated re-entry (ghost rings) — both join the long list of
protection-erosion failures. The behavioral logic looks saturated for
hit-rate. Redirect the next cycle to the two genuinely untested directions:
(a) looser/larger protected sizing and bucket-geometry interactions as the
last plausible hit-rate lever, and (b) recognizing that cost-side wins like
v003 are real but unrewardable under a hit-rate-only fitness — consider whether
a per-op-time axis is needed before spending more cohort slots on them.
