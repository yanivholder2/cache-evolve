# Variant v002 Insight

## Helped

- per-op-time: put latency dropped. The swap-based insertion mechanism is unchanged, so this is probably measurement noise or a small layout effect, not a result of the len/3 target. Get latency did not move, as expected since the get path was untouched.

## Hurt

- hit-rate: Moving the insertion index from about len/4 to len/3 lowered hit rate on workload-a. Entries inserted closer to the protected end are evicted later, so one-touch entries linger and push out entries that would be reused. That outweighs any benefit from giving new entries more time to prove reuse.

## Takeaway

On this trace, the len/4 insertion point beat len/3, so the optimum is at or below len/4 and not between len/4 and the MRU side. The change added no metadata and no get-path branches, so metadata-size and get time were unaffected. Insertion depth is therefore a pure hit-rate knob that costs nothing else. I could not read the overlay source in this session, so this rests on the hypothesis text and the two fitness vectors.
