# Cross-Cycle Insights

_Ledger covers 1 completed cycle (0001). Nothing below has been confirmed across more than one cycle yet. Treat every item as provisional._

## Recurring wins

- No pattern has repeated across cycles yet. Cycle 0001 suggests two candidates:
  - Changing admission and residency structure (W-TinyLFU window plus SLRU) improves hit-rate.
  - Making hashing cheaper (one cheap hash instead of repeated SipHash) improves per-op time.
- The hit-rate gains were largest at small cache sizes and much smaller at large ones. Future hit-rate ideas should be judged mainly at small capacity.
- Hit-rate and latency problems looked separable: each winning fix stayed on its own axis. Combining them is expected to cause little conflict, but this has not been tested.

## Recurring failures

- No failure has repeated across cycles yet.
- Aging alone, on top of the u32 counter layout (periodic halving, capped counters), raised hit-rate but made median latency worse. Do not try it again in that form. Only retry it with packed, low-cost counters.
- Several changes made `get` p99 worse, each for a different reason (region branching and relinks; sketch memory misses). The get tail is the weakest per-op-time dimension so far.

## Open questions

- Does window/SLRU structure plus sketch aging improve hit-rate more than either alone, or does the window already handle most of the stale-hot-key problem?
- Is the get-tail regression under single-hash real, or is it benchmark noise? Would a cache-line-blocked sketch fix it?
- Can the per-node region tag be packed into spare bits so metadata cost goes back to the baseline level?
- The `put` tail improved under both hit-rate variants, with no explanation. Is that gain real?

## Per-policy notes

- **Baseline (strict admit-or-drop, sketch that never forgets, SipHash):** two separate weaknesses. Admission policy held back hit-rate, and hashing cost held back latency.
- **W-TinyLFU window + SLRU:** clearest hit-rate gain, mostly at small sizes. Costs: higher `get` p99 and a per-node region byte that padding turns into a full word.
- **Sketch aging (halving, capped u32 counters):** some hit-rate gain, but median latency got worse. Packed 4-bit counters are the planned way to fix this.
- **Single cheap hash:** largest per-op-time gain. Exposed sketch memory misses in the `get` tail.
- **Planned next:** merge the window/SLRU structure with the single-hash fast path, add packed 4-bit aging, then target the get tail (cache-line-blocked sketch) and remove the region byte.
