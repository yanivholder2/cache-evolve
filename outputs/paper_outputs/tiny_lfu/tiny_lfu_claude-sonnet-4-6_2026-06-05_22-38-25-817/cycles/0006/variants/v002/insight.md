# Variant v002 Insight

## Helped

- hit-rate: Switching `CountMinSketch::increment()` to conservative update — incrementing only the rows already at the current minimum — reduced collision-inflated frequency estimates, causing both the k=4 victim scan and the PROMO_THRESHOLD gate to act on sharper signal and make fewer wrong eviction/promotion decisions.

## Hurt

## Takeaway

Conservative update in the CMS produced a clear, positive improvement on this Zipfian workload: sketch noise was still a meaningful drag even after the k=4 scan and PROMO_THRESHOLD were in place. The gain is not from any structural cache policy change but purely from the quality of frequency discrimination — tighter estimates let the existing machinery (victim selection and the ≥ 8 promotion gate) pick the right items more often, confirming that reducing overcounting without changing sketch size is a net win at this capacity.
