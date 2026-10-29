# Variant v002 Insight

## Helped

- hit-rate (`hm_0`, 16K and 128K): Fresh stash→target promotees now need one counted target hit (`TARGET_HITS_BEFORE_PROBING = 1`, carried in `lfu_counter`) before a hottest-position hit can move them to probing. That shields current probing residents from one-off promotees, so fewer of them are evicted and pushed down the probing → target → stash cascade. On `hm_0`, which has strong short-term reuse, the residents being protected are more valuable than the newcomers they would have lost to. The gains are small, but both sizes moved the same way.

## Hurt

- hit-rate (`web_0`, 128K): This is the cell the hypothesis aimed to lift, and it moved the wrong way. With spare target room, delaying the target→probing move keeps a genuinely hot promotee in the target for an extra hit. Meanwhile it holds a target slot and competes with other target residents, instead of moving up to the probing tier where `web_0` at 128K seems to benefit from it. The rationale blamed churn from one-hit promotion for this flat cell. The result suggests that churn was not the binding cost.

## Takeaway

On `web_0` at 128K, making a fresh promotee prove itself once more in the target hurts. It costs a little more than it saves, so the parent's one-hit fast track to probing was not what kept that cell flat. On `hm_0` it gives a small but consistent benefit. The net effect is roughly neutral, so a promotion-timing gate alone does not unlock `web_0` at 128K. Also note that the counter keeps accumulating through non-hottest refreshes and is reset only on leaving the target. Any first target hit therefore satisfies the gate, wherever the item sits in the bucket. Only promotees whose very first target hit lands at the hottest position are actually delayed, so this is a weak filter rather than a real "proven in target" test.
