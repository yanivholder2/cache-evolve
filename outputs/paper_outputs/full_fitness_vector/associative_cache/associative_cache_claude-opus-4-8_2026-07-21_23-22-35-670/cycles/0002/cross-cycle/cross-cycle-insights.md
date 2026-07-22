## Recurring wins

- Trimming dead metadata off the eval path keeps paying off cycle after
  cycle: it stays monotonic and cheap, shedding fields carried only by
  off-eval-path policies with no hit-rate cost.
- Permanent, binary reuse-protection is a durable hit-rate win on this
  trace and composes cleanly with layout slimming — the two proven
  mechanisms touch disjoint code and stack additively at no extra byte cost.
- Structural, well-motivated changes beat speculative geometry tuning;
  the clearest gains come from removing weight rather than adding capacity.

## Recurring failures

- Buying hit rate by lengthening a linear scan (wider buckets, scan-based
  eviction protection) is consistently a bad trade: every such gain is
  paid back in per-op time, since bucket/eviction scan cost is the
  dominant hidden price in this cache.
- Plain associativity increases don't earn their keep at this working-set
  size — conflict-miss headroom is thin, so raising it barely moves hit rate.
- Weakening protection (second-chance / CLOCK-style aging) sheds immunity
  too fast and collapses back toward plain LRU; refinements should preserve
  permanent protection, not dilute it.

## Open questions

- Where is the metadata floor — how much further can fields shrink before
  colliding with something the eval path actually reads?
- Can scan resistance be gained without adding a linear scan, e.g. an
  O(1)-maintained eviction candidate rather than a front-to-back re-scan?
- Is there any hit-rate headroom left beyond the composed reuse-protection
  win, or is the trace near its ceiling for these mechanisms?
- The recurring upward p99 tail drift under flat p50 on metadata cuts —
  structural effect or persistent measurement noise?

## Per-policy notes

- ClassicLRU / eval path: never touches timestamp-style metadata, which is
  why those fields are safe to peel off; the eviction pick is the cost
  center whenever it becomes a scan.
- Reuse-protection policy: permanent binary protection is the right fit for
  this trace — cold-but-once-hot "squatters" are not what caps hit rate,
  so aging-based variants underperform.
- Higher-associativity / wider-bucket geometry: unfavorable here; extra
  buckets add scan cost without unlocking meaningful conflict-miss headroom.
