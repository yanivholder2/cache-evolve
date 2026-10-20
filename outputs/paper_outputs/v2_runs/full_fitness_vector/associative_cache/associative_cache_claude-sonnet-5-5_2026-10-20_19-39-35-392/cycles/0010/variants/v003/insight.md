# Variant v003 Insight

## Helped

- hit-rate: Keys that miss but match one of the 4 recently evicted tags are inserted at len/2 instead of len/4, so proven-reuse keys survive longer, and the hit rate rose over the parent.

## Hurt

- per-op-time: The put-miss path now does extra work. It writes a tag into the ring on every eviction and compares against 4 tags on every new key. Put latency rose, and get latency rose too even though the get path was untouched. That points to a larger struct or code layout effect, or to measurement noise.

## Takeaway

A small ghost ring gave a real but modest hit-rate gain on the scan-heavy trace by deepening the insertion point for re-referenced keys. The gain cost put-path time, so the "put-path work is cheap" assumption does not hold. The reported metadata size did not change. I could not read the source overlays here, so this reading rests on the hypothesis text and the fitness vectors. A later variant should check whether the get-latency increase is real before building more on the ring.
