# Variant v001 Insight

## Helped

## Hurt

## Takeaway

Increasing the decay coefficient from 0.75× to 0.8× yields no measurable improvement in hit-rate on this workload. This suggests the frequency decay mechanism has already reached a plateau of effectiveness at 0.75×—gentler decay beyond this point does not preserve frequency discrimination any further in a way that benefits eviction decisions. The workload's access patterns may be sufficiently captured by the 0.75× curve, or the bottleneck for improvement lies elsewhere (e.g., the reset interval or frequency granularity).
