## Recurring wins

- **Capacity reallocation over per-entry protection.** Moving room from the
  small admission filter into main lifted hit-rate repeatedly (early cycles),
  though the gains flattened toward a knee just past the narrowest split tried.
- **Reward demonstrated reuse, never loosen the filter.** A higher freq
  ceiling for already-hot keys gave small, real, split-orthogonal gains.
- **Ghost length has a dominating knee at ~50% of main.** That point banked
  the put-latency win *and* recovered hit-rate — a rare non-tradeoff move;
  both wider and narrower windows were strictly worse on one axis.
- **Eliminate scans structurally, don't reorder them.** O(1) hash indices
  removed the resident-get walk (get path) and the ghost-membership walk (put
  path), each lowering median *and* tail together — the strongest moves seen.
- **Metadata discipline as a free dividend.** Retuning existing fields beat
  adding state; once the index made the stored `key` redundant, deleting and
  then relocating fields collapsed scored `Entry` size with zero hot-path cost.

## Recurring failures

- **Loosening admission/retention.** First-reuse promotion, non-zero ghost
  re-admission, and freq=1 seeding for re-admitted keys all hurt hit-rate; the
  baseline's strict freq=0 re-prove filter is load-bearing noise-screening.
- **Widening the ghost queue.** Longer reuse-distance coverage readmitted
  stale far-reuse keys, regressing hit-rate and growing metadata; this trace
  rewards only narrow near reuse.
- **Reordering a linear scan to make the median shallow.** Front-loading hot
  keys, a dedicated hot segment, and bounded rotation loops all missed their
  median targets and merely shifted cost into p99.
- **Adding struct state for speculative benefit** (ghost sizing field, bounded
  rotation) bought nothing and taxed metadata.

## Open questions

- The **unscored container-memory frontier** — separate ghost set vs.
  folded-tombstone map — has been flagged for several cycles but never
  directly contested; they make opposite tradeoffs for identical scored results.
- Is the cohort now optimizing the **measurement** (padding, field location)
  rather than real footprint? Does scoring still track the cost designs differ on?
- With hit-rate, both hot-path scans, and scored `Entry` size all at floors,
  is any scored axis left with genuine headroom?

## Per-policy notes

- **S3-FIFO on workload-a (all cycles).** Strict promotion/ghost thresholds
  are protective, not over-aggressive — this workload is best served by
  filtering tightly and spending freed capacity on main residency.
- Reuse distance rewarded is **narrow but past the most-recent 25%** of
  evictions; the recoverable near-reuse band sits at 25–50% of recent victims.
- Latency tracked exactly one scan at a time (ghost membership on put, resident
  walk on get); both are now O(1), so no hot-path linear walk remains.
- Hit-rate has held **bit-identical for many cycles** — the policy is
  hit-rate-bound at this capacity, and the remaining game is serving the same
  hits more cheaply (latency, then scored/container memory).
