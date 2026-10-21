## Recurring wins

- Rewarding proven reuse is the theme that has held up best. Changes that protect keys with demonstrated repeat reuse in main have paid off: a smaller small queue (cycles 1–2), a moderate main credit cap (cycle 3), and recording main victims in the shared ghost (cycle 5).
- Widening the ghost's evidence a little helps, as long as the bypass into main stays just as strict. This showed up in cycle 5 (ghosting main victims) and possibly in cycle 6 (the ring's slightly wider window).
- Front-of-main re-admission at frequency 0 is the stable choice. Every alternative tried so far has been worse.
- Changes that only touch representation, verified against a reference, have always kept the policy exact and combined cleanly. These include the indexed slab/index port, the fingerprint-ring ghost, the identity hasher and 32-bit fingerprints.
- Main-queue eviction order was the last lever left untouched. The SIEVE-style hand that keeps survivors in place (cycle 7) gave the first real hit-rate gain after several cycles of near-noise results.

## Recurring failures

- Making ghost re-admission more generous loses hit-rate. This held for credit on re-entry (cycle 2), a larger or coupled ghost (cycle 2), and a separate ghost budget for main victims (cycle 6). Each ghost hit bypasses the small filter and forces a main eviction, which churns keys that have proven reuse.
- Making ghost re-admission stricter also loses hit-rate. Tail insertion (cycle 3) and probation through the small queue (cycle 4) both regressed. The small queue's window is much shorter than the ghost's reuse horizon, so it rejects the keys the ghost correctly identified.
- Easier graduation from small to main is negative or flat. With a frequency reset it lost (cycle 1), and without one it was flat (cycle 5), because the ghost already recovers keys that were hit once in small.
- Retention tuning under FIFO reinsertion flattened out above cap 7. Raising the cap was flat (cycle 4) and halving credit was slightly negative (cycle 5).
- Adapting the split based on which source a ghost hit came from regressed (cycle 6). Small rejects dominate the signal, and the adaptation reacts after the key has already been re-admitted.
- Per-op-time and metadata-size have been requested every cycle but have gone unmeasured for 7 cycles. No other trace or cache size has been tested either. As a result, none of the cost-side changes can be judged.

## Open questions

- Under SIEVE, what are the right cap and credit rule (decrement vs clear-on-pass)? The cap and decay verdicts were only settled under FIFO reinsertion.
- Does the SIEVE gain come from frequency-0 ghost re-entries getting a full sweep? That mechanism is inferred and has not been logged.
- What do the indexed port, ring ghost, identity hasher and narrow fingerprints actually save in per-op-time and metadata-size? Do SIEVE's pointer-chasing sweeps and the double hash on misses cancel those savings out?
- Do the accumulated choices hold on other traces and cache sizes, or are they fitted to workload-a at one size? The choices are the 5% split, cap 7, ghosted main victims, the ring ghost and SIEVE main.

## Per-policy notes

- S3-FIFO (the only family explored), on workload-a:
  - Partition: the small queue shrank from the original split to 10% and then 5%. The curve is close to flat at 5%, so further shrinking should give diminishing returns. Don't reopen this lever.
  - Ghost: shared, capped budget fed by small rejects and main victims, implemented as a fingerprint ring. Don't enlarge it, split it, or credit re-entries.
  - Main: cap 7 with SIEVE-style in-place survival is the current baseline. Retention knobs need re-tuning under SIEVE, because a credit now means an extra hand lap rather than a trip back to the head.
  - Under FIFO reinsertion, the knobs for partition, graduation, re-admission, cap and decay looked saturated. SIEVE showed that eviction order was still a source of headroom.
