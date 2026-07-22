## Recurring wins

- Trimming dead metadata off the eval path is cheap and monotonic: fields
  carried only by off-path policies peel off repeatedly with zero hit-rate
  cost, driving `Item` down to a stable 16-byte floor.
- Keeping state off the measured `Item` (in the unmeasured bucket struct)
  makes scan-resistance essentially free on the size axis — hit-rate and
  metadata stopped being antagonistic once state moved off-`Item`.
- Stateless probationary-front admission lifts hit-rate at the floor with
  no per-item state and clean tails; the front slot is the settled optimum
  on the admission-geometry axis.
- The index-preserving swap reliably collapses the quadratic p99 tails that
  key-matched rebuilds inflate, with hit-rate held bit-for-bit — the proven
  fix for any stateful reuse-vector path.
- Genuine segment separation (2Q/SLRU-style protected segment) is the one
  lever that broke the long-standing hit-rate ceiling, and it does so while
  holding the metadata floor.

## Recurring failures

- Frequency/LFU-based victim choice loses to recency-front every time it is
  tried; permanent binary reuse-protection fits this trace better.
- Weakening protection — second-chance/CLOCK aging, reuse-bit promotion
  gates — sheds immunity too fast or is redundant with position, collapsing
  back toward plain LRU.
- Re-timing promotion on a single short bucket (single-step, gradual) does
  not buy hit-rate: hot keys are evicted before they ratchet out, and small
  buckets lack the slots gradual promotion needs.
- Widening the probation window (midpoint insertion) is strictly worse than
  front admission — never-reused scan items linger above the eviction slot.
- Stacking explicit reuse protection onto front-admission adds nothing: the
  two are the same lever under move-to-back-on-hit.

## Open questions

- Can the SLRU/segmented hit-rate win shed its per-access rotate/rebalance
  latency (lazy demotion, cheaper boundary bookkeeping)?
- What protected-segment cap is optimal for short buckets, and does the
  gain survive at the smallest bucket sizes?
- How much headroom remains above the two-segment ceiling, or is
  segmentation near the structural limit for this trace?
- The recurring upward p99/tail drift under flat p50 on metadata cuts —
  structural or persistent measurement noise?

## Per-policy notes

- LRU / front-admission: front slot is the admission optimum; move-to-back-
  on-hit makes the frontmost slot always probationary, so added reuse bits
  are redundant with position.
- Reuse-bit / protected (stateful): permanent binary protection is the
  right fit; keep state off `Item` and relocate it via index-preserving
  swaps to avoid quadratic tails.
- SLRU / 2Q segmentation: the only ceiling-breaking policy; trades latency
  (dual-segment rotate/rebalance) rather than metadata for hit-rate.
- LFU / frequency: consistently a worse victim signal than recency on
  saturated buckets — drop.
- Overlay hygiene (all policies): single-file variants repeatedly forfeit
  parent wins (e.g. the 16-byte `Item`) by omission; enforce full
  parent-layout inheritance.
