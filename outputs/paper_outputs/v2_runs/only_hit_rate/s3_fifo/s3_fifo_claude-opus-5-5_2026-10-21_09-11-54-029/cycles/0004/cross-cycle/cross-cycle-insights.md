## Recurring wins

- Giving main more capacity has paid off for two cycles running (cycles 1–2). The small queue went from baseline to 10% to 5%, and the gain each time came from a bigger main queue, not a bigger ghost. The curve is now nearly flat.
- On workload-a, protecting keys with proven reuse is the one lesson that has held every cycle. Changes that guard main-queue residents (smaller small queue, higher frequency cap up to 7) helped. Changes that made it easier to enter main did not.
- It works to separate representation from policy. Behaviour-neutral indexed ports (cycles 1, 3, 4) matched their parents' hit-rate exactly each time. That means structure and policy changes can be stacked safely.
- Changing one thing per variant makes results easy to attribute. The one variant that changed two things together (cycle 1's v002) couldn't be explained.

## Recurring failures

- Ghost re-admission changes have lost hit-rate in every form tried:
  - extra credit (cycle 2)
  - tail-of-main insertion (cycle 3)
  - small-queue probation (cycle 4)
  - a larger or decoupled ghost (cycle 2)

  A bare ghost hit is enough evidence of reuse, and front-of-main at freq 0 is the settled point. Stop exploring this axis.
- Earlier graduation from the small queue (cycle 1) hurt. Easy entry pushes proven keys out of main.
- Raising the frequency cap above 7 stopped paying off (cycle 4 ended in a tie). Eviction victims are mostly freq-0 re-entries, so extra credit rarely changes which key is evicted. The higher cap only makes sweeps longer.
- Per-op-time and metadata-size have not been measured in any of the four cycles, even though every Continuation asked for them. Every ghost-growing or cap-raising Winner adds a scan or sweep cost that nobody has measured. That leaves the Winner choices only partly justified.

## Open questions

- How much per-op-time does the indexed port save, and how much metadata-size does it add (duplicated keys, stale ghost deque, compaction spikes)? This is the main blocker. Measure it first.
- Does cap 15 on the linear structure cost enough per-op-time to make it a net loss compared with cap 7?
- Is there hit-rate headroom left on workload-a now that partition size, cap and re-admission are flat or settled? Credit aging or decay in main is the main retention lever not yet tested.
- Could decoupling the graduation threshold from the cap help, with the threshold changed alone and no frequency reset?
- Do these gains hold on other traces and cache sizes? Every result so far comes from a single workload.

## Per-policy notes

- S3-FIFO (the only policy explored so far): the effective baseline is a 5% small queue, cap 7 and front-of-main freq-0 ghost re-admission, running on the indexed slab/index/lazy-ghost structure.
- S3-FIFO partition: improved from baseline to 10% to 5%, with diminishing returns. Going below 5% is probably not worth it.
- S3-FIFO ghost: it does most of the filtering at small partition sizes. Keep it coupled to main and non-generous.
- S3-FIFO main retention: the cap axis paid off up to 7 and is flat beyond that. Future work should change how credit is spent or aged, not how much credit there is.
