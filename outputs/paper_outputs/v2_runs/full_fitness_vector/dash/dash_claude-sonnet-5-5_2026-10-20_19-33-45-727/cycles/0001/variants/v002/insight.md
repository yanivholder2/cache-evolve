# Variant v002 Insight

## Helped

- hit-rate: Admitting stash hits into the target bucket at Front (cold), while promoting proven-hot target items into probing at Back, keeps one-shot stash hits from displacing established residents, which gave a modest gain over the single-position baseline.
- per-op-time (get): The change adds no work to the get path; it only swaps a position argument, so the improved get latency is most likely run-to-run variance or fewer cascades, not a mechanical speedup.

## Hurt

- per-op-time (put): Back-inserted promotions and their cascades now run through demote_from_normal_bucket with Back, and cold Front admissions leave items sitting where they are evicted sooner, so put latency regressed slightly. The effect is small and may be noise.

## Takeaway

Choosing the insert position by tier (cold for first-time stash admission, hot for target-to-probing promotion) is a cheap way to add scan resistance with no metadata cost. The hit-rate gain is small, so insert position is a weak lever on this workload. The latency shifts have no clear code cause and should be confirmed by re-measuring before they are trusted.
