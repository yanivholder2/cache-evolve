# Variant v003 Insight

## Helped

- hit-rate: The fused `promote_with_demote` leaves final ordering and residency byte-identical to v001's promote-then-demote sequence (promotee to protected MRU, protected LRU to probation MRU, `prot_len` net-unchanged), so the SLRU geometry and the 0.7241 record were preserved exactly as the hypothesis required.
- metadata-size: The change is pure control-flow on `get`'s probationary branch — no field added to `Node`, no change to the packed `u32`/`PROT_MASK` layout — so the node stayed in the 24-byte class.

## Hurt

- per-op-time: The "fusion" removed no real work — both paths still do two `unlink`s plus two `push_back`s — so the only net change is an added `prot_len == prot_cap` branch in `get` and a second resident code path (`promote_with_demote` alongside the retained dead-code `demote_protected_lru`). That extra branching and code bloat regressed `get` latency, and dragged the untouched `put` path down with it, consistent with a codegen/layout (inlining, instruction-cache) perturbation rather than any algorithmic saving.

## Takeaway

The hypothesis misdiagnosed the promote+demote pair as redundant churn: the demote-on-overflow case already performs exactly four list surgeries, and routing it through one fused function performs the same four. Collapsing two correct calls into one new routine bought no work reduction while adding a hot-path branch and a parallel code path, so per-op-time moved the wrong way — and the regression spilling onto the unmodified `put` path is the tell that the cost is codegen perturbation, not the cache logic itself.
