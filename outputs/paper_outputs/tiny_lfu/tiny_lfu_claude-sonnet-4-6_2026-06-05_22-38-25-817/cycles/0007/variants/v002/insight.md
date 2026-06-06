# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Lowering `PROMO_THRESHOLD` from 8 to 4 admitted medium-frequency items into protected more aggressively, but the extra promotions introduced marginal churn in the protected segment without netting additional hits on this Zipfian workload.

## Takeaway

With conservative CMS update already in place, `PROMO_THRESHOLD=8` turns out to be the right calibration: items that genuinely belong in protected accumulate true counts well above 4, so dropping the gate to 4 mainly promoted second-tier Zipfian items that cycle in and out of protected rather than stay there. The hypothesis that conservative update deflates estimates enough to strand medium-hot items below 8 did not hold in practice — the threshold needed no downward adjustment to recapture those items.
