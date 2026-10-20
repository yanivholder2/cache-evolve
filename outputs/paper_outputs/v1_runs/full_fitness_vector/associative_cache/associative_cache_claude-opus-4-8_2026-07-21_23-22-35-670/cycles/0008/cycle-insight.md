# Cycle 0008 Insight

## Cohort outcome

The cohort turned fully onto the latency frontier that cycle 7 pointed to,
attacking the incumbent's per-access costs from three angles: skip the
below-cap protected rotation (v001), replace the O(n²) reuse-vector rebuild
with an index-preserving move (v002), and re-probe the protected cap at a
mid-range setting (v003). The real win was v002 — collapsing both p99 tails
while holding the best-on-record hit-rate and the 16-byte floor bit-for-bit.
v001 proved its target inert but unmeasurable; v003 confirmed the cap is a
nearly flat lever.

## Patterns

- The incumbent's blown-out tails were never intrinsic to carrying
  scan-resistance — they were a snapshot-and-rebuild artifact. Mirroring the
  policy's own single-element relocation onto the parallel reuse vector
  (v002), instead of reconstructing it by key-search each hit, makes the
  scan-resistance state free on latency as well as metadata. This is the
  cycle's clean, portable latency win.
- Not every per-access move is worth optimizing: the below-cap protected
  rotation is a single-element shift beneath the harness's 100ns timer
  resolution, so removing it (v001) is provably hit-rate-inert yet cannot
  register as a p50/p99 gain. Latency effort must target costly paths
  (per-hit rebuilds, front shifts), not sub-resolution single-element moves.
- The protected-cap knob is now mapped end to end and reads as nearly flat
  across its usable range. Both extremes were falsified in prior cycles; the
  mid-range `3*max_size/4` (v003) moves hit-rate only a hair and pays for it
  in a doubled `get` tail from deeper protected-MRU rotations. This lever is
  effectively exhausted as a hit-rate source.
- Metadata hygiene held throughout — every change stayed off the measured
  `Item`, and the anchored-parent discipline (v002 chose its parent
  explicitly to avoid the 40-byte overlay trap that sank the earlier attempt)
  kept the 16-byte floor intact.

## Open questions

- With the O(n²) rebuild gone, where do the incumbent's remaining tails sit —
  the `insert(0,…)`/`remove(0)` front shifts, or is the path now essentially
  at its latency floor?
- Is there any latency headroom left that the harness can actually resolve,
  or has sub-100ns granularity become the binding constraint on further
  micro-optimization?

## Continuation

Adopt v002's index-preserving reuse-vector move as the incumbent: it
delivers the record hit-rate with clean p99 tails at the 16-byte floor, and
supersedes the rebuild-based path. Stop tuning the protected cap — all three
regions (aggressive, near-total, mid-range) are now explored and it is a
flat lever; treat it as settled at the parent default. Do not pursue
sub-resolution single-element optimizations like v001's rotation skip; they
are correct but unrewardable by the eval harness. If latency is pursued
further, target the costlier front-shift paths (`insert(0)`/`remove(0)`) and
first confirm there is a harness-resolvable gain to capture, rather than
another hit-rate probe — the hit-rate ceiling remains firm.
