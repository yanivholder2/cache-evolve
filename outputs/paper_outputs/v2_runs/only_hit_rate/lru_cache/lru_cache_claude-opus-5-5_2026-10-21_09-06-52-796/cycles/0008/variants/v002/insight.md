# Variant v002 Insight

## Helped

(No measured dimension improved. The overlay matches H02 as written. `small`, the ghost, the split constants, `MAX_FREQ`, `get` and both admission routes into `main` are byte-for-byte the same as in the parent. The only behavioural change is that `sieve_evict_main()` replaces the `pop_front`/`push_back` reinsertion loop. So the result is a clean test of scan order.)

## Hurt

- hit-rate: Under FIFO-with-reinsertion, every key entering `main` was queued behind every resident and every survivor that got reinserted. That gave it close to a full rotation of `main` to collect a hit. Under SIEVE, new arrivals are linked at `main_head`, and the hand walks tail→head toward them. How long an arrival lasts now depends on where the hand happens to be. If the hand is near the head, the arrival is checked after only a few steps. On workload-a, many keys that enter `main` with `freq = 0` are reused, just not right away. That includes both ghost hits and promotions from `small`, because `evict_one()` resets `freq` to 0 on promotion. Cutting their grace period evicts them before that reuse arrives. Meanwhile, survivors that stay in place bring no gain that makes up for it.

## Takeaway

On workload-a at size 2048, keys that enter `main` with `freq = 0` are not mostly dead weight. Demoting them faster with a SIEVE hand cost slightly more hits than it saved. That points to the full-rotation grace period of FIFO-with-reinsertion as useful protection, not wasted space. As H02 predicted, scan order in `main` is not the limiting factor here. The weakness the parent identified is that `main` entries arrive with zero credit. Shortening their stay made that weakness worse, so the fix should give them evidence on entry instead. One example is a starting `freq` of 1 for ghost-admitted keys.
