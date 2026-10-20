## Recurring wins

- Landing position beats extra item movement: placing a re-referenced stash item at the MRU end of its target bucket was the only change that helped, so far the one winning lever is where an item lands, not how often it moves.

## Recurring failures

- Adding movement between tiers has not paid off: re-inserting probing-tier hits pushed proven-hot keys back through target competition and lost hit-rate.
- Shortening the demotion path in isolation has not paid off: skipping the target retry for foreign evictions lowered hit-rate, and the expected per-op-time gain was never measured, so the trade was never justified.

## Open questions

- Does the MRU landing for stash hits hold on workloads other than the single LIRS trace it was measured on?
- Per-op-time and metadata-size have not been measured under any variant; does any tier change move them at all?
- Would the removed target retry pay off if paired with better stash capacity handling, or is it a dead end regardless?

## Per-policy notes

- Only one policy family (the LIRS-style tiered probing/stash/normal setup) has been exercised; no per-policy regularities can be separated out yet.
- Treat the stash-hit landing result as a hit-rate result on one workload, not a demonstrated general improvement, until it is measured on more workloads.
