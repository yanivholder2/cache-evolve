# Cycle 0007 Insight

## Cohort outcome

This cycle drilled entirely into cycle 0006's one fresh seam — the stash
relocation hop — testing three ways to deepen it: grafting the single hop
onto the proven protected-tier winner (v001), balancing admission across both
homes via power-of-two-choices (v002), and extending the single hop into a
bounded multi-hop cuckoo cascade (v003). The declared winner was v001, but its
gain was noise-level; v003's cascade was the only clean, regression-free
positive and is the meaningful result.

## Patterns

- The stash dwell-time seam responds to *depth*, not breadth. A bounded
  multi-hop cascade — evicting an LRU resident at each full bucket and chasing
  free space a few buckets deep — converts would-be drops into survivors and
  lifts hit-rate with capacity conserved. The single hop left this headroom on
  the table precisely because the lone alternate home was usually full too.
- Pre-emptive load-balancing is the wrong frame for the same seam. Spreading
  cold keys evenly across both homes at admission time scatters keys away from
  the primary-first locality the get path expects, deferring rather than
  preventing drops, and regressed. Reactive rescue of genuine collision victims
  beats proactive evenness — the value is dwell time, not balanced occupancy.
- The late slivers do not compound. Stacking the single-hop relocation onto the
  protected-tier/two-speed-promotion winner came back flat: the two seams chase
  the same boundary key pool, and keys the relocation would rescue are already
  reclaimed through promotion. Independently-discovered gains overlap rather
  than add.

## Open questions

- Does the cascade keep paying out if deepened further (larger hop budget,
  smarter victim selection than LRU), or does it saturate once the reachable
  bucket neighborhood fills — the same occupancy ceiling in a new guise?
- With combination shown non-additive here, is there *any* pair of discovered
  seams on this trace that stacks, or is every lever ultimately competing for
  the same thin pool of boundary-oscillating keys?

## Continuation

The live frontier is the stash cascade depth (v003), not the single hop or
balanced admission — drop power-of-two-choices admission (it regressed; primary
locality matters more than even fill) and stop trying to graft the single hop
onto the protected-tier winner (confirmed overlapping, not additive). Next
cycle should deepen v003: test victim-selection policy and hop-budget
sensitivity to find where the cascade saturates. The recurring finding that
seams overlap rather than compound, plus the five-cycle skew/occupancy
bottleneck, makes validating against a second, less-skewed trace overdue before
investing further in this single workload.
