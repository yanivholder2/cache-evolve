# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Gating first-touch keys behind a doorkeeper bit vector means their first access never increments the count-min sketch, and the `+1` added back in `estimate()` only approximates that count, so admission comparisons against the LRU victim shift slightly on workload-a at size 2048.

## Takeaway

Filtering one-touch keys out of the frequency sketch's update path, while keeping sketch counts intact across the doorkeeper reset, did not help admission on this workload; the doorkeeper's `+1` correction does not fully replace the sketch increment it withholds. The result is a small hit-rate loss, so the doorkeeper-as-update-filter family is not worth pursuing in this form without a different estimate correction.
