# Cross-Cycle Insights (through cycle 0003)

## Recurring wins

- **The adaptive small/main split with `freq > 1` (two-touch) graduation has stayed the incumbent policy since cycle 0001.** Movement-rule tweaks (0002), ARC-direction signals (0003) and the paper's static 10/90 setup (0003) have all failed to beat it on hit-rate.
- **Changes that leave the policy alone and cut real work pay off on latency.** Hashed indexing (0002) and the fingerprint ring ghost (0003) both won while keeping hit-rate flat or better. When a `put` gain can be traced to specific removed work (scans, clones, compaction, resize branches), treat it as real.
- **Lineage can be stacked.** Every cycle's winner made a good parent for the next: adaptive split (0001), then hashed (0002), then fingerprint ring (0003). Each step kept the earlier gains.

## Recurring failures

- **Movement-rule changes stacked on the adaptive split.** These include a looser graduation threshold, ghosting main victims, and letting main-ghost hits bypass `small`. On hit-rate they come out flat or worse. They seem to push the split adaptation around, and the adaptation cancels or reverses the intended effect.
- **Changing two levers in one variant (both cycle 0003 losers).** When the variant loses, we can't tell which lever caused it.
- **Wholesale policy replacements**, either ARC-style rewrites or copying the published static setup. Neither has beaten the incumbent.
- **Crediting `get` latency changes when the `get` code did not change.** This has happened repeatedly: first scan artifacts, then benchmark variance or host drift. These changes can't be relied on.
- **Relying on `metadata-size`.** It only counts the main entry struct, so it has missed ghost cost and structural overhead in every cycle.
- **Rank-only Winners.** Several Winners (0002 v002; 0003 v001 and v003) lost to their parent and qualified only because their cohort was weak. Don't use them as parents.
- **Infrastructure.** In cycle 0001, failed builds (HTTP 429 from the dependency repository) wiped out two hypotheses that had nothing wrong with them. A rejected variant can mean a failed build, not a bad idea.

## Open questions

- **Where does `small_size` settle?** Every cycle since 0001 has asked for this and no variant has reported it. Without it we can't explain how the adaptation interacts with movement or ghost changes.
- **Adaptive versus the best fixed split.** Is the incumbent's margin coming from the adaptation itself, or from where it happens to settle? Test this with a fixed split near the settled value.
- **Single-lever splits of the 0003 losers:**
  - a static split with the incumbent's `freq > 1` graduation;
  - main-ghost hits reinserted into `small`, or the original small-ghost direction kept.
- **Ghost size and hole handling as a hit-rate knob.** The ring's shorter effective memory seemed to help. Can ghost capacity be tuned on purpose?
- **A decoupled main-victim ghost** that does not feed the split adaptation. Not tried yet.
- **Same-host re-baselining** to tell real `get` latency gains from noise.

## Per-policy notes

- **S3-FIFO family (adaptive-split incumbent lineage):**
  - Ghost size and contents act as a policy input, not neutral plumbing, because ghost-hit frequency drives the split.
  - The two resize triggers are lopsided: the main-full-on-graduation trigger dominates in steady state, so the split probably settles at a clamp-driven balance point.
- **ARC-style two-ghost signals:** signals in the correct direction still lost a little hit-rate on workload-a, and they add ghost traffic that pushes the split around.
- **Static S3-FIFO (paper's 10/90 split, `freq >= 1` graduation):** a little below the adaptive incumbent. Not yet tested with the incumbent's graduation rule.
- **Baseline fixed 25/75 split:** worse than adaptive sizing on workload-a. The split ratio matters.
