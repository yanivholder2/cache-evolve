# Cycle 0009 Insight

## Cohort outcome

The cohort tested position-split decay on the accumulated single-queue stack (v001),
SLRU with wide-bucket geometry added (v002), and a full SLRU combination that merged
cold-first eviction, decay-by-2 confined to probationary, and wide buckets (v003).
All three fell short of the parent incumbent; v001 was selected as winner for having
the smallest regression.

## Patterns

- **Counter value, not bucket position, is the reliable frequency discriminator under
  cap-4.** Upper-half items span the full counter range 1–4, so sparing them with a
  gentler rate-1 decay preserves lukewarm items just as often as hot ones — the
  position split provides no useful signal that the counter itself does not already
  encode. Uniform rate-2 decay remains the better choice.
- **Wide geometry closes most but not all of the SLRU gap.** Adding the 4× wide-bucket
  overlay to SLRU (v002) confirmed geometry was suppressing v002's prior result, yet
  the design still trailed the single-queue family by a meaningful margin. The
  bottleneck is SLRU's demotion/promotion logic or its probationary-first eviction
  priority, not the bucket layout.
- **The strongest SLRU configuration yet still does not match the accumulated stack.**
  v003's three-way combination — SLRU tier separation, wide geometry, cold-first +
  decay scoped to probationary — produced the best SLRU result across any cycle, yet
  it was measured against ClassicLRU as baseline rather than against the parent
  incumbent, and it did not win. The single-queue family's implicit recency encoding
  (mid-point insertion + positional eviction) appears to carry less overhead than
  SLRU's explicit bookkeeping while achieving equivalent or better ordering.
- **The SLRU branch is converging toward a ceiling.** Two focused cycles of SLRU
  exploration (0008–0009) have each improved the design but have not crossed the
  single-queue threshold.

## Open questions

- What proportion of SLRU's deficit is attributable to the 50/50 segment split? A
  protected-heavy split (e.g., 25/75) might retain more of the working set per
  bucket.
- Per-op latency remains unmeasured. If v003's SLRU combination nearly matches the
  single-queue hit-rate, a lower per-op cost could make it the preferred
  implementation even without a hit-rate lead.
- Could a counter-value-based asymmetric decay (rate-1 for items at counter ≥ 3,
  rate-2 for items at counter ≤ 2) unlock the incremental gain that position-split
  decay failed to deliver?

## Continuation

The single-queue accumulated stack continues to hold. The SLRU branch has been
thoroughly explored and is approaching its apparent ceiling without overtaking the
incumbent — further SLRU geometry or eviction tuning is unlikely to be decisive. The
next cycle should execute the long-deferred latency audit: measure per-op cost for the
full accumulated stack versus the best SLRU configuration (v003 this cycle) to
determine whether the hit-rate delta justifies the mechanism complexity. If latency
parity or advantage is found for SLRU, it becomes the preferred baseline for further
exploration; if not, the single-queue branch should be declared the winner and
exploration should pivot to a qualitatively different policy family. Do not revisit
position-split decay — counter value is the correct discrimination axis.
