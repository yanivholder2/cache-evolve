# Variant v001 Insight

## Helped

- (none): widening the resident-HIR window produced no gain on the only measured dimension.

## Hurt

- hit-rate: Changing `l_hir` from `capacity / 100` to `capacity / 20` (1% to 5%) lowered hit-rate, because `l_lir = capacity - l_hir` shrinks the protected LIR set by the same amount — trading ~80 low-IRR resident slots for a wider one-shot drain queue that did not recover them in reuse.

## Takeaway

The SLRU 95/5 probationary geometry does not transfer to LIRS: in LIRS the HIR window and the LIR budget are zero-sum (`l_lir = capacity - l_hir`), so enlarging the reuse-detection queue directly evicts low-IRR blocks the policy was protecting. Contrary to the hypothesis, LIRS prefers a *narrow* resident-HIR window — the 1% parent configuration was already near its optimum, and 5% over-drains it.
