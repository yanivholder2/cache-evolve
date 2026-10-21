# Variant v002 Insight

## Helped

(No measured dimension improved.)

## Hurt

- hit-rate: H02 is refuted: hit-rate fell slightly below the parent, landing just under its stated threshold, which the refutation clause covers. The change is negligible in size, but it points the wrong way. The likely cause: with `record_ghost_if_unhit`, a resident that was hit and later aged out to count 0 no longer leaves a fingerprint. When it returns, it is admitted with counter 0 instead of 1 and sits under the LIP hand as the next victim. The small loss suggests these returns were worth slightly more than the never-hit slots that took their place in the 4-entry ring.

## Takeaway

On this GCLOCK/LIP line, the hand rests on the newcomer, so most evictions are already never-hit newcomers. The `HIT_FLAG` filter therefore removes only a thin slice of ghost-ring insertions. That is why it can barely move hit rate in either direction. The slice it does remove, aged-out formerly-hit residents, carries a small positive return value, so the unfiltered ring in v001 was not "diluted" in any way that hurt. Making the ring trigger more selective is close to exhausted on CLOCK. The bit-8 `HIT_FLAG` packed into `lfu_counter` works as a mechanism, but here it gives no leverage on hit rate.
