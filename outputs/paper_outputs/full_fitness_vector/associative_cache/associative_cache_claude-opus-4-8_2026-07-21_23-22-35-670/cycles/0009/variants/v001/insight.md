# Variant v001 Insight

## Helped

- hit-rate: Swapping SipHash for the FxHash-style rotate/multiply fold plus a two-round Fibonacci avalanche kept bucket distribution approximately uniform, so hit-rate held (even nudged up a hair) despite dropping the keyed permutation.

## Hurt

- per-op-time: The claimed latency win from removing the SipHash setup did not appear — `get` p99 and `put` p50 each landed a bucket higher than the parent, and the p50 summary stayed flat, so the change traded away nothing but bought nothing measurable either.

## Takeaway

Replacing SipHash with a one-multiply integer mixer (with `write_u64`/`write_usize` fast paths for the eval's word-sized keys) is correct and distribution-safe, but the tens-of-nanosecond SipHash tax the hypothesis targeted is below the eval timer's ~100ns granularity — so a real per-op saving can't be resolved here, and the coarse buckets even let noise push tails the wrong way. Hashing is not the observable latency lever on this harness; per-op-time is gated by timer resolution, not hash cost.
