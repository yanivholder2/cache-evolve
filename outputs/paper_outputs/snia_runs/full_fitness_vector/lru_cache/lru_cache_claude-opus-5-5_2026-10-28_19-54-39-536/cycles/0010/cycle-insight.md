# Cycle 0010 Insight

## Cohort outcome

Two variants were latency experiments on the old ghost-free cycle-0004 v002 base, and neither changed policy. v001 swapped SipHash for a fixed-seed folded-multiply hasher. v003 split `prev`/`next` into a parallel `Vec<Link>`. v002, on the cycle-0009 v001 base, changed how reprieves are granted: keys that return to the ghost a second time are escalated into protected. All three are Winners. v002 has the only summary hit-rate gain, almost all of it from hm_0 at 16K. v001 is the cleanest latency win: put and both tails improved with bit-identical hit-rate. v003 won only on the get tail.

## Patterns

- Get p50 did not move for any lever again. Cheaper hashing (v001) and denser links (v003) both left the get median flat or slightly worse, and v002's small get-p50 drop has no code cause. Hash cost and neighbour density are now ruled out as limits on the median. What remains is the hit entry's own memory traffic: the map probe and the node line(s). Splitting the node into two lines made the median worse.
- Get p99 has now been "fixed" by two unrelated changes (a hasher swap and a layout split), just as in cycle 7. The parent's get-p99 outlier is most likely seeding/layout noise. Neither tail gain is established.
- Put cost tracks hash cost, now for the sixth cycle. A cheaper hasher cut put on both hashes of an evicting put. Extra lines touched (v003) or longer escalation paths (v002) raise the put tail.
- Escalation on a second ghost hit works only where the ghost window is wide enough to see a second return. That holds at full width at 16K and fails at capacity/4 at 131K. Taking a tag bit for the repeat flag cost web_0 at both sizes through more collisions. Collisions in the ghost now have a visible price whenever they carry policy state.
- The noise-floor measurement asked for in cycle 9 was not run. Every per-op claim this cycle is still from single runs.

## Open questions

- Does the folded-multiply hasher stack cleanly onto the cycle-0009 v001/cycle-0010 v002 lineage, which already uses a zero-seeded ghost hash?
- Can the repeat signal be kept without giving up a tag bit, for example in a side bit or a separate tiny table, to remove the web_0 loss?
- At 131K, is a second-return signal reachable at all with a capacity/4 ghost, or does it need a wider window for returners only?
- Does a smaller node (not a split one) move get p50?

## Continuation

Make the base cycle-0010 v002 with v001's folded-multiply hasher ported onto it. Recover web_0 by storing the repeat flag without shrinking the ghost tag. Drop the SoA link split and drop hashing as a get-p50 lever. If get p50 is pursued at all, target the hit entry's own line count (node shrinking, probe locality), and only after a paired, repeated noise-floor run exists.
