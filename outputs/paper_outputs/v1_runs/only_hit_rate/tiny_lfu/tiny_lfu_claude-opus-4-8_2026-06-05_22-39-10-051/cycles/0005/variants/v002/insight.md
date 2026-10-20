# Variant v002 Insight

## Helped

- hit-rate: Gating the overflow promotion on a `sketch.estimate` contest between the promotee and `prot_head` stops a single-hit (recency-only) key from displacing a frequency-hotter protected resident, so the protected tier now holds keys that are actually re-referenced more often — a marginal hit-rate gain over the parent's unconditional demote.

## Hurt

- (none observed): hit-rate is the only dimension and it did not regress; the extra two `estimate` lookups and the `push_back_prob` fallback path carry no measured cost here.

## Takeaway

On workload-a at size 2048, making protected-tier admission a frequency contest rather than an unconditional promote is directionally right but barely moves the needle: the contest only ever fires on protected overflow, and on this trace the recency order of the protected segment already correlates closely with sketch frequency, so very few promotions are actually reversed by the gate. The mechanism correctly targets the "head hides a much-hotter key" case the hypothesis named, but that case is rare enough on this workload that the win is a hair.
