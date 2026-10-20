# Variant v001 Insight

## Helped

- hit-rate: Raising `protected_cap` from `(capacity * 4) / 5` to `(capacity * 9) / 10` gives twice-accessed hot keys longer residency before demotion, and on this reuse-heavy LIRS trace that extra protected tenure converts more repeat accesses into hits.

## Hurt

## Takeaway

Pushing the SLRU split further toward protected (90/10) beats the parent's 80/20 on this trace: shrinking the probationary segment to 205 slots still leaves a non-empty victim pool for probationary-first eviction, so one-hit-wonders keep draining while genuinely hot keys gain residency. The lever rewards more protected room, not less — confirming the direction cycle 0002 left open, with no other change to credit since the only diff is the ratio constant.
