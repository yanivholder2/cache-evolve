# Variant v002 Insight

## Helped

- metadata-size: The two-hit gate added zero bytes — its "referenced" marker rode the otherwise-spare high bit of `next` via `REF_MASK` (mirroring `PROT_MASK` on `prev`), so `Node<u64,u64>` stayed in the 24-byte class exactly as the hypothesis required.

## Hurt

- hit-rate: Requiring a *second* probationary hit before promotion delayed protection of genuinely-reused keys on workload-a; with reuse distances short enough that the first-hit refresh leaves keys exposed at the probation MRU, some about-to-be-hot entries were evicted before earning protected status, nudging hit-rate below the parent rather than above it.
- per-op-time: The probationary `get` branch grew an extra `referenced()` test plus a full `unlink`+`push_back` refresh on the first hit (instead of a single promote), and every `next`-link read/write now masks/preserves `REF_MASK`, so both get and put p50/p99 rose over v001.

## Takeaway

On this trace the two-hit frequency filter was a net-negative lever: it bought no retention headroom and instead cost both hit-rate and latency, because workload-a's reuse is short-distance enough that a second-hit gate mostly delays promotion of truly-hot keys rather than screening out one-hit-wonders. The in-node bit-packing did prove the gate can be added with no size cost — the mechanism is free, but the policy it encodes moved the wrong way here.
