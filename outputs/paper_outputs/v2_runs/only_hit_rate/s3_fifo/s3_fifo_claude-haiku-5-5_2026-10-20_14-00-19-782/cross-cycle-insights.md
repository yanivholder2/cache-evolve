## Recurring wins

- Behavior-preserving refactors (resident-key index, probe removal, bit-folding, hasher swap, ghost-structure changes) keep hit-rate flat across many cycles. That is a reliable property of the changes, but the cost gains they target have never been reported, so none is a confirmed win.
- The only measured hit-rate gain is the cold-victim ghost feed (cycle 0009, main-queue evictions at zero frequency routed into the ghost queue). It is the one ghost-side change that has moved hit-rate, and cycle 0010 supports its shape: a capped side ring lost the gain, and a warm-victim feed was flat. It is still a single-trace, marginal result.

## Recurring failures

- Ghost re-admission routing keeps losing or failing to help: widening the ghost window (0003), routing ghost hits through the small queue (0004), and giving re-admissions an extra main-queue pass (0007). A capped side ring for main-fed entries also lost (0010).
- Frequency levers look inert or negative: raising the main-queue frequency cap (0002), halving the decay step (0005), and carrying frequency 1 into main on re-admission (0007) produced no signal or a loss.
- Promotion-bar and promotion-timing tweaks do not change residency: relaxing the bar to one hit lost hits (0001), raising it was neutral (0003), and earlier promotion of second-hit entries only moved timing (0008).
- Adaptive small-queue sizing driven by ghost hits did not beat the fixed split (0005). Ratio tweaks in the first cycle were also neutral.
- Retaining warm main-queue residents with a tail reinsert lost hits (0006).

## Open questions

- Does the cold-victim ghost feed hold on other traces and at other capacities? Every cycle so far used lirs workload-a at capacity 2048.
- Does the cold-victim gain survive the unchanged linear ghost scan and `retain` cost, or does the scan cost offset it?
- Do any of the cost-side changes (resident index, probe removal, bit-folding, hasher swap, ghost refactor, early return) lower per-op-time or metadata-size? Ten cycles in, no run has reported either.
- Did the intended overlay edits land in the built code? Several variants across cycles could not confirm their edits, and some flat or negative results may reflect no-op or mismatched builds.

## Per-policy notes

- Small queue: sizing (fixed and adaptive) and promotion bar (one-hit, two-hit, three-hit, timing) have shown no reliable hit-rate signal on this trace.
- Main queue: frequency cap, decay, reinsert position, and warm-victim retention have not helped. Cold-victim eviction is the only main-side path tied to a gain, and it works through the ghost queue.
- Ghost queue: the cold-victim feed is the lead hit-rate candidate. Window size, small-queue routing, re-admission frequency, and fingerprint or structure changes have not produced gains. Its lookup cost remains unmeasured.
- Resident index and membership: removing probes, folding membership into state bits, early return, and hasher swaps are cost-side work, all hit-rate neutral, and all unmeasured on cost.
- Evidence scope: all cycles are S3-FIFO, one trace (lirs workload-a), one capacity (2048).
