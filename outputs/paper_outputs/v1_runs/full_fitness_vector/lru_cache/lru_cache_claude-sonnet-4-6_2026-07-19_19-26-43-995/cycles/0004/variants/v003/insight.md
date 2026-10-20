# Variant v003 Insight

## Helped

- **hit-rate**: Requiring two probationary accesses before graduation marginally filters one-hit wonders, yielding a slight improvement on the Zipfian workload where the very hottest items dominate hit traffic regardless of admission policy.

## Hurt

- **per-op-time**: First probationary hits now execute `unlink` + `push_back_prob` + a `SEEN_BIT` OR-assignment instead of returning immediately, pushing the get p50 from the parent's near-zero measurement up to the 100 ns tier — the extra work per qualifying miss erases the latency advantage the parent held on the common probationary-hit path.

## Takeaway

Encoding SEEN_BIT in bit 30 of the existing `prev` field costs nothing in node layout (24 bytes preserved), but the two-hit deferral introduces an unconditional extra list-splice on every first probationary access, and on workload-a the gain in protected-slot quality is too small to compensate for that per-operation overhead. The hypothesis is valid in theory, but this workload's Zipfian distribution already ensures that genuinely hot items accumulate two accesses quickly, so the filter adds latency without materially reducing false graduations.
