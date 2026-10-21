# Variant v002 Insight

## Helped

- (none) hit-rate: Flat against the parent; the change is within noise. The duel almost never fires a loss, because the target's eviction victim is nearly always a counter-0 item (see Takeaway), so promotions go ahead exactly as in 0003 v002.

## Hurt

- (none) hit-rate: No measurable loss. The one possible cost is that duel losers re-sit in stash with `PROMOTE_THRESHOLD - 1`, so v002's second-chance scan shields them. That adds a little stash pressure, but losses are too rare for it to show.

## Takeaway

In this design a TinyLFU-style duel at the stash→target edge has nothing to work with, because normal-tier frequency and position are tied together. Every normal-tier hit that bumps `lfu_counter` also moves the item away from the LRU end (or promotes it to probing, taking its counter along). Stash promotions and probing demotions land at `Front`, which under ClassicLRU is the eviction end. So `eviction_candidate_index()` almost always returns a fresh arrival with counter 0, and the candidate wins. Two more things make the duel weaker. The candidate's score is always exactly `PROMOTE_THRESHOLD` (2), so the duel is really a fixed rule: block only if the victim's counter is at least 2. And halving on a loss drops a victim at 2–3 back to 1, so any protection lasts only one or two duels. For frequency-aware admission to change anything here, the victim has to be chosen by frequency, or the placement has to stop pushing newcomers to the eviction end. Comparing against the LRU-end resident is not enough.
