# Cross-Cycle Insights

## Recurring wins

- Separating "seen once" from "proven," and protecting proven keys from
  transient/scan traffic, has raised hit-rate wherever it was tried —
  via promotion-on-second-hit, FIFO admission, or a frequency estimate.
- Gains have come from changing *which* resident is evicted, not from
  reworking eviction mechanics; the baseline's O(1) slot reuse was left
  intact.
- Gating/admission logic, rather than capacity tuning, has been the
  decisive lever when one was identified.

## Recurring failures

- No approach has yet failed across cycles; too few cycles to mark a
  pattern as consistently unrewarding.

## Open questions

- Hit-rate is the only scored dimension, so per-access overhead (extra
  lists, ghost maps, sketches) is invisible — would the winning edge
  survive if cost were ever scored?
- The segmentation, ghost-queue, and frequency-admission strategies are
  largely orthogonal — would combining a frequency/ghost signal with
  segmentation compound gains or just add overhead?
- How sensitive are results to the probationary/protected split?

## Per-policy notes

- SLRU (segmented): emerged as the standout on scan-polluted traces;
  promotion rule credited over the protected-segment cap.
- 2Q with ghost queue: improved hit-rate; viable as a frequency/recency
  signal source to layer onto segmentation.
- TinyLFU (frequency-gated admission): improved hit-rate; admission gate
  credited over any structural change. Lower overhead, but that edge is
  unmeasured while only hit-rate is scored.
