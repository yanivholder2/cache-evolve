# Variant v003 Insight

## Helped

(No dimension improved.)

- hit-rate: Flat, matching the parent exactly. This is the expected result. `if e.freq < MAX_FREQ { e.freq += 1 }` behaves the same as `(e.freq + 1).min(MAX_FREQ)`, and the new `get_freq_steps_then_saturates` test pins the 1, 2, 3 then hold sequence. So eviction order, graduation (`freq > 1`) and the adaptive split all follow the same path as the parent. Settled `small_size`: I could not find the `h01_split_report` stdout in `eval/`, so I can't quote a number. `put`, the split logic and the replay harness are untouched, and the hit-rate is bit-identical, so the settled split is the same as the parent's by construction. The next eval should save that test's `--nocapture` output.
- metadata-size: Flat, as expected. `Entry` (`{ value, freq }`, 16 bytes) and the `S3Fifo` header are unchanged, and `entry_is_16_bytes` still holds.

## Hurt

- per-op-time (`get`, the summary): `get` regressed at both p50 and p99, the opposite of what was predicted. The `get` code did change, so in principle a move here can be credited to it. But the evidence argues against crediting this one. `put` regressed by a similar or larger proportion at both percentiles, and the overlay does not change `put`, eviction, `alloc_with_key` or the ghost at all. A uniform slowdown across an unchanged path points to the run environment, not to the guarded store. On a single-threaded benchmark, a store to a line that is already in L1 costs almost nothing, so removing it was unlikely to show up at nanosecond granularity anyway. The new branch is a load, compare and conditional jump in place of a load, add, `min` and unconditional store. That is not plausibly enough extra work to explain a regression of this size.
- per-op-time (`put`): Regressed with no code change on its path. This confirms that the benchmark baseline drifted between cycle 0006 and this run. It also means the cross-cycle `get` noise flagged earlier affects `put` too.

## Takeaway

Skipping the saturated `freq` store is a correct, policy-neutral rewrite, but this run cannot show any benefit from it. The "dirty line" cost the hypothesis targeted barely exists for a single-threaded cache whose hot entries stay in L1. The measured slowdown is not trustworthy either, because unchanged `put` slowed by the same proportion. Before using `get` micro-trims as evidence, the loop needs a same-run parent baseline (benchmark parent and child back-to-back on the same machine). Without one, sub-10ns deltas on a two-line change like this cannot be separated from drift. The code is safe to keep or revert. Neither choice should be justified by this cycle's per-op-time.
