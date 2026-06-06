# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-06-06T06:11:26+00:00

## timeline

### 06:11:38 thinking



### 06:11:38 tool: Bash

```json
{
  "command": "pwd && ls",
  "description": "Show working directory and contents"
}
```

### 06:11:47 thinking



### 06:11:56 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- Latency wins come from removing hidden hot-path cost, not algorithmic\n  cleverness: O(n) front-shifts, an `Instant::now()` syscall, a candidate\n  scan, trait-object dispatch, and SipHash overkill on integer keys have each\n  paid off when deleted.\n- `Item` slimming is near-free under the positional eval policy: every\n  metadata field the policy never reads (timestamp, LFU counter) is pure\n  size+latency overhead, and removing the last one collapsed alignment padding\n  to a key+value floor.\n- The container/segment pivot pays where per-entry tuning stalls: relocating\n  the promotion-heat signal into a segment-side map reached the long-sought\n  Pareto corner (best hit-rate AT the metadata floor) that the\n  counter-carrying layout could not.\n- Structural latency changes are harness-resolvable when micro-reorders are\n  not: inner-policy devirtualization + inlining, and trivializing the\n  side-table hasher, both moved median/tail above the noise floor.\n- MRU placement (`InsertPosition::Back`) reliably captures the trace's warm\n  mass and was a safe foundation from cycle 0001 onward.\n\n## Recurring failures\n\n- Hit-rate tuning is exhausted across every class tried: placement, gate\n  width, gate shape (positional vs. frequency), associativity, and\n  admission/ghost-list all left hit-rate at or below noise. Hit-rate is\n  capacity/compulsory-bound, not conflict- or admission-bound.\n- Promoting *more* keys (ungated promotion) floods the probing tier and evicts\n  resident keys — the hottest-only / bounded gate is load-bearing.\n- Micro-latency reorders (single scan/branch per hit) sit below the\n  measurement noise floor and sometimes move the wrong way.\n- Stacking the two banked corners fails: side-table relocation and hot-path\n  devirtualization are substitutive on the shared get path (the side-table\n  hash probe, not the vtable, bounds latency), so they do not compose.\n- Removing indirection by fattening layout backfires: outer-Bucket\n  monomorphization and a position-aligned parallel `Vec<u8>` both regressed by\n  hurting locality / paying O(n) ride-along shifts.\n\n## Open questions\n\n- The put-p99 tail has resisted every lever and is gated by an unidentified\n  mechanism the side table cannot reach — what bounds it, and is it reachable?\n- Hasher swap (median-bound) and capacity pre-reservation (get-tail/rehash-\n  bound) target disjoint causes — do they compose into a single additive\n  win, unlike the substitutive stacks?\n- Is there any hot-path layout where side-table relocation and\n  devirtualization stop competing, or are the size corner and latency floor\n  mutually exclusive?\n\n## Per-policy notes\n\n- ClassicLRU (active eval policy): purely positional — ignores all\n  recency/frequency metadata, making such fields dead weight and justifying\n  the `Item` floor. Its reorder is a remove+push that costs O(n) under shift,\n  so any data made to ride along positionally (parallel arrays) pays that cost\n  on every hit; target-only hashed side-table probes avoid it. Gate shape and\n  MRU placement act as substitutes here, not additive levers.\n"
}
```

### 06:12:02 chat

Wrote `cross-cycle-insights.md` as a fresh projection of the 6-cycle ledger, using the four fixed sections:

- **Recurring wins** — latency from removing hidden hot-path cost, `Item` slimming under positional eval, the container/segment pivot reaching the Pareto corner, structural (vs. micro) latency changes being measurable, and MRU placement as a safe base.
- **Recurring failures** — hit-rate fully capped across all classes, over-promotion churn, sub-noise micro-reorders, the substitutive size+latency stack, and layout-fattening "de-indirection."
- **Open questions** — the unexplained put-p99 tail, the orthogonal hasher-swap + capacity-reservation compose, and whether the size corner and latency floor can ever coexist.
- **Per-policy notes** — ClassicLRU's positional nature: dead metadata, O(n) remove+push penalizing ride-along data, gate/placement as substitutes.


---

- finished: 2026-06-06T06:12:02+00:00
- duration: 36.09s
- status: ok
- input_tokens: 5
- output_tokens: 2429
- cache_read_tokens: 63520
- cache_write_tokens: 9719
- cost_usd: 0.096779
