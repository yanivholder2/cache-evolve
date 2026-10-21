## Recurring wins

- Shrinking the small (probation) queue to give the main queue more room has won twice in a row (baseline → 10% → 5%). The gains come from a larger main queue, not from a larger ghost queue. The second step gained less than the first, so the curve is close to flat.
- Rewarding proven repeated reuse beats making entry easier. On workload-a, protecting main capacity for keys with demonstrated reuse is the most reliable way to gain hit-rate.
- Single-change variants give clear attribution. Where one thing was changed (partition, ghost size), the result could be explained.

## Recurring failures

- Making admission or re-admission more lenient keeps losing hit-rate. This includes lowering the graduation threshold, giving ghost-readmitted keys a free frequency credit, and simply enlarging the ghost queue. One reuse signal is weak evidence on workload-a, and keys let in on it push out keys with proven reuse.
- No cycle has measured per-op-time or metadata-size, even though both Continuations asked for them. Every ghost-growing winner adds to unmeasured linear-scan cost.
- The port of the winning policy onto the behaviour-neutral indexed structure has been carried over twice and never done.
- Changing two things at once (threshold plus frequency reset in cycle 0001) left a regression that can't be attributed.

## Open questions

- What per-op-time and metadata-size cost does the current winner carry under linear ghost scans? Does the indexed structure remove that cost?
- Does making ghost re-admission stricter pay off? Examples: requiring repeat ghost hits, or inserting at the main tail. Would a stricter rule make a larger ghost worth having?
- Does the partition gain hold on other traces and cache sizes, or is it specific to workload-a?
- Does a lower graduation threshold alone, without the frequency reset, still regress?

## Per-policy notes

- S3-FIFO (the only policy explored so far, on workload-a only):
  - Partition: smaller small-queue fractions help, with diminishing returns. The current baseline is 5%, and going below it is unlikely to pay off much.
  - Ghost queue: it already acts as a safety net for keys that miss graduation, so changes to the probation window or promotion timing matter less than expected. Changes that make the ghost more generous hurt; only changes that make it more selective are still unexplored.
  - Graduation: earlier promotion hurt. Only the threshold change on its own is still untested.
  - Data structures: the indexed-slab variant kept hit-rate unchanged, but its latency and memory effects are still unmeasured.
