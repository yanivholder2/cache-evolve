# Variant v003 Insight

## Helped

- hit-rate: hm_0 at 131072 rose slightly, and the summary moved up by a hair. At the large size the ghost queue (bounded by `main_size`) is long enough to still hold a main victim when it is requested again. Re-admitting that key with `freq = 1` lets it survive one `evict_main()` sweep, so the reuse signal pays off there. The gain was predicted at the small size, not here.

## Hurt

- hit-rate: The cell the hypothesis targeted, hm_0 at 16384, did not move at all, and neither did web_0 at 16384. At the small size the ghost queue is short, and only main victims with `reused` set are ghosted. So main-victim ghost hits there are apparently too rare to change a single hit. web_0 at 131072 dropped slightly. The extra credit can keep a returning main victim in main through a sweep in which it is no longer hot, at the cost of a live resident. The hypothesis expected limiting credit to main-victim ghosts to prevent this, but it did not fully do so on web_0.
- per-op-time: `get` p50/p99 and `put` p50 each went up by a nanosecond or two. The change adds only a shift/OR in `push_ghost` and a bit test on the ghost-hit path of `put`, and `get` is untouched. So the `get` movement is almost certainly run-to-run noise, and the `put` movement is at most a negligible real cost.

(metadata-size did not move: the source tag lives in the low bit of the existing ghost sequence value, so `Entry<u64,u64>` and `S3Fifo<u64,u64>` keep their sizes.)

## Takeaway

Giving re-admitted main-victim ghosts one survived sweep (`freq = 1`) has no effect at the small cache size it targeted. The mechanism only fires on a ghost hit for a key that was reused in main, and at 16384 the short ghost queue makes those events too rare to matter. At 131072 it fires enough to show up, but the effect is mixed: it helps on hm_0 and hurts slightly on web_0. Small-cache hm_0 retention needs a lever that acts on every main eviction or on the small/main split, not on the ghost re-admission path.
