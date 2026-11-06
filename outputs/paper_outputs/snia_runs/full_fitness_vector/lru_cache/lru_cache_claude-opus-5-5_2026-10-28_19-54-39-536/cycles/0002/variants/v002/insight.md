# Variant v002 Insight

## Helped

- hit-rate: The 2-bit counter that decrements by one lets a key hit several times survive several sweeps, where the parent's single bit gave it only one. This recovers some of the frequency signal the parent lacked. Almost all of the gain is on hm_0 at the large size. web_0 at the large size, the case the hypothesis named as the parent's weakness, barely moved. So the claimed threshold was crossed, but the hypothesis's diagnosis was only half right.

## Hurt

- per-op-time (put p99): Each pass now takes a counter down by one instead of clearing it to zero, so hot slots need up to three passes before they can be evicted. A sweep through a hit-heavy region visits many more slots per eviction. The `2 * capacity` cap is O(capacity), and it does nothing to shorten the long sweeps that set the tail.
- per-op-time (put p50): Longer sweeps are not limited to rare evictions. With counters rising to 3 and falling only one step per pass, the ring stays non-zero more of the time, so even a typical evicting put walks several slots.
- per-op-time (get p50/p99): The only change in `get` was replacing an unconditional byte store with a compare-and-increment. A slowdown this large is not explained by the instruction cost. The likely causes are a data-dependent branch that mispredicts as counters vary, and cache lines being evicted by the much longer put sweeps interleaved in the benchmark. Treat this as a measurement worth re-checking, not as an intrinsic cost of GCLOCK's get.

## Takeaway

Putting a saturating counter in the padding byte did keep `Node<u64,u64>` at 24 bytes and bought a small hit-rate gain. Combined with decrement-by-one aging, though, sweep length is multiplied by `FREQ_MAX`, and that wiped out the parent's latency advantage on every op. The hit-rate gain came mostly from hm_0, not web_0. Frequency alone is not what web_0 at the large size is missing. A follow-up should decouple aging cost from eviction, for example by aging the counters in bulk or halving them per pass, or by using a much smaller sweep cap, and not raise `FREQ_MAX` further.
