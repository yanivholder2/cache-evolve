# Variant v003 Insight

## Helped

- hit-rate (large-cache cells): Both 131072-size cells improved, as the hypothesis predicted. Before this change, a single hit on the hottest target resident could push it into a full probing bucket. That set off the full probing-LRU → foreign-retry → target-LRU → stash cascade. With the guard, that move now needs a second hottest hit, so fewer reused residents in probing and in the neighbouring target get pushed out on weak evidence.
- hit-rate (small-cache cells): Both 16384-size cells also improved, and hm_0 improved by about as much as the large cells. This reverses the small-cache slip v001 left. In v001 the extra hit was a stash requirement, and a small cache can evict a key from stash before that hit arrives. Here the extra hit is required of a key that is already in target, and target residence lasts longer even in a small cache. So the guard filters noise without losing the key.
- per-op-time (put median): Put p50 moved down a hair. This fits fewer cascades feeding stash, but `put` was not edited and the change is within noise.

## Hurt

- per-op-time (get tail): Get p99 rose noticeably, and it is the clearest cost of this variant. The common hit paths are unchanged. But the `is_hottest` arm now adds an `is_full()` check, a counter read, and, on a deferred hit, a counter write. Each guarded promotion also takes two `get`s instead of one. When the promotion finally runs, it is the full-probing cascade, which is still the slowest path, now preceded by extra branching. So the tail gets the guard overhead without losing the slow path.
- per-op-time (put tail): Put p99 rose slightly. `put` was not edited. One possible indirect cause: fewer promotions into probing leave more keys in target, so targets stay fuller and more stash→target moves hit v001's full-target guard, which leaves stash more crowded. This is not confirmed and may be noise.

## Takeaway

v001's door-guard idea also works one tier higher: one extra hottest-position hit before entering a *full* probing bucket raised hit-rate in every cell. The gain probably comes partly from where the guard sits. A key in target lasts long enough to collect the second hit, so unlike the stash-side guard, this one does not cost the small cache. Two caveats. First, unlike v001, the guard did not pay for itself in latency: get p99 got worse, so the cascade it defers is still the main cost of the tail. Second, the counter is not strictly "consecutive". A counter of 1 survives later non-hottest hits until the key is promoted or demoted. In practice the rule is "two hottest hits during one residence", and a stricter reset on non-hottest hits is a separate, untested variant.
