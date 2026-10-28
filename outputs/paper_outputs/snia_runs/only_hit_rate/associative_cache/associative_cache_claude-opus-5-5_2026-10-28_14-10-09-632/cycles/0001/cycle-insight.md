# Cycle 0001 Insight

## Cohort outcome

All three variants changed ClassicLRU's per-bucket ordering. One changed where new items are inserted (v001, LIP-style insertion at len/4). One limited how far a hit is promoted (v002). One replaced the ordering with CLOCK second-chance (v003). The Winner, v001, is the only variant that changes where *new* items land. It raised hit-rate on every trace×size cell.

## Patterns

- What decides hit-rate on the MSR traces is how a new item ranks against a re-referenced item. v001 put new items below reused ones and won. v002 left new items at MRU but weakened promotion, so new items outranked reused ones, and hit-rate dropped (most on hm_0 at the larger size).
- Scan resistance matters more in the larger 1024-slot buckets. In the 128-slot buckets, v001's fixed-fraction probation window is short, so some real reuse is cut off.
- Coarser recency tracking (CLOCK) did not hurt hit-rate. It was level or slightly better, especially at the small size. Exact LRU order does not appear to be what drives hit-rate here.
- None of the variants measured per-op-time or metadata-size. Any latency claim (H03's main goal, H02's side claim, v001's cost of inserting mid-vector) is still untested.

## Open questions

- Does CLOCK (v003) actually lower get/put latency, now that we know it costs no hit-rate?
- Would a deeper insertion point, or a probation depth that scales with bucket size, recover the lost hits at 16384 without losing the gain at 131072?
- Can the two ideas be combined, i.e. CLOCK with a cold insertion position? Does graded promotion become useful once inserts land low?

## Continuation

Build on v001. Tune `INSERT_FRACTION_DIV` toward a deeper insertion point, or make probation depth scale with bucket size, focusing on the 16384 size. Make sure per-op-time is measured, and re-test CLOCK for latency, possibly combined with cold insertion. Drop bounded promotion on its own while new items still enter at MRU.
