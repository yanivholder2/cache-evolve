# Cycle 0002 Insight

## Cohort outcome

Three variants were tried: two tweaks to the segmented LRU (a ghost list for direct-to-protected re-admission, and a smaller protected share) and a fresh CLOCK second-chance rewrite of the baseline. Only the CLOCK variant (v001 in the winners list, per its hit-rate gain over strict LRU) won. Both segmented-LRU refinements slightly lowered hit-rate.

## Patterns

- Both tweaks to the segmented design lost hit-rate. Each one weakened the protected segment, either by letting ghost-hit keys displace proven entries or by shrinking it.
- Evidence in this cycle points toward a large, stable protected set. The useful direction is at or above the current split, not below it.
- The CLOCK variant won a small hit-rate gain through a different mechanism. Unreferenced insertion plus a second-chance bit gives scan resistance without list manipulation.
- Metadata-size and per-op-time were again missing from every fitness vector. All size and speed claims, including CLOCK's cheaper hits and smaller nodes, are still unverified.
- All results come from one trace (the scan-heavy LIRS workload) at one cache size. The gains and losses are small.

## Open questions

- Is the measurement gap for metadata-size and per-op-time still unresolved? It has now blocked judgment for two cycles.
- Does the CLOCK gain hold on other traces and cache sizes, or is it specific to this one?
- Would a gentler ghost use (insert at probation MRU, or adaptive segment sizing as in ARC) help where the direct-to-protected version hurt?
- Does a protected share above 80% help further?

## Continuation

Fix or confirm metadata-size and per-op-time measurement before proposing more variants, because otherwise CLOCK's main claimed advantage cannot be judged. Build on the CLOCK winner next, for example with a CLOCK variant that keeps frequency or ghost information, and check it on more traces. Drop the smaller protected share and unconditional ghost-to-protected admission; if the segmented line is revisited, try a larger protected share or a gentler ghost policy.
