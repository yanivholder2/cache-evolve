## Recurring wins

- Protecting the warm working set is the dominant winning family: every
  early gain came from giving the promotion-protected normal tier more room
  or shielding it from transient keys. Reallocating budget into that tier
  carried cycles 0001–0002 and remains the best-so-far baseline.
- Dwell time is what makes the stash load-bearing. Levers that let a cold
  key survive long enough to catch a second reference paid off (relocation
  hop, bounded multi-hop cascade on a clean baseline); the value is giving
  new keys time to prove themselves, not balanced occupancy.
- Two-speed promotion is a genuinely distinct positive: landing the
  stash→target hop slow while keeping target→probing fast reserves the
  protected tier for twice-seen keys and blocks one-hit spikes.
- Demotion-path insert position is independent of the hit-path position:
  re-landing probing-evicted keys at MRU protects the oscillating working set
  where hit-path tweaks had flattened.
- Orthogonality beats depth for stacking. The only seam that stacked on the
  winner was the ghost gate for dropped-then-recurring keys — a population
  promotion never recaptures. Reapplying an inert idea to a *hotter* victim
  pool (breadth-of-homes on the probing tier vs. cold stash) also worked.

## Recurring failures

- Removing or withholding stash admission regresses repeatedly: gutting the
  admission step (cycle 0004) and TinyLFU-style doorkeeper filtering (0010)
  both forfeit hits because the contended tail recurs too fast to filter.
- Promotion-gate widening is exhausted in every shape — rank-based quarter
  and half gates, density-independent touch counters, len-2 loosening — all
  blocked by the same structural small-bucket occupancy ceiling, not by rule
  shape.
- Front-landing cold keys (stash insert-position differentiation) regresses,
  shown twice (0004, 0005): it makes new keys the immediate eviction victim
  and denies the dwell time the stash exists to provide.
- Bucket geometry trades are dead: widening buckets while halving count
  regressed (0003); the reverse width-for-count trade was exactly neutral
  (0009). Under conserved budget, addressability and depth are interchangeable.
- Independently discovered seams overlap rather than compound. Stacking the
  single hop, deepening the cascade, victim quality, breadth, and combining
  the two 0009 winners all washed out — they compete for the same thin pool
  of boundary keys the protected tier already reclaims.

## Open questions

- Has single-workload search fully saturated? Nearly every probe for the last
  several cycles resolves to overlap or emptiness because the served
  populations are thin on this one trace.
- Are the recurring non-additivity and occupancy ceilings properties of this
  single high-skew trace, or general truths? Only a second, less-skewed trace
  can disambiguate — this validation has been deferred since cycle 0003 and is
  now the highest-value next step.
- Are there other inert stash ideas worth re-pointing at the warm
  probing/protected tiers under the "spare warm victims, not cold ones"
  principle?

## Per-policy notes

- DASH (the only policy exercised): the protected normal tier wants
  recency/promote-on-hit, not LFU — forcing LFU eviction regressed (0002) by
  retaining stale-but-frequent keys over the live set.
- Bucket count is a value to preserve, not a free associativity knob; hot-key
  collisions are governed by total capacity, not its partition.
- The stash behaves as a dwell-time buffer, not probation; its admission
  filter is structurally load-bearing and must not be removed or delayed.
- Promotion-side mechanics (gate width, promotion position, touch signals)
  are occupancy-saturated on this trace; eviction/demotion dynamics and
  orthogonal warm-population seams have been the more productive frontier.
