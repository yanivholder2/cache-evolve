# Variant v001 Insight

## Helped

- hit-rate: In `evict_main`, a hand pass now sets the survivor's `freq` to 0 instead of decrementing it. Under the parent, a key that saturated at `MAX_FREQ = 7` could stay resident for up to seven hand laps after its last hit. Now it gets exactly one lap, and with no new hit it becomes the victim on the next pass. Once-hot keys that have gone stale leave main sooner. That frees slots for freq-0 ghost re-admissions, which sit near the head behind the hand. The gain is small but in the direction H01 predicted, and it lands just outside the ±0.001 band the hypothesis set as the threshold for an informative result. It can be attributed to the retention schedule alone, for two reasons. Small-queue graduation (`freq > 1`), the saturating counter in `get()`, re-admission, ghosting and the 5% split are all unchanged. And the updated `RefS3Fifo` reference still checks main order, per-entry freq and hand position exactly.

## Hurt

(No measured dimension regressed. Hit-rate is the only dimension in the eval. One effect goes unmeasured: a sweep can no longer spend several laps draining multi-credit survivors. It now finds a victim within at most about one lap. Worst-case `evict_main` work should therefore be lower than the parent's, but per-op time is not evaluated.)

## Takeaway

Under SIEVE in-place retention, multi-lap credit does not pay for itself on workload-a. Keeping a single visited bit's worth of retention in main beats the cycle-7 decrement-by-one with cap 7. This flips the cap-7 verdict from earlier cycles, which was measured under FIFO reinsertion. Two things are still worth testing. One is whether a small cap or decrement (for example, a cap of 2–3 applied only to main retention) recovers anything over clear-on-pass. The other is whether clearing on pass also allows the small-graduation counter to be separated from the main retention signal.
