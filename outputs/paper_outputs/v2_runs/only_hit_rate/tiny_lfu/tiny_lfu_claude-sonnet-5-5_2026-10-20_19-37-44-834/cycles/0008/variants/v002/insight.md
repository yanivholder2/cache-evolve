# Variant v002 Insight

## Helped

- No reported dimension improved. The only dimension is hit-rate, and it did not beat the parent.

## Hurt

- hit-rate: Halving every sketch counter after about 32x capacity increments slightly lowered hit-rate. Even at this long period, the decay removed frequency history that the admission duel was using to separate hot keys from one-hit wonders. The wide u8 sketch from v001 evidently had too few saturated or stale counters for decay to help. I could not read the overlay source in this session, so this rests on the hypothesis text and the fitness vectors.

## Takeaway

A never-decaying sketch does not appear to be the limiter on workload-a at size 2048. Adding periodic halving, even at a gentler period than the earlier failed aging variants, still cost a little hit-rate. That points to aging being a poor lever here, so the next gains probably lie in the admission rule or window sizing.
