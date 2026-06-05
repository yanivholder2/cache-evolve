# Variant v003 Insight

## Helped

- hit-rate: doubling the promotion credit from +1 to +2 gave stash-promoted items enough of a frequency head start to survive one additional eviction cycle in primary before being re-evicted, converting some immediate stash round-trips into genuine secondary hits.

## Hurt

*(no dimensions hurt)*

## Takeaway

The +1 credit used by the parent was marginally too weak: promoted items arrived in primary with just enough counter weight to tie with other low-frequency occupants, making them likely victims on the very next cold-burst eviction. The +2 credit clears that tie, giving promoted items one extra cycle of protection. The improvement is real but small, which suggests that promotion credit is a fine-tuning lever rather than a structural one — the stash mechanism itself does the heavy lifting, and credit size controls only how long a promoted item holds its position before becoming a candidate again.
