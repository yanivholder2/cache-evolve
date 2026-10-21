# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-20T20:14:57+00:00

## timeline

### 20:15:10 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Scan resistance via insertion policy is what moves hit rate. Capacity and associativity were never the limit (cycles 1–2).\n- Mid-bucket (about len/4) insertion plus gradual transpose-by-one promotion beats plain LRU. It has been reproduced from baseline three or more times (cycles 3, 6, 8). It is stateless and adds no metadata.\n- Cutting put-path work (no half-bucket shift, hash once) lowers put latency without hurting hit rate (cycle 4). Approximate order among displaced entries doesn't matter. Where the new entry lands does.\n- The only adaptive gain with a real mechanism came from a global hit/miss counter that switches insertion depth (cycle 8). It gave better hit rate and cheaper puts, and it fit in existing padding.\n- Metadata has never grown. The 40-byte budget has never been the binding constraint so far.\n\n## Recurring failures\n\n- Get latency has had no real gain in nine cycles. Item movement, hashing, arithmetic, layout (tags, alignment) and extra per-hit logic have all failed. Every per-hit addition (flag, counter, compare, swap) taxed get.\n- Retuning the insertion constant (len/3, len/5) and other position-only tweaks are saturated. len/4 is a local optimum.\n- Stacking scan-resistance mechanisms (reference bit plus low insertion, a second reuse gate on promotion) doesn't compound. It costs both hit rate and time.\n- Faster or position-gated promotion (geometric jumps, two-slot jumps) removes the frequency filter that transpose-by-one provides. Hit rate drops or gains stay marginal.\n- Stateless randomization (hash-parity victim, hashed MRU escape) carries no reuse signal and only adds branches.\n- Weak signals fail: one-entry ghost filters, victim-position proxies for adaptivity, and per-slot frequency counters.\n- Micro-fusing the put scan, restructuring put-hit control flow and alignment-only tweaks give noise, not wins. The apparent put-tail gains didn't reproduce.\n- Winner status is repeatedly decoupled from a variant's own evidence. Seven straight cycles include Winners whose insights report losses. Put-latency gains with no mechanism have been noise every time.\n\n## Open questions\n\n- Which part of the cycle 8 adaptive gain is the global hit/miss signal? Can it be updated cheaply, off the get path or with amortized sampling?\n- Can swap insertion keep its put saving without demoting the len/4 occupant to the tail (for example, shift a short span instead)?\n- Where does get time go? Harness overhead, dispatch, and the key/value path are untested.\n- Is the Winner label meaningful at all? A noise check and repeated runs are needed.\n- Do any results hold on a second workload, or are they specific to workload-a's scan-heavy traffic?\n- Can a real reuse signal fit under the metadata budget and pay for its cost?\n\n## Per-policy notes\n\n- LRU baseline: wider buckets and partial promotion don't help. Rotate versus remove+push has no effect on get.\n- CLOCK reference bit: it helps alone by making put and the get tail cheaper. It loses when stacked on low insertion.\n- Frequency counter: marginal hit gain, slows get and put. Redundant once scan resistance exists.\n- LIP/BIP-style low insertion: good alone. Pushing all the way to the LRU end starves new entries.\n- Mid insertion plus transpose: the settled baseline. Swap-based len/4 is cheap on put but loses some hit rate to the displaced entry.\n- Adaptive insertion depth (len/4 versus len/2): the best lead when driven by a global hit/miss counter. Proxies based on victim position don't reproduce it.\n- Ghost admission filter: a single-entry ghost is too weak. A deeper or more selective history is untested.\n"
}
```

### 20:15:15 chat

I wrote `cross-cycle-insights.md` to the cycle 0009 output directory. It has the four required sections in the fixed order: Recurring wins, Recurring failures, Open questions and Per-policy notes. I built it only from the ledger digest in the prompt and didn't read the previous insights file.

The main points it records:

- **Wins:** Insertion at about len/4 plus one-step transpose promotion has been reproduced from baseline several times. The only adaptive gain with a real mechanism came from the global hit/miss counter in cycle 8.
- **Failures:**
  - Get latency has had no real gain in nine cycles.
  - Tuning the insertion position, stacking scan-resistance mechanisms, and faster promotion all fail to help.
  - Winner labels often contradict a variant's own evidence.
- **Open questions:**
  - Whether the cycle 8 adaptive signal can be updated cheaply.
  - Whether swap insertion can avoid demoting the displaced entry.
  - Where get time actually goes.
  - Whether the Winner status survives a noise check.
  - Whether any result holds on a second workload.
- **Per-policy notes:** These are grouped under LRU, CLOCK, frequency counter, low insertion, mid insertion with transpose, adaptive depth and ghost filter.

I kept the numbers qualitative, as instructed.


---

- finished: 2026-10-20T20:15:15+00:00
- duration: 17.57s
- status: ok
- input_tokens: 4
- output_tokens: 1915
- cache_read_tokens: 40727
- cache_write_tokens: 12731
- cost_usd: 0.074998
