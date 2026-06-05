# Cycle 0001 Insight

## Cohort outcome

All three variants attacked the same target — one-hit-wonder / scan
pollution of a flat LRU — from three classic angles (SLRU segmentation,
2Q with a ghost queue, and TinyLFU frequency-gated admission). Every
variant raised hit-rate on this trace; the segmented SLRU (v001) landed
as the sole Winner.

## Patterns

- The shared winning idea across the whole cohort is separating "seen
  once" from "proven," and protecting proven keys from transient traffic
  — whether enforced by promotion-on-second-hit, FIFO admission, or a
  frequency estimate.
- None of the variants needed to alter the baseline's O(1) arena/slot
  reuse; all gains came from changing *which* resident is evicted, not
  how eviction mechanics work.
- Gating logic, not capacity tuning, was repeatedly identified as the
  decisive lever (v001 credited the promotion rule over the protected
  cap; v003 credited the admission gate over any restructuring).

## Open questions

- Hit-rate is the only measured dimension, so per-access overhead
  (extra lists, ghost map, count-min sketch) is currently invisible —
  is the winner's edge robust if cost is ever scored?
- The three strategies are largely orthogonal; would combining
  admission-frequency signal with segmentation compound the gains or
  just add overhead?

## Continuation

Pursue SLRU-style segmentation as the proven base for the next cycle and
build hypotheses on top of v001. Worth probing: layering a lightweight
frequency/ghost signal onto the SLRU promotion decision (hybrid of v001
and v002/v003), and varying the probationary/protected split. Drop pure
flat-LRU-with-admission-filter framings as standalone parents unless a
new dimension makes their lower overhead matter.
