## Recurring wins

- Reallocating fixed budget toward the promotion-protected normal tier was
  the dominant early win: it climbed hit-rate from the baseline through past
  even and into normal-dominant before saturating.
- Protecting the warm working set is the through-line behind every gain —
  whether by capacity, insertion position, promotion gating, or demotion
  landing — transient/cold keys must be kept from displacing resident hot keys.
- Stash dwell-time is the most durable late-stage seam: giving cold keys time
  to prove a second reference (two-speed promotion, relocation hop, bounded
  multi-hop cascade) yielded the only fresh positives once geometry saturated.
- Reactive rescue of genuine collision victims beats proactive balancing;
  preserving primary-bucket locality consistently outperforms even fill.

## Recurring failures

- Removing or gutting stash admission regresses sharply — the admission step
  is load-bearing warm-set protection, not dead probation.
- Front-inserting cold keys at the stash eviction edge regressed twice; cold
  keys need dwell time, so denying residency is consistently wrong.
- Promotion-gate widening of every shape (quarter, half, len-2 loosening,
  density-independent touch counters) is capped by a structural small-bucket
  occupancy ceiling, not by rule shape — repeatedly flat or negative.
- Trading bucket count for bucket width regressed: addressability beats depth.
- Independently-discovered seams overlap rather than compound — stacking late
  slivers (relocation onto protected tier; depth/victim/breadth on the winner)
  comes back flat because they chase the same boundary-key pool.

## Open questions

- Is every hit-rate lever ultimately gated by the same hashing skew /
  small-bucket occupancy, or is stash dwell-time structurally different?
- With combination shown non-additive repeatedly, does any pair of discovered
  seams stack on this trace, or do all compete for one thin boundary pool?
- The cohort has searched a single high-skew trace across all cycles —
  validating against a second, less-skewed trace is long overdue and now the
  highest-value next step before further single-workload variants.

## Per-policy notes

- Recency / promote-on-hit (protected normal tier): the workhorse; suits the
  skewed trace and underpins every banked win. Promotion mechanics (position,
  two-speed) gave only slivers once capacity saturated.
- LFU on the protected tier: a dead lever — retained stale-but-frequent keys
  over the live working set and regressed.
- Stash / admission probation: LRU-style dwell-time is the value; relocation
  and bounded cuckoo cascades extend it, but depth inverts once the protected
  tier is present (it evicts already-warmed residents).
