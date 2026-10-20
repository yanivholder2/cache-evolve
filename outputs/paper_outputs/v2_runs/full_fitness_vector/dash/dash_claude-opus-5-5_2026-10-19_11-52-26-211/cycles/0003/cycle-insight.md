# Cycle 0003 Insight

## Cohort outcome

All three variants were built on cycle-2's v003 (fresh puts at stash Front, demotees at stash Back). v001 ported the target→probing→stash lookup order. v002 required a second stash hit before promoting to target. v003 blocked target→probing promotions that would evict a native neighbour. All three are listed as Winners, but each one met its own falsification condition. None raised hit-rate. v002 and v003 shortened the get tail but lost a little hit-rate. v001 left hit-rate unchanged, as expected, but made get p50 worse.

## Patterns

- The parent's admission policy now looks like a local optimum on both edges. Cycle 2 showed that hotter stash→target promotion hurts. This cycle showed that stricter promotion on either edge also hurts: a second-hit gate on stash→target, and a native-ownership gate on target→probing. Every filter tried so far has blocked real reuse rather than noise. The tiny buckets (2-slot stash, small target) can't hold items long enough for an extra wait to pay off.
- How hits are spread across tiers depends on placement policy. Changes that hold for one cache state can stop holding when placement changes. Fresh-at-stash-Front placement sends every key's first reuse through the stash. That made stash hits common enough to flip the lookup-order result: cycle 1 found target-first was a win, and here stash-first is better.
- The two latency wins share one cause: skipping the remove → promote → demote → stash-insert cascade. Both removed it by promoting less often, and both paid for it in hit-rate. The tail cost comes from how expensive each cascade is, not from how often admission is allowed.
- The signal used to decide admission matters. Bucket position and native/foreign ownership are free to read. But neither tracks recency well enough to beat plain LRU-ordered promotion here.
- Metadata-size stayed the same in every variant. Put-path movements had no cause in the code changes and look like noise.

## Open questions

- Can the promotion cascade be made cheaper without changing which items get promoted? Options include in-place swaps instead of remove + reinsert, or limiting how far a cascade can go.
- Can each bucket scan be made cheaper so get p50 falls under any lookup order? Small per-slot fingerprints would do this, but they would cost metadata-size.
- Would a lookup order chosen per key or adaptively beat a fixed stash-first order? Or does the extra branching cost more than it saves?
- Stash capacity and associativity haven't been tested. Bigger stash buckets might let a second-hit filter work where 2-slot buckets couldn't.

## Continuation

Keep cycle-2's v003 as the parent. Nothing this cycle beat it on hit-rate, and v001's lookup order is a regression on this lineage. Drop admission filters on both promotion edges (second-hit, ownership-based) and drop fixed lookup-order changes. Go after get latency through cheaper cascades and cheaper bucket scans that leave cache state unchanged. For hit-rate, try structural levers instead: stash sizing and associativity, or eviction order inside the target buckets.
