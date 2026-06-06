# Cross-Cycle Insights

## Recurring wins

- Reallocating fixed budget toward the promotion-protected normal tier was the
  strongest early lever: shrinking the over-provisioned stash and pushing the
  split from baseline → even → normal-dominant climbed hit-rate for two cycles
  before saturating.
- Protecting the warm working set pays off repeatedly, whichever mechanism: a
  selective stash gate, MRU insertion, or capacity all moved the same way.
- Eviction/demotion-path dynamics are a genuinely independent seam from the
  hit-path levers: landing demoted keys at MRU end gained where hit-path
  position had already flattened.
- Two-speed promotion (slow stash→target hop, fast target→probing hop) reserves
  the protected tier for twice-referenced keys and gains where uniform
  promotion flattened.
- Stash dwell-time is the most durable source of gains: giving new cold keys
  time to prove a second reference helps, and a cuckoo-style relocation hop into
  an alternate bucket is the freshest positive seam late in the search.

## Recurring failures

- Removing or gutting stash admission regresses sharply — the filter is
  load-bearing warm-set protection, not dead probation. Confirmed twice.
- Front-inserting cold keys into the stash makes them the immediate eviction
  victim and denies dwell time; regressed in two separate cycles.
- Promotion-gate widening is exhausted from every angle — rank fraction
  (quarter, half), len-2 loosening, and density-independent touch counting all
  hit the same small-bucket occupancy ceiling and return slivers or negatives.
- Swapping the protected tier to LFU eviction regressed: it retains
  stale-but-frequent keys over the live working set.
- Trading bucket count for bucket width regressed; addressability beats depth at
  conserved budget — bucket count is a value to preserve.
- Stacking a selective-admission gate on top of the capacity reshape was
  redundant, not additive — the two chase the same imbalance.

## Open questions

- Is the recurring small-bucket occupancy ceiling a structural property of the
  trace's hashing, or is there a path to denser target buckets that would
  reopen the promotion-side levers?
- Does the stash relocation hop deepen (multi-hop cascade, better victim
  selection, larger alternate set) or saturate as fast as prior levers once
  alternate buckets fill?
- Are the late slivers additive or do they chase the same boundary-oscillating
  key pool (gate width + demotion-MRU; relocation + two-speed promotion)?
- Validation against a second, less-skewed trace is now overdue — levers have
  flattened for several cycles, and the diminishing returns may be a property of
  this single high-skew workload rather than the design.

## Per-policy notes

- Recency / promote-on-hit ranking suits the high-skew trace well; fast re-touch
  means "earn your way up" delays cost few hits, so promotion-position tweaks
  yield only slivers.
- LFU on the protected tier is a dead lever on this workload — it holds stale
  frequent keys against the oscillating live set.
- Stash admission is settled and protective: dwell time is its value; its insert
  position is closed in both directions (protective MRU default holds,
  cold-front-insertion hurts).
- Tier geometry (split ratio, promotion position, bucket count) is near-saturated;
  the live frontier is stash dwell-time and eviction/demotion dynamics.
