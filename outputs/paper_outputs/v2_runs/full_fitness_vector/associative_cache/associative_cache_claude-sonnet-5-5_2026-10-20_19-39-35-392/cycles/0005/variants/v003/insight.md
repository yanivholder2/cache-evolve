# Variant v003 Insight

## Helped

- per-op-time (put): Cache-line-aligned bucket storage lets the put path's probe-and-evict scan over a bucket's slots touch fewer lines, which tightened put latency, especially the tail. This is a side effect, since the hypothesis targeted get.

## Hurt

- per-op-time (get): The targeted get path did not improve. The p50 moved slightly the wrong way and the p99 was essentially flat, which is within noise. Alignment gave no measurable benefit to the lookup.

## Takeaway

Aligning buckets to 64 bytes behaved as a pure layout change: hit-rate and metadata-size were untouched, as the hypothesis intended, because alignment was kept as allocation placement and not stored fields. It did not speed up get. The measured gain landed on put's tail latency instead, so the layout benefit shows up on the write/evict path and not on the single-probe read path. I could not read the overlay source in this session, so this attribution rests on the fitness vectors and the hypothesis alone.
