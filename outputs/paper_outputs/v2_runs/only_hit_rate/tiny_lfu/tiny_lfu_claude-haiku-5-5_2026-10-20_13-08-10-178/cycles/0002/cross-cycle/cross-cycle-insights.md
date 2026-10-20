## Recurring wins

- Layout-only changes to the frequency sketch's storage, which keep the logical counter mapping and the admission rule intact, have not regressed hit-rate in either cycle. This is "no regression," not a measured gain.
- Keeping the sketch's counts and the admission comparison unchanged is the only approach that has produced Winners in more than one cycle.

## Recurring failures

- Changing the sketch's behaviour has lost hit-rate in both cycles. Fixed-schedule halving discarded signal admission needed at this capacity, and remapping key-to-counter positions produced a small dip.
- Changing when or where admission sees a new key has lost hit-rate. A recency window placed ahead of the LRU did not capture the one-touch bursts it was meant to handle.
- No cohort in either cycle has recorded a hit-rate gain. Claimed benefits (memory footprint, per-op cost) sit on dimensions the fitness vector does not measure, so Winners have advanced only by not regressing.

## Open questions

- Would a memory or per-operation timing signal in the fitness vector confirm the layout gains, and would that justify keeping them?
- Is the small hit-rate dip from the key-to-counter remapping within noise, or a real regression that rules out that family of changes?
- Would a gentler or staleness-gated sketch decay recover signal without the admission losses seen so far?
- Does a narrower counter width create admission ties or saturation on traces with very hot keys?
- Is there a structure that helps one-touch bursts without weakening the sketch signal at this capacity?

## Per-policy notes

- TinyLFU admission (sketch counts and admission comparison): every variant that altered sketch counts, decay, or position mapping lost hit-rate. Only storage-layout changes have held.
- Recency window and LRU main region: placing a window ahead of the LRU with admission at window exit lost hit-rate. Treat this structure as unpromising at the current capacity.
- Sketch storage layout (counter width, array order): the only consistently non-regressing family, but its gains are unmeasured until a memory or timing dimension is added.
