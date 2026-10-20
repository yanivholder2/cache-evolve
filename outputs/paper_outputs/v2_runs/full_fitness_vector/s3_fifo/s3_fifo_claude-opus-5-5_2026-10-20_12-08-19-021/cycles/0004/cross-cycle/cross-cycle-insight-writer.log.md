# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-10-20T12:37:56+00:00

## timeline

### 12:38:16 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (cycles 0001–0004)\n\n## Recurring wins\n\n- **Adaptive small/main sizing has held up for four cycles straight.** It first won in 0001. Since then it has outlasted ARC-direction signals (0003), the paper's static 10/90 split (0003) and a frozen split that kept `freq > 1` (0004). The margin is small but it keeps showing up, so the adaptation earns its place. It is not just an escape from a bad 25/75 start.\n- **Cutting machine work on the hot path gives real latency gains, with hit-rate unchanged.** Examples: hashed indexing replacing linear scans (0002), a fingerprint ring replacing key-cloning ghosts (0003), and modulo-free wrap plus inlined eviction on `put` (0004). These refactors have been the cleanest Winners, because they improve one axis and leave the policy alone.\n- **The lineage that works is: keep the incumbent policy and make it cheaper.** Each adopted parent since 0002 has been a policy-neutral or near-neutral refactor of the adaptive-split policy.\n- **`put` latency claims hold when they trace to removed work.** Removed clones, `retain` compaction and resize branches all produced gains that held up.\n\n## Recurring failures\n\n- **Movement-rule tweaks on top of the adaptive split.** A looser graduation threshold was flat. Ghosting main victims was slightly worse. The adaptation seems to absorb or reverse these changes, because extra graduations or ghost hits push the split around.\n- **Replacing the adaptation with a static split** has lost hit-rate twice (0003, 0004), including with the incumbent graduation rule kept.\n- **Variants that change two levers at once** (0003's H01 and H03) lose hit-rate without telling us which lever caused the loss. Even variants labelled \"one lever\" keep tying ghost size to the split (0004).\n- **Ghost-size changes** barely move hit-rate, because the adaptation clamps absorb them. Stop shrinking the ghost.\n- **Crediting `get` latency changes when the `get` path is untouched.** `get` has moved in several cycles with byte-identical code, so it is noise or host drift. This is a problem because the headline latency metric tracks `get`.\n- **Trusting `metadata-size`.** It counts only `Entry<K,V>`, so it has missed every ghost-heap change and every structural overhead change.\n- **Ranking-only Winners.** Several Winners lost to their parent on one or both axes and got in only on rank within a small cohort. Read the comparison with the parent, not the Winner label.\n- **Infrastructure:** 0001 lost two variants to Maven HTTP 429s, so an unevaluated variant tells us nothing.\n- **Unverified premises.** 0004's claim of \"redundant hashing\" was a miscount. Count hashes and clones in the parent before treating them as a lever.\n\n## Open questions\n\n- **Where does `small_size` settle?** This has been asked in every cycle and never reported. It is the single most important missing piece of instrumentation.\n- **Static 25/75 split versus the adaptive split.** Test a static 25/75 with ghost size pinned to the parent's value, to separate the adaptation from the split ratio and the ghost size.\n- **Adaptation clamps and trigger strength.** The `capacity/16` and `capacity/2` clamps and the lopsided \"main full on graduation\" trigger may decide the balance point. Loosening or tightening them might turn ghost length back into a lever that matters.\n- **Main-ghost hits reinserted into `small`, or a decoupled main-victim ghost that does not feed \"grow main\".** Both are still untested in isolation.\n- **Hole handling in the ring.** It shortens the ghost's effective memory. Is it a hit-rate lever?\n- **A single-probe miss path** (precomputed-hash or raw-entry index) for a further `put` gain.\n- **Is the `get` latency signal real at all?** This needs same-host re-baselining of the parent.\n\n## Per-policy notes\n\n- **S3-FIFO with adaptive split** (the incumbent lineage, v003 from 0001 onwards): it starts at 25/75 and adapts using ghost hits and a graduation-pressure trigger. The trigger is lopsided, so the split likely settles near a fixed point set by the clamps. It uses `freq > 1` (two-touch) graduation. Ghost size and contents feed the adaptation directly, so any ghost change is a policy change.\n- **S3-FIFO with static split** (paper 10/90, `freq >= 1`; and 10/90 with `freq > 1`): consistently slightly below the adaptive incumbent on hit-rate.\n- **ARC-style two-ghost adaptation:** it lost a little hit-rate. The result is confounded because main-ghost hits also bypassed `small`.\n- **Ghost representations:** the hashed key ghost (0002) was replaced by a fingerprint ring (0003). The ring is cheaper on `put` and drifted the split favourably. A halved ring (0004) was roughly neutral.\n- **Current parent:** cycle 0004's v003, which is the fingerprint-ring adaptive S3-FIFO with a streamlined `put` path.\n"
}
```

### 12:38:24 thinking



### 12:38:24 chat

I wrote `cross-cycle-insights.md` from the four-cycle ledger, using the four required sections in order. I built it only from the ledger and didn't read the previous file.

- **Recurring wins:** The adaptive small/main split has beaten every alternative tried, four cycles in a row. The cleanest Winners have been refactors that leave the policy alone and make the hot path cheaper: hashed indexing, the fingerprint ring for ghosts, and the streamlined `put`.
- **Recurring failures:**
  - Tweaking the movement rules (graduation threshold, ghosting main victims) on top of the adaptive split.
  - Static splits, which lost hit-rate twice.
  - Changing two levers in one variant, so a loss can't be traced.
  - Shrinking the ghost, which the adaptation clamps absorb.
  - Crediting `get` latency changes when the `get` code didn't change.
  - Trusting `metadata-size`, which misses ghost and structural overhead.
  - Treating Winners that only got in on rank within a small cohort as real improvements.
- **Open questions:** The most important gap is where `small_size` settles. Every cycle asked for it and no variant has reported it. Also still open:
  - Static 25/75 with the ghost size pinned.
  - Tuning the adaptation clamps and trigger strength.
  - Feeding main-ghost hits back into `small`, or a separate main-victim ghost.
  - Whether the ring's holes matter for hit-rate.
  - A single-probe miss path for `put`.
  - Re-measuring `get` latency on the same host.
- **Per-policy notes:** Covers the adaptive incumbent, the static splits, the ARC-style variant, how the ghost was represented over time, and which variant is the current parent (cycle 0004's v003).


---

- finished: 2026-10-20T12:38:24+00:00
- duration: 28.59s
- status: ok
- input_tokens: 4
- output_tokens: 2513
- cache_read_tokens: 36842
- cache_write_tokens: 10071
- cost_usd: 0.138212
