# Variant v003 Insight

Note: the variant's source files could not be read during this write-up, so
this insight is based on the hypothesis (H03), the parent insight, and the two
fitness vectors. The code-level claims below are inferred from the hypothesis
and should be checked against the overlay diff.

## Helped

- hit-rate: The bounded ghost ring of recently evicted fingerprints lets returning keys skip probation and enter the protected segment directly, which is the scan-resistance mechanism H03 predicted for the scan-heavy workload-a trace.

## Hurt

- per-op-time: The put path now hashes every new key and scans the `capacity`-length ghost ring on each miss, and it also does a `demote_prot_head()` when a ghost hit lands on a full protected segment; both add work that the baseline put path does not do, and the put tail latency rose the most.
- per-op-time: The get path was meant to be untouched, but its median latency also rose slightly; the hypothesis gives no mechanism for that, so it is more likely layout or measurement noise than a direct effect of the ghost ring.
- metadata-size: The struct footprint grew well beyond the two fields H03 adds (a `Vec` header and a `usize`), so either other fields changed or padding was introduced. The ring's `capacity * 4` bytes of heap storage is not captured by this metric at all, so the real memory cost is understated.

## Takeaway

Remembering evicted keys in a small ring and promoting ghost hits straight to protected is a real scan-resistance gain on this trace, but the scan over `capacity` and the per-miss hash move cost into the put path and the ring's memory is invisible to the struct-size metric. A follow-up should bound the ring scan (for example with a hash-indexed set or a smaller ring) and report the ring's heap size alongside the struct size.
