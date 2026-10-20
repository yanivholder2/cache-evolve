# Variant v002 Insight

## Helped

- per-op-time: Get and put latencies at both p50 and p99 came in slightly below the parent, which is plausible because the sketch footprint roughly halved and fewer sketch cells are touched per operation; H02 did not target timing, so this is an incidental effect from a single run and should be treated as provisional.
- metadata-size: The count-min sketch allocation dropped to half the parent's bytes, so the sketch is smaller than H02 specified; the headline metadata summary is unchanged because it tracks `Node<u64,u64>`, which stayed at 32 bytes as required.

## Hurt

- hit-rate: Hit-rate on workload-a at size 2048 fell slightly against the parent, so the u8 saturating-counter change did not reduce collisions enough to pay for the smaller sketch. Since the sketch shrank instead of keeping its size while doubling columns, the likeliest cause is fewer effective counters per row, not the intended collision reduction.

## Takeaway

Narrowing the sketch counters to u8 while doubling columns was meant to keep the sketch at 65536 bytes, but the measured sketch is 32768 bytes, so the column doubling most likely did not land as specified and the variant tested a smaller sketch. The hit-rate drop alongside the latency gain suggests the sketch's collision-reduction benefit depends on its total size, not just counter width. The source could not be inspected in this review (no file-search or shell access), so the row and column counts remain unverified and should be checked before the next attempt.
