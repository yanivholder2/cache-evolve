# Variant v002 Insight

## Helped

- per-op-time: Nothing meaningful moved. The counter increments only in the sketch's `increment` path and `get` is untouched, so the amortized halving sweep (once per 10×capacity accesses) cost almost nothing at p50/p99.

## Hurt

- hit-rate: Halving the sketch counters every ~10×capacity accesses made hit-rate worse than parent v003 on workload-a. The trace's frequencies are stable enough that decaying them discards useful history, and the admission comparison loses discriminating power right after each sweep.
- metadata-size: The new `u32` counter (plus alignment padding) grew the cache struct by a few bytes. The sketch and per-node size are unchanged, so the cost is small but strictly negative.

## Takeaway

Even with a period 10× longer than the earlier aging attempt, periodic halving does not pay off on this trace: it shows no frequency drift for aging to exploit. Aging is a cost without a benefit here, so future hit-rate gains should come from elsewhere, such as admission or sketch accuracy, rather than from decay.
