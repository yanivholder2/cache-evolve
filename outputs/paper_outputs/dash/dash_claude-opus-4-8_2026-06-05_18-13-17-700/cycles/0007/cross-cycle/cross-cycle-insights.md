## Recurring wins

- Protecting the warm working set is the dominant theme: every early gain came
  from giving the promotion-protected normal tier more room or shielding it
  from transient keys (capacity reallocation, insertion position, promotion
  gating).
- Capacity reallocation toward the protected tier was the strongest single
  lever — it paid off across two cycles past the even split before saturating.
- Stash dwell-time is the deepest still-live seam: letting cold keys survive
  long enough to catch a second reference keeps yielding clean, capacity-
  conserving gains when promotion-side levers have flattened.
- Eviction/demotion-path placement (landing demoted keys at MRU) is a genuinely
  independent lever from hit-path placement and gained where the hit-path had
  already exhausted.
- Reactive rescue of real collision victims (multi-hop relocation cascade)
  beats single-hop and beats proactive balancing — depth pays where breadth
  does not.

## Recurring failures

- Promotion-gate widening is exhausted from every angle (rank, fraction,
  density-independent touch counting): target buckets skew so hard toward
  len-1/len-2 that no signal shape beats the small-bucket occupancy ceiling.
- Removing or gutting stash admission regresses sharply — the stash filter is
  load-bearing warm-set protection, not dead probation.
- Front-landing cold stash keys (cold/warm differentiation by insert position)
  regressed twice: cold keys need dwell time, not immediate eviction-victim
  status.
- Swapping the protected tier to LFU regressed — recency/promote-on-hit suits
  the skew; LFU retains stale-but-frequent keys.
- Trading bucket count for bucket width regressed — addressability beats depth;
  bucket count is a value to preserve.
- Pre-emptive load-balancing (power-of-two-choices admission) regressed —
  primary-first locality matters more than even fill.
- Independently-discovered gains tend to overlap rather than add: stacked levers
  (gate+demotion, single-hop+protected-tier) came back flat, all chasing the
  same thin pool of boundary-oscillating keys.

## Open questions

- Does the stash relocation cascade keep paying as it deepens (larger hop
  budget, smarter-than-LRU victim selection), or does it saturate once the
  reachable bucket neighborhood fills — the occupancy ceiling in a new guise?
- Is there *any* pair of discovered seams that compounds, or is every lever
  ultimately competing for the same small boundary-key pool?
- Is the recurring skew/occupancy bottleneck a property of the design or of a
  single high-skew trace? Validating against a second, less-skewed trace is now
  long overdue before investing further.

## Per-policy notes

- Recency / promote-on-hit (protected normal tier): the right default for this
  skew; benefits from more capacity and MRU demotion placement, but its
  promotion-gate and split levers are now saturated.
- LFU on the protected tier: a dead lever here — retains stale frequent keys
  over the live working set and regresses.
- Stash (admission/probation tier): the only consistently live axis; its value
  is dwell time. Protective MRU default holds; relocation-cascade depth is the
  freshest positive direction.
