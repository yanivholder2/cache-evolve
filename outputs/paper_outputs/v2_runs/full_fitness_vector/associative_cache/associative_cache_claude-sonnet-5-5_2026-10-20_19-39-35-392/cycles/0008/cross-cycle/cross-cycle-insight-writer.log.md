# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-10-20T20:11:21+00:00

## timeline

### 20:11:33 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Stateless or near-stateless scan resistance via insertion position works. Mid-bucket (about len/4) insertion plus gradual transpose-by-one promotion beat plain LRU, reproduced from baseline several times (cycles 0003, 0006, 0008).\n- Cheapening the put path pays off when it removes real work. Swap-based O(1) mid-insertion replaced the half-bucket shift, and hash-once indexing cut redundant arithmetic. Approximate order among displaced entries does not matter; where the new entry lands does.\n- Policies that keep one-hit-wonders from displacing reused entries move hit rate. Capacity and associativity do not (cycles 0001, 0002).\n- Adaptive insertion driven by a global hit/miss counter (cycle 0008) is the only recent mechanism-backed gain. It fits in existing padding.\n\n## Recurring failures\n\n- Get latency has not improved in eight cycles. Item movement, hashing, arithmetic, bucket layout, tag-first scans and alignment are all ruled out. Every per-hit addition (flag, counter, branch) taxed the hot path.\n- Stacking scan-resistance mechanisms (reference bit + low insertion, extra gating on promotion) does not compound and usually loses.\n- Tuning the insertion constant is exhausted. len/3 and len/5 both failed to beat len/4.\n- Faster or gated promotion (geometric jump, seen-once gate) hurts hit rate. Transpose-by-one already acts as a frequency filter.\n- Stateless hash-bit randomization for eviction adds noise and branches, with no reuse signal.\n- Weak admission signals (one-entry ghost per bucket) cost put time and hit rate.\n- Micro-fusing or restructuring the put path (single-scan, put-hit tweaks) adds variance and silently changes placement. Most puts are misses.\n- Winner status is often disconnected from a variant's own evidence. Many Winners have insights reporting losses or put-latency gains with no mechanism. Treat these as likely noise.\n- Allocation-only layout tweaks (alignment) did not reproduce their put-tail gain.\n\n## Open questions\n\n- Where does get time go if not bucket memory, hashing or reordering? Harness, dispatch, or the key/value path are untested.\n- How much of the adaptive-insertion gain comes from the adaptive switch versus fixed swap insertion? A fixed-position control is missing.\n- Can adaptivity be driven from the put-miss path alone, to leave the get hot path untouched?\n- Does v002's unintended placement (first-empty-slot fill, victim overwrite) help hit rate when adopted deliberately?\n- Is there a middle ground on promotion, such as a jump only after a second hit?\n- Can a real reuse signal fit in the 40-byte metadata budget? No variant has grown metadata so far.\n- Do any gains hold on a second workload, and survive a noise check? Everything so far is on workload-a only.\n\n## Per-policy notes\n\n- LRU (baseline): position-only tweaks, wide buckets and rotate-based moves were all dead ends (cycle 0001).\n- CLOCK reference bit: cheap, and it reduced reordering, but it did not stack with low insertion.\n- Frequency counter: small gain, with a per-hit and per-put latency cost. Dropped.\n- LIP/BIP-style low insertion: clearest early gain. Pushing too deep starves new entries, so mid-bucket is the sweet spot.\n- Mid-insertion + transpose (swap-based, about len/4): current settled baseline. Its cost is a data-dependent put tail.\n- Adaptive len/4 vs len/2 insertion: current lead. Better hit rate and cheaper puts, with a slightly higher median get.\n- Ghost/admission filters: single-entry versions lose. Deeper or more selective history is untested.\n"
}
```

### 20:11:39 chat

I wrote `cross-cycle-insights.md` in the cycle 0008 cross-cycle output directory. It has the four required sections in order: Recurring wins, Recurring failures, Open questions, Per-policy notes.

- **Recurring wins:**
  - Mid-bucket (about len/4) insertion with transpose-by-one promotion has beaten plain LRU repeatedly.
  - Swap-based O(1) insertion cut put cost.
  - Adaptive insertion from cycle 0008 is the latest mechanism-backed gain.
- **Recurring failures:**
  - Get latency has not improved in eight cycles.
  - Stacked scan-resistance schemes, insertion-constant tuning, aggressive or gated promotion, and hash-bit eviction have all failed.
  - Winner labels often contradict a variant's own insight, so the file says to discount them.
- **Open questions:**
  - Where get time actually goes.
  - How much of the adaptive gain comes from the adaptive switch versus fixed swap insertion.
  - Whether adaptivity can be driven from the put-miss path alone.
  - Whether results hold on a second workload.
- **Per-policy notes:** LRU, CLOCK reference bit, frequency counter, low-position insertion, mid-insertion with transpose, adaptive insertion and ghost filters.

I kept everything qualitative, with no invented numbers, and built it only from the ledger digest.


---

- finished: 2026-10-20T20:11:39+00:00
- duration: 17.85s
- status: ok
- input_tokens: 4
- output_tokens: 1923
- cache_read_tokens: 39875
- cache_write_tokens: 11862
- cost_usd: 0.071432
