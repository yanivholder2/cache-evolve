## Recurring wins

- No pattern has repeated yet because the ledger has only one cycle. These are provisional signals that a later cycle needs to confirm:
  - Moving capacity toward the main queue paid off on workload-a. That means a smaller small queue (around 10%), with the ghost queue growing along with main.
  - Keeping policy changes separate from data-structure changes produced a clean result. A behaviour-neutral structural rewrite left hit-rate unchanged, so it can serve as a base for per-op-time work.

## Recurring failures

- Not recurring yet, with one cycle. Seen once:
  - Making it easier to graduate into main (a lower threshold) lowered hit-rate. The policy seems to reward keys that are reused repeatedly over keys reused once.
  - Changing two things in one variant (threshold plus a frequency reset) made the outcome impossible to attribute.
  - Reporting hit-rate only left the per-op-time and metadata-size claims untested.

## Open questions

- Does a larger ghost queue hurt per-op-time while ghost lookups still use linear scans?
- Does the indexed-slab structure lower get/put latency, and what does it cost in metadata-size?
- Was the graduation regression caused by the lower threshold or by the frequency reset? Test the threshold alone.
- Where is the best small-queue split near 10%? Does sizing the ghost queue separately from main help?
- Do these results hold beyond workload-a?

## Per-policy notes

- S3-FIFO:
  - The ghost queue acts as a safety net. Keys that miss graduation can still get back into main, which dampens the effect of probation-window and promotion-threshold changes.
  - Current best policy baseline: the 10% small-queue split. The next step is to combine it with the indexed structure.
  - Every variant should report hit-rate, per-op-time and metadata-size.
