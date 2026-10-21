# Cycle 0006 Insight

## Cohort outcome

All three variants came from cycle 5's v001 (ghost at an eighth of capacity, `small` at a tenth). Each changed one knob:
- **v001** halved the ghost to a sixteenth of capacity. Hit rate was flat.
- **v002** widened `main`'s frequency cap from 3 to 7. It gained a little.
- **v003** halved `small` to a twentieth of capacity and kept the ghost at an eighth. It won.

All the gains were small.

## Patterns

- **Ghost and `small` act as one admission window.** v001 and v003 point the same way. With `small` at a tenth of capacity, a ghost smaller than `small` adds almost nothing. Once `small` shrinks, a ghost larger than `small` does real work as a cheap second stage of probation. It costs a ghost key, not a resident slot, and the freed slots go to `main`. The question is no longer "how small can the ghost be". It is "how should the combined window be split between resident probation and ghost".
- **Ghost-only tuning is done.** The ghost-size curve has gone flat. Taken alone, v001 suggests the fast path is dispensable. v003 shows that conclusion holds only while `small` stays large.
- **Retention is a separate lever, and it barely moves.** v002 is the first winner-grade result outside admission. Its gain is about as small as the last ghost halving. Telling moderately hot keys apart from very hot keys matters little on this trace. It is orthogonal to v003's change, though, so the two could stack.
- **The evidence base is getting thin.** Every gain this cycle is small, and all of them come from one trace at one cache size. Results at a second size were asked for in cycles 4 and 5 and still haven't come. v003's in-file tests don't exercise the new ratio at their tiny capacities. The fingerprint-ring port that cycle 5 asked for was not attempted. `metadata-size` and `per-op-time` are unmeasured for the sixth cycle running.

## Open questions

- With `small` at a twentieth, what ghost size is best? Is the right rule to keep `small + ghost` roughly constant while moving slots from `small` to the ghost?
- How far can `small` shrink before keys that arrive in `main` with no frequency credit start to hurt?
- Does v002's wider cap stack with v003's split, or do the two overlap?
- Does any of this hold at a second cache size?

## Continuation

Use this cycle's v003 as the parent. Probe the `small`/ghost split together: for example, `small` at a fortieth, or the ghost moved back up while `small` stays at a twentieth. Separately, port v002's `MAX_FREQ = 7` onto v003 to test whether the two stack. Add tests at capacities large enough to exercise the new ratio.

Stop probing ghost size on its own. Before chasing more gains this small, get a second cache size measured. Still pending: porting the fingerprint ring, and measuring `metadata-size` and `per-op-time`.
