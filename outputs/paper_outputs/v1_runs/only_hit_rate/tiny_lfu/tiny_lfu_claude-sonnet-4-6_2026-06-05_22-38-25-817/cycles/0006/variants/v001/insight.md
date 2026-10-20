# Variant v001 Insight

## Helped

## Hurt

## Takeaway

Applying the k=4 minimum-frequency scan to protected-segment demotion — mirroring the probationary victim scan that proved valuable in cycle 0004 — produced zero movement in hit-rate on this workload. The likely explanation is that items admitted to protected have already cleared the PROMO_THRESHOLD=8 gate (or the `was_protected` bypass, which marks items that were already hot), so the LRU end of protected holds items with narrow frequency spread; the scan rarely finds a meaningfully colder victim than `prot_head` itself. The asymmetry between the two segments is instructive: probationary's LRU end mixes cold newcomers with warm survivors, creating heterogeneity the k=4 scan can exploit, whereas protected's LRU end is already pre-filtered to be relatively frequency-homogeneous.
