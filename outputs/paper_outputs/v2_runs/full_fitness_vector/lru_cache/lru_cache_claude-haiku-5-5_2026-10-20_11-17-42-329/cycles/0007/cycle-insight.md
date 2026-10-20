# Cycle 0007 Insight

## Cohort outcome

The cohort tried three levers: extra mixing in the hasher's finish step (v001), promoting returning ghost keys straight into the protected segment (v002), and packing the flags into the top bits of the link field (v003). None of them improved per-operation cost while keeping policy intact. v003 bought a real node-size reduction at a latency cost, and the insights record no clear helped dimension for v001. The Winner labels for v001 and v003 are not supported by their insights' Helped sections, so the fitness vectors should be checked before crediting them.

## Patterns

- Extra finish-step mixing gave no spread benefit on integer keys and added per-lookup cost on both `get` and `put`. Together with the cycle 0006 hasher win, the gain on the key-map path comes from cheaper hashing, not from more mixing.
- Ghost-driven admission into protected lost hit-rate on the workload tested. Recency of re-reference did not pick out keys worth protecting, and the miss path picked up extra cost. The ghost idea has weakened since cycle 0006.
- Masking flag bits out of the link field removed the padding on the node and was policy-neutral, but every index read and write paid the mask. The saving is a size-for-latency trade, not a free change.
- Across the three variants, node or struct growth and per-operation slowdowns appeared without matching hit-rate gains. Changes that touch the shared index helpers have been the most reliable source of per-operation regressions.
- The v002 node growth is not explained by its hypothesis text and has not been checked against the diff. Its per-operation attribution is also inferred.

## Open questions

- Is the v002 node growth a side effect of the diff rather than the hypothesis, and does moving the ghost set out of the node path remove it?
- Can the flag-bit mask be kept off the `get` path, for example by folding it into a single index helper or keeping flags in a parallel array, so that the node-size gain survives?
- Does the ghost's re-reference signal carry value on workloads other than workload-a at 2048?
- Do the fitness vectors actually show a Winner-level gain for v001 and v003, or were they labelled Winners on size alone?

## Continuation

Next cycle, keep the zero-state map hasher as the base and do not add further finish-step mixing. Drop ghost-to-protected promotion; if the ghost idea is pursued again, keep ghost hits probationary and measure the ghost's hit rate before changing admission. Pursue the flag packing only if the mask is removed from the read path, otherwise prefer the byte-sized flags. Re-measure per-operation deltas before crediting any of these changes.
