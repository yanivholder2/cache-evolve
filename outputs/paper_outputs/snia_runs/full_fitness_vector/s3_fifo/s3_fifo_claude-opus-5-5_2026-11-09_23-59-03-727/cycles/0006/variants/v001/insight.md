# Variant v001 Insight

## Helped

- per-op-time: `put` p99 dropped slightly. The only code change is the divisor in `try_new`, which runs once and is off the hot path, so this is almost certainly run-to-run noise. A smaller `main_size` could in principle shorten some `evict_main()` sweeps a little, but nothing in the result separates that from noise.

## Hurt

- hit-rate: All four trace/size cells fell slightly, including web_0 at 131072, the cell the hypothesis targeted. Moving about 8% of capacity from main to small does not help web_0 collect more second hits. It does cost main residency, and the ghost bound tied to `main_size` shrinks with it. As a result, keys that have already proven reuse get fewer `MAX_FREQ = 7` second-chance sweeps, and fewer ghosted main victims are remembered for readmission.
- per-op-time: `get` p50/p99 and `put` p50 rose by about 1 ns, which moved the summary. `get()` is byte-for-byte unchanged, so this is noise, not a cost of the new split.

(metadata-size did not move: the struct layout is unchanged, and `Entry<u64,u64>` stays 24 bytes.)

## Takeaway

The 1/4 small-queue share is not too short for web_0. Cycle 5's drop at 10% and this drop at 1/3 both fall from the parent, which points to a local optimum near 1/4. web_0's misses are therefore not caused by probation length. Since a bigger small queue costs main capacity and ghost capacity on every trace, future work on the small/main split should look elsewhere, for example the `freq > 1` graduation threshold or decoupling the ghost bound from `main_size`, rather than resizing the small queue again.
