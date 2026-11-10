# Variant v003 Insight

## Helped

(none — no hit-rate cell improved on the parent)

## Hurt

- hit-rate (web_0, 131072): this is the cell the hypothesis targeted, and it did not move. The idea was that long reuse distances at this size made ghost returnees die before their next hit. If so, one extra pass through main would have turned some of those deaths into hits. It didn't, so returnees here either come back too rarely to matter, or need far more than one extra sweep to reach their next access.
- hit-rate (hm_0, 131072): this cell took the clearest loss. Main stays full in steady state, so every credit a returnee keeps through a sweep forces out a freq-0 entry ahead of it in the queue. On hm_0, keys returning through ghost are mostly not reused again in main. The credit kept dead keys around for an extra lap and pushed out entries that would have been hit.
- hit-rate (web_0, 16384): this cell also fell. In a small main, one sweep is short, so the extra lap a credited returnee gets costs a larger share of the queue. That pushes out freq-0 small graduates sooner.
- hit-rate (hm_0, 16384): roughly flat, slightly down. The small cache sends few keys back through ghost, so the credit changes little here.
- hit-rate (summary): every cell held or fell, so the aggregate is below the parent.

## Takeaway

In v001, a small tail with only one reuse goes to ghost once main is full, so the ghost-hit path is no longer reserved for strong long-range reuse. It now also carries the weak keys that v001 deliberately refused to graduate. Giving that path `freq = 1` reverses v001's ranking: a key with one small reuse plus a ghost return now enters main ahead of a freq ≥ 2 graduate, which is reset to 0. The results suggest this credit protected the wrong population.

This is the hypothesis's own failure case ("main is already well-filtered"). It also suggests that ghost-return credit, if tried again, should depend on evidence the key carries with it, such as its small-queue freq at the time it went to ghost, rather than being a flat bonus.

One caveat: the new rotation cap in `evict_main` is not a pure safety guard. When every entry in main has credit, the cap evicts a key that still has freq > 0, where v001 would have kept rotating. That is probably rare and minor, but it is a second behavioural change mixed into this result. Only hit-rate was measured. `Entry`'s layout is unchanged, so metadata size should be the same.
