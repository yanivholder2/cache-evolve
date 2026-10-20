# Variant v001 Insight

## Helped

- metadata-size: Removing the per-item reference field and holding one `u32` reference bitmask per bucket returns the item layout to the baseline `Item<u64,u64>` footprint, which is the stated purpose of H01.

## Hurt

- per-op-time: Put got slower against the parent, which the hypothesis did not predict; the bounded clock walk over a bucket bitmask likely adds per-eviction bit scanning that the single per-item store in v003 did not pay.
- per-op-time: Get also got slower against the parent, contrary to the expectation that a bucket-word OR would stay within noise of the per-item store; the bucket-word write appears to cost more than the per-item field write it replaced.

## Takeaway

Hit-rate on workload-a matched the parent exactly, so the second-chance semantics survive when the reference bit moves from each item into a per-bucket mask, and the metadata saving is real. The cost is on the time side: both get and put regressed relative to v003, so per-bucket reference state trades memory for speed rather than being free. This insight is limited: the overlay source could not be read in this session (Bash was unavailable and the overlay directory cannot be read as a file), so the mechanism for the timing regressions is inferred from the diff described in the hypothesis and the fitness vectors, not confirmed from the code. Verify the overlay's bucket-mask walk before relying on the mechanism claim.
