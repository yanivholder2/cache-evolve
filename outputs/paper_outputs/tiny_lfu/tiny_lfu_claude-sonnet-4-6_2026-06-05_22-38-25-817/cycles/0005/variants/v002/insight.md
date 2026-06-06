# Variant v002 Insight

## Helped

## Hurt

## Takeaway

Doubling `EVICTION_SCAN_K` from 4 to 8 produced no measurable change in hit rate on this Zipfian workload. The parent insight already observed that the minimum-frequency probationary candidate tends to appear within the first few LRU-tail positions; the k=4 window is wide enough to reach a cold victim in virtually every eviction decision, so scanning four additional entries encounters entries of similarly low frequency and changes no outcomes. The result confirms that the scan-window bottleneck is not the limiting factor here — further gains will require changing what is scanned or how the victim is compared, not simply how many candidates are inspected.
