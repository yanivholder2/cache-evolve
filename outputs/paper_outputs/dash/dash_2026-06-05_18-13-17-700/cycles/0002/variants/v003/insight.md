# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Forcing the normal (protected) tier onto a fixed LFU policy, while leaving stash buckets on the seeded policy, slightly lowered hit-rate — LFU's frequency-counter ranking evicted keys that the baseline's recency-based eviction would have kept resident on this trace.

## Takeaway

On the LIRS workload-a trace, swapping only the protected tier's eviction policy to LFU does not beat baseline recency eviction; the trace's skew is already well-served by promote-on-hit recency, and LFU's slow-to-adapt counters retain stale-but-frequent keys at the expense of the live working set. Changing *how* the protected tier ranks residents — rather than *how much* room it gets — is not a productive lever here.
