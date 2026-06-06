# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-06-06T06:56:05+00:00

## timeline

### 06:56:25 thinking



### 06:56:25 chat

I'll write the cross-cycle insights as a fresh projection of the ledger digest.

### 06:56:35 tool: Write

```json
{
  "file_path": "cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Latency wins come from removing hidden hot-path cost, not cleverness:\n  eliminating an `Instant::now()` syscall, an O(n) front-shift, a\n  candidate-scan branch, vtable dispatch on the inner policy, and — biggest —\n  the keyed SipHash permutation on integer keys (swapped for a trivial `u64`\n  hasher) all paid off cleanly.\n- Structural change beats per-entry tuning *when it preserves locality*:\n  devirtualizing the inner policy and trivializing the side-table hasher both\n  cleared the noise floor; relocating the heat signal out of `Item` into a\n  segment-side map reached the Pareto corner (best hit-rate AT the metadata\n  floor).\n- Placement (MRU / `InsertPosition::Back`) was the early, durable hit-rate win\n  and captures most protectable warm mass on this trace.\n- Capacity pre-reservation / ~2× sizing is a safe, hit-rate-neutral tail/variance\n  reducer (cuts rehash spikes and slot-collision perturbation).\n- Metadata slimming to the byte floor was free under the active policy and\n  composed orthogonally with placement.\n\n## Recurring failures\n\n- Every hit-rate lever beyond placement keeps returning null: wider/reshaped\n  promotion gates, frequency gating, raised associativity, and admission/ghost\n  lists all left hit-rate at the recency-only ceiling — it is\n  capacity/compulsory-bound, not conflict-bound.\n- Micro-latency moves sit below the harness noise floor and often regress:\n  branch reorders and deferring \"wasted\" index arithmetic moved metrics the\n  wrong way.\n- Stacking the two banked Pareto winners (side-table size + devirtualization)\n  is substitutive, not additive — they compete on the shared get path.\n- Removing indirection by *fattening layout* backfires: outer-Bucket\n  monomorphization and a position-aligned parallel `Vec<u8>` both regressed by\n  hurting locality / adding O(n) reorder shifts.\n\n## Open questions\n\n- Is there a conflict-resolution path that does NOT terminate in the stash, so\n  the put-tail win and the get-median cost decouple instead of trading?\n- Residency cleared the long-standing cap only by a hair at steep put-p99 cost —\n  is any meaningful hit-rate above it reachable at tolerable tail, or is the cap\n  the practical ceiling?\n- Are the characterized trades (shallow cascade: put-p99↓ / hit-rate↓; deeper\n  cascade: hit-rate↑ / put-p99↑; relocated put clear: p50↓ / tails↑) worth\n  banking, or only worth a strictly-free version?\n\n## Per-policy notes\n\n- ClassicLRU (active eval policy) is purely positional/recency: it never reads\n  `timestamp` or the LFU counter, which made those fields pure size+latency dead\n  weight and enabled metadata slimming. Hit-rate is effectively capped under it\n  across placement, gate, associativity, and admission axes.\n- Dash/side-table segment: get median is hasher/probe-bound (a hard floor once\n  the hasher is trivialized), and the median get hit actually resolves in the\n  STASH, not the promoted tiers. The stash is a shared resource coupling\n  put-side cascade work to get-side latency — cascade depth is a bidirectional\n  latency↔hit-rate dial, and any cascade economy ending in the stash pays at the\n  get median. The put-p99 tail is owned by eviction/cascade work, not the\n  side-table touch.\n"
}
```

### 06:56:42 chat

Wrote `cross-cycle-insights.md` as a fresh projection of the 9-cycle ledger. Summary of what each section captures:

- **Recurring wins**: latency gains come from removing hidden hot-path cost (syscalls, O(n) shifts, SipHash→trivial `u64` hasher, devirtualization); locality-preserving structural moves and the side-table relocation reach the Pareto corner; MRU placement, capacity pre-reservation, and metadata slimming are durable/safe.
- **Recurring failures**: all hit-rate levers beyond placement (gate shape, associativity, admission) return null — it's capacity-bound; micro-latency reorders sit below noise; stacking the two banked winners is substitutive; removing indirection by fattening layout backfires.
- **Open questions**: a conflict-resolution path that doesn't terminate in the stash, whether residency above the cap is reachable cheaply, and whether the characterized cascade trades are worth banking.
- **Per-policy notes**: ClassicLRU's positional-only behavior (enables slimming, caps hit-rate) and the dash/side-table stash coupling (median lives in the stash; cascade is a bidirectional dial; put-p99 owned by eviction).


---

- finished: 2026-06-06T06:56:43+00:00
- duration: 37.52s
- status: ok
- input_tokens: 3
- output_tokens: 2473
- cache_read_tokens: 42673
- cache_write_tokens: 12957
- cost_usd: 0.105917
