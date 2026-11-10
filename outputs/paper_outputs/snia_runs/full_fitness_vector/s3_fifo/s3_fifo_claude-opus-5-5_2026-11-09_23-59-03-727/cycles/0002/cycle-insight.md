# Cycle 0002 Insight

## Cohort outcome

Two variants were policy tweaks on top of cycle-1 parents: v001 split the ghost into a small-demotee region and a main-victim region, and v002 lowered the graduation threshold to one hit and reset freq on promotion. A third variant, v003, kept the policy and replaced linear scans with a slab plus hash indexes. The two policy tweaks again moved hit-rate gains between hm_0 and web_0 without a net win. v003 matched its parent's hit rates exactly, and it is the first per-op-time gain that the diff itself explains.

## Patterns

- hm_0 keeps rewarding anything that gives reused keys more main residency or history: restoring the demotee ghost, keeping main-victim ghosts, or promoting after one hit. web_0 regressed under both policy variants, so cycle 1's trade-off between the two traces is still in place.
- Both policy hypotheses got web_0's mechanism wrong. Restoring demotee ghost history in full did not rescue web_0 at the large size, which rules out ghost-slot competition as the cause there. The one-hit threshold only affects keys that were already reused, while web_0's loss involves freq-0 keys. The likelier culprits are freq-0 keys re-admitted from the main-victim ghost straight into main, plus the freq reset on promotion. v002 bundled that reset with the threshold change, so the two effects can't be separated.
- Policy and implementation now separate cleanly. v003 shows that structural speedups can be made without changing hit rate, so timing credit can go to real complexity changes. Unexplained timing drift in v001 and v002 is still environmental noise.
- The metadata-size dimension (per-entry footprint) stays flat even when heap use per key grows. Both v001 (ghost key storage) and v003 (hash maps, stale ghost tuples) added memory the metric doesn't see.

## Open questions

- Does ghosting only main victims that once had a nonzero frequency keep hm_0's gain without hurting web_0?
- On web_0, is the freq reset on promotion or the looser threshold the cause of the regression? Test each alone.
- Should metadata-size be supplemented with heap bytes per resident key, now that hashed structures are on the table?

## Continuation

Use v003's slab-plus-hash structure as the base for future policy variants: per-op-time is now solved structurally and comparisons will be faster. Stop resizing ghost regions. Next, test whether main victims should be ghosted at all, for example only victims that had a nonzero frequency or re-admitting ghost hits with some credit. Test the freq-reset-on-promotion change on its own, separate from the threshold. Make one change per variant so web_0's mechanism can finally be pinned down.
