# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-11-10T04:46:39+00:00

## timeline

### 04:46:57 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **One change per variant.** Bundled changes in cycles 1–2 hid what caused what and kept trading hm_0 against web_0. Once variants became isolated single-change children (cycle 3 onward), the effects could be read cleanly and every real win followed.\n- **Credit only for reuse earned in main.** Gating main-victim ghosts on reuse, resetting freq/reused on graduation (merged in cycle 4), and decrement-by-one aging with a cap of 7 (cycle 5) all reward keys only for reuse inside main. Each of these helped, and their gains added up.\n- **Spend ghost budget selectively.** Filtering what goes into the ghost lengthens the useful history for free. Making the ghost bigger does not.\n- **Gate changes on cache size.** Retention at 131072 is saturated. A change limited to small capacities (the cycle-9 1/5 split at ≤32768) cannot lose at the large size, so a small gain at 16384 survives intact.\n- **Structural changes that cut real work per op.** Slab plus hash indexes (cycle 2) and the Fx hasher (cycle 3) gave timing gains the diffs explain, with hit rate bit-identical. These are the only timing results that earned credit.\n- **Hit-rate-preserving refactors reproduce the parent exactly.** The eviction and ghost semantics are well understood, so implementation-only variants are safe to trust.\n\n## Recurring failures\n\n- **Resizing the small queue globally.** 10% hurt web_0 and 1/3 hurt everywhere, so 1/4 is a local optimum at the large size. Probation length is ruled out as web_0's lever.\n- **Tuning main tenure in either direction.** Halving freq on each sweep, a `MAX_FREQ` of 15, full freq carry-over at graduation, and a graduation credit of 1 all regressed. Tenure a key has not earned in main blocks keys that are actually reused.\n- **Changing ghost→main admission in either direction.** Looser admission (ghost hit → freq 1) and stricter admission (re-admit to small) both lost on web_0 at 131072. Extra in-small reinsertion only duplicated the ghost's second-chance path.\n- **Bigger ghost bounds or resized ghost regions.** These let stale reuse displace live residents, or just moved the hm_0/web_0 trade-off.\n- **Container reshaping for timing.** A ghost ring, a CLOCK ring for main, a merged index, over-allocated indexes, fingerprint keys with the default hasher, and `entry()`-based probe merging all kept hit rate identical but made the `put` tail worse.\n- **Unbounded or hidden per-miss work.** Multi-pop eviction loops, longer sweeps, constant liveness probes, and the hidden `reserve(1)` rehashes behind `entry()` all inflated p99.\n- **Timing credit for policy-only diffs.** The ~1 ns shifts these produced, and the large environmental drift seen in cycle 1, are noise.\n\n## Open questions\n\n- Is the 1/5 share at 16384 the optimum, or would one nearby value or threshold add a bit more? After a single probe, treat hit rate as converged.\n- Is web_0's gap at 131072 out of reach for this policy family? Every structural and admission candidate has failed. Pursue it only if a genuinely new mechanism appears.\n- Can the number of sweep steps per miss be bounded (cap the steps and fall back to the hand position) without changing eviction order in practice?\n- Can a hash computation be removed per miss, for example with fingerprints plus a pass-through hasher, or with `get` followed by `remove` only on a match (no reserve)?\n- Should the metadata metric be supplemented with heap bytes per resident key? Ghost and index heap growth has stayed invisible to it.\n\n## Per-policy notes\n\n- **S3-FIFO (the only policy explored so far).**\n  - Current parent: cycle-9 v001, which is Fx slab + reuse-gated main-victim ghost + reset on graduation + `MAX_FREQ` 7 with decrement aging + small share of 1/5 at ≤32768 and 1/4 above, with the ghost bound at `main_size`.\n  - hm_0 responds to main retention and history. web_0 at large sizes resists every lever tried.\n  - Hit rate is close to converged. All remaining hit-rate room is at 16384.\n  - The `put` tail tracks sweep steps, hash computations, and rare rehash or `retain` events. It does not track bytes touched or container shape.\n  - Per-entry metadata has stayed flat at 24 bytes across every cycle.\n"
}
```

### 04:47:02 thinking



### 04:47:02 chat

I wrote `cross-cycle-insights.md` in the cycle-0009 output dir. It's built only from the ledger digest, with the four required sections in order and no numbers beyond what the ledger itself states.

What it says across the nine cycles:

- **Hit rate is close to converged.** All the remaining room is at the small size (16384). Changes that give keys credit only for reuse inside main have added up. The current parent is cycle-9 v001, the size-gated 1/5 small-queue split.
- **The large size has run out of levers.** Retention there is saturated. web_0's large-size gap has resisted every structural and admission change tried, in both directions.
- **Timing depends on work per miss.** The `put` tail follows sweep steps, hash computations, and occasional rehash events. Reshaping containers and timing gains from policy-only changes have repeatedly not paid off.
- **Next steps:** one more fine-tuning probe of the small share at 16384, then bounding sweep steps per miss and removing a hash per miss for timing. Only S3-FIFO has been explored, so the per-policy notes have a single entry.


---

- finished: 2026-11-10T04:47:03+00:00
- duration: 23.15s
- status: ok
- input_tokens: 4
- output_tokens: 2086
- cache_read_tokens: 42997
- cache_write_tokens: 14755
- cost_usd: 0.169394
