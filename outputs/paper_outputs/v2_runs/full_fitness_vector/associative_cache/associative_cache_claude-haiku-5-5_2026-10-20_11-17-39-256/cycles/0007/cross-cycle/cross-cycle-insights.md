## Recurring wins

- Retention signals that separate re-referenced keys from one-shot inserts keep helping hit-rate on workload-a. Frequency counters, the second-chance bit, and the 2-bit re-reference value all show this.
- Behavior-preserving rewrites of an existing mechanism can recover time lost to a state-layout change. The word-level scan over the per-bucket mask recovered put median time that the bit-by-bit walk had lost.
- Keeping the exact global-minimum victim search has been the stable choice. Every shortcut to it has either lost hit-rate or failed to improve the put tail.

## Recurring failures

- Shortcuts to the victim search (rotating window, cached minimum, cached two-candidate pair, unreferenced-only scan, merged halving) either lost hit-rate or did not shorten the put path. The scan length does not appear to be where the put-side cost lives.
- Put-tail fixes aimed at a single suspected source have not moved the tail. The aging rounds (rewritten as bulk-add) and the zero-counter early exit both failed to change it.
- Bucket-local position changes (midpoint promotion, midpoint reinsertion) and the bucket-local ghost ring did not raise hit-rate. Lengthening the ghost ring did not help either. The reinsertion idea itself looks weak here.
- Fingerprint prefilters add a second scan on get and extra write work on put without a hit-rate gain.
- Extra decay on top of the periodic halving erased frequency protection too fast. The periodic schedule already bounds aging.

## Open questions

- Where does the put tail come from on full-bucket puts? The aging rounds and the scan have both been ruled out as the dominant source.
- Do the timing winners and losses survive repeated runs? Many get and put movements were single-run and lacked a matching diff.
- Why did the doorkeeper gain hit-rate in Cycle 0006 and lose it in Cycle 0007? Is the parent or the exact reject rule responsible?
- Do the hit-rate gains from frequency, second-chance, and the 2-bit re-reference value hold on traces other than workload-a and at other cache sizes?
- Does the metadata-size probe count per-bucket and per-slot state? It has reported no change in variants that clearly add state.
- Is the recurring get-side movement caused by struct size or layout?

## Per-policy notes

- Frequency-based victim selection: the first retention gain on workload-a. It needs the exact minimum search, and the periodic halving schedule should be kept as is.
- Second-chance reference bit: gained hit-rate and lowered put cost in one cycle, but grew per-item metadata and added a get-side write per hit. The mechanism is not yet confirmed from overlay code.
- 2-bit re-reference value per slot: gained hit-rate, but its aging rounds raised the put tail. Single-workload evidence so far.
- Per-bucket reference mask: hit-rate parity with its parent and a metadata saving, paid for with get and put time. The word-level scan recovered part of the put time.
- Doorkeeper admission gate: contradictory results (gain in Cycle 0006, loss in Cycle 0007). Hold until reconciled. If retried, use a looser admit-on-second-sighting rule and keep its bitset out of the inline struct.
- Fibonacci hash on bucket index: no hit-rate change, so bucket collisions are not the miss driver. Its timing Winner status is probably noise until re-measured.
- Narrowed or cached victim-search variants (rotating window, cached pair): not productive; drop as put-side fixes.
- Bucket-local ghost ring and midpoint variants: not productive; do not tune further.
