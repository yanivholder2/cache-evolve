# Variant v001 Insight

## Helped

## Hurt

## Takeaway

Eagerly promoting a small-queue entry to main the instant its `freq` hits
`MAX_FREQ` (rather than waiting for `evict_small`) left hit-rate exactly
unchanged on workload-a/2048 — a pure timing lever with no measurable
effect. As the hypothesis itself foresaw, a key reaching `freq == 3` in
small already clears the `freq > 1` promotion test, so it lands in main at
eviction anyway; advancing only the *moment* of promotion neither admits
new keys nor changes net residency, and on this trace the few keys that
saturate `freq` in the tiny `capacity/24` small queue would have been
promoted before their next reuse regardless. The earlier-freed admission
slot did not surface any extra reuse, confirming this timing window is not
a live lever on this workload.
