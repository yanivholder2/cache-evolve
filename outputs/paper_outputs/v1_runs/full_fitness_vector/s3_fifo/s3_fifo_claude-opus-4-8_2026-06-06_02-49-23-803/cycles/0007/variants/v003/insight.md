# Variant v003 Insight

## Helped

- per-op-time: promoting a `main` entry into the short `hot` queue once it hits `MAX_FREQ` lets the heaviest repeat-access keys resolve in a bounded `hot_size` scan instead of a deep `main` walk, sharply collapsing the get p99 tail.
- per-op-time: scanning `hot` first in `find_entry_mut` also shortens put's pre-insert lookup for saturated keys, pulling put p50 down.
- hit-rate: carving `hot_size` out of the `main` budget (rather than adding capacity) kept the resident set fixed, so segmentation was hit-rate-neutral — a hair higher, within noise.

## Hurt

- per-op-time: the median hit does not reliably sit in `hot`, so the common case now pays an extra full `hot` probe before reaching `main` — get p50, the hypothesis's actual target, ticked the wrong way, and put p99 widened from the added promotion/demotion bookkeeping in `get`.
- metadata-size: the scored `Entry` stayed at 24 bytes as intended, but the `S3Fifo` container grew (extra `hot` VecDeque plus the `hot_size` field), raising per-instance overhead even though the scored summary held.

## Takeaway

A dedicated hot front-queue fed by `MAX_FREQ` saturation is effective at what it physically does — bounding the worst repeat-hit scan — and that shows up as a large get p99 drop and faster put lookups. But it missed its stated goal: median hits don't land in `hot` often enough to offset the added hot-then-main probe, so get p50 regressed. The segmentation trades tail latency for a slightly worse median, the inverse of the intended effect.
