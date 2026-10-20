# Variant v003 Insight

## Helped

- hit-rate: Newly inserted keys are linked at a midpoint marker three-eighths of the way from the LRU end instead of at the MRU head, so one-time keys in the workload-a scan stream no longer displace the hot head as quickly; this is the mechanism H03 predicted and the only dimension this variant reports.

## Hurt

## Takeaway

Placing fills at a mid-recency point, rather than at MRU or LRU, is a useful lever on workload-a at size 2048, which is the axis cycles 0005–0007 left ambiguous for tail insertion. The variant's eval records only hit-rate, so the metadata-size and per-op-time effects of the single marker are unmeasured here. The H03 engagement check (whether insertions landed at the marker) is not in the eval output, so the gain is consistent with the midpoint mechanism but not yet confirmed as caused by it.
