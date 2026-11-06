# Variant v002 Insight

## Helped

- hit-rate (hm_0/16K, summary): hm_0/16K gained much more than any other point and accounts for nearly all of the summary gain. At 16K the ghost is full width (`capacity.next_power_of_two()`), so it keeps entries long enough for a key to go through ghost hit, SC|RET stay, eviction without a hit, and a second ghost hit. On hm_0, keys that do this have a stable reuse distance a little above the cache size, and sending them to the protected MRU keeps them through the next probation churn, where the parent's single reprieve did not.
- hit-rate (hm_0/131K): The gain here is marginal. At 131K the ghost is only capacity/4 wide, so it seldom remembers a key long enough for a second ghost hit, and few keys get escalated.
- per-op-time (get p50, the summary; get p99): `on_hit` changed in only two ways: the AND immediate is now `!(SC_BIT | RET_BIT)`, and the demotion branch moved into an `#[inline] demote_overflow`. Neither adds work, so the drop most likely comes from codegen or layout changes, or from noise, not from a real speedup. Treat it as "get was not harmed" rather than as a gain.
- per-op-time (put p50): The ghost table is the same size as the parent's. The new work in the `Vacant` arm is a mask, a bit read and a branch, which is too little to slow the common put.

## Hurt

- hit-rate (web_0/16K, web_0/131K): Both web_0 points slipped slightly. This is the opposite of what the hypothesis predicted for web_0/131K. Taking a bit for `repeat` leaves 7-bit tags, which roughly doubles collisions. A spurious match can now also pick up a stale repeat bit, which escalates an unrelated key into protected and pushes a real protected LRU down via `demote_overflow`. On web_0 there are too few true repeat returners to offset this, and at 131K the capacity/4 ghost window is too short for many to qualify.
- per-op-time (put p99): The tail rose slightly. An escalated put now does a protected `push_back` and possibly a demotion (`unlink` + `push_back`) on top of the eviction, so the rare escalation path is longer than before.

## Takeaway

Escalating keys only after a second ghost hit works when the ghost is wide enough to see that second return. That holds at 16K, where hm_0 benefited a lot. At 131K, the quarter-width ghost almost never catches a second return, so the change does little on hm_0 and slightly hurts web_0, where the 7-bit tag collisions cost more than the few escalations gain. To hit the original web_0/131K target, a follow-up would need to keep the repeat signal without giving up a tag bit, or widen the ghost window above `GHOST_MAX_CAPACITY`. Changing the escalation rule alone will not get there.
