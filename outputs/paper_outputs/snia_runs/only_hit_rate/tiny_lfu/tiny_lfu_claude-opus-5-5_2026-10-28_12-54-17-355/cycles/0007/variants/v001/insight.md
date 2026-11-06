# Variant v001 Insight

## Helped

- hit-rate (hm_0, small size): went up only by a noise-level amount. At 16384 entries the period of 10×capacity fires a halving many times over the trace. So stale probation/protected residents do lose some of their accumulated lead, and a few recency-driven hm_0 keys win admission. But the effect is far too small to count as the "real recency weight" the hypothesis expected.

## Hurt

- hit-rate (web_0, small size): this is the clearest loss. Halving keeps the order of hot keys, but it collapses the cold tail: counts of 1 truncate to 0, and 2–3 become 1. The probation-head sample from `sampled_main_victim` is made mostly of these cold keys, so right after each reset its minimum drops to about 0. Meanwhile a window candidate still gets the unchanged miss+`put` double count. The strict `cand_freq > victim_freq` test is therefore passed more easily after each reset, and admission churn rises. On a frequency-skewed trace, that churn displaces moderately useful entries.
- hit-rate (hm_0, large size): a small regression. At 131072 entries the period is so long that only a handful of halvings occur. Each one is a sudden, global drop in the frequency filter, not a gradual recency signal: it briefly lets low-value newcomers into a large, well-populated main.
- hit-rate (web_0, large size): a slight dip that stays inside the hypothesis's 0.001 guard. It comes from the same few rare resets. That confirms rare halving costs little here, but it does not buy anything either.

## Takeaway

Plain periodic halving, added to the coldest-of-4 SLRU lineage, is net-negative. The reason is that this lineage's admission edge is already given to newcomers by the miss+`put` double count, which the hypothesis deliberately left untouched. Halving does not reorder hot keys. It mainly zeroes the cold probation tail that the sampled victim is drawn from, so it amplifies the newcomer bias instead of adding recency. So "the unaged sketch is the root cause" is not supported in isolation. Aging is only likely to help if it is paired with removing the double increment, or with a tie/threshold rule that stops post-reset zeros from becoming free admissions.
