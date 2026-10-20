# Variant v001 Insight

## Helped

- get p99: Asymmetric admission gates new items more strictly (requiring incoming_freq > victim_freq + 1), reducing admission churn and eviction storms that cause tail-latency spikes in gets, improving p99 by 23.5%.
- put p50: Stricter admission filtering reduces the number of new items contending for eviction, decreasing average put latency by ~6% as the cache experiences less insertion pressure during steady state.
- put p99: Worst-case put latency drops 16.7% (390 → 325 ns) because the asymmetric gate prevents the scenario where high-frequency new arrivals trigger a cascade of frequent eviction decisions and sketch lookups.

## Hurt

- hit-rate: Requiring new items to exceed victim frequency by more than 1 (vs. just exceeding it) filters out marginal arrivals that are hot enough to evict but not hot enough for the stricter threshold, reducing hit-rate by 0.096%.

## Takeaway

Composing asymmetric admission with N=3 victim selection achieved the opposite of the hypothesis prediction: latency improved substantially while hit-rate declined slightly. The stricter gate for new items—distinguishing updates (always admitted) from arrivals (higher threshold)—acts as an anti-pollution mechanism that suppresses eviction churn at the cost of missing borderline-hot entries. This reveals that the two mechanisms are not orthogonal in the expected way: v003's semantic distinction does not compound v001's frequency diversity but instead constrains admission pressure, trading recall for predictability.

