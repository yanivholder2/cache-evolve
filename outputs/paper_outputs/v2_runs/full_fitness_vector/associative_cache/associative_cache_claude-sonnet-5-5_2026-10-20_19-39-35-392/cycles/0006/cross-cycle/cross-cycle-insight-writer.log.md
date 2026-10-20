# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-20T20:03:09+00:00

## timeline

### 20:03:21 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Stateless, cheap scan resistance moves hit rate: low or mid-bucket insertion with transpose-by-one promotion. Cycles 0002, 0003 and 0006 reproduced this.\n- Cutting put-path cost works. Swap-based O(1) mid-insertion (0004) removed the half-bucket shift, and hash-once indexing also cut put time. Hit rate and metadata were unchanged.\n- Exact order among displaced entries doesn't matter. Only where the new entry lands does.\n- Explicit promotion on hit is needed once insertion is low.\n- Metadata size has never changed in any cycle. Every win so far came without extra state.\n\n## Recurring failures\n\n- Position-only tweaks to LRU and geometry (0001): wider buckets and partial promotion gave no gain.\n- Retuning the insertion point (len/3 in 0005, len/5 in 0006) failed on both sides. len/4 is a local optimum, and the axis is exhausted.\n- Stacking scan-resistance mechanisms (reference bit plus low insertion) did not compound. It lost hit rate and speed.\n- Per-slot frequency counters and one-entry ghost filters add per-access work for little or negative hit-rate gain.\n- Get latency has not improved in six cycles. Item movement, arithmetic, hashing, tag-first scans and alignment are all ruled out. Small get shifts are noise or codegen effects.\n- Allocation-only layout changes such as 64-byte alignment are not reliable. The put-tail gain in 0005 did not repeat in 0006.\n- Latency gains with no mechanistic reason (put latency in 0001, 0002 and 0005) have been noise. Discount them.\n- Winner labels are weakly tied to evidence. Since 0003, many Winners have insights reporting a loss. Treat such Winners as suspect.\n\n## Open questions\n\n- Which Winner statuses are real? A noise check and repeat runs are needed.\n- Where does get time go if not bucket memory, hashing or reordering? Candidates are harness or dispatch overhead and the key/value path. It has never been measured directly.\n- Do any gains hold on a second workload, or are they specific to workload-a's scan-heavy traffic?\n- Can mid-insertion with transpose avoid its put-tail penalty when combined with swap-based insertion?\n- Can the hit-promotion path or a small per-slot signal under the metadata budget give a gain that insertion position can't?\n- Would a deeper or more selective admission history help, or is scan resistance already saturated?\n\n## Per-policy notes\n\n- Baseline LRU: hit rate is insensitive to associativity and to partial promotion. Rotate-based moves gave no get speedup.\n- CLOCK reference bit: it cheapened put and the get tail on its own (0002). Stacked on low insertion it lost (0003), because a one-bit second chance is too coarse.\n- 2-bit frequency counter: small gain, latency cost on every hit and put. Dropped.\n- Low-position (LIP/BIP-style) insertion: the clearest single gain (0002). Going all the way to the LRU end starves new entries, so a hashed MRU escape is marginal.\n- Mid-bucket or len/4 insertion with transpose promotion: the settled policy baseline. Swap-based insertion is its cheap implementation.\n- Ghost admission filter (one entry per bucket): too weak. It slows put and slightly hurts hit rate.\n"
}
```

### 20:03:24 chat

I wrote `cross-cycle-insights.md` in the cycle 0006 cross-cycle output directory. It has the four required sections in order and is built only from the six-cycle ledger digest.

- **Recurring wins:** Cheap, stateless scan resistance is the one thing that has paid off across cycles. That means low or mid-bucket insertion with transpose promotion, done with a swap-based insert. Cutting put-path cost also held up.
- **Recurring failures:**
  - Retuning the insertion point failed in both directions.
  - Stacking scan-resistance mechanisms did not compound.
  - Get latency has not improved in six cycles.
  - Winner labels are weakly tied to the variants' own evidence, so I flagged Winners whose insights report a loss as suspect.
- **Open questions:**
  - Which Winner statuses are real, which needs a noise check.
  - Where get time actually goes.
  - Whether any gain holds on a second workload.
  - Whether the mid-insertion put-tail cost can be avoided.
- **Per-policy notes:** These are grouped by policy: baseline LRU, CLOCK reference bit, frequency counter, low-position insertion, mid-bucket insertion with transpose, and the ghost filter.

I kept everything qualitative and invented no numbers.


---

- finished: 2026-10-20T20:03:25+00:00
- duration: 16.02s
- status: ok
- input_tokens: 4
- output_tokens: 1727
- cache_read_tokens: 38195
- cache_write_tokens: 10033
- cost_usd: 0.061821
