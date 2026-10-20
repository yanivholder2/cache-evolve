# Cycle 0010 Insight

## Cohort outcome

All three variants branched from cycle 0009 v001 and each tested one previously untried axis. v001 added a rotate finalizer to the hash, a representation-only probe-length change. v002 sized the small queue adaptively, ARC-style, from the ghost's source tag. v003 pre-set VISITED on ghost returners. All three are listed as Winners, but none is a real gain. v001 is behaviour-neutral with latency moves inside noise, and v002 and v003 both lost hit-rate.

## Patterns

- Two more levers are closed. Hash quality is ruled out for the map probe: workload-a keys already spread well, so the remaining `get` cost is memory touches, not collisions. Admission strength is now bracketed from both sides: a returner at the hand (0007) and a returner with pre-set credit (this cycle) both lose. Plain tail insertion with VISITED clear looks like a local optimum.
- Main capacity is still the scarce resource. Both policy losses took effective room from main's long-lived keys. v002's adaptation drifted toward a larger small queue, and v003 let one-shot returners crowd out promoted keys that had real hits.
- A shared ghost table carries biased signals. Small-head evictions far outnumber main evictions, so raw per-source counts measure eviction volume, not which queue was too short.
- Another credit asymmetry showed up: promoted small keys enter main with no credit even though they were hit while resident, yet they now rank below ghost returners. Any differentiation should be based on evidence gathered while the key is resident.
- The S3-FIFO lineage has now gone nine cycles without being checked at a second trace or cache size, even though the last three Continuations asked for that check.

## Open questions

- Would ARC-style adaptation help if it were normalized by each ghost source's population, or if the two sources were tracked separately? Or does workload-a just have no phases to adapt to?
- Should promoted small keys keep their earned VISITED bit when they enter main, instead of returners getting extra credit?
- Can a smaller or denser index, such as a fingerprint-indexed or open-addressed table into the arena, cut memory touches per lookup?

## Continuation

Keep cycle 0009 v001 as the parent. v001's rotate is neutral and need not be carried forward. Drop hash-mixing changes, returner credit and raw-count adaptive splits. Next cycle should first put the parent on a second trace or cache size. After that, the remaining candidates are: preserve VISITED on small-to-main promotion, try population-normalized adaptation, and, for latency, try a denser index layout that needs fewer cache lines per probe.
