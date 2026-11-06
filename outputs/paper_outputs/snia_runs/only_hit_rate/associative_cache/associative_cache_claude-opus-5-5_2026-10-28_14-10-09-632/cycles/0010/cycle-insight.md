# Cycle 0010 Insight

## Cohort outcome

All three variants built on cycle 0009 v001 (GCLOCK ceiling 7, size-gated linear/halving decay, floored probation, correlated-reference filter, reset-to-0). v001 stacked the veteran grace bit. v002 turned the probation FIFO into a CLOCK ring that gives recent-only slots one extra lap. v003 widened the correlated window in large buckets only. v002 is the Winner. It gave a small hm_0 gain at the small size and held or slightly raised web_0 at both sizes, but it dipped hm_0 at the large size, so its own claim is only half confirmed. v001 got the large-size gains it predicted but cost hm_0 at the small size, landing right on the tolerance bound. v003 regressed both large cells.

## Patterns

- Every lever in this lineage works at only one bucket size. Linear decay pays off only in small mains and the veteran bit only in large halving mains. v002's lap rule helps where the window is narrow (small rings) and hurts where it is wide (large rings). The cycle 0008 finding that changes should be sized per region now holds for every lever. Each new mechanism should come with a size gate.
- The veteran bit and linear tiers collide in small mains. In large buckets, v001 matched the veteran bit's standalone gain exactly, so it stacks cleanly there. In small mains it blurs the fine tiers. Gating it to halving buckets should keep the gain and cost nothing.
- Probation retention is zero-sum. In a position-based ring, each extra lap for one slot shortens probation for the slots behind it. The net sign depends on how wide the window is compared with the ring. This is a different version of "probation depth is closed", not a way around it.
- Window width is now used up at both sizes. Widening the large-bucket window filtered web_0's real early reuse, just as cycle 0009 v002 filtered hm_0's in small buckets. About 16 slots is the most either size tolerates. The window grows its gain only up to that point.
- hm_0 at the large size is still the unexplained gap. This cycle, every change touching it was neutral or slightly negative except the veteran bit.

## Open questions

- Do v002's lap rule (gated to small buckets) and v001's veteran bit (gated to halving buckets) combine cleanly? On paper they act on disjoint sizes and disjoint regions.
- Would a narrower large-bucket window, below 16 slots, recover any of web_0's early reuse without bringing bursts back?
- Is the large-size hm_0 gap driven by long-distance reuse that any probation-side lever is structurally unable to see?

## Continuation

Make cycle 0010 v002 the parent and gate its one-lap rule to small (non-halving) buckets. Then add the veteran bit gated to halving buckets. Expect both to keep their gains at their own size with no cross-size cost. Stop widening the correlated window at any size. At most, test one narrower large-bucket width. Keep reset-to-0. Probation depth, hot-main admission gates, higher ceilings, ghost admission and non-zero promotion credit stay retired. Large-size hm_0 still needs a new kind of lever, not more probation or main-ranking tuning. Per-op-time is still unmeasured, and v002 adds a bounded probation walk per miss.
