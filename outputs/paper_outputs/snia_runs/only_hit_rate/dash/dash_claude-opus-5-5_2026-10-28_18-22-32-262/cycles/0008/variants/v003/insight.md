# Variant v003 Insight

## Helped

None of the cells improved. The cell this hypothesis aimed at did not move:

- hit-rate (128K, `web_0`): Identical to the parent. A promotion into a full target is not where the `web_0` 128K giveback comes from. A likely reason is that v002's rescue already absorbed the cascade this change was meant to remove. In the parent, the demoted target resident went to the stash MRU. The stash victim it pushed out was then usually rescued into the same kind of free normal slot this change now fills up front. Either way the free slot gets used and the ghost is not touched, so the end state is nearly the same.

## Hurt

- hit-rate (128K, `hm_0`): A small loss, the only cell that moved. A proven stash item now skips the target tier and takes a free probing slot. That leaves the cold LRU target resident in place, where before it would have been demoted to the stash. It also uses up probing headroom. Without that headroom, later target-hottest→probing promotions displace a foreign resident, triggering a retry cascade, where before they landed for free. It also takes slots that rescue would otherwise have used for stash-overflow victims. On `hm_0` at 128K, that probing headroom was worth more than keeping the target LRU resident.
- hit-rate (16K, both traces): Unchanged, as the hypothesis predicted. At 16K a full target almost never sits next to a free probing slot, so the new branch rarely fires.

## Takeaway

"Free-slot-only, no-cascade" is not good by itself: it matters which item a free slot goes to and which tier it lands in. Once rescue already sends cascade victims into free normal slots, also sending gated stash promotions into free probing slots only changes the order in which the same headroom gets used. It does not reduce ghost churn. Spending probing headroom early on items that skipped the target tier costs a little on `hm_0`. The `web_0` 128K giveback must come from somewhere other than stash→full-target promotions. Further changes that only add uses for free slots look exhausted in this lineage.
