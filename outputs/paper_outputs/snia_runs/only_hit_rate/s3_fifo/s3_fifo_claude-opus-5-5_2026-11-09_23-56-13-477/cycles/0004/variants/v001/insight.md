# Variant v001 Insight

## Helped

- hit-rate (web_0 @ 16384): This is the one cell that clearly improved. Ghost returnees that were never reused in small (ghost freq 0) no longer go straight into a full main and force a main eviction. Under heavy main pressure on web_0, those weak keys were displacing residents that would have hit again. Sending them back through small with freq 1 lets them earn a place with one more reuse.

## Hurt

- hit-rate (hm_0 @ 16384): A small loss. On hm_0, some freq-0 ghost returnees are real reuses. Routing them into small delays them in three ways: they take one of the few small slots, they push another small entry into the ghost, and they reach main only after one more hit. If that hit doesn't come before small turns over, they are ghosted again, and the access that would have been a main hit is lost.
- hit-rate (web_0 @ 131072, hm_0 @ 131072): Essentially flat with a slight downward drift. At the larger size, the ghost covers a longer reuse window and main pressure per admission is lower. So whether the freq-0 ghost path goes to main or to small rarely changes which key is evicted.

## Takeaway

Using the freq a key carried into the ghost as the gate for the ghost→main fast path partly confirms the hypothesis. It helps where admitting into main is expensive (web_0 at the small size) and costs a little where ghost returns are strong evidence (hm_0 at the small size). So "ghost returns are weak evidence" holds per trace, not in general. The effect is concentrated at the small cache size, and at the larger size the gate barely matters. The summary hit-rate rises slightly, so freq-gating is a safe base that improves on its parent. The next lever is how freq-0 returnees are handled: for example, a stronger initial credit in small, or letting them go to main only when main's tail has freq 0. The aim is to recover hm_0 while keeping the web_0 gain. One implementation detail is needed for correctness: the weak-returnee branch removes the key's ghost index entry *before* `evict_small()`. Otherwise `ghost_insert`'s eviction loop could treat that key's stale deque slot as a live ghost.
