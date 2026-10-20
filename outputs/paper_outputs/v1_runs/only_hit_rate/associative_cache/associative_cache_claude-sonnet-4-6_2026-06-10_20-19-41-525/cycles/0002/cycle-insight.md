# Cycle 0002 Insight

## Cohort outcome

The cohort probed three anti-scan-pollution strategies — further bucket widening,
cold-first eviction, and mid-point insertion — on top of cycle 0001's geometry win.
The cold-first eviction variant (v002) was the clear winner, delivering a large
hit-rate gain built on the wider-bucket foundation; the other two produced smaller
or marginal returns without being selected.

## Patterns

- **Geometry widening has reached diminishing returns.** Pushing the bucket factor
  from 4 to 16 yielded only a fraction of the gain the first widening step produced.
  The LIRS workload-a hot set largely fits within 64-slot buckets; further geometric
  expansion is unlikely to pay off.
- **Cold-first eviction is the strongest lever found so far.** Preferring
  counter==0 items as eviction candidates shields any item that has received even a
  single GET hit from being displaced by scan insertions. The wider buckets from
  cycle 0001 are a necessary enabler: they provide enough resident cold items to
  absorb scan traffic without immediately exhausting the cold pool and collapsing
  back to plain LRU.
- **Mid-point insertion addresses the same root cause from the insertion side.**
  Landing new items at the bucket midpoint rather than the MRU back also improved
  hit-rate without hurting any measured dimension, but produced a smaller gain than
  cold-first eviction and carries an unmeasured O(n) insertion-latency cost.
- **Both winning angles converge on scan-pollution as the dominant bottleneck.**
  v002 and v003 attacked the problem from eviction and insertion respectively;
  neither was combined with the other this cycle.

## Open questions

- Can cold-first eviction (v002) and mid-point insertion (v003) be stacked, and do
  they compound or merely overlap in their benefit?
- Does v002's linear scan across bucket slots to find a cold item add meaningful
  per-op latency at scale, particularly in wide buckets?
- As the cache warms up over a long trace, does the cold pool thin out — causing
  the fallback to plain LRU to trigger more often — and if so, does the win degrade?
- Do the gains from v002 and v003 generalise beyond the LIRS workload-a trace?

## Continuation

The cold-first eviction approach (v002) is the strongest lead: it produced a large
hit-rate gain, has a clear correctness story, and the geometry axis it depends on is
already confirmed. Next cycle should explore stacking mid-point insertion on top of
v002, probe whether a frequency threshold greater than 1 (e.g., counter ≥ 2) improves
separation further, and add a `per-op-time` evaluation pass to v002 to measure the
scan cost before widening buckets further. Drop pure geometry pushing — that axis
is saturated.
