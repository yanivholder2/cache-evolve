# Variant v003 Insight

## Helped

- hit-rate: Choosing the eviction victim as the least-hit of the four oldest entries, using a per-entry hit counter, protects re-referenced keys near the tail that plain LRU would evict, which is the mechanism the hypothesis targeted on workload-a.

## Hurt

## Takeaway

Making eviction frequency-aware within a bounded four-entry tail window produced only a marginal hit-rate gain over the parent on workload-a at size 2048, so the signal is weak on this single trace. The change also adds a per-entry counter whose memory cost is not measured by the current fitness vector, so any retention decision should wait for a size dimension or a second trace before treating this as a real improvement.
