# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Shrinking `protected_cap()` from 4/5 to 3/4 of capacity leaves less room for keys that earned protection through a second reference, so on the scan-heavy workload-a trace more re-referenced keys are demoted back to probation and lost before they are hit again. The hypothesis predicted the opposite direction, and this is the one metric the change directly targets.
- per-op-time: The change is a single integer constant in `protected_cap()`, and the get and put code paths, including `select_victim`, `demote_prot_head`, `unlink`, and `push_back`, are byte-identical to the parent. Nothing in the diff adds work per operation, so the apparent latency movement is most likely measurement variance or code-layout effects rather than something this edit caused. The tail moved more than the median, which points toward noise.

## Takeaway

On workload-a at 2048 entries, cutting the protected share from 4/5 to 3/4 costs hits, which shows the protected segment is doing real work for this trace and that the 4/5 ratio is not oversized. The `metadata-size` dimension did not move, as expected for a constant-only change.
