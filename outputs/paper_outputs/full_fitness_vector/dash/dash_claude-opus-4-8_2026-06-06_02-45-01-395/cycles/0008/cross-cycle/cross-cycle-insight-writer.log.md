# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-06-06T06:42:43+00:00

## timeline

### 06:43:02 thinking



### 06:43:03 tool: Bash

```json
{
  "command": "ls && echo \"---\" && ls output 2>/dev/null; echo \"pwd:\"; pwd",
  "description": "List working directory to find output location"
}
```

### 06:43:16 thinking



### 06:43:24 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- Removing hidden hot-path cost is the most repeatable latency win: O(n)\n  front-shifts, `Instant::now()` syscalls, and SipHash-on-integer-keys all\n  trimmed op-latency once excised, with no algorithmic cleverness needed.\n- Structural latency changes that *remove indirection while preserving\n  locality* pay off: devirtualizing the inner eviction-policy trait object,\n  and swapping the keyed permutation for a trivial `u64` hasher on the\n  side-table probe, both cleared the noise floor.\n- Per-entry metadata slimming was a near-free size+latency win until it\n  bottomed out — every field the positional policy never reads (timestamp,\n  LFU counter) was pure padding once removed.\n- Relocating the promotion-heat signal out of `Item` into a segment-side\n  table reached the long-sought Pareto corner: best hit-rate held at the\n  metadata floor.\n- Variance reducers are safe and hit-rate-neutral: capacity pre-reservation\n  and a 2×-sized direct-mapped array cut p99 tails and put-p50 bit-for-bit.\n\n## Recurring failures\n\n- Tuning hit-rate via placement, gate width, gate shape, set-associativity,\n  or admission/ghost lists — all saturate. Gains are noise-level because\n  hit-rate is capacity/compulsory-bound, and gate shape is a *substitute* for\n  MRU placement, not additive.\n- Sub-scan micro-optimizations (branch reorders, removing one scan per hit)\n  sit below the harness noise floor and often move latency the wrong way.\n- Stacking two banked winners on one hot path keeps failing: the side-table\n  size corner and the devirtualized latency floor are *substitutive* on the\n  shared get path (the hash probe, not the vtable, bounds it).\n- Moving indirection the wrong way regresses locality: outer-layer\n  monomorphization fattened the `Vec` stride; a position-aligned parallel\n  `Vec<u8>` paid an O(n) shift on every reorder. Heap pointers were buying\n  locality.\n- Relocating work *off* the median path is not free — it resurfaces on the\n  destination branches as a tail (p99) regression (per-put heat clear,\n  eviction-site moves).\n\n## Open questions\n\n- The get median resolves in the **stash**, not the promoted tiers — does a\n  stash-*first* reordering actually win get-p50, or just relocate the cost?\n- The put-p99 cascade is now a priced latency-vs-hit-rate trade (single-hop\n  demotion rescues a few hits). Is it worth banking, and is there a cheaper\n  conflict-mitigation step that keeps most rescued hits?\n- Is get latency fully converged at the hasher floor, or is any median lever\n  left?\n- \"Hit-rate fully capped\" is no longer exact: the demotion/conflict machinery\n  shows real sensitivity. How much hit-rate does the cascade actually buy?\n\n## Per-policy notes\n\n- **ClassicLRU (active eval policy):** purely positional/recency-driven. It\n  reads no frequency or recency metadata, so all such fields are dead weight\n  on size and latency, and frequency/admission levers move hit-rate not at\n  all. Warm mass already concentrates near the MRU head, so MRU placement\n  captures most protectable retention and makes gate-shape tuning redundant.\n  Hit-rate is recency-bound and largely capacity/compulsory-limited, with the\n  only known crack in the demotion-cascade trade.\n"
}
```

### 06:43:31 chat

Wrote `cross-cycle-insights.md` as a fresh projection of the 8-cycle ledger. Summary of what it captures:

- **Recurring wins** — removing hidden hot-path cost (O(n) shifts, syscalls, SipHash), locality-preserving devirtualization, metadata slimming, the side-table Pareto corner, and safe variance reducers.
- **Recurring failures** — saturated hit-rate levers, sub-noise micro-opts, substitutive winner-stacking, locality-regressing layout changes, and median→tail relocation.
- **Open questions** — stash-first get-p50, the priced put-p99/hit-rate cascade trade, get-median convergence, and the newly cracked "hit-rate capped" assumption.
- **Per-policy notes** — ClassicLRU's positional/recency-only nature explaining why metadata and frequency/admission levers are inert.


---

- finished: 2026-06-06T06:43:31+00:00
- duration: 47.97s
- status: ok
- input_tokens: 5
- output_tokens: 2605
- cache_read_tokens: 68188
- cache_write_tokens: 12196
- cost_usd: 0.111838
