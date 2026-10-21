# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Admitting the newcomer when it ties the sampled victim at estimate >= 2 (`TIE_ADMIT_MIN_FREQ`) made the cache drop older residents for keys that were no more frequent. On workload-a this replaced useful residents with newcomers that did not pay off. The threshold of 2 was meant to filter out one-hit wonders, but it probably does not. Both `get` and `put` increment the sketch, so a first-time key replayed as a get-miss followed by a put already shows estimate 2 at the contest. The new inline test `warm_tie_at_threshold_admits_newcomer` builds exactly this get-then-put case. Count-min overestimation raises cold estimates further. The victim from the 8 oldest LRU entries is usually also a seen-once key at estimate 2. So most "warm ties" are probably two cold keys swapping places, which adds churn and loses the older resident's chance of a reuse hit. This hypothesis predicted a gain, and the result refutes it.

## Takeaway

Breaking frequency ties in favor of recency hurts hit-rate on workload-a at size 2048. Keep the victim on a tie, so strict `>` was not the bottleneck that cycles 0001–0002 suspected. A fixed estimate threshold cannot tell one-hit wonders apart when every miss bumps the sketch twice (get, then put) and the sketch never ages. Before using the absolute estimate value as a signal again, define "warm" relative to how the replay increments the sketch, or make the estimate meaningful through aging or doorkeeper-style deduplication.
