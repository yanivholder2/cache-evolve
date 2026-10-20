## Recurring wins

- Only one cycle so far, so nothing has recurred yet. The points below are early candidates, not confirmed patterns.
- Moving away from strict LRU recency order raised hit-rate on workload-a every time it was tried. Three different mechanisms did it: segmentation (SLRU), second-chance bits (CLOCK) and loop-resistant insertion (BIP).
- Speed gains tracked memory locality: smaller nodes, no relinking on a hit, and churn concentrated on a few slots. Get and put improved at both p50 and p99.
- Node layout (fewer links, or narrow `u32` indices) is the main lever on metadata-size, and it likely helps speed too.

## Recurring failures

- No failure has repeated across cycles yet.
- Plain LRU with small changes to its order is deprioritized. Every structural departure from it beat it in cycle 0001.
- Changes that leave the node layout untouched (e.g. BIP) don't lower metadata-size. In cycle 0001, BIP added one fixed per-cache word.

## Open questions

- Do the results hold at other cache sizes and on traces other than workload-a? All evidence so far comes from one trace at one size.
- Do the orthogonal mechanisms stack (insertion position, a hit path with no relinking, probation/protection segments), or do they overlap because they all fight scan/loop flushing?
- CLOCK's put tail when most of the ring is referenced and sweeps get long.
- How much of the measured speed gains is real and how much is benchmark noise (BIP especially)?

## Per-policy notes

- **LRU (baseline):** strict recency order is a liability on loop- or scan-heavy access. Treat it as a reference point, not something to keep tuning.
- **SLRU:** second-touch promotion protects reused keys. Narrowing links to `u32` shrank nodes and sped up both ops. It is a natural place to try BIP-style insertion into probation.
- **CLOCK:** it beat expectations on hit-rate and removes relinking on a hit, giving the smallest metadata. The worst-case sweep tail is still untested. It is a candidate base for CLOCK-Pro, a 2-bit clock or a segmented clock.
- **BIP:** loop-resistant insertion gains hit-rate and speed with a cheap change. Metadata stays flat (plus one per-cache word). The insertion idea should carry over to other list-based policies.
