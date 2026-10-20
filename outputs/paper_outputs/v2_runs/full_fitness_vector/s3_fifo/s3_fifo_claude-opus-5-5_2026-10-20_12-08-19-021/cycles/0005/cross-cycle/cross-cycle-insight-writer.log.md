# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-20T14:20:00+00:00

## timeline

### 14:20:20 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0005)\n\n## Recurring wins\n\n- **Adaptive small/main sizing is the durable hit-rate lever.** It won in cycle 0001 and has beaten every replacement since: ARC-direction signals (0003), the static 10/90 split (0003, 0004), and stacked movement-rule tweaks (0002). Its margin is small but consistent.\n- **The adaptation's bounds are the one live policy knob.** Lowering the small-queue floor clamp (0005) gave the only hit-rate gain since cycle 0001. Changes to the movement rules or the ghost mostly get absorbed by the adaptation.\n- **Policy-neutral refactors that remove real machine work give dependable `put` gains.** Examples: hashed indexing (0002), the fingerprint ring replacing key clones (0003), and a compare-based ring wrap plus inlined eviction (0004). Each traced to specific removed work, and none changed hit-rate.\n- **Testing one lever per variant has made results readable.** Since 0004 it has been possible to attribute outcomes. The two-lever variants in 0003 could not be interpreted.\n\n## Recurring failures\n\n- **Making the ghost or readmission path stronger backfires.** Ghosting main victims (0002), extra main-ghost traffic (0003), and readmitting with `freq` credit (0005) all lost hit-rate. Extra ghost signal pushes the split around, or keeps one-time-reuse keys in main too long.\n- **Static splits lose to the adaptation.** This held both for the paper's 10/90 with `freq >= 1` (0003) and for 10/90 with `freq > 1` kept (0004). Ghost size was not pinned either time, which partly confounds the result.\n- **Changing graduation thresholds alone gives nothing.** On the adaptive base the result was flat (0002), and the change was dropped.\n- **Shrinking the ghost doesn't matter.** The clamps absorbed a halved ring (0004).\n- **Inline-payload index layouts slow `get`.** Wider, non-power-of-two buckets cost more than the saved dependent load (0005). Counting dependent loads predicted the wrong direction.\n- **Some failures were in measurement or attribution, not policy:**\n  - `get` latency moves even when the `get` path is byte-identical (0003, 0004, 0005), so it is mostly noise.\n  - `metadata-size` misses all ghost and index heap.\n  - \"Redundant work\" premises were sometimes miscounted (0004).\n  - Two hypotheses in 0001 were lost to Maven 429 build failures.\n- **Many Winners got in on cohort rank, not merit.** Cohorts are small, so a variant that lost to its parent on both axes can still be a Winner (0002, 0004, 0005). Check deltas against the parent, not just the label.\n\n## Open questions\n\n- **Where does `small_size` settle?** Five cycles have asked. Instrumentation exists from 0005 but its output has never reached an insight. Until it does, we can't tell split-shift effects from residency effects.\n- **How much of the adaptive gain comes from the adaptation itself?** The comparison needed is a static split at the parent's start point (25/75), or at the settled value, with ghost size pinned at the parent's.\n- **Is the grow-small trigger strength, or the upper clamp, now a stronger lever than the floor?**\n- **Would conditional readmit credit behave differently from blanket credit?** Candidates: only on a second ghost hit, or only while small is at its floor.\n- **Does a decoupled main-victim ghost that doesn't feed the split adaptation help?** This was proposed in 0002 and is still untested.\n- **Can a key-free slab `Entry` behind the narrow `(K, u32)` index cut `metadata-size` without slowing `get`?**\n- **Would a true single-probe miss path (precomputed hash or raw-entry index) give a real `put` gain?**\n- **Is any `get` latency gain real?** Answering this needs a same-host re-baseline of the parent.\n\n## Per-policy notes\n\n- **S3-FIFO with adaptive split (incumbent lineage):**\n  - Parent chain: 0001 v003 → 0002 v003 (hashed) → 0003 v002 (fingerprint ring) → 0004 v003 (faster `put`) → 0005 v001 (`capacity/64` floor).\n  - Graduation uses `freq > 1`.\n  - The two resize triggers are lopsided (\"main full on graduation\" dominates).\n- **Ghost structure (within S3-FIFO):**\n  - Ghost size and contents feed the adaptation directly, so the ghost acts as a policy knob.\n  - The fingerprint ring's holes shorten its effective memory.\n  - Any split experiment must pin ghost size on its own; it has been coupled by accident twice (0003, 0004).\n- **Static S3-FIFO (paper 10/90):**\n  - Lost slightly in both trials.\n  - Never tested cleanly with ghost size held constant.\n- **ARC-style two-ghost signals:**\n  - Lost hit-rate in their only trial (0003), which changed two levers at once.\n  - Dropped as a direction.\n- **Metric caveats that apply to every policy:**\n  - Credit `get` latency only when the `get` path or the index layout changed.\n  - Treat `metadata-size` as counting `Entry` only.\n"
}
```

### 14:20:26 chat

I've written `cross-cycle-insights.md` from the cycle 0001–0005 ledger, using the four required sections in order. I didn't read the previous version of the file.

**Main takeaways for future cycles:**
- **Adaptive split:** The adaptive small/main split is still the policy to beat. It has outlasted every replacement tried: static splits, ARC-style signals and movement-rule tweaks.
- **Bounds on the split:** The limits on how far the split can move are the only policy setting still producing hit-rate gains. In cycle 0005, lowering the floor on the small queue gave the first gain since cycle 0001.
- **Ghost and readmission:** Making these paths stronger keeps backfiring, and so do static splits. In both static-split trials, the ghost size changed along with the split, so those losses aren't a clean test.
- **Measurement:**
  - `get` latency moves even when the `get` code is untouched, so treat it as mostly noise.
  - `metadata-size` doesn't count ghost or index memory.
  - Several Winners got in on rank in a small cohort despite losing to their parent.
- **Top open question:** Where `small_size` actually settles has been asked for five cycles and still hasn't reached any insight. Until it does, it's hard to tell whether a change worked by shifting the split or by some other effect.


---

- finished: 2026-10-20T14:20:26+00:00
- duration: 26.79s
- status: ok
- input_tokens: 4
- output_tokens: 2390
- cache_read_tokens: 37435
- cache_write_tokens: 11978
- cost_usd: 0.151127
