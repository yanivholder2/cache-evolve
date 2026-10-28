# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-10-28T14:41:17+00:00

## timeline

### 14:41:34 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (cycles 0001–0003)\n\n## Recurring wins\n\n- Where new items are inserted matters more than anything else. Every Winner so far (0001 v001, 0002 v002, 0003 v003) is in the cold-insertion lineage, which places new items below reused ones. Changes that leave new items at MRU have lost.\n- The GCLOCK lineage improves hit-rate reliably. Exact LRU order hasn't been needed: CLOCK matched LRU in 0001, and 2-bit GCLOCK improved on CLOCK in every cell in 0002. The gains are largest in the large buckets, where the insertion lineage has the least room left.\n- Small-bucket changes can recover hm_0 reuse that lies beyond the probation window. Deeper probation and occasional MRU insert both did this, though only at the small size.\n\n## Recurring failures\n\n- Fixed rules that apply to every workload split by trace. hm_0 wants a large protected region and web_0 wants deep probation. Both rules split the same fixed bucket space, so a fixed BIP period or a fixed floor helps one trace and hurts the other. The floor and BIP sampling don't stack (0003).\n- Per-bucket memory structures sized below the reuse distance they target have failed: sample rings, ghost rings, PSEL trackers, and short probation windows. They forget items before they can produce a signal. Feedback that credits hits probation would have caught anyway is miscalibrated too.\n- The large buckets have not improved since 0001 for the insertion lineage. Floors never engage at the large size, MRU sampling pollutes web_0 there, and warm admission without enough memory pollutes GCLOCK.\n- Promotion tweaks (bounded or graded promotion) while new items still enter at MRU: these made hm_0 worse.\n- In three cycles, nobody has measured per-op-time or metadata-size, and every cycle has added more unmeasured cost: mid-vector inserts, GCLOCK sweeps, full-bucket scans, linear scans of rings on the miss path.\n\n## Open questions\n\n- Can true leader/follower set dueling between the shallow and deep probation/protected splits choose the right split per workload, where per-bucket self-feedback failed?\n- Does feedback work once it is sized to the bucket's lifetime (for example a per-item sampled bit) and credits only hits after probation?\n- Can GCLOCK take a cold-insertion analogue, such as starting new items below referenced ones? Does ghost admission with full-rotation coverage do anything? If not, drop ghost admission.\n- What is the real latency cost of each lineage? In particular, does CLOCK/GCLOCK beat mid-vector LRU insertion once worst-case sweep cost is counted?\n\n## Per-policy notes\n\n- **ClassicLRU + cold insertion (LIP-style):** This is the base of every Winner. A fixed-fraction probation depth is too short in 128-slot buckets (it loses hm_0 reuse) and works well in 1024-slot buckets (it drains scans).\n- **BIP / occasional MRU insert:** Helps hm_0 at the small size and hurts web_0 at the large size. Tuning the period alone is exhausted. A PSEL rate driven by a short ring pushed it toward over-sampling.\n- **Absolute probation floor:** Affects only small buckets and trades web_0 gains for hm_0 losses. Don't stack it on BIP.\n- **Bounded or graded promotion:** Harmful while new items enter at MRU. Revisit it only on top of cold insertion.\n- **CLOCK → GCLOCK:** A credible second lineage with no element shifting and the strongest scaling to large buckets. Ghost-ring warm admission had no effect at its current sizing and credit. It gets one more try with full-rotation coverage.\n"
}
```

### 14:41:41 thinking



### 14:41:41 chat

I've written `cross-cycle-insights.md` from the three cycles in the ledger, using the four required sections in order. I didn't read the previous file, and it contains no numbers.

The main patterns it records:

- **Recurring wins:**
  - All three Winners so far come from one approach: inserting new items below reused ones (cold insertion).
  - The CLOCK/GCLOCK line is a second strong line. It costs no hit-rate and does best in the large buckets.
- **Recurring failures:**
  - Fixed rules that apply to every workload split the results by trace: hm_0 wants a large protected region, web_0 wants deep probation.
  - Per-bucket memory structures (sample rings, ghost rings, feedback counters) keep failing because they're sized too small to remember items long enough to be useful.
  - Since cycle 0001, nothing in the insertion line has improved the large buckets.
  - Promotion tweaks don't help while new items still enter at the top of the order.
  - Per-op-time and metadata-size have gone unmeasured for three cycles running.
- **Open questions:**
  - True leader/follower set dueling to choose the probation/protected split per workload.
  - Feedback state sized to the full lifetime of a bucket.
  - Cold insertion and full-rotation ghost admission on GCLOCK.
  - The actual latency cost of each approach.
- **Per-policy notes:** grouped by LIP-style insertion, BIP, the probation floor, bounded promotion, and CLOCK/GCLOCK.

The file is in the cycle 0003 `cross-cycle` output directory.


---

- finished: 2026-10-28T14:41:42+00:00
- duration: 24.52s
- status: ok
- input_tokens: 4
- output_tokens: 1955
- cache_read_tokens: 36868
- cache_write_tokens: 8347
- cost_usd: 0.113266
