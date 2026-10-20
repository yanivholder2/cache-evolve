# Cycle 0005 Insight

## Cohort outcome

The cohort attacked the plateaued hit-rate ceiling exactly where cycle 4
pointed — the base policy's promotion ordering — from three angles:
gradual single-step promotion (v001), a two-hit reuse-bit probation gate
(v002), and a genuine two-segment SLRU bucket (v003). No variant qualified
as a Winner, but the cohort produced its first real evidence that hit-rate
headroom exists and is reachable only by adding true segment structure, not
by re-timing a single-list promotion.

## Patterns

- Re-timing promotion on a single list does not buy hit-rate on these
  short associative buckets. Slowing promotion (single-step, v001) actively
  regressed — a hot key gets evicted before it can ratchet out of the front
  zone — and gating promotion behind a reuse bit (v002) left the resident
  set unchanged, buying latency but not accuracy. Gradual/frequency-like
  promotion needs more slots than a small bucket provides.
- Genuine segment separation is the lever that moved hit-rate. Only v003,
  which gives twice-hit keys a reserved protected segment a one-hit scan
  burst cannot evict, crossed the front-admission ceiling. This confirms
  cycle 4's read that remaining headroom requires changing the base
  ordering itself, and identifies *segmentation* (2Q/SLRU-style) as the
  specific structural change that unlocks it.
- The hit-rate gain is bought with latency, not metadata. v003 keeps its
  segment boundary off-`Item` (16-byte floor holds), but every access now
  rotates and rebalances two segments, pushing get/put tails up. The new
  frontier is a hit-rate/latency trade, no longer a hit-rate/metadata one.
- The index-preserving swap keeps paying off (v002 collapsed the quadratic
  p99 tails as predicted), but the overlay-hygiene hazard recurred: v002
  carried only the bucket file and silently reverted `Item` off the 16-byte
  floor to the 40-byte base — a metadata regression by omission, not logic.

## Open questions

- Can v003's SLRU win keep its hit-rate gain while shedding the per-access
  rotate/rebalance latency — e.g. cheaper boundary bookkeeping or lazy
  demotion?
- What protected-segment cap is optimal for these short buckets, and does
  the gain survive at the smallest bucket sizes?
- How much headroom is left above the SLRU ceiling, or is two-segment
  separation itself near the structural limit for this trace?

## Continuation

Promote v003's segmented (SLRU) bucket to the incumbent worth building on —
it is the only path this cycle that broke the hit-rate ceiling, and it does
so at the metadata floor. Next cycle should focus on reclaiming its latency:
reduce the per-access rotate/rebalance work and tune the protected cap,
rather than chasing further hit-rate. Drop single-list promotion re-timing
(single-step under-protects; reuse-bit gating is redundant with position) —
the evidence is now firm that segmentation, not promotion timing, is the
lever. Carry v002's index-preserving swap into any stateful path, and
enforce full parent-layout inheritance so wins like the 16-byte `Item` are
not dropped by single-file overlays.
