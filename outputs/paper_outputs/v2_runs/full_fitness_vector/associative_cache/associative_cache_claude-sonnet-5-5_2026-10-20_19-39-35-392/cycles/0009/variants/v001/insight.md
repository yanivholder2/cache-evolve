# Variant v001 Insight

## Helped

- metadata-size: Held at the parent's level. The miss-pressure counter fits in existing padding, so the no-growth constraint was met. This is a neutral result, not an improvement.

## Hurt

- hit-rate: Slightly below the parent. Choosing between len/4 and len/2 insertion from a victim-position heuristic on the put-miss path is a noisier signal than the parent's fixed len/4 insertion. It likely lets scan-like keys enter mid-bucket too often, and it has no hit-side feedback.
- per-op-time: Both get and put latency rose, and put rose more. This is the opposite of the hypothesis's goal. Leaving the get path untouched did not keep it fast. The extra counter update and the data-dependent branch on every miss add put-side cost. The get-side slowdown likely comes from the changed bucket layout or insertion positions, or from measurement noise. I could not inspect the source diff, so the mechanism is unconfirmed.

## Takeaway

Moving the adaptivity signal off the hit path and onto the put-miss path did not make it free. The put-side counter and branch cost time, and the victim-position proxy gave up some hit rate against the parent's static len/4 insertion. A fixed insertion point was the better trade here, and any adaptive scheme would need a clearly more informative signal than this one to beat it.
