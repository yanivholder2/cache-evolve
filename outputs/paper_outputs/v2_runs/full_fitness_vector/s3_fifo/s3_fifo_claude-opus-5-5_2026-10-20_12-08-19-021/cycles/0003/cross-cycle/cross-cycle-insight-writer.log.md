# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-10-20T12:31:15+00:00

## timeline

### 12:31:33 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0003)\n\n## Recurring wins\n\n- **The adaptive small/main split with `freq > 1` (two-touch) graduation has stayed the incumbent policy since cycle 0001.** Movement-rule tweaks (0002), ARC-direction signals (0003) and the paper's static 10/90 setup (0003) have all failed to beat it on hit-rate.\n- **Changes that leave the policy alone and cut real work pay off on latency.** Hashed indexing (0002) and the fingerprint ring ghost (0003) both won while keeping hit-rate flat or better. When a `put` gain can be traced to specific removed work (scans, clones, compaction, resize branches), treat it as real.\n- **Lineage can be stacked.** Every cycle's winner made a good parent for the next: adaptive split (0001), then hashed (0002), then fingerprint ring (0003). Each step kept the earlier gains.\n\n## Recurring failures\n\n- **Movement-rule changes stacked on the adaptive split.** These include a looser graduation threshold, ghosting main victims, and letting main-ghost hits bypass `small`. On hit-rate they come out flat or worse. They seem to push the split adaptation around, and the adaptation cancels or reverses the intended effect.\n- **Changing two levers in one variant (both cycle 0003 losers).** When the variant loses, we can't tell which lever caused it.\n- **Wholesale policy replacements**, either ARC-style rewrites or copying the published static setup. Neither has beaten the incumbent.\n- **Crediting `get` latency changes when the `get` code did not change.** This has happened repeatedly: first scan artifacts, then benchmark variance or host drift. These changes can't be relied on.\n- **Relying on `metadata-size`.** It only counts the main entry struct, so it has missed ghost cost and structural overhead in every cycle.\n- **Rank-only Winners.** Several Winners (0002 v002; 0003 v001 and v003) lost to their parent and qualified only because their cohort was weak. Don't use them as parents.\n- **Infrastructure.** In cycle 0001, failed builds (HTTP 429 from the dependency repository) wiped out two hypotheses that had nothing wrong with them. A rejected variant can mean a failed build, not a bad idea.\n\n## Open questions\n\n- **Where does `small_size` settle?** Every cycle since 0001 has asked for this and no variant has reported it. Without it we can't explain how the adaptation interacts with movement or ghost changes.\n- **Adaptive versus the best fixed split.** Is the incumbent's margin coming from the adaptation itself, or from where it happens to settle? Test this with a fixed split near the settled value.\n- **Single-lever splits of the 0003 losers:**\n  - a static split with the incumbent's `freq > 1` graduation;\n  - main-ghost hits reinserted into `small`, or the original small-ghost direction kept.\n- **Ghost size and hole handling as a hit-rate knob.** The ring's shorter effective memory seemed to help. Can ghost capacity be tuned on purpose?\n- **A decoupled main-victim ghost** that does not feed the split adaptation. Not tried yet.\n- **Same-host re-baselining** to tell real `get` latency gains from noise.\n\n## Per-policy notes\n\n- **S3-FIFO family (adaptive-split incumbent lineage):**\n  - Ghost size and contents act as a policy input, not neutral plumbing, because ghost-hit frequency drives the split.\n  - The two resize triggers are lopsided: the main-full-on-graduation trigger dominates in steady state, so the split probably settles at a clamp-driven balance point.\n- **ARC-style two-ghost signals:** signals in the correct direction still lost a little hit-rate on workload-a, and they add ghost traffic that pushes the split around.\n- **Static S3-FIFO (paper's 10/90 split, `freq >= 1` graduation):** a little below the adaptive incumbent. Not yet tested with the incumbent's graduation rule.\n- **Baseline fixed 25/75 split:** worse than adaptive sizing on workload-a. The split ratio matters.\n"
}
```

### 12:31:40 chat

I wrote `cross-cycle-insights.md` to the cycle 0003 cross-cycle output directory. It has the four required sections, in order, built only from the three-cycle ledger.

- **Recurring wins:**
  - The adaptive small/main split with two-touch (`freq > 1`) graduation has held up since cycle 0001. Movement-rule tweaks, ARC-style signals and the paper's static 10/90 setup have all failed to beat it.
  - Changes that leave the policy alone and remove real work (hashed indexing in 0002, the fingerprint ring ghost in 0003) are where the latency gains came from.
  - Each cycle's winner made a good parent for the next, and the gains carried forward.
- **Recurring failures:**
  - Movement-rule changes stacked on the adaptive split come out flat or worse; the split adaptation seems to cancel them out.
  - Variants that change two levers at once lose without telling us which lever cost the hit-rate.
  - Wholesale policy replacements haven't beaten the incumbent.
  - Credited `get` latency gains often come with no change to the `get` code.
  - The `metadata-size` metric misses ghost and structural cost.
  - Several Winners only qualified because their cohort was weak, and they lost to their parent.
  - In cycle 0001, two hypotheses were rejected because the build failed, not because of their ideas.
- **Open questions:**
  - Where `small_size` settles. Every cycle since 0001 has asked for this and no variant has reported it.
  - Whether the adaptive split beats the best fixed split.
  - Single-lever versions of the two cycle 0003 losers.
  - Ghost capacity as a hit-rate knob.
  - A separate ghost for main-queue victims that doesn't feed the split adaptation.
  - Re-measuring `get` latency on the same host to separate real gains from noise.
- **Per-policy notes:** grouped into the adaptive S3-FIFO lineage, ARC-style two-ghost signals, static S3-FIFO 10/90, and the fixed 25/75 baseline.

There are no numbers in the file, as required.


---

- finished: 2026-10-20T12:31:41+00:00
- duration: 25.45s
- status: ok
- input_tokens: 4
- output_tokens: 2268
- cache_read_tokens: 35733
- cache_write_tokens: 8619
- cost_usd: 0.121475
