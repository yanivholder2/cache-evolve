## Recurring wins

- Put-path-only edits with a named mechanism have paid off: the ghost-membership hash index and the bounded eviction scan both gained cost on insertion.
- Pure data-structure swaps that leave the policy logic alone keep hit rate unchanged, so they can be taken on cost grounds alone.

## Recurring failures

- Lowering the small-queue promotion bar (`freq > 0`) moves singly-reused keys into main early and displaces useful residents. It showed no attributable gain and a hit-rate loss.
- Hard caps on work per eviction (the 8-rotation second-chance budget) buy cost by forcing evictions of hot entries, so the hit-rate price is paid in the same place the saving appears.
- Fixed per-entry overhead from added index structures offsets part of the win on small or lightly loaded runs.

## Open questions

- Is the get-path latency shift seen across variants real, or a shared environment or measurement artifact? No variant's change explains it, so it needs a rerun before any get-latency credit is given.
- How often does the rotation budget actually run out under the evaluated workloads, and what is its aggregate hit-rate cost?
- Once membership is O(1), does the ghost queue's O(n) pop-back and position-based removal still dominate its maintenance cost?

## Per-policy notes

- Small queue: promotion threshold tuning has not paid off; drop further `freq > 0` variants and look elsewhere for admission gains.
- Main queue: bounded second-chance rotation is the only lever tried so far; its cost-versus-hot-entry-eviction trade is the thing to measure next.
- Ghost queue: membership is now cheap, but maintenance (pop-back, positional removal) remains the candidate bottleneck and is the most promising put-path direction.
- Cross-policy: only one completed cycle so far, so no regularity has yet been confirmed across multiple cycles; treat the above as provisional.
