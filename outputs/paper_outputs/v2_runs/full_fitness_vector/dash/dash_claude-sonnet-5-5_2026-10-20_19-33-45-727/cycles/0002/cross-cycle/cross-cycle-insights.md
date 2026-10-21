## Recurring wins

- Admission-side changes (second-hit gate) pay off more reliably than insert-position changes; they skip remove-and-promote work on first hits.
- Metadata-free implementation matters: bit-packing the admission flag into spare bits kept the hit-rate benefit, restored baseline item size, and improved latency (cycle 2 v001 is the current best base).
- Scan resistance (keeping one-hit-wonders out of hot tiers) is the theme behind every gain so far.

## Recurring failures

- Stacking position levers (cold first admission, cold demotion, flag reset) on top of admission or uniform MRU: levers overlap rather than add, giving weak or mixed results with a hit-rate/latency trade-off.
- Position-only tweaks: small gains, no metadata or work change, latency shifts likely noise. Looks near exhausted as a lever.
- Unmeasured latency claims: several latency changes were explained only by inference, so they are not trustworthy.

## Open questions

- Can the spare bits in the packed flag hold a small hit counter or an age/recency bit, giving richer admission signals at zero metadata cost?
- Does the packed filter keep its latency gain with a milder demotion penalty (cold demotion without flag reset)?
- Are the small hit-rate gains (e.g. cold first admission on uniform MRU) real or within trace noise? Needs latency/hit-rate re-measurement.
- Does uniform MRU's gain come from promotion or from demotion cascades (never separated)?
- Results come from the recency-skewed LIRS trace only; do they generalize to other traces?

## Per-policy notes

- DASH segment promotion policy (the only policy probed so far): base for future work is the bit-packed second-hit filter at baseline item size. Prefer richer free-metadata admission signals over more insert-position variants.
