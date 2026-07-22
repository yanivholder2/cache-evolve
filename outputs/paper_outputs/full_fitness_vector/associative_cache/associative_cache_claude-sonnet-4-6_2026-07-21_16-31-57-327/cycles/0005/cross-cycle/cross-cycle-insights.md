## Recurring wins

- **32-byte aligned struct is a universally safe base.** Padding the compact struct to 32 bytes eliminated cache-line straddling and halved p99; every cycle that dropped it regressed. Treat this as a non-negotiable structural starting point.
- **Bounded promotion loop with early-break is the confirmed best promotion form.** A 3-swap cap with an early-break on a failed swap lets hot Zipfian keys exit after one iteration while giving mid-bucket items enough budget to converge. Single-swap was too conservative; three swaps resolved the convergence lag cleanly and landed the only cycle-0005 winner.
- **Unconditional positional promotion outperforms frequency-gated promotion on this workload.** Removing the `lfu_counter` neighbour check and always advancing keeps the tail genuinely cold without writing per-get state. The frequency gate was stranding recently-reused items below cold neighbours across multiple cycles.
- **Strict tail eviction (pure positional LRU) consistently beats frequency overlays at the eviction boundary.** Every attempt to apply a frequency signal at the eviction site (two-tail comparison, LFU counter checks) degraded hit rate relative to the plain LRU tail victim.

## Recurring failures

- **Per-get counter writes reliably raise p99 tail latency.** Across cycles 0002–0005, any variant that wrote an `lfu_counter` on the get path surfaced tail-latency pressure. The write itself is the cache-line liability, not the conditional logic around it. No frequency encoding that requires a get-path write has passed the latency bar.
- **Two-tail frequency eviction comparison degrades hit rate.** Tested in cycles 0004 and 0005; both times it overrode clean positional recency and ejected recently-promoted items ahead of genuine cold victims. Closed avenue.
- **Two-zone (hot/cold) partitioning failed definitively.** Tested in cycle 0003: the boundary shifted with every insertion, pushing boundary items into the cold zone, while the hot-zone bypass saved no real work. Not worth revisiting at these bucket sizes.
- **Swap-to-back (replace remove+push with in-place swap) is a double loss.** Failed cycle 0002 on both hit rate and latency; the linear shift is not the bottleneck, and the approximation harms ordering quality. Closed.
- **Single-swap promotion cap is too conservative.** Confirmed across cycles 0003–0004: one swap per access is insufficient for mid-bucket items to converge toward the head on a Zipfian workload, eroding the hit-rate benefit of frequency-aware ordering.

## Open questions

- **Do the 3-swap loop and the unconditional always-advance rule target the same bottleneck, or are they additive?** Cycle 0004 showed both improve hit rate independently; a clean composition experiment is still missing.
- **Can frequency information be integrated without any per-get write?** Epoch-based or eviction-triggered counter updates could retain scan-burst resilience without the cache-line write liability on the hot path. This is the most promising unexplored axis for further hit-rate gains.
- **Has the promotion budget hit diminishing returns at 3 swaps?** The jump from 1→3 was meaningful; whether 4–5 swaps continues to gain on this Zipfian concentration or plateaus has not been tested.
- **Is the per-op-time measurement floor still masking gains?** Cycles 0004–0005 show per-op-time pinned at the timer-resolution floor for write-free variants; only write-heavy get paths (cycle 0005 v003) produced a clearly observable p99 signal. Latency differentiation for low-write variants may require a higher-operation-count benchmark.

## Per-policy notes

- **LFU-style ordering:** Hit-rate advantage is real and reproducible on scan-heavy and Zipfian traces. Constant-time eviction (tail removal rather than scan) resolved the O(n) cost from cycle 0001. The remaining risk is per-get counter writes; a write-free frequency signal (sampled or epoch-based) is the natural next step.
- **Positional LRU with bounded promotion:** The confirmed winner as of cycle 0005. The 3-swap bounded loop with early-break on a failed swap is the best-known implementation. Unconditional advance (no frequency gate) is superior to gated advance on this workload. No frequency overlay at eviction time has improved upon pure positional ordering across any cycle.
- **Two-zone partitioning:** Definitively closed. The dynamic boundary defeats the intent of zone separation at small bucket sizes.
- **Swap-to-back promotion:** Definitively closed. Both the latency and hit-rate arguments against it are confirmed.
