# Cross-Cycle Insights (through cycle 0005)

## Recurring wins

- **Adaptive small/main sizing is the durable hit-rate lever.** It won in cycle 0001 and has beaten every replacement since: ARC-direction signals (0003), the static 10/90 split (0003, 0004), and stacked movement-rule tweaks (0002). Its margin is small but consistent.
- **The adaptation's bounds are the one live policy knob.** Lowering the small-queue floor clamp (0005) gave the only hit-rate gain since cycle 0001. Changes to the movement rules or the ghost mostly get absorbed by the adaptation.
- **Policy-neutral refactors that remove real machine work give dependable `put` gains.** Examples: hashed indexing (0002), the fingerprint ring replacing key clones (0003), and a compare-based ring wrap plus inlined eviction (0004). Each traced to specific removed work, and none changed hit-rate.
- **Testing one lever per variant has made results readable.** Since 0004 it has been possible to attribute outcomes. The two-lever variants in 0003 could not be interpreted.

## Recurring failures

- **Making the ghost or readmission path stronger backfires.** Ghosting main victims (0002), extra main-ghost traffic (0003), and readmitting with `freq` credit (0005) all lost hit-rate. Extra ghost signal pushes the split around, or keeps one-time-reuse keys in main too long.
- **Static splits lose to the adaptation.** This held both for the paper's 10/90 with `freq >= 1` (0003) and for 10/90 with `freq > 1` kept (0004). Ghost size was not pinned either time, which partly confounds the result.
- **Changing graduation thresholds alone gives nothing.** On the adaptive base the result was flat (0002), and the change was dropped.
- **Shrinking the ghost doesn't matter.** The clamps absorbed a halved ring (0004).
- **Inline-payload index layouts slow `get`.** Wider, non-power-of-two buckets cost more than the saved dependent load (0005). Counting dependent loads predicted the wrong direction.
- **Some failures were in measurement or attribution, not policy:**
  - `get` latency moves even when the `get` path is byte-identical (0003, 0004, 0005), so it is mostly noise.
  - `metadata-size` misses all ghost and index heap.
  - "Redundant work" premises were sometimes miscounted (0004).
  - Two hypotheses in 0001 were lost to Maven 429 build failures.
- **Many Winners got in on cohort rank, not merit.** Cohorts are small, so a variant that lost to its parent on both axes can still be a Winner (0002, 0004, 0005). Check deltas against the parent, not just the label.

## Open questions

- **Where does `small_size` settle?** Five cycles have asked. Instrumentation exists from 0005 but its output has never reached an insight. Until it does, we can't tell split-shift effects from residency effects.
- **How much of the adaptive gain comes from the adaptation itself?** The comparison needed is a static split at the parent's start point (25/75), or at the settled value, with ghost size pinned at the parent's.
- **Is the grow-small trigger strength, or the upper clamp, now a stronger lever than the floor?**
- **Would conditional readmit credit behave differently from blanket credit?** Candidates: only on a second ghost hit, or only while small is at its floor.
- **Does a decoupled main-victim ghost that doesn't feed the split adaptation help?** This was proposed in 0002 and is still untested.
- **Can a key-free slab `Entry` behind the narrow `(K, u32)` index cut `metadata-size` without slowing `get`?**
- **Would a true single-probe miss path (precomputed hash or raw-entry index) give a real `put` gain?**
- **Is any `get` latency gain real?** Answering this needs a same-host re-baseline of the parent.

## Per-policy notes

- **S3-FIFO with adaptive split (incumbent lineage):**
  - Parent chain: 0001 v003 → 0002 v003 (hashed) → 0003 v002 (fingerprint ring) → 0004 v003 (faster `put`) → 0005 v001 (`capacity/64` floor).
  - Graduation uses `freq > 1`.
  - The two resize triggers are lopsided ("main full on graduation" dominates).
- **Ghost structure (within S3-FIFO):**
  - Ghost size and contents feed the adaptation directly, so the ghost acts as a policy knob.
  - The fingerprint ring's holes shorten its effective memory.
  - Any split experiment must pin ghost size on its own; it has been coupled by accident twice (0003, 0004).
- **Static S3-FIFO (paper 10/90):**
  - Lost slightly in both trials.
  - Never tested cleanly with ghost size held constant.
- **ARC-style two-ghost signals:**
  - Lost hit-rate in their only trial (0003), which changed two levers at once.
  - Dropped as a direction.
- **Metric caveats that apply to every policy:**
  - Credit `get` latency only when the `get` path or the index layout changed.
  - Treat `metadata-size` as counting `Entry` only.
