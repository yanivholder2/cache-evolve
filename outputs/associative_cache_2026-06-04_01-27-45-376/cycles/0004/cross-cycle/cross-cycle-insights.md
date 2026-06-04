## Recurring wins

- Eviction-side discrimination on top of preserved LRU ordering: cycle
  0001/v003's "evict an untouched slot before a touched one" filter
  has held the Pareto front through three subsequent cohorts of
  attempted refinement.
- Reusing the existing `Item.lfu_counter` as a binary touched/untouched
  signal — zero added per-item bytes, no trait changes — is the only
  use of that field that has paid off; richer interpretations have all
  lost.
- Per-bucket state that doesn't grow per-item metadata (cycle 0004/v002's
  cursor) is at least metadata-flat and reaches the Pareto front even
  when the hit-rate idea it carried fails; the vehicle is worth
  separating from the payload.

## Recurring failures

- Withholding or shortening promotion inside the ~16-slot bucket
  (cycle 0001/v001 delayed promotion, 0001/v002 CLOCK, 0002/v003 one-
  slot swap): a hot newcomer has no headroom to survive its first-hit
  window, and the access-path axis has lost in every cohort that
  touched it.
- Letting any derived signal override the raw `lfu_counter == 0`
  predicate or LRU position: on-hit decay (0003/v001), positional
  gating (0003/v002), cardinality-based regime switches (0003/v003),
  insertion gap-fill (0004/v001), CLOCK rotation (0004/v002), and
  reverse scan (0004/v003) all regressed the parent.
- Treating axes as independent of ClassicLRU's move-to-back: cycle
  0004 showed that insertion placement, scan direction, and victim
  fairness are all coupled back into v003's filter through slot
  ordering, so "orthogonal" refinements keep destroying a signal v003
  is silently consuming.
- Counter-rank eviction without aging (cycle 0002/v001 tiering,
  0002/v002 min-counter): once every slot is touched, lifetime hit
  counts fight recency and lose on this trace.

## Open questions

- Does the existing trait surface expose any axis that is *not*
  coupled through move-to-back into v003's filter? Every candidate so
  far has turned out to share that coupling.
- Would widening the `Bucket` / `EvictionPolicy` trait (cycle 0001
  flagged the `&[Item]` eviction signature; cycles 0003–0004 echoed
  the call) unlock genuinely new state — bucket-side reference bit,
  per-slot insertion tick — that the current surface can't carry?
- Is hit-rate on this trace effectively saturated for the signals the
  current trait exposes, making it time to shift attention to the
  metadata-size axis?
- Is per-bucket state (cursor-style, no per-item bytes) worth keeping
  as a vehicle for *other* signals even though rotation-based fairness
  lost?

## Per-policy notes

- ClassicLRU: the bucket's move-to-back is doing more work than it
  appears. Slot index correlates with age, freshly-inserted untouched
  slots tend to land at index 0, and v003's leftmost-first scan
  exploits that for free. Policies that touch promotion, insertion
  placement, or scan direction disturb this hidden signal.
- CLOCK / CLOCK-like rotations: tried twice (0001/v002 as the access
  policy, 0004/v002 as a fairness layer over v003) and lost both
  times — the two-level reference bit and rotation across candidates
  strip recency information the exact remove+push was preserving.
- LFU-counter-based eviction: only the binary zero/non-zero reading
  has won. Multi-bit tiering, min-counter rank, and on-hit decay have
  each regressed; any future counter-based policy almost certainly
  needs an explicit aging or decay step before tiering can compete.
- Bucket size (~16 slots) is the regime constraint behind most
  failures: too narrow for scan-resistance via withheld promotion,
  too narrow for CLOCK-style coarsening, and small enough that the
  "all touched" regime arrives quickly and breaks counter-rank
  schemes.
