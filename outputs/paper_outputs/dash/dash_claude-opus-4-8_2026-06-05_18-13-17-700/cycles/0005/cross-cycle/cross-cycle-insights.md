## Recurring wins

- Protecting the warm working set in the promotion-protected normal tier is
  the dominant source of hit-rate. Reallocating fixed budget away from the
  over-provisioned stash and into the protected tier paid off repeatedly and
  was the largest single lever found.
- Eviction/demotion-path dynamics are a separate, still-live seam from the
  hit-path mechanics: re-landing demoted keys at MRU protects the
  oscillating working set and gains where promotion-position levers had
  already flattened.
- Two-speed promotion (slow first hop into target, fast second hop into
  probing) reserves the protected tier for twice-touched keys and blocks
  one-hit cold spikes — a genuinely distinct working lever.
- Every win has been budget-conserving and invariant-preserving; gains come
  from reshaping where budget sits, not from spending more of it.

## Recurring failures

- Removing or gutting stash admission regresses sharply (shown twice). The
  stash filter is load-bearing: it gives new keys dwell time to prove
  themselves, and without it cold one-hit-wonders displace resident hot keys.
- Stacking a selective-admission frequency gate on top of an already-enlarged
  protected tier is redundant — most benefit is captured by tier capacity, not
  by extra admission filtering.
- Front-inserting cold admissions (differentiating them from warm demotions
  by stash position) regresses: it makes new keys the immediate eviction
  victim and denies the dwell time the stash exists to provide.
- Trading bucket count for bucket width regresses — addressability beats
  depth; hot keys collide into fewer landing spots.
- Pushing the normal/stash split past its peak no longer climbs; the
  capacity-reallocation axis is saturated.

## Open questions

- Both surviving promotion-side levers (fractional gate, two-speed promotion)
  are throttled by the same root cause: target buckets skew to len-1/len-2, so
  no rank/recency rule has daylight. Is there any path to denser target
  buckets, or does the trace's hashing structurally cap every promotion lever?
- Are the freshest winning levers (demotion-MRU, widened gate, two-speed
  promotion) additive, or do they all chase the same thin pool of
  boundary-oscillating keys?
- Has the cohort over-fit to a single high-skew trace? The flattening of
  nearly every lever may be a property of this workload — validation against a
  second, less-skewed trace is overdue before chasing further slivers.

## Per-policy notes

- DASH tiering (stash + promotion-protected normal tier): the consistent
  winning structure. Budget belongs in the protected tier; the stash should be
  selective but never removed.
- Recency / promote-on-hit on the protected tier: well-matched to the skewed
  trace and the basis of every gain.
- LFU eviction on the protected tier: a dead lever — it retained
  stale-but-frequent keys over the live working set and regressed.
