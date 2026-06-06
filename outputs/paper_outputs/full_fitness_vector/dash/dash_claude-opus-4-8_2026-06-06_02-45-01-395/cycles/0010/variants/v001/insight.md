# Variant v001 Insight

## Helped

- (none) — the change removed the second-order cascade's `core.put_at(Slot::Stash, …)`, but no dimension improved; the predicted put-p99 and get-p50 wins did not appear.

## Hurt

- hit-rate: Dropping the second-order cascade victim (and clearing its `target_heat`) instead of re-homing it in the stash sheds exactly the handful of rescued residents the hypothesis flagged as the trade — those keys now leave the cache, so the hit rate fell.
- per-op-time: Every op moved the wrong way — get and put p50 and p99 all rose versus the parent — even though the edit only deletes work on the rare foreign-resident-retry-overflow path, so the latency change cannot be the deleted `put_at` paying off.

## Takeaway

Decoupling the stash by dropping the terminal cascade victim did not buy the latency-vs-residency trade the hypothesis modeled: it paid the predicted hit-rate cost yet delivered no tail win, and the uniform across-the-board latency rise on a path that fires only for second-order conflicts looks like run variance rather than an effect of removing one stash insert. Shedding the victim outright is strictly worse here than the parent's stash re-insert — the residency it concedes is real while the put-path saving it was supposed to unlock is not observable.
