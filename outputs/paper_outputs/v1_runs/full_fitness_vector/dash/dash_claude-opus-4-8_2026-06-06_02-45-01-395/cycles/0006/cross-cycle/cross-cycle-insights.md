# Cross-Cycle Insights

## Recurring wins

- Latency wins come from removing hidden hot-path cost, not algorithmic
  cleverness: O(n) front-shifts, an `Instant::now()` syscall, a candidate
  scan, trait-object dispatch, and SipHash overkill on integer keys have each
  paid off when deleted.
- `Item` slimming is near-free under the positional eval policy: every
  metadata field the policy never reads (timestamp, LFU counter) is pure
  size+latency overhead, and removing the last one collapsed alignment padding
  to a key+value floor.
- The container/segment pivot pays where per-entry tuning stalls: relocating
  the promotion-heat signal into a segment-side map reached the long-sought
  Pareto corner (best hit-rate AT the metadata floor) that the
  counter-carrying layout could not.
- Structural latency changes are harness-resolvable when micro-reorders are
  not: inner-policy devirtualization + inlining, and trivializing the
  side-table hasher, both moved median/tail above the noise floor.
- MRU placement (`InsertPosition::Back`) reliably captures the trace's warm
  mass and was a safe foundation from cycle 0001 onward.

## Recurring failures

- Hit-rate tuning is exhausted across every class tried: placement, gate
  width, gate shape (positional vs. frequency), associativity, and
  admission/ghost-list all left hit-rate at or below noise. Hit-rate is
  capacity/compulsory-bound, not conflict- or admission-bound.
- Promoting *more* keys (ungated promotion) floods the probing tier and evicts
  resident keys — the hottest-only / bounded gate is load-bearing.
- Micro-latency reorders (single scan/branch per hit) sit below the
  measurement noise floor and sometimes move the wrong way.
- Stacking the two banked corners fails: side-table relocation and hot-path
  devirtualization are substitutive on the shared get path (the side-table
  hash probe, not the vtable, bounds latency), so they do not compose.
- Removing indirection by fattening layout backfires: outer-Bucket
  monomorphization and a position-aligned parallel `Vec<u8>` both regressed by
  hurting locality / paying O(n) ride-along shifts.

## Open questions

- The put-p99 tail has resisted every lever and is gated by an unidentified
  mechanism the side table cannot reach — what bounds it, and is it reachable?
- Hasher swap (median-bound) and capacity pre-reservation (get-tail/rehash-
  bound) target disjoint causes — do they compose into a single additive
  win, unlike the substitutive stacks?
- Is there any hot-path layout where side-table relocation and
  devirtualization stop competing, or are the size corner and latency floor
  mutually exclusive?

## Per-policy notes

- ClassicLRU (active eval policy): purely positional — ignores all
  recency/frequency metadata, making such fields dead weight and justifying
  the `Item` floor. Its reorder is a remove+push that costs O(n) under shift,
  so any data made to ride along positionally (parallel arrays) pays that cost
  on every hit; target-only hashed side-table probes avoid it. Gate shape and
  MRU placement act as substitutes here, not additive levers.
