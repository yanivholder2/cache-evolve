# Variant v003 Insight

## Helped

- hit-rate: Hit rate rose above the hypothesis's 0.702 bar, so its hit-rate claim holds. With `FRESH_PUT_FILLS_TARGET_HOLE`, a fresh `put` first goes into a free slot of its target bucket at Front, and only falls back to the stash when the target is full. That puts idle normal-tier capacity to use at insert time. It also takes overflow pressure off the 2-slot stash buckets, so fewer fresh items (and fewer reuse-proven demotees at stash Back) are pushed out of the cache before they are reused.

## Hurt

- per-op-time (get p50): The median `get` got slower rather than faster, so the hypothesis's latency claim is refuted. One likely cause is that cold one-hit items now occupy target slots that used to stay empty. Later stash→target promotions then find the target full more often. Each of those now pays for a `demote_from_normal_bucket` eviction plus an extra stash insert, where before it was a free insert. In addition, a fresh item that is the only one in its bucket is the promotion candidate, so its first reuse is a target→probing `remove_at` + `put_at`, which can cascade. The cheap first-touch path the hypothesis expected does not make up for these costs.
- per-op-time (get p99 / put p50, p99): The `get` tail and both `put` percentiles also rose. On `put` this fits the new up-front work on every call: an extra `get_index` hash plus a `buckets[t].is_full()` check, which touches a normal-bucket cache line even when the put then falls back to the stash. On `get`, the tail rises because full-target promotion cascades now happen more often.

(metadata-size did not change: no fields were added, and `Item` stays at 40 bytes.)

## Takeaway

Filling empty target slots with fresh puts at insert time does convert idle normal-bucket capacity into hit rate. But it is not a latency win on this lineage. Fresh items that were never reused now sit in target as "pre-occupants". This makes the normal tier fill sooner, so later reuse-proven promotions pay eviction and demotion cascades they used to avoid. The cost moves to those promotions instead of going away. A hole-filling put trades hit rate for get/put time. To keep the hit-rate gain without the latency cost, fresh items would have to be placed in target without getting in the way of later stash→target promotions. Simply sharing the target's LRU end with promoted items does not achieve this.
