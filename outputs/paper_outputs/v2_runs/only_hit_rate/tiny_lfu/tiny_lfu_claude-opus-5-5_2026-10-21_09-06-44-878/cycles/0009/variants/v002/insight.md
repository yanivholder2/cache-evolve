# Variant v002 Insight

## Helped

## Hurt

## Takeaway

Cutting `VICTIM_SAMPLE` from 8 to 3 (the only logic change; `select_victim` and the admission predicate are otherwise identical to v001) left workload-a's hit-rate essentially flat. The tiny dip is within noise, so the 1–8 sampling depth is not where this cache's misses come from. A shallower scan has two effects that work against each other. It spares recently admitted, low-count probation residents, as the hypothesis intended. But the minimum over 3 entries is usually higher than the minimum over 8, so the victim estimate rises. Under the strict `incoming_freq > victim_freq` gate, fewer newcomers get admitted. The gain in residency for young keys is roughly cancelled by the loss in admissions. Sampling depth only reshuffles which low-count entry loses the contest; it does nothing about the non-aging u32 sketch that makes young keys look cold in the first place. The next lever should change the estimates themselves, for example by aging or halving the counters, rather than how many probation entries the scan reads.
