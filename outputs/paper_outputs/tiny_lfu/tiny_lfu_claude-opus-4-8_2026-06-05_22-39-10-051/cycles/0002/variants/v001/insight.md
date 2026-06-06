# Variant v001 Insight

## Helped

- (none): Broadening the victim sample produced no measured gain on any dimension.

## Hurt

- hit-rate: Raising `SAMPLE` from 5 to 16 widened the frequency-ranked scan deep
  enough into the LRU list that the lowest-`estimate` victim is often a
  recently-admitted node whose frequency hasn't accumulated yet, so eviction
  discards recency-warm keys that the shallower sample left protected.

## Takeaway

The frequency-ranked eviction gain does not scale with sample breadth — it
plateaus and then reverses. At SAMPLE=16 pure lowest-frequency selection
overrides the recency signal encoded in LRU position, evicting newly-admitted
items before they prove cold, which refutes H01's premise that a larger sample
keeps lifting hit-rate. The slim SAMPLE=5 win was near the sweet spot, not the
start of a trend.
