# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Widening the frequency-aware tail scan from four to eight entries made the least-hit victim search reach fresher, lower-count entries, so eviction overrode recency for many more candidates and evicted items the trace goes on to re-reference.

## Takeaway

The hypothesis predicted that a wider window would extend the four-entry gain, and it reversed it: the frequency bias is only useful when the victim search stays close to the LRU end, and past that depth it trades away recency it cannot afford to lose on workload-a. The parent's open question about window scaling is now answered negatively for this trace, so further window growth should not be pursued without a change to how ties or recency are weighted.
