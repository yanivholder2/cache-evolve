# Variant v001 Insight

## Helped

- per-op-time: Get p50 dipped only by a hair, far less than the hypothesis predicted. If the `rposition` scan from the MRU end reached hit keys sooner, the gain is close to run-to-run noise. Put p99 also eased, which fits the existing-key check in `put` now reaching recently promoted keys sooner.

## Hurt

- per-op-time: Get p99 and put p50 both got worse. Neither path should have been slowed: misses still scan the whole bucket in either direction, and new-key puts still pay the same quarter-depth `Vec::insert` memmove. The likeliest cause is that a backward scan is a little less friendly to the hardware prefetcher and to auto-vectorization than a forward `position` scan. Noise of this size is also possible, so this is a weak signal of a small cost on scan-bound paths, not proof of a regression.

## Takeaway

Reversing the bucket scan under LIP/BIP kept every hit-rate cell and the metadata size exactly the same as the parent, so the invariant held. But it did not deliver the get-p50 gain it was built for. That undercuts the claim that most hits land in the last few slots. Quarter-depth insertion leaves many first-time hits sitting near the front, and the benchmark's get mix may not be dominated by hit-scan length at all. Lookup cost here seems set by doing a linear compare over the whole bucket, not by which direction the scan starts from. Further get-latency work should cut the number of key compares per lookup (for example, a compact tag/fingerprint pre-filter that can be SIMD-scanned) rather than reorder the same scan.
