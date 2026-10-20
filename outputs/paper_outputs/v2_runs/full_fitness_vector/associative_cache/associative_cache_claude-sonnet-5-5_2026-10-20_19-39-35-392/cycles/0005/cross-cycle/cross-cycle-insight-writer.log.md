# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-20T19:59:26+00:00

## timeline

### 19:59:38 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Scan-resistant insertion/eviction policy is the only lever that moved hit rate (cycles 0002-0004). Keeping one-hit-wonders from displacing reused entries beat any capacity, geometry or position-only change.\n- Cheap, stateless mechanisms (insertion position, swap-based placement) win over stateful ones. Gains rank by how little bookkeeping they add.\n- Put-path cost reductions are real and repeatable: removing the half-bucket shift (O(1) swap insertion) and redundant hash/modulo work both cut put latency without hurting hit rate.\n- Exact ordering among displaced entries doesn't matter. Where the new entry lands matters.\n- Metadata size has never grown in any cycle; the extra state fit existing padding or wasn't needed.\n\n## Recurring failures\n\n- Get-latency gains: five cycles, none real. Arithmetic, hashing, item movement, bucket layout (tags, alignment) and geometry are all ruled out. Small get shifts look like noise or codegen effects.\n- Position-only LRU tweaks and geometry sweeps (cycle 0001) were dead ends. Associativity is not the hit-rate limit.\n- Stacking scan-resistance mechanisms does not compound (reference bit plus low insertion lost on both axes). Overlapping signals cost more than they add.\n- Added per-access logic (frequency counters, hash-conditional branches, parallel tags kept in sync) slows put and get slightly.\n- Weak admission signals, such as a one-entry-per-bucket ghost, add put work and slightly hurt hit rate.\n- Put-latency gains with no mechanism (cycles 0001, 0002, 0005) keep appearing and are probably noise. Winner labels often rest on this thin evidence, even when the variant's own insight reports a loss.\n\n## Open questions\n\n- Where does get time go, if not hashing, reordering or layout? Candidates are harness overhead, dispatch and the key/value path. This is unmeasured after five cycles.\n- Is the optimum insertion point below len/4 (such as len/5 or len/8)? Moving toward the protected end lost hit rate.\n- Does plain 64-byte alignment reliably improve put tail latency, and does it stack with swap insertion?\n- Which Winner statuses are real versus noise? Several Winners had negative self-reported insights.\n- Do any gains hold on a second workload, or are they specific to workload-a's scan-heavy traffic? No cycle has tested this.\n- Would a deeper or more selective admission history help, or is scan resistance already saturated?\n\n## Per-policy notes\n\n- Baseline LRU with remove+push promotion: hit rate and get are insensitive to geometry, promotion form (rotate) and partial promotion. Partial promotion hurts hit rate.\n- CLOCK reference bit: cheap, and it lowers put cost and get tail when used alone. It hurts when stacked on low insertion, because a one-bit second chance is too coarse.\n- Frequency counter (2-bit, LRU tie-break): marginal hit gain, per-access latency cost. Dropped.\n- LIP/BIP-style low insertion: clearest single hit-rate gain. Inserting at the pure LRU end starves new entries. The sweet spot is len/4 to mid-bucket, with explicit transpose promotion on hit.\n- Mid/len/4 insertion with transpose, using swap-based O(1) placement: current policy baseline. Moving toward len/3 lost hit rate.\n- Ghost admission filter (single entry): net negative. It is not worth pursuing in that form.\n"
}
```

### 19:59:42 chat

I wrote `cross-cycle-insights.md` in the output dir (`cycles/0005/cross-cycle/`). It has the four fixed sections in order, built only from the five-cycle ledger digest.

- **Recurring wins:** Scan-resistant insertion and eviction is the only thing that has moved hit rate. Cheap stateless mechanisms beat stateful ones, and put-path cost cuts (swap-based insertion, hashing once) hold up. Metadata size never grew.
- **Recurring failures:**
  - No get-latency gain has been real in five cycles.
  - Stacking scan-resistance mechanisms doesn't compound.
  - Extra per-access logic slows things down.
  - Winner labels often rest on thin evidence.
- **Open questions:**
  - Where get time actually goes.
  - Whether the best insertion point is below len/4.
  - Whether 64-byte alignment reliably helps put tail latency.
  - Whether any gain holds on a second workload.
  - Which Winners are real.
- **Per-policy notes:** The baseline is LRU. After that come the CLOCK reference bit, the frequency counter, low insertion, mid-insertion with transpose (the current baseline) and the ghost filter.

I used only qualitative terms and no numbers.


---

- finished: 2026-10-20T19:59:42+00:00
- duration: 16.08s
- status: ok
- input_tokens: 4
- output_tokens: 1773
- cache_read_tokens: 37387
- cache_write_tokens: 9276
- cost_usd: 0.059089
