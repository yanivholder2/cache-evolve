## Recurring wins

- One change per variant. Bundled changes in cycles 1–2 kept trading hm_0 against web_0. Once changes were isolated (cycles 3–5), the mechanisms could be told apart and the gains could be stacked.
- Filters that grant credit only for reuse inside main. Gating main-victim ghosts on reuse and resetting freq/reused at graduation each helped hm_0 without hurting web_0 (cycle 3). Merged, they gained at every trace/size point (cycle 4). This is the most reliable lever found so far.
- Spend the ghost budget selectively, don't enlarge it. Filters that free ghost slots lengthen useful history for free.
- Structural changes that keep hit rates bit-identical (slab + hash indexes, Fx hasher) are the only per-op-time gains the diff explains. They are a stable base for later policy work.
- hm_0 responds to main retention: more main residency, main-victim history, a deeper freq cap. Recent gains sit mostly at the small cache size.

## Recurring failures

- Fixed-ratio and adaptive small/main splits (cycles 1, 5). They shift the hm_0 vs web_0-large trade-off without improving the summary. Shrinking the small queue hurts web_0 at both sizes.
- Resizing or enlarging the ghost (cycles 1, 2, 4). Splitting ghost regions didn't rescue web_0. A bigger bound lets stale reuse back into main, where it evicts live residents.
- Looser graduation (`freq >= 1`). This was cycle 2's web_0 regression, and later isolation pinned the loss on it.
- Hash-table layout tweaks aimed at probe count or load factor. A merged index (denser) and pre-sized indexes (sparser) both made tails worse with hit rates unchanged. Footprint and locality matter more than probe count.
- Timing credit for policy-only diffs. Unexplained per-op-time drift has shown up every cycle and is environmental noise. `put` p99 drifts slightly upward across variants.
- Per-entry metadata is flat in every cycle while heap use (ghost keys, index overallocation) grows unseen.

## Open questions

- web_0 large-size gap: now looks like a probation-window problem. Test a longer probation (small share above 25%) or easier graduation applied only inside probation, without bringing back the `freq >= 1` regression.
- Can main-retention gains at the small size (`MAX_FREQ`, threshold) be made size-aware so they also help at the large size, where main looks saturated?
- Can the ghost index be shrunk (fingerprints, compact keys) to cut `put` tail cost?
- Should the metadata metric include heap bytes per resident key, including ghost and index memory?
- Is a pinned, same-machine benchmark environment achievable, so policy-variant timing becomes interpretable?

## Per-policy notes

- S3-FIFO (the only policy explored so far). Current lineage: Fx slab + two-table index → reuse-gated main-victim ghost + credit reset at graduation (`freq > 1` threshold kept) → `MAX_FREQ` 7. Ghost bound stays at `main_size` or below. Small share stays at 25% or above. Raising `MAX_FREQ` further is unlikely to pay off. Implementation work should only shrink tables, starting with the ghost index.
- Trace behaviour: hm_0 is driven by how main retains keys and is near-saturated at large size. web_0 is driven by how long probation lasts and is sensitive to graduation strictness and small-queue size.
