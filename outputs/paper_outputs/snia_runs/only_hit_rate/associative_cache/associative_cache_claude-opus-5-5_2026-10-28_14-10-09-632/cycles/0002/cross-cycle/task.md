# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

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


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Two variants extended cycle 0001's cold-insertion winner (v001). v001 added an absolute probation floor. v002 added BIP-style occasional MRU insertion. The third variant, v003, upgraded CLOCK to a 2-bit GCLOCK counter. The Winner is v002. Its overall gain was small and came with a split by trace: it helped hm_0, mostly at the small size, and hurt web_0.

## Patterns

- Both v001-lineage variants went after the same small-bucket shortfall, and both recovered some of it. hm_0 benefited more than web_0 each time, so hm_0 has meaningful reuse beyond the original probation window at 16384.
- Gains at the small size don't carry over to the large size. Lengthening probation left 1024-slot buckets unchanged. Sending items to MRU hurt them on web_0, where cold traffic is mostly scan-like and already drained well. A single fixed rule can't serve both trace types.
- The frequency-aware CLOCK line (v003) improved on its parent in every trace×size cell, especially on hm_0 at the large size, and it does no element shifting. It was not the Winner, but it is now a credible second lineage. Its advantage grows with bucket size, where the v001 line has the least room left to improve.
- For the second cycle in a row, no variant measured per-op-time or metadata-size. Every cost claim is still untested: mid-vector inserts, GCLOCK's multi-rotation sweeps, and the full-bucket candidate scan.

## Open questions

- Would adaptive insertion fix v002's trace split? For example, DIP-style set dueling between LIP and BIP, or an MRU-insert rate that scales with bucket size.
- Do the absolute floor (v001) and occasional MRU insert (v002) stack, or do they recover the same reuse?
- Does GCLOCK's hit-rate gain hold once its worst-case sweep cost is measured? And can it take a cold-insertion analogue, such as starting new items below already-referenced ones?

## Continuation

Build on v002, but replace its fixed period with an adaptive policy (set dueling, or a rate tied to bucket size and trace behavior) so web_0 stops losing hits. Try combining it with v001's probation floor. Keep the GCLOCK lineage (cycle 0002 v003) going as a parallel branch, since it scales well to large buckets. Make per-op-time measurement a requirement, not an option. Drop further tuning of the BIP period alone.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
