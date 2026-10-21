## Recurring wins

- On workload-a, rewarding keys that have shown repeated reuse beats making entry easier. This has held across nearly every policy cycle (0001–0006, 0008).
- Main-queue eviction order (cycles 0007–0008) has been the only lever to give gains beyond noise since the early partition sweep. SIEVE-style in-place survivors won first. Clear-on-pass, a single visited lap, then improved on that.
- Shrinking the small queue (to 10%, then 5%) gave small but consistent gains. They came from a larger main queue, not a larger ghost. The curve was close to flat by 5%.
- Ghost re-admission works best at front-of-main with zero credit. A bare ghost hit is enough evidence of reuse. Ghosting main victims in the shared, capped budget added a sliver.
- Cost-only representation changes have been policy-neutral every time and compose cleanly: the indexed slab, the fingerprint ring, the identity hasher and 32-bit fingerprints. Each was verified against a reference.

## Recurring failures

- Making re-admission more generous (freq credit, a decoupled or larger ghost, a separate main-victim ghost budget) keeps losing. Each ghost hit bypasses the small filter and forces a main eviction.
- Making re-admission stricter also loses: tail insertion and small-queue probation. The small window is shorter than the ghost's reuse horizon, so these reject keys the ghost correctly identified.
- Earlier graduation from small loses or stays flat both with a frequency reset and without one. The ghost already recovers keys that were hit once.
- Protecting newcomers longer loses under both FIFO and SIEVE. Under SIEVE, behind-hand insertion regressed.
- Adaptive splits driven by which source a ghost hit came from regressed. The signal is biased toward small rejects and arrives too late.
- Measuring only hit-rate has been the constant process failure. Per-op-time and metadata-size have gone unmeasured for eight cycles, and only one trace and one size have ever been run. No cost-side change has been judged on its actual purpose.

## Open questions

- Under SIEVE, does a small main-only cap (2–3 laps) beat a single visited bit?
- With clear-on-pass, should graduation from small use a counter separate from main retention?
- Should ghost re-admissions get less protection than graduates under SIEVE, for example insertion nearer the hand?
- What do the indexed, ring, hasher and fingerprint changes actually save in per-op-time and metadata-size? Does SIEVE's pointer-chasing cancel those savings out?
- Do the accumulated choices (5% split, front-of-main re-admission, ghosted main victims, SIEVE with clear-on-pass) hold on other traces and cache sizes?

## Per-policy notes

- S3-FIFO, FIFO-reinsertion main (cycles 0001–0006): a frequency cap of 7 was best. Going 3→7 helped, 15 was flat, and decay hurt. The partition, graduation, re-admission and ghost-capacity knobs are all settled, flat or negative. The family looked saturated on workload-a.
- S3-FIFO, SIEVE main (cycles 0007–0008): retention conclusions from the FIFO era do not carry over. Less retention is better, and clear-on-pass beat multi-lap credit. Quickly demoting newcomers near the hand does useful filtering. The gain comes from survivors staying in place. Retention, graduation and re-admission protection under SIEVE are the live policy space.
- Ghost representation: the fingerprint ring with 32-bit fingerprints and identity hashing is the current form. It is policy-neutral, and false-hit risk is negligible at this size.
