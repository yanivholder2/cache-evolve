# Variant v002 Insight

## Helped

- hit-rate: Preserved exactly — `get_position_rev` (`rposition`) returns the same unique index as the forward scan, so the resident found, the `remove_at`/`promote_to_target` path, and all promotion timing are byte-for-byte identical.
- metadata-size: Untouched — the change adds only a trait method and swaps one probe's direction, carrying no per-item or per-segment state.

## Hurt

- per-op-time: get-p50, the metric the hypothesis explicitly targeted, did not fall — it rose, and get-p99 rose alongside it; the MRU-first stash probe did not shorten the median stash hit's scan.

## Takeaway

Reverse-scanning the stash bucket refutes the hypothesis's premise that the median get hit sits near the bucket's Back (MRU) end: if it did, `rposition` would have shortened the median probe and lowered get-p50, but get-p50 moved the wrong way. Tellingly, put-p50 and put-p99 also shifted by comparable magnitudes despite the put path being completely untouched — so the per-op deltas read as run-to-run measurement variance, not a real cost signal, meaning stash-scan *direction* is not the get-p50 lever the hypothesis assumed it was.
