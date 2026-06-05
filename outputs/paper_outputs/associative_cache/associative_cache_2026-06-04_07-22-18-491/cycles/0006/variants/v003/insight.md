# Variant v003 Insight

## Helped

## Hurt

## Takeaway

Switching `ClassicLRU::eviction_candidate_index` from position order to LFU-weighted selection produced no change in hit rate on top of the cycle/0005 probe zone. Once the probe filter gates entry to primary, every resident carries at least one confirmed access (`lfu_counter >= 1` from promotion), and the frequency distribution among primary items is already uniform enough that discriminating between them does not affect which items stay. The probe's admission filter is the decisive mechanism here; the choice of which confirmed primary item to displace is not a bottleneck on this workload.
