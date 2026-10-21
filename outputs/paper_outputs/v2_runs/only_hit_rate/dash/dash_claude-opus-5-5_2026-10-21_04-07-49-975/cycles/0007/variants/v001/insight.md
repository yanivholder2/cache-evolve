# Variant v001 Insight

> Note: I could not list or open the overlay source in this session. I only had file read and write, with no directory listing or diff, and `manifest.toml` was not at the variant root. The mechanism below comes from H01's stated change, which is to make the ghost-miss fresh-key branch always insert at the stash LRU (probation) end and remove the BIP state. I did not check the diff itself.

## Helped

- hit-rate: With BIP's blind MRU admission gone, a fresh key can only reach the protected stash MRU region through a ghost-table fingerprint match. Every protected slot now goes to either a key with evidence of reuse (a ghost hit or a promoted probation hit) or a demoted ex-resident. No one-touch keys are sampled in at random. The improvement continues the trend from the 4× rarer BIP in the parent, so even the residual 1-in-`BIP_PERIOD` admissions were still a net drain on this trace.

## Hurt

## Takeaway

Reuse that BIP caught and the ghost window missed was worth less than the protected capacity BIP spent to catch it. Fresh keys get one probation residency, their stash hits are promoted through the normal threshold, and ghost hits go straight to MRU. Together that covers the useful reuse here, so on this LIRS trace at size 2048 the optimum `BIP_PERIOD` is effectively infinity, not a finite value. Removing BIP also drops its per-segment counter and the per-insert update. The gain is small and was measured on a single trace at a single size, so workloads with reuse distances longer than the ghost window are the open case where removing BIP could still hurt. The ghost window size is now the main lever for admission into protected space.
