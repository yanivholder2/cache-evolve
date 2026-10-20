# Cycle 0010 Insight

## Cohort outcome

Three changes were tried: porting the flat-table and slab storage onto the cycle-0007 lineage (v001), isolating the single-hash sketch derivation by giving the sketch independent per-row hashes (v002), and an adaptive hill-climbing window (v003). v001 and v002 were Winners; v003 was not. The useful result is diagnostic: v002 shows where the cycle-0009 speedup came from.

## Patterns

- The cycle-0009 latency gain came mainly from deriving sketch rows from one shared hash, not from the flat table. Removing that sharing roughly doubled get and put latency, undoing most of the gain.
- The flat table does not transfer to the cycle-0007 lineage. At this cache size the pointer map is cheap, and probing, backward-shift deletion and slab indirection made both paths slower and grew the Node. This repeats the earlier pattern of baseline-only speedups failing to stack.
- Shared-hash reuse has a small hit-rate cost from sketch collision differences, tiny next to the speed it buys.
- The adaptive window added nothing for hit-rate and cost space and latency. Together with the cycle-0008 window result, window sizing on workload-a is a closed plateau.
- Hit-rate was flat or marginally down in every variant again. Winner status for v001 and v002 reflects the selection rule, not real gains.
- v001 also showed a sketch heap larger than its parent's, and v003 showed unexplained Node growth. Both suggest unintended sizing or overlay edits, so some attributions are inferred.

## Open questions

- Does single-hash sketch derivation, applied alone without the flat table, give the cycle-0009 speedup on the stronger hit-rate lineages? Earlier attempts to stack it on strong parents failed.
- Can sketch hashing be made cheaper still, for example by fewer rows or a cheaper mixer, without hit-rate loss?
- Is workload-a at its hit-rate ceiling for this architecture?

## Continuation

Pursue the single-hash sketch derivation alone, with no table swap, on the best hit-rate incumbent, checked with repeat runs, and keep any sketch size unchanged. Look for other cuts to hashing cost in the sketch path. Drop the flat table and slab port, the adaptive window, and any further window-size or Node-layout work. Audit overlay diffs for stray sizing changes before building on them.
