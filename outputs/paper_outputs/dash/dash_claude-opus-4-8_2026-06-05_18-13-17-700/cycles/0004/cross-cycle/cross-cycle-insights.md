## Recurring wins

- Reallocating fixed budget toward the promotion-protected normal tier was
  the dominant winning lever across cycles 0001–0002: the baseline stash was
  over-provisioned, and shifting capacity from even toward normal-dominant
  kept climbing hit-rate until it saturated.
- Protecting the live working set against transient/cold keys consistently
  pays off, whatever the mechanism — capacity shift, selective stash
  admission, or insert position all moved hit-rate the same direction.
- Insert-position levers on the warm path are mildly positive: MRU promotion
  (0003) and MRU re-landing of demoted keys (0004) both gained, the latter
  being a genuinely independent lever from the hit-path position.
- Wins are budget-conserving and invariant-preserving; the productive cohorts
  moved hit-rate along one axis without regressing other dimensions.

## Recurring failures

- Stacking a second stash-protection lever (selective-admission / frequency
  gate) on top of an already-enlarged protected tier is redundant, not
  additive — most benefit is captured by how much room the tier gets.
- Pushing the normal/stash split past its peak stops paying and edges lower:
  the capacity axis is saturated, and further starving the stash costs
  admission as much as it saves.
- Removing or gutting the stash admission step regresses sharply — it is
  load-bearing warm-set protection, not dead probation.
- Spending bucket count for bucket width regresses: addressability beats
  depth, hot keys collide into fewer landing spots. Bucket count is a value
  to preserve.

## Open questions

- Where exactly is the headroom now that the split axis and promotion-position
  mechanics are near-exhausted and every recent lever returns only slivers?
- Are the surviving live levers (widened promotion gate, demotion-MRU landing)
  additive, or do they chase the same small pool of boundary-oscillating keys?
- Is lever reach structurally capped by target-bucket occupancy on this
  heavily-hashed trace — would denser buckets unlock the gate rule?
- Has the cohort over-fit to a single high-skew trace? Diminishing returns may
  be a workload property; validation on a less-skewed trace is overdue.

## Per-policy notes

- Recency / promote-on-hit (protected normal tier): the right fit for this
  skewed trace; fast re-touch means hot keys climb cheaply and capacity is
  the main lever.
- LFU on the protected tier: a dead lever — it retained stale-but-frequent
  keys over the live working set and regressed.
- Stash admission filter: protective and non-negotiable; selective gating on
  top is redundant once capacity is generous, but removal is harmful.
- Promotion gate (target→probing): real but occupancy-bottlenecked; the
  quarter rule rarely engages because target buckets are too small, capping
  the gain.
