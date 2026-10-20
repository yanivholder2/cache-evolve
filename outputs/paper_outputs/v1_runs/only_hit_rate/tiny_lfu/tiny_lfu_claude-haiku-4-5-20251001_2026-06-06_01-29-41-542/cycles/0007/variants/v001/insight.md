# Variant v001 Insight

## Helped

(none)

## Hurt

(none)

## Takeaway

Adding burstiness tracking to victim selection—which computes a per-item clustering factor based on accesses within a sliding window—produced no change in hit-rate on workload-a. Despite the theoretical motivation that access burstiness captures temporal clustering independent of frequency and age, the existing frequency + age scoring with a denominator constant of 2.0 was already sufficient for this workload, or the trace exhibits no strong access burstiness patterns that distinguish from the frequency signal.
